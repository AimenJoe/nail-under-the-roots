//! 큰 나무의 재주(docs/decisions/0047): 반딧불로 깨우는 영구 기술.
//!
//! 자리 넷(굴파기·패링·드릴·먹기), 자리마다 둘 중 하나를 고른다(Hades 밤의 거울의 두 갈래). 한쪽을 얻으면 다른 쪽의
//! 좋은 점을 버린다 — 세지는 것보다 "다르게 싸우는 것"(0040). 깨운 것끼리는 마을에서 언제든 바꿔 낀다(공짜).
//! 수치는 이미 있는 것(tuning)에 배율·값을 얹는다. 플레이어·월드는 `Talents`만 본다.

use crate::i18n::Tr;
use crate::tuning as t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Burrow,
    Parry,
    Drill,
    Eat,
}

pub const SLOTS: [Slot; 4] = [Slot::Burrow, Slot::Parry, Slot::Drill, Slot::Eat];

pub struct Talent {
    pub name: Tr,
    pub desc: Tr,
    /// 깨우는 값(반딧불).
    pub cost: u32,
}

/// 자리별 (이름, [갈래 A, 갈래 B]).
pub const TABLE: [(Tr, [Talent; 2]); 4] = [
    (
        |s| s.tal_burrow,
        [
            Talent {
                name: |s| s.tal_burrow_a,
                desc: |s| s.tal_burrow_a_desc,
                cost: 40,
            },
            Talent {
                name: |s| s.tal_burrow_b,
                desc: |s| s.tal_burrow_b_desc,
                cost: 40,
            },
        ],
    ),
    (
        |s| s.tal_parry,
        [
            Talent {
                name: |s| s.tal_parry_a,
                desc: |s| s.tal_parry_a_desc,
                cost: 60,
            },
            Talent {
                name: |s| s.tal_parry_b,
                desc: |s| s.tal_parry_b_desc,
                cost: 60,
            },
        ],
    ),
    (
        |s| s.tal_drill,
        [
            Talent {
                name: |s| s.tal_drill_a,
                desc: |s| s.tal_drill_a_desc,
                cost: 80,
            },
            Talent {
                name: |s| s.tal_drill_b,
                desc: |s| s.tal_drill_b_desc,
                cost: 80,
            },
        ],
    ),
    (
        |s| s.tal_eat,
        [
            Talent {
                name: |s| s.tal_eat_a,
                desc: |s| s.tal_eat_a_desc,
                cost: 50,
            },
            Talent {
                name: |s| s.tal_eat_b,
                desc: |s| s.tal_eat_b_desc,
                cost: 50,
            },
        ],
    ),
];

impl Slot {
    pub fn index(self) -> usize {
        SLOTS.iter().position(|s| *s == self).unwrap_or(0)
    }

    fn key(self) -> &'static str {
        match self {
            Slot::Burrow => "burrow",
            Slot::Parry => "parry",
            Slot::Drill => "drill",
            Slot::Eat => "eat",
        }
    }
}

/// 기록 키(바꾸지 않는다, 0041): 깨움(깃발), 낀 갈래(수 0 = 없음, 1 = A, 2 = B), 맡긴 반딧불(수).
pub fn key_awake(slot: Slot, side: usize) -> String {
    format!(
        "talent.{}.{}",
        slot.key(),
        if side == 0 { "a" } else { "b" }
    )
}
pub fn key_pick(slot: Slot) -> String {
    format!("talent_pick.{}", slot.key())
}
pub const KEY_LIGHT: &str = "glowbugs_banked";

/// 이번 런에 낀 재주. 자리마다 0 = 없음, 1 = A, 2 = B.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Talents {
    pub pick: [u8; 4],
}

impl Talents {
    fn side(self, slot: Slot) -> u8 {
        self.pick[slot.index()]
    }

    /// 굴파기 A 깊은 굴: 오래 숨고 크게 솟구치지만 다시 파기까지 길다. B 얕은 굴: 짧게 숨고 금방 다시.
    pub fn burrow_time(self) -> f32 {
        match self.side(Slot::Burrow) {
            1 => t::TAL_DEEP_TIME,
            2 => t::TAL_SHALLOW_TIME,
            _ => t::BURROW_MAX_TIME,
        }
    }
    pub fn burrow_cd_mul(self) -> f32 {
        match self.side(Slot::Burrow) {
            1 => t::TAL_DEEP_CD,
            2 => t::TAL_SHALLOW_CD,
            _ => 1.0,
        }
    }
    pub fn emerge_radius(self) -> f32 {
        match self.side(Slot::Burrow) {
            1 => t::TAL_DEEP_RADIUS,
            2 => t::TAL_SHALLOW_RADIUS,
            _ => t::EMERGE_RADIUS,
        }
    }

    /// 패링 A 되받아치기: 튕겨낸 돌격에 피해(게이지는 안 찬다). B 빛 모으기: 게이지가 크게 찬다.
    pub fn parry_sp(self) -> f32 {
        match self.side(Slot::Parry) {
            1 => 0.0,
            2 => t::TAL_PARRY_SP,
            _ => t::PARRY_SP,
        }
    }
    pub fn parry_counter(self) -> i32 {
        if self.side(Slot::Parry) == 1 {
            t::TAL_PARRY_COUNTER
        } else {
            0
        }
    }

    /// 드릴 A 긴 드릴: 길게 달리지만 게이지가 느리게 찬다. B 짧은 드릴: 짧게, 자주.
    pub fn drill_mul(self) -> f32 {
        match self.side(Slot::Drill) {
            1 => t::TAL_LONG_DRILL,
            2 => t::TAL_SHORT_DRILL,
            _ => 1.0,
        }
    }
    pub fn sp_gain_mul(self) -> f32 {
        match self.side(Slot::Drill) {
            1 => t::TAL_LONG_DRILL_SP,
            2 => t::TAL_SHORT_DRILL_SP,
            _ => 1.0,
        }
    }

    /// 먹기 A 꼭꼭 씹기: 오래 먹고 산딸기가 더 채운다. B 한입에: 빨리 먹는다.
    pub fn eat_time_mul(self) -> f32 {
        match self.side(Slot::Eat) {
            1 => t::TAL_CHEW_TIME,
            2 => t::TAL_GULP_TIME,
            _ => 1.0,
        }
    }
    pub fn berry_heal(self) -> i32 {
        if self.side(Slot::Eat) == 1 {
            t::TAL_CHEW_HEAL
        } else {
            t::BERRY_HEAL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(slot: Slot, side: u8) -> Talents {
        let mut k = Talents::default();
        k.pick[slot.index()] = side;
        k
    }

    #[test]
    fn none_picked_is_the_base_game() {
        let k = Talents::default();
        assert_eq!(k.burrow_time(), t::BURROW_MAX_TIME);
        assert_eq!(k.emerge_radius(), t::EMERGE_RADIUS);
        assert_eq!(k.parry_sp(), t::PARRY_SP);
        assert_eq!(k.parry_counter(), 0);
        assert_eq!(k.drill_mul(), 1.0);
        assert_eq!(k.berry_heal(), t::BERRY_HEAL);
    }

    /// 옆으로(0040): 갈래마다 좋은 점이 있으면 나쁜 점도 있다.
    #[test]
    fn every_side_trades_something() {
        let deep = with(Slot::Burrow, 1);
        assert!(deep.emerge_radius() > t::EMERGE_RADIUS && deep.burrow_cd_mul() > 1.0);
        let shallow = with(Slot::Burrow, 2);
        assert!(shallow.burrow_cd_mul() < 1.0 && shallow.emerge_radius() < t::EMERGE_RADIUS);
        let counter = with(Slot::Parry, 1);
        assert!(counter.parry_counter() > 0 && counter.parry_sp() < t::PARRY_SP);
        let gather = with(Slot::Parry, 2);
        assert!(gather.parry_sp() > t::PARRY_SP && gather.parry_counter() == 0);
        let long = with(Slot::Drill, 1);
        assert!(long.drill_mul() > 1.0 && long.sp_gain_mul() < 1.0);
        let short = with(Slot::Drill, 2);
        assert!(short.drill_mul() < 1.0 && short.sp_gain_mul() > 1.0);
        let chew = with(Slot::Eat, 1);
        assert!(chew.berry_heal() > t::BERRY_HEAL && chew.eat_time_mul() > 1.0);
        let gulp = with(Slot::Eat, 2);
        assert!(gulp.eat_time_mul() < 1.0 && gulp.berry_heal() == t::BERRY_HEAL);
    }

    #[test]
    fn keys_are_stable() {
        assert_eq!(key_awake(Slot::Drill, 1), "talent.drill.b");
        assert_eq!(key_pick(Slot::Eat), "talent_pick.eat");
    }
}
