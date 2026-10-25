//! 쉼터(docs/decisions/0030): 길이 끝날 때마다 내려오는 뿌리 밑 빈터. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 싸움이 없는 곳. 들어오면 체력이 가득 차고, 저장 고리(S)가 쓰러졌을 때 돌아올 곳이 된다.
//! 모닥불(F) 둘레에 이웃이 모인다: 포포의 가게(k 좌판, K 포포), 되돌아온 보리(b), 아직 빈 자리(M — 모모가 돌아올 곳).
//! 아래쪽 뿌리 길(G) 곁에서 확인을 누르면 길 지도가 열린다. 화면보다 조금 넓어서(26×14칸) 살짝 걷는다.

use crate::map::{self, Parsed};

pub const ROWS: [&str; 14] = [
    "##########################",
    "##########################",
    "#.........#######........#",
    "#..T......#######...R....#",
    "#........................#",
    "#..k.K......P............#",
    "#..........F.......b.....#",
    "#.......S................#",
    "#...M....................#",
    "#.................T......#",
    "#..R.........G...........#",
    "#........................#",
    "##########################",
    "##########################",
];

pub fn parse() -> Parsed {
    map::parse(&ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Cell;

    #[test]
    fn camp_has_save_gate_shop_and_seats() {
        let p = parse();
        assert_eq!(p.grid.find(Cell::Save).len(), 1);
        assert_eq!(p.grid.find(Cell::Gate).len(), 1);
        assert!(p.enemies.is_empty(), "쉼터엔 적이 없다");
        for m in ['K', 'b', 'M'] {
            assert!(p.marks.iter().any(|(c, _)| *c == m), "{m}");
        }
        // 이웃 자리 바로 곁은 걸을 수 있다(말 걸 수 있게).
        for (_, at) in &p.marks {
            let (cx, cy) = ((at.x / 16.0) as usize, (at.y / 16.0) as usize);
            assert_eq!(p.grid.get(cx, cy + 1), Cell::Floor);
        }
    }
}
