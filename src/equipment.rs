//! 장비: 캐릭터마다 쓸 수 있는 무기 종류가 다르다(docs/decisions/0011).
//! 두더지 Naïl은 발톱만, 굴토끼 모모는 뒷발만(docs/decisions/0056). 캐릭터마다 회피·특기도 다르다(player.rs).
//! 다른 캐릭터(보스 → 동료 → 플레이어블, 0003)는 자기 무기 종류를 갖는다.
//!
//! 능력치(docs/decisions/0017): 기본값 + 장신구(2칸, 아무 캐릭터나) + 뿌리 심장(최대 체력 영구 +1).
//! 플레이어 코드는 `Stats`만 본다. 무기는 콤보 모양(moveset), 장신구는 수치 보정.

use macroquad::color::Color;

use crate::i18n::Tr;
use crate::moveset::{self, Moveset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Character {
    #[default]
    Mole,
    /// 돌아온 모모(0046) → 플레이어블(0056): 발차기 3연타, 회피 = 깡충(공중 무적), 특기 = 이어 뛰기(기 게이지).
    Momo,
    /// 돌아온 도토(0052) → 플레이어블(0195): 도토리 던지기 3타, 회피 = 뒤로 뛰기(공중 무적), 특기 = 도토리 묻기.
    Doto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Claw,
    Kick,
    /// 도토의 던지는 도토리(0195).
    Acorn,
}

impl Character {
    pub fn name(self) -> &'static str {
        (self.def().name)(crate::i18n::s())
    }

    pub fn weapon_kind(self) -> WeaponKind {
        self.def().weapon_kind
    }

    /// 필살기 게이지의 종류. 두더지는 분노(맞히면 찬다), 모모는 기(氣).
    pub fn gauge(self) -> Gauge {
        self.def().gauge
    }

    /// 처음 들고 시작하는 무기(WEAPONS 번호).
    pub fn starting_weapon(self) -> usize {
        self.def().starting_weapon
    }
}

/// 필살기 게이지. 채우는 규칙은 지금 모두 "맞히면 찬다"(tuning SP_*). 이름·색만 다르다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gauge {
    Rage,
    Ki,
    /// 도토(0195): 볼주머니에 모은 도토리.
    Acorn,
}

impl Gauge {
    pub fn name(self) -> Tr {
        match self {
            Gauge::Rage => |s| s.gauge_rage,
            Gauge::Ki => |s| s.gauge_ki,
            Gauge::Acorn => |s| s.gauge_acorn,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Gauge::Rage => Color::new(1.0, 0.55, 0.22, 1.0),
            Gauge::Ki => Color::new(0.45, 0.80, 1.0, 1.0),
            Gauge::Acorn => Color::new(0.90, 0.66, 0.30, 1.0),
        }
    }
}

pub struct Weapon {
    pub name: Tr,
    pub desc: Tr,
    pub kind: WeaponKind,
    pub moveset: &'static Moveset,
    /// 가방 칸 아이콘 색.
    pub color: Color,
    /// 가졌는지 남는 기록 깃발(0246). 있으면 그 캐릭터의 런마다 가방에 들고 간다. None = 처음부터 든 무기.
    pub key: Option<&'static str>,
    /// 길의 상자·쉼터 가게에서 나온다(아직 없으면, 0246).
    pub found_on_road: bool,
}

/// 새싹 발톱(수수가 만들어 준다, 0053). `WEAPONS` 번호.
pub const SPROUT_CLAW: usize = 0;
/// 바위 발톱(콩이가 준다, 0054).
pub const STONE_CLAW: usize = 2;
/// 독 발톱(예전 무기, 0052). 지금은 Naïl의 고유 재주라(0247) 어디서도 주지 않는다 — 번호만 남는다.
#[cfg(test)]
pub const VENOM_CLAW: usize = 3;
/// 도토의 도토리(0195).
pub const ACORN: usize = 5;
/// 모모 무기(0246): 가시덩굴 각반(길에서)·조약돌 덧신(콩이)·꽃잎 발목끈(저널).
/// 길에서 찾는 둘(`BRIAR`·`PINECONE`)은 표의 `found_on_road`로 고르니 이름은 시험만 쓴다.
#[cfg_attr(not(test), allow(dead_code))]
pub const BRIAR: usize = 6;
pub const PEBBLE: usize = 7;
pub const PETAL: usize = 8;
/// 도토 무기(0246): 솔방울(길에서)·호두(수수)·밤송이(저널).
#[cfg_attr(not(test), allow(dead_code))]
pub const PINECONE: usize = 9;
pub const WALNUT: usize = 10;
pub const CHESTNUT: usize = 11;
/// Naïl의 길 무기(0251): 딱정벌레 발톱. 표의 `found_on_road`로 고르니 이름은 시험만 쓴다.
#[cfg_attr(not(test), allow(dead_code))]
pub const BEETLE: usize = 12;

pub static WEAPONS: [Weapon; 13] = [
    Weapon {
        name: |s| s.w_sprout,
        desc: |s| s.w_sprout_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::SPROUT,
        color: Color::new(0.62, 0.86, 0.48, 1.0),
        key: Some(crate::profile::key::SPROUT),
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_mole,
        desc: |s| s.w_mole_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::CLAWS,
        color: Color::new(0.97, 0.93, 0.85, 1.0),
        key: None,
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_stone,
        desc: |s| s.w_stone_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::STONE,
        color: Color::new(0.66, 0.64, 0.62, 1.0),
        key: Some(crate::profile::key::STONE),
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_venom,
        desc: |s| s.w_venom_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::VENOM,
        color: Color::new(0.52, 0.86, 0.36, 1.0),
        // 독 발톱은 Naïl의 고유 재주가 됐다(0247): 기록 깃발(`weapon.venom`)은 그 재주 칸의 키. 이 줄은
        // 예전 저장(가방 번호)을 읽으려고 자리만 남는다 — 어디서도 주지 않는다.
        key: None,
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_kick,
        desc: |s| s.w_kick_desc,
        kind: WeaponKind::Kick,
        moveset: &moveset::KICKS,
        color: Color::new(1.0, 0.80, 0.86, 1.0),
        key: None,
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_acorn,
        desc: |s| s.w_acorn_desc,
        kind: WeaponKind::Acorn,
        moveset: &moveset::ACORNS,
        color: Color::new(0.86, 0.62, 0.32, 1.0),
        key: None,
        found_on_road: false,
    },
    // 모모·도토 무기 셋씩(0246).
    Weapon {
        name: |s| s.w_briar,
        desc: |s| s.w_briar_desc,
        kind: WeaponKind::Kick,
        moveset: &moveset::BRIAR,
        color: Color::new(0.56, 0.80, 0.42, 1.0),
        key: Some("weapon.briar"),
        found_on_road: true,
    },
    Weapon {
        name: |s| s.w_pebble,
        desc: |s| s.w_pebble_desc,
        kind: WeaponKind::Kick,
        moveset: &moveset::PEBBLE,
        color: Color::new(0.70, 0.68, 0.66, 1.0),
        key: Some("weapon.pebble"),
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_petal,
        desc: |s| s.w_petal_desc,
        kind: WeaponKind::Kick,
        moveset: &moveset::PETAL,
        color: Color::new(1.0, 0.62, 0.78, 1.0),
        key: Some("weapon.petal"),
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_pine,
        desc: |s| s.w_pine_desc,
        kind: WeaponKind::Acorn,
        moveset: &moveset::PINECONE,
        color: Color::new(0.60, 0.42, 0.24, 1.0),
        key: Some("weapon.pinecone"),
        found_on_road: true,
    },
    Weapon {
        name: |s| s.w_walnut,
        desc: |s| s.w_walnut_desc,
        kind: WeaponKind::Acorn,
        moveset: &moveset::WALNUT,
        color: Color::new(0.80, 0.68, 0.48, 1.0),
        key: Some("weapon.walnut"),
        found_on_road: false,
    },
    Weapon {
        name: |s| s.w_chestnut,
        desc: |s| s.w_chestnut_desc,
        kind: WeaponKind::Acorn,
        moveset: &moveset::CHESTNUT,
        color: Color::new(0.50, 0.30, 0.18, 1.0),
        key: Some("weapon.chestnut"),
        found_on_road: false,
    },
    // Naïl도 길에서 하나(0251, 사장 "나일도 길에서 하나 찾도록"). 번호는 끝에 붙인다(가방 저장이 번호라).
    Weapon {
        name: |s| s.w_beetle,
        desc: |s| s.w_beetle_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::BEETLE,
        color: Color::new(0.40, 0.66, 0.62, 1.0),
        key: Some("weapon.beetle"),
        found_on_road: true,
    },
];

/// 플레이어가 실제로 쓰는 수치. `Stats::compute`로만 만든다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub max_hp: i32,
    /// 발톱 한 타 피해에 더한다.
    pub damage: i32,
    /// 스윙 시간 배율의 역수(1.2 = 20% 빠름). 활성·캔슬·후딜레이가 같이 줄어든다.
    pub attack_speed: f32,
    /// 굴파기 쿨다운 배율(0.7 = 30% 짧음).
    pub burrow_cd: f32,
    /// SP 차는 배율.
    pub sp_gain: f32,
    /// 마지막 타(콤보 끝) 넉백 배율.
    pub finisher_knockback: f32,
    /// 모모 이어 뛰기 밟기 반지름에 더한다(px).
    pub chain_radius: f32,
    /// 실에 감기는 시간 배율(0165, 흰 실 매듭): 가장 깊은 곳의 당기기·토끼풀의 실.
    pub wrap_mul: f32,
    /// 도토(0195 → B3): 던진 도토리의 사거리 배율, 뒤로 뛰기 거리 배율, 큰 도토리·묻은 도토리가 터지는 둘레 배율.
    pub throw_reach: f32,
    pub leap_mul: f32,
    pub blast: f32,
    /// 0246(장신구 셋씩·큰 나무 재주): 강공격 모으는 시간 배율, 솟구치기 둘레 배율, 드릴 길이 배율, 박자 타의 게이지 배율,
    /// 모모 밟기 피해 더하기, 도토 묻는 도토리 수 더하기, 맞힌 적 경직 더하기(초), 박자 창 배율, 걷는 속도 배율,
    /// 산딸기 회복 더하기, 실뿌리 더하기, 가게 값 배율, 장신구 칸 더하기.
    pub charge: f32,
    pub emerge: f32,
    pub drill: f32,
    pub beat_sp: f32,
    pub stomp: i32,
    pub bury: i32,
    pub stun: f32,
    pub beat: f32,
    pub speed: f32,
    pub berry_heal: i32,
    pub lives: u32,
    pub bargain: f32,
    pub charm_slots: usize,
}

impl Stats {
    pub const BASE: Stats = Stats {
        max_hp: crate::tuning::PLAYER_HP,
        damage: 0,
        attack_speed: 1.0,
        burrow_cd: 1.0,
        sp_gain: 1.0,
        finisher_knockback: 1.0,
        chain_radius: 0.0,
        wrap_mul: 1.0,
        throw_reach: 1.0,
        leap_mul: 1.0,
        blast: 1.0,
        charge: 1.0,
        emerge: 1.0,
        drill: 1.0,
        beat_sp: 1.0,
        stomp: 0,
        bury: 0,
        stun: 0.0,
        beat: 1.0,
        speed: 1.0,
        berry_heal: 0,
        lives: 0,
        bargain: 1.0,
        charm_slots: 0,
    };

    /// 기본값에 장신구·큰 나무 재주(`talent`, 0246)·뿌리 심장을 더한다. 배율은 곱, 더하기는 합. 너무 극단으로 가지 않게 묶는다.
    pub fn compute(charms: &[usize], root_hearts: i32, talent: &Mods) -> Stats {
        let mut s = Stats::BASE;
        for m in charms
            .iter()
            .map(|&c| &CHARMS[c].mods)
            .chain(std::iter::once(talent))
        {
            s.max_hp += m.max_hp;
            s.damage += m.damage;
            s.attack_speed *= m.attack_speed;
            s.burrow_cd *= m.burrow_cd;
            s.sp_gain *= m.sp_gain;
            s.finisher_knockback *= m.finisher_knockback;
            s.chain_radius += m.chain_radius;
            s.wrap_mul *= m.wrap_mul;
            s.throw_reach *= m.throw_reach;
            s.leap_mul *= m.leap_mul;
            s.blast *= m.blast;
            s.charge *= m.charge;
            s.emerge *= m.emerge;
            s.drill *= m.drill;
            s.beat_sp *= m.beat_sp;
            s.stomp += m.stomp;
            s.bury += m.bury;
            s.stun += m.stun;
            s.beat *= m.beat;
            s.speed *= m.speed;
            s.berry_heal += m.berry_heal;
            s.lives += m.lives;
            s.bargain *= m.bargain;
            s.charm_slots += m.charm_slots;
        }
        s.max_hp = (s.max_hp + root_hearts.min(crate::tuning::ROOT_HEART_MAX))
            .clamp(1, crate::tuning::MAX_HP_CAP);
        s.attack_speed = s.attack_speed.clamp(0.6, 1.8);
        s.burrow_cd = s.burrow_cd.clamp(0.3, 2.0);
        s.charge = s.charge.clamp(0.4, 1.5);
        s.speed = s.speed.clamp(0.7, 1.4);
        s.bargain = s.bargain.clamp(0.4, 1.0);
        s
    }

    /// 낄 수 있는 장신구 수(기본 + 큰 나무, 0246).
    pub fn charm_slots(&self) -> usize {
        CHARM_SLOTS + self.charm_slots
    }
}

/// 장신구 하나의 보정. 기본은 "바꾸지 않음".
#[derive(Debug, Clone, Copy)]
pub struct Mods {
    pub max_hp: i32,
    pub damage: i32,
    pub attack_speed: f32,
    pub burrow_cd: f32,
    pub sp_gain: f32,
    pub finisher_knockback: f32,
    pub chain_radius: f32,
    pub wrap_mul: f32,
    pub throw_reach: f32,
    pub leap_mul: f32,
    pub blast: f32,
    /// 0246(장신구 셋씩·큰 나무 재주): 강공격 모으는 시간 배율, 솟구치기 둘레 배율, 드릴 길이 배율, 박자 타의 게이지 배율,
    /// 모모 밟기 피해 더하기, 도토 묻는 도토리 수 더하기, 맞힌 적 경직 더하기(초), 박자 창 배율, 걷는 속도 배율,
    /// 산딸기 회복 더하기, 실뿌리 더하기, 가게 값 배율, 장신구 칸 더하기.
    pub charge: f32,
    pub emerge: f32,
    pub drill: f32,
    pub beat_sp: f32,
    pub stomp: i32,
    pub bury: i32,
    pub stun: f32,
    pub beat: f32,
    pub speed: f32,
    pub berry_heal: i32,
    pub lives: u32,
    pub bargain: f32,
    pub charm_slots: usize,
}

pub const NO_MODS: Mods = Mods {
    max_hp: 0,
    damage: 0,
    attack_speed: 1.0,
    burrow_cd: 1.0,
    sp_gain: 1.0,
    finisher_knockback: 1.0,
    chain_radius: 0.0,
    wrap_mul: 1.0,
    throw_reach: 1.0,
    leap_mul: 1.0,
    blast: 1.0,
    charge: 1.0,
    emerge: 1.0,
    drill: 1.0,
    beat_sp: 1.0,
    stomp: 0,
    bury: 0,
    stun: 0.0,
    beat: 1.0,
    speed: 1.0,
    berry_heal: 0,
    lives: 0,
    bargain: 1.0,
    charm_slots: 0,
};

/// 장신구: 2칸. 얻는 곳은 상자·상점(맵 단계)·저널. 대부분 누구나 끼고, 모모 것(0142)·도토 것(0195)은 그 캐릭터 런에서만 나온다.
pub struct Charm {
    pub name: Tr,
    pub desc: Tr,
    pub mods: Mods,
    pub color: Color,
    /// 이 캐릭터의 런에서만 나온다(상자·가게·저널로 받은 것 모두, 0142). None = 누구나.
    pub only: Option<Character>,
    /// 저널로 연다(0048): 받기 전에는 상자·가게에 없다.
    pub journal: bool,
}

impl Charm {
    pub fn fits(&self, ch: Character) -> bool {
        self.only.is_none_or(|c| c == ch)
    }
}

/// 동시에 낄 수 있는 장신구 수.
pub const CHARM_SLOTS: usize = 2;
/// 들꽃 화관(저널 momo_deep의 보상).
pub const CROWN: usize = 10;
/// 흰 실 매듭(저널 "가족"의 보상, 0165).
pub const SILK_KNOT: usize = 11;
/// 단풍잎 목도리(저널 doto_deep의 보상).
pub const MAPLE_SCARF: usize = 14;

pub static CHARMS: [Charm; 24] = [
    Charm {
        name: |s| s.c_shell,
        desc: |s| s.c_shell_desc,
        mods: Mods {
            max_hp: 1,
            ..NO_MODS
        },
        color: Color::new(0.80, 0.62, 0.42, 1.0),
        only: None,
        journal: false,
    },
    Charm {
        name: |s| s.c_whisker,
        desc: |s| s.c_whisker_desc,
        mods: Mods {
            attack_speed: 1.2,
            ..NO_MODS
        },
        color: Color::new(0.86, 0.86, 0.92, 1.0),
        only: None,
        journal: false,
    },
    Charm {
        name: |s| s.c_soil,
        desc: |s| s.c_soil_desc,
        mods: Mods {
            burrow_cd: 0.6,
            ..NO_MODS
        },
        color: Color::new(0.50, 0.36, 0.26, 1.0),
        // 굴파기는 Naïl의 회피라 Naïl 런에서만(사장 2026-10-29: 도토에게 굴파기 장신구가 나왔다).
        only: Some(Character::Mole),
        journal: false,
    },
    Charm {
        name: |s| s.c_flint,
        desc: |s| s.c_flint_desc,
        mods: Mods {
            damage: 1,
            attack_speed: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.62, 0.66, 0.74, 1.0),
        only: None,
        journal: false,
    },
    Charm {
        name: |s| s.c_amulet,
        desc: |s| s.c_amulet_desc,
        mods: Mods {
            sp_gain: 1.5,
            ..NO_MODS
        },
        color: Color::new(0.95, 0.80, 0.35, 1.0),
        only: None,
        journal: false,
    },
    // 저널로 연다(0048). 옆으로: 얻는 만큼 잃는다.
    // 들쥐 방울(보리를 되찾으면): 분노가 빨리 차고 발톱이 조금 느리다.
    Charm {
        name: |s| s.c_bell,
        desc: |s| s.c_bell_desc,
        mods: Mods {
            sp_gain: 1.3,
            attack_speed: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.90, 0.74, 0.30, 1.0),
        only: None,
        journal: true,
    },
    // 할매의 실(가장 깊은 곳): 굴을 자주 파지만 분노가 느리다.
    Charm {
        name: |s| s.c_thread,
        desc: |s| s.c_thread_desc,
        mods: Mods {
            burrow_cd: 0.7,
            sp_gain: 0.8,
            ..NO_MODS
        },
        color: Color::new(0.90, 0.88, 0.96, 1.0),
        // 이것도 "굴파기" 장신구라 Naïl 런에서만.
        only: Some(Character::Mole),
        journal: true,
    },
    // 모모의 꽃잎(모모가 돌아오면): 체력 +2, 발톱이 느리다.
    Charm {
        name: |s| s.c_petal,
        desc: |s| s.c_petal_desc,
        mods: Mods {
            max_hp: 2,
            attack_speed: 0.85,
            ..NO_MODS
        },
        color: Color::new(1.0, 0.66, 0.77, 1.0),
        only: None,
        journal: true,
    },
    // 모모 런에서만(0142). 토끼풀 잎: 깡충이 금방 돌아오지만 몸이 약하다.
    Charm {
        name: |s| s.c_clover,
        desc: |s| s.c_clover_desc,
        mods: Mods {
            burrow_cd: 0.7,
            max_hp: -1,
            ..NO_MODS
        },
        color: Color::new(0.52, 0.80, 0.44, 1.0),
        only: Some(Character::Momo),
        journal: false,
    },
    // 솜꼬리: 뒷발 마지막 타가 멀리 날리지만 발차기가 조금 느리다.
    Charm {
        name: |s| s.c_cotton,
        desc: |s| s.c_cotton_desc,
        mods: Mods {
            finisher_knockback: 1.4,
            attack_speed: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.96, 0.95, 0.92, 1.0),
        only: Some(Character::Momo),
        journal: false,
    },
    // 들꽃 화관(저널 "모모와 가장 깊은 곳"): 이어 뛰기 밟기가 넓지만 기가 느리게 찬다.
    Charm {
        name: |s| s.c_crown,
        desc: |s| s.c_crown_desc,
        mods: Mods {
            chain_radius: 8.0,
            sp_gain: 0.85,
            ..NO_MODS
        },
        color: Color::new(0.98, 0.84, 0.48, 1.0),
        only: Some(Character::Momo),
        journal: true,
    },
    Charm {
        name: |s| s.c_knot,
        desc: |s| s.c_knot_desc,
        mods: Mods {
            wrap_mul: 0.5,
            ..NO_MODS
        },
        color: Color::new(0.94, 0.92, 0.99, 1.0),
        only: None,
        journal: true,
    },
    // 도토 런에서만(0195). 솔방울: 도토리가 멀리 날지만 던지기가 조금 느리다.
    Charm {
        name: |s| s.c_pinecone,
        desc: |s| s.c_pinecone_desc,
        mods: Mods {
            throw_reach: 1.3,
            attack_speed: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.58, 0.40, 0.24, 1.0),
        only: Some(Character::Doto),
        journal: false,
    },
    // 도토리 깍정이 모자: 뒤로 뛰기가 멀지만 몸이 약하다.
    Charm {
        name: |s| s.c_acorn_cap,
        desc: |s| s.c_acorn_cap_desc,
        mods: Mods {
            leap_mul: 1.3,
            max_hp: -1,
            ..NO_MODS
        },
        color: Color::new(0.70, 0.56, 0.34, 1.0),
        only: Some(Character::Doto),
        journal: false,
    },
    // 단풍잎 목도리(저널 "도토와 가장 깊은 곳"): 터지는 둘레가 넓지만 도토리 게이지가 느리게 찬다.
    Charm {
        name: |s| s.c_maple,
        desc: |s| s.c_maple_desc,
        mods: Mods {
            blast: 1.35,
            sp_gain: 0.85,
            ..NO_MODS
        },
        color: Color::new(0.92, 0.50, 0.24, 1.0),
        only: Some(Character::Doto),
        journal: true,
    },
    // Naïl 장신구 셋(0246). 흙 묻은 장갑: 강공격을 빨리 모으지만 마무리가 덜 민다.
    Charm {
        name: |s| s.c_glove,
        desc: |s| s.c_glove_desc,
        mods: Mods {
            charge: 0.7,
            finisher_knockback: 0.85,
            ..NO_MODS
        },
        color: Color::new(0.62, 0.46, 0.32, 1.0),
        only: Some(Character::Mole),
        journal: false,
    },
    // 지렁이 굴 지도: 솟구침 둘레가 넓지만 굴파기 대기가 길다.
    Charm {
        name: |s| s.c_wormmap,
        desc: |s| s.c_wormmap_desc,
        mods: Mods {
            emerge: 1.3,
            burrow_cd: 1.15,
            ..NO_MODS
        },
        color: Color::new(0.86, 0.58, 0.56, 1.0),
        only: Some(Character::Mole),
        journal: false,
    },
    // 두더지 앞니: 드릴이 길게 달리지만 분노가 느리게 찬다.
    Charm {
        name: |s| s.c_incisor,
        desc: |s| s.c_incisor_desc,
        mods: Mods {
            drill: 1.35,
            sp_gain: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.96, 0.92, 0.78, 1.0),
        only: Some(Character::Mole),
        journal: false,
    },
    // 모모 장신구 셋(0246). 민들레 솜: 깡충이 멀리 가지만 몸이 약하다.
    Charm {
        name: |s| s.c_dandelion,
        desc: |s| s.c_dandelion_desc,
        mods: Mods {
            leap_mul: 1.2,
            max_hp: -1,
            ..NO_MODS
        },
        color: Color::new(0.98, 0.94, 0.70, 1.0),
        only: Some(Character::Momo),
        journal: false,
    },
    // 아침 이슬: 박자에 맞춘 차기가 기를 두 배로 채우지만 차기가 조금 느리다.
    Charm {
        name: |s| s.c_dew,
        desc: |s| s.c_dew_desc,
        mods: Mods {
            beat_sp: 2.0,
            attack_speed: 0.92,
            ..NO_MODS
        },
        color: Color::new(0.66, 0.86, 0.98, 1.0),
        only: Some(Character::Momo),
        journal: false,
    },
    // 클로버 꽃반지: 이어 뛰기 밟기 피해 +1, 기는 느리게 찬다.
    Charm {
        name: |s| s.c_clover_ring,
        desc: |s| s.c_clover_ring_desc,
        mods: Mods {
            stomp: 1,
            sp_gain: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.60, 0.86, 0.52, 1.0),
        only: Some(Character::Momo),
        journal: false,
    },
    // 도토 장신구 셋(0246). 볼주머니: 도토리를 하나 더 묻지만 게이지가 느리게 찬다.
    Charm {
        name: |s| s.c_cheek,
        desc: |s| s.c_cheek_desc,
        mods: Mods {
            bury: 1,
            sp_gain: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.92, 0.72, 0.52, 1.0),
        only: Some(Character::Doto),
        journal: false,
    },
    // 꼬리털 빗: 뒤로 뛰기가 금방 돌아오지만 짧게 뛴다.
    Charm {
        name: |s| s.c_comb,
        desc: |s| s.c_comb_desc,
        mods: Mods {
            burrow_cd: 0.7,
            leap_mul: 0.85,
            ..NO_MODS
        },
        color: Color::new(0.80, 0.60, 0.42, 1.0),
        only: Some(Character::Doto),
        journal: false,
    },
    // 송진: 맞은 적이 더 오래 멈추지만 도토리가 덜 난다.
    Charm {
        name: |s| s.c_resin,
        desc: |s| s.c_resin_desc,
        mods: Mods {
            stun: 0.15,
            throw_reach: 0.9,
            ..NO_MODS
        },
        color: Color::new(0.96, 0.76, 0.30, 1.0),
        only: Some(Character::Doto),
        journal: false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// 규칙(docs/decisions/0009): 모든 무기(발톱·뒷발)의 마지막 타가 가장 세게 밀치고 가장 오래 멈춘다.
    #[test]
    fn finisher_is_strongest_for_every_weapon() {
        for w in &WEAPONS {
            let name = (w.name)(&crate::i18n::KO);
            let hits = w.moveset.hits;
            let last = hits[hits.len() - 1];
            for h in &hits[..hits.len() - 1] {
                assert!(last.knockback > h.knockback, "{name}");
                assert!(last.hitstop >= h.hitstop, "{name}");
            }
        }
    }

    /// 캐릭터마다 길에서 찾는 무기가 하나씩(0251): Naïl 딱정벌레 발톱(모든 타가 등딱지를 연다)·모모 가시덩굴 각반·도토 솔방울.
    #[test]
    fn each_character_finds_one_weapon_on_the_road() {
        for kind in [WeaponKind::Claw, WeaponKind::Kick, WeaponKind::Acorn] {
            let road: Vec<usize> = (0..WEAPONS.len())
                .filter(|&w| WEAPONS[w].found_on_road && WEAPONS[w].kind == kind)
                .collect();
            assert_eq!(road.len(), 1, "{kind:?}");
            assert!(WEAPONS[road[0]].key.is_some(), "얻으면 기록에 남는다");
        }
        assert!(WEAPONS[BEETLE].moveset.hits.iter().all(|h| h.crush));
        assert!(WEAPONS[BEETLE].found_on_road);
    }

    /// 독 발톱은 Naïl의 고유 재주가 됐다(0247): 기록 깃발로도, 길에서도 무기를 주지 않는다. 독은 재주가 발톱 타에 건다.
    #[test]
    fn venom_is_a_talent_not_a_weapon() {
        let w = &WEAPONS[VENOM_CLAW];
        assert!(w.key.is_none() && !w.found_on_road);
        let on = crate::talents::Talents {
            sig: true,
            ..Default::default()
        };
        let claw = moveset::CLAWS.hits[0];
        assert!(on.with_signature(claw, false).poison > 0.0);
        assert!(on.with_signature(claw, true).poison > on.with_signature(claw, false).poison);
        assert_eq!(
            crate::talents::Talents::default()
                .with_signature(claw, true)
                .poison,
            claw.poison
        );
    }

    /// 캐릭터의 첫 무기는 그 캐릭터가 쓰는 종류다(0056).
    #[test]
    fn starting_weapon_matches_character() {
        for ch in [Character::Mole, Character::Momo, Character::Doto] {
            assert_eq!(WEAPONS[ch.starting_weapon()].kind, ch.weapon_kind());
        }
    }

    #[test]
    fn stats_combine_and_clamp() {
        assert_eq!(Stats::compute(&[], 0, &NO_MODS), Stats::BASE);
        let s = Stats::compute(&[0, 3], 2, &NO_MODS);
        assert_eq!(s.max_hp, Stats::BASE.max_hp + 1 + 2);
        assert_eq!(s.damage, 1);
        assert!((s.attack_speed - 0.9).abs() < 1e-6);
        // 뿌리 심장은 상한까지만.
        assert_eq!(
            Stats::compute(&[], 99, &NO_MODS).max_hp,
            Stats::BASE.max_hp + crate::tuning::ROOT_HEART_MAX
        );
        // 0253: 시작 3칸, 무엇을 더해도 12칸까지(재주 +3, 뿌리 심장 +5, 굳은 껍질 +1·꽃잎 +2 = 14 → 12).
        assert_eq!(Stats::BASE.max_hp, 3);
        let talent = Mods {
            max_hp: 3,
            ..NO_MODS
        };
        let petal = (0..CHARMS.len())
            .find(|&c| CHARMS[c].mods.max_hp == 2)
            .unwrap();
        let shell = (0..CHARMS.len())
            .find(|&c| CHARMS[c].mods.max_hp == 1)
            .unwrap();
        assert_eq!(Stats::compute(&[shell, petal], 5, &talent).max_hp, 12);
    }

    #[test]
    fn weight_orders_cancel_windows() {
        // 가벼울수록 빨리 끊긴다: 새싹·두더지·독은 언제나, 바위는 휘두른 뒤에.
        assert_eq!(WEAPONS[0].moveset.hits[0].dodge_cancel_after, 0.0);
        assert_eq!(WEAPONS[1].moveset.hits[0].dodge_cancel_after, 0.0);
        assert!(WEAPONS[2].moveset.hits[0].dodge_cancel_after > 0.0);
        assert!(WEAPONS[3].moveset.hits.iter().all(|h| h.poison > 0.0));
    }
}
