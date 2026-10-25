//! 플레이어(Naïl). 이동, 발톱 3연타, 굴파기, 필살기 드릴. `Actions`와 `Room`만 본다.
//!
//! 상태 기계: Free ⇄ Attack / Burrow → Emerge → Free, Free·Attack → Drill → Recover → Free. 피격·사망은 어느 상태에서나.
//! 수치는 `stats`(장비·장신구가 정한 것, equipment::Stats)를 거친다.
//! 명세 초안: docs/phase-1-combat-prototype.md. 수치: tuning.rs.

use macroquad::prelude::*;

use crate::combat::{self, Shape, Strike, StrikeKind};
use crate::equipment::Stats;
use crate::input::Actions;
use crate::moveset::{self, Moveset};
use crate::room::Room;
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Free,
    Attack {
        idx: usize,
        time: f32,
        dir: Vec2,
    },
    Burrow {
        time: f32,
    },
    Emerge {
        time: f32,
    },
    /// 필살기: 직진하며 연타. `dir`은 조금씩만 틀어진다. `next_hit`까지 남은 시간.
    Drill {
        time: f32,
        dir: Vec2,
        next_hit: f32,
    },
    /// 드릴 뒤 경직.
    Recover {
        time: f32,
    },
    Dead,
}

/// 이번 틱에 일어난 일(사운드·이펙트용). 로직은 이걸 다시 읽지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Swing(usize),
    Burrow,
    Emerge,
    Hurt,
    Die,
    Drill,
    DrillWall,
}

pub struct Player {
    pub pos: Vec2,
    /// 직전 틱 위치(렌더 보간용).
    pub prev_pos: Vec2,
    /// 넉백·돌진 속도. 입력 이동과 별도로 감쇠한다.
    pub vel: Vec2,
    pub facing: Vec2,
    pub hp: i32,
    /// 장비가 정한 수치(앱이 장착을 바꾸면 갱신).
    pub stats: Stats,
    /// 필살기 게이지(0..=SP_MAX). 발톱이 맞으면 찬다.
    pub sp: f32,
    pub invuln: f32,
    pub flash: f32,
    pub state: State,
    pub burrow_cd: f32,
    pub events: Vec<Event>,
    /// 콤보 표. 무기·캐릭터에 따라 바뀐다(moveset.rs).
    pub moveset: &'static Moveset,
    /// 스프라이트 좌우(마지막으로 본 가로 방향).
    pub face_left: bool,
    /// 무기를 들었나. 첫 시작에는 맨손이라 공격이 안 나간다(docs/decisions/0012).
    pub armed: bool,
    attack_buf: f32,
    dodge_buf: f32,
    combo_next: usize,
    combo_timer: f32,
    no_attack: f32,
    swing_id: u32,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
            facing: vec2(1.0, 0.0),
            hp: t::PLAYER_HP,
            stats: Stats::BASE,
            sp: 0.0,
            invuln: 0.0,
            flash: 0.0,
            state: State::Free,
            burrow_cd: 0.0,
            events: Vec::new(),
            moveset: &moveset::CLAWS,
            face_left: false,
            armed: true,
            attack_buf: 0.0,
            dodge_buf: 0.0,
            combo_next: 0,
            combo_timer: 0.0,
            no_attack: 0.0,
            swing_id: 0,
        }
    }

    pub fn radius(&self) -> f32 {
        t::PLAYER_RADIUS
    }

    pub fn is_alive(&self) -> bool {
        self.state != State::Dead
    }

    pub fn is_burrowed(&self) -> bool {
        matches!(self.state, State::Burrow { .. })
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln > 0.0
            || self.is_burrowed()
            || matches!(self.state, State::Drill { .. })
            || !self.is_alive()
    }

    pub fn sp_full(&self) -> bool {
        self.sp >= t::SP_MAX
    }

    /// 발톱이 적을 맞혔다: SP를 채운다(드릴 자체는 채우지 않는다).
    pub fn gain_sp(&mut self, finisher: bool) {
        let mut g = t::SP_PER_HIT;
        if finisher {
            g += t::SP_FINISHER_BONUS;
        }
        self.sp = (self.sp + g * self.stats.sp_gain).min(t::SP_MAX);
    }

    /// 장비가 바뀌었다. 최대 체력이 줄면 체력도 따라 줄인다.
    pub fn set_stats(&mut self, s: Stats) {
        self.stats = s;
        self.hp = self.hp.min(s.max_hp);
    }

    pub fn heal(&mut self, n: i32) {
        self.hp = (self.hp + n).min(self.stats.max_hp);
    }

    /// 쓰러진 자리에서 일어난다(쥬시 지렁이).
    pub fn revive(&mut self) {
        let hp = (self.stats.max_hp as f32 * t::REVIVE_HP_FRAC).ceil() as i32;
        self.hp = hp.clamp(1, self.stats.max_hp);
        self.state = State::Free;
        self.invuln = t::REVIVE_INVULN;
        self.vel = Vec2::ZERO;
        self.combo_timer = 0.0;
    }

    /// 히트스톱 중에도 호출된다: 눌림을 버퍼에 넣기만 한다.
    pub fn buffer(&mut self, a: &Actions) {
        if a.attack {
            self.attack_buf = t::INPUT_BUFFER;
        }
        if a.dodge {
            self.dodge_buf = t::INPUT_BUFFER;
        }
    }

    /// 한 로직 틱. 이번 틱에 활성인 공격 히트박스를 돌려준다.
    pub fn update(&mut self, a: &Actions, room: &Room, dt: f32) -> Option<Strike> {
        self.events.clear();
        self.prev_pos = self.pos;
        self.buffer(a);

        self.invuln = (self.invuln - dt).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        self.combo_timer = (self.combo_timer - dt).max(0.0);
        self.burrow_cd = (self.burrow_cd - dt).max(0.0);
        self.no_attack = (self.no_attack - dt).max(0.0);

        let move_dir = a.move_dir;
        if move_dir != Vec2::ZERO && !matches!(self.state, State::Attack { .. }) {
            self.facing = move_dir.normalize();
        }

        let mut strike = None;
        match self.state {
            State::Dead => {}
            State::Free => {
                if a.special && self.armed && self.sp_full() {
                    self.start_drill(move_dir);
                } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
                    self.start_burrow();
                } else if self.armed && self.attack_buf > 0.0 && self.no_attack <= 0.0 {
                    let idx = if self.combo_timer > 0.0 {
                        self.combo_next
                    } else {
                        0
                    };
                    self.start_swing(idx, a.aim);
                } else {
                    self.pos += move_dir * t::PLAYER_SPEED * dt;
                }
            }
            State::Attack { idx, time, dir } => {
                let ms = self.moveset;
                let hit = ms.hits[idx];
                let last = ms.last();
                // 공격 속도: 스윙 시계를 빨리(느리게) 돌린다. 활성·캔슬·후딜레이가 함께 줄어든다.
                let sdt = dt * self.stats.attack_speed;
                let time = time + sdt;
                let (a0, a1) = hit.active;
                if time >= a0 && time - sdt < a1 {
                    strike = Some(self.claw_strike(idx, dir));
                }
                self.pos += move_dir * t::PLAYER_SPEED * ms.move_factor * dt;
                self.state = State::Attack { idx, time, dir };

                if a.special && self.sp_full() && time >= a1 {
                    // 필살기도 휘두른 뒤(활성 구간 뒤)라면 캔슬로 낼 수 있다.
                    self.start_drill(move_dir);
                } else if self.dodge_buf > 0.0
                    && self.burrow_cd <= 0.0
                    && time >= hit.dodge_cancel_after
                {
                    // 굴파기 캔슬: 후딜레이를 끊고 다음 콤보로 섞는다.
                    self.start_burrow();
                } else if time >= a1 && self.attack_buf > 0.0 && idx < last {
                    // 활성 구간이 끝났으면 다음 타로 바로 이어진다.
                    self.start_swing(idx + 1, a.aim);
                } else if time >= hit.swing + hit.recovery {
                    self.state = State::Free;
                    if idx < last {
                        self.combo_next = idx + 1;
                        self.combo_timer = ms.combo_window;
                    } else {
                        self.combo_timer = 0.0;
                    }
                }
            }
            State::Burrow { time } => {
                let time = time + dt;
                self.pos += move_dir * t::PLAYER_SPEED * t::BURROW_SPEED_FACTOR * dt;
                self.state = State::Burrow { time };
                let early = (self.dodge_buf > 0.0 && time > 0.05) || self.attack_buf > 0.0;
                if time >= t::BURROW_MAX_TIME || early {
                    strike = Some(self.start_emerge());
                }
            }
            State::Emerge { time } => {
                let time = time + dt;
                self.state = if time >= t::EMERGE_TIME {
                    State::Free
                } else {
                    State::Emerge { time }
                };
            }
            State::Drill {
                time,
                dir,
                next_hit,
            } => {
                let time = time + dt;
                // 직진성: 입력 쪽으로 초당 DRILL_TURN 라디안만 튼다.
                let mut dir = dir;
                if move_dir != Vec2::ZERO {
                    let want = move_dir.normalize();
                    let ang = dir.angle_between(want);
                    let step = ang.clamp(-t::DRILL_TURN * dt, t::DRILL_TURN * dt);
                    dir = Vec2::from_angle(step).rotate(dir).normalize();
                }
                self.facing = dir;
                self.pos += dir * t::DRILL_SPEED * dt;
                let mut next_hit = next_hit - dt;
                if next_hit <= 0.0 {
                    next_hit += t::DRILL_HIT_EVERY;
                    self.swing_id += 1;
                    strike = Some(Strike {
                        id: self.swing_id,
                        kind: StrikeKind::Drill,
                        shape: Shape::Circle {
                            center: self.pos + dir * 6.0,
                            r: t::DRILL_RADIUS,
                        },
                        damage: t::DRILL_DAMAGE + self.stats.damage,
                        knockback: t::DRILL_KNOCKBACK,
                        stun: t::DRILL_STUN,
                        hitstop: t::DRILL_HITSTOP,
                        shake: 0.1,
                        finisher: false,
                        poison: 0.0,
                    });
                }
                self.state = if time >= t::DRILL_TIME {
                    State::Recover {
                        time: t::DRILL_RECOVER,
                    }
                } else {
                    State::Drill {
                        time,
                        dir,
                        next_hit,
                    }
                };
            }
            State::Recover { time } => {
                let time = time - dt;
                self.state = if time <= 0.0 {
                    State::Free
                } else {
                    State::Recover { time }
                };
            }
        }

        self.attack_buf = (self.attack_buf - dt).max(0.0);
        self.dodge_buf = (self.dodge_buf - dt).max(0.0);

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        let bumped = room.resolve_circle(&mut self.pos, r);
        // 드릴이 벽(기둥)에 부딪히면 쿵: 충격파 + 경직. 시작하자마자 벽에 붙어 있던 건 치지 않는다.
        if let State::Drill { time, dir, .. } = self.state {
            if bumped && time > 0.06 {
                self.swing_id += 1;
                self.vel = -dir * 80.0;
                self.state = State::Recover {
                    time: t::DRILL_WALL_RECOVER,
                };
                self.events.push(Event::DrillWall);
                strike = Some(Strike {
                    id: self.swing_id,
                    kind: StrikeKind::DrillWall,
                    shape: Shape::Circle {
                        center: self.pos + dir * 4.0,
                        r: t::DRILL_WALL_RADIUS,
                    },
                    damage: t::DRILL_WALL_DAMAGE + self.stats.damage,
                    knockback: t::DRILL_WALL_KNOCKBACK,
                    stun: t::DRILL_WALL_STUN,
                    hitstop: t::DRILL_WALL_HITSTOP,
                    shake: t::DRILL_WALL_SHAKE,
                    finisher: false,
                    poison: 0.0,
                });
            }
        }
        if self.facing.x.abs() > 0.2 {
            self.face_left = self.facing.x < 0.0;
        }
        strike
    }

    fn start_swing(&mut self, idx: usize, aim: Vec2) {
        let dir = if aim != Vec2::ZERO {
            aim.normalize()
        } else {
            self.facing
        };
        self.facing = dir;
        self.swing_id += 1;
        self.vel += dir * self.moveset.hits[idx].lunge;
        self.attack_buf = 0.0;
        self.combo_timer = 0.0;
        self.state = State::Attack {
            idx,
            time: 0.0,
            dir,
        };
        self.events.push(Event::Swing(idx));
    }

    fn start_drill(&mut self, move_dir: Vec2) {
        let dir = if move_dir != Vec2::ZERO {
            move_dir.normalize()
        } else {
            self.facing
        };
        self.sp = 0.0;
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.state = State::Drill {
            time: 0.0,
            dir,
            next_hit: 0.0,
        };
        self.events.push(Event::Drill);
    }

    fn start_burrow(&mut self) {
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.vel = Vec2::ZERO;
        self.state = State::Burrow { time: 0.0 };
        self.events.push(Event::Burrow);
    }

    fn start_emerge(&mut self) -> Strike {
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.burrow_cd = t::BURROW_COOLDOWN * self.stats.burrow_cd;
        self.no_attack = t::EMERGE_NO_ATTACK;
        self.swing_id += 1;
        self.state = State::Emerge { time: 0.0 };
        self.events.push(Event::Emerge);
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Emerge,
            shape: Shape::Circle {
                center: self.pos,
                r: t::EMERGE_RADIUS,
            },
            damage: t::EMERGE_DAMAGE,
            knockback: t::EMERGE_KNOCKBACK,
            stun: t::EMERGE_STUN,
            hitstop: t::EMERGE_HITSTOP,
            shake: t::EMERGE_SHAKE,
            finisher: false,
            poison: 0.0,
        }
    }

    fn claw_strike(&self, idx: usize, dir: Vec2) -> Strike {
        let h = self.moveset.hits[idx];
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Claw(idx),
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: h.range,
            },
            damage: h.damage + self.stats.damage,
            knockback: h.knockback,
            stun: h.stun,
            hitstop: h.hitstop,
            shake: h.shake,
            finisher: idx == self.moveset.last(),
            poison: h.poison,
        }
    }

    /// 피해를 입힌다. 무적이면 false.
    pub fn take_damage(&mut self, from: Vec2, dmg: i32) -> bool {
        if self.is_invulnerable() {
            return false;
        }
        self.hp -= dmg;
        self.invuln = t::PLAYER_HURT_INVULN;
        self.flash = t::HIT_FLASH;
        let away = (self.pos - from).normalize_or_zero();
        let away = if away == Vec2::ZERO {
            -self.facing
        } else {
            away
        };
        self.vel = away * t::PLAYER_HURT_KNOCKBACK;
        if self.hp <= 0 {
            self.hp = 0;
            self.state = State::Dead;
            self.events.push(Event::Die);
        } else {
            // 공격·솟구침 중이었으면 끊긴다.
            self.state = State::Free;
            self.combo_timer = 0.0;
            self.events.push(Event::Hurt);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(p: &mut Player, room: &Room, a: &Actions) -> (Option<Strike>, Vec<Event>) {
        let s = p.update(a, room, t::TICK_DT);
        (s, p.events.clone())
    }

    /// 필살기: SP가 차야 나가고, 벽에 부딪히면 충격파(DrillWall) + 경직.
    #[test]
    fn drill_needs_full_sp_and_bonks_walls() {
        let room = Room::new();
        let mut p = Player::new(vec2(room.inner.right() - 40.0, room.center().y));
        let go = Actions {
            move_dir: vec2(1.0, 0.0),
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        assert!(
            !matches!(p.state, State::Drill { .. }),
            "SP가 없으면 안 나간다"
        );
        p.sp = t::SP_MAX;
        tick(&mut p, &room, &go);
        assert!(matches!(p.state, State::Drill { .. }));
        assert_eq!(p.sp, 0.0);
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        let mut bonk = false;
        let mut hits = 0;
        for _ in 0..60 {
            let (s, ev) = tick(&mut p, &room, &hold);
            if let Some(s) = s {
                match s.kind {
                    StrikeKind::Drill => hits += 1,
                    StrikeKind::DrillWall => {
                        assert!(s.damage >= t::DRILL_WALL_DAMAGE);
                        bonk = true;
                    }
                    _ => {}
                }
            }
            if ev.contains(&Event::DrillWall) {
                assert!(matches!(p.state, State::Recover { .. }));
            }
        }
        assert!(bonk, "벽 쿵이 나야 한다");
        assert!(hits >= 3, "벽까지 가는 동안 연타");
    }

    /// 직진성: 반대쪽을 눌러도 한 틱에 조금만 돈다.
    #[test]
    fn drill_steers_only_a_little() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.sp = t::SP_MAX;
        let go = Actions {
            move_dir: vec2(1.0, 0.0),
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        let back = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        tick(&mut p, &room, &back);
        if let State::Drill { dir, .. } = p.state {
            let turned = vec2(1.0, 0.0).angle_between(dir).abs();
            assert!(turned > 0.0 && turned <= t::DRILL_TURN * t::TICK_DT + 1e-4);
        } else {
            panic!("드릴 중이어야 한다");
        }
    }

    #[test]
    fn attack_speed_shortens_the_swing() {
        let room = Room::new();
        let swing_ticks = |speed: f32| {
            let mut p = Player::new(room.center());
            p.stats.attack_speed = speed;
            let atk = Actions {
                attack: true,
                ..Default::default()
            };
            tick(&mut p, &room, &atk);
            let mut n = 0;
            while matches!(p.state, State::Attack { .. }) && n < 200 {
                tick(&mut p, &room, &Actions::default());
                n += 1;
            }
            n
        };
        assert!(swing_ticks(1.5) < swing_ticks(1.0));
    }
}
