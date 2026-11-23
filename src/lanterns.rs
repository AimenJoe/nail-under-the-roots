//! 이웃 등불(0304, v1.1 "건네주는 빛"): 마을 창에서 모은 반딧불로 등불을 만들어 이웃에게 건네면, 그 이웃의 짧은 이야기가 하나씩 열린다.
//! 이웃마다 셋 — ① 지난날 ② 다른 이웃과의 사이 ③ 할매를 놓아준 뒤. 힘(영구 수치)은 주지 않는다: 재주 나무를 다 깨운 뒤에도
//! 반딧불이 쓰일 곳이자 사이드 스토리(0303 A1+B1).
//!
//! 규칙(시험이 본다):
//! - 차례대로 하나씩. 값은 단계마다 `COST`. 기록은 수 `lamp.<이웃>`(0..=3) — 키 이름은 바꾸지 않는다(0041).
//! - 그 이웃이 마을에 있어야 한다(`present`: 돌아온 뒤, 지금 걷는 캐릭터 자신이면 마을에 따로 서 있지 않다).
//! - 장면마다 더 붙는 조건(`gate`): 짝이 될 이웃이 돌아왔는지, 셋째는 반쪽 엔딩을 본 기록(할매의 정체를 먼저 말하지 않는다, 0273과 같게).
//! - `about`: 그 장면이 이야기하는 플레이어블 캐릭터. 그 캐릭터로 걷는 동안에는 들을 수 없다 — 이웃은 눈앞의 캐릭터를
//!   제삼자로 말하지 않는다(0264).
//! - 셋째 장면을 들으면 그 이웃 곁에 등불이 걸린다(`village::State::lamps`).
//!
//! 글은 열한 언어 표(`lamp_<이웃>_<단계>_t`·`_<줄>`). 지금은 포포 셋 — 나머지 이웃은 한국어 초안을 사장이 본 뒤(0304).

use crate::art::Face;
use crate::equipment::Character;
use crate::i18n::Tr;
use crate::profile::{key, Profile};
use crate::talents::KEY_LIGHT;
use crate::world::Who;

/// 단계마다 값(반딧불). 재주 나무 한 칸(120~495)·런 하나(약 240)와 견준 처음 값(0304).
pub const COST: [u32; 3] = [120, 200, 300];

pub struct Scene {
    pub title: Tr,
    pub lines: &'static [(Tr, Face)],
    /// 이 장면이 이야기하는 플레이어블 캐릭터(그 캐릭터로 걸을 때는 못 듣는다).
    pub about: Option<Character>,
    /// 더 붙는 조건(짝 이웃·엔딩). `None`은 이웃이 있으면 늘.
    pub gate: Option<fn(&Profile) -> bool>,
}

pub struct Lamp {
    pub who: Who,
    pub name: Tr,
    /// 기록 키(수 0..=3).
    pub key: &'static str,
    /// 마을에 서 있나(지금 걷는 캐릭터 빼고).
    pub present: fn(&Profile, Character) -> bool,
    pub scenes: [Scene; 3],
}

fn after_release(p: &Profile) -> bool {
    p.flag(key::HALF_ENDING) || p.flag(key::TRUE_ENDING)
}

pub const ALL: [Lamp; 1] = [Lamp {
    who: Who::Popo,
    name: |s| s.popo_name,
    key: "lamp.popo",
    present: |_, _| true,
    scenes: [
        Scene {
            title: |s| s.lamp_popo_1_t,
            lines: &[
                (|s| s.lamp_popo_1_1, Face::Happy),
                (|s| s.lamp_popo_1_2, Face::Normal),
                (|s| s.lamp_popo_1_3, Face::Normal),
                (|s| s.lamp_popo_1_4, Face::Worried),
                (|s| s.lamp_popo_1_5, Face::Happy),
            ],
            about: None,
            gate: None,
        },
        Scene {
            title: |s| s.lamp_popo_2_t,
            lines: &[
                (|s| s.lamp_popo_2_1, Face::Normal),
                (|s| s.lamp_popo_2_2, Face::Happy),
                (|s| s.lamp_popo_2_3, Face::Worried),
                (|s| s.lamp_popo_2_4, Face::Normal),
            ],
            about: Some(Character::Momo),
            gate: Some(|p| p.flag(key::MOMO_BACK)),
        },
        Scene {
            title: |s| s.lamp_popo_3_t,
            lines: &[
                (|s| s.lamp_popo_3_1, Face::Normal),
                (|s| s.lamp_popo_3_2, Face::Normal),
                (|s| s.lamp_popo_3_3, Face::Normal),
                (|s| s.lamp_popo_3_4, Face::Happy),
                (|s| s.lamp_popo_3_5, Face::Happy),
            ],
            about: None,
            // 셋째는 샘 그릇(포포의 부탁, 0273 B)이 있어야 말이 된다.
            gate: Some(|p| after_release(p) && crate::quests::done(p, crate::quests::POPO)),
        },
    ],
}];

/// 들은 장면 수(0..=3).
pub fn level(p: &Profile, i: usize) -> usize {
    (p.count(ALL[i].key) as usize).min(3)
}

/// 다음에 무엇이 되나.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// 셋 다 들었다.
    Done,
    /// 다음 장면의 조건이 아직(짝 이웃·엔딩).
    Later,
    /// 다음 장면이 지금 걷는 캐릭터에 관한 이야기 — 다른 캐릭터로 오면.
    Other,
    /// 건넬 수 있다(값이 모자라도 — 모자라는지는 부르는 쪽이 본다).
    Ready { stage: usize, cost: u32 },
}

pub fn next(p: &Profile, i: usize, ch: Character) -> Next {
    let stage = level(p, i);
    let Some(scene) = ALL[i].scenes.get(stage) else {
        return Next::Done;
    };
    if scene.about == Some(ch) {
        Next::Other
    } else if !scene.gate.is_none_or(|g| g(p)) {
        Next::Later
    } else {
        Next::Ready {
            stage,
            cost: COST[stage],
        }
    }
}

/// 마을 창 등불 탭에 보이는 줄(마을에 서 있는 이웃만, 표 순서).
pub fn rows(p: &Profile, ch: Character) -> Vec<usize> {
    (0..ALL.len())
        .filter(|&i| (ALL[i].present)(p, ch))
        .collect()
}

/// 건넨다: 반딧불을 내고 단계를 올린다. 건넸으면 들을 장면 번호.
pub fn give(p: &mut Profile, i: usize, ch: Character) -> Option<usize> {
    let Next::Ready { stage, cost } = next(p, i, ch) else {
        return None;
    };
    let have = p.count(KEY_LIGHT);
    if have < cost {
        return None;
    }
    p.set_count(KEY_LIGHT, have - cost);
    p.set_count(ALL[i].key, stage as u32 + 1);
    Some(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich() -> Profile {
        let mut p = Profile::default();
        p.set_count(KEY_LIGHT, 10_000);
        p
    }

    /// 차례대로, 값만큼 빠지고, 셋 뒤엔 끝.
    #[test]
    fn lamps_go_in_order_and_cost() {
        let mut p = rich();
        p.set(key::MOMO_BACK);
        p.set(key::HALF_ENDING);
        crate::quests::advance(&mut p, crate::quests::POPO, crate::quests::Step::Ask);
        crate::quests::advance(&mut p, crate::quests::POPO, crate::quests::Step::Thanks);
        let ch = Character::Mole;
        assert_eq!(
            next(&p, 0, ch),
            Next::Ready {
                stage: 0,
                cost: COST[0]
            }
        );
        assert_eq!(give(&mut p, 0, ch), Some(0));
        assert_eq!(p.count(KEY_LIGHT), 10_000 - COST[0]);
        assert_eq!(give(&mut p, 0, ch), Some(1));
        assert!(level(&p, 0) < 3);
        assert_eq!(give(&mut p, 0, ch), Some(2));
        assert_eq!(level(&p, 0), 3);
        assert_eq!(next(&p, 0, ch), Next::Done);
        assert_eq!(give(&mut p, 0, ch), None);
        assert_eq!(p.count(KEY_LIGHT), 10_000 - COST.iter().sum::<u32>());
    }

    /// 모자라면 건네지 않는다(빠지지도 않는다).
    #[test]
    fn lamps_need_enough_glowbugs() {
        let mut p = Profile::default();
        p.set_count(KEY_LIGHT, COST[0] - 1);
        assert_eq!(give(&mut p, 0, Character::Mole), None);
        assert_eq!(p.count(KEY_LIGHT), COST[0] - 1);
        assert_eq!(level(&p, 0), 0);
    }

    /// 짝 이웃·엔딩 조건, 그 캐릭터로 걸을 때는 그 캐릭터 이야기를 하지 않는다.
    #[test]
    fn lamp_scenes_wait_for_their_gates() {
        let mut p = rich();
        give(&mut p, 0, Character::Mole);
        assert_eq!(
            next(&p, 0, Character::Mole),
            Next::Later,
            "모모가 돌아오기 전"
        );
        p.set(key::MOMO_BACK);
        assert_eq!(
            next(&p, 0, Character::Momo),
            Next::Other,
            "모모로 걸을 때 모모 이야기"
        );
        assert!(matches!(next(&p, 0, Character::Doto), Next::Ready { .. }));
        give(&mut p, 0, Character::Mole);
        assert_eq!(
            next(&p, 0, Character::Mole),
            Next::Later,
            "할매를 놓아주기 전"
        );
        p.set(key::TRUE_ENDING);
        assert_eq!(
            next(&p, 0, Character::Mole),
            Next::Later,
            "샘 그릇(포포의 부탁) 전"
        );
        crate::quests::advance(&mut p, crate::quests::POPO, crate::quests::Step::Ask);
        crate::quests::advance(&mut p, crate::quests::POPO, crate::quests::Step::Thanks);
        assert!(matches!(
            next(&p, 0, Character::Mole),
            Next::Ready { stage: 2, .. }
        ));
    }

    /// 장면마다 줄이 있고, 이웃마다 셋, 기록 키는 겹치지 않는다.
    #[test]
    fn lamp_table_is_whole() {
        let ko = &crate::i18n::KO;
        let mut keys: Vec<&str> = ALL.iter().map(|l| l.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), ALL.len());
        for l in &ALL {
            assert!(!(l.name)(ko).is_empty());
            for sc in &l.scenes {
                assert!(!(sc.title)(ko).is_empty());
                assert!((3..=6).contains(&sc.lines.len()));
                for (t, _) in sc.lines {
                    assert!(!t(ko).is_empty());
                }
            }
        }
    }
}
