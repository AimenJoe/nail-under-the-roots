//! 떨어지는 것: 화폐(반딧불)와 가방 아이템(산딸기·쥬시 지렁이), 생명의 씨앗. 적이 쓰러질 때 튀어나와 바닥에 구르고,
//! 다가가면 줍는다(docs/decisions/0016, 0017).
//!
//! - 반딧불: 땅속에서 빛은 귀하다 → 이웃들의 돈. 상점(2단계)에서 쓴다. 가까이 가면 끌려온다(자석).
//! - 산딸기(체력 1)·쥬시 지렁이(체력 가득 / 목숨): 가방에 들어간다. 가방이 가득이면 줍지 않고 남는다. 끌려오지 않는다.
//! - 생명의 씨앗: 최대 체력 +1(영구). 지금은 개발 시나리오에만, 나중에 보스·숨은 곳.
//! - 산딸기가 떨어질 확률은 깎인 체력에 비례하고, 오래 안 나오면 반드시 나온다(불운 보정). 지렁이는 드물게 고정 확률.

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::room::Room;
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootKind {
    Glowbug,
    Berry,
    Worm,
    LifeSeed,
}

pub struct Loot {
    pub kind: LootKind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    vel: Vec2,
    /// 튀어 오른 높이(그릴 때 위로). 0 = 바닥.
    pub z: f32,
    vz: f32,
    /// 나온 뒤 흐른 시간. 짧은 동안은 줍지 않는다(튀어나오는 걸 보게).
    pub age: f32,
    /// 그리기 흔들림·깜빡임을 서로 다르게.
    pub phase: f32,
}

impl Loot {
    /// 적 위치에서 아무 방향으로 튀어나온다.
    pub fn spawn(kind: LootKind, at: Vec2) -> Self {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(t::LOOT_POP_SPEED * 0.4, t::LOOT_POP_SPEED);
        Self {
            kind,
            pos: at,
            prev_pos: at,
            vel: vec2(ang.cos(), ang.sin()) * sp,
            z: 0.0,
            vz: rand::gen_range(t::LOOT_POP_UP * 0.7, t::LOOT_POP_UP),
            age: 0.0,
            phase: rand::gen_range(0.0, 1.0),
        }
    }

    /// 튀지 않고 그 자리에 놓는다(개발 시나리오용).
    pub fn place(kind: LootKind, at: Vec2) -> Self {
        Self {
            vel: Vec2::ZERO,
            vz: 0.0,
            age: t::LOOT_PICKUP_DELAY,
            ..Self::spawn(kind, at)
        }
    }

    /// 한 틱. `can_take`: 지금 주울 수 있나(산딸기는 체력이 가득이면 아니오). 주웠으면 true.
    pub fn update(&mut self, dt: f32, player: Vec2, can_take: bool, room: &Room) -> bool {
        self.prev_pos = self.pos;
        self.age += dt;
        // 튀어 오름(가짜 높이)
        if self.z > 0.0 || self.vz > 0.0 {
            self.vz -= t::LOOT_GRAVITY * dt;
            self.z = (self.z + self.vz * dt).max(0.0);
            if self.z <= 0.0 && self.vz < 0.0 {
                self.vz = if self.vz < -40.0 {
                    -self.vz * 0.35
                } else {
                    0.0
                };
            }
        }
        let ready = self.age >= t::LOOT_PICKUP_DELAY;
        let to = player - self.pos;
        let dist = to.length();
        if ready && can_take && self.kind == LootKind::Glowbug && dist < t::GLOWBUG_MAGNET_RADIUS {
            // 가까울수록 세게 끌린다.
            let k = 1.0 - dist / t::GLOWBUG_MAGNET_RADIUS;
            self.vel += to / dist.max(0.01) * t::GLOWBUG_MAGNET_ACCEL * (0.3 + k) * dt;
            let max = t::GLOWBUG_MAGNET_MAX_SPEED;
            if self.vel.length() > max {
                self.vel = self.vel.normalize() * max;
            }
        } else {
            self.vel *= (1.0 - t::LOOT_FRICTION * dt).max(0.0);
        }
        self.pos += self.vel * dt;
        if room.resolve_circle(&mut self.pos, 2.0) {
            self.vel *= 0.5;
        }
        ready && can_take && (player - self.pos).length() <= t::LOOT_COLLECT_RADIUS
    }
}

/// 적 종류별 반딧불 수(최소, 최대).
pub fn glowbugs_for(kind: Kind) -> (u32, u32) {
    match kind {
        Kind::Charger => (2, 3),
        Kind::Shooter => (3, 4),
        Kind::Dummy => (0, 0),
    }
}

/// 이 처치에서 산딸기가 떨어질 확률. 깎인 체력만큼 오르고, `since_heal`번 연속 안 나왔으면 반드시.
pub fn berry_chance(hp: i32, max_hp: i32, since_heal: u32, kind: Kind) -> f32 {
    let missing = (max_hp - hp).max(0);
    if kind == Kind::Dummy || missing == 0 {
        return 0.0;
    }
    if since_heal >= t::BERRY_PITY_KILLS {
        return 1.0;
    }
    (t::BERRY_BASE_CHANCE + t::BERRY_CHANCE_PER_MISSING * missing as f32).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_berry_at_full_hp_or_from_dummy() {
        assert_eq!(berry_chance(5, 5, 100, Kind::Charger), 0.0);
        assert_eq!(berry_chance(1, 5, 100, Kind::Dummy), 0.0);
    }

    #[test]
    fn berry_chance_grows_with_missing_hp_and_pity_guarantees() {
        let one = berry_chance(4, 5, 0, Kind::Charger);
        let three = berry_chance(2, 5, 0, Kind::Charger);
        assert!(three > one && one > 0.0);
        assert_eq!(berry_chance(4, 5, t::BERRY_PITY_KILLS, Kind::Shooter), 1.0);
    }

    #[test]
    fn dummy_drops_no_glowbugs() {
        assert_eq!(glowbugs_for(Kind::Dummy), (0, 0));
        assert!(glowbugs_for(Kind::Shooter).0 >= glowbugs_for(Kind::Charger).0);
    }
}
