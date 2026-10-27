//! 쉼터(docs/decisions/0030): 길이 끝날 때마다 내려오는 뿌리 밑 빈터. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 싸움이 없는 곳. 들어오면 체력이 가득 차고, 저장 고리(S)가 쓰러졌을 때 돌아올 곳이 된다.
//! 모닥불(F) 둘레에 이웃이 모인다: 포포의 가게(k 좌판, K 포포), 되돌아온 보리(b), 아직 빈 자리(M — 모모가 돌아올 곳).
//! 첫 쉼터에만 땅강아지 장인 수수(z, 그루터기 곁, 0053).
//! 아래쪽 뿌리 길(G) 곁에서 확인을 누르면 길 지도가 열린다. 화면보다 조금 넓어서(26×14칸) 살짝 걷는다.
//!
//! 쉼터마다 지도가 다르다(0151): 첫 쉼터 = 뿌리 빈터(`ROWS`), 둘째 = 뿌리 동굴(`ROWS_2`), 셋째(마지막 길 앞) = 버섯 숲(`ROWS_3`, 심층).
//! 지도마다 구석에 숨은 자리(*)가 하나: 반딧불 한 줌과 씨앗. 기호는 같아서 이웃·가게·저장·뿌리 길 자리는 지도가 정한다.

use crate::biome::Biome;
use crate::map::{self, Parsed};

pub const ROWS: [&str; 14] = [
    "##########################",
    "##########################",
    "#.........#######.......*#",
    "#..T......#######...R....#",
    "#........................#",
    "#..k.K......P............#",
    "#..........F.......b.....#",
    "#.......S................#",
    "#...M....................#",
    "#.................T..z...#",
    "#..R.........G...........#",
    "#........................#",
    "##########################",
    "##########################",
];

/// 둘째 쉼터: 뿌리 동굴. 오른쪽 아래 바위 너머(아래로 돌아 들어가는) 작은 굴에 숨은 자리.
pub const ROWS_2: [&str; 14] = [
    "##########################",
    "##########################",
    "#####.........##.........#",
    "#R...........####....T...#",
    "#.....k.K.........R......#",
    "#..............P.........#",
    "#.T.......F......b.......#",
    "#..................#######",
    "#....S......M......#..*..#",
    "#..................R.....#",
    "#.....R.....G............#",
    "###.................T....#",
    "##########################",
    "##########################",
];

/// 셋째 쉼터(마지막 길 앞): 버섯 숲(심층). 왼쪽 위 좁은 틈으로 들어가는 방에 숨은 자리.
pub const ROWS_3: [&str; 14] = [
    "##########################",
    "##########################",
    "#..*.#####.......#########",
    "#....#####..T....#########",
    "##.###.....k.K...........#",
    "##.......................#",
    "###....P.....F.....b.....#",
    "##.......................#",
    "#...........M..R.....T...#",
    "#...######..........######",
    "#...######...S......######",
    "#.T.......G..............#",
    "##########################",
    "##########################",
];

/// `stage`번째 쉼터(1부터)의 지도. 셋째 뒤(가장 깊은 곳 앞 등)는 셋째 지도.
pub fn parse(stage: u32) -> Parsed {
    match stage {
        0 | 1 => map::parse(&ROWS),
        2 => map::parse(&ROWS_2),
        _ => map::parse(&ROWS_3),
    }
}

/// 쉼터의 지대(바닥색·빛·입자). 마지막 길 앞 버섯 숲만 심층.
pub fn biome(stage: u32) -> Biome {
    if stage >= 3 {
        Biome::Deep
    } else {
        Biome::Roots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Cell;

    #[test]
    fn camp_has_save_gate_shop_and_seats() {
        for stage in 1..=3 {
            let p = parse(stage);
            assert_eq!(p.grid.find(Cell::Save).len(), 1, "쉼터 {stage}");
            assert_eq!(p.grid.find(Cell::Gate).len(), 1, "쉼터 {stage}");
            assert!(p.enemies.is_empty(), "쉼터엔 적이 없다");
            for m in ['K', 'b', 'M', '*'] {
                assert!(p.marks.iter().any(|(c, _)| *c == m), "쉼터 {stage}: {m}");
            }
            // 이웃 자리 바로 곁은 걸을 수 있다(말 걸 수 있게).
            for (c, at) in &p.marks {
                let (cx, cy) = ((at.x / 16.0) as usize, (at.y / 16.0) as usize);
                if *c != '*' {
                    assert_eq!(p.grid.get(cx, cy + 1), Cell::Floor, "쉼터 {stage}: {c}");
                }
            }
        }
    }

    /// 지도를 고쳐도(0151) 시작 자리에서 저장·뿌리 길·숨은 자리·이웃 곁에 모두 걸어서 닿는다(소품은 막는다).
    #[test]
    fn every_camp_spot_is_reachable() {
        for stage in 1..=3 {
            let p = parse(stage);
            let (w, h) = (p.grid.w, p.grid.h);
            let cell = |v: macroquad::prelude::Vec2| ((v.x / 16.0) as usize, (v.y / 16.0) as usize);
            let blocked: Vec<(usize, usize)> =
                p.props.iter().map(|q| cell(q.rect.center())).collect();
            let open = |x: usize, y: usize| {
                x < w && y < h && p.grid.get(x, y) != Cell::Wall && !blocked.contains(&(x, y))
            };
            let mut seen = vec![false; w * h];
            let s = cell(p.start);
            let mut todo = vec![s];
            seen[s.1 * w + s.0] = true;
            while let Some((x, y)) = todo.pop() {
                for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                    if open(nx, ny) && !seen[ny * w + nx] {
                        seen[ny * w + nx] = true;
                        todo.push((nx, ny));
                    }
                }
            }
            // 숨은 자리는 그 칸, 이웃·빈 자리는 말 거는 앞 칸(빈 자리 M은 소품이라 막는다).
            let mut spots: Vec<(usize, usize)> = p
                .marks
                .iter()
                .map(|(c, at)| {
                    let (x, y) = cell(*at);
                    if *c == '*' {
                        (x, y)
                    } else {
                        (x, y + 1)
                    }
                })
                .collect();
            spots.extend(p.grid.find(Cell::Save).into_iter().map(cell));
            spots.extend(p.grid.find(Cell::Gate).into_iter().map(cell));
            for (x, y) in spots {
                assert!(seen[y * w + x], "쉼터 {stage}: ({x},{y})에 못 간다");
            }
        }
    }
}
