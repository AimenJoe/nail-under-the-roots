//! 이야기 조각(0273 C): 저널 보상 `Reward::Tale`로 받는 이웃의 옛이야기 한 장 — 그림 + 제목 + 두세 줄.
//! 힘이 아닌 보상이라 경제에 닿지 않는다. 받으면 기록 `tale.<id>`, 다시 보기는 앨범(`app/album.rs`)이 이 표를 읽는다.
//! 아이디는 바꾸지 않는다(기록 키). 새 조각 = 표 한 줄 + 글 둘(`tale_<id>`·`tale_<id>_t`) + 그것을 주는 목표.

use crate::i18n::Tr;
use crate::illust::Illust;
use crate::profile::Profile;

pub struct Tale {
    /// 기록 키 `tale.<id>`.
    pub id: &'static str,
    /// 칸·보기의 그림(그 이웃이 나오는 있는 그림).
    pub illust: Illust,
    pub title: Tr,
    pub text: Tr,
}

pub const POPO: usize = 0;
pub const BORI: usize = 1;
pub const MOMO: usize = 2;
pub const DOTO: usize = 3;
pub const KONGI: usize = 4;
pub const CLOVER: usize = 5;

pub const ALL: [Tale; 6] = [
    Tale {
        id: "popo",
        illust: Illust::GreatTree,
        title: |s| s.tale_popo_t,
        text: |s| s.tale_popo,
    },
    Tale {
        id: "bori",
        illust: Illust::BoriFreed,
        title: |s| s.tale_bori_t,
        text: |s| s.tale_bori,
    },
    Tale {
        id: "momo",
        illust: Illust::MomoBloom,
        title: |s| s.tale_momo_t,
        text: |s| s.tale_momo,
    },
    Tale {
        id: "doto",
        illust: Illust::DotoMemory,
        title: |s| s.tale_doto_t,
        text: |s| s.tale_doto,
    },
    Tale {
        id: "kongi",
        illust: Illust::KongiDoor,
        title: |s| s.tale_kongi_t,
        text: |s| s.tale_kongi,
    },
    Tale {
        id: "clover",
        illust: Illust::CloverCocoon,
        title: |s| s.tale_clover_t,
        text: |s| s.tale_clover,
    },
];

pub fn key(i: usize) -> String {
    format!("tale.{}", ALL[i].id)
}

pub fn owned(p: &Profile, i: usize) -> bool {
    p.flag(&key(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tale_ids_are_unique_and_named_consts_match() {
        for (i, t) in ALL.iter().enumerate() {
            assert!(ALL[..i].iter().all(|u| u.id != t.id), "{}", t.id);
        }
        for (i, id) in [
            (POPO, "popo"),
            (BORI, "bori"),
            (MOMO, "momo"),
            (DOTO, "doto"),
            (KONGI, "kongi"),
            (CLOVER, "clover"),
        ] {
            assert_eq!(ALL[i].id, id);
        }
    }
}
