//! 안내 화면의 주제 표(0178): 일시정지 메뉴의 "안내" 탭. 튜토리얼을 지난 뒤에도 기능을 찾아볼 수 있게.
//! 글은 `i18n`(열한 언어). 새 기능을 넣으면 여기에 한 줄 + 열한 언어 두 키 + 그림(`Pic`·`render/help.rs`).
//!
//! 안내는 조작·규칙·시스템만 쓴다. 마을의 장소·물건·이웃 이야기는 마을 창의 수첩(`village_notes.rs`)에 —
//! 같은 글을 두 곳에 쓰지 않는다(0269 R36, 시험 `help_and_notes_share_no_text`).
//!
//! 잠금(0269 R36): `gate`가 있는 주제는 기록이 그 자리에 닿았을 때만 보인다(엔딩 전 스포일러·처음 보는 사람의
//! 긴 목록을 막는다). 새로 열린 주제에는 "새" 점이 붙고, 한 번 보면 `help.<id>` 깃발이 서서 점이 사라진다.

use crate::i18n::Tr;
use crate::profile::{key, Profile};

pub struct Topic {
    /// 기록 키 `help.<id>`의 이름(바꾸지 않는다, 0041).
    pub id: &'static str,
    pub title: Tr,
    pub body: Tr,
    /// 설명 위의 움직이는 힌트 그림(0180, `render/help.rs`).
    pub pic: Pic,
    /// 보이는 조건. `None`은 처음부터(튜토리얼에서 배우는 것).
    pub gate: Option<fn(&Profile) -> bool>,
}

impl Topic {
    pub fn open(&self, p: &Profile) -> bool {
        self.gate.is_none_or(|g| g(p))
    }

    /// 잠겼다가 열린 주제를 아직 안 봤다: 목록에 "새" 점.
    pub fn fresh(&self, p: &Profile) -> bool {
        self.gate.is_some() && self.open(p) && !p.flag(&seen_key(self.id))
    }
}

pub fn seen_key(id: &str) -> String {
    format!("help.{id}")
}

/// 지금 기록에서 보이는 주제의 번호(표 순서).
pub fn visible(p: &Profile) -> Vec<usize> {
    (0..TOPICS.len()).filter(|&i| TOPICS[i].open(p)).collect()
}

/// 첫 런에 나섰다(튜토리얼 뒤 뿌리 길로 내려가 굴을 만난다).
fn ran(p: &Profile) -> bool {
    p.count(key::RUNS) > 0
}
/// 마을에 와 봤다(튜토리얼을 마치면 마을로 온다).
fn village(p: &Profile) -> bool {
    p.flag(key::TUTORIAL)
}
/// 모모나 도토를 고를 수 있다.
fn runner(p: &Profile) -> bool {
    p.flag(key::MOMO_BACK) || p.flag(key::DOTO_MEMORY)
}
fn doto(p: &Profile) -> bool {
    p.flag(key::DOTO_MEMORY)
}
/// 할매가 마을에 왔다(실뜨기·실의 꿈을 고르는 곳, 0269 D10 — 그 전엔 숨긴다).
fn halmae(p: &Profile) -> bool {
    p.flag(key::HALF_ENDING) || p.flag(key::TRUE_ENDING)
}

/// 안내 주제의 힌트 그림(0180): 진짜 그림(Naïl·생쥐·소품)으로 그 기능 한 장면을 되풀이한다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pic {
    Move,
    Combo,
    Heavy,
    Burrow,
    Parry,
    Special,
    Eat,
    Enemy,
    Loot,
    Camp,
    Gear,
    Break,
    /// 도토로 싸우기(0195): 던지기 셋·부채꼴, 뒤로 뛰기, 묻은 도토리.
    Doto,
    /// 달리기·멀리뛰기(0209): 모모가 깡충 → 달리기 → 멀리뛰기 착지 밟기.
    Run,
    /// 굴의 방(0257): 땅이 부풀어 무리가 솟는다(물결·버티기).
    Rooms,
    /// 방 표지(0259·0263): 문 안쪽 바닥에 새긴 보상 그림 넷, 붉은 겹테.
    Signs,
    /// 길 위의 것: 샘 · 상자 · 내려가는 구멍.
    Road,
    /// 위험물(0147·0165): 가시뿌리 덤불 · 무너지는 천장의 예고 · 흰 실 그물.
    Hazards,
    /// 길 지도 칸: 이어진 칸들(굴·샘·실타래·길의 주인).
    Nodes,
    /// 큰 나무(예전 `Village`): 반딧불이 나무로 올라간다.
    Tree,
    /// 발자국·짐: 발자국이 찍히고 짐 꾸러미가 쌓인다.
    Journal,
    /// 실뜨기·실의 꿈: 실 고리가 돈다.
    Knot,
    /// 실의 기억(0273 A): 흰 실 타래를 밟아 줍는다.
    Memory,
    /// 이웃의 부탁(0273 B): 굴 바닥의 부탁 물건을 밟아 줍는다(넷이 번갈아).
    Quest,
}

pub const TOPICS: [Topic; 24] = [
    Topic {
        id: "move",
        title: |s| s.h_move,
        body: |s| s.h_move_d,
        pic: Pic::Move,
        gate: None,
    },
    Topic {
        id: "combo",
        title: |s| s.h_combo,
        body: |s| s.h_combo_d,
        pic: Pic::Combo,
        gate: None,
    },
    Topic {
        id: "heavy",
        title: |s| s.h_heavy,
        body: |s| s.h_heavy_d,
        pic: Pic::Heavy,
        gate: None,
    },
    Topic {
        id: "burrow",
        title: |s| s.h_burrow,
        body: |s| s.h_burrow_d,
        pic: Pic::Burrow,
        gate: None,
    },
    Topic {
        id: "parry",
        title: |s| s.h_parry,
        body: |s| s.h_parry_d,
        pic: Pic::Parry,
        gate: None,
    },
    Topic {
        id: "special",
        title: |s| s.h_special,
        body: |s| s.h_special_d,
        pic: Pic::Special,
        gate: None,
    },
    Topic {
        id: "eat",
        title: |s| s.h_eat,
        body: |s| s.h_eat_d,
        pic: Pic::Eat,
        gate: None,
    },
    Topic {
        id: "enemy",
        title: |s| s.h_enemy,
        body: |s| s.h_enemy_d,
        pic: Pic::Enemy,
        gate: None,
    },
    // 모모·도토는 회피를 쥐면 달리고 떼면 멀리 뛴다(0209).
    Topic {
        id: "run",
        title: |s| s.h_run,
        body: |s| s.h_run_d,
        pic: Pic::Run,
        gate: Some(runner),
    },
    // 고를 수 있는 이웃(0195): 도토는 멀리서 싸운다 — 조작이 다른 캐릭터는 안내에 한 주제.
    Topic {
        id: "doto",
        title: |s| s.h_doto,
        body: |s| s.h_doto_d,
        pic: Pic::Doto,
        gate: Some(doto),
    },
    // 굴 안(0257 → 0269 2.6): 싸움방 넷 · 바닥 표지 · 길 위의 것 · 위험물 · 길 지도 칸. 같은 말을 두 주제에 쓰지 않는다.
    Topic {
        id: "rooms",
        title: |s| s.h_rooms,
        body: |s| s.h_rooms_d,
        pic: Pic::Rooms,
        gate: Some(ran),
    },
    Topic {
        id: "signs",
        title: |s| s.h_signs,
        body: |s| s.h_signs_d,
        pic: Pic::Signs,
        gate: Some(ran),
    },
    Topic {
        id: "road",
        title: |s| s.h_road,
        body: |s| s.h_road_d,
        pic: Pic::Road,
        gate: Some(ran),
    },
    Topic {
        id: "break",
        title: |s| s.h_break,
        body: |s| s.h_break_d,
        pic: Pic::Break,
        gate: Some(ran),
    },
    Topic {
        id: "hazards",
        title: |s| s.h_hazards,
        body: |s| s.h_hazards_d,
        pic: Pic::Hazards,
        gate: Some(ran),
    },
    Topic {
        id: "nodes",
        title: |s| s.h_nodes,
        body: |s| s.h_nodes_d,
        pic: Pic::Nodes,
        gate: Some(ran),
    },
    Topic {
        id: "camp",
        title: |s| s.h_camp,
        body: |s| s.h_camp_d,
        pic: Pic::Camp,
        gate: Some(ran),
    },
    Topic {
        id: "loot",
        title: |s| s.h_loot,
        body: |s| s.h_loot_d,
        pic: Pic::Loot,
        gate: None,
    },
    Topic {
        id: "gear",
        title: |s| s.h_gear,
        body: |s| s.h_gear_d,
        pic: Pic::Gear,
        gate: None,
    },
    // 마을의 규칙(0269 2.7): 큰 나무 · 발자국과 짐 · 실뜨기와 꿈(할매가 온 뒤). 장소 이야기는 수첩에.
    Topic {
        id: "tree",
        title: |s| s.h_tree,
        body: |s| s.h_tree_d,
        pic: Pic::Tree,
        gate: Some(village),
    },
    Topic {
        id: "journal",
        title: |s| s.h_journal,
        body: |s| s.h_journal_d,
        pic: Pic::Journal,
        gate: Some(village),
    },
    Topic {
        id: "knot",
        title: |s| s.h_knot,
        body: |s| s.h_knot_d,
        pic: Pic::Knot,
        gate: Some(halmae),
    },
    // 실의 기억(0273 A): 첫 조각을 주운 뒤에 열린다(정체·장면은 말하지 않고 줍는 규칙만).
    Topic {
        id: "memory",
        title: |s| s.h_memory,
        body: |s| s.h_memory_d,
        pic: Pic::Memory,
        gate: Some(|p| p.flag(&key::memory(1))),
    },
    // 이웃의 부탁(0273 B): 처음 부탁을 들은 뒤에 열린다.
    Topic {
        id: "quest",
        title: |s| s.h_quest,
        body: |s| s.h_quest_d,
        pic: Pic::Quest,
        gate: Some(|p| {
            (0..crate::quests::ALL.len())
                .any(|i| p.flag(&crate::quests::key_of(i, crate::quests::ASKED)))
        }),
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    #[test]
    fn topic_ids_are_unique() {
        let mut ids: Vec<&str> = TOPICS.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "안내 주제 id는 기록 키라 겹치면 안 된다");
    }

    #[test]
    fn gates_follow_progress() {
        let ids = |p: &Profile| -> Vec<&str> { visible(p).iter().map(|&i| TOPICS[i].id).collect() };
        let mut p = Profile::default();
        let first = ids(&p);
        assert!(first.contains(&"move") && first.contains(&"combo"));
        for hidden in ["rooms", "doto", "run", "tree", "knot"] {
            assert!(!first.contains(&hidden), "{hidden}는 처음엔 숨는다");
        }
        p.add(key::RUNS, 1);
        p.set(key::TUTORIAL);
        let s = ids(&p);
        assert!(s.contains(&"rooms") && s.contains(&"signs") && s.contains(&"tree"));
        assert!(!s.contains(&"knot"), "실뜨기는 할매가 온 뒤(0269 D10)");
        p.set(key::HALF_ENDING);
        assert!(ids(&p).contains(&"knot"));
        // "새" 점: 잠겼다 열린 주제만, 한 번 보면 사라진다.
        let rooms = TOPICS.iter().find(|t| t.id == "rooms").unwrap();
        assert!(rooms.fresh(&p));
        p.set(&seen_key("rooms"));
        assert!(!rooms.fresh(&p));
        assert!(!TOPICS[0].fresh(&p), "처음부터 보이는 주제엔 점이 없다");
    }

    /// 안내(규칙)와 마을 수첩(장소·이웃)은 같은 글을 쓰지 않는다(0269 R36).
    #[test]
    fn help_and_notes_share_no_text() {
        use crate::village_notes::NOTES;
        for &l in Lang::ALL {
            let s = l.strings();
            let help: Vec<&str> = TOPICS
                .iter()
                .flat_map(|t| [(t.title)(s), (t.body)(s)])
                .collect();
            for n in &NOTES {
                for text in [(n.title)(s), (n.body)(s)] {
                    assert!(
                        !help.contains(&text),
                        "{l:?}: 수첩 글이 안내에도 있다: {text}"
                    );
                }
            }
        }
    }
}
