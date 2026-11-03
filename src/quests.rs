//! 이웃의 부탁(0273 B): 부탁 → 찾기 → 돌려주기 → 마을의 흔적. 표 한 곳, 진행은 모두 기록(profile)에서.
//!
//! - **부탁**: 마을에서 그 이웃에게 말을 걸면 한 번(`talk` → `Step::Ask`). 열린 부탁(들었고 아직 안 돌려줌)이 있으면 다른 이웃은 부탁하지 않는다.
//! - **찾기**: 들은 뒤의 첫 시작 런마다, 맞는 지대의 굴(표 `biome`, 없으면 어느 굴이든)의 조용한 방에 이야기 물건 하나
//!   (`StoryItem::Quest`, 월드는 기록을 모른다 — 앱이 `StorySeed::quest`로 넣는다). 주우면 `found`.
//! - **돌려주기**: 주운 뒤 그 이웃에게 말을 걸면 고맙다는 한마디(`Step::Thanks`) → `done`, 마을에 흔적(`village::State::traces`).
//! - **보상**: 저널 목표 `quest_<id>`(이야기 조각 `tales`). 힘이 아닌 것.
//!
//! 기록 키 `quest.<id>.asked|found|done` — 아이디는 바꾸지 않는다.

use crate::biome::Biome;
use crate::i18n::Tr;
use crate::profile::{key, Profile};
use crate::world::Who;

pub struct Quest {
    pub id: &'static str,
    pub who: Who,
    /// 이 깃발이 서야 부탁한다(그 이웃이 마을에 있다).
    pub opens: &'static str,
    /// 찾을 것이 놓이는 지대(None = 어느 굴이든).
    pub biome: Option<Biome>,
    pub ask: Tr,
    /// 주웠을 때 알림.
    pub found: Tr,
    pub thanks: Tr,
}

pub const POPO: usize = 0;
pub const BORI: usize = 1;
pub const MOMO: usize = 2;
pub const DOTO: usize = 3;

pub const ALL: [Quest; 4] = [
    Quest {
        id: "popo",
        who: Who::Popo,
        opens: key::TUTORIAL,
        biome: None,
        ask: |s| s.q_popo_ask,
        found: |s| s.q_popo_found,
        thanks: |s| s.q_popo_thanks,
    },
    Quest {
        id: "bori",
        who: Who::Bori,
        opens: key::BORI_BACK,
        biome: None,
        ask: |s| s.q_bori_ask,
        found: |s| s.q_bori_found,
        thanks: |s| s.q_bori_thanks,
    },
    Quest {
        id: "momo",
        who: Who::Momo,
        opens: key::MOMO_BACK,
        biome: Some(Biome::Meadow),
        ask: |s| s.q_momo_ask,
        found: |s| s.q_momo_found,
        thanks: |s| s.q_momo_thanks,
    },
    Quest {
        id: "doto",
        who: Who::Doto,
        opens: key::DOTO_MEMORY,
        biome: Some(Biome::Roots),
        ask: |s| s.q_doto_ask,
        found: |s| s.q_doto_found,
        thanks: |s| s.q_doto_thanks,
    },
];

/// 부탁 한 단계의 기록 키.
pub fn key_of(i: usize, step: &str) -> String {
    format!("quest.{}.{step}", ALL[i].id)
}
pub const ASKED: &str = "asked";
pub const FOUND: &str = "found";
pub const DONE: &str = "done";

/// 저널 목표가 보는 깃발(`quest.<id>.done`) — 문자열 상수라 표에 쓴다.
pub const DONE_KEYS: [&str; 4] = [
    "quest.popo.done",
    "quest.bori.done",
    "quest.momo.done",
    "quest.doto.done",
];

pub fn done(p: &Profile, i: usize) -> bool {
    p.flag(&key_of(i, DONE))
}

/// 들었고 아직 돌려주지 않은 부탁(한 번에 하나).
pub fn open(p: &Profile) -> Option<usize> {
    (0..ALL.len()).find(|&i| p.flag(&key_of(i, ASKED)) && !done(p, i))
}

/// 이번 런에 굴에 놓을 것: 열린 부탁 중 아직 못 찾은 것.
pub fn seek(p: &Profile) -> Option<usize> {
    open(p).filter(|&i| !p.flag(&key_of(i, FOUND)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Ask,
    Thanks,
}

/// 마을에서 그 이웃에게 말을 걸 때 부탁의 차례가 있나. 열린 부탁이 있으면 그 이웃만, 찾아 왔을 때만(고맙다).
/// 없으면 그 이웃의 아직 안 들은 부탁(그 이웃이 마을에 있을 때).
pub fn talk(who: Who, p: &Profile) -> Option<(usize, Step)> {
    if let Some(i) = open(p) {
        return (ALL[i].who == who && p.flag(&key_of(i, FOUND))).then_some((i, Step::Thanks));
    }
    (0..ALL.len())
        .find(|&i| ALL[i].who == who && p.flag(ALL[i].opens) && !p.flag(&key_of(i, ASKED)))
        .map(|i| (i, Step::Ask))
}

/// 부탁 한 단계를 기록에 적는다(`Ask` → 들음, `Thanks` → 돌려줌).
pub fn advance(p: &mut Profile, i: usize, step: Step) {
    let k = match step {
        Step::Ask => key_of(i, ASKED),
        Step::Thanks => key_of(i, DONE),
    };
    p.set(&k);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_keys_match_the_table() {
        for (i, k) in DONE_KEYS.iter().enumerate() {
            assert_eq!(key_of(i, DONE), *k);
        }
        assert_eq!(ALL[POPO].who, Who::Popo);
        assert_eq!(ALL[BORI].who, Who::Bori);
        assert_eq!(ALL[MOMO].who, Who::Momo);
        assert_eq!(ALL[DOTO].who, Who::Doto);
    }

    /// 부탁 → 찾기 → 돌려주기, 한 번에 하나: 열린 동안 다른 이웃은 부탁하지 않고, 돌려준 뒤에야 다음.
    #[test]
    fn one_request_at_a_time_from_ask_to_thanks() {
        let mut p = Profile::default();
        assert_eq!(talk(Who::Popo, &p), None, "튜토리얼 전에는 없다");
        p.set(key::TUTORIAL);
        p.set(key::BORI_BACK);
        assert_eq!(talk(Who::Popo, &p), Some((POPO, Step::Ask)));
        advance(&mut p, POPO, Step::Ask);
        assert_eq!(seek(&p), Some(POPO));
        assert_eq!(talk(Who::Bori, &p), None, "포포의 부탁이 열려 있다");
        assert_eq!(talk(Who::Popo, &p), None, "아직 못 찾았다");
        p.set(&key_of(POPO, FOUND));
        assert_eq!(seek(&p), None, "찾았으면 굴에 더 놓지 않는다");
        assert_eq!(talk(Who::Popo, &p), Some((POPO, Step::Thanks)));
        advance(&mut p, POPO, Step::Thanks);
        assert!(done(&p, POPO));
        assert_eq!(talk(Who::Popo, &p), None, "한 번뿐");
        assert_eq!(talk(Who::Bori, &p), Some((BORI, Step::Ask)));
        assert_eq!(talk(Who::Momo, &p), None, "모모가 아직 안 돌아왔다");
    }
}
