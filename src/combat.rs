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
    Emerge,
    /// 필살기 드릴의 연타 한 번.
    Drill,
    /// 드릴이 벽에 부딪힌 충격파.
    DrillWall,
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
}

impl Feel {
    /// 매 틱, 히트스톱 여부와 무관하게 호출.
    pub fn tick(&mut self, dt: f32) {
        self.trauma = (self.trauma - tuning::SHAKE_DECAY * dt).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
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
}

/// 넉백 속도 감쇠.
pub fn apply_friction(vel: &mut Vec2, dt: f32) {
    *vel *= (-tuning::KNOCKBACK_FRICTION * dt).exp();
    if vel.length_squared() < 1.0 {
        *vel = Vec2::ZERO;
    }
}
