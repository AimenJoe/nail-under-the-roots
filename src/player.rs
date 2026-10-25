//! 플레이어(Naïl). 이동, 발톱 3연타, 굴파기, 필살기 드릴. `Actions`와 `Room`만 본다.
//!
//! 상태 기계: Free ⇄ Attack / Burrow → Emerge → Free, Free·Attack → Drill → Recover → Free, Free → Eat → Free. 피격·사망은 어느 상태에서나.
//! 모든 행동은 바로 다시 나가지 않는다(docs/decisions/0038): 발톱 = 스윙·후딜레이, 굴파기 = 대기, 필살기 = 게이지, 먹기 = 먹는 시간 + 대기, 패링 창 = 대기.
//! 수치는 `stats`(장비·장신구가 정한 것, equipment::Stats)를 거친다.
//! 명세 초안: docs/phase-1-combat-prototype.md. 수치: tuning.rs.

use macroquad::prelude::*;

use crate::combat::{self, Shape, Strike, StrikeKind};
use crate::equipment::Stats;
use crate::input::Actions;
use crate::items::Consumable;
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
    /// 필살기: 시동(`time < DRILL_REV`, 제자리) 뒤 직진하며 연타. `dir`은 조금씩만 틀어진다. `next_hit`까지 남은 시간.
    Drill {
        time: f32,
        dir: Vec2,
        next_hit: f32,
    },
    /// 드릴 뒤 경직.
    Recover {
        time: f32,
    },
    /// 퀵 아이템을 먹는 중. 다 먹으면 `Event::Ate`(가방에서 꺼내 회복하는 건 월드). 맞으면 끊긴다.
    Eat {
        item: Consumable,
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
    /// 필살기 시동(소리 시작).
    DrillRev,
    /// 시동이 끝나고 튀어 나감.
    Drill,
    DrillWall,
    Parry,
    /// 다 먹었다(월드가 가방에서 꺼내 회복).
    Ate(Consumable),
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
    /// 다음 먹기까지 남은 시간(먹기 시작 순간부터 ITEM_COOLDOWN).
    pub item_cd: f32,
    /// 먹기 대기 배율(실뜨기 5, docs/decisions/0043).
    pub eat_cd_mul: f32,
    /// 실뜨기 8·9(0053): 패링 창 배율, 굴파기 대기 배율.
    pub parry_mul: f32,
    pub burrow_knot: f32,
    /// 큰 나무의 재주(docs/decisions/0047). 앱이 기록에서 정한다.
    pub talents: crate::talents::Talents,
    /// 받는 피해를 줄이는 비율(0..1). 도움 모드 "포포의 응원"(docs/decisions/0041). 앱이 기록에서 정한다.
    pub resist: f32,
    /// 줄인 피해의 소수 부분을 모아 둔다(체력은 정수라 1이 차면 그때 깎는다).
    dmg_acc: f32,
    pub events: Vec<Event>,
    /// 콤보 표. 무기·캐릭터에 따라 바뀐다(moveset.rs).
    pub moveset: &'static Moveset,
    /// 스프라이트 좌우(마지막으로 본 가로 방향).
    pub face_left: bool,
    /// 무기를 들었나. 첫 시작에는 맨손이라 공격이 안 나간다(docs/decisions/0012).
    pub armed: bool,
    attack_buf: f32,
    dodge_buf: f32,
    item_buf: f32,
    /// 다음 패링 창이 열릴 수 있기까지. 지금 스윙에 창이 있나는 `parry_open`.
    parry_cd: f32,
    parry_open: bool,
    combo_next: usize,
    combo_timer: f32,
    no_attack: f32,
    swing_id: u32,
    /// 지금 스윙을 시작하고 흐른 실제 시간(패링 창).
    swing_age: f32,
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
            item_cd: 0.0,
            eat_cd_mul: 1.0,
            parry_mul: 1.0,
            burrow_knot: 1.0,
            talents: crate::talents::Talents::default(),
            resist: 0.0,
            dmg_acc: 0.0,
            events: Vec::new(),
            moveset: &moveset::CLAWS,
            face_left: false,
            armed: true,
            attack_buf: 0.0,
            dodge_buf: 0.0,
            item_buf: 0.0,
            parry_cd: 0.0,
            parry_open: false,
            combo_next: 0,
            combo_timer: 0.0,
            no_attack: 0.0,
            swing_id: 0,
            swing_age: f32::MAX,
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

    /// 굴파기 대기 한 번의 길이(장신구 × 재주 × 실뜨기). 상태 탭·HUD 막대도 이 값을 쓴다.
    pub fn burrow_cd_full(&self) -> f32 {
        t::BURROW_COOLDOWN * self.stats.burrow_cd * self.talents.burrow_cd_mul() * self.burrow_knot
    }

    /// 패링 창이 열려 있으면 공격 방향. 스윙 시작 직후 PARRY_WINDOW 동안(실뜨기 8단계면 짧다).
    pub fn parry_dir(&self) -> Option<Vec2> {
        match self.state {
            State::Attack { dir, .. }
                if self.parry_open && self.swing_age < t::PARRY_WINDOW * self.parry_mul =>
            {
                Some(dir)
            }
            _ => None,
        }
    }

    /// 패링 성공: SP를 크게 채운다. 다음 창은 바로 열 수 있다.
    pub fn on_parry(&mut self) {
        self.sp = (self.sp + self.talents.parry_sp()).min(t::SP_MAX);
        self.parry_cd = 0.0;
        self.events.push(Event::Parry);
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
        self.sp = (self.sp + g * self.stats.sp_gain * self.talents.sp_gain_mul()).min(t::SP_MAX);
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
        // 지렁이를 먹고 일어났다 = 먹기. 바로 또 먹지 못한다.
        self.item_cd = t::ITEM_COOLDOWN;
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
        if a.use_item {
            self.item_buf = t::INPUT_BUFFER;
        }
    }

    /// 먹기를 눌렀고(버퍼) 지금 먹기 시작할 수 있나. 무엇을 먹을지·먹을 필요가 있는지는 월드가 본다.
    pub fn wants_to_eat(&self) -> bool {
        self.item_buf > 0.0 && self.item_cd <= 0.0 && self.state == State::Free
    }

    /// 먹기 시작. 대기는 지금부터 센다(끊겨도 그대로).
    pub fn start_eat(&mut self, item: Consumable) {
        self.item_buf = 0.0;
        self.item_cd = t::ITEM_COOLDOWN * self.eat_cd_mul;
        self.state = State::Eat { item, time: 0.0 };
    }

    /// 먹는 중이면 얼마나 먹었나(0..1).
    pub fn eat_progress(&self) -> Option<(Consumable, f32)> {
        match self.state {
            State::Eat { item, time } => Some((
                item,
                (time / (item.eat_time() * self.talents.eat_time_mul())).min(1.0),
            )),
            _ => None,
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
        self.item_cd = (self.item_cd - dt).max(0.0);
        self.parry_cd = (self.parry_cd - dt).max(0.0);
        self.no_attack = (self.no_attack - dt).max(0.0);
        self.swing_age += dt;

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
                if time >= self.talents.burrow_time() || early {
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
                let mut dir = dir;
                let mut next_hit = next_hit;
                if time < t::DRILL_REV {
                    // 시동: 제자리. 방향은 입력대로 바로.
                    let aim = if a.aim != Vec2::ZERO { a.aim } else { move_dir };
                    if aim != Vec2::ZERO {
                        dir = aim.normalize();
                    }
                } else {
                    if time - dt < t::DRILL_REV {
                        self.events.push(Event::Drill);
                    }
                    // 직진성: 입력 쪽으로 초당 DRILL_TURN 라디안만 튼다.
                    if move_dir != Vec2::ZERO {
                        let want = move_dir.normalize();
                        let ang = dir.angle_between(want);
                        let step = ang.clamp(-t::DRILL_TURN * dt, t::DRILL_TURN * dt);
                        dir = Vec2::from_angle(step).rotate(dir).normalize();
                    }
                    self.pos += dir * drill_speed(time - t::DRILL_REV) * dt;
                    next_hit -= dt;
                }
                self.facing = dir;
                if time >= t::DRILL_REV && next_hit <= 0.0 {
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
                        shake: t::DRILL_HIT_SHAKE,
                        finisher: false,
                        poison: 0.0,
                    });
                }
                self.state = if time >= t::DRILL_REV + t::DRILL_TIME * self.talents.drill_mul() {
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
            State::Eat { item, time } => {
                let time = time + dt;
                self.pos += move_dir * t::PLAYER_SPEED * t::EAT_MOVE_FACTOR * dt;
                if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
                    // 굴파기로 끊는다: 먹지 않은 것으로. 대기는 이미 흐르고 있다.
                    self.start_burrow();
                } else if time >= item.eat_time() * self.talents.eat_time_mul() {
                    self.events.push(Event::Ate(item));
                    self.state = State::Free;
                } else {
                    self.state = State::Eat { item, time };
                }
            }
        }

        self.attack_buf = (self.attack_buf - dt).max(0.0);
        self.dodge_buf = (self.dodge_buf - dt).max(0.0);
        self.item_buf = (self.item_buf - dt).max(0.0);

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        let bumped = room.resolve_circle(&mut self.pos, r);
        // 드릴이 벽(기둥)에 부딪히면 쿵: 충격파 + 경직. 시작하자마자 벽에 붙어 있던 건 치지 않는다.
        if let State::Drill { time, dir, .. } = self.state {
            if bumped && time > t::DRILL_REV + 0.06 {
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
        self.swing_age = 0.0;
        // 패링 창은 대기가 끝났을 때만 열린다. 연타하면 몇 번에 한 번만(docs/decisions/0038).
        self.parry_open = self.parry_cd <= 0.0;
        if self.parry_open {
            self.parry_cd = t::PARRY_COOLDOWN;
        }
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
        self.events.push(Event::DrillRev);
    }

    /// 필살기 시동 중(제자리에서 힘 모으기). 월드는 이 동안 적을 멈춘다.
    pub fn drill_revving(&self) -> bool {
        matches!(self.state, State::Drill { time, .. } if time < t::DRILL_REV)
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
        self.burrow_cd = self.burrow_cd_full();
        self.no_attack = t::EMERGE_NO_ATTACK;
        self.swing_id += 1;
        self.state = State::Emerge { time: 0.0 };
        self.events.push(Event::Emerge);
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Emerge,
            shape: Shape::Circle {
                center: self.pos,
                r: self.talents.emerge_radius(),
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
        // 도움 모드: 피해를 비율만큼 줄인다. 소수는 모아 두었다가 1이 되면 깎는다(맞은 반응은 그대로).
        let dmg = if self.resist > 0.0 {
            self.dmg_acc += dmg as f32 * (1.0 - self.resist.min(0.95));
            let d = self.dmg_acc.floor();
            self.dmg_acc -= d;
            d as i32
        } else {
            dmg
        };
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
            // 공격·솟구침·먹기 중이었으면 끊긴다(먹던 것은 먹지 않은 것으로).
            self.state = State::Free;
            self.combo_timer = 0.0;
            self.events.push(Event::Hurt);
        }
        true
    }
}

/// 돌진 속도: 시동 뒤 `dash_t`초. 처음엔 붙고(DRILL_ACCEL), 그 뒤 최고 속도.
pub fn drill_speed(dash_t: f32) -> f32 {
    let k = (dash_t / t::DRILL_ACCEL).clamp(0.0, 1.0);
    t::DRILL_SPEED_START + (t::DRILL_SPEED - t::DRILL_SPEED_START) * k
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
        let mut p = Player::new(vec2(room.inner.right() - 120.0, room.center().y));
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
        let x0 = p.pos.x;
        // 시동 동안은 제자리.
        for _ in 0..((t::DRILL_REV / t::TICK_DT) as usize - 1) {
            tick(&mut p, &room, &hold);
            assert!(p.drill_revving());
        }
        assert!((p.pos.x - x0).abs() < 0.01, "시동 중엔 움직이지 않는다");
        for _ in 0..120 {
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
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        while p.drill_revving() {
            tick(&mut p, &room, &hold);
        }
        let back = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        let before = match p.state {
            State::Drill { dir, .. } => dir,
            _ => panic!("드릴 중이어야 한다"),
        };
        assert!(before.angle_between(vec2(1.0, 0.0)).abs() < 1e-3);
        tick(&mut p, &room, &back);
        if let State::Drill { dir, .. } = p.state {
            let turned = vec2(1.0, 0.0).angle_between(dir).abs();
            assert!(turned > 0.0 && turned <= t::DRILL_TURN * t::TICK_DT + 1e-4);
        } else {
            panic!("드릴 중이어야 한다");
        }
    }

    /// 돌진 거리는 화면 한 폭 남짓(320px ± 10%).
    #[test]
    fn drill_travels_about_a_screen() {
        let n = (t::DRILL_TIME / t::TICK_DT).round() as usize;
        let d: f32 = (0..n)
            .map(|i| drill_speed(i as f32 * t::TICK_DT) * t::TICK_DT)
            .sum();
        assert!((290.0..=355.0).contains(&d), "{d}");
    }

    /// 패링 창: 스윙 시작 직후 PARRY_WINDOW 동안만, 공격 방향으로.
    /// 도움 모드: 피해가 비율만큼 줄고, 소수는 모였다가 깎인다.
    #[test]
    fn assist_resist_spreads_damage_over_hits() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.resist = 0.5;
        let start = p.hp;
        for _ in 0..4 {
            p.invuln = 0.0;
            assert!(
                p.take_damage(p.pos + vec2(10.0, 0.0), 1),
                "맞은 반응(넉백·무적)은 그대로"
            );
        }
        assert_eq!(start - p.hp, 2, "1 피해 네 번 × 0.5 = 2");
    }

    /// 연타로 패링 창을 이어 붙이지 못한다: 창은 PARRY_COOLDOWN마다 한 번. 성공하면 바로 다시(0038).
    #[test]
    fn mashing_attack_does_not_chain_parry_windows() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        let atk = Actions {
            attack: true,
            ..Default::default()
        };
        // 1초 동안 매 틱 공격을 누른다: 창이 열려 있던 시간의 비율.
        let ticks = (1.0 / t::TICK_DT) as usize;
        let mut open = 0;
        for _ in 0..ticks {
            tick(&mut p, &room, &atk);
            if p.parry_dir().is_some() {
                open += 1;
            }
        }
        let frac = open as f32 / ticks as f32;
        let most = t::PARRY_WINDOW / t::PARRY_COOLDOWN + 0.05;
        assert!(frac <= most, "연타 창 비율 {frac} > {most}");
        assert!(open > 0, "연타해도 가끔은 걸린다");

        // 성공하면 다음 스윙에 바로 창이 열린다.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &atk);
        assert!(p.parry_dir().is_some());
        p.on_parry();
        for _ in 0..((0.13 / t::TICK_DT) as usize) {
            tick(&mut p, &room, &Actions::default());
        }
        tick(&mut p, &room, &atk);
        assert!(p.parry_dir().is_some(), "튕겨낸 뒤엔 대기 없이");
    }

    /// 먹기: 먹는 시간이 지나야 Ate, 맞으면 끊기고, 대기 동안은 다시 못 먹는다(0038).
    #[test]
    fn eating_takes_time_breaks_on_hit_and_has_a_cooldown() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        let eat = Actions {
            use_item: true,
            ..Default::default()
        };
        p.buffer(&eat);
        assert!(p.wants_to_eat());
        p.start_eat(Consumable::Berry);
        let need = (Consumable::Berry.eat_time() / t::TICK_DT).ceil() as usize;
        for _ in 0..need - 1 {
            let (_, ev) = tick(&mut p, &room, &Actions::default());
            assert!(
                !ev.contains(&Event::Ate(Consumable::Berry)),
                "먹는 중엔 아직"
            );
        }
        let (_, ev) = tick(&mut p, &room, &Actions::default());
        assert!(ev.contains(&Event::Ate(Consumable::Berry)));
        assert_eq!(p.state, State::Free);
        // 다 먹었어도 대기가 남아 있다.
        p.buffer(&eat);
        assert!(!p.wants_to_eat(), "같은 행동이 바로 다시 나가면 안 된다");
        let wait = (t::ITEM_COOLDOWN / t::TICK_DT).ceil() as usize;
        for _ in 0..wait {
            tick(&mut p, &room, &Actions::default());
        }
        p.buffer(&eat);
        assert!(p.wants_to_eat());

        // 먹다가 맞으면 끊긴다(Ate 없음).
        p.start_eat(Consumable::Worm);
        tick(&mut p, &room, &Actions::default());
        assert!(p.take_damage(p.pos + vec2(10.0, 0.0), 1));
        assert_eq!(p.state, State::Free);
        for _ in 0..((t::WORM_EAT_TIME / t::TICK_DT) as usize + 2) {
            let (_, ev) = tick(&mut p, &room, &Actions::default());
            assert!(!ev.contains(&Event::Ate(Consumable::Worm)));
        }

        // 굴파기로 끊을 수 있다.
        let mut p = Player::new(room.center());
        p.start_eat(Consumable::Berry);
        let dig = Actions {
            dodge: true,
            ..Default::default()
        };
        tick(&mut p, &room, &dig);
        assert!(p.is_burrowed());
    }

    #[test]
    fn parry_window_opens_only_at_swing_start() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        assert!(p.parry_dir().is_none());
        let atk = Actions {
            attack: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        tick(&mut p, &room, &atk);
        assert_eq!(p.parry_dir(), Some(vec2(1.0, 0.0)));
        let ticks = (t::PARRY_WINDOW / t::TICK_DT).ceil() as usize + 1;
        for _ in 0..ticks {
            tick(&mut p, &room, &Actions::default());
        }
        assert!(p.parry_dir().is_none(), "창이 닫혀야 한다");
        let sp = p.sp;
        p.on_parry();
        assert!(p.sp > sp);
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
