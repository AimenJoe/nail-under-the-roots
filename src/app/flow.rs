//! 런의 갈림길 결정(0230 R15 ②): 런을 어디서 어떻게 시작하나, 쓰러지면 어디로 가나.
//!
//! 앱(`App::start`·`enter_first_scene`·쓰러짐)은 소리·그림·월드를 들고 있어 단위 시험으로 못 띄운다.
//! 그래서 "무엇을 할지"만 여기서 순수 함수로 정하고(사실 몇 개 → 결정 하나), 앱은 결정을 그대로 한다.
//! 최근 고친 흐름 버그(마을에서 곧장 길로 0226, 쉼터 이어하기)가 다 이 갈림길이었다.

use crate::equipment::Character;
use crate::scenario::Scenario;

/// 이 런의 캐릭터(0056): 런(첫 시작·오늘의 굴)과 개발의 가장 깊은 곳은 뿌리 길에서 고른 캐릭터,
/// 개발의 도토·모모 시험장은 그 캐릭터, 그 밖(실의 꿈·개발)은 Naïl. 실의 꿈은 Naïl만: 꿈 목록에 모모가 있다.
/// 마을도 고른 캐릭터로 걷는다(사장 2026-10-29: 도토로 고르면 엔딩 뒤 올라오는 길·마을까지 도토) — 그 이웃은 자리에서 빠진다.
pub fn character(s: Scenario, picked: Character) -> Character {
    match s {
        Scenario::Normal | Scenario::Daily | Scenario::Deepest | Scenario::Village => picked,
        // 개발: 도토로 싸워 보기(0195).
        Scenario::DotoArena | Scenario::DotoMobs => Character::Doto,
        Scenario::MomoArena => Character::Momo,
        Scenario::Hall(c) => c,
        // 개발 "무기 비교"(0246): 고른 캐릭터의 무기를 다 들고.
        Scenario::AllClaws => picked,
        _ => Character::Mole,
    }
}

/// 튜토리얼을 마친 기록이 있는 첫 시작: 오프닝·튜토리얼을 건너뛰고 발톱을 든 채 시작한다(0041).
pub fn veteran(s: Scenario, tutorial_done: bool) -> bool {
    s == Scenario::Normal && tutorial_done
}

/// 런의 첫 장면.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum First {
    /// 마을(엔딩 뒤 올라오는 길이 아니면 들어설 때 한마디).
    Village,
    /// 마을 뿌리 길에서 첫 쉼터로 곧장 내려간다(0225). 닿으면 런 첫 한마디, 그다음 길 지도(0226).
    Descend,
    /// 들판에서(보리를 아직 못 깨웠거나, 쉼터 이어하기). `skip_boss`: 공터 보스는 이미 돌아왔다.
    /// `near_entrance`: 쉼터 이어하기는 곧바로 그 쉼터를 다시 세운다(`resume_run`). `say`: 런 첫 한마디.
    Field {
        skip_boss: bool,
        near_entrance: bool,
        say: bool,
    },
    /// 오프닝 컷씬. `tutorial`: 끝나면 플레이 튜토리얼(첫 시작), 아니면 빈 방(개발 "컷씬만").
    Opening { tutorial: bool },
    /// 가장 깊은 곳(0162)에 바로.
    Deepest,
    /// 실의 꿈(0055).
    Dream,
    /// 이야기 일러스트 미리 보기(개발).
    StoryPreview,
    /// 곧장 싸움(개발 시나리오). `tutorial`: 컷씬 없이 튜토리얼만.
    Play { tutorial: bool },
}

pub fn first(s: Scenario, veteran: bool, bori_back: bool, resuming: bool) -> First {
    if s == Scenario::Village {
        return First::Village;
    }
    if veteran {
        if bori_back && !resuming {
            return First::Descend;
        }
        return First::Field {
            skip_boss: bori_back,
            near_entrance: bori_back,
            say: !resuming,
        };
    }
    match s {
        Scenario::Normal => First::Opening { tutorial: true },
        Scenario::CutsceneOnly => First::Opening { tutorial: false },
        Scenario::Deepest => First::Deepest,
        Scenario::Dream => First::Dream,
        Scenario::StoryPreview => First::StoryPreview,
        Scenario::Tutorial => First::Play { tutorial: true },
        _ => First::Play { tutorial: false },
    }
}

/// 쓰러진 뒤 다시 하기를 눌렀다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fall {
    /// 모험(필드·던전)에서는 저장 지점에서 깨어난다. `wake_line`: 깨어난 뒤 이웃의 한마디(기록이 남는 런만).
    Respawn { wake_line: bool },
    /// 실뿌리가 다 감긴 뒤 또 쓰러졌다(0210): 런이 끝난다. 이야기 런은 마을로, 그 밖(오늘의 굴)은 타이틀로.
    RunLost { to_village: bool },
    /// 아레나·개발 시나리오는 처음부터.
    Restart,
}

/// 런이 끝나면 마을로 돌아가나(이야기 런). 아니면 타이틀로(오늘의 굴·개발).
pub fn back_to_village(s: Scenario) -> bool {
    s == Scenario::Normal
}

pub fn fall(s: Scenario, in_arena: bool, alive: bool, run_over: bool) -> Fall {
    if in_arena || alive {
        Fall::Restart
    } else if run_over {
        Fall::RunLost {
            to_village: back_to_village(s),
        }
    } else {
        Fall::Respawn {
            wake_line: s == Scenario::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_plays_opening_then_tutorial() {
        assert!(!veteran(Scenario::Normal, false));
        assert_eq!(
            first(Scenario::Normal, false, false, false),
            First::Opening { tutorial: true }
        );
        assert_eq!(
            first(Scenario::CutsceneOnly, false, false, false),
            First::Opening { tutorial: false }
        );
        assert_eq!(
            first(Scenario::Tutorial, false, false, false),
            First::Play { tutorial: true }
        );
    }

    #[test]
    fn veteran_run_descends_from_village_once_bori_is_back() {
        assert!(veteran(Scenario::Normal, true));
        assert!(!veteran(Scenario::Daily, true), "오늘의 굴은 늘 제 길");
        assert_eq!(first(Scenario::Normal, true, true, false), First::Descend);
        // 보리를 아직 못 깨웠으면 들판의 공터 싸움부터, 런 첫 한마디.
        assert_eq!(
            first(Scenario::Normal, true, false, false),
            First::Field {
                skip_boss: false,
                near_entrance: false,
                say: true
            }
        );
    }

    #[test]
    fn resuming_rebuilds_the_camp_without_lines() {
        assert_eq!(
            first(Scenario::Normal, true, true, true),
            First::Field {
                skip_boss: true,
                near_entrance: true,
                say: false
            }
        );
        assert_eq!(
            first(Scenario::Normal, true, false, true),
            First::Field {
                skip_boss: false,
                near_entrance: false,
                say: false
            }
        );
    }

    #[test]
    fn village_and_dev_entries() {
        assert_eq!(first(Scenario::Village, false, true, false), First::Village);
        assert_eq!(
            first(Scenario::Deepest, false, false, false),
            First::Deepest
        );
        assert_eq!(first(Scenario::Dream, false, false, false), First::Dream);
        assert_eq!(
            first(Scenario::StoryPreview, false, false, false),
            First::StoryPreview
        );
        assert_eq!(
            first(Scenario::Arena, false, false, false),
            First::Play { tutorial: false }
        );
    }

    #[test]
    fn characters_follow_the_pick_only_on_runs() {
        for s in [Scenario::Normal, Scenario::Daily, Scenario::Deepest] {
            assert_eq!(character(s, Character::Momo), Character::Momo);
        }
        assert_eq!(character(Scenario::Dream, Character::Momo), Character::Mole);
        assert_eq!(
            character(Scenario::Village, Character::Doto),
            Character::Doto
        );
        assert_eq!(
            character(Scenario::DotoArena, Character::Mole),
            Character::Doto
        );
    }

    #[test]
    fn falling_respawns_until_the_threads_run_out() {
        assert_eq!(
            fall(Scenario::Normal, false, false, false),
            Fall::Respawn { wake_line: true }
        );
        assert_eq!(
            fall(Scenario::Daily, false, false, false),
            Fall::Respawn { wake_line: false }
        );
        assert_eq!(
            fall(Scenario::Normal, false, false, true),
            Fall::RunLost { to_village: true }
        );
        assert_eq!(
            fall(Scenario::Daily, false, false, true),
            Fall::RunLost { to_village: false }
        );
        assert_eq!(fall(Scenario::Arena, true, false, false), Fall::Restart);
    }
}
