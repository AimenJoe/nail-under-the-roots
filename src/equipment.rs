//! 장비: 캐릭터마다 쓸 수 있는 무기 종류가 다르다(docs/decisions/0011).
//! 두더지 Naïl은 발톱만. 다른 캐릭터(2단계 보스 → 3단계 플레이어블)는 자기 무기 종류를 갖는다.

use macroquad::color::Color;

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

    /// 처음 들고 시작하는 무기(WEAPONS 번호).
    pub fn starting_weapon(self) -> usize {
        match self {
            Character::Mole => 1,
        }
    }
}

pub struct Weapon {
    pub name: &'static str,
    pub desc: &'static str,
    pub kind: WeaponKind,
    pub moveset: &'static Moveset,
    /// 가방 칸 아이콘 색.
    pub color: Color,
}

pub static WEAPONS: [Weapon; 4] = [
    Weapon {
        name: "새싹 발톱",
        desc: "가벼움. 4연타가 짧고 빠르게 이어진다. 한 방은 약하다.",
        kind: WeaponKind::Claw,
        moveset: &moveset::SPROUT,
        color: Color::new(0.62, 0.86, 0.48, 1.0),
    },
    Weapon {
        name: "두더지 발톱",
        desc: "기본. 3연타, 세 번째가 묵직하다. 언제든 굴파기로 끊는다.",
        kind: WeaponKind::Claw,
        moveset: &moveset::CLAWS,
        color: Color::new(0.97, 0.93, 0.85, 1.0),
    },
    Weapon {
        name: "바위 발톱",
        desc: "무거움. 2연타, 넓고 세게 밀친다. 휘두른 뒤에야 굴파기로 끊는다.",
        kind: WeaponKind::Claw,
        moveset: &moveset::STONE,
        color: Color::new(0.66, 0.64, 0.62, 1.0),
    },
    Weapon {
        name: "독 발톱",
        desc: "맞은 적에게 독을 건다. 마지막 타의 독이 길다. 독은 경직 없이 체력만 깎는다.",
        kind: WeaponKind::Claw,
        moveset: &moveset::VENOM,
        color: Color::new(0.52, 0.86, 0.36, 1.0),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// 규칙(docs/decisions/0009): 모든 발톱의 마지막 타가 가장 세게 밀치고 가장 오래 멈춘다.
    #[test]
    fn finisher_is_strongest_for_every_claw() {
        for w in &WEAPONS {
            let hits = w.moveset.hits;
            let last = hits[hits.len() - 1];
            for h in &hits[..hits.len() - 1] {
                assert!(last.knockback > h.knockback, "{}", w.name);
                assert!(last.hitstop >= h.hitstop, "{}", w.name);
            }
            assert_eq!(w.kind, WeaponKind::Claw);
        }
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
