//! 절차적 던전(docs/decisions/0019). 필드의 굴 입구로 들어갈 때마다 시드로 새로 만든다.
//!
//! - 방 = 20×12칸(320×192, 화면 하나 + 위아래 6px). 5×5 칸에 방을 붙여 놓은 큰 격자 하나가 던전 전체.
//!   카메라는 지금 방 안에만 머물고, 문을 지나면 옆 방으로 미끄러진다(밀기 전환).
//! - 방 그래프: 시작방에서 무작위로 가지를 뻗는다(나무) + 고리 하나. 시작에서 가장 먼 방 = 보스,
//!   막다른 방 하나 = 보물(장신구 상자), 다른 막다른 방 = 휴식(산딸기). 나머지는 전투.
//! - 전투방: 안으로 들어서면 문이 닫히고(Room::set_doors_locked), 다 잡으면 열린다. 보스방을 비우면 출구(U)와 생명의 씨앗.
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
    pub fn chance(&mut self, p: f32) -> bool {
        (self.next() % 10_000) as f32 / 10_000.0 < p
    }
}

/// 던전 하나를 만든다: (던전 상태, 그 격자로 지은 Room).
pub fn generate(seed: u64, depth: u32) -> (Dungeon, Room) {
    let mut rng = Rng::new(seed);
    let start = (GRID / 2, GRID - 1);
    let target = 7 + depth.min(3) as usize;
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
    // 고리 하나: 이웃인데 안 이어진 쌍
    'outer: for _ in 0..20 {
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
    rooms[boss].kind = RoomKind::Boss;
    let leaves: Vec<usize> = (0..rooms.len())
        .filter(|&i| {
            i != boss
                && rooms[i].kind == RoomKind::Combat
                && rooms[i].links.iter().filter(|l| **l).count() == 1
        })
        .collect();
    if let Some(&t) = leaves.first() {
        rooms[t].kind = RoomKind::Treasure;
        rooms[t].cleared = true;
    }
    if let Some(&r) = leaves.get(1) {
        rooms[r].kind = RoomKind::Rest;
        rooms[r].cleared = true;
    }

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
        match room.kind {
            // 출구는 구석(문 앞 길을 막지 않게).
            RoomKind::Start => grid.set(ox + 3, oy + 3, Cell::Exit),
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
                    if !lane {
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
                let count = if room.kind == RoomKind::Boss {
                    5 + depth as usize
                } else {
                    2 + depth.min(2) as usize + rng.below(2) as usize
                };
                for k in 0..count {
                    let kind = if rng.chance(0.55) {
                        Kind::Charger
                    } else {
                        Kind::Shooter
                    };
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
            }
            RoomKind::Rest => {}
        }
    }
    let dungeon = Dungeon {
        rooms,
        start,
        depth,
    };
    (dungeon, Room::from_grid(grid, props, Vec::new()))
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
    fn generates_connected_dungeon_with_one_boss() {
        for seed in 1..40 {
            let (d, room) = generate(seed, 1);
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

    #[test]
    fn same_seed_same_dungeon() {
        let (a, _) = generate(7, 2);
        let (b, _) = generate(7, 2);
        let sig = |d: &Dungeon| {
            d.rooms
                .iter()
                .map(|r| (r.at, r.kind, r.links))
                .collect::<Vec<_>>()
        };
        assert_eq!(sig(&a), sig(&b));
    }
}
