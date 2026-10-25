//! 적 2종: 돌격형(Charger), 원거리형(Shooter). 플레이어 위치와 `Room`만 본다.

use macroquad::prelude::*;

use crate::combat;
use crate::room::Room;
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Charger,
    Shooter,
    /// 튜토리얼 허수아비. 움직이지도 때리지도 않는다.
    Dummy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// 돌격형: 플레이어 쪽으로 걷기.
    Approach,
    /// 돌격형: 돌진 예고(멈춰서 떨림).
    Windup { time: f32 },
    /// 돌격형: 돌진. 방향은 시작 시 고정.
    Charge { time: f32, dir: Vec2 },
    /// 돌격형: 돌진 뒤 멈춤(남은 시간).
    Rest { time: f32 },
    /// 원거리형: 거리 유지.
    Keep,
    /// 원거리형: 조준 예고.
    Aim { time: f32 },
    /// 공통: 경직(남은 시간).
    Stunned { time: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Charge,
    WallHit,
    Shoot,
}

pub struct Enemy {
    pub kind: Kind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub hp: i32,
    pub state: State,
    pub facing: Vec2,
    pub flash: f32,
    pub alive: bool,
    /// 마지막으로 맞은 스윙 번호. 같은 스윙에 두 번 맞지 않는다.
    pub last_hit_by: u32,
    fire_cd: f32,
}

pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub alive: bool,
}

impl Projectile {
    pub fn radius(&self) -> f32 {
        t::PROJECTILE_RADIUS
    }

    pub fn update(&mut self, room: &Room, dt: f32) {
        self.prev_pos = self.pos;
        self.pos += self.vel * dt;
        if room.is_solid(self.pos, self.radius()) {
            self.alive = false;
        }
    }
}

impl Enemy {
    pub fn new(kind: Kind, pos: Vec2) -> Self {
        let (hp, state) = match kind {
            Kind::Charger => (t::CHARGER_HP, State::Approach),
            Kind::Shooter => (t::SHOOTER_HP, State::Keep),
            Kind::Dummy => (t::DUMMY_HP, State::Rest { time: f32::MAX }),
        };
        Self {
            kind,
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
            hp,
            state,
            facing: vec2(-1.0, 0.0),
            flash: 0.0,
            alive: true,
            last_hit_by: 0,
            fire_cd: t::SHOOTER_FIRE_INTERVAL * 0.5,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.kind {
            Kind::Charger => t::CHARGER_RADIUS,
            Kind::Shooter => t::SHOOTER_RADIUS,
            Kind::Dummy => t::CHARGER_RADIUS,
        }
    }

    /// 지금 닿으면 플레이어가 입는 피해.
    pub fn contact_damage(&self) -> Option<i32> {
        match self.state {
            State::Charge { .. } => Some(t::CHARGER_DAMAGE),
            _ => None,
        }
    }

    /// 돌진이 플레이어에게 명중했을 때.
    pub fn on_hit_player(&mut self) {
        if matches!(self.state, State::Charge { .. }) {
            self.state = State::Rest {
                time: t::CHARGER_REST,
            };
        }
    }

    pub fn update(
        &mut self,
        player_pos: Vec2,
        player_targetable: bool,
        room: &Room,
        dt: f32,
        projectiles: &mut Vec<Projectile>,
        events: &mut Vec<Event>,
    ) {
        self.prev_pos = self.pos;
        self.flash = (self.flash - dt).max(0.0);
        if self.kind == Kind::Dummy {
            // 허수아비: 넉백만 받고 제자리로 돌아오지 않는다. 경직은 즉시 푼다.
            self.pos += self.vel * dt;
            combat::apply_friction(&mut self.vel, dt);
            let r = self.radius();
            room.resolve_circle(&mut self.pos, r);
            return;
        }
        let to_player = player_pos - self.pos;
        let dist = to_player.length();
        let dir_to_player = to_player.normalize_or_zero();

        match self.state {
            State::Stunned { time } => {
                let time = time - dt;
                if time <= 0.0 {
                    self.state = self.idle_state();
                } else {
                    self.state = State::Stunned { time };
                }
            }
            State::Approach => {
                if player_targetable {
                    self.facing = dir_to_player;
                    if dist < t::CHARGER_TRIGGER_RANGE {
                        self.state = State::Windup { time: 0.0 };
                    } else {
                        self.pos += dir_to_player * t::CHARGER_WALK_SPEED * dt;
                    }
                }
            }
            State::Windup { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= t::CHARGER_WINDUP {
                    self.state = State::Charge {
                        time: 0.0,
                        dir: self.facing,
                    };
                    events.push(Event::Charge);
                } else {
                    self.state = State::Windup { time };
                }
            }
            State::Charge { time, dir } => {
                let time = time + dt;
                self.pos += dir * t::CHARGER_CHARGE_SPEED * dt;
                let r = self.radius();
                if room.resolve_circle(&mut self.pos, r) {
                    self.state = State::Rest {
                        time: t::CHARGER_WALL_REST,
                    };
                    self.flash = t::HIT_FLASH;
                    events.push(Event::WallHit);
                } else if time >= t::CHARGER_CHARGE_TIME {
                    self.state = State::Rest {
                        time: t::CHARGER_REST,
                    };
                } else {
                    self.state = State::Charge { time, dir };
                }
            }
            State::Rest { time } => {
                let time = time - dt;
                self.state = if time <= 0.0 {
                    State::Approach
                } else {
                    State::Rest { time }
                };
            }
            State::Keep => {
                if player_targetable {
                    self.facing = dir_to_player;
                    if dist < t::SHOOTER_KEEP_MIN {
                        self.pos -= dir_to_player * t::SHOOTER_SPEED * dt;
                    } else if dist > t::SHOOTER_KEEP_MAX {
                        self.pos += dir_to_player * t::SHOOTER_SPEED * dt;
                    }
                    self.fire_cd -= dt;
                    if self.fire_cd <= 0.0 {
                        self.state = State::Aim { time: 0.0 };
                    }
                }
            }
            State::Aim { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= t::SHOOTER_AIM_TIME {
                    let start =
                        self.pos + self.facing * (self.radius() + t::PROJECTILE_RADIUS + 1.0);
                    projectiles.push(Projectile {
                        pos: start,
                        prev_pos: start,
                        vel: self.facing * t::PROJECTILE_SPEED,
                        alive: true,
                    });
                    events.push(Event::Shoot);
                    self.fire_cd = t::SHOOTER_FIRE_INTERVAL;
                    self.state = State::Keep;
                } else {
                    self.state = State::Aim { time };
                }
            }
        }

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        room.resolve_circle(&mut self.pos, r);
    }

    fn idle_state(&self) -> State {
        match self.kind {
            Kind::Charger => State::Approach,
            Kind::Shooter => State::Keep,
            Kind::Dummy => State::Rest { time: f32::MAX },
        }
    }

    /// 피격. 넉백 방향은 단위 벡터.
    pub fn hurt(&mut self, dmg: i32, knock_dir: Vec2, knockback: f32, stun: f32) {
        self.hp -= dmg;
        self.flash = t::HIT_FLASH;
        self.vel = knock_dir * knockback;
        self.state = State::Stunned { time: stun };
        if self.hp <= 0 {
            self.alive = false;
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self.state {
            State::Approach => "approach",
            State::Windup { .. } => "windup",
            State::Charge { .. } => "CHARGE",
            State::Rest { .. } => "rest",
            State::Keep => "keep",
            State::Aim { .. } => "aim",
            State::Stunned { .. } => "stunned",
        }
    }
}
