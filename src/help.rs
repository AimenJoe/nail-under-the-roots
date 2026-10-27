//! 안내 화면의 주제 표(0178): 일시정지 메뉴·타이틀의 "안내". 튜토리얼을 지난 뒤에도 기능을 찾아볼 수 있게.
//! 글은 `i18n`(열한 언어). 새 기능을 넣으면 여기에 한 줄 + 열한 언어 두 키.

use crate::i18n::Tr;

pub struct Topic {
    pub title: Tr,
    pub body: Tr,
}

pub const TOPICS: [Topic; 13] = [
    Topic {
        title: |s| s.h_move,
        body: |s| s.h_move_d,
    },
    Topic {
        title: |s| s.h_combo,
        body: |s| s.h_combo_d,
    },
    Topic {
        title: |s| s.h_heavy,
        body: |s| s.h_heavy_d,
    },
    Topic {
        title: |s| s.h_burrow,
        body: |s| s.h_burrow_d,
    },
    Topic {
        title: |s| s.h_parry,
        body: |s| s.h_parry_d,
    },
    Topic {
        title: |s| s.h_special,
        body: |s| s.h_special_d,
    },
    Topic {
        title: |s| s.h_eat,
        body: |s| s.h_eat_d,
    },
    Topic {
        title: |s| s.h_enemy,
        body: |s| s.h_enemy_d,
    },
    Topic {
        title: |s| s.h_loot,
        body: |s| s.h_loot_d,
    },
    Topic {
        title: |s| s.h_camp,
        body: |s| s.h_camp_d,
    },
    Topic {
        title: |s| s.h_gear,
        body: |s| s.h_gear_d,
    },
    Topic {
        title: |s| s.h_break,
        body: |s| s.h_break_d,
    },
    Topic {
        title: |s| s.h_village,
        body: |s| s.h_village_d,
    },
];
