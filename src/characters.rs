//! 플레이어블 캐릭터 표(0170 R14). 새 캐릭터 = 여기 한 줄 + 회피·특기의 수(`player/`) + 그림(`render/`) + 글(`i18n/`).
//! 누구인가(`equipment::Character`)는 열거형 그대로 — 기록 키(`clears.<캐릭터>`)·장신구 `only`·재주 표가 그 이름을 쓴다.
//! 회피·특기만 다르고 나머지 틀은 같다(0003·0056).

use crate::art::Portrait;
use crate::equipment::{Character, Gauge, WeaponKind};
use crate::i18n::Tr;
use crate::profile::{key, Profile};
use crate::talents::Talent;

/// 회피 종류: 굴파기(땅속으로, 떼면 솟구친다) / 깡충(공중 무적).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dodge {
    Burrow,
    Hop,
}

/// 특기 종류: 드릴(화면 길이 돌진) / 이어 뛰기(세 번 찍기).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Special {
    Drill,
    Chain,
}

pub struct CharacterDef {
    pub id: Character,
    /// 이어하기·고르기 기록의 번호(바꾸지 않는다, 0056·0080).
    pub code: u8,
    pub name: Tr,
    /// 고를 수 있는 조건(기록 깃발). None = 처음부터.
    pub unlock: Option<&'static str>,
    pub weapon_kind: WeaponKind,
    pub gauge: Gauge,
    /// 처음 들고 시작하는 무기(`equipment::WEAPONS` 번호).
    pub starting_weapon: usize,
    pub dodge: Dodge,
    pub special: Special,
    /// 큰 나무의 재주 표와 자리 넷의 기록 키 이름(자리 2·4는 캐릭터끼리 같은 키, 0142).
    pub talents: &'static [(Tr, [Talent; 2]); 4],
    pub talent_keys: [&'static str; 4],
    /// 고르기 창의 초상화(털빛을 받는다)와 소개.
    pub portrait: fn(u8) -> Portrait,
    pub pick_text: Tr,
    /// 상태 탭의 회피 대기 이름, 등딱지·덤불·그물에 처음 걸렸을 때의 한마디(캐릭터마다 말투).
    pub stat_dodge_cd: Tr,
    pub toast_shell: Tr,
    pub toast_thorn: Tr,
    pub toast_web: Tr,
    /// 파 내려가는 전환(`draw_descent`)의 그림 줄. None = Naïl의 털빛 줄.
    pub dig_row: Option<u32>,
}

pub static ALL: [CharacterDef; 2] = [
    CharacterDef {
        id: Character::Mole,
        code: 0,
        name: |s| s.name_nail,
        unlock: None,
        weapon_kind: WeaponKind::Claw,
        gauge: Gauge::Rage,
        starting_weapon: 1,
        dodge: Dodge::Burrow,
        special: Special::Drill,
        talents: &crate::talents::TABLE,
        talent_keys: ["burrow", "parry", "drill", "eat"],
        portrait: Portrait::nail,
        pick_text: |s| s.pick_nail,
        stat_dodge_cd: |s| s.stat_dig_cd,
        toast_shell: |s| s.toast_shell,
        toast_thorn: |s| s.toast_thorn,
        toast_web: |s| s.toast_web,
        dig_row: None,
    },
    // 돌아온 모모(0046) → 플레이어블(0056): 발차기 3연타, 깡충, 이어 뛰기(기 게이지). 자리 1·3의 재주는 제 것(0142).
    CharacterDef {
        id: Character::Momo,
        code: 1,
        name: |s| s.name_momo,
        unlock: Some(key::MOMO_BACK),
        weapon_kind: WeaponKind::Kick,
        gauge: Gauge::Ki,
        starting_weapon: 4,
        dodge: Dodge::Hop,
        special: Special::Chain,
        talents: &crate::talents::MOMO_TABLE,
        talent_keys: ["hop", "parry", "chain", "eat"],
        portrait: |_| Portrait::MOMO,
        pick_text: |s| s.pick_momo,
        stat_dodge_cd: |s| s.stat_hop_cd,
        toast_shell: |s| s.toast_shell_momo,
        toast_thorn: |s| s.toast_thorn_momo,
        toast_web: |s| s.toast_web_momo,
        dig_row: Some(4),
    },
];

impl CharacterDef {
    /// 이 기록으로 고를 수 있나.
    pub fn available(&self, p: &Profile) -> bool {
        self.unlock.is_none_or(|k| p.flag(k))
    }
}

impl Character {
    /// 표의 줄.
    pub fn def(self) -> &'static CharacterDef {
        ALL.iter()
            .find(|d| d.id == self)
            .expect("캐릭터 표에 줄이 있다")
    }

    /// 기록 번호 → 캐릭터(모르는 번호는 Naïl).
    pub fn from_code(code: u8) -> Character {
        ALL.iter()
            .find(|d| d.code == code)
            .map_or(Character::Mole, |d| d.id)
    }
}

/// 이 기록으로 고를 수 있는 캐릭터들(표 순서).
pub fn available(p: &Profile) -> impl Iterator<Item = &'static CharacterDef> + '_ {
    ALL.iter().filter(move |d| d.available(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 표의 번호·키는 기록 형식이라 바꾸지 않는다(0041·0056·0080). 줄마다 제 번호로 돌아온다.
    #[test]
    fn codes_and_keys_are_stable() {
        for d in &ALL {
            assert_eq!(d.id.def().id, d.id);
            assert_eq!(Character::from_code(d.code), d.id);
            assert!(!(d.name)(&crate::i18n::KO).is_empty());
        }
        assert_eq!(Character::from_code(9), Character::Mole);
        assert_eq!(
            Character::Mole.def().talent_keys,
            ["burrow", "parry", "drill", "eat"]
        );
        assert_eq!(
            Character::Momo.def().talent_keys,
            ["hop", "parry", "chain", "eat"]
        );
        let p = Profile::default();
        assert_eq!(available(&p).count(), 1, "모모는 돌아온 뒤에만");
    }
}
