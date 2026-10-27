//! 히트박스 모양·판정, 타격 정보, 전투감 상태(히트스톱·흔들림).
//!
//! 히트스톱은 로직 시계만 멈춘다: `World::tick`이 `Feel::hitstop > 0`이면 입력 버퍼만 갱신하고 돌아간다.
//! 렌더는 계속 돌아가므로 흔들림·플래시는 멈춘 화면 위에서 보인다.

use macroquad::prelude::*;

use crate::tuning;

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Circle {
        center: Vec2,
        r: f32,
    },
    /// 전방 반원. `dir`은 단위 벡터.
    Sector {
        origin: Vec2,
        dir: Vec2,
        r: f32,
    },
}

impl Shape {
    /// 원(적·투사체)과 겹치는가.
    /// 부채꼴이면 그 방향, 원이면 None(0162 매듭 불똥 방향용).
    pub fn dir(&self) -> Option<Vec2> {
        match *self {
            Shape::Sector { dir, .. } => Some(dir),
            Shape::Circle { .. } => None,
        }
    }

    pub fn overlaps_circle(&self, c: Vec2, cr: f32) -> bool {
        match *self {
            Shape::Circle { center, r } => (c - center).length_squared() <= (r + cr) * (r + cr),
            Shape::Sector { origin, dir, r } => {
                let d = c - origin;
                let dist2 = d.length_squared();
                if dist2 > (r + cr) * (r + cr) {
                    return false;
                }
                // 바로 붙어 있으면 방향과 무관하게 맞는다. 아니면 전방 180도만.
                dist2 <= cr * cr || d.dot(dir) >= 0.0
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrikeKind {
    Claw(usize),
    /// 강공격(0161): 모았다 떼는 한 방. 발톱처럼 등딱지에 튕기고 분노를 채운다.
    Heavy,
    Emerge,
    /// 필살기 드릴의 연타 한 번.
    Drill,
    /// 드릴이 벽에 부딪힌 충격파.
    DrillWall,
    /// 모모의 이어 뛰기 착지(0056).
    Stomp,
}

/// 플레이어가 이번 틱에 휘두르는 공격 하나.
#[derive(Debug, Clone, Copy)]
pub struct Strike {
    /// 스윙마다 다른 번호. 같은 스윙은 같은 적을 두 번 때리지 않는다.
    pub id: u32,
    pub kind: StrikeKind,
    pub shape: Shape,
    pub damage: i32,
    pub knockback: f32,
    pub stun: f32,
    pub hitstop: f32,
    pub shake: f32,
    /// 콤보의 마지막 타인가(튜토리얼·통계용).
    pub finisher: bool,
    /// 맞은 적에게 거는 독(초).
    pub poison: f32,
}

impl Strike {
    /// 공격이 향한 쪽(원형이면 `fallback`).
    pub fn shape_dir_or(&self, fallback: Vec2) -> Vec2 {
        self.shape.dir().unwrap_or(fallback)
    }
}

/// 전투감 상태. 수치는 tuning.rs.
#[derive(Default)]
pub struct Feel {
    /// 남은 히트스톱(초). 로직 시계를 멈춘다.
    pub hitstop: f32,
    /// 흔들림 "외상" 0..1. 오프셋 = max × trauma². (Squirrel Eiserloh, GDC 2016 "Math for Game Programmers: Juicing Your Cameras With Math")
    pub trauma: f32,
    /// 이번 틱의 흔들림 오프셋(정수 px).
    pub shake_offset: Vec2,
    /// 화면 번쩍임(남은 초). 렌더가 하얗게 덮는다.
    pub flash: f32,
    /// 카메라 킥(0168): 맞힌 쪽으로 찼다가 지수로 돌아오는 오프셋(px). 흔들림의 제곱 곡선이 못 담는 작은 타격을 맡는다.
    pub kick: Vec2,
}

impl Feel {
    /// 매 틱, 히트스톱 여부와 무관하게 호출.
    pub fn tick(&mut self, dt: f32) {
        self.trauma = (self.trauma - tuning::SHAKE_DECAY * dt).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        self.kick *= (-tuning::KICK_DECAY * dt).exp();
        if self.kick.length_squared() < 0.04 {
            self.kick = Vec2::ZERO;
        }
        let amount = tuning::SHAKE_MAX_OFFSET * self.trauma * self.trauma;
        self.shake_offset = if amount < 0.5 {
            Vec2::ZERO
        } else {
            vec2(
                (rand::gen_range(-1.0, 1.0) * amount).round(),
                (rand::gen_range(-1.0, 1.0) * amount).round(),
            )
        };
    }

    pub fn add_hitstop(&mut self, s: f32) {
        self.hitstop = self.hitstop.max(s);
    }

    pub fn add_trauma(&mut self, t: f32) {
        self.trauma = (self.trauma + t).min(1.0);
    }

    /// 카메라를 `dir` 쪽으로 `px`만큼 찬다(큰 쪽이 남는다).
    pub fn add_kick(&mut self, dir: Vec2, px: f32) {
        let k = dir.normalize_or_zero() * px;
        if k.length_squared() > self.kick.length_squared() {
            self.kick = k;
        }
    }
}

/// 넉백 속도 감쇠.
pub fn apply_friction(vel: &mut Vec2, dt: f32) {
    *vel *= (-tuning::KNOCKBACK_FRICTION * dt).exp();
    if vel.length_squared() < 1.0 {
        *vel = Vec2::ZERO;
    }
}

/// 점에서 선분까지 거리.
pub fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let k = if ab.length_squared() > 1e-6 {
        ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + ab * k - p).length()
}
