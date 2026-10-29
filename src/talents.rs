//! 큰 나무의 재주(docs/decisions/0047): 반딧불로 깨우는 영구 기술.
//!
//! 자리 넷(굴파기·패링·드릴·먹기), 자리마다 둘 중 하나를 고른다(Hades 밤의 거울의 두 갈래). 한쪽을 얻으면 다른 쪽의
//! 좋은 점을 버린다 — 세지는 것보다 "다르게 싸우는 것"(0040). 깨운 것끼리는 마을에서 언제든 바꿔 낀다(공짜).
//! 수치는 이미 있는 것(tuning)에 배율·값을 얹는다. 플레이어·월드는 `Talents`만 본다.
//! 모모(0142)는 자리 1·3이 모모 것(깡충·이어 뛰기, `MOMO_TABLE`·키 `talent.hop.*`·`talent.chain.*`), 2·4는 함께.
//! 도토(0195)도 같다(뒤로 뛰기·도토리 묻기, `DOTO_TABLE`·키 `talent.leap.*`·`talent.bury.*`).
//!
//! 0246(사장: "재주는 3개 분기 8개 깊이"): 캐릭터마다 갈래 셋(뿌리 = 몸, 가지 = 싸움, 잎 = 길) × 깊이 여덟. 앞 칸을 깨워야
//! 다음 칸을 깨운다. 갈래마다 한 칸은 위의 두 갈래 고르기(`Node::Pick`, 키·고른 것 그대로) — 나머지는 깨우면 늘 켜지는
//! 덤(`Node::Plus(Boon)`, 키 `tree.<캐릭터>.<갈래>.<깊이>`). 덤은 `Mods`로 장신구와 같은 길(`Stats::compute`)을 탄다.

use crate::equipment::{Character, Mods, NO_MODS};
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

/// `cost`는 처음 정한 값(0047), 실제 값은 `TALENT_PRICE_MUL`배(0225).
const fn talent(name: Tr, desc: Tr, cost: u32) -> Talent {
    Talent {
        name,
        desc,
        cost: cost * t::TALENT_PRICE_MUL,
    }
}

/// 자리 2·4(패링·먹기)는 두 캐릭터가 함께 쓴다(0142).
const PARRY: (Tr, [Talent; 2]) = (
    |s| s.tal_parry,
    [
        talent(|s| s.tal_parry_a, |s| s.tal_parry_a_desc, 60),
        talent(|s| s.tal_parry_b, |s| s.tal_parry_b_desc, 60),
    ],
);
const EAT: (Tr, [Talent; 2]) = (
    |s| s.tal_eat,
    [
        talent(|s| s.tal_eat_a, |s| s.tal_eat_a_desc, 50),
        talent(|s| s.tal_eat_b, |s| s.tal_eat_b_desc, 50),
    ],
);

/// 자리별 (이름, [갈래 A, 갈래 B]). Naïl.
pub const TABLE: [(Tr, [Talent; 2]); 4] = [
    (
        |s| s.tal_burrow,
        [
            talent(|s| s.tal_burrow_a, |s| s.tal_burrow_a_desc, 40),
            talent(|s| s.tal_burrow_b, |s| s.tal_burrow_b_desc, 40),
        ],
    ),
    PARRY,
    (
        |s| s.tal_drill,
        [
            talent(|s| s.tal_drill_a, |s| s.tal_drill_a_desc, 80),
            talent(|s| s.tal_drill_b, |s| s.tal_drill_b_desc, 80),
        ],
    ),
    EAT,
];

/// 모모(0142): 자리 1 = 깡충(멀리뛰기·발 구르기), 자리 3 = 이어 뛰기(네 번 뛰기·높이 뛰기). 값은 Naïl과 같다.
pub const MOMO_TABLE: [(Tr, [Talent; 2]); 4] = [
    (
        |s| s.tal_hop,
        [
            talent(|s| s.tal_hop_a, |s| s.tal_hop_a_desc, 40),
            talent(|s| s.tal_hop_b, |s| s.tal_hop_b_desc, 40),
        ],
    ),
    PARRY,
    (
        |s| s.tal_chain,
        [
            talent(|s| s.tal_chain_a, |s| s.tal_chain_a_desc, 80),
            talent(|s| s.tal_chain_b, |s| s.tal_chain_b_desc, 80),
        ],
    ),
    EAT,
];

/// 도토(0195): 자리 1 = 뒤로 뛰기(떨구고 뛰기·가벼운 발), 자리 3 = 도토리 묻기(한가득·빨리 터지기).
pub const DOTO_TABLE: [(Tr, [Talent; 2]); 4] = [
    (
        |s| s.tal_leap,
        [
            talent(|s| s.tal_leap_a, |s| s.tal_leap_a_desc, 40),
            talent(|s| s.tal_leap_b, |s| s.tal_leap_b_desc, 40),
        ],
    ),
    PARRY,
    (
        |s| s.tal_bury,
        [
            talent(|s| s.tal_bury_a, |s| s.tal_bury_a_desc, 80),
            talent(|s| s.tal_bury_b, |s| s.tal_bury_b_desc, 80),
        ],
    ),
    EAT,
];

/// 캐릭터의 재주 표(캐릭터 표에서, 0170 R14).
pub fn table(ch: Character) -> &'static [(Tr, [Talent; 2]); 4] {
    ch.def().talents
}

impl Slot {
    pub fn index(self) -> usize {
        SLOTS.iter().position(|s| *s == self).unwrap_or(0)
    }

    /// 기록 키의 자리 이름. 모모의 자리 1·3은 따로(0142), 2·4는 Naïl과 같은 키.
    fn key(self, ch: Character) -> &'static str {
        ch.def().talent_keys[self.index()]
    }
}

/// 기록 키(바꾸지 않는다, 0041): 깨움(깃발), 낀 갈래(수 0 = 없음, 1 = A, 2 = B), 맡긴 반딧불(수).
pub fn key_awake(ch: Character, slot: Slot, side: usize) -> String {
    format!(
        "talent.{}.{}",
        slot.key(ch),
        if side == 0 { "a" } else { "b" }
    )
}
pub fn key_pick(ch: Character, slot: Slot) -> String {
    format!("talent_pick.{}", slot.key(ch))
}

/// 모든 캐릭터의 모든 갈래 깨움 키(겹치는 자리 2·4는 한 번). 저널 "깨운 재주 수"가 센다.
pub fn all_awake_keys() -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for ch in crate::characters::ALL.iter().map(|d| d.id) {
        for slot in SLOTS {
            for side in 0..2 {
                let k = key_awake(ch, slot, side);
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        for (b, (_, nodes)) in BRANCHES.iter().enumerate() {
            for (d, n) in nodes.iter().enumerate() {
                if matches!(n, Node::Plus(_)) {
                    keys.push(key_plus(ch, b, d));
                }
            }
        }
    }
    keys
}
pub const KEY_LIGHT: &str = "glowbugs_banked";

/// 이번 런에 낀 재주. 자리마다 0 = 없음, 1 = A, 2 = B. 자리 1·3은 `who`의 표(0142·0195).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Talents {
    pub pick: [u8; 4],
    pub who: Character,
    /// 깨운 덤의 수(`Boon` 순서, 0246).
    pub boons: [u8; BOONS.len()],
}

impl Talents {
    /// 깨운 덤을 모은 보정(0246). `Stats::compute`가 장신구와 함께 더한다.
    pub fn mods(&self) -> Mods {
        let mut m = NO_MODS;
        for (b, &n) in BOONS.iter().zip(self.boons.iter()) {
            for _ in 0..n {
                b.apply(&mut m);
            }
        }
        m
    }

    fn side(self, slot: Slot) -> u8 {
        self.pick[slot.index()]
    }

    /// `ch`의 표에서 고른 자리 1·3 갈래(다른 캐릭터면 없음). 자리 2·4는 `side`.
    fn own(self, ch: Character, slot: Slot) -> u8 {
        if self.who == ch {
            self.side(slot)
        } else {
            0
        }
    }

    /// Naïl 표의 자리 1·3 갈래.
    fn mole(self, slot: Slot) -> u8 {
        self.own(Character::Mole, slot)
    }

    /// 모모 표의 자리 1·3 갈래.
    fn momo_side(self, slot: Slot) -> u8 {
        self.own(Character::Momo, slot)
    }

    /// 도토 표의 자리 1·3 갈래(0195).
    fn doto_side(self, slot: Slot) -> u8 {
        self.own(Character::Doto, slot)
    }

    /// 굴파기 A 깊은 굴: 오래 숨고 크게 솟구치지만 다시 파기까지 길다. B 얕은 굴: 짧게 숨고 금방 다시.
    pub fn burrow_time(self) -> f32 {
        match self.mole(Slot::Burrow) {
            1 => t::TAL_DEEP_TIME,
            2 => t::TAL_SHALLOW_TIME,
            _ => t::BURROW_MAX_TIME,
        }
    }
    /// 회피 대기 배율. 모모는 깡충(A 멀리뛰기·B 발 구르기 모두 길어진다), 도토는 뒤로 뛰기(A 길게·B 짧게).
    pub fn burrow_cd_mul(self) -> f32 {
        match (self.who, self.side(Slot::Burrow)) {
            (Character::Mole, 1) => t::TAL_DEEP_CD,
            (Character::Mole, 2) => t::TAL_SHALLOW_CD,
            (Character::Momo, 1) => t::TAL_LEAP_CD,
            (Character::Momo, 2) => t::TAL_STAMP_CD,
            (Character::Doto, 1) => t::TAL_DROP_CD,
            (Character::Doto, 2) => t::TAL_LIGHT_CD,
            _ => 1.0,
        }
    }
    pub fn emerge_radius(self) -> f32 {
        match self.mole(Slot::Burrow) {
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
        match self.mole(Slot::Drill) {
            1 => t::TAL_LONG_DRILL,
            2 => t::TAL_SHORT_DRILL,
            _ => 1.0,
        }
    }
    /// 게이지 차는 배율: Naïl은 드릴 갈래, 모모는 네 번 뛰기, 도토는 한가득 묻기(느리게 찬다).
    pub fn sp_gain_mul(self) -> f32 {
        match (self.who, self.side(Slot::Drill)) {
            (Character::Mole, 1) => t::TAL_LONG_DRILL_SP,
            (Character::Mole, 2) => t::TAL_SHORT_DRILL_SP,
            (Character::Momo, 1) => t::TAL_FOUR_HOPS_SP,
            (Character::Doto, 1) => t::TAL_HEAP_SP,
            _ => 1.0,
        }
    }

    /// 모모 깡충 A 멀리뛰기: 더 멀리 뛴다(대기가 길다).
    pub fn hop_dist(self) -> f32 {
        if self.momo_side(Slot::Burrow) == 1 {
            t::TAL_LEAP_DIST
        } else {
            t::HOP_DIST
        }
    }
    /// 모모 깡충 B 발 구르기: 착지에 작은 밟기(회피가 공격도 된다, 0090의 모모 쪽 답).
    pub fn hop_stamp(self) -> bool {
        self.momo_side(Slot::Burrow) == 2
    }

    /// 모모 이어 뛰기: A 네 번 뛰기, B 높이 뛰기(두 번, 넓고 오래 멈추게).
    pub fn chain_hops(self) -> u32 {
        match self.momo_side(Slot::Drill) {
            1 => t::TAL_FOUR_HOPS,
            2 => t::TAL_HIGH_HOPS,
            _ => t::CHAIN_HOPS,
        }
    }
    pub fn chain_radius(self) -> f32 {
        if self.momo_side(Slot::Drill) == 2 {
            t::TAL_HIGH_RADIUS
        } else {
            t::CHAIN_RADIUS
        }
    }
    pub fn chain_stun(self) -> f32 {
        if self.momo_side(Slot::Drill) == 2 {
            t::TAL_HIGH_STUN
        } else {
            t::CHAIN_STUN
        }
    }

    /// 도토 뒤로 뛰기(0195): A 떨구고 뛰기 — 뛰어오른 자리에 도토리 하나를 떨군다(대기가 길다).
    /// B 가벼운 발 — 짧게 뛰는 대신 금방 다시.
    pub fn leap_drop(self) -> bool {
        self.doto_side(Slot::Burrow) == 1
    }
    pub fn leap_dist(self) -> f32 {
        if self.doto_side(Slot::Burrow) == 2 {
            t::BACKLEAP_DIST * t::TAL_LIGHT_DIST
        } else {
            t::BACKLEAP_DIST
        }
    }

    /// 도토 도토리 묻기(0195): A 한가득 — 다섯(게이지가 느리게 찬다). B 빨리 터지기 — 금방 터지지만 둘레가 작다.
    pub fn bury_count(self) -> usize {
        if self.doto_side(Slot::Drill) == 1 {
            t::TAL_HEAP_COUNT
        } else {
            t::BURY_COUNT
        }
    }
    pub fn bury_fuse(self) -> f32 {
        if self.doto_side(Slot::Drill) == 2 {
            t::TAL_QUICK_FUSE
        } else {
            t::BURY_FUSE
        }
    }
    pub fn bury_radius(self) -> f32 {
        if self.doto_side(Slot::Drill) == 2 {
            t::TAL_QUICK_RADIUS
        } else {
            t::BURY_RADIUS
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

// ---- 큰 나무 갈래 셋 × 깊이 여덟(0246) -----------------------------------------

/// 깨우면 늘 켜지는 덤. 수치는 `apply`(장신구와 같은 `Mods`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boon {
    /// 최대 체력 +1.
    Hp,
    /// 산딸기 회복 +1.
    Berry,
    /// 맞힌 적이 조금 더 오래 멈춘다(+0.1초).
    Stun,
    /// 실뿌리 하나 더(0210).
    Lives,
    /// 마무리 타가 20% 더 멀리 민다.
    Knock,
    /// 공격 8% 빨리.
    Haste,
    /// 박자 창 25% 넓게.
    Beat,
    /// 강공격 모으기 20% 빨리.
    Charge,
    /// 게이지 15% 빨리.
    Ki,
    /// 모든 타 피해 +1.
    Edge,
    /// 걷기 6% 빨리.
    Speed,
    /// 회피 대기 12% 짧게.
    Dodge,
    /// 쉼터 가게 값 15% 싸게.
    Bargain,
    /// 장신구 칸 +1.
    Charm,
}

pub const BOONS: [Boon; 14] = [
    Boon::Hp,
    Boon::Berry,
    Boon::Stun,
    Boon::Lives,
    Boon::Knock,
    Boon::Haste,
    Boon::Beat,
    Boon::Charge,
    Boon::Ki,
    Boon::Edge,
    Boon::Speed,
    Boon::Dodge,
    Boon::Bargain,
    Boon::Charm,
];

impl Boon {
    pub fn index(self) -> usize {
        BOONS.iter().position(|b| *b == self).unwrap_or(0)
    }

    fn apply(self, m: &mut Mods) {
        match self {
            Boon::Hp => m.max_hp += 1,
            Boon::Berry => m.berry_heal += 1,
            Boon::Stun => m.stun += t::TREE_STUN,
            Boon::Lives => m.lives += 1,
            Boon::Knock => m.finisher_knockback *= t::TREE_KNOCK,
            Boon::Haste => m.attack_speed *= t::TREE_HASTE,
            Boon::Beat => m.beat *= t::TREE_BEAT,
            Boon::Charge => m.charge *= t::TREE_CHARGE,
            Boon::Ki => m.sp_gain *= t::TREE_KI,
            Boon::Edge => m.damage += 1,
            Boon::Speed => m.speed *= t::TREE_SPEED,
            Boon::Dodge => m.burrow_cd *= t::TREE_DODGE,
            Boon::Bargain => m.bargain *= t::TREE_BARGAIN,
            Boon::Charm => m.charm_slots += 1,
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Boon::Hp => |s| s.bn_hp,
            Boon::Berry => |s| s.bn_berry,
            Boon::Stun => |s| s.bn_stun,
            Boon::Lives => |s| s.bn_lives,
            Boon::Knock => |s| s.bn_knock,
            Boon::Haste => |s| s.bn_haste,
            Boon::Beat => |s| s.bn_beat,
            Boon::Charge => |s| s.bn_charge,
            Boon::Ki => |s| s.bn_ki,
            Boon::Edge => |s| s.bn_edge,
            Boon::Speed => |s| s.bn_speed,
            Boon::Dodge => |s| s.bn_dodge,
            Boon::Bargain => |s| s.bn_bargain,
            Boon::Charm => |s| s.bn_charm,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Boon::Hp => |s| s.bn_hp_d,
            Boon::Berry => |s| s.bn_berry_d,
            Boon::Stun => |s| s.bn_stun_d,
            Boon::Lives => |s| s.bn_lives_d,
            Boon::Knock => |s| s.bn_knock_d,
            Boon::Haste => |s| s.bn_haste_d,
            Boon::Beat => |s| s.bn_beat_d,
            Boon::Charge => |s| s.bn_charge_d,
            Boon::Ki => |s| s.bn_ki_d,
            Boon::Edge => |s| s.bn_edge_d,
            Boon::Speed => |s| s.bn_speed_d,
            Boon::Dodge => |s| s.bn_dodge_d,
            Boon::Bargain => |s| s.bn_bargain_d,
            Boon::Charm => |s| s.bn_charm_d,
        }
    }
}

/// 갈래의 한 칸.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    /// 두 갈래 고르기(위의 자리, 그 캐릭터의 표).
    Pick(Slot),
    Plus(Boon),
}

pub const DEPTH: usize = 8;

/// 갈래 셋(모든 캐릭터 같은 꼴, 고르기 칸만 그 캐릭터의 표). 이름은 큰 나무의 뿌리·가지·잎.
pub const BRANCHES: [(Tr, [Node; DEPTH]); 3] = [
    (
        |s| s.tb_root,
        [
            Node::Pick(Slot::Parry),
            Node::Plus(Boon::Hp),
            Node::Pick(Slot::Eat),
            Node::Plus(Boon::Berry),
            Node::Plus(Boon::Hp),
            Node::Plus(Boon::Stun),
            Node::Plus(Boon::Lives),
            Node::Plus(Boon::Hp),
        ],
    ),
    (
        |s| s.tb_branch,
        [
            Node::Pick(Slot::Drill),
            Node::Plus(Boon::Knock),
            Node::Plus(Boon::Haste),
            Node::Plus(Boon::Beat),
            Node::Plus(Boon::Charge),
            Node::Plus(Boon::Ki),
            Node::Plus(Boon::Haste),
            Node::Plus(Boon::Edge),
        ],
    ),
    (
        |s| s.tb_leaf,
        [
            Node::Pick(Slot::Burrow),
            Node::Plus(Boon::Speed),
            Node::Plus(Boon::Dodge),
            Node::Plus(Boon::Bargain),
            Node::Plus(Boon::Speed),
            Node::Plus(Boon::Dodge),
            Node::Plus(Boon::Bargain),
            Node::Plus(Boon::Charm),
        ],
    ),
];

/// 덤 칸의 값(반딧불): 깊을수록 비싸다. 고르기 칸은 제 표의 값.
pub const fn plus_cost(depth: usize) -> u32 {
    t::TREE_COST[depth] * t::TALENT_PRICE_MUL
}

/// 덤 칸의 기록 키(바꾸지 않는다, 0041).
pub fn key_plus(ch: Character, branch: usize, depth: usize) -> String {
    format!("tree.{ch:?}.{branch}.{depth}")
}

/// 같은 갈래 앞쪽에 같은 덤이 몇 번 나왔나(이름 뒤 II·III).
pub fn plus_rank(branch: usize, depth: usize) -> usize {
    let Node::Plus(b) = BRANCHES[branch].1[depth] else {
        return 0;
    };
    BRANCHES[branch].1[..depth]
        .iter()
        .filter(|n| **n == Node::Plus(b))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(slot: Slot, side: u8) -> Talents {
        let mut k = Talents::default();
        k.pick[slot.index()] = side;
        k
    }

    fn momo(slot: Slot, side: u8) -> Talents {
        Talents {
            who: Character::Momo,
            ..with(slot, side)
        }
    }

    fn doto(slot: Slot, side: u8) -> Talents {
        Talents {
            who: Character::Doto,
            ..with(slot, side)
        }
    }

    /// 모모(0142): 자리 1·3은 모모 것. 갈래마다 맞바꿈이 있고, Naïl의 굴파기·드릴 수치는 먹지 않는다.
    #[test]
    fn momo_sides_trade_and_ignore_mole_numbers() {
        let base = Talents {
            who: Character::Momo,
            ..Talents::default()
        };
        assert_eq!(base.hop_dist(), t::HOP_DIST);
        assert_eq!(base.chain_hops(), t::CHAIN_HOPS);
        let leap = momo(Slot::Burrow, 1);
        assert!(leap.hop_dist() > t::HOP_DIST && leap.burrow_cd_mul() > 1.0);
        assert_eq!(leap.emerge_radius(), t::EMERGE_RADIUS);
        let stamp = momo(Slot::Burrow, 2);
        assert!(
            stamp.hop_stamp() && stamp.burrow_cd_mul() > 1.0 && stamp.hop_dist() == t::HOP_DIST
        );
        let four = momo(Slot::Drill, 1);
        assert!(four.chain_hops() > t::CHAIN_HOPS && four.sp_gain_mul() < 1.0);
        assert_eq!(four.drill_mul(), 1.0);
        let high = momo(Slot::Drill, 2);
        assert!(high.chain_hops() < t::CHAIN_HOPS);
        assert!(high.chain_radius() > t::CHAIN_RADIUS && high.chain_stun() > t::CHAIN_STUN);
        // Naïl에게 모모 갈래는 없다.
        assert!(!with(Slot::Burrow, 2).hop_stamp());
        assert_eq!(with(Slot::Drill, 1).chain_hops(), t::CHAIN_HOPS);
    }

    /// 도토(0195): 자리 1·3은 도토 것. 갈래마다 맞바꿈, 다른 캐릭터의 수치는 먹지 않는다.
    #[test]
    fn doto_sides_trade_and_stay_their_own() {
        let base = doto(Slot::Burrow, 0);
        assert_eq!(base.leap_dist(), t::BACKLEAP_DIST);
        assert_eq!(base.bury_count(), t::BURY_COUNT);
        let drop = doto(Slot::Burrow, 1);
        assert!(drop.leap_drop() && drop.burrow_cd_mul() > 1.0);
        assert_eq!(drop.emerge_radius(), t::EMERGE_RADIUS);
        assert_eq!(drop.hop_dist(), t::HOP_DIST);
        let light = doto(Slot::Burrow, 2);
        assert!(light.leap_dist() < t::BACKLEAP_DIST && light.burrow_cd_mul() < 1.0);
        let heap = doto(Slot::Drill, 1);
        assert!(heap.bury_count() > t::BURY_COUNT && heap.sp_gain_mul() < 1.0);
        assert_eq!(heap.drill_mul(), 1.0);
        assert_eq!(heap.chain_hops(), t::CHAIN_HOPS);
        let quick = doto(Slot::Drill, 2);
        assert!(quick.bury_fuse() < t::BURY_FUSE && quick.bury_radius() < t::BURY_RADIUS);
        // 다른 캐릭터에게 도토 갈래는 없다.
        assert!(!with(Slot::Burrow, 1).leap_drop());
        assert!(!momo(Slot::Burrow, 1).leap_drop());
        assert_eq!(momo(Slot::Drill, 1).bury_count(), t::BURY_COUNT);
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
        assert_eq!(key_awake(Character::Mole, Slot::Drill, 1), "talent.drill.b");
        assert_eq!(key_pick(Character::Mole, Slot::Eat), "talent_pick.eat");
        assert_eq!(key_awake(Character::Momo, Slot::Burrow, 0), "talent.hop.a");
        assert_eq!(key_pick(Character::Momo, Slot::Drill), "talent_pick.chain");
        // 패링·먹기는 함께.
        assert_eq!(key_pick(Character::Momo, Slot::Parry), "talent_pick.parry");
        assert_eq!(key_awake(Character::Doto, Slot::Burrow, 1), "talent.leap.b");
        assert_eq!(key_pick(Character::Doto, Slot::Drill), "talent_pick.bury");
        assert_eq!(all_awake_keys().len(), 16 + 3 * 20);
        assert_eq!(key_plus(Character::Doto, 2, 7), "tree.Doto.2.7");
    }

    /// 큰 나무(0246): 갈래 셋 × 깊이 여덟, 옛 고르기 자리 넷이 한 번씩, 덤 값은 깊을수록 오른다.
    #[test]
    fn three_branches_eight_deep_keep_the_old_picks() {
        for (_, nodes) in BRANCHES.iter() {
            assert_eq!(nodes.len(), DEPTH);
            assert!(
                matches!(nodes[0], Node::Pick(_)),
                "갈래 첫 칸은 고르기(옛 깨움이 바로 이어진다)"
            );
        }
        for slot in SLOTS {
            let n = BRANCHES
                .iter()
                .flat_map(|(_, nodes)| nodes.iter())
                .filter(|n| **n == Node::Pick(slot))
                .count();
            assert_eq!(n, 1, "{slot:?}");
        }
        for d in 2..DEPTH {
            assert!(plus_cost(d) > plus_cost(d - 1));
        }
        assert_eq!(plus_rank(0, 4), 1, "뿌리의 둘째 질긴 가죽 = II");
        assert_eq!(plus_rank(0, 7), 2);
    }

    /// 깨운 덤은 `Mods`로 모인다: 체력은 더하기, 배율은 곱.
    #[test]
    fn boons_add_up_into_mods() {
        let mut k = Talents::default();
        assert_eq!(
            crate::equipment::Stats::compute(&[], 0, &k.mods()),
            crate::equipment::Stats::BASE
        );
        k.boons[Boon::Hp.index()] = 3;
        k.boons[Boon::Haste.index()] = 2;
        k.boons[Boon::Charm.index()] = 1;
        let m = k.mods();
        assert_eq!(m.max_hp, 3);
        assert!((m.attack_speed - t::TREE_HASTE * t::TREE_HASTE).abs() < 1e-5);
        let st = crate::equipment::Stats::compute(&[], 0, &m);
        assert_eq!(st.charm_slots(), crate::equipment::CHARM_SLOTS + 1);
        assert_eq!(st.max_hp, t::PLAYER_HP + 3);
    }
}
