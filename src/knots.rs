//! 실뜨기: 엔딩 뒤 스스로 고르는 어려움(docs/decisions/0040, 0043).
//!
//! 누룩 할매의 실뜨기 모양 하나가 다음 런의 어려움 하나다. **단계식**(Slay the Spire Ascension):
//! N단계를 고르면 1~N의 매듭이 모두 걸린다. 캐릭터마다 따로 열리고, 지금 열린 가장 높은 단계로
//! 가장 깊은 곳에 닿으면 다음 단계가 열린다. 첫 클리어(0단계)가 1단계를 연다.
//! 매듭은 이미 있는 수치만 바꾼다. 보상은 힘이 아닌 것(0040).

use crate::i18n::Tr;
use crate::tuning as t;

/// 지금 만든 단계 수(계획은 10, 0040).
pub const MAX_LEVEL: u8 = 5;

/// 단계별 매듭 이름(1단계부터).
pub const NAMES: [Tr; MAX_LEVEL as usize] = [
    |s| s.knot_1,
    |s| s.knot_2,
    |s| s.knot_3,
    |s| s.knot_4,
    |s| s.knot_5,
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
    }

    /// 적에는 한 번만 건다(월드가 틱마다 불러도 겹치지 않는다).
    #[test]
    fn enemy_gets_knots_once() {
        use crate::enemy::Enemy;
        use crate::enemy::Kind;
        let mut e = Enemy::new(Kind::FieldMouse, macroquad::prelude::Vec2::ZERO);
        let (hp, wind) = (e.max_hp, e.windup());
        e.apply_knots(Knots::new(2));
        e.apply_knots(Knots::new(2));
        assert_eq!(e.max_hp, (hp as f32 * t::KNOT_ENEMY_HP).ceil() as i32);
        assert_eq!(e.hp, e.max_hp);
        assert!((e.windup() - wind * t::KNOT_WINDUP).abs() < 1e-6);
    }
}
