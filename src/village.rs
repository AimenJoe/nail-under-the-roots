//! 큰 나무 아래 마을(docs/decisions/0045): 런과 런 사이의 허브. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 튜토리얼을 마친 뒤 "시작"이 여기로 온다. 아래 뿌리 길(G)로 내려가면 런이 시작되고, 가장 깊은 곳에 닿으면 여기로 돌아온다.
//! - 위의 큰 나무(Y): 깬 횟수만큼 잎이 돌아온다(concept "나무가 조금씩 살아난다").
//! - 포포(K, 좌판 k): 마을 꾸미기 가게. 돈은 마을 씨앗(런이 끝나면 남은 씨앗 + 보너스).
//! - 보리(b, 모닥불 F): 돌아온 뒤. 누룩 할매(H): 반쪽 엔딩 뒤, 실뜨기를 고른다. 빈 자리(M): 모모를 기다린다.
//! - 도토(d, 오른쪽 아래 그루터기 밑): 씨앗 자리를 기억해 낸 뒤(0052). 콩이(g, 오른쪽 위 바위 곁): 이긴 뒤(0054).
//! - 꾸미기 자리 1~6: 산 것만 놓인다. 7~9: 저널 보상으로만(0048).
//! - 물웅덩이(W): 저널로 연 털빛을 비춰 본다(0048).
//! - 실 고리(X, 할매 곁): 할매가 온 뒤, 실의 꿈에서 이긴 길의 주인과 다시 싸운다(0055).

use crate::i18n::Tr;
use crate::map::{self, Parsed};

pub const ROWS: [&str; 14] = [
    "##########################",
    "##########################",
    "#..T.......#######..8..R.#",
    "#.....1....###Y###...g...#",
    "#.......W..........2.....#",
    "#..k.K..............H....#",
    "#..........F.......b.....#",
    "#....3............4...X..#",
    "#...M....................#",
    "#.......5..........6.T...#",
    "#..R........PG.......d...#",
    "#.....7.............9....#",
    "##########################",
    "##########################",
];

pub fn parse() -> Parsed {
    map::parse(&ROWS)
}

/// 꾸미기 하나: 이름, 마을 씨앗 값.
pub struct Decor {
    pub name: Tr,
    pub price: u32,
}

pub const DECOR: [Decor; 6] = [
    Decor {
        name: |s| s.decor_1,
        price: 5,
    },
    Decor {
        name: |s| s.decor_2,
        price: 8,
    },
    Decor {
        name: |s| s.decor_3,
        price: 8,
    },
    Decor {
        name: |s| s.decor_4,
        price: 6,
    },
    Decor {
        name: |s| s.decor_5,
        price: 10,
    },
    Decor {
        name: |s| s.decor_6,
        price: 12,
    },
];

/// 저널 보상으로만 얻는 꾸미기(번호 6~8, 자리 7~9, docs/decisions/0048): 반딧불 병, 들꽃 화분, 실 그네.
pub const JOURNAL_DECOR: [Tr; 3] = [|s| s.decor_7, |s| s.decor_8, |s| s.decor_9];
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
    pub decor: [bool; DECOR_ALL],
    /// 큰 나무의 잎(0 = 거의 없음 … 4 = 꽃).
    pub growth: u8,
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
            'K', 'b', 'M', 'H', 'Y', 'W', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ] {
            assert!(p.marks.iter().any(|(c, _)| *c == m), "{m}");
        }
        // 이웃 자리 바로 아래는 걸을 수 있다(말 걸 수 있게).
        for (c, at) in &p.marks {
            if matches!(c, 'K' | 'b' | 'H' | 'M') {
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
