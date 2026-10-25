//! 적: 뿌리에 물든 땅속 이웃. 종류(`Kind`)마다 행동 하나와 수치가 `species.rs` 표에 있다.
//! 행동: 돌격(쥐들), 던지기(다람쥐), 뛰어 덮치기(청설모). 플레이어 위치와 `Room`만 본다.

use macroquad::prelude::*;

use crate::combat;
use crate::room::Room;
use crate::species::{self, Behavior, Species};
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// 들쥐: 기본 돌격.
    FieldMouse,
    /// 생쥐: 작고 빠르고 약하다. 떼로.
    HouseMouse,
    /// 시궁쥐: 크고 단단하고 세게 돌진.
    SewerRat,
    /// 다람쥐: 도토리를 던진다.
    Chipmunk,
    /// 청설모: 뛰어 덮친다.
    RedSquirrel,
    /// 튜토리얼 허수아비. 움직이지도 때리지도 않는다.
    Dummy,
}

impl Kind {
    pub fn species(self) -> &'static Species {
        species::of(self)
    }
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
    /// 뛰기형: 웅크림(예고). 착지할 자리를 정해 둔다.
    Crouch { time: f32, target: Vec2 },
    /// 뛰기형: 체공. `from`에서 `to`로 포물선.
    Leap { time: f32, from: Vec2, to: Vec2 },
    /// 공통: 경직(남은 시간).
    Stunned { time: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Charge,
    WallHit,
    Shoot,
    /// 뛰기형이 튀어 오름 / 내려앉음.
    Leap,
    Land,
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
    /// 필드의 적: 플레이어가 가까이 오거나 맞기 전까지 제자리.
    pub asleep: bool,
    /// 던전 방: 이 시간 동안 제자리(들어온 플레이어가 자리 잡게).
    pub wake_delay: f32,
}

pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub alive: bool,
    /// 패링으로 되돌린 것: 적을 때리고 플레이어는 맞지 않는다.
    pub friendly: bool,
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
        let sp = kind.species();
        let hp = sp.hp;
        let state = idle_of(sp.behavior);
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
            fire_cd: sp.fire_every * 0.5,
            poison: 0.0,
            poison_tick: t::POISON_TICK,
            died_of_poison: false,
            asleep: false,
            wake_delay: 0.0,
        }
    }

    pub fn sp(&self) -> &'static Species {
        self.kind.species()
    }

    pub fn radius(&self) -> f32 {
        self.sp().radius
    }

    /// 지금 닿으면 플레이어가 입는 피해.
    pub fn contact_damage(&self) -> Option<i32> {
        match self.state {
            State::Charge { .. } => Some(self.sp().damage),
            // 내려앉기 직전 몇 틱.
            State::Leap { time, .. } if time >= self.sp().dash_time - t::LEAP_LAND_WINDOW => {
                Some(self.sp().damage)
            }
            _ => None,
        }
    }

    /// 피해 판정 반지름(내려앉는 순간은 둘레까지).
    pub fn contact_radius(&self) -> f32 {
        match self.state {
            State::Leap { .. } => self.sp().land_radius,
            _ => self.radius(),
        }
    }

    /// 체공 높이(그리기용 px). 바닥 = 0.
    pub fn height(&self) -> f32 {
        match self.state {
            State::Leap { time, .. } => {
                let k = (time / self.sp().dash_time).clamp(0.0, 1.0);
                4.0 * t::LEAP_HEIGHT * k * (1.0 - k)
            }
            _ => 0.0,
        }
    }

    /// 돌진이 플레이어에게 명중했을 때.
    pub fn on_hit_player(&mut self) {
        if matches!(self.state, State::Charge { .. }) {
            self.state = State::Rest {
                time: self.sp().rest,
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
        let sp = self.sp();
        if sp.behavior == Behavior::Dummy {
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
        if self.asleep && player_targetable && dist < t::ENEMY_AGGRO_RADIUS {
            self.asleep = false;
        }
        if self.wake_delay > 0.0 {
            self.wake_delay -= dt;
        }
        if self.asleep || self.wake_delay > 0.0 {
            // 자는 동안은 넉백만.
            self.pos += self.vel * dt;
            combat::apply_friction(&mut self.vel, dt);
            let r = self.radius();
            room.resolve_circle(&mut self.pos, r);
            return;
        }

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
                    if dist < sp.trigger {
                        self.state = State::Windup { time: 0.0 };
                    } else {
                        self.pos += dir_to_player * sp.speed * dt;
                    }
                }
            }
            State::Windup { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= sp.windup {
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
                self.pos += dir * sp.dash_speed * dt;
                let r = self.radius();
                if room.resolve_circle(&mut self.pos, r) {
                    self.state = State::Rest { time: sp.wall_rest };
                    self.flash = t::HIT_FLASH;
                    events.push(Event::WallHit);
                } else if time >= sp.dash_time {
                    self.state = State::Rest { time: sp.rest };
                } else {
                    self.state = State::Charge { time, dir };
                }
            }
            State::Rest { time } => {
                let time = time - dt;
                self.state = if time <= 0.0 {
                    idle_of(sp.behavior)
                } else {
                    State::Rest { time }
                };
            }
            State::Keep => {
                if player_targetable {
                    self.facing = dir_to_player;
                    let (near, far) = sp.keep;
                    if dist < near {
                        self.pos -= dir_to_player * sp.speed * dt;
                    } else if dist > far {
                        self.pos += dir_to_player * sp.speed * dt;
                    } else if sp.behavior == Behavior::Leaper {
                        // 청설모: 거리 안에서는 옆으로 돈다(가만히 있지 않는다).
                        let side = vec2(-dir_to_player.y, dir_to_player.x);
                        self.pos += side * sp.speed * 0.6 * dt;
                    }
                    self.fire_cd -= dt;
                    if self.fire_cd <= 0.0 {
                        self.state = match sp.behavior {
                            Behavior::Leaper => State::Crouch {
                                time: 0.0,
                                target: player_pos,
                            },
                            _ => State::Aim { time: 0.0 },
                        };
                    }
                }
            }
            State::Crouch { time, target } => {
                let time = time + dt;
                self.facing = (target - self.pos).normalize_or(self.facing);
                if time >= sp.windup {
                    // 너무 멀면 갈 수 있는 데까지만.
                    let reach = (target - self.pos).clamp_length_max(sp.trigger * 1.2);
                    self.state = State::Leap {
                        time: 0.0,
                        from: self.pos,
                        to: self.pos + reach,
                    };
                    events.push(Event::Leap);
                } else {
                    self.state = State::Crouch { time, target };
                }
            }
            State::Leap { time, from, to } => {
                let time = time + dt;
                let k = (time / sp.dash_time).min(1.0);
                self.pos = from + (to - from) * k;
                if time >= sp.dash_time {
                    events.push(Event::Land);
                    self.fire_cd = sp.fire_every;
                    self.state = State::Rest { time: sp.rest };
                } else {
                    self.state = State::Leap { time, from, to };
                }
            }
            State::Aim { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= sp.windup {
                    let start =
                        self.pos + self.facing * (self.radius() + t::PROJECTILE_RADIUS + 1.0);
                    projectiles.push(Projectile {
                        pos: start,
                        prev_pos: start,
                        vel: self.facing * sp.shot_speed,
                        alive: true,
                        friendly: false,
                    });
                    events.push(Event::Shoot);
                    self.fire_cd = sp.fire_every;
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
        idle_of(self.sp().behavior)
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

    /// 피격. 넉백 방향은 단위 벡터. 무거운 종은 덜 밀린다. 체공 중엔 떨어진다(그 자리에서 경직).
    pub fn hurt(&mut self, dmg: i32, knock_dir: Vec2, knockback: f32, stun: f32) {
        self.asleep = false;
        self.wake_delay = 0.0;
        self.hp -= dmg;
        self.flash = t::HIT_FLASH;
        self.vel = knock_dir * knockback * self.sp().knock_mul;
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
            State::Crouch { .. } => "crouch",
            State::Leap { .. } => "LEAP",
            State::Stunned { .. } => "stunned",
        }
    }
}

fn idle_of(b: Behavior) -> State {
    match b {
        Behavior::Charger => State::Approach,
        Behavior::Shooter | Behavior::Leaper => State::Keep,
        Behavior::Dummy => State::Rest { time: f32::MAX },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD_HP: i32 = species::FIELD_MOUSE.hp;

    /// 청설모: 웅크렸다가 뛰고, 내려앉는 순간에만 둘레 피해.
    #[test]
    fn squirrel_crouches_leaps_and_lands() {
        let room = Room::new();
        let mut e = Enemy::new(Kind::RedSquirrel, room.center());
        e.fire_cd = 0.0;
        let player = room.center() + vec2(70.0, 0.0);
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut saw = (false, false, false);
        for _ in 0..240 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            match e.state {
                State::Crouch { .. } => saw.0 = true,
                State::Leap { .. } => {
                    saw.1 = true;
                    if e.contact_damage().is_some() {
                        saw.2 = true;
                        assert!(e.contact_radius() > e.radius());
                    }
                }
                _ => assert!(e.contact_damage().is_none()),
            }
        }
        assert!(saw.0 && saw.1 && saw.2, "{saw:?}");
        assert!(ev.contains(&Event::Land));
    }

    #[test]
    fn poison_ticks_down_hp_and_kills() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(100.0, 100.0));
        assert_eq!(e.hp, FIELD_HP);
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
        let mut e = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        e.apply_poison(t::VENOM_POISON);
        for _ in 0..600 {
            e.tick_poison(t::TICK_DT);
        }
        assert_eq!(e.hp, FIELD_HP - 1);
        assert_eq!(e.poison, 0.0);
        assert!(e.alive);
    }
}
