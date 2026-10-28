//! 안내 화면의 주제 표(0178): 일시정지 메뉴·타이틀의 "안내". 튜토리얼을 지난 뒤에도 기능을 찾아볼 수 있게.
//! 글은 `i18n`(열한 언어). 새 기능을 넣으면 여기에 한 줄 + 열한 언어 두 키.

use crate::i18n::Tr;

pub struct Topic {
    pub title: Tr,
    pub body: Tr,
    /// 설명 위의 움직이는 힌트 그림(0180, `render/help.rs`).
    pub pic: Pic,
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
    Village,
    /// 도토로 싸우기(0195): 던지기 셋·부채꼴, 뒤로 뛰기, 묻은 도토리.
    Doto,
    /// 달리기·멀리뛰기(0209): 모모가 깡충 → 달리기 → 멀리뛰기 착지 밟기.
    Run,
}

pub const TOPICS: [Topic; 15] = [
    Topic {
        title: |s| s.h_move,
        body: |s| s.h_move_d,
        pic: Pic::Move,
    },
    Topic {
        title: |s| s.h_combo,
        body: |s| s.h_combo_d,
        pic: Pic::Combo,
    },
    Topic {
        title: |s| s.h_heavy,
        body: |s| s.h_heavy_d,
        pic: Pic::Heavy,
    },
    Topic {
        title: |s| s.h_burrow,
        body: |s| s.h_burrow_d,
        pic: Pic::Burrow,
    },
    Topic {
        title: |s| s.h_parry,
        body: |s| s.h_parry_d,
        pic: Pic::Parry,
    },
    Topic {
        title: |s| s.h_special,
        body: |s| s.h_special_d,
        pic: Pic::Special,
    },
    Topic {
        title: |s| s.h_eat,
        body: |s| s.h_eat_d,
        pic: Pic::Eat,
    },
    Topic {
        title: |s| s.h_enemy,
        body: |s| s.h_enemy_d,
        pic: Pic::Enemy,
    },
    Topic {
        title: |s| s.h_loot,
        body: |s| s.h_loot_d,
        pic: Pic::Loot,
    },
    Topic {
        title: |s| s.h_camp,
        body: |s| s.h_camp_d,
        pic: Pic::Camp,
    },
    Topic {
        title: |s| s.h_gear,
        body: |s| s.h_gear_d,
        pic: Pic::Gear,
    },
    Topic {
        title: |s| s.h_break,
        body: |s| s.h_break_d,
        pic: Pic::Break,
    },
    Topic {
        title: |s| s.h_village,
        body: |s| s.h_village_d,
        pic: Pic::Village,
    },
    // 고를 수 있는 이웃(0195): 도토는 멀리서 싸운다 — 조작이 다른 캐릭터는 안내에 한 주제.
    Topic {
        title: |s| s.h_doto,
        body: |s| s.h_doto_d,
        pic: Pic::Doto,
    },
    // 모모·도토는 회피를 쥐면 달리고 떼면 멀리 뛴다(0209).
    Topic {
        title: |s| s.h_run,
        body: |s| s.h_run_d,
        pic: Pic::Run,
    },
];
