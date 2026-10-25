//! 장비: 캐릭터마다 쓸 수 있는 무기 종류가 다르다(docs/decisions/0011).
//! 두더지 Naïl은 발톱만. 다른 캐릭터(2단계 보스 → 3단계 플레이어블)는 자기 무기 종류를 갖는다.
//!
//! 능력치(docs/decisions/0017): 기본값 + 장신구(2칸, 아무 캐릭터나) + 뿌리 심장(최대 체력 영구 +1).
//! 플레이어 코드는 `Stats`만 본다. 무기는 콤보 모양(moveset), 장신구는 수치 보정.

use macroquad::color::Color;

use crate::i18n::Tr;
use crate::moveset::{self, Moveset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    Mole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Claw,
}

impl Character {
    pub fn name(self) -> &'static str {
        match self {
            Character::Mole => "Naïl",
        }
    }

    pub fn weapon_kind(self) -> WeaponKind {
        match self {
            Character::Mole => WeaponKind::Claw,
        }
    }

    /// 필살기 게이지의 종류. 두더지는 분노(맞히면 찬다), 다음 캐릭터는 기(氣) 등.
    pub fn gauge(self) -> Gauge {
        match self {
            Character::Mole => Gauge::Rage,
        }
    }

    /// 처음 들고 시작하는 무기(WEAPONS 번호).
    pub fn starting_weapon(self) -> usize {
        match self {
            Character::Mole => 1,
        }
    }
}

/// 필살기 게이지. 채우는 규칙은 지금 모두 "맞히면 찬다"(tuning SP_*). 이름·색만 다르다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gauge {
    Rage,
    #[allow(dead_code)]
    Ki,
}

impl Gauge {
    pub fn name(self) -> Tr {
        match self {
            Gauge::Rage => |s| s.gauge_rage,
            Gauge::Ki => |s| s.gauge_ki,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Gauge::Rage => Color::new(1.0, 0.55, 0.22, 1.0),
            Gauge::Ki => Color::new(0.45, 0.80, 1.0, 1.0),
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
}

pub static WEAPONS: [Weapon; 4] = [
    Weapon {
        name: |s| s.w_sprout,
        desc: |s| s.w_sprout_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::SPROUT,
        color: Color::new(0.62, 0.86, 0.48, 1.0),
    },
    Weapon {
        name: |s| s.w_mole,
        desc: |s| s.w_mole_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::CLAWS,
        color: Color::new(0.97, 0.93, 0.85, 1.0),
    },
    Weapon {
        name: |s| s.w_stone,
        desc: |s| s.w_stone_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::STONE,
        color: Color::new(0.66, 0.64, 0.62, 1.0),
    },
    Weapon {
        name: |s| s.w_venom,
        desc: |s| s.w_venom_desc,
        kind: WeaponKind::Claw,
        moveset: &moveset::VENOM,
        color: Color::new(0.52, 0.86, 0.36, 1.0),
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
}

impl Stats {
    pub const BASE: Stats = Stats {
        max_hp: crate::tuning::PLAYER_HP,
        damage: 0,
        attack_speed: 1.0,
        burrow_cd: 1.0,
        sp_gain: 1.0,
    };

    /// 기본값에 장신구와 뿌리 심장을 더한다. 배율은 곱, 더하기는 합. 너무 극단으로 가지 않게 묶는다.
    pub fn compute(charms: &[usize], root_hearts: i32) -> Stats {
        let mut s = Stats::BASE;
        for &c in charms {
            let m = CHARMS[c].mods;
            s.max_hp += m.max_hp;
            s.damage += m.damage;
            s.attack_speed *= m.attack_speed;
            s.burrow_cd *= m.burrow_cd;
            s.sp_gain *= m.sp_gain;
        }
        s.max_hp = (s.max_hp + root_hearts.min(crate::tuning::ROOT_HEART_MAX)).max(1);
        s.attack_speed = s.attack_speed.clamp(0.6, 1.8);
        s.burrow_cd = s.burrow_cd.clamp(0.3, 2.0);
        s
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
}

const NO_MODS: Mods = Mods {
    max_hp: 0,
    damage: 0,
    attack_speed: 1.0,
    burrow_cd: 1.0,
    sp_gain: 1.0,
};

/// 장신구: 캐릭터와 상관없이 낀다. 2칸. 얻는 곳은 상자·상점(맵 단계).
pub struct Charm {
    pub name: Tr,
    pub desc: Tr,
    pub mods: Mods,
    pub color: Color,
}

/// 동시에 낄 수 있는 장신구 수.
pub const CHARM_SLOTS: usize = 2;
/// 처음부터 상자에서 나오는 장신구 수. 그 뒤는 저널로 연다(docs/decisions/0048).
pub const BASE_CHARMS: usize = 5;

pub static CHARMS: [Charm; 8] = [
    Charm {
        name: |s| s.c_shell,
        desc: |s| s.c_shell_desc,
        mods: Mods {
            max_hp: 1,
            ..NO_MODS
        },
        color: Color::new(0.80, 0.62, 0.42, 1.0),
    },
    Charm {
        name: |s| s.c_whisker,
        desc: |s| s.c_whisker_desc,
        mods: Mods {
            attack_speed: 1.2,
            ..NO_MODS
        },
        color: Color::new(0.86, 0.86, 0.92, 1.0),
    },
    Charm {
        name: |s| s.c_soil,
        desc: |s| s.c_soil_desc,
        mods: Mods {
            burrow_cd: 0.6,
            ..NO_MODS
        },
        color: Color::new(0.50, 0.36, 0.26, 1.0),
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
    },
    Charm {
        name: |s| s.c_amulet,
        desc: |s| s.c_amulet_desc,
        mods: Mods {
            sp_gain: 1.5,
            ..NO_MODS
        },
        color: Color::new(0.95, 0.80, 0.35, 1.0),
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
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// 규칙(docs/decisions/0009): 모든 발톱의 마지막 타가 가장 세게 밀치고 가장 오래 멈춘다.
    #[test]
    fn finisher_is_strongest_for_every_claw() {
        for w in &WEAPONS {
            let name = (w.name)(&crate::i18n::KO);
            let hits = w.moveset.hits;
            let last = hits[hits.len() - 1];
            for h in &hits[..hits.len() - 1] {
                assert!(last.knockback > h.knockback, "{name}");
                assert!(last.hitstop >= h.hitstop, "{name}");
            }
            assert_eq!(w.kind, WeaponKind::Claw);
        }
    }

    #[test]
    fn stats_combine_and_clamp() {
        assert_eq!(Stats::compute(&[], 0), Stats::BASE);
        let s = Stats::compute(&[0, 3], 2);
        assert_eq!(s.max_hp, Stats::BASE.max_hp + 1 + 2);
        assert_eq!(s.damage, 1);
        assert!((s.attack_speed - 0.9).abs() < 1e-6);
        // 뿌리 심장은 상한까지만.
        assert_eq!(
            Stats::compute(&[], 99).max_hp,
            Stats::BASE.max_hp + crate::tuning::ROOT_HEART_MAX
        );
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
