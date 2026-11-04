//! 마을 수첩(0269 R36, 요구 2.7): 마을의 장소·물건·이웃을 적은 표. 마을 창(`app/village_menu.rs`)의 셋째 탭이 읽는다.
//!
//! 나누는 기준: 안내(`help.rs`)는 조작·규칙·시스템, 수첩은 장소·이웃·분위기. 수첩에는 버튼·수치·규칙을 쓰지 않고,
//! 같은 글을 두 곳에 쓰지 않는다(시험 `help::tests::help_and_notes_share_no_text`).
//!
//! 본 것만 열린다: 이웃은 돌아온 뒤, 부탁의 흔적은 돌려준 뒤(0273 B), 할매와 실 고리는 할매가 마을에 온 뒤(엔딩 전 스포일러를 막는다 — 2.1과 같은 기준,
//! 정체를 말하지 않는 결로). 잠금은 이미 있는 기록 깃발만 쓴다(새 키 없음).

use crate::i18n::Tr;
use crate::profile::{key, Profile};

/// 수첩 한 줄이 가리키는 것. 그림을 고르는 데 쓴다(`render::VillagePic::Spot`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Tree,
    Fire,
    Gate,
    Puddle,
    /// 무무를 풀어 준 뒤 물웅덩이 자리(0275 P5).
    Spring,
    Stall,
    Seat,
    Bori,
    Momo,
    Doto,
    Kongi,
    Clover,
    Halmae,
    Loop,
    /// 들어준 부탁의 흔적(0273 B, `quests::ALL` 번호) — 그림은 마을의 흔적 그대로.
    Trace(u8),
}

impl Spot {
    /// 이웃이면 초상화로 그린다.
    pub fn neighbor(self) -> bool {
        matches!(
            self,
            Spot::Stall
                | Spot::Bori
                | Spot::Momo
                | Spot::Doto
                | Spot::Kongi
                | Spot::Clover
                | Spot::Halmae
        )
    }
}

pub struct Note {
    pub spot: Spot,
    pub title: Tr,
    pub body: Tr,
    /// 보이는 조건. `None`은 처음부터.
    pub gate: Option<fn(&Profile) -> bool>,
}

impl Note {
    pub fn open(&self, p: &Profile) -> bool {
        self.gate.is_none_or(|g| g(p))
    }
}

fn halmae(p: &Profile) -> bool {
    p.flag(key::HALF_ENDING) || p.flag(key::TRUE_ENDING)
}

pub const NOTES: [Note; 21] = [
    Note {
        spot: Spot::Tree,
        title: |s| s.vn_tree,
        body: |s| s.vn_tree_d,
        gate: None,
    },
    Note {
        spot: Spot::Fire,
        title: |s| s.vn_fire,
        body: |s| s.vn_fire_d,
        gate: None,
    },
    Note {
        spot: Spot::Gate,
        title: |s| s.vn_gate,
        body: |s| s.vn_gate_d,
        gate: None,
    },
    // 물웅덩이는 무무를 풀어 주면(0275 P5) 다시 솟는 샘의 줄로 바뀐다.
    Note {
        spot: Spot::Puddle,
        title: |s| s.vn_puddle,
        body: |s| s.vn_puddle_d,
        gate: Some(|p| !p.flag(key::MUMU_FREED)),
    },
    Note {
        spot: Spot::Spring,
        title: |s| s.vn_spring,
        body: |s| s.vn_spring_d,
        gate: Some(|p| p.flag(key::MUMU_FREED)),
    },
    Note {
        spot: Spot::Stall,
        title: |s| s.popo_name,
        body: |s| s.vn_popo_d,
        gate: None,
    },
    // 모모를 기다리는 빈 자리: 모모가 돌아오면 그 자리는 모모의 줄이 된다.
    Note {
        spot: Spot::Seat,
        title: |s| s.vn_seat,
        body: |s| s.vn_seat_d,
        gate: Some(|p| !p.flag(key::MOMO_BACK)),
    },
    Note {
        spot: Spot::Bori,
        title: |s| s.bori_name,
        body: |s| s.vn_bori_d,
        gate: Some(|p| p.flag(key::BORI_BACK)),
    },
    Note {
        spot: Spot::Momo,
        title: |s| s.name_momo,
        body: |s| s.vn_momo_d,
        gate: Some(|p| p.flag(key::MOMO_BACK)),
    },
    Note {
        spot: Spot::Doto,
        title: |s| s.name_doto,
        body: |s| s.vn_doto_d,
        gate: Some(|p| p.flag(key::DOTO_MEMORY)),
    },
    Note {
        spot: Spot::Kongi,
        title: |s| s.name_kongi,
        body: |s| s.vn_kongi_d,
        gate: Some(|p| p.flag(key::KONGI_BACK)),
    },
    Note {
        spot: Spot::Clover,
        title: |s| s.name_clover,
        body: |s| s.vn_clover_d,
        gate: Some(|p| p.flag(key::CLOVER_BACK)),
    },
    Note {
        spot: Spot::Halmae,
        title: |s| s.name_halmae,
        body: |s| s.vn_halmae_d,
        gate: Some(halmae),
    },
    Note {
        spot: Spot::Loop,
        title: |s| s.vn_loop,
        body: |s| s.vn_loop_d,
        gate: Some(halmae),
    },
    // 들어준 부탁의 흔적(0273 B): 돌려준 뒤에만.
    Note {
        spot: Spot::Trace(0),
        title: |s| s.vn_tr_popo,
        body: |s| s.vn_tr_popo_d,
        gate: Some(|p| crate::quests::done(p, 0)),
    },
    Note {
        spot: Spot::Trace(1),
        title: |s| s.vn_tr_bori,
        body: |s| s.vn_tr_bori_d,
        gate: Some(|p| crate::quests::done(p, 1)),
    },
    Note {
        spot: Spot::Trace(2),
        title: |s| s.vn_tr_momo,
        body: |s| s.vn_tr_momo_d,
        gate: Some(|p| crate::quests::done(p, 2)),
    },
    Note {
        spot: Spot::Trace(3),
        title: |s| s.vn_tr_doto,
        body: |s| s.vn_tr_doto_d,
        gate: Some(|p| crate::quests::done(p, 3)),
    },
    Note {
        spot: Spot::Trace(4),
        title: |s| s.vn_tr_susu,
        body: |s| s.vn_tr_susu_d,
        gate: Some(|p| crate::quests::done(p, 4)),
    },
    Note {
        spot: Spot::Trace(5),
        title: |s| s.vn_tr_kongi,
        body: |s| s.vn_tr_kongi_d,
        gate: Some(|p| crate::quests::done(p, 5)),
    },
    Note {
        spot: Spot::Trace(6),
        title: |s| s.vn_tr_clover,
        body: |s| s.vn_tr_clover_d,
        gate: Some(|p| crate::quests::done(p, 6)),
    },
];

/// 지금 기록에서 보이는 수첩 줄(표 순서).
pub fn visible(p: &Profile) -> Vec<usize> {
    (0..NOTES.len()).filter(|&i| NOTES[i].open(p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbors_and_halmae_wait_for_their_flags() {
        let mut p = Profile::default();
        let spots =
            |p: &Profile| -> Vec<Spot> { visible(p).iter().map(|&i| NOTES[i].spot).collect() };
        let first = spots(&p);
        assert!(first.contains(&Spot::Tree) && first.contains(&Spot::Seat));
        for hidden in [Spot::Bori, Spot::Momo, Spot::Halmae, Spot::Loop] {
            assert!(!first.contains(&hidden), "{hidden:?}는 처음엔 안 보인다");
        }
        p.set(key::MOMO_BACK);
        let s = spots(&p);
        assert!(
            s.contains(&Spot::Momo) && !s.contains(&Spot::Seat),
            "모모가 오면 빈 자리 대신 모모"
        );
        p.set(key::HALF_ENDING);
        let s = spots(&p);
        assert!(s.contains(&Spot::Halmae) && s.contains(&Spot::Loop));
    }
}
