//! 실의 기억(0273 A): 누룩 할매의 지난날 여섯 조각 — 표 한 곳. 굴에서 줍는 장치는 `world`(`StoryItem::Memory`·`memory_tick`),
//! 장면은 `cutscene::memory`, 다시 보기는 앨범(`app/album.rs`)이 이 표를 읽는다.
//!
//! 줄기: 첫 숲 → 마른 여름 → 혼자 → 잊힌 도토리 하나가 싹(큰 나무) → 다시 외롭지 않음 → 큰 나무가 늙자 그 여름이 무서워 실이 물듦.
//! 참 엔딩(도토가 잊은 씨앗이 새싹)과 고리를 이룬다. 반쪽 엔딩(정체를 본 기록) 뒤에만 나온다.

use crate::i18n::Tr;
use crate::illust::Illust;
use crate::profile::{key, Profile};

pub struct Memory {
    /// 장면의 그림. 새 그림(0273 프롬프트)이 오면 바꾼다 — 지금은 있는 그림.
    pub illust: Illust,
    pub text: Tr,
}

pub const ALL: [Memory; crate::world::MEMORIES as usize] = [
    Memory {
        illust: Illust::MemForest,
        text: |s| s.mem_1,
    },
    Memory {
        illust: Illust::HalmaeDeep,
        text: |s| s.mem_2,
    },
    Memory {
        illust: Illust::HalmaeDeep,
        text: |s| s.mem_3,
    },
    Memory {
        illust: Illust::DotoMemory,
        text: |s| s.mem_4,
    },
    Memory {
        illust: Illust::GreatTree,
        text: |s| s.mem_5,
    },
    Memory {
        illust: Illust::HalmaeDeep,
        text: |s| s.mem_6,
    },
];

/// n번째(1..=6) 조각.
pub fn get(n: u8) -> &'static Memory {
    &ALL[(n.clamp(1, crate::world::MEMORIES) - 1) as usize]
}

pub fn seen(p: &Profile, n: u8) -> bool {
    p.flag(&key::memory(n))
}

/// 이번 런에 놓을 다음 조각: 정체를 본 기록(반쪽·참 엔딩)에서, 아직 안 본 가장 앞 것. 차례는 줍는 순서로.
pub fn next(p: &Profile) -> Option<u8> {
    if !(p.flag(key::HALF_ENDING) || p.flag(key::TRUE_ENDING)) {
        return None;
    }
    (1..=crate::world::MEMORIES).find(|n| !seen(p, *n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_memory_key_matches_the_table() {
        assert_eq!(key::memory(crate::world::MEMORIES), key::MEMORY_LAST);
    }

    #[test]
    fn memories_wait_for_the_ending_and_come_in_order() {
        let mut p = Profile::default();
        assert_eq!(next(&p), None, "정체를 보기 전에는 없다");
        p.set(key::HALF_ENDING);
        assert_eq!(next(&p), Some(1));
        p.set(&key::memory(1));
        p.set(&key::memory(2));
        assert_eq!(next(&p), Some(3));
        for n in 3..=crate::world::MEMORIES {
            p.set(&key::memory(n));
        }
        assert_eq!(next(&p), None, "다 보면 더 없다");
    }
}
