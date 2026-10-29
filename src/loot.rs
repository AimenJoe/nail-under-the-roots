//! 떨어지는 것: 화폐 둘(반딧불·씨앗)과 가방 아이템(산딸기·쥬시 지렁이), 뿌리 심장. 적이 쓰러질 때 튀어나와 바닥에 구르고,
//! 다가가면 줍는다(docs/decisions/0016, 0017, 0021).
//!
//! - 반딧불: 기술을 깨우는 빛(기술 해금, 나중에). 많이 나온다. 가까이 가면 끌려온다(자석).
//! - 씨앗: 상점 화폐. 드물게(종마다 확률), 상자·보스에서 한 줌. 끌려온다.
//! - 산딸기(체력 1)·쥬시 지렁이(체력 가득 / 목숨): 가방에 들어간다. 가방이 가득이면 줍지 않고 남는다. 끌려오지 않는다.
//! - 뿌리 심장: 최대 체력 +1(영구). 던전 보스.
//! - 산딸기가 떨어질 확률은 깎인 체력에 비례하고, 오래 안 나오면 반드시 나온다(불운 보정). 지렁이는 드물게 고정 확률.

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::room::Room;
use crate::tuning as t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootKind {
    Glowbug,
    /// 상점 화폐.
    Seed,
    Berry,
    Worm,
    /// 최대 체력 +1.
    RootHeart,
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
    /// 굴러갈 수 있는 사각형(던전: 떨어진 방의 안쪽). 문 칸으로 굴러가 주우려다 옆 방으로 넘어가는 일을 막는다.
    pub bounds: Option<Rect>,
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
            bounds: None,
        }
    }

    /// 이 사각형 밖으로는 굴러가지 않는다. 나온 자리가 밖이면 안으로 옮긴다.
    pub fn within(mut self, bounds: Option<Rect>) -> Self {
        self.bounds = bounds;
        self.keep_inside();
        self.prev_pos = self.pos;
        self
    }

    fn keep_inside(&mut self) {
        let Some(b) = self.bounds else {
            return;
        };
        let p = vec2(
            self.pos.x.clamp(b.x, b.right()),
            self.pos.y.clamp(b.y, b.bottom()),
        );
        if p.x != self.pos.x {
            self.vel.x = -self.vel.x * 0.4;
        }
        if p.y != self.pos.y {
            self.vel.y = -self.vel.y * 0.4;
        }
        self.pos = p;
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
        if ready && can_take && dist < t::LOOT_MAGNET_RADIUS {
            // 가까울수록 세게 끌린다.
            let k = 1.0 - dist / t::LOOT_MAGNET_RADIUS;
            self.vel += to / dist.max(0.01) * t::LOOT_MAGNET_ACCEL * (0.3 + k) * dt;
            let max = t::LOOT_MAGNET_MAX_SPEED;
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
        self.keep_inside();
        ready && can_take && (player - self.pos).length() <= t::LOOT_COLLECT_RADIUS
    }
}

/// 적 종류별 반딧불 수(최소, 최대).
pub fn glowbugs_for(kind: Kind) -> (u32, u32) {
    kind.species().glowbugs
}

/// 이 처치에서 산딸기가 떨어질 확률. 깎인 체력의 비율만큼 오르고(최대 체력과 무관, 0252), `since_heal`번 연속 안 나왔으면 반드시.
pub fn berry_chance(hp: i32, max_hp: i32, since_heal: u32, kind: Kind) -> f32 {
    let missing = (max_hp - hp).max(0);
    if kind == Kind::Dummy || missing == 0 {
        return 0.0;
    }
    if since_heal >= t::BERRY_PITY_KILLS {
        return 1.0;
    }
    let ratio = missing as f32 / max_hp.max(1) as f32;
    (t::BERRY_BASE_CHANCE + t::BERRY_CHANCE_MISSING_ALL * ratio).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 던전 방 안쪽에 묶인 것은 문 쪽으로 세게 튀어도 안쪽에 남는다.
    #[test]
    fn bounded_loot_stays_inside() {
        let room = Room::new();
        let b = Rect::new(40.0, 40.0, 100.0, 60.0);
        let mut l = Loot::spawn(LootKind::Berry, vec2(10.0, 70.0)).within(Some(b));
        assert!(b.contains(l.pos), "나온 자리가 밖이면 안으로");
        l.vel = vec2(-500.0, 0.0);
        for _ in 0..60 {
            l.update(t::TICK_DT, vec2(300.0, 300.0), true, &room);
            assert!(l.pos.x >= b.x && l.pos.x <= b.right());
        }
    }

    #[test]
    fn no_berry_at_full_hp_or_from_dummy() {
        assert_eq!(berry_chance(5, 5, 100, Kind::FieldMouse), 0.0);
        assert_eq!(berry_chance(1, 5, 100, Kind::Dummy), 0.0);
    }

    #[test]
    fn berry_chance_grows_with_missing_hp_and_pity_guarantees() {
        let one = berry_chance(4, 5, 0, Kind::FieldMouse);
        let three = berry_chance(2, 5, 0, Kind::FieldMouse);
        assert!(three > one && one > 0.0);
        assert_eq!(berry_chance(4, 5, t::BERRY_PITY_KILLS, Kind::Chipmunk), 1.0);
        // 0252: 시작 체력에서 한 칸 = 5%, 그리고 최대 체력이 커도 같은 비율이면 같은 확률.
        let hp = t::PLAYER_HP;
        let base_one = berry_chance(hp - 1, hp, 0, Kind::FieldMouse);
        assert!((base_one - (t::BERRY_BASE_CHANCE + 0.05)).abs() < 1e-6);
        assert!((berry_chance(8, 10, 0, Kind::FieldMouse) - one).abs() < 1e-6);
        assert!(
            berry_chance(9, 10, 0, Kind::FieldMouse) < one,
            "큰 체력의 한 칸은 덜"
        );
    }

    /// 모든 떨어진 것이 반딧불만큼 끌려와 주워진다. 못 담는 것(can_take = false)은 끌리지 않는다(0207).
    #[test]
    fn every_loot_kind_is_pulled_in_like_glowbugs() {
        let room = Room::new();
        let player = vec2(160.0, 90.0);
        let from = player + vec2(t::LOOT_MAGNET_RADIUS - 4.0, 0.0);
        for kind in [
            LootKind::Glowbug,
            LootKind::Seed,
            LootKind::Berry,
            LootKind::Worm,
            LootKind::RootHeart,
        ] {
            let mut l = Loot::place(kind, from);
            let got = (0..120).any(|_| l.update(t::TICK_DT, player, true, &room));
            assert!(got, "{kind:?}는 자석 거리 안에서 주워진다");
            let mut full = Loot::place(kind, from);
            let got = (0..120).any(|_| full.update(t::TICK_DT, player, false, &room));
            assert!(
                !got && full.pos == from,
                "{kind:?}: 못 담으면 끌리지 않는다"
            );
        }
    }

    #[test]
    fn dummy_drops_no_glowbugs() {
        assert_eq!(glowbugs_for(Kind::Dummy), (0, 0));
        assert!(glowbugs_for(Kind::Chipmunk).0 >= glowbugs_for(Kind::FieldMouse).0);
    }
}
