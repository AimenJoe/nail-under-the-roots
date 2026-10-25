//! 타일 격자와 ASCII 지도(docs/decisions/0019). 필드는 손으로 그린 ASCII, 던전은 생성기가 같은 격자를 채운다.
//!
//! 칸 16px. 글자:
//!   `#` 벽(벼랑)  `.` 바닥  `T` 그루터기  `R` 바위  `D` 던전 입구  `U` 출구(위로)  `S` 저장 지점  `+` 문
//!   `P` 시작 위치  `o` 스폰 지점(튜토리얼)  적: `c` 들쥐  `m` 생쥐 떼  `r` 시궁쥐  `s` 다람쥐  `q` 청설모  `B` 들쥐 대장(보스)
//!   쉼터(docs/decisions/0030): `G` 뿌리 길(길 지도)  `F` 모닥불  `k` 가게 좌판  `K` 포포 자리  `b` 보리 자리  `M` 빈 자리(통나무)
//!   마을(docs/decisions/0045): `Y` 큰 나무(벽) `H` 누룩 할매 자리 `1`~`6` 꾸미기 자리
//!   (마커 칸은 바닥, `Y`만 벽)

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::room::{Prop, PropKind};

pub const TILE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Floor,
    Wall,
    /// 던전 방의 문. 잠기면 벽처럼 막는다(Room::set_doors_locked).
    Door,
    /// 필드의 굴 입구 → 던전.
    Entrance,
    /// 던전에서 필드로 올라가는 구멍.
    Exit,
    /// 저장 지점(빛나는 버섯 고리). 닿으면 다시 시작할 곳이 된다.
    Save,
    /// 쉼터의 뿌리 길: 곁에서 확인하면 길 지도가 열린다(밟아도 바닥).
    Gate,
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn filled(w: usize, h: usize, c: Cell) -> Self {
        Self {
            w,
            h,
            cells: vec![c; w * h],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.w + x]
    }

    /// 범위 밖은 벽.
    pub fn get_i(&self, x: i32, y: i32) -> Cell {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            Cell::Wall
        } else {
            self.get(x as usize, y as usize)
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Cell) {
        if x < self.w && y < self.h {
            self.cells[y * self.w + x] = c;
        }
    }

    /// 이 종류의 칸 가운데 위치들.
    pub fn find(&self, c: Cell) -> Vec<Vec2> {
        let mut out = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.get(x, y) == c {
                    out.push(cell_center(x, y));
                }
            }
        }
        out
    }
}

pub fn cell_center(x: usize, y: usize) -> Vec2 {
    vec2((x as f32 + 0.5) * TILE, (y as f32 + 0.5) * TILE)
}

/// ASCII 지도를 읽은 결과.
pub struct Parsed {
    pub grid: Grid,
    pub props: Vec<Prop>,
    pub start: Vec2,
    pub spawn_points: Vec<Vec2>,
    pub enemies: Vec<(Kind, Vec2)>,
    /// 이웃·마을 자리 표시(`K` `b` `M` `H` `Y` `1`~`6`)와 그 칸 가운데.
    pub marks: Vec<(char, Vec2)>,
}

/// 줄 길이가 다르면 짧은 줄은 벽으로 채운다.
pub fn parse(rows: &[&str]) -> Parsed {
    let h = rows.len();
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let mut grid = Grid::filled(w, h, Cell::Wall);
    let mut props = Vec::new();
    let mut start = cell_center(1, 1);
    let mut spawn_points = Vec::new();
    let mut enemies = Vec::new();
    let mut marks = Vec::new();
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let at = cell_center(x, y);
            let rect = Rect::new(x as f32 * TILE, y as f32 * TILE, TILE, TILE);
            let cell = match ch {
                '#' => Cell::Wall,
                'D' => Cell::Entrance,
                'U' => Cell::Exit,
                'S' => Cell::Save,
                '+' => Cell::Door,
                'G' => Cell::Gate,
                'Y' => Cell::Wall,
                _ => Cell::Floor,
            };
            grid.set(x, y, cell);
            match ch {
                'T' => props.push(Prop {
                    rect,
                    kind: PropKind::Stump,
                }),
                'R' => props.push(Prop {
                    rect,
                    kind: PropKind::Rock,
                }),
                'P' => start = at,
                'o' => spawn_points.push(at),
                'c' => enemies.push((Kind::FieldMouse, at)),
                'm' => enemies.push((Kind::HouseMouse, at)),
                'r' => enemies.push((Kind::SewerRat, at)),
                's' => enemies.push((Kind::Chipmunk, at)),
                'q' => enemies.push((Kind::RedSquirrel, at)),
                'B' => enemies.push((Kind::Chief, at)),
                'F' | 'k' | 'M' => props.push(Prop {
                    rect,
                    kind: match ch {
                        'F' => PropKind::Campfire,
                        'k' => PropKind::Stall,
                        _ => PropKind::Seat,
                    },
                }),
                _ => {}
            }
            if matches!(ch, 'K' | 'b' | 'M' | 'H' | 'Y' | '1'..='6') {
                marks.push((ch, at));
            }
        }
    }
    Parsed {
        grid,
        props,
        start,
        spawn_points,
        enemies,
        marks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cells_markers_and_props() {
        let p = parse(&["#####", "#P.c#", "#oTD#", "#####"]);
        assert_eq!((p.grid.w, p.grid.h), (5, 4));
        assert_eq!(p.grid.get(0, 0), Cell::Wall);
        assert_eq!(p.grid.get(3, 2), Cell::Entrance);
        assert_eq!(p.grid.get(2, 2), Cell::Floor, "소품 칸은 바닥 + 소품");
        assert_eq!(p.props.len(), 1);
        assert_eq!(p.start, cell_center(1, 1));
        assert_eq!(p.enemies.len(), 1);
        assert_eq!(p.spawn_points.len(), 1);
        assert_eq!(p.grid.get_i(-1, 0), Cell::Wall);
    }
}
