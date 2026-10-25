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
    /// 남은 독(초)과 다음 독 피해까지 남은 시간.
    pub poison: f32,
    poison_tick: f32,
    /// 이번 틱에 독으로 쓰러졌다(월드가 처치로 센다).
    pub died_of_poison: bool,
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
            poison: 0.0,
            poison_tick: t::POISON_TICK,
            died_of_poison: false,
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
        self.tick_poison(dt);
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

    /// 독을 건다. 이미 걸려 있으면 더 긴 쪽.
    pub fn apply_poison(&mut self, secs: f32) {
        if self.poison <= 0.0 {
            self.poison_tick = t::POISON_TICK;
        }
        self.poison = self.poison.max(secs);
    }

    fn tick_poison(&mut self, dt: f32) {
        if self.poison <= 0.0 || !self.alive {
            return;
        }
        self.poison -= dt;
        self.poison_tick -= dt;
        if self.poison_tick <= 0.0 {
            self.poison_tick += t::POISON_TICK;
            self.hp -= 1;
            self.flash = t::HIT_FLASH * 0.5;
            if self.hp <= 0 {
                self.alive = false;
                self.died_of_poison = true;
            }
        }
        if self.poison < 0.0 {
            self.poison = 0.0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_ticks_down_hp_and_kills() {
        let mut e = Enemy::new(Kind::Charger, vec2(100.0, 100.0));
        assert_eq!(e.hp, t::CHARGER_HP);
        e.apply_poison(t::VENOM_POISON_FINISHER);
        let dt = t::TICK_DT;
        let mut secs = 0.0;
        while e.alive && secs < 10.0 {
            e.tick_poison(dt);
            secs += dt;
        }
        // 2.5초 독 = 0.8초마다 1 → 3틱, 체력 3인 돌격형은 쓰러진다.
        assert!(!e.alive);
        assert!(e.died_of_poison);
        assert!(secs <= t::VENOM_POISON_FINISHER + dt);
    }

    #[test]
    fn short_poison_deals_one_tick() {
        let mut e = Enemy::new(Kind::Charger, vec2(0.0, 0.0));
        e.apply_poison(t::VENOM_POISON);
        for _ in 0..600 {
            e.tick_poison(t::TICK_DT);
        }
        assert_eq!(e.hp, t::CHARGER_HP - 1);
        assert_eq!(e.poison, 0.0);
        assert!(e.alive);
    }
}
