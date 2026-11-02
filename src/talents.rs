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
use crate::moveset::{Hit, Throw};
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

/// 자리 2·4(받아치기·먹기)도 캐릭터마다 따로(0263, 사장 "각자 어울리는 재주 셋으로 서로 공유하지 않아야") — 전에는
/// 세 캐릭터가 같은 표·같은 키(0142)였다. Naïl은 옛 것 그대로(되받아치기·빛 모으기 / 꼭꼭 씹기·한입에).
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
/// 모모 받아차기: A 깡충 되감기(받아치면 회피 대기가 끝난다) · B 넘어뜨리기(받아친 적이 오래 멈춘다).
const MOMO_PARRY: (Tr, [Talent; 2]) = (
    |s| s.tal_parry_momo,
    [
        talent(|s| s.tal_parry_momo_a, |s| s.tal_parry_momo_a_desc, 60),
        talent(|s| s.tal_parry_momo_b, |s| s.tal_parry_momo_b_desc, 60),
    ],
);
/// 모모 풀 뜯기: A 먹고 깡충(다 먹으면 회피 대기가 끝난다) · B 풀잎 숨기(먹는 동안 맞지 않지만 오래 먹는다).
const MOMO_EAT: (Tr, [Talent; 2]) = (
    |s| s.tal_eat_momo,
    [
        talent(|s| s.tal_eat_momo_a, |s| s.tal_eat_momo_a_desc, 50),
        talent(|s| s.tal_eat_momo_b, |s| s.tal_eat_momo_b_desc, 50),
    ],
);
/// 도토 볼 받기: A 되던지기(받아치면 도토리 둘을 던진다) · B 다람쥐 눈(받아치기 창이 넓다).
const DOTO_PARRY: (Tr, [Talent; 2]) = (
    |s| s.tal_parry_doto,
    [
        talent(|s| s.tal_parry_doto_a, |s| s.tal_parry_doto_a_desc, 60),
        talent(|s| s.tal_parry_doto_b, |s| s.tal_parry_doto_b_desc, 60),
    ],
);
/// 도토 볼주머니: A 까먹기(다 먹으면 게이지가 찬다) · B 씨 뱉기(다 먹으면 도토리 셋을 뱉는다).
const DOTO_EAT: (Tr, [Talent; 2]) = (
    |s| s.tal_eat_doto,
    [
        talent(|s| s.tal_eat_doto_a, |s| s.tal_eat_doto_a_desc, 50),
        talent(|s| s.tal_eat_doto_b, |s| s.tal_eat_doto_b_desc, 50),
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
    MOMO_PARRY,
    (
        |s| s.tal_chain,
        [
            talent(|s| s.tal_chain_a, |s| s.tal_chain_a_desc, 80),
            talent(|s| s.tal_chain_b, |s| s.tal_chain_b_desc, 80),
        ],
    ),
    MOMO_EAT,
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
    DOTO_PARRY,
    (
        |s| s.tal_bury,
        [
            talent(|s| s.tal_bury_a, |s| s.tal_bury_a_desc, 80),
            talent(|s| s.tal_bury_b, |s| s.tal_bury_b_desc, 80),
        ],
    ),
    DOTO_EAT,
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
        for (b, (_, nodes)) in tree(ch).iter().enumerate() {
            for (d, n) in nodes.iter().enumerate() {
                match n {
                    Node::Plus(_) => keys.push(key_plus(ch, b, d)),
                    Node::Sig => keys.push(ch.def().signature.key().to_string()),
                    Node::Pick(_) => {}
                }
            }
        }
    }
    keys
}
pub const KEY_LIGHT: &str = "glowbugs_banked";

/// 0263 옮기기: 받아치기·먹기 자리는 세 캐릭터가 같은 키(`talent.parry.*`·`talent.eat.*`)였다. 이제 모모·도토는 제 키.
/// 이미 깨운 것은 그 캐릭터에 남는다(사장 "이미 깨운 재주는 캐릭터에 귀속") — 옛 키로 깬 것을 모모·도토의 같은 자리에도 깨워 두고,
/// 고른 쪽도 옮긴다. Naïl은 옛 키 그대로. 기록을 읽을 때 한 번(`profile::load_slot`).
pub fn migrate(p: &mut crate::profile::Profile) {
    const DONE: &str = "mig.0263";
    if p.flag(DONE) {
        return;
    }
    for (old, news) in [
        ("parry", ["momo_parry", "doto_parry"]),
        ("eat", ["momo_eat", "doto_eat"]),
    ] {
        for new in news {
            for side in ["a", "b"] {
                if p.flag(&format!("talent.{old}.{side}")) {
                    p.set(&format!("talent.{new}.{side}"));
                }
            }
            let pick = p.count(&format!("talent_pick.{old}"));
            if pick > 0 {
                p.set_count(&format!("talent_pick.{new}"), pick);
            }
        }
    }
    p.set(DONE);
}

/// 이번 런에 낀 재주. 자리마다 0 = 없음, 1 = A, 2 = B. 자리 1·3은 `who`의 표(0142·0195).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Talents {
    pub pick: [u8; 4],
    pub who: Character,
    /// 깨운 덤의 수(`Boon` 순서, 0246).
    pub boons: [u8; BOONS.len()],
    /// 고유 재주(`Node::Sig`, 0247)를 깨웠나. 무엇인지는 `who`의 표(`CharacterDef::signature`).
    pub sig: bool,
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

    /// 깨운 고유 재주(없으면 None).
    pub fn signature(&self) -> Option<Signature> {
        self.sig.then(|| self.who.def().signature)
    }

    /// 무기 한 타에 고유 재주를 얹은 타(0247 → 0248 R27). 플레이어는 이것만 본다 — 재주가 닿는 자리는 여기 하나.
    /// 독 발톱은 모든 타에 독(마무리·강타는 길게), 가시 발·갈라지는 열매는 마무리·강타에만. `finisher`: 콤보 마지막 타 또는 강타.
    pub fn with_signature(&self, mut h: Hit, finisher: bool) -> Hit {
        match self.signature() {
            Some(Signature::Venom) => {
                let p = if finisher {
                    t::VENOM_POISON_FINISHER
                } else {
                    t::VENOM_POISON
                };
                h.poison = h.poison.max(p);
            }
            Some(Signature::Thorns) if finisher && h.throw == Throw::None => h.thorns = true,
            Some(Signature::Split) if finisher => {
                if let Throw::Shot { split, .. } = &mut h.throw {
                    *split = (*split).max(t::SIG_SPLIT);
                }
            }
            _ => {}
        }
        // 튕기는 마무리(0267): 마무리 던지기가 벽에 한 번 튕긴다.
        if finisher && self.n(Boon::Ricochet) > 0 {
            if let Throw::Shot { bounce, .. } = &mut h.throw {
                *bounce = (*bounce).max(1);
            }
        }
        h
    }

    /// 여진(0267): 솟구친 자리에서 다시 울리는 피해(0 = 없음).
    pub fn aftershock(self) -> i32 {
        self.n(Boon::Aftershock) as i32
    }

    /// 흙 묻은 발톱(0267): 솟구친 뒤 발톱 피해 +1이 남는 시간(초, 0 = 없음).
    pub fn earth_claw_time(self) -> f32 {
        t::TREE_EARTH_CLAW * self.n(Boon::EarthClaw) as f32
    }

    /// 볼링 핀(0267): 부딪힌 적이 더 입는 피해와 날아가는 시간 배율.
    pub fn bowl_bonus(self) -> i32 {
        self.n(Boon::Pins) as i32
    }

    pub fn launch_mul(self) -> f32 {
        1.0 + t::TREE_PINS_LAUNCH * self.n(Boon::Pins) as f32
    }

    fn side(self, slot: Slot) -> u8 {
        self.pick[slot.index()]
    }

    /// 깨운 덤 `b`의 수.
    fn n(self, b: Boon) -> u32 {
        self.boons[b.index()] as u32
    }

    /// `k`를 `n`번 곱한 배율.
    fn pow(k: f32, n: u32) -> f32 {
        k.powi(n as i32)
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
        let base = match self.mole(Slot::Burrow) {
            1 => t::TAL_DEEP_TIME,
            2 => t::TAL_SHALLOW_TIME,
            _ => t::BURROW_MAX_TIME,
        };
        base + t::TREE_DEEP_BREATH * self.n(Boon::DeepBreath) as f32
    }
    /// 회피 대기 배율. 모모는 깡충(A 멀리뛰기·B 발 구르기 모두 길어진다), 도토는 뒤로 뛰기(A 길게·B 짧게).
    pub fn burrow_cd_mul(self) -> f32 {
        let quick = Self::pow(t::TREE_DODGE, self.n(Boon::QuickHop) + self.n(Boon::Nimble));
        quick
            * match (self.who, self.side(Slot::Burrow)) {
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
        let base = match self.mole(Slot::Burrow) {
            1 => t::TAL_DEEP_RADIUS,
            2 => t::TAL_SHALLOW_RADIUS,
            _ => t::EMERGE_RADIUS,
        };
        base + t::TREE_QUAKE * self.n(Boon::Quake) as f32
    }
    /// 솟구쳐 맞힌 적이 더 멈추는 시간(Naïl 덤 "땅울림", 0263).
    pub fn emerge_stun_add(self) -> f32 {
        t::TREE_TREMOR * self.n(Boon::Tremor) as f32
    }
    /// 드릴 피해 더하기(Naïl 덤 "드릴 날", 0263).
    pub fn drill_damage_add(self) -> i32 {
        self.n(Boon::DrillEdge) as i32
    }
    /// 큰 도토리(강타) 피해 더하기(도토 덤 "묵직한 도토리", 0263).
    pub fn heavy_damage_add(self) -> i32 {
        2 * self.n(Boon::BigNut) as i32
    }

    /// 패링 A 되받아치기: 튕겨낸 돌격에 피해(게이지는 안 찬다). B 빛 모으기: 게이지가 크게 찬다.
    /// Naïl만(0263). 덤 "빛 받기"는 받아칠 때마다 게이지를 더 채운다.
    pub fn parry_sp(self) -> f32 {
        let base = match self.mole(Slot::Parry) {
            1 => 0.0,
            2 => t::TAL_PARRY_SP,
            _ => t::PARRY_SP,
        };
        base + t::TREE_PARRY_LIGHT * self.n(Boon::ParryLight) as f32
    }
    pub fn parry_counter(self) -> i32 {
        if self.mole(Slot::Parry) == 1 {
            t::TAL_PARRY_COUNTER
        } else {
            0
        }
    }

    /// 모모 받아차기 A 깡충 되감기: 받아치면 회피 대기가 바로 끝난다(0263).
    pub fn parry_rewinds_dodge(self) -> bool {
        self.momo_side(Slot::Parry) == 1
    }
    /// 받아친 적이 멈추는 시간 배율: 모모 받아차기 B 넘어뜨리기(0263).
    pub fn parry_stun_mul(self) -> f32 {
        if self.momo_side(Slot::Parry) == 2 {
            t::TAL_TRIP_STUN
        } else {
            1.0
        }
    }
    /// 도토 볼 받기 A 되던지기: 받아치면 바라보는 쪽으로 도토리 둘(0263).
    pub fn parry_toss(self) -> bool {
        self.doto_side(Slot::Parry) == 1
    }
    /// 받아치기 창 배율: 도토 볼 받기 B 다람쥐 눈(0263).
    pub fn parry_window_mul(self) -> f32 {
        if self.doto_side(Slot::Parry) == 2 {
            t::TAL_SQUIRREL_EYE
        } else {
            1.0
        }
    }

    /// 드릴 A 긴 드릴: 길게 달리지만 게이지가 느리게 찬다. B 짧은 드릴: 짧게, 자주.
    pub fn drill_mul(self) -> f32 {
        let base = match self.mole(Slot::Drill) {
            1 => t::TAL_LONG_DRILL,
            2 => t::TAL_SHORT_DRILL,
            _ => 1.0,
        };
        base * (1.0 + t::TREE_LONG_BORE * self.n(Boon::LongBore) as f32)
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
        let base = if self.momo_side(Slot::Burrow) == 1 {
            t::TAL_LEAP_DIST
        } else {
            t::HOP_DIST
        };
        base * (1.0 + t::TREE_FAR * self.n(Boon::FarHop) as f32)
    }
    /// 멀리뛰기 거리(모모 덤 "용수철 다리", 0263). 도토는 늘 `JUMP_DIST`.
    pub fn jump_dist(self) -> f32 {
        t::JUMP_DIST * (1.0 + t::TREE_FAR * self.n(Boon::Spring) as f32)
    }
    /// 달리기 배율(도토 덤 "종종걸음", 0263).
    pub fn run_mul(self) -> f32 {
        1.0 + t::TREE_SCAMPER * self.n(Boon::Scamper) as f32
    }
    /// 모모 깡충 B 발 구르기: 착지에 작은 밟기(회피가 공격도 된다, 0090의 모모 쪽 답).
    pub fn hop_stamp(self) -> bool {
        self.momo_side(Slot::Burrow) == 2
    }

    /// 모모 이어 뛰기: A 네 번 뛰기, B 높이 뛰기(두 번, 넓고 오래 멈추게).
    pub fn chain_hops(self) -> u32 {
        let base = match self.momo_side(Slot::Drill) {
            1 => t::TAL_FOUR_HOPS,
            2 => t::TAL_HIGH_HOPS,
            _ => t::CHAIN_HOPS,
        };
        base + self.n(Boon::ExtraHop)
    }
    pub fn chain_radius(self) -> f32 {
        let base = if self.momo_side(Slot::Drill) == 2 {
            t::TAL_HIGH_RADIUS
        } else {
            t::CHAIN_RADIUS
        };
        base + t::TREE_STOMP * self.n(Boon::Stomp) as f32
    }
    pub fn chain_stun(self) -> f32 {
        let base = if self.momo_side(Slot::Drill) == 2 {
            t::TAL_HIGH_STUN
        } else {
            t::CHAIN_STUN
        };
        base + t::TREE_STUN * self.n(Boon::Daze) as f32
    }

    /// 도토 뒤로 뛰기(0195): A 떨구고 뛰기 — 뛰어오른 자리에 도토리 하나를 떨군다(대기가 길다).
    /// B 가벼운 발 — 짧게 뛰는 대신 금방 다시.
    pub fn leap_drop(self) -> bool {
        self.doto_side(Slot::Burrow) == 1
    }
    pub fn leap_dist(self) -> f32 {
        let base = if self.doto_side(Slot::Burrow) == 2 {
            t::BACKLEAP_DIST * t::TAL_LIGHT_DIST
        } else {
            t::BACKLEAP_DIST
        };
        base * (1.0 + t::TREE_FAR * self.n(Boon::Bound) as f32)
    }

    /// 도토 도토리 묻기(0195): A 한가득 — 다섯(게이지가 느리게 찬다). B 빨리 터지기 — 금방 터지지만 둘레가 작다.
    pub fn bury_count(self) -> usize {
        let base = if self.doto_side(Slot::Drill) == 1 {
            t::TAL_HEAP_COUNT
        } else {
            t::BURY_COUNT
        };
        base + self.n(Boon::Cache) as usize
    }
    pub fn bury_fuse(self) -> f32 {
        let base = if self.doto_side(Slot::Drill) == 2 {
            t::TAL_QUICK_FUSE
        } else {
            t::BURY_FUSE
        };
        base * Self::pow(t::TREE_FUSE, self.n(Boon::Fuse))
    }
    pub fn bury_radius(self) -> f32 {
        let base = if self.doto_side(Slot::Drill) == 2 {
            t::TAL_QUICK_RADIUS
        } else {
            t::BURY_RADIUS
        };
        base + t::TREE_BLAST * self.n(Boon::Blast) as f32
    }

    /// 먹기 A 꼭꼭 씹기: 오래 먹고 산딸기가 더 채운다. B 한입에: 빨리 먹는다.
    /// Naïl 꼭꼭 씹기·한입에, 모모 풀잎 숨기(오래 먹는다, 0263).
    pub fn eat_time_mul(self) -> f32 {
        match (self.mole(Slot::Eat), self.momo_side(Slot::Eat)) {
            (1, _) => t::TAL_CHEW_TIME,
            (2, _) => t::TAL_GULP_TIME,
            (_, 2) => t::TAL_HIDE_EAT_TIME,
            _ => 1.0,
        }
    }
    /// 모모 풀 뜯기 A 먹고 깡충: 다 먹으면 회피 대기가 끝난다(0263).
    pub fn eat_rewinds_dodge(self) -> bool {
        self.momo_side(Slot::Eat) == 1
    }
    /// 모모 풀 뜯기 B 풀잎 숨기: 먹는 동안 맞지 않는다(0263).
    pub fn eat_hidden(self) -> bool {
        self.momo_side(Slot::Eat) == 2
    }
    /// 도토 볼주머니 A 까먹기: 다 먹으면 게이지가 이만큼(0..1 비율) 찬다(0263).
    pub fn eat_ki(self) -> f32 {
        if self.doto_side(Slot::Eat) == 1 {
            t::TAL_SNACK_KI
        } else {
            0.0
        }
    }
    /// 도토 볼주머니 B 씨 뱉기: 다 먹으면 바라보는 쪽으로 도토리 셋(0263).
    pub fn eat_spit(self) -> bool {
        self.doto_side(Slot::Eat) == 2
    }
    pub fn berry_heal(self) -> i32 {
        if self.mole(Slot::Eat) == 1 {
            t::TAL_CHEW_HEAL
        } else {
            t::BERRY_HEAL
        }
    }
}

// ---- 큰 나무 갈래 셋 × 깊이 여덟(0246) -----------------------------------------

/// 깨우면 늘 켜지는 덤(0246 → 0263). 캐릭터마다 제 덤만 — 한 덤은 한 캐릭터의 나무에만 나온다(시험이 본다).
/// 체력·실뿌리·장신구 칸·가게 할인 같은 "누구나 같은" 덤은 뺐다(사장 "캐릭 기본 없이"). 수치는 `apply`(장신구와 같은
/// `Mods`)이거나 그 캐릭터의 동작 수치(`Talents`의 게터)다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boon {
    /// 땅속에 0.25초 더 오래(Naïl).
    DeepBreath,
    /// 솟구칠 때 둘레가 5px 넓게(Naïl).
    Quake,
    /// 솟구쳐 맞힌 적이 0.15초 더 멈춘다(Naïl).
    Tremor,
    /// 박자 창 25% 넓게(Naïl).
    BeatClaw,
    /// 발톱 피해 +1(Naïl).
    Edge,
    /// 드릴 피해 +1(Naïl).
    DrillEdge,
    /// 드릴이 15% 더 멀리(Naïl).
    LongBore,
    /// 게이지 15% 빨리(Naïl).
    Ki,
    /// 받아치면 게이지가 더 찬다(Naïl).
    ParryLight,
    /// 깡충 10% 더 멀리(모모).
    FarHop,
    /// 깡충 대기 12% 짧게(모모).
    QuickHop,
    /// 걷기 6% 빨리(모모).
    Fleet,
    /// 차기 8% 빨리(모모).
    Haste,
    /// 마무리 차기가 20% 더 멀리 민다(모모).
    Knock,
    /// 이어 뛰기 쿵의 둘레가 4px 넓게(모모).
    Stomp,
    /// 이어 뛰기 쿵에 맞은 적이 0.1초 더 멈춘다(모모).
    Daze,
    /// 이어 뛰기 한 번 더(모모).
    ExtraHop,
    /// 멀리뛰기 10% 더 멀리(모모).
    Spring,
    /// 큰 도토리 모으기 20% 빨리(도토).
    Charge,
    /// 맞힌 적이 0.1초 더 멈춘다(도토).
    Heft,
    /// 큰 도토리 피해 +2(도토).
    BigNut,
    /// 묻는 도토리 하나 더(도토).
    Cache,
    /// 묻은 도토리가 터지는 둘레 4px 넓게(도토).
    Blast,
    /// 묻은 도토리가 15% 빨리 터진다(도토).
    Fuse,
    /// 뒤로 뛰기 10% 더 멀리(도토).
    Bound,
    /// 뒤로 뛰기 대기 12% 짧게(도토).
    Nimble,
    /// 달리기 8% 빨리(도토).
    Scamper,
    /// 여진(0267, Naïl): 솟구친 자리에서 잠시 뒤 한 번 더 땅이 울린다(깨운 수만큼 피해).
    Aftershock,
    /// 흙 묻은 발톱(0267, Naïl): 솟구친 뒤 잠깐 발톱 피해 +1(깨운 수만큼 길게).
    EarthClaw,
    /// 볼링 핀(0267, 모모): 날려 보낸 적이 더 오래 날고 부딪힌 적이 더 아프다.
    Pins,
    /// 튕기는 마무리(0267, 도토): 마무리 던지기·큰 도토리가 벽에 한 번 튕긴다.
    Ricochet,
}

pub const BOONS: [Boon; 31] = [
    Boon::DeepBreath,
    Boon::Quake,
    Boon::Tremor,
    Boon::BeatClaw,
    Boon::Edge,
    Boon::DrillEdge,
    Boon::LongBore,
    Boon::Ki,
    Boon::ParryLight,
    Boon::FarHop,
    Boon::QuickHop,
    Boon::Fleet,
    Boon::Haste,
    Boon::Knock,
    Boon::Stomp,
    Boon::Daze,
    Boon::ExtraHop,
    Boon::Spring,
    Boon::Charge,
    Boon::Heft,
    Boon::BigNut,
    Boon::Cache,
    Boon::Blast,
    Boon::Fuse,
    Boon::Bound,
    Boon::Nimble,
    Boon::Scamper,
    Boon::Aftershock,
    Boon::EarthClaw,
    Boon::Pins,
    Boon::Ricochet,
];

impl Boon {
    pub fn index(self) -> usize {
        BOONS.iter().position(|b| *b == self).unwrap_or(0)
    }

    /// `Mods`로 가는 덤(나머지는 0 — 동작 수치는 `Talents`의 게터가 센다).
    pub fn apply(self, m: &mut Mods) {
        match self {
            Boon::BeatClaw => m.beat *= t::TREE_BEAT,
            Boon::Edge => m.damage += 1,
            Boon::Ki => m.sp_gain *= t::TREE_KI,
            Boon::Fleet => m.speed *= t::TREE_SPEED,
            Boon::Haste => m.attack_speed *= t::TREE_HASTE,
            Boon::Knock => m.finisher_knockback *= t::TREE_KNOCK,
            Boon::Charge => m.charge *= t::TREE_CHARGE,
            Boon::Heft => m.stun += t::TREE_STUN,
            _ => {}
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Boon::DeepBreath => |s| s.bn_deep_breath,
            Boon::Quake => |s| s.bn_quake,
            Boon::Tremor => |s| s.bn_tremor,
            Boon::BeatClaw => |s| s.bn_beat,
            Boon::Edge => |s| s.bn_edge,
            Boon::DrillEdge => |s| s.bn_drill_edge,
            Boon::LongBore => |s| s.bn_long_bore,
            Boon::Ki => |s| s.bn_ki,
            Boon::ParryLight => |s| s.bn_parry_light,
            Boon::FarHop => |s| s.bn_far_hop,
            Boon::QuickHop => |s| s.bn_quick_hop,
            Boon::Fleet => |s| s.bn_speed,
            Boon::Haste => |s| s.bn_haste,
            Boon::Knock => |s| s.bn_knock,
            Boon::Stomp => |s| s.bn_stomp,
            Boon::Daze => |s| s.bn_daze,
            Boon::ExtraHop => |s| s.bn_extra_hop,
            Boon::Spring => |s| s.bn_spring,
            Boon::Charge => |s| s.bn_charge,
            Boon::Heft => |s| s.bn_stun,
            Boon::BigNut => |s| s.bn_big_nut,
            Boon::Cache => |s| s.bn_cache,
            Boon::Blast => |s| s.bn_blast,
            Boon::Fuse => |s| s.bn_fuse,
            Boon::Bound => |s| s.bn_bound,
            Boon::Nimble => |s| s.bn_nimble,
            Boon::Scamper => |s| s.bn_scamper,
            Boon::Aftershock => |s| s.bn_aftershock,
            Boon::EarthClaw => |s| s.bn_earth_claw,
            Boon::Pins => |s| s.bn_pins,
            Boon::Ricochet => |s| s.bn_ricochet,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Boon::DeepBreath => |s| s.bn_deep_breath_d,
            Boon::Quake => |s| s.bn_quake_d,
            Boon::Tremor => |s| s.bn_tremor_d,
            Boon::BeatClaw => |s| s.bn_beat_d,
            Boon::Edge => |s| s.bn_edge_d,
            Boon::DrillEdge => |s| s.bn_drill_edge_d,
            Boon::LongBore => |s| s.bn_long_bore_d,
            Boon::Ki => |s| s.bn_ki_d,
            Boon::ParryLight => |s| s.bn_parry_light_d,
            Boon::FarHop => |s| s.bn_far_hop_d,
            Boon::QuickHop => |s| s.bn_quick_hop_d,
            Boon::Fleet => |s| s.bn_speed_d,
            Boon::Haste => |s| s.bn_haste_d,
            Boon::Knock => |s| s.bn_knock_d,
            Boon::Stomp => |s| s.bn_stomp_d,
            Boon::Daze => |s| s.bn_daze_d,
            Boon::ExtraHop => |s| s.bn_extra_hop_d,
            Boon::Spring => |s| s.bn_spring_d,
            Boon::Charge => |s| s.bn_charge_d,
            Boon::Heft => |s| s.bn_stun_d,
            Boon::BigNut => |s| s.bn_big_nut_d,
            Boon::Cache => |s| s.bn_cache_d,
            Boon::Blast => |s| s.bn_blast_d,
            Boon::Fuse => |s| s.bn_fuse_d,
            Boon::Bound => |s| s.bn_bound_d,
            Boon::Nimble => |s| s.bn_nimble_d,
            Boon::Scamper => |s| s.bn_scamper_d,
            Boon::Aftershock => |s| s.bn_aftershock_d,
            Boon::EarthClaw => |s| s.bn_earth_claw_d,
            Boon::Pins => |s| s.bn_pins_d,
            Boon::Ricochet => |s| s.bn_ricochet_d,
        }
    }
}

/// 캐릭터마다 하나인 고유 재주(0247): 같은 칸 자리(가지 갈래), 같은 값, 내용만 다르다. 예전의 "독 발톱 줄"(0052,
/// Naïl만·도토를 되찾아야)을 대신한다 — 캐릭터 사이 예외 없이 같은 꼴.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signature {
    /// Naïl 독 발톱: 어느 발톱이든 맞힌 적에게 독(마무리·강타는 길게).
    Venom,
    /// 모모 가시 발: 마무리 차기·강차기가 발 앞에 가시 덤불.
    Thorns,
    /// 도토 갈라지는 열매: 마무리 던지기·큰 도토리가 맞은 자리에서 조각으로.
    Split,
}

impl Signature {
    /// 기록 키(바꾸지 않는다, 0041). Naïl은 예전 독 발톱 깃발을 그대로 — 깨운 사람은 깬 채로.
    pub fn key(self) -> &'static str {
        match self {
            Signature::Venom => crate::profile::key::VENOM,
            Signature::Thorns => "sig.momo.thorns",
            Signature::Split => "sig.doto.split",
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Signature::Venom => |s| s.sig_venom,
            Signature::Thorns => |s| s.sig_thorns,
            Signature::Split => |s| s.sig_split,
        }
    }

    pub fn desc(self) -> Tr {
        match self {
            Signature::Venom => |s| s.sig_venom_d,
            Signature::Thorns => |s| s.sig_thorns_d,
            Signature::Split => |s| s.sig_split_d,
        }
    }
}

/// 갈래의 한 칸.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    /// 두 갈래 고르기(위의 자리, 그 캐릭터의 표).
    Pick(Slot),
    Plus(Boon),
    /// 그 캐릭터의 고유 재주(0247).
    Sig,
}

pub const DEPTH: usize = 8;

/// 한 캐릭터의 큰 나무: 갈래 셋(뿌리 = 몸, 가지 = 싸움, 잎 = 길) × 깊이 여덟. 고르기·고유 재주 칸의 자리는 세 나무가 같다
/// (칸 모양 `cells`가 하나 — 시험이 본다). 덤은 캐릭터마다 제 것(0263).
pub type Tree = [(Tr, [Node; DEPTH]); 3];
pub const BRANCH_COUNT: usize = 3;

/// Naïl(두더지): 땅속·솟구침·드릴·박자 발톱·받아치기.
pub const NAIL_TREE: Tree = [
    (
        |s| s.tb_root,
        [
            Node::Pick(Slot::Parry),
            Node::Plus(Boon::DeepBreath),
            Node::Pick(Slot::Eat),
            Node::Plus(Boon::ParryLight),
            Node::Plus(Boon::EarthClaw),
            Node::Plus(Boon::Tremor),
            Node::Plus(Boon::ParryLight),
            Node::Plus(Boon::Quake),
        ],
    ),
    (
        |s| s.tb_branch,
        [
            Node::Pick(Slot::Drill),
            Node::Plus(Boon::Aftershock),
            Node::Plus(Boon::DrillEdge),
            Node::Plus(Boon::Ki),
            Node::Plus(Boon::EarthClaw),
            Node::Plus(Boon::LongBore),
            Node::Sig,
            Node::Plus(Boon::Edge),
        ],
    ),
    (
        |s| s.tb_leaf,
        [
            Node::Pick(Slot::Burrow),
            Node::Plus(Boon::Quake),
            Node::Plus(Boon::Ki),
            Node::Plus(Boon::LongBore),
            Node::Plus(Boon::Tremor),
            Node::Plus(Boon::DrillEdge),
            Node::Plus(Boon::Aftershock),
            Node::Plus(Boon::DeepBreath),
        ],
    ),
];

/// 모모(굴토끼): 깡충·차기·이어 뛰기·멀리뛰기. 박자 귀는 모모의 것(0267: 발 구르는 토끼, 박자 무기 꽃잎 발목끈).
pub const MOMO_TREE: Tree = [
    (
        |s| s.tb_root,
        [
            Node::Pick(Slot::Parry),
            Node::Plus(Boon::Fleet),
            Node::Pick(Slot::Eat),
            Node::Plus(Boon::QuickHop),
            Node::Plus(Boon::BeatClaw),
            Node::Plus(Boon::FarHop),
            Node::Plus(Boon::QuickHop),
            Node::Plus(Boon::Spring),
        ],
    ),
    (
        |s| s.tb_branch,
        [
            Node::Pick(Slot::Drill),
            Node::Plus(Boon::Knock),
            Node::Plus(Boon::Haste),
            Node::Plus(Boon::Stomp),
            Node::Plus(Boon::Haste),
            Node::Plus(Boon::Daze),
            Node::Sig,
            Node::Plus(Boon::ExtraHop),
        ],
    ),
    (
        |s| s.tb_leaf,
        [
            Node::Pick(Slot::Burrow),
            Node::Plus(Boon::Spring),
            Node::Plus(Boon::FarHop),
            Node::Plus(Boon::Stomp),
            Node::Plus(Boon::Knock),
            Node::Plus(Boon::Pins),
            Node::Plus(Boon::BeatClaw),
            Node::Plus(Boon::Fleet),
        ],
    ),
];

/// 도토(다람쥐): 던지기·큰 도토리·묻기·뒤로 뛰기·달리기.
pub const DOTO_TREE: Tree = [
    (
        |s| s.tb_root,
        [
            Node::Pick(Slot::Parry),
            Node::Plus(Boon::Nimble),
            Node::Pick(Slot::Eat),
            Node::Plus(Boon::Bound),
            Node::Plus(Boon::Heft),
            Node::Plus(Boon::Nimble),
            Node::Plus(Boon::Bound),
            Node::Plus(Boon::Heft),
        ],
    ),
    (
        |s| s.tb_branch,
        [
            Node::Pick(Slot::Drill),
            Node::Plus(Boon::Charge),
            Node::Plus(Boon::BigNut),
            Node::Plus(Boon::Ricochet),
            Node::Plus(Boon::Charge),
            Node::Plus(Boon::BigNut),
            Node::Sig,
            Node::Plus(Boon::Cache),
        ],
    ),
    (
        |s| s.tb_leaf,
        [
            Node::Pick(Slot::Burrow),
            Node::Plus(Boon::Scamper),
            Node::Plus(Boon::Blast),
            Node::Plus(Boon::Fuse),
            Node::Plus(Boon::Scamper),
            Node::Plus(Boon::Blast),
            Node::Plus(Boon::Fuse),
            Node::Plus(Boon::Cache),
        ],
    ),
];

/// 캐릭터의 큰 나무.
pub fn tree(ch: Character) -> &'static Tree {
    match ch {
        Character::Mole => &NAIL_TREE,
        Character::Momo => &MOMO_TREE,
        Character::Doto => &DOTO_TREE,
    }
}

/// 덤 칸의 값(반딧불): 깊을수록 비싸다. 고르기 칸은 제 표의 값.
pub const fn plus_cost(depth: usize) -> u32 {
    t::TREE_COST[depth] * t::TALENT_PRICE_MUL
}

/// 덤 칸의 기록 키(바꾸지 않는다, 0041).
pub fn key_plus(ch: Character, branch: usize, depth: usize) -> String {
    format!("tree.{ch:?}.{branch}.{depth}")
}

/// 같은 갈래 앞쪽에 같은 덤이 몇 번 나왔나(이름 뒤 II·III).
pub fn plus_rank(ch: Character, branch: usize, depth: usize) -> usize {
    let nodes = &tree(ch)[branch].1;
    let Node::Plus(b) = nodes[depth] else {
        return 0;
    };
    nodes[..depth]
        .iter()
        .filter(|n| **n == Node::Plus(b))
        .count()
}

// ---- 큰 나무 셈(0248 R28): 기록만 보고 계산하는 순수한 쪽. 앱은 그리기·입력만 ----------------

/// 목표 칸의 기록 키(0247, 캐릭터마다).
pub fn goal_key(ch: Character) -> String {
    format!("tree_goal.{ch:?}")
}

/// 목표 칸 ↔ 기록의 수: 줄 × 16 + 칸 + 1, 0 = 없음.
pub fn goal_code(goal: Option<(usize, usize)>) -> u32 {
    goal.map_or(0, |(r, c)| (r * 16 + c + 1) as u32)
}

/// 한 칸의 내용(그리기·설명용). 이름은 `name` + 같은 덤의 몇째(`rank`, 0 = 처음 — II·III…).
pub struct NodeInfo {
    pub cost: u32,
    pub name: Tr,
    pub rank: usize,
    pub desc: Tr,
    pub awake: bool,
}

/// 한 캐릭터의 큰 나무를 기록(`Profile`)으로 본 것. 줄 = 갈래, 칸 = (깊이, 고르기 쪽).
pub struct TreeView<'a> {
    pub p: &'a crate::profile::Profile,
    pub ch: Character,
}

impl TreeView<'_> {
    /// 갈래 한 줄의 칸들: (깊이, 갈래 A/B). 고르기 칸은 두 칸, 덤·고유 재주는 한 칸.
    pub fn cells(branch: usize) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        // 칸 모양은 세 나무가 같다(고르기·고유 재주 자리, 시험이 본다).
        for (d, n) in NAIL_TREE[branch].1.iter().enumerate() {
            match n {
                Node::Pick(_) => v.extend([(d, 0), (d, 1)]),
                Node::Plus(_) | Node::Sig => v.push((d, 0)),
            }
        }
        v
    }

    pub fn width(row: usize) -> usize {
        Self::cells(row).len()
    }

    /// (줄, 칸) → (깊이, 쪽). 칸 번호가 넘치면 끝 칸.
    pub fn depth_side(row: usize, cell: usize) -> (usize, usize) {
        let cells = Self::cells(row);
        cells[cell.min(cells.len() - 1)]
    }

    /// 한 깊이가 깨었나(고르기 칸은 어느 쪽이든).
    pub fn awake(&self, branch: usize, depth: usize) -> bool {
        match tree(self.ch)[branch].1[depth] {
            Node::Pick(slot) => (0..2).any(|side| self.p.flag(&key_awake(self.ch, slot, side))),
            Node::Plus(_) => self.p.flag(&key_plus(self.ch, branch, depth)),
            Node::Sig => self.p.flag(self.ch.def().signature.key()),
        }
    }

    /// 지금 깨울 수 있는 깊이인가: 맨 앞이거나 앞 칸이 깨었다.
    pub fn open(&self, branch: usize, depth: usize) -> bool {
        depth == 0 || self.awake(branch, depth - 1)
    }

    /// 깨울 때 세우는 기록 깃발(덤·고유 재주 칸). 고르기 칸은 쪽마다라 없음(`key_awake`).
    pub fn key(&self, branch: usize, depth: usize) -> Option<String> {
        match tree(self.ch)[branch].1[depth] {
            Node::Pick(_) => None,
            Node::Plus(_) => Some(key_plus(self.ch, branch, depth)),
            Node::Sig => Some(self.ch.def().signature.key().to_string()),
        }
    }

    /// 칸 하나: 값·이름·설명·깨었나(고르기 칸은 그 쪽).
    pub fn node(&self, branch: usize, depth: usize, side: usize) -> NodeInfo {
        match tree(self.ch)[branch].1[depth] {
            Node::Pick(slot) => {
                let tal = &table(self.ch)[slot.index()].1[side];
                NodeInfo {
                    cost: tal.cost,
                    name: tal.name,
                    rank: 0,
                    desc: tal.desc,
                    awake: self.p.flag(&key_awake(self.ch, slot, side)),
                }
            }
            Node::Plus(b) => NodeInfo {
                cost: plus_cost(depth),
                name: b.name(),
                rank: plus_rank(self.ch, branch, depth),
                desc: b.desc(),
                awake: self.awake(branch, depth),
            },
            Node::Sig => {
                let sig = self.ch.def().signature;
                NodeInfo {
                    cost: plus_cost(depth),
                    name: sig.name(),
                    rank: 0,
                    desc: sig.desc(),
                    awake: self.awake(branch, depth),
                }
            }
        }
    }

    /// (줄, 칸)을 깨웠나.
    pub fn cell_awake(&self, row: usize, cell: usize) -> bool {
        let (d, side) = Self::depth_side(row, cell);
        self.node(row, d, side).awake
    }

    /// 목표 칸. 표가 바뀌어 없는 칸이면 없음.
    pub fn goal(&self) -> Option<(usize, usize)> {
        let v = self.p.count(&goal_key(self.ch));
        if v == 0 {
            return None;
        }
        let (r, c) = (((v - 1) / 16) as usize, ((v - 1) % 16) as usize);
        (r < BRANCH_COUNT && c < Self::width(r)).then_some((r, c))
    }

    /// 그 칸까지 아직 안 깬 칸 수와 드는 반딧불. 지나가는 고르기 깊이는 싼 쪽, 그 칸이면 그 쪽.
    pub fn path(&self, row: usize, cell: usize) -> (usize, u32) {
        let cells = Self::cells(row);
        let (d, side) = Self::depth_side(row, cell);
        let (mut n, mut cost) = (0, 0);
        for k in 0..=d {
            if k == d {
                let node = self.node(row, d, side);
                if !node.awake {
                    n += 1;
                    cost += node.cost;
                }
            } else if !self.awake(row, k) {
                n += 1;
                cost += cells
                    .iter()
                    .filter(|(cd, _)| *cd == k)
                    .map(|&(_, sd)| self.node(row, k, sd).cost)
                    .min()
                    .unwrap_or(0);
            }
        }
        (n, cost)
    }

    /// 칸이 목표(또는 고른 칸)까지 깨울 길 위에 있나: 같은 갈래, 그 깊이 이하, 아직 안 깸.
    /// 고르기 짝 중 하나가 깬 깊이는 지나간 깊이. 목표 칸 자신은 (안 깼으면) 늘 길.
    pub fn on_path(&self, row: usize, cell: usize, target: (usize, usize)) -> bool {
        let (tr_, tc) = target;
        if row != tr_ {
            return false;
        }
        let (d, _) = Self::depth_side(row, cell);
        let (td, _) = Self::depth_side(row, tc);
        if d > td || self.cell_awake(row, cell) {
            return false;
        }
        cell == tc || (d < td && !self.awake(row, d))
    }

    /// 가진 반딧불로 지금 깨울 수 있는 칸이 있나.
    pub fn affordable(&self, have: u32) -> bool {
        (0..BRANCH_COUNT).any(|b| {
            Self::cells(b).into_iter().any(|(d, side)| {
                let n = self.node(b, d, side);
                !n.awake && self.open(b, d) && n.cost <= have
            })
        })
    }

    /// 기록 → 이번 런에 낄 재주(깨운 갈래만, 0142·0246·0247).
    pub fn talents(&self) -> Talents {
        let ch = self.ch;
        let mut k = Talents {
            who: ch,
            ..Talents::default()
        };
        for (i, slot) in SLOTS.iter().enumerate() {
            let side = self.p.count(&key_pick(ch, *slot)).min(2) as u8;
            if side > 0 && self.p.flag(&key_awake(ch, *slot, side as usize - 1)) {
                k.pick[i] = side;
            }
        }
        // 갈래 셋 × 깊이 여덟의 덤(0246): 깨운 것은 늘 켜진다.
        for (b, (_, nodes)) in tree(ch).iter().enumerate() {
            for (d, n) in nodes.iter().enumerate() {
                if let Node::Plus(boon) = n {
                    if self.p.flag(&key_plus(ch, b, d)) {
                        k.boons[boon.index()] += 1;
                    }
                }
            }
        }
        // 고유 재주(0247): 캐릭터마다 한 칸, 같은 자리.
        k.sig = self.p.flag(ch.def().signature.key());
        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 큰 나무 셈(0248 R28): 길·합계·길 위·목표 번호·깨우기 한도.
    #[test]
    fn tree_view_paths_goals_and_affordability() {
        use crate::profile::Profile;
        let ch = Character::Mole;
        let mut p = Profile::default();
        // 가지 갈래(1): 0 고르기(드릴 둘), 1 넉백, 2 빠른 손 …, 6 고유 재주.
        let v = TreeView { p: &p, ch };
        let sig_cell = TreeView::cells(1)
            .iter()
            .position(|&(d, _)| NAIL_TREE[1].1[d] == Node::Sig)
            .unwrap();
        let (n, cost) = v.path(1, sig_cell);
        // 고르기 깊이 하나(싼 쪽) + 덤 다섯 + 고유 재주 = 일곱 칸.
        assert_eq!(n, 7);
        let drill = TABLE[2].1[0].cost.min(TABLE[2].1[1].cost);
        let plus: u32 = (1..=6).map(plus_cost).sum();
        assert_eq!(cost, drill + plus);
        // 앞 칸을 다 깨우면 그 칸만 남는다.
        p.set(&key_awake(ch, Slot::Drill, 0));
        for d in 1..6 {
            p.set(&key_plus(ch, 1, d));
        }
        let v = TreeView { p: &p, ch };
        assert_eq!(v.path(1, sig_cell), (1, plus_cost(6)));
        assert!(v.open(1, 6) && !v.awake(1, 6));
        assert!(v.affordable(plus_cost(6)) && !v.affordable(0));
        // 깬 칸은 길 위가 아니고, 목표 칸은 길 위.
        assert!(!v.on_path(1, 0, (1, sig_cell)));
        assert!(v.on_path(1, sig_cell, (1, sig_cell)));
        assert!(!v.on_path(0, 0, (1, sig_cell)), "다른 갈래");
        // 고유 재주를 깨우면 기록 키 = 그 캐릭터의 서명 키, 재주에 켜진다.
        assert_eq!(v.key(1, 6).as_deref(), Some(crate::profile::key::VENOM));
        p.set(crate::profile::key::VENOM);
        let v = TreeView { p: &p, ch };
        assert_eq!(v.path(1, sig_cell), (0, 0));
        assert!(v.talents().sig);
        // 목표 번호 왕복, 없는 칸은 없음.
        for g in [None, Some((0, 0)), Some((2, 8)), Some((1, sig_cell))] {
            p.set_count(&goal_key(ch), goal_code(g));
            assert_eq!(TreeView { p: &p, ch }.goal(), g);
        }
        p.set_count(&goal_key(ch), goal_code(Some((2, 15))));
        assert_eq!(TreeView { p: &p, ch }.goal(), None);
        p.set_count(&goal_key(ch), goal_code(Some((7, 0))));
        assert_eq!(TreeView { p: &p, ch }.goal(), None);
    }

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

    /// 고유 재주(0247): 세 캐릭터 모두 같은 칸 자리(가지 갈래 하나), 내용만 다르다. 깨우기 전엔 아무것도 없다.
    #[test]
    fn signature_is_one_cell_per_character_with_its_own_effect() {
        let sig_cells: Vec<(usize, usize)> = NAIL_TREE
            .iter()
            .enumerate()
            .flat_map(|(b, (_, nodes))| {
                nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n == Node::Sig)
                    .map(move |(d, _)| (b, d))
            })
            .collect();
        assert_eq!(sig_cells.len(), 1, "고유 재주 칸은 하나");
        let on = |who| Talents {
            who,
            sig: true,
            ..Talents::default()
        };
        let claw = crate::moveset::CLAWS.hits[0];
        let kick = crate::moveset::KICKS.hits[0];
        let acorn = crate::moveset::ACORNS.hits[0];
        let split = |h: Hit| match h.throw {
            Throw::Shot { split, .. } => split,
            Throw::None => 0,
        };
        let nail = on(Character::Mole);
        assert!(nail.with_signature(claw, false).poison > 0.0);
        assert!(nail.with_signature(claw, true).poison > nail.with_signature(claw, false).poison);
        let momo = on(Character::Momo);
        assert!(momo.with_signature(kick, true).thorns && !momo.with_signature(kick, false).thorns);
        assert_eq!(momo.with_signature(kick, true).poison, kick.poison);
        let doto = on(Character::Doto);
        assert!(split(doto.with_signature(acorn, true)) >= t::SIG_SPLIT);
        assert_eq!(split(doto.with_signature(acorn, false)), split(acorn));
        let off = Talents {
            who: Character::Momo,
            ..Talents::default()
        };
        assert!(off.signature().is_none() && !off.with_signature(kick, true).thorns);
        // 기록 키는 캐릭터마다 달라야 한다(Naïl은 예전 독 발톱 깃발 그대로).
        let keys: Vec<&str> = crate::characters::ALL
            .iter()
            .map(|d| d.signature.key())
            .collect();
        assert_eq!(keys[0], crate::profile::key::VENOM);
        assert!(keys.iter().enumerate().all(|(i, k)| !keys[..i].contains(k)));
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
        // 받아치기·먹기도 캐릭터마다(0263, 옛 키는 Naïl이 그대로).
        assert_eq!(
            key_pick(Character::Momo, Slot::Parry),
            "talent_pick.momo_parry"
        );
        assert_eq!(
            key_awake(Character::Doto, Slot::Eat, 0),
            "talent.doto_eat.a"
        );
        assert_eq!(key_awake(Character::Doto, Slot::Burrow, 1), "talent.leap.b");
        assert_eq!(key_pick(Character::Doto, Slot::Drill), "talent_pick.bury");
        assert_eq!(all_awake_keys().len(), 3 * 8 + 3 * 20);
        assert_eq!(key_plus(Character::Doto, 2, 7), "tree.Doto.2.7");
    }

    /// 큰 나무(0246 → 0263): 캐릭터마다 갈래 셋 × 깊이 여덟, 고르기 자리 넷이 한 번씩, 고르기·고유 재주 칸 자리는 세 나무가 같다
    /// (칸 모양 하나). 덤은 캐릭터마다 제 것 — 한 덤이 두 캐릭터의 나무에 나오지 않는다. 덤 값은 깊을수록 오른다.
    #[test]
    fn each_character_has_its_own_tree_of_the_same_shape() {
        let chars = [Character::Mole, Character::Momo, Character::Doto];
        for ch in chars {
            for (b, (_, nodes)) in tree(ch).iter().enumerate() {
                assert!(matches!(nodes[0], Node::Pick(_)), "갈래 첫 칸은 고르기");
                for (d, n) in nodes.iter().enumerate() {
                    let shape = NAIL_TREE[b].1[d];
                    assert_eq!(
                        std::mem::discriminant(n),
                        std::mem::discriminant(&shape),
                        "{ch:?} {b} {d}: 칸 모양이 같다"
                    );
                    if let (Node::Pick(x), Node::Pick(y)) = (n, shape) {
                        assert_eq!(*x, y);
                    }
                }
            }
            for slot in SLOTS {
                let n = tree(ch)
                    .iter()
                    .flat_map(|(_, nodes)| nodes.iter())
                    .filter(|n| **n == Node::Pick(slot))
                    .count();
                assert_eq!(n, 1, "{ch:?} {slot:?}");
            }
        }
        // 덤은 한 캐릭터만.
        for b in BOONS {
            let owners = chars
                .iter()
                .filter(|&&ch| {
                    tree(ch)
                        .iter()
                        .any(|(_, nodes)| nodes.contains(&Node::Plus(b)))
                })
                .count();
            assert_eq!(owners, 1, "{b:?}");
        }
        // 고르기 표도 캐릭터마다(받아치기·먹기까지 이름이 다르다).
        for slot in SLOTS {
            let names: Vec<String> = chars
                .iter()
                .map(|&ch| (table(ch)[slot.index()].0)(crate::i18n::Lang::Ko.strings()).to_string())
                .collect();
            assert!(
                names
                    .iter()
                    .enumerate()
                    .all(|(i, n)| !names[..i].contains(n)),
                "{slot:?} {names:?}"
            );
        }
        for d in 2..DEPTH {
            assert!(plus_cost(d) > plus_cost(d - 1));
        }
        assert_eq!(
            plus_rank(Character::Mole, 0, 6),
            1,
            "뿌리의 둘째 받아치기 빛 = II"
        );
    }

    /// 0267: 한 나무에서 같은 덤은 두 칸까지(셋·넷이 겹쳐 고를 맛이 없던 것), 동작을 바꾸는 덤이 캐릭터마다 있다.
    #[test]
    fn no_boon_fills_more_than_two_cells_of_a_tree() {
        for ch in [Character::Mole, Character::Momo, Character::Doto] {
            for b in BOONS {
                let n = tree(ch)
                    .iter()
                    .flat_map(|(_, nodes)| nodes.iter())
                    .filter(|&&nd| nd == Node::Plus(b))
                    .count();
                assert!(n <= 2, "{ch:?} {b:?} ×{n}");
            }
        }
        let has =
            |ch: Character, b: Boon| tree(ch).iter().any(|(_, ns)| ns.contains(&Node::Plus(b)));
        assert!(has(Character::Mole, Boon::Aftershock) && has(Character::Mole, Boon::EarthClaw));
        assert!(has(Character::Momo, Boon::Pins) && has(Character::Momo, Boon::BeatClaw));
        assert!(has(Character::Doto, Boon::Ricochet));
        assert!(!has(Character::Mole, Boon::BeatClaw), "박자 귀는 모모로");
    }

    /// 튕기는 마무리(0267): 마무리 던지기만 벽에 한 번 튕긴다. 볼링 핀은 볼링 피해·날아가는 시간을 늘린다.
    #[test]
    fn ricochet_bounces_only_the_finisher_and_pins_scale_the_bowl() {
        let mut tal = Talents::default();
        tal.boons[Boon::Ricochet.index()] = 1;
        let ms = &crate::moveset::ACORNS;
        let bounce = |h: Hit| match h.throw {
            Throw::Shot { bounce, .. } => bounce,
            Throw::None => 99,
        };
        assert_eq!(bounce(tal.with_signature(ms.hits[0], false)), 0);
        assert_eq!(bounce(tal.with_signature(ms.hits[2], true)), 1);
        assert_eq!(
            bounce(Talents::default().with_signature(ms.hits[2], true)),
            0
        );
        let mut pins = Talents::default();
        assert_eq!((pins.bowl_bonus(), pins.launch_mul()), (0, 1.0));
        pins.boons[Boon::Pins.index()] = 1;
        assert!(pins.bowl_bonus() == 1 && pins.launch_mul() > 1.0);
    }

    /// 0263 옮기기: 옛 받아치기·먹기 키로 깬 것은 모모·도토의 같은 자리에도 남고, 고른 쪽도 옮긴다. 한 번만.
    #[test]
    fn shared_parry_and_eat_keys_move_to_each_character() {
        let mut p = crate::profile::Profile::default();
        p.set("talent.parry.b");
        p.set_count("talent_pick.parry", 2);
        p.set("talent.eat.a");
        migrate(&mut p);
        for ch in [Character::Momo, Character::Doto] {
            assert!(p.flag(&key_awake(ch, Slot::Parry, 1)));
            assert_eq!(p.count(&key_pick(ch, Slot::Parry)), 2);
            assert!(p.flag(&key_awake(ch, Slot::Eat, 0)));
        }
        assert!(
            p.flag(&key_awake(Character::Mole, Slot::Parry, 1)),
            "Naïl은 옛 키"
        );
        // 옮긴 뒤 Naïl이 새로 깬 것은 다른 캐릭터에 번지지 않는다.
        p.set("talent.eat.b");
        migrate(&mut p);
        assert!(!p.flag(&key_awake(Character::Momo, Slot::Eat, 1)));
    }

    /// 깨운 덤은 `Mods`(모두 곱·더하기)와 캐릭터 동작 수치로 간다.
    #[test]
    fn boons_reach_mods_and_kit_numbers() {
        let mut k = Talents::default();
        assert_eq!(
            crate::equipment::Stats::compute(&[], 0, &k.mods()),
            crate::equipment::Stats::BASE
        );
        k.boons[Boon::Haste.index()] = 2;
        k.boons[Boon::Edge.index()] = 1;
        let m = k.mods();
        assert!((m.attack_speed - t::TREE_HASTE * t::TREE_HASTE).abs() < 1e-5);
        assert_eq!(m.damage, 1);
        let mut n = Talents::default();
        n.boons[Boon::DeepBreath.index()] = 2;
        n.boons[Boon::Quake.index()] = 1;
        n.boons[Boon::DrillEdge.index()] = 1;
        assert!((n.burrow_time() - (t::BURROW_MAX_TIME + 2.0 * t::TREE_DEEP_BREATH)).abs() < 1e-5);
        assert!(n.emerge_radius() > t::EMERGE_RADIUS && n.drill_damage_add() == 1);
        let mut m2 = Talents {
            who: Character::Momo,
            ..Talents::default()
        };
        m2.boons[Boon::ExtraHop.index()] = 1;
        m2.boons[Boon::Spring.index()] = 1;
        assert_eq!(m2.chain_hops(), t::CHAIN_HOPS + 1);
        assert!(m2.jump_dist() > t::JUMP_DIST);
        let mut d = Talents {
            who: Character::Doto,
            ..Talents::default()
        };
        d.boons[Boon::Cache.index()] = 2;
        d.boons[Boon::Fuse.index()] = 1;
        d.boons[Boon::BigNut.index()] = 1;
        assert_eq!(d.bury_count(), t::BURY_COUNT + 2);
        assert!(d.bury_fuse() < t::BURY_FUSE && d.heavy_damage_add() == 2);
    }

    /// 받아치기·먹기 고르기(0263): 모모·도토 것은 제 캐릭터에만, Naïl의 옛 수치는 다른 캐릭터가 먹지 않는다.
    #[test]
    fn parry_and_eat_picks_are_per_character() {
        assert!(momo(Slot::Parry, 1).parry_rewinds_dodge());
        assert!(momo(Slot::Parry, 2).parry_stun_mul() > 1.0);
        assert_eq!(
            momo(Slot::Parry, 2).parry_counter(),
            0,
            "Naïl 되받아치기 아님"
        );
        assert!(doto(Slot::Parry, 1).parry_toss());
        assert!(doto(Slot::Parry, 2).parry_window_mul() > 1.0);
        assert!(momo(Slot::Eat, 1).eat_rewinds_dodge());
        assert!(momo(Slot::Eat, 2).eat_hidden() && momo(Slot::Eat, 2).eat_time_mul() > 1.0);
        assert!(doto(Slot::Eat, 1).eat_ki() > 0.0);
        assert!(doto(Slot::Eat, 2).eat_spit());
        assert_eq!(
            doto(Slot::Eat, 1).berry_heal(),
            t::BERRY_HEAL,
            "Naïl 꼭꼭 씹기 아님"
        );
        assert!(!with(Slot::Parry, 1).parry_rewinds_dodge() && !with(Slot::Eat, 2).eat_spit());
    }
}
