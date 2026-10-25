//! 절차적 던전(docs/decisions/0019). 필드의 굴 입구로 들어갈 때마다 시드로 새로 만든다.
//!
//! - 방 = 20×12칸(320×192, 화면 하나 + 위아래 6px). 5×5 칸에 방을 붙여 놓은 큰 격자 하나가 던전 전체.
//!   카메라는 지금 방 안에만 머물고, 문을 지나면 옆 방으로 미끄러진다(밀기 전환).
//! - 방 그래프: 시작방에서 무작위로 가지를 뻗는다(나무) + 고리 하나. 시작에서 가장 먼 방 = 보스,
//!   막다른 방 하나 = 보물(장신구 상자), 다른 막다른 방 = 휴식(산딸기). 나머지는 전투.
//! - 전투방: 안으로 들어서면 문이 닫히고(Room::set_doors_locked), 다 잡으면 열린다. 보스방을 비우면 출구(U)와 뿌리 심장.
//! - 시작방에는 필드로 돌아가는 출구가 있다.
//!
//! 같은 시드 → 같은 던전(테스트·버그 재현).

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::map::{cell_center, Cell, Grid};
use crate::room::{Prop, PropKind, Room};

pub const ROOM_W: usize = 20;
pub const ROOM_H: usize = 12;
const GRID: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomKind {
    Start,
    Combat,
    Treasure,
    Rest,
    Boss,
}

/// 문 방향: 위, 오른쪽, 아래, 왼쪽.
pub const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Debug, Clone)]
pub struct DRoom {
    pub at: (i32, i32),
    pub kind: RoomKind,
    pub links: [bool; 4],
    pub cleared: bool,
    pub visited: bool,
    /// 들어설 때 나오는 적(월드 좌표).
    pub enemies: Vec<(Kind, Vec2)>,
}

impl DRoom {
    /// 이 방의 월드 사각형.
    pub fn rect(&self) -> Rect {
        room_rect(self.at)
    }

    /// 문이 닫히는 안쪽(벽에서 한 칸 들어온 곳). 여기 들어서면 잠근다.
    pub fn interior(&self) -> Rect {
        let r = self.rect();
        Rect::new(r.x + 24.0, r.y + 40.0, r.w - 48.0, r.h - 64.0)
    }
}

pub fn room_rect(at: (i32, i32)) -> Rect {
    Rect::new(
        at.0 as f32 * ROOM_W as f32 * 16.0,
        at.1 as f32 * ROOM_H as f32 * 16.0,
        ROOM_W as f32 * 16.0,
        ROOM_H as f32 * 16.0,
    )
}

/// 월드 좌표 → 방 칸.
pub fn room_at(p: Vec2) -> (i32, i32) {
    (
        (p.x / (ROOM_W as f32 * 16.0)).floor() as i32,
        (p.y / (ROOM_H as f32 * 16.0)).floor() as i32,
    )
}

pub struct Dungeon {
    pub rooms: Vec<DRoom>,
    pub start: (i32, i32),
    pub depth: u32,
    pub goal: Goal,
    /// 보스방의 주인(있으면 무리 대신 혼자, 0046).
    pub boss: Option<Kind>,
}

impl Dungeon {
    pub fn room(&self, at: (i32, i32)) -> Option<&DRoom> {
        self.rooms.iter().find(|r| r.at == at)
    }

    pub fn room_mut(&mut self, at: (i32, i32)) -> Option<&mut DRoom> {
        self.rooms.iter_mut().find(|r| r.at == at)
    }

    /// 시작방에서 처음 서는 곳(출구 아래).
    pub fn start_pos(&self) -> Vec2 {
        let r = room_rect(self.start);
        vec2(r.center().x, r.y + 8.5 * 16.0)
    }
}

/// 작은 결정적 난수(xorshift). 전역 rand와 섞이지 않게.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1) ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    /// 0..1.
    pub fn unit(&mut self) -> f32 {
        (self.next() % 10_000) as f32 / 10_000.0
    }
    pub fn chance(&mut self, p: f32) -> bool {
        (self.next() % 10_000) as f32 / 10_000.0 < p
    }
}

/// 끝방(시작에서 가장 먼 방)이 무엇인가. 끝방을 마치면 출구(U)가 열린다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// 보스방: 큰 무리 + 뿌리 심장 + 씨앗.
    Boss,
    /// 끝 무리: 다 잡으면 씨앗 한 줌(길 지도의 굴·둥지).
    Pack,
    /// 상자방이 끝(저장고). 출구는 처음부터.
    Treasure,
    /// 샘(쉼방)이 끝. 싸움 없이 체력을 채운다. 출구는 처음부터.
    Spring,
}

/// 굴 모양(길 지도의 칸 종류가 정한다, route.rs).
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub depth: u32,
    /// 방 수(시작방 포함).
    pub rooms: usize,
    pub goal: Goal,
    /// 막다른 방 하나를 보물방으로.
    pub treasure: bool,
    /// 막다른 방 하나를 쉼방으로.
    pub rest: bool,
    /// 적 뽑기에 더하는 깊이(둥지 = 센 무리).
    pub tough: u32,
    /// 시작방에 돌아가는 출구(필드에서 들어온 던전만).
    pub start_exit: bool,
    /// 보스방에 무리 대신 이 보스 혼자(첫 길의 모모, 0046).
    pub boss: Option<Kind>,
}

impl Params {
    pub fn new(depth: u32) -> Self {
        Self {
            depth,
            rooms: 4,
            goal: Goal::Pack,
            treasure: false,
            rest: false,
            tough: 0,
            start_exit: false,
            boss: None,
        }
    }

    /// 필드의 굴 입구로 들어가는 예전 던전(개발 시나리오 "던전 바로").
    pub fn classic(depth: u32) -> Self {
        Self {
            rooms: 7 + depth.min(3) as usize,
            goal: Goal::Boss,
            treasure: true,
            rest: true,
            start_exit: true,
            ..Self::new(depth)
        }
    }
}

/// 던전 하나를 만든다: (던전 상태, 그 격자로 지은 Room).
pub fn generate(seed: u64, prm: &Params) -> (Dungeon, Room) {
    let mut rng = Rng::new(seed);
    let start = (GRID / 2, GRID - 1);
    let depth = prm.depth;
    let target = prm.rooms.max(2);
    let mut rooms: Vec<DRoom> = vec![DRoom {
        at: start,
        kind: RoomKind::Start,
        links: [false; 4],
        cleared: true,
        visited: true,
        enemies: Vec::new(),
    }];
    // 가지 뻗기
    let mut guard = 0;
    while rooms.len() < target && guard < 1000 {
        guard += 1;
        let i = rng.below(rooms.len() as u64) as usize;
        let d = rng.below(4) as usize;
        let (dx, dy) = DIRS[d];
        let at = (rooms[i].at.0 + dx, rooms[i].at.1 + dy);
        if at.0 < 0 || at.1 < 0 || at.0 >= GRID || at.1 >= GRID || rooms.iter().any(|r| r.at == at)
        {
            continue;
        }
        rooms[i].links[d] = true;
        let mut links = [false; 4];
        links[(d + 2) % 4] = true;
        rooms.push(DRoom {
            at,
            kind: RoomKind::Combat,
            links,
            cleared: false,
            visited: false,
            enemies: Vec::new(),
        });
    }
    // 고리 하나: 이웃인데 안 이어진 쌍(큰 굴만)
    'outer: for _ in 0..if target >= 5 { 20 } else { 0 } {
        let i = rng.below(rooms.len() as u64) as usize;
        let d = rng.below(4) as usize;
        let (dx, dy) = DIRS[d];
        let at = (rooms[i].at.0 + dx, rooms[i].at.1 + dy);
        if let Some(j) = rooms.iter().position(|r| r.at == at) {
            if !rooms[i].links[d] {
                rooms[i].links[d] = true;
                rooms[j].links[(d + 2) % 4] = true;
                break 'outer;
            }
        }
    }
    // 시작에서의 거리(BFS) → 보스
    let dist = distances(&rooms, start);
    let boss = (0..rooms.len()).max_by_key(|&i| dist[i]).unwrap_or(0);
    rooms[boss].kind = match prm.goal {
        Goal::Boss | Goal::Pack => RoomKind::Boss,
        Goal::Treasure => RoomKind::Treasure,
        Goal::Spring => RoomKind::Rest,
    };
    if matches!(prm.goal, Goal::Treasure | Goal::Spring) {
        rooms[boss].cleared = true;
    }
    let leaves: Vec<usize> = (0..rooms.len())
        .filter(|&i| {
            i != boss
                && rooms[i].kind == RoomKind::Combat
                && rooms[i].links.iter().filter(|l| **l).count() == 1
        })
        .collect();
    let mut leaf = leaves.iter().copied();
    if prm.treasure {
        if let Some(t) = leaf.next() {
            rooms[t].kind = RoomKind::Treasure;
            rooms[t].cleared = true;
        }
    }
    if prm.rest {
        if let Some(r) = leaf.next() {
            rooms[r].kind = RoomKind::Rest;
            rooms[r].cleared = true;
        }
    }

    let goal_at = rooms[boss].at;
    // 격자
    let mut grid = Grid::filled(GRID as usize * ROOM_W, GRID as usize * ROOM_H, Cell::Wall);
    let mut props = Vec::new();
    for room in &mut rooms {
        let (ox, oy) = (room.at.0 as usize * ROOM_W, room.at.1 as usize * ROOM_H);
        for y in 2..ROOM_H - 1 {
            for x in 1..ROOM_W - 1 {
                grid.set(ox + x, oy + y, Cell::Floor);
            }
        }
        let [n, e, s, w] = room.links;
        if n {
            for y in 0..2 {
                grid.set(ox + 9, oy + y, Cell::Door);
                grid.set(ox + 10, oy + y, Cell::Door);
            }
        }
        if s {
            grid.set(ox + 9, oy + ROOM_H - 1, Cell::Door);
            grid.set(ox + 10, oy + ROOM_H - 1, Cell::Door);
        }
        if w {
            grid.set(ox, oy + 6, Cell::Door);
            grid.set(ox, oy + 7, Cell::Door);
        }
        if e {
            grid.set(ox + ROOM_W - 1, oy + 6, Cell::Door);
            grid.set(ox + ROOM_W - 1, oy + 7, Cell::Door);
        }
        let is_goal = room.at == goal_at;
        // 끝방이 상자방·샘이면 출구는 처음부터(구석, 문 앞 길을 막지 않게).
        if is_goal && matches!(prm.goal, Goal::Treasure | Goal::Spring) {
            grid.set(ox + 16, oy + 3, Cell::Exit);
        }
        // 방 모양(0063): 모서리·가장자리 혹·바위 무리. 굴 모양 난수와 따로라 같은 시드면 방 배치·적은 그대로다.
        shape_room(&mut grid, ox, oy, room.kind, seed ^ hash_at(room.at));
        match room.kind {
            RoomKind::Start => {
                if prm.start_exit {
                    grid.set(ox + 3, oy + 3, Cell::Exit);
                }
            }
            RoomKind::Treasure => props.push(Prop {
                rect: cell_rect(ox + 9, oy + 6),
                kind: PropKind::Chest { opened: false },
            }),
            RoomKind::Combat | RoomKind::Boss => {
                // 소품: 문 앞 길(가운데 십자)과 가장자리를 피해서.
                let n_props = 1 + rng.below(3) as usize;
                for _ in 0..n_props {
                    let x = 2 + rng.below(16) as usize;
                    let y = 3 + rng.below(7) as usize;
                    let lane = (8..=11).contains(&x) || (5..=8).contains(&y);
                    // 방 모양이 벽으로 만든 칸에는 놓지 않는다(난수는 그대로 써서 뒤의 적 배치가 같다).
                    if !lane && grid.get(ox + x, oy + y) == Cell::Floor {
                        props.push(Prop {
                            rect: cell_rect(ox + x, oy + y),
                            kind: if rng.chance(0.5) {
                                PropKind::Rock
                            } else {
                                PropKind::Stump
                            },
                        });
                    }
                }
                let count = match (room.kind, prm.goal) {
                    (RoomKind::Boss, Goal::Boss) => 5 + depth as usize,
                    (RoomKind::Boss, _) => 3 + depth.min(2) as usize + prm.tough as usize,
                    _ => 2 + depth.min(2) as usize + rng.below(2) as usize,
                };
                for k in 0..count {
                    let kind = crate::species::roll(rng.unit(), depth + prm.tough);
                    // 방 가장자리 쪽에(가운데는 들어오는 플레이어 자리).
                    let corner = [
                        (4, 4),
                        (15, 4),
                        (4, 9),
                        (15, 9),
                        (10, 3),
                        (10, 10),
                        (3, 7),
                        (16, 7),
                    ];
                    let (cx, cy) = corner[(k + rng.below(8) as usize) % corner.len()];
                    room.enemies.push((kind, cell_center(ox + cx, oy + cy)));
                }
                // 보스가 정해져 있으면 무리 대신 혼자, 방 가운데 위쪽에. 난수는 위에서 그대로 써서
                // 같은 시드면 굴 모양이 보스가 있든 없든 같다.
                if let (RoomKind::Boss, Some(k)) = (room.kind, prm.boss) {
                    room.enemies = vec![(k, cell_center(ox + 10, oy + 4))];
                }
            }
            RoomKind::Rest => {}
        }
    }
    let dungeon = Dungeon {
        rooms,
        start,
        depth,
        goal: prm.goal,
        boss: prm.boss,
    };
    (dungeon, Room::from_grid(grid, props, Vec::new()))
}

/// 적·보스·상자·출구·시작 자리(방 안 칸). 방 모양이 이 칸과 그 둘레 한 칸을 벽으로 만들지 않는다.
const KEEP: [(usize, usize); 14] = [
    (4, 4),
    (15, 4),
    (4, 9),
    (15, 9),
    (10, 3),
    (10, 10),
    (3, 7),
    (16, 7),
    (10, 4),
    (9, 6),
    (16, 3),
    (3, 3),
    (10, 6),
    (10, 8),
];

fn hash_at(at: (i32, i32)) -> u64 {
    (at.0 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (at.1 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
}

/// 네모난 방을 굴처럼(0063): 모서리를 메우고, 가장자리에 1~2칸 혹을 내고, 가운데 쪽에 바위 무리를 하나 둔다.
/// 문 앞 십자 길(가로 5~8줄, 세로 8~11칸)과 `KEEP` 자리(둘레 한 칸까지)는 건드리지 않는다.
/// 다 만든 뒤 모든 바닥이 방 가운데에서 이어지는지 보고, 안 이어지면 네모로 되돌린다.
fn shape_room(grid: &mut Grid, ox: usize, oy: usize, kind: RoomKind, seed: u64) {
    let mut rng = Rng::new(seed | 1);
    let lane = |x: usize, y: usize| (8..=11).contains(&x) || (5..=8).contains(&y);
    let kept = |x: usize, y: usize| {
        KEEP.iter()
            .any(|&(kx, ky)| kx.abs_diff(x) <= 1 && ky.abs_diff(y) <= 1)
    };
    let free = |x: usize, y: usize| {
        (1..=18).contains(&x) && (2..=10).contains(&y) && !lane(x, y) && !kept(x, y)
    };
    let before: Vec<Cell> = (0..ROOM_H)
        .flat_map(|y| (0..ROOM_W).map(move |x| (x, y)))
        .map(|(x, y)| grid.get(ox + x, oy + y))
        .collect();
    let mut wall = |x: usize, y: usize| {
        if free(x, y) {
            grid.set(ox + x, oy + y, Cell::Wall);
        }
    };
    // 모서리: 한 칸은 늘, 계단 한 칸 더는 반쯤.
    for (cx, cy, sx, sy) in [
        (1, 2, 1i32, 1i32),
        (18, 2, -1, 1),
        (1, 10, 1, -1),
        (18, 10, -1, -1),
    ] {
        wall(cx, cy);
        if rng.chance(0.6) {
            wall((cx as i32 + sx) as usize, cy);
        }
        if rng.chance(0.6) {
            wall(cx, (cy as i32 + sy) as usize);
        }
    }
    // 가장자리 혹: 위·아래 가장자리는 가로로, 왼·오른쪽은 세로로 2~4칸, 깊이 1(가끔 2).
    for edge in 0..4 {
        let n = 1 + rng.below(3);
        for _ in 0..n {
            let len = 2 + rng.below(3) as usize;
            let deep = if rng.chance(0.3) { 2 } else { 1 };
            match edge {
                0 | 1 => {
                    let x0 = 2 + rng.below(14) as usize;
                    for x in x0..(x0 + len).min(18) {
                        for d in 0..deep {
                            let y = if edge == 0 { 2 + d } else { 10 - d };
                            wall(x, y);
                        }
                    }
                }
                _ => {
                    let y0 = 3 + rng.below(6) as usize;
                    for y in y0..(y0 + len).min(10) {
                        for d in 0..deep {
                            let x = if edge == 2 { 1 + d } else { 18 - d };
                            wall(x, y);
                        }
                    }
                }
            }
        }
    }
    // 바위 무리(싸우는 방·보스방): 사분면 하나에 2~4칸.
    if matches!(kind, RoomKind::Combat | RoomKind::Boss) && rng.chance(0.7) {
        let (qx, qy) = (rng.below(2), rng.below(2));
        let (bx, by) = (
            if qx == 0 { 4 } else { 13 } + rng.below(3) as usize,
            if qy == 0 { 3 } else { 8 } + rng.below(2) as usize,
        );
        wall(bx, by);
        for _ in 0..1 + rng.below(3) {
            let (dx, dy) = DIRS[rng.below(4) as usize];
            wall((bx as i32 + dx) as usize, (by as i32 + dy) as usize);
        }
    }
    // 모든 바닥이 가운데에서 이어지는가.
    let open = |g: &Grid, x: usize, y: usize| g.get(ox + x, oy + y) != Cell::Wall;
    let mut seen = vec![false; ROOM_W * ROOM_H];
    let mut q = std::collections::VecDeque::from([(10usize, 6usize)]);
    seen[6 * ROOM_W + 10] = true;
    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in DIRS {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx < 0 || ny < 0 || nx >= ROOM_W as i32 || ny >= ROOM_H as i32 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if !seen[ny * ROOM_W + nx] && open(grid, nx, ny) {
                seen[ny * ROOM_W + nx] = true;
                q.push_back((nx, ny));
            }
        }
    }
    let cut_off =
        (0..ROOM_H).any(|y| (0..ROOM_W).any(|x| open(grid, x, y) && !seen[y * ROOM_W + x]));
    if cut_off {
        for y in 0..ROOM_H {
            for x in 0..ROOM_W {
                grid.set(ox + x, oy + y, before[y * ROOM_W + x]);
            }
        }
    }
}

fn cell_rect(x: usize, y: usize) -> Rect {
    Rect::new(x as f32 * 16.0, y as f32 * 16.0, 16.0, 16.0)
}

fn distances(rooms: &[DRoom], from: (i32, i32)) -> Vec<u32> {
    let mut dist = vec![u32::MAX; rooms.len()];
    let idx = |at: (i32, i32)| rooms.iter().position(|r| r.at == at);
    let Some(s) = idx(from) else { return dist };
    dist[s] = 0;
    let mut q = std::collections::VecDeque::from([s]);
    while let Some(i) = q.pop_front() {
        for (d, (dx, dy)) in DIRS.iter().enumerate() {
            if !rooms[i].links[d] {
                continue;
            }
            if let Some(j) = idx((rooms[i].at.0 + dx, rooms[i].at.1 + dy)) {
                if dist[j] == u32::MAX {
                    dist[j] = dist[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_boss_replaces_the_pack_and_keeps_the_layout() {
        let prm = Params {
            goal: Goal::Boss,
            ..Params::new(1)
        };
        let named = Params {
            boss: Some(Kind::Momo),
            ..prm
        };
        let (a, _) = generate(7, &prm);
        let (b, _) = generate(7, &named);
        let at = |d: &Dungeon| d.rooms.iter().map(|r| r.at).collect::<Vec<_>>();
        assert_eq!(at(&a), at(&b), "같은 시드 = 같은 굴");
        let boss = b.rooms.iter().find(|r| r.kind == RoomKind::Boss).unwrap();
        assert_eq!(boss.enemies.len(), 1);
        assert_eq!(boss.enemies[0].0, Kind::Momo);
        assert_eq!(b.boss, Some(Kind::Momo));
    }

    #[test]
    fn generates_connected_dungeon_with_one_boss() {
        for seed in 1..40 {
            let (d, room) = generate(seed, &Params::classic(1));
            assert!(d.rooms.len() >= 8, "seed {seed}");
            assert_eq!(
                d.rooms.iter().filter(|r| r.kind == RoomKind::Boss).count(),
                1
            );
            let dist = distances(&d.rooms, d.start);
            assert!(
                dist.iter().all(|x| *x != u32::MAX),
                "모든 방이 이어져야 한다 seed {seed}"
            );
            // 이어진 방 사이에는 문이 있고, 문 칸은 양쪽 방에 붙어 있다.
            let g = room.grid.as_ref().unwrap();
            for r in &d.rooms {
                let (ox, oy) = (r.at.0 as usize * ROOM_W, r.at.1 as usize * ROOM_H);
                if r.links[1] {
                    assert_eq!(g.get(ox + ROOM_W - 1, oy + 6), Cell::Door);
                    assert_eq!(g.get(ox + ROOM_W, oy + 6), Cell::Door, "옆 방 왼쪽 문");
                }
            }
            // 시작 위치는 바닥.
            assert_eq!(room.cell_at(d.start_pos()), Some(Cell::Floor));
        }
    }

    /// 방 모양(0063): 적·상자·출구 자리와 문 앞 길은 바닥이고, 방 안 바닥은 다 이어지며, 대부분 방은 네모가 아니다.
    #[test]
    fn shaped_rooms_keep_spawns_and_stay_connected() {
        let mut shaped = 0;
        let mut total = 0;
        for seed in 1..60 {
            let (d, room) = generate(seed, &Params::classic(2));
            let g = room.grid.as_ref().unwrap();
            for r in &d.rooms {
                let (ox, oy) = (r.at.0 as usize * ROOM_W, r.at.1 as usize * ROOM_H);
                for &(x, y) in &KEEP {
                    assert_ne!(
                        g.get(ox + x, oy + y),
                        Cell::Wall,
                        "seed {seed} 방 {:?} 자리 ({x}, {y})",
                        r.at
                    );
                }
                for (_, p) in &r.enemies {
                    assert_ne!(room.cell_at(*p), Some(Cell::Wall), "적은 바닥에");
                }
                // 문 앞 십자 길.
                for x in 1..19 {
                    for y in 5..=8 {
                        assert_ne!(g.get(ox + x, oy + y), Cell::Wall, "가로 길 seed {seed}");
                    }
                }
                let walls = (2..=10)
                    .flat_map(|y| (1..=18).map(move |x| (x, y)))
                    .filter(|&(x, y)| g.get(ox + x, oy + y) == Cell::Wall)
                    .count();
                total += 1;
                if walls > 0 {
                    shaped += 1;
                }
            }
        }
        assert!(
            shaped * 10 >= total * 8,
            "대부분 방이 네모가 아니다: {shaped}/{total}"
        );
    }

    #[test]
    fn same_seed_same_dungeon() {
        let (a, _) = generate(7, &Params::classic(2));
        let (b, _) = generate(7, &Params::classic(2));
        let sig = |d: &Dungeon| {
            d.rooms
                .iter()
                .map(|r| (r.at, r.kind, r.links))
                .collect::<Vec<_>>()
        };
        assert_eq!(sig(&a), sig(&b));
    }
}
