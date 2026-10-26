//! 지금 걷는 곳의 모양: 경계, 충돌 사각형(벽·소품), 그리는 소품, (있으면) 타일 격자.
//!
//! - 아레나(`Room::new`): 화면 하나짜리 방. 웨이브·개발 시나리오의 시험장. 격자 없이 벽은 경계, 기둥 두 개.
//! - 격자 방(`Room::from_grid`): 필드·던전(docs/decisions/0019). 16px 칸, 벽 칸은 줄마다 이어 붙여 사각형으로.
//!   잠긴 문 칸은 `set_doors_locked`로 막았다 연다.
//!
//! 충돌은 원 vs 사각형 하나뿐이다. 다른 모듈은 `resolve_circle`·`is_solid`·`cell_at`만 쓴다.

use macroquad::prelude::*;

use crate::map::{Cell, Grid, TILE};
use crate::tuning;

/// 그리는 소품. 그루터기·바위는 막고, 상자는 막고(열면 비킴), 나머지 칸 종류는 격자에 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Stump,
    Rock,
    /// 보물 상자. 닿으면 열린다(던전 보물방).
    Chest {
        opened: bool,
    },
    /// 쉼터의 모닥불(docs/decisions/0030).
    Campfire,
    /// 쉼터 포포의 가게 좌판.
    Stall,
    /// 쉼터의 빈 자리(통나무). 돌아올 이웃의 자리.
    Seat,
}

#[derive(Debug, Clone, Copy)]
pub struct Prop {
    /// 바닥에 닿는 16×16 칸(충돌·정렬 기준). 그림은 위로 더 크다.
    pub rect: Rect,
    pub kind: PropKind,
}

impl Prop {
    pub fn solid(&self) -> bool {
        true
    }
}

pub struct Room {
    /// 걸을 수 있는 바깥 경계. 아레나는 벽 안쪽, 격자 방은 지도 전체.
    pub inner: Rect,
    /// 막는 사각형(벽 칸 묶음 + 소품).
    pub solids: Vec<Rect>,
    pub props: Vec<Prop>,
    pub spawn_points: Vec<Vec2>,
    pub grid: Option<Grid>,
    doors_locked: bool,
    /// 잠긴 문 그림이 얼마나 자랐나(0 = 열림 .. 1 = 다 막힘, 0094). 잠그면 자라고 열면 걷힌다.
    pub door_anim: f32,
    /// 지난 틱에 잠겨 있었나(열리는 순간을 알기 위해).
    door_was_locked: bool,
}

impl Room {
    /// 아레나: 화면 하나. 위쪽은 3/4 벼랑(WALL_TOP), 기둥 두 개.
    pub fn new() -> Self {
        let w = tuning::WALL;
        let pillars = [
            Rect::new(104.0, 56.0, 16.0, 16.0),
            Rect::new(200.0, 108.0, 16.0, 16.0),
        ];
        Self {
            inner: Rect::new(
                w,
                tuning::WALL_TOP,
                tuning::VIEW_W - 2.0 * w,
                tuning::VIEW_H - tuning::WALL_TOP - w,
            ),
            solids: pillars.to_vec(),
            props: vec![
                Prop {
                    rect: pillars[0],
                    kind: PropKind::Stump,
                },
                Prop {
                    rect: pillars[1],
                    kind: PropKind::Rock,
                },
            ],
            spawn_points: vec![
                vec2(40.0, 40.0),
                vec2(280.0, 40.0),
                vec2(40.0, 140.0),
                vec2(280.0, 140.0),
                vec2(160.0, 40.0),
                vec2(160.0, 152.0),
            ],
            grid: None,
            doors_locked: false,
            door_anim: 0.0,
            door_was_locked: false,
        }
    }

    /// 격자 방(필드·던전).
    pub fn from_grid(grid: Grid, props: Vec<Prop>, spawn_points: Vec<Vec2>) -> Self {
        let inner = Rect::new(0.0, 0.0, grid.w as f32 * TILE, grid.h as f32 * TILE);
        let mut r = Self {
            inner,
            solids: Vec::new(),
            props,
            spawn_points,
            grid: Some(grid),
            doors_locked: false,
            door_anim: 0.0,
            door_was_locked: false,
        };
        r.rebuild_solids();
        r
    }

    /// 벽 칸을 줄마다 이어 붙여 사각형으로. 잠긴 문도 벽처럼.
    fn rebuild_solids(&mut self) {
        let mut solids = Vec::new();
        if let Some(g) = &self.grid {
            for y in 0..g.h {
                let mut x = 0;
                while x < g.w {
                    let blocks =
                        |c: Cell| c == Cell::Wall || (c == Cell::Door && self.doors_locked);
                    if blocks(g.get(x, y)) {
                        let s = x;
                        while x < g.w && blocks(g.get(x, y)) {
                            x += 1;
                        }
                        solids.push(Rect::new(
                            s as f32 * TILE,
                            y as f32 * TILE,
                            (x - s) as f32 * TILE,
                            TILE,
                        ));
                    } else {
                        x += 1;
                    }
                }
            }
        }
        solids.extend(self.props.iter().filter(|p| p.solid()).map(|p| p.rect));
        self.solids = solids;
    }

    /// 던전 방의 문을 잠그거나 연다(격자 방만).
    /// 문 그림 시계를 한 틱 돌린다. 방금 열렸으면 true(월드가 풀리는 실·흙먼지를 낸다).
    pub fn tick_doors(&mut self, dt: f32) -> bool {
        self.door_anim = if self.doors_locked {
            (self.door_anim + dt / tuning::DOOR_CLOSE_TIME).min(1.0)
        } else {
            (self.door_anim - dt / tuning::DOOR_OPEN_TIME).max(0.0)
        };
        let opened = self.door_was_locked && !self.doors_locked;
        self.door_was_locked = self.doors_locked;
        opened
    }

    /// 문 칸 가운데(월드 좌표). 문이 열릴 때 효과를 낼 자리.
    pub fn door_cells(&self) -> Vec<Vec2> {
        let Some(g) = &self.grid else {
            return Vec::new();
        };
        let mut v = Vec::new();
        for y in 0..g.h {
            for x in 0..g.w {
                if g.get(x, y) == Cell::Door {
                    v.push(vec2(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0));
                }
            }
        }
        v
    }

    pub fn set_doors_locked(&mut self, locked: bool) {
        if self.doors_locked != locked {
            self.doors_locked = locked;
            self.rebuild_solids();
        }
    }

    pub fn doors_locked(&self) -> bool {
        self.doors_locked
    }

    /// 상자를 연다(그림만 바뀌고 계속 막는다).
    pub fn open_chest(&mut self, i: usize) {
        if let Some(p) = self.props.get_mut(i) {
            p.kind = PropKind::Chest { opened: true };
        }
    }

    pub fn center(&self) -> Vec2 {
        self.inner.center()
    }

    /// 이 위치의 칸(격자 방만).
    pub fn cell_at(&self, p: Vec2) -> Option<Cell> {
        let g = self.grid.as_ref()?;
        if p.x < 0.0 || p.y < 0.0 {
            return None;
        }
        let (x, y) = ((p.x / TILE) as usize, (p.y / TILE) as usize);
        (x < g.w && y < g.h).then(|| g.get(x, y))
    }

    /// 원을 벽·기둥 밖으로 밀어낸다. 무언가에 닿았으면 true.
    pub fn resolve_circle(&self, pos: &mut Vec2, r: f32) -> bool {
        let mut hit = false;
        let clamped = vec2(
            pos.x.clamp(self.inner.x + r, self.inner.right() - r),
            pos.y.clamp(self.inner.y + r, self.inner.bottom() - r),
        );
        if clamped != *pos {
            *pos = clamped;
            hit = true;
        }
        for s in &self.solids {
            hit |= push_out_rect(pos, r, *s);
        }
        hit
    }

    /// `from`에서 `dir`(단위 벡터)로 반지름 `r`인 원이 막히지 않고 갈 수 있는 거리(최대 `max`). 공격 예고 길이에 쓴다.
    pub fn ray_len(&self, from: Vec2, dir: Vec2, r: f32, max: f32) -> f32 {
        let step = 2.0;
        let mut d = 0.0;
        while d < max {
            let next = (d + step).min(max);
            if self.is_solid(from + dir * next, r) {
                return d;
            }
            d = next;
        }
        max
    }

    /// 점이 벽/기둥 안에 있는가(투사체용).
    pub fn is_solid(&self, p: Vec2, r: f32) -> bool {
        if p.x - r < self.inner.x
            || p.x + r > self.inner.right()
            || p.y - r < self.inner.y
            || p.y + r > self.inner.bottom()
        {
            return true;
        }
        self.solids.iter().any(|rect| {
            let c = vec2(
                p.x.clamp(rect.x, rect.right()),
                p.y.clamp(rect.y, rect.bottom()),
            );
            (p - c).length_squared() < r * r
        })
    }
}

/// 원을 사각형 밖으로 민다. 겹쳤으면 true.
fn push_out_rect(pos: &mut Vec2, r: f32, rect: Rect) -> bool {
    let closest = vec2(
        pos.x.clamp(rect.x, rect.right()),
        pos.y.clamp(rect.y, rect.bottom()),
    );
    let d = *pos - closest;
    let dist2 = d.length_squared();
    if dist2 >= r * r {
        return false;
    }
    if dist2 > 1e-6 {
        let dist = dist2.sqrt();
        *pos += d / dist * (r - dist);
    } else {
        // 중심이 사각형 안: 가장 얕은 축으로 밀어낸다.
        let left = pos.x - rect.x;
        let right = rect.right() - pos.x;
        let top = pos.y - rect.y;
        let bottom = rect.bottom() - pos.y;
        let m = left.min(right).min(top).min(bottom);
        if m == left {
            pos.x = rect.x - r;
        } else if m == right {
            pos.x = rect.right() + r;
        } else if m == top {
            pos.y = rect.y - r;
        } else {
            pos.y = rect.bottom() + r;
        }
    }
    true
}
