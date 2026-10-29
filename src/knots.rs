//! 실뜨기: 엔딩 뒤 스스로 고르는 어려움(docs/decisions/0040, 0043).
//!
//! 누룩 할매의 실뜨기 모양 하나가 다음 런의 어려움 하나다. **단계식**(Slay the Spire Ascension):
//! N단계를 고르면 1~N의 매듭이 모두 걸린다. 캐릭터마다 따로 열리고, 지금 열린 가장 높은 단계로
//! 가장 깊은 곳에 닿으면 다음 단계가 열린다. 첫 클리어(0단계)가 1단계를 연다.
//! 매듭은 이미 있는 수치만 바꾼다. 보상은 단계마다 마을로 보내는 반딧불이 늘어나는 것(0247 — 0040의 "힘이 아닌 보상"에 더해).

use crate::i18n::Tr;
use crate::tuning as t;

/// 단계 수(0040의 계획대로 10, 6~10은 0053).
pub const MAX_LEVEL: u8 = 10;

/// 단계별 매듭 이름(1단계부터).
pub const NAMES: [Tr; MAX_LEVEL as usize] = [
    |s| s.knot_1,
    |s| s.knot_2,
    |s| s.knot_3,
    |s| s.knot_4,
    |s| s.knot_5,
    |s| s.knot_6,
    |s| s.knot_7,
    |s| s.knot_8,
    |s| s.knot_9,
    |s| s.knot_10,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Knots {
    pub level: u8,
}

impl Knots {
    pub fn new(level: u8) -> Self {
        Self {
            level: level.min(MAX_LEVEL),
        }
    }

    fn has(self, n: u8) -> bool {
        self.level >= n
    }

    /// 1: 물든 이웃이 더 단단하다.
    pub fn enemy_hp_mul(self) -> f32 {
        if self.has(1) {
            t::KNOT_ENEMY_HP
        } else {
            1.0
        }
    }

    /// 2: 공격 예고가 짧다.
    pub fn windup_mul(self) -> f32 {
        if self.has(2) {
            t::KNOT_WINDUP
        } else {
            1.0
        }
    }

    /// 3: 산딸기가 덜 떨어진다(불운 보정은 그대로).
    pub fn berry_mul(self) -> f32 {
        if self.has(3) {
            t::KNOT_BERRY
        } else {
            1.0
        }
    }

    /// 4: 가게 값이 오른다.
    pub fn price(self, base: u32) -> u32 {
        if self.has(4) {
            (base as f32 * t::KNOT_PRICE).ceil() as u32
        } else {
            base
        }
    }

    /// 5: 먹기 대기가 길다(0038).
    pub fn eat_cd_mul(self) -> f32 {
        if self.has(5) {
            t::KNOT_EAT_CD
        } else {
            1.0
        }
    }

    /// 6: 물든 이웃이 더 빨리 걷는다(돌진·뛰기 속도는 그대로).
    pub fn enemy_speed_mul(self) -> f32 {
        if self.has(6) {
            t::KNOT_SPEED
        } else {
            1.0
        }
    }

    /// 7: 길의 주인(보스)이 더 단단하다. 1단계에 곱한다.
    pub fn boss_hp_mul(self) -> f32 {
        if self.has(7) {
            t::KNOT_BOSS_HP
        } else {
            1.0
        }
    }

    /// 8: 패링 창이 짧다.
    pub fn parry_mul(self) -> f32 {
        if self.has(8) {
            t::KNOT_PARRY
        } else {
            1.0
        }
    }

    /// 9: 굴파기 대기가 길다.
    pub fn burrow_cd_mul(self) -> f32 {
        if self.has(9) {
            t::KNOT_BURROW_CD
        } else {
            1.0
        }
    }

    /// 보상(0247): 마을로 보내는 반딧불이 단계마다 `KNOT_GLOW_STEP`씩 는다. 더한 퍼센트.
    pub fn glow_bonus_pct(self) -> u32 {
        (self.level as f32 * t::KNOT_GLOW_STEP * 100.0).round() as u32
    }

    /// 보상을 더한 반딧불 수(반올림).
    pub fn glow(self, n: u32) -> u32 {
        (n as f32 * (1.0 + self.level as f32 * t::KNOT_GLOW_STEP)).round() as u32
    }

    /// 10: 쥬시 지렁이가 떨어지지 않는다(가게에는 있다).
    pub fn worm_mul(self) -> f32 {
        if self.has(10) {
            0.0
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_stack_knots_in_order() {
        let none = Knots::new(0);
        assert_eq!(none.enemy_hp_mul(), 1.0);
        assert_eq!(none.price(10), 10);
        let two = Knots::new(2);
        assert!(two.enemy_hp_mul() > 1.0 && two.windup_mul() < 1.0);
        assert_eq!(two.berry_mul(), 1.0, "3단계 매듭은 아직");
        let five = Knots::new(5);
        assert!(five.berry_mul() < 1.0 && five.price(10) > 10 && five.eat_cd_mul() > 1.0);
        assert_eq!(Knots::new(99).level, MAX_LEVEL);
        assert_eq!(five.enemy_speed_mul(), 1.0, "6단계 매듭은 아직");
        let ten = Knots::new(10);
        assert!(ten.enemy_speed_mul() > 1.0 && ten.boss_hp_mul() > 1.0);
        assert!(ten.parry_mul() < 1.0 && ten.burrow_cd_mul() > 1.0);
        assert_eq!(ten.worm_mul(), 0.0);
        assert_eq!(NAMES.len(), MAX_LEVEL as usize);
    }

    /// 보상: 0단계는 그대로, 단계마다 반딧불이 는다.
    #[test]
    fn higher_levels_bank_more_glowbugs() {
        assert_eq!(Knots::new(0).glow(20), 20);
        assert_eq!(Knots::new(0).glow_bonus_pct(), 0);
        assert_eq!(Knots::new(2).glow(20), 26);
        assert_eq!(Knots::new(10).glow(20), 50);
        assert_eq!(Knots::new(10).glow_bonus_pct(), 150);
        for l in 1..=MAX_LEVEL {
            assert!(Knots::new(l).glow(40) > Knots::new(l - 1).glow(40));
        }
    }

    /// 적에는 한 번만 건다(월드가 틱마다 불러도 겹치지 않는다).
    #[test]
    fn enemy_gets_knots_once() {
        use crate::enemy::Enemy;
        use crate::enemy::Kind;
        let mut e = Enemy::new(Kind::FieldMouse, macroquad::prelude::Vec2::ZERO);
        let (hp, wind) = (e.max_hp, e.windup());
        e.apply_knots(Knots::new(2), 1.0);
        e.apply_knots(Knots::new(2), 1.0);
        assert_eq!(e.max_hp, (hp as f32 * t::KNOT_ENEMY_HP).ceil() as i32);
        assert_eq!(e.hp, e.max_hp);
        assert!((e.windup() - wind * t::KNOT_WINDUP).abs() < 1e-6);
    }

    /// 7단계: 보스만 체력이 한 번 더 는다. 6단계: 걸음이 빨라진다.
    #[test]
    fn boss_gets_extra_hp_and_mobs_walk_faster() {
        use crate::enemy::Enemy;
        use crate::enemy::Kind;
        let mut b = Enemy::new(Kind::Momo, macroquad::prelude::Vec2::ZERO);
        let hp = b.max_hp;
        b.apply_knots(Knots::new(7), 1.0);
        let once = (hp as f32 * t::KNOT_ENEMY_HP).ceil();
        assert_eq!(b.max_hp, (once * t::KNOT_BOSS_HP).ceil() as i32);
        let mut m = Enemy::new(Kind::FieldMouse, macroquad::prelude::Vec2::ZERO);
        let sp = m.speed();
        m.apply_knots(Knots::new(7), 1.0);
        assert_eq!(
            m.max_hp,
            (crate::species::FIELD_MOUSE.hp as f32 * t::KNOT_ENEMY_HP).ceil() as i32
        );
        assert!(m.speed() > sp);
    }
}
