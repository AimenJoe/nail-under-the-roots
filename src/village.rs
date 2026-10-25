//! 큰 나무 아래 마을(docs/decisions/0045): 런과 런 사이의 허브. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 튜토리얼을 마친 뒤 "시작"이 여기로 온다. 아래 뿌리 길(G)로 내려가면 런이 시작되고, 가장 깊은 곳에 닿으면 여기로 돌아온다.
//! - 위의 큰 나무(Y): 깬 횟수만큼 잎이 돌아온다(concept "나무가 조금씩 살아난다").
//! - 포포(K, 좌판 k): 마을 꾸미기 가게. 돈은 마을 씨앗(런이 끝나면 남은 씨앗 + 보너스).
//! - 보리(b, 모닥불 F): 돌아온 뒤. 누룩 할매(H): 반쪽 엔딩 뒤, 실뜨기를 고른다. 빈 자리(M): 모모를 기다린다.
//! - 꾸미기 자리 1~6: 산 것만 놓인다.

use crate::i18n::Tr;
use crate::map::{self, Parsed};

pub const ROWS: [&str; 14] = [
    "##########################",
    "##########################",
    "#..T.......#######.....R.#",
    "#.....1....###Y###.......#",
    "#..................2.....#",
    "#..k.K..............H....#",
    "#..........F.......b.....#",
    "#....3............4......#",
    "#...M....................#",
    "#.......5..........6.T...#",
    "#..R........PG...........#",
    "#........................#",
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

/// 마을 모습(앱이 기록에서 만들어 월드에 준다. 월드는 그리기·이웃 배치에만 쓴다).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub bori: bool,
    pub halmae: bool,
    pub decor: [bool; 6],
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
        for m in ['K', 'b', 'M', 'H', 'Y', '1', '2', '3', '4', '5', '6'] {
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
