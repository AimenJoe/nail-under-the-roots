//! 큰 나무 아래 마을(docs/decisions/0045): 런과 런 사이의 허브. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 튜토리얼을 마친 뒤 "시작"이 여기로 온다. 아래 뿌리 길(G)로 내려가면 런이 시작되고, 가장 깊은 곳에 닿으면 여기로 돌아온다.
//! - 위의 큰 나무(Y): 깬 횟수만큼 잎이 돌아온다(concept "나무가 조금씩 살아난다").
//! - 포포(K, 좌판 k): 마을 꾸미기 가게. 돈은 마을 씨앗(런이 끝나면 남은 씨앗 + 보너스).
//! - 보리(b, 할매 아래·통나무 의자 왼쪽 — 왼쪽 가게·포포 곁은 복작이고 오른쪽 끝은 구석이라, 사장 2026-10-28): 돌아온 뒤. 누룩 할매(H): 반쪽 엔딩 뒤, 실뜨기를 고른다. 빈 자리(M): 모모를 기다린다.
//! - 도토(d, 오른쪽 아래 그루터기 밑): 씨앗 자리를 기억해 낸 뒤(0052). 콩이(g, 오른쪽 위 바위 곁): 이긴 뒤(0054).
//! - 토끼풀 가족(j, 모모 자리 곁): 굴토끼 옛 굴에서 깨운 뒤(0165).
//! - Naïl(n, 큰 나무 아래 모닥불 곁 — 모모의 "큰 나무를 돌보고 있겠지"): 모모·도토로 마을을 걸을 때 — 주인공이 마을에서 사라지지 않게(0249).
//! - 꾸미기 자리 1~6: 산 것만 놓인다. 7~9: 저널 보상으로만(0048).
//! - 물웅덩이(W): 저널로 연 털빛을 비춰 본다(0048).
//! - 실 고리(X, 할매 곁): 할매가 온 뒤, 실의 꿈에서 이긴 길의 주인과 다시 싸운다(0055).

use crate::i18n::Tr;
use crate::map::{self, Parsed};

/// 위 벽 네 줄(0238): 새 큰 나무(키 약 86px, `art::TREE_*`)의 잎이 지도 위 끝에 잘리지 않게 두 줄을 더했다.
pub const ROWS: [&str; 16] = [
    "##########################",
    "##########################",
    "##########################",
    "##########################",
    "#..T.......#######..8..R.#",
    "#.....1....###Y###...g...#",
    "#.......W..........2.....#",
    "#..k.K....n.........H....#",
    "#..........F....b........#",
    "#....3............4...X..#",
    "#...M..j.................#",
    "#.......5..........6.T...#",
    "#..R........PG.......d...#",
    "#...................9....#",
    "##########################",
    "##########################",
];

pub fn parse() -> Parsed {
    map::parse(&ROWS)
}

/// 꾸미기 하나: 이름, 한 줄 설명(마을 창 자세히 칸, 0269), 마을 씨앗 값.
pub struct Decor {
    pub name: Tr,
    pub desc: Tr,
    pub price: u32,
}

pub const DECOR: [Decor; 6] = [
    Decor {
        name: |s| s.decor_1,
        desc: |s| s.decor_1_d,
        price: 5,
    },
    Decor {
        name: |s| s.decor_2,
        desc: |s| s.decor_2_d,
        price: 8,
    },
    Decor {
        name: |s| s.decor_3,
        desc: |s| s.decor_3_d,
        price: 8,
    },
    Decor {
        name: |s| s.decor_4,
        desc: |s| s.decor_4_d,
        price: 6,
    },
    Decor {
        name: |s| s.decor_5,
        desc: |s| s.decor_5_d,
        price: 10,
    },
    Decor {
        name: |s| s.decor_6,
        desc: |s| s.decor_6_d,
        price: 12,
    },
];

/// 짐 꾸리기(0259): 마을 씨앗으로 다음 런 가방에 넣어 둘 것. 꾸미기 말고도 마을 씨앗을 쓸 곳(사장 "꾸미기 외에 사용할 수 있어야 함").
/// 다음 런을 시작할 때 꺼내고 지운다(`App::unpack`). 한 런에 `max`까지.
pub struct Pack {
    pub kind: PackKind,
    pub name: Tr,
    pub desc: Tr,
    pub price: u32,
    pub max: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackKind {
    Berry,
    Worm,
    /// 씨앗 주머니: 런 씨앗 `PACK_POUCH_SEEDS`(쉼터 가게 돈). 값보다 조금 적게 — 런이 끝나면 남은 씨앗이 다시 마을 씨앗이 되니
    /// 오가며 불어나지 않게.
    Pouch,
}

pub const PACK: [Pack; 3] = [
    Pack {
        kind: PackKind::Berry,
        name: |s| s.it_berry,
        desc: |s| s.pack_berry_d,
        price: 3,
        max: 3,
    },
    Pack {
        kind: PackKind::Worm,
        name: |s| s.it_worm,
        desc: |s| s.pack_worm_d,
        price: 10,
        max: 1,
    },
    Pack {
        kind: PackKind::Pouch,
        name: |s| s.pack_pouch,
        desc: |s| s.pack_pouch_d,
        price: 6,
        max: 2,
    },
];

/// 씨앗 주머니 하나 = 런 씨앗 다섯.
pub const PACK_POUCH_SEEDS: u32 = 5;

/// 꾸려 둔 수의 기록 키.
pub fn key_pack(k: PackKind) -> &'static str {
    match k {
        PackKind::Berry => "pack.berry",
        PackKind::Worm => "pack.worm",
        PackKind::Pouch => "pack.pouch",
    }
}

/// 저널 보상으로만 얻는 꾸미기(번호 6~8, 자리 7~9, docs/decisions/0048): (옛 반딧불 병), 들꽃 화분, 실 그네.
/// 6번 반딧불 병은 0269에서 지웠다(코드 그림이 주변 그림과 맞지 않았다) — 지도 자리 '7'도 없고 어디서도 주지 않는다.
/// 번호는 기록 키(`decor.<i>`)라 줄은 자리만 남긴다(빼면 7·8번이 6·7번으로 밀린다).
pub const JOURNAL_DECOR: [Tr; 3] = [|s| s.decor_7, |s| s.decor_8, |s| s.decor_9];
/// 저널 꾸미기의 설명(6번 자리는 지운 병이라 비움).
pub const JOURNAL_DECOR_DESC: [Tr; 3] = [|_| "", |s| s.decor_8_d, |s| s.decor_9_d];
/// 마을 창에 보이는 저널 꾸미기 번호(지운 6번 병은 빼고, 0269).
pub const JOURNAL_SHOWN: [usize; 2] = [7, 8];
/// 꾸미기 전체 수(가게 + 저널).
pub const DECOR_ALL: usize = DECOR.len() + JOURNAL_DECOR.len();

/// 꾸미기 이름(가게 것이든 저널 것이든).
pub fn decor_name(i: usize) -> Tr {
    match i.checked_sub(DECOR.len()) {
        Some(j) => JOURNAL_DECOR[j.min(JOURNAL_DECOR.len() - 1)],
        None => DECOR[i].name,
    }
}

/// 마을 모습(앱이 기록에서 만들어 월드에 준다. 월드는 그리기·이웃 배치에만 쓴다).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub bori: bool,
    pub halmae: bool,
    /// 모모가 돌아와 빈 자리에 앉았다(0046).
    pub momo: bool,
    /// 도토가 씨앗 자리를 기억해 내고 마을에 왔다(0052).
    pub doto: bool,
    /// 콩이가 문을 열고 마을에 왔다(0054).
    pub kongi: bool,
    /// 토끼풀이 아기들과 마을에 왔다(0165): 모모 자리 곁.
    pub clover: bool,
    /// 다른 캐릭터로 걷는다: Naïl이 큰 나무 곁에 이웃으로 선다(0249).
    pub nail: bool,
    pub decor: [bool; DECOR_ALL],
    /// 큰 나무의 잎(0 = 거의 없음 … 4 = 꽃).
    pub growth: u8,
    /// 맡긴 반딧불로 깨울 수 있는 재주가 있다: 밑동이 빛난다(0061).
    pub tree_ready: bool,
    /// 맡긴(아직 쓰지 않은) 반딧불의 단계 0..4(`glow_level`): 반딧불 등이 그만큼 밝다(0269, 요구 2.9 A).
    pub glow: u8,
}

/// 맡긴 반딧불 수 → 빛의 단계(0 = 없음, 4 = 가득). 큰 나무를 다 깨운 뒤에도 쌓이는 빛이 마을에 보이게(0269 2.9 A).
/// 기준은 재주 한 칸 값(120~495, 0246)과 런 하나에 모이는 양(약 240, 0236)에서: 한 런·두 런·나무 한 칸 넘게·많이.
pub fn glow_level(banked: u32) -> u8 {
    match banked {
        0 => 0,
        1..=119 => 1,
        120..=399 => 2,
        400..=999 => 3,
        _ => 4,
    }
}

/// 기록 키.
pub fn key_decor(i: usize) -> String {
    format!("decor.{i}")
}
pub const KEY_SEEDS: &str = "village_seeds";

/// 깬 횟수 → 나무 잎 단계.
pub fn growth(clears: u32) -> u8 {
    clears.min(4) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Cell;

    #[test]
    fn village_has_gate_neighbors_tree_and_six_decor_slots() {
        let p = parse();
        assert_eq!(p.grid.find(Cell::Gate).len(), 1);
        assert!(p.enemies.is_empty());
        for m in [
            'K', 'b', 'M', 'H', 'Y', 'W', 'd', 'g', 'j', 'n', 'X', '1', '2', '3', '4', '5', '6',
            '8', '9',
        ] {
            // 'j'(토끼풀 가족, 0165)가 마커 목록에서 빠져 마을에 나오지 않았다 — 이웃 자리는 모두 여기서 본다.
            assert!(p.marks.iter().any(|(c, _)| *c == m), "{m}");
        }
        // 이웃 자리 바로 아래는 걸을 수 있다(말 걸 수 있게).
        for (c, at) in &p.marks {
            if matches!(c, 'K' | 'b' | 'H' | 'M' | 'n') {
                let (cx, cy) = ((at.x / 16.0) as usize, (at.y / 16.0) as usize);
                assert_eq!(p.grid.get(cx, cy + 1), Cell::Floor, "{c}");
            }
        }
        // 나무 자리는 막혀 있다.
        let y = p.marks.iter().find(|(c, _)| *c == 'Y').unwrap().1;
        assert_eq!(
            p.grid.get((y.x / 16.0) as usize, (y.y / 16.0) as usize),
            Cell::Wall
        );
    }

    #[test]
    fn growth_caps_at_four() {
        assert_eq!(growth(0), 0);
        assert_eq!(growth(2), 2);
        assert_eq!(growth(99), 4);
    }
}

#[cfg(test)]
mod glow_tests {
    /// 0269 2.9 A: 맡긴 빛이 많을수록 반딧불 등이 밝다 — 거꾸로 가지 않는다. 옛 반딧불 병 자리(7)는 지도에 없다.
    #[test]
    fn lantern_brightens_with_banked_light() {
        assert!(!super::parse().marks.iter().any(|(c, _)| *c == '7'));
        assert_eq!(super::glow_level(0), 0);
        assert_eq!(super::glow_level(1), 1);
        assert_eq!(super::glow_level(240), 2);
        assert_eq!(super::glow_level(5000), 4);
        let mut last = 0;
        for n in (0..3000).step_by(7) {
            let g = super::glow_level(n);
            assert!(g >= last);
            last = g;
        }
    }
}
