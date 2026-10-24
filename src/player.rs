//! 플레이어(Naïl). 이동, 발톱 3연타, 굴파기. `Actions`와 `Room`만 본다.
//!
//! 상태 기계: Free ⇄ Attack / Burrow → Emerge → Free. 피격·사망은 어느 상태에서나.
//! 명세 초안: docs/phase-1-combat-prototype.md. 수치: tuning.rs.

use macroquad::prelude::*;

use crate::combat::{self, Shape, Strike, StrikeKind};
use crate::input::Actions;
use crate::room::Room;
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Free,
    Attack { idx: usize, time: f32, dir: Vec2 },
    Burrow { time: f32 },
    Emerge { time: f32 },
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
}

pub struct Player {
    pub pos: Vec2,
    /// 직전 틱 위치(렌더 보간용).
    pub prev_pos: Vec2,
    /// 넉백·돌진 속도. 입력 이동과 별도로 감쇠한다.
    pub vel: Vec2,
    pub facing: Vec2,
    pub hp: i32,
    pub invuln: f32,
    pub flash: f32,
    pub state: State,
    pub burrow_cd: f32,
    pub events: Vec<Event>,
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
            invuln: 0.0,
            flash: 0.0,
            state: State::Free,
            burrow_cd: 0.0,
            events: Vec::new(),
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
        self.invuln > 0.0 || self.is_burrowed() || !self.is_alive()
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
                if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
                    self.start_burrow();
                } else if self.attack_buf > 0.0 && self.no_attack <= 0.0 {
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
                let time = time + dt;
                let (a0, a1) = t::CLAW_ACTIVE[idx];
                if time >= a0 && time - dt < a1 {
                    strike = Some(self.claw_strike(idx, dir));
                }
                self.pos += move_dir * t::PLAYER_SPEED * t::CLAW_MOVE_FACTOR * dt;
                self.state = State::Attack { idx, time, dir };

                if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
                    // 굴파기는 언제든 스윙을 취소한다.
                    self.start_burrow();
                } else if time >= a1 && self.attack_buf > 0.0 && idx < 2 {
                    // 활성 구간이 끝났으면 다음 타로 바로 이어진다.
                    self.start_swing(idx + 1, a.aim);
                } else if time >= t::CLAW_SWING_TIME[idx] + t::CLAW_RECOVERY[idx] {
                    self.state = State::Free;
                    if idx < 2 {
                        self.combo_next = idx + 1;
                        self.combo_timer = t::CLAW_COMBO_WINDOW;
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
        }

        self.attack_buf = (self.attack_buf - dt).max(0.0);
        self.dodge_buf = (self.dodge_buf - dt).max(0.0);

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        room.resolve_circle(&mut self.pos, r);
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
        self.vel += dir * t::CLAW_LUNGE;
        self.attack_buf = 0.0;
        self.combo_timer = 0.0;
        self.state = State::Attack {
            idx,
            time: 0.0,
            dir,
        };
        self.events.push(Event::Swing(idx));
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
        self.burrow_cd = t::BURROW_COOLDOWN;
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
        }
    }

    fn claw_strike(&self, idx: usize, dir: Vec2) -> Strike {
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Claw(idx),
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: t::CLAW_RANGE,
            },
            damage: t::CLAW_DAMAGE[idx],
            knockback: t::CLAW_KNOCKBACK[idx],
            stun: t::CLAW_STUN[idx],
            hitstop: t::CLAW_HITSTOP[idx],
            shake: t::CLAW_SHAKE[idx],
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
