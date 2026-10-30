//! 절차적 던전(docs/decisions/0019). 필드의 굴 입구로 들어갈 때마다 시드로 새로 만든다.
//!
//! - 방 = 20×12칸(320×192, 화면 하나 + 위아래 6px). 5×5 칸에 방을 붙여 놓은 큰 격자 하나가 던전 전체.
//!   카메라는 지금 방 안에만 머물고, 문을 지나면 옆 방으로 미끄러진다(밀기 전환).
//! - 방 그래프: 시작방에서 무작위로 가지를 뻗는다(나무) + 고리 하나. 시작에서 가장 먼 방 = 보스,
//!   막다른 방 하나 = 보물(장신구 상자), 다른 막다른 방 = 휴식(산딸기). 나머지는 전투.
//! - 넓은 방(0244): 이어진 싸움방 둘을 하나로(가로 2칸·세로 2칸). 사이 벽을 터서 카메라가 그 안을 따라 움직인다.
//! - 방 지형(0244): 싸움방마다 `Layout`(굴 그대로·뿌리 기둥·골짜기·돌밭) 하나.
//! - 전투방: 안으로 들어서면 문이 닫히고(Room::set_doors_locked), 다 잡으면 열린다. 보스방을 비우면 출구(U)와 뿌리 심장.
//! - 시작방에는 필드로 돌아가는 출구가 있다.
//!
//! 같은 시드 → 같은 던전(테스트·버그 재현).

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::enemy::{Entry, Kind};
use crate::map::{cell_center, Cell, Grid};
use crate::room::{Prop, PropKind, Room};
use crate::tuning as t;

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
    /// 첫 적이 엘리트(실타래 칸의 끝방, 0147).
    pub elite: bool,
    /// 넓은 방(0244)이면 짝 방이 있는 문 방향(`DIRS` 번호). 둘은 함께 잠기고 함께 비워진다.
    pub hall: Option<usize>,
    /// 싸움 짜임(0257): 다 잡기 · 물결 · 매복 · 버티기.
    pub encounter: Encounter,
    /// 뒤에 나오는 무리(0257): 물결이면 물결마다 한 줄, 버티기면 첫 줄이 솟아날 무리.
    pub reserve: Vec<Vec<(Kind, Vec2)>>,
    /// 처음 무리가 나타나는 방법(0257).
    pub entry: Entry,
    /// 비우면 나오는 곁방 보상(0257). 옆 방의 문 위 표지로 미리 보인다.
    pub bounty: Option<Bounty>,
    /// 방 지형(0244·0257).
    pub layout: Layout,
}

/// 방의 싸움 짜임(0257, 사장 "방들이 너무 반복되는 느낌"): 늘 "들어서면 잠기고 다 잡으면 열림"이던 것을 넷으로.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encounter {
    /// 다 잡기(예전 그대로).
    Clear,
    /// 물결: 남은 적이 `WAVE_NEXT_AT` 아래로 줄면 다음 무리가 땅에서 솟는다.
    Waves,
    /// 매복: 빈 방처럼 보이다가 가운데쯤 오면 잠기고 둘레에서 떨어진다.
    Ambush,
    /// 버티기: `HOLD_TIME` 동안 무리가 계속 솟는다. 시간이 다 되면 남은 것은 무너지는 흙에 흩어진다.
    Hold,
}

/// 곁방 보상(0257): 끝방이 아닌 싸움방에 — 옆 방에서 그 문 위 표지로 보인다(Hades의 문 보상).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bounty {
    Seeds,
    Glowbugs,
    Berry,
    /// 길 무기·장신구(없으면 반딧불). 버티기 방에서만.
    Gift,
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

    /// 넓은 방(0244)의 짝 방 칸.
    pub fn partner(&self, at: (i32, i32)) -> Option<(i32, i32)> {
        let d = self.room(at)?.hall?;
        Some((at.0 + DIRS[d].0, at.1 + DIRS[d].1))
    }

    /// 이 방이 차지하는 월드 사각형(넓은 방이면 둘을 합친 것). 카메라·떨어진 것이 이 안에 머문다.
    pub fn area(&self, at: (i32, i32)) -> Rect {
        let r = room_rect(at);
        match self.partner(at) {
            Some(p) => r.combine_with(room_rect(p)),
            None => r,
        }
    }

    /// 잠그는 안쪽(넓은 방이면 둘의 안쪽을 합친 것).
    pub fn area_interior(&self, at: (i32, i32)) -> Rect {
        let Some(r) = self.room(at) else {
            return room_rect(at);
        };
        match self.partner(at).and_then(|p| self.room(p)) {
            Some(q) => r.interior().combine_with(q.interior()),
            None => r.interior(),
        }
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
    /// 적 뽑기 표를 고르는 지대(0146). 굴을 지을 때 장소가 정한다. 장애물도 정한다(0147).
    pub biome: Biome,
    /// 끝방에 엘리트 하나 + 졸개 둘(실타래 칸, 0147).
    pub elite: bool,
    /// 넓은 방(이어진 싸움방 둘을 하나로, 0244)을 몇 개까지.
    pub halls: usize,
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
            biome: Biome::Meadow,
            elite: false,
            halls: 0,
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
            halls: 1,
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
        elite: false,
        hall: None,
        encounter: Encounter::Clear,
        reserve: Vec::new(),
        entry: Entry::Wake,
        bounty: None,
        layout: Layout::Plain,
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
            elite: false,
            hall: None,
            encounter: Encounter::Clear,
            reserve: Vec::new(),
            entry: Entry::Wake,
            bounty: None,
            layout: Layout::Plain,
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

    // 넓은 방(0244): 이어진 싸움방 둘(둘 다 아직 짝이 없는)을 하나로. 굴 모양 난수와 따로라 방 배치는 그대로다.
    let mut hrng = Rng::new((seed ^ 0x4A11_5EED) | 1);
    for _ in 0..prm.halls {
        let pairs: Vec<(usize, usize, usize)> = (0..rooms.len())
            .flat_map(|i| (0..4).map(move |d| (i, d)))
            .filter(|&(i, d)| rooms[i].links[d] && rooms[i].kind == RoomKind::Combat)
            .filter_map(|(i, d)| {
                let at = (rooms[i].at.0 + DIRS[d].0, rooms[i].at.1 + DIRS[d].1);
                let j = rooms.iter().position(|r| r.at == at)?;
                (j > i
                    && rooms[j].kind == RoomKind::Combat
                    && rooms[i].hall.is_none()
                    && rooms[j].hall.is_none())
                .then_some((i, j, d))
            })
            .collect();
        if pairs.is_empty() || !hrng.chance(t::HALL_CHANCE) {
            continue;
        }
        let (i, j, d) = pairs[hrng.below(pairs.len() as u64) as usize];
        rooms[i].hall = Some(d);
        rooms[j].hall = Some((d + 2) % 4);
    }

    let goal_at = rooms[boss].at;
    // 굴 안 세기 곡선(0257): 시작 바로 옆은 작은 싸움, 끝방 바로 앞(끝방과 이어진 방)은 숨 돌리는 작은 싸움, 넓은 방은 큰 싸움.
    // 들어오는 문 = 시작 쪽 이웃(거리가 하나 적은 방)으로 가는 문. 뒷줄(던지기·혀·포자)은 그 문에서 먼 자리부터.
    let near_goal: Vec<bool> = (0..rooms.len())
        .map(|i| {
            dist[i] + 1 == dist[boss]
                && (0..4).any(|d| {
                    rooms[i].links[d]
                        && (rooms[i].at.0 + DIRS[d].0, rooms[i].at.1 + DIRS[d].1) == goal_at
                })
        })
        .collect();
    let entry_dir: Vec<Option<usize>> = (0..rooms.len())
        .map(|i| {
            (0..4).find(|&d| {
                rooms[i].links[d]
                    && rooms
                        .iter()
                        .position(|r| {
                            r.at == (rooms[i].at.0 + DIRS[d].0, rooms[i].at.1 + DIRS[d].1)
                        })
                        .is_some_and(|j| dist[j] + 1 == dist[i])
            })
        })
        .collect();
    // 격자
    let mut grid = Grid::filled(GRID as usize * ROOM_W, GRID as usize * ROOM_H, Cell::Wall);
    let mut props = Vec::new();
    let mut thorns = Vec::new();
    let mut webs = Vec::new();
    for (ri, room) in rooms.iter_mut().enumerate() {
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
        // 방 지형(0244): 싸움방(이름 있는 주인의 방은 빼고)마다 하나. 다른 난수라 굴 모양 난수는 그대로다.
        let layout = match room.kind {
            RoomKind::Combat => Layout::roll(&mut rng_at(seed, room.at, 0x1A70_0F17)),
            RoomKind::Boss if prm.boss.is_none() => {
                Layout::roll(&mut rng_at(seed, room.at, 0x1A70_0F17))
            }
            _ => Layout::Plain,
        };
        room.layout = layout;
        shape_room(
            &mut grid,
            ox,
            oy,
            room.kind,
            layout,
            seed ^ hash_at(room.at),
        );
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
                    let lane = passage_lane(x, y, room.hall);
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
                // 무리(0257): 지대 무리 표에서 하나, 몇 마리인지는 굴 안 세기 곡선으로. 다른 난수라 굴 모양·소품은 그대로.
                let mut er = rng_at(seed, room.at, 0x5A0D_0257);
                let goal_room = room.kind == RoomKind::Boss;
                let small = !goal_room && room.hall.is_none() && (dist[ri] <= 1 || near_goal[ri]);
                let medium = 2 + depth.min(2) as usize + er.below(2) as usize;
                let count = match (room.kind, prm.goal) {
                    (RoomKind::Boss, Goal::Boss) => 5 + depth as usize,
                    (RoomKind::Boss, _) => 3 + depth.min(2) as usize + prm.tough as usize,
                    _ if small => 2 + depth.min(1) as usize,
                    // 넓은 방(0244)은 둘이 함께 깨어나니 한 방에 하나씩 덜(둘 합쳐 한 방의 1.5배쯤) — 대신 물결이 온다.
                    _ if room.hall.is_some() => medium.saturating_sub(1).max(2),
                    _ => medium,
                };
                let lvl = depth + prm.tough;
                let squad = crate::species::squad(er.unit(), lvl, prm.biome);
                let spots = room_spots(layout);
                let door = entry_dir[ri];
                let kinds = squad.members(count, || er.unit(), lvl, prm.biome);
                room.enemies = place(&kinds, &spots, door, &mut er)
                    .into_iter()
                    .map(|(k, (cx, cy))| (k, cell_center(ox + cx, oy + cy)))
                    .collect();
                // 싸움 짜임(0257): 끝방은 다 잡기. 시작 옆·숨 돌리는 방은 다 잡기, 넓은 방은 물결, 나머지는 굴린다.
                room.encounter = if goal_room || small {
                    Encounter::Clear
                } else if room.hall.is_some() {
                    Encounter::Waves
                } else {
                    let r = er.unit();
                    let [a, b, c] = t::ENCOUNTER_ODDS;
                    if r < a {
                        Encounter::Clear
                    } else if r < b {
                        Encounter::Waves
                    } else if r < c {
                        Encounter::Ambush
                    } else {
                        Encounter::Hold
                    }
                };
                let wave = |n: usize, er: &mut Rng| -> Vec<(Kind, Vec2)> {
                    let kinds = squad.members(n, || er.unit(), lvl, prm.biome);
                    // 뒤 무리는 앞자리(core)를 다시 쓰지 않게 — 무리의 성격(첫 줄)은 처음 무리가 보인다.
                    place(&kinds, &spots, door, er)
                        .into_iter()
                        .map(|(k, (cx, cy))| (k, cell_center(ox + cx, oy + cy)))
                        .collect()
                };
                match room.encounter {
                    Encounter::Waves => {
                        room.reserve = vec![wave(2 + depth.min(1) as usize, &mut er)];
                    }
                    Encounter::Hold => {
                        // 처음엔 `HOLD_ALIVE`까지, 나머지와 덤은 솟아날 줄로.
                        let mut all = std::mem::take(&mut room.enemies);
                        all.extend(wave(t::HOLD_EXTRA + depth.min(2) as usize, &mut er));
                        let rest = all.split_off(t::HOLD_ALIVE.min(all.len()));
                        room.enemies = all;
                        room.reserve = vec![rest];
                    }
                    Encounter::Ambush => {
                        // 둘레에서 떨어진다: 자리를 방 가운데 쪽으로 당긴다.
                        // 당긴 자리가 벽(기둥·틀)이면 제자리 그대로.
                        let c = cell_center(ox + 10, oy + 6) - vec2(8.0, 8.0);
                        for (_, p) in &mut room.enemies {
                            let q = c + (*p - c) * t::AMBUSH_PULL;
                            let (qx, qy) = ((q.x / 16.0) as usize, (q.y / 16.0) as usize);
                            if grid.get(qx, qy) != Cell::Wall {
                                *p = q;
                            }
                        }
                    }
                    Encounter::Clear => {}
                }
                room.entry = match room.encounter {
                    Encounter::Ambush => Entry::Drop,
                    Encounter::Hold => Entry::Burrow,
                    _ if goal_room => Entry::Wake,
                    _ => {
                        let r = er.unit();
                        let [w, b] = if prm.biome == Biome::Meadow {
                            t::ENTRY_ODDS_MEADOW
                        } else {
                            t::ENTRY_ODDS
                        };
                        if r < w {
                            Entry::Wake
                        } else if r < b {
                            Entry::Burrow
                        } else {
                            Entry::Drop
                        }
                    }
                };
                // 곁방 보상(0257): 끝방·시작 옆 작은 방은 없다. 물결·버티기는 늘(버티기는 물건 기회), 나머지는 반쯤.
                // 넓은 방은 짝 가운데 한 방(오른쪽·아래에 짝이 있는 방)에만.
                let first_half = room.hall.is_none_or(|d| d == 1 || d == 2);
                if !goal_room && !(dist[ri] <= 1 && room.hall.is_none()) && first_half {
                    let mut br = rng_at(seed, room.at, 0xB0_0257);
                    let sure = matches!(room.encounter, Encounter::Waves | Encounter::Hold);
                    if sure || br.chance(t::BOUNTY_CHANCE) {
                        let r = br.unit();
                        room.bounty =
                            Some(if room.encounter == Encounter::Hold && r < t::BOUNTY_GIFT {
                                Bounty::Gift
                            } else {
                                let r = br.unit();
                                let [a, b] = t::BOUNTY_ODDS;
                                if r < a {
                                    Bounty::Seeds
                                } else if r < b {
                                    Bounty::Glowbugs
                                } else {
                                    Bounty::Berry
                                }
                            });
                    }
                }
                // 보스가 정해져 있으면 무리 대신 혼자, 방 가운데 위쪽에. 난수는 위에서 그대로 써서
                // 같은 시드면 굴 모양이 보스가 있든 없든 같다.
                if let (RoomKind::Boss, Some(k)) = (room.kind, prm.boss) {
                    room.enemies = vec![(k, cell_center(ox + 10, oy + 4))];
                }
                // 실타래 칸(0147): 끝방은 엘리트 하나(방 가운데 위) + 무리에서 둘. 생쥐 떼·포자버섯은 엘리트가 되지 않는다.
                if room.kind == RoomKind::Boss && prm.elite && prm.boss.is_none() {
                    let mut k = Kind::FieldMouse;
                    for _ in 0..8 {
                        k = crate::species::roll(rng.unit(), depth + 1, prm.biome);
                        if !matches!(k, Kind::HouseMouse | Kind::SporeCap) {
                            break;
                        }
                    }
                    if matches!(k, Kind::HouseMouse | Kind::SporeCap) {
                        k = Kind::SewerRat;
                    }
                    room.enemies.truncate(t::ELITE_ESCORTS);
                    room.enemies.insert(0, (k, cell_center(ox + 10, oy + 4)));
                    room.elite = true;
                }
                // 적이 서는 자리(뒤 무리까지)에는 덤불·그물·부술 것을 두지 않는다(틀 방은 자리가 `KEEP`과 다르다).
                let spawn_marks: Vec<Vec2> = room
                    .enemies
                    .iter()
                    .chain(room.reserve.iter().flatten())
                    .map(|(_, p)| *p)
                    .collect();
                // 뿌리 지대(0147): 가시뿌리 덤불 무리 1~2개. 굴 모양·적과 다른 난수라 다른 지대의 굴은 그대로다.
                if prm.biome == Biome::Roots && prm.boss.is_none() {
                    thorn_patches(
                        &grid,
                        &props,
                        &spawn_marks,
                        ox,
                        oy,
                        seed ^ hash_at(room.at) ^ 0x7407_2A11,
                        room.hall,
                        &mut thorns,
                    );
                }
                // 흰 실 굴(0165): 흰 실 그물 무리 1~2개. 덤불과 같은 자리 규칙(길·적 자리·소품 밖).
                if prm.biome == Biome::Silk && prm.boss.is_none() {
                    thorn_patches(
                        &grid,
                        &props,
                        &spawn_marks,
                        ox,
                        oy,
                        seed ^ hash_at(room.at) ^ 0x5E1C_0B1A,
                        room.hall,
                        &mut webs,
                    );
                }
            }
            RoomKind::Rest => {}
        }
        // 부술 수 있는 것(0175): 싸움방·끝방 1~3개, 쉼방 2~3개, 시작방 1개. 굴 모양·적·덤불과 다른 난수.
        let n_break = match room.kind {
            RoomKind::Combat | RoomKind::Boss => 1 + rng_at(seed, room.at, 0xB4EA_4B1E).below(3),
            RoomKind::Rest => 2 + rng_at(seed, room.at, 0xB4EA_4B1E).below(2),
            RoomKind::Start => 1,
            RoomKind::Treasure => 0,
        } as usize;
        let marks: Vec<Vec2> = room
            .enemies
            .iter()
            .chain(room.reserve.iter().flatten())
            .map(|(_, p)| *p)
            .collect();
        breakables(
            &grid,
            &mut props,
            &marks,
            &thorns,
            &webs,
            ox,
            oy,
            seed ^ hash_at(room.at) ^ 0xB4EA_4B1E,
            prm.biome,
            room.hall,
            n_break,
        );
    }
    // 넓은 방(0244): 짝 방 사이 벽을 튼다(문 칸도 바닥으로). 방 모양을 다 만든 뒤라 둘 다 제 모양 그대로 이어진다.
    for room in &rooms {
        let (ox, oy) = (room.at.0 as usize * ROOM_W, room.at.1 as usize * ROOM_H);
        match room.hall {
            // 오른쪽 짝: 이 방 오른쪽 벽 줄 + 짝 방 왼쪽 벽 줄.
            Some(1) => {
                for y in 2..ROOM_H - 1 {
                    grid.set(ox + ROOM_W - 1, oy + y, Cell::Floor);
                    grid.set(ox + ROOM_W, oy + y, Cell::Floor);
                }
            }
            // 아래 짝: 이 방 아래 벽 줄 + 짝 방 위 벽 두 줄.
            Some(2) => {
                for x in 1..ROOM_W - 1 {
                    grid.set(ox + x, oy + ROOM_H - 1, Cell::Floor);
                    grid.set(ox + x, oy + ROOM_H, Cell::Floor);
                    grid.set(ox + x, oy + ROOM_H + 1, Cell::Floor);
                }
            }
            _ => {}
        }
    }
    let dungeon = Dungeon {
        rooms,
        start,
        depth,
        goal: prm.goal,
        boss: prm.boss,
    };
    let mut room = Room::from_grid(grid, props, Vec::new());
    room.thorns = thorns;
    room.webs = webs;
    (dungeon, room)
}

/// 가시뿌리 덤불 무리(0147): 바닥 칸 하나에서 이웃 바닥 칸으로 1~2칸 더. 문 앞 십자 길·`KEEP` 자리·소품 칸은 피한다.
fn thorn_patches(
    grid: &Grid,
    props: &[Prop],
    avoid: &[Vec2],
    ox: usize,
    oy: usize,
    seed: u64,
    hall: Option<usize>,
    out: &mut Vec<Vec2>,
) {
    let mut rng = Rng::new(seed | 1);
    let ok = |x: usize, y: usize, out: &Vec<Vec2>| {
        let c = cell_center(ox + x, oy + y);
        (2..=17).contains(&x)
            && (3..=10).contains(&y)
            && !passage_lane(x, y, hall)
            && !KEEP.contains(&(x, y))
            && grid.get(ox + x, oy + y) == Cell::Floor
            && !props.iter().any(|p| p.rect.contains(c))
            && !avoid.iter().any(|a| a.distance(c) < 12.0)
            && !out.contains(&c)
    };
    for _ in 0..1 + rng.below(2) {
        let (mut x, mut y) = (0, 0);
        let mut found = false;
        for _ in 0..12 {
            x = 2 + rng.below(16) as usize;
            y = 3 + rng.below(8) as usize;
            if ok(x, y, out) {
                found = true;
                break;
            }
        }
        if !found {
            continue;
        }
        out.push(cell_center(ox + x, oy + y));
        for _ in 0..1 + rng.below(2) {
            let (dx, dy) = DIRS[rng.below(4) as usize];
            let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
            if ok(nx, ny, out) {
                out.push(cell_center(ox + nx, oy + ny));
                (x, y) = (nx, ny);
            }
        }
    }
}

/// 문 앞 십자 길과 넓은 방의 열린 이음새까지 잇는 접근로는 장애물 없이 둔다.
fn passage_lane(x: usize, y: usize, hall: Option<usize>) -> bool {
    (8..=11).contains(&x)
        || (5..=8).contains(&y)
        || match hall {
            Some(0) => (3..=4).contains(&y),
            Some(1) => (15..=17).contains(&x),
            Some(2) => (9..=10).contains(&y),
            Some(3) => (2..=4).contains(&x),
            _ => false,
        }
}

fn rng_at(seed: u64, at: (i32, i32), salt: u64) -> Rng {
    Rng::new((seed ^ hash_at(at) ^ salt) | 1)
}

/// 부술 수 있는 것(0175) `n`개: 문 앞 십자 길·`KEEP` 자리·소품·덤불·그물 칸을 피해 바닥 칸에. 생김새는 지대 것이
/// 대부분, 가끔 흙더미. 소품이라 막고, 깨지면 빠진다(`World::prop_break`).
#[allow(clippy::too_many_arguments)]
fn breakables(
    grid: &Grid,
    props: &mut Vec<Prop>,
    avoid: &[Vec2],
    thorns: &[Vec2],
    webs: &[Vec2],
    ox: usize,
    oy: usize,
    seed: u64,
    biome: Biome,
    hall: Option<usize>,
    n: usize,
) {
    use crate::room::Breakable;
    let mut rng = Rng::new(seed | 1);
    let local = match biome {
        Biome::Meadow => Breakable::Mound,
        Biome::Roots => Breakable::Gall,
        Biome::Deep => Breakable::RockPile,
        Biome::Silk => Breakable::SilkBall,
    };
    for _ in 0..n {
        for _ in 0..12 {
            let x = 2 + rng.below(16) as usize;
            let y = 3 + rng.below(8) as usize;
            let c = cell_center(ox + x, oy + y);
            let free = !passage_lane(x, y, hall)
                && !KEEP.contains(&(x, y))
                && grid.get(ox + x, oy + y) == Cell::Floor
                && !props.iter().any(|p| p.rect.contains(c))
                && !thorns.contains(&c)
                && !avoid.iter().any(|a| a.distance(c) < 12.0)
                && !webs.contains(&c);
            if free {
                let kind = if rng.chance(0.25) {
                    Breakable::Mound
                } else {
                    local
                };
                props.push(Prop {
                    rect: cell_rect(ox + x, oy + y),
                    kind: PropKind::Breakable(kind),
                });
                break;
            }
        }
    }
}

/// 방에 적을 놓는 자리(지형이 틀이 아니면, 0257 전부터): 가장자리 쪽 여덟 — 가운데는 들어오는 플레이어 자리.
const SPOTS: [(usize, usize); 8] = [
    (4, 4),
    (15, 4),
    (4, 9),
    (15, 9),
    (10, 3),
    (10, 10),
    (3, 7),
    (16, 7),
];

/// 손으로 짠 방 틀(0257, D): 방 안 18×9(x 1..=18, y 2..=10). `#` 벽, `e` 적 자리, `.` 바닥.
/// 규칙: 문 앞 십자 길 — 가운데 두 열(x 9·10)과 가운데 두 줄(y 6·7) — 은 늘 비우고, 적 자리는 여섯 넘게, 문 곁(두 칸 안)엔 두지 않는다.
/// 좌우로 뒤집어 둘씩 쓴다(시험 `templates_keep_lanes_spots_and_connect`).
pub const TEMPLATES: [[&str; 9]; 6] = [
    // 네 기둥: 큰 기둥 넷 사이로 도는 길.
    [
        "..................",
        ".e..###....###..e.",
        "....###....###....",
        "......e....e......",
        "..................",
        "..................",
        "....###....###....",
        ".e..###....###..e.",
        "..................",
    ],
    // 두 능선: 위아래에서 내려온 능선 사이 좁은 목.
    [
        "......#....#......",
        ".e....#....#....e.",
        "......#....#.e....",
        "..##..........##..",
        "..................",
        "..................",
        "..##..........##..",
        ".e..e.#....#....e.",
        "......#....#......",
    ],
    // 계단 담: 위아래 담장 끝이 꺾여 들어온다.
    [
        "..................",
        ".e.####....####.e.",
        ".......#..#.......",
        "...e..........e...",
        "..................",
        "..................",
        "...e..........e...",
        ".e.####....####.e.",
        "..................",
    ],
    // 네 모서리 비탈: 모서리마다 비스듬한 벽.
    [
        ".#..............#.",
        ".##.e........e.##.",
        "..##..........##..",
        "......e...........",
        "..................",
        "..................",
        "..##.......e..##..",
        ".##.e........e.##.",
        ".#..............#.",
    ],
    // 긴 골짜기: 위·아래 띠가 긴 담으로 갈린다. 가운데로만 오간다.
    [
        "..................",
        ".e....e....e....e.",
        ".######....######.",
        "..................",
        "..................",
        "..................",
        ".######....######.",
        ".e....e....e....e.",
        "..................",
    ],
    // 섬: 작은 바위섬이 흩어진 방.
    [
        "..................",
        "..e..##....##..e..",
        ".#..............#.",
        "....e........e....",
        "..................",
        "..................",
        ".#..e........e..#.",
        "..................",
        "..e..##....##..e..",
    ],
];

/// 틀 `i`의 (x, y) 칸이 벽인가(방 칸, 뒤집기 반영).
fn template_wall(i: usize, flip: bool, x: usize, y: usize) -> bool {
    template_cell(i, flip, x, y) == b'#'
}

fn template_cell(i: usize, flip: bool, x: usize, y: usize) -> u8 {
    if !(1..=18).contains(&x) || !(2..=10).contains(&y) {
        return b'#';
    }
    let x = if flip { 19 - x } else { x };
    TEMPLATES[i][y - 2].as_bytes()[x - 1]
}

/// 그 방 지형의 적 자리(방 칸).
pub fn room_spots(layout: Layout) -> Vec<(usize, usize)> {
    match layout {
        Layout::Template(i, flip) => (2..=10)
            .flat_map(|y| (1..=18).map(move |x| (x, y)))
            .filter(|&(x, y)| template_cell(i as usize, flip, x, y) == b'e')
            .collect(),
        _ => SPOTS.to_vec(),
    }
}

/// 무리를 자리에 놓는다(0257): 뒷줄(던지기·혀·포자, `species::back_row`)은 들어오는 문에서 먼 자리부터, 나머지는 남은 자리를
/// 섞어서. 자리보다 많으면 처음부터 다시(둘이 한 자리에 — 곧 밀려 갈라선다).
fn place(
    kinds: &[Kind],
    spots: &[(usize, usize)],
    door: Option<usize>,
    rng: &mut Rng,
) -> Vec<(Kind, (usize, usize))> {
    let door_at = door.map(|d| match d {
        0 => (9.5, 0.0),
        1 => (19.0, 6.5),
        2 => (9.5, 11.0),
        _ => (0.0, 6.5),
    });
    let mut order: Vec<usize> = (0..spots.len()).collect();
    for i in (1..order.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        order.swap(i, j);
    }
    let far = |i: usize| {
        door_at.map_or(0.0, |(dx, dy): (f32, f32)| {
            let (x, y) = spots[i];
            (x as f32 - dx).hypot(y as f32 - dy)
        })
    };
    let mut by_far = order.clone();
    by_far.sort_by(|a, b| far(*b).total_cmp(&far(*a)));
    let mut used = vec![false; spots.len()];
    let take = |list: &[usize], used: &mut Vec<bool>| -> usize {
        if used.iter().all(|u| *u) {
            used.iter_mut().for_each(|u| *u = false);
        }
        let i = *list.iter().find(|&&i| !used[i]).unwrap_or(&list[0]);
        used[i] = true;
        i
    };
    let mut out = Vec::new();
    for &k in kinds.iter().filter(|k| crate::species::back_row(**k)) {
        out.push((k, spots[take(&by_far, &mut used)]));
    }
    for &k in kinds.iter().filter(|k| !crate::species::back_row(**k)) {
        out.push((k, spots[take(&order, &mut used)]));
    }
    out
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

/// 방 지형(0244). 굴 모양(모서리·혹·바위 무리) 위에 얹는다. 모두 문 앞 십자 길과 `KEEP` 자리는 비운다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    /// 굴 그대로(0063).
    Plain,
    /// 뿌리 기둥: 네 사분면에 2×2 기둥(위)·2×1 기둥(아래). 돌아 들어가며 싸운다.
    Pillars,
    /// 골짜기: 위·아래 벽에서 내려오는 능선 둘(가운데 길에서 끊긴다). 방이 세 갈래로 나뉜다.
    Canyon,
    /// 돌밭: 한 칸짜리 돌 5~8개를 흩는다. 원거리 적의 탄을 막고, 돌진을 끊는다.
    Rubble,
    /// 손으로 짠 방 틀(0257): `TEMPLATES` 번호, 좌우 뒤집기. 굴 모양(모서리·혹·바위)도 틀이 대신한다.
    Template(u8, bool),
}

impl Layout {
    pub fn roll(rng: &mut Rng) -> Self {
        if rng.chance(t::TEMPLATE_CHANCE) {
            return Self::Template(rng.below(TEMPLATES.len() as u64) as u8, rng.chance(0.5));
        }
        let r = rng.unit();
        if r < t::LAYOUT_ODDS[0] {
            Self::Plain
        } else if r < t::LAYOUT_ODDS[1] {
            Self::Pillars
        } else if r < t::LAYOUT_ODDS[2] {
            Self::Canyon
        } else {
            Self::Rubble
        }
    }
}

/// 네모난 방을 굴처럼(0063): 모서리를 메우고, 가장자리에 1~2칸 혹을 내고, 가운데 쪽에 바위 무리를 하나 둔다.
/// 문 앞 십자 길(가로 5~8줄, 세로 8~11칸)과 `KEEP` 자리(둘레 한 칸까지)는 건드리지 않는다.
/// 다 만든 뒤 모든 바닥이 방 가운데에서 이어지는지 보고, 안 이어지면 네모로 되돌린다.
fn shape_room(grid: &mut Grid, ox: usize, oy: usize, kind: RoomKind, layout: Layout, seed: u64) {
    if let Layout::Template(i, flip) = layout {
        // 손으로 짠 틀(0257): 틀이 방 모양 전부. 이어짐은 시험이 본다(`templates_keep_lanes_spots_and_connect`).
        for y in 2..=10 {
            for x in 1..=18 {
                if template_wall(i as usize, flip, x, y) {
                    grid.set(ox + x, oy + y, Cell::Wall);
                }
            }
        }
        return;
    }
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
    // 방 지형(0244).
    match layout {
        Layout::Plain => {}
        Layout::Pillars => {
            for (x, y) in [
                (6, 3),
                (7, 3),
                (6, 4),
                (7, 4),
                (12, 3),
                (13, 3),
                (12, 4),
                (13, 4),
                (6, 9),
                (7, 9),
                (12, 9),
                (13, 9),
            ] {
                wall(x, y);
            }
        }
        Layout::Canyon => {
            for x in [6, 13] {
                for y in [2, 3, 4, 9, 10] {
                    wall(x, y);
                }
            }
        }
        Layout::Template(..) => {}
        Layout::Rubble => {
            let n = 5 + rng.below(4);
            for _ in 0..n {
                for _ in 0..8 {
                    let (x, y) = (2 + rng.below(16) as usize, 3 + rng.below(7) as usize);
                    if free(x, y) {
                        wall(x, y);
                        break;
                    }
                }
            }
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

    /// 흰 실 굴에만 흰 실 그물(0165). 덤불과 같은 규칙, 다른 지대엔 없다.
    #[test]
    fn webs_only_in_silk_warrens() {
        let silk = Params {
            biome: Biome::Silk,
            ..Params::new(1)
        };
        let mut any = false;
        for seed in 1..40 {
            let (_, room) = generate(seed, &silk);
            let (_, again) = generate(seed, &silk);
            assert_eq!(room.webs, again.webs, "seed {seed}");
            any |= !room.webs.is_empty();
            assert!(room.thorns.is_empty(), "흰 실 굴엔 덤불이 없다");
            for c in &room.webs {
                let (x, y) = (
                    ((c.x / 16.0) as i32).rem_euclid(ROOM_W as i32),
                    ((c.y / 16.0) as i32).rem_euclid(ROOM_H as i32),
                );
                assert!(
                    !(8..=11).contains(&x) && !(5..=8).contains(&y),
                    "seed {seed}: 길 위"
                );
                assert!(!room.props.iter().any(|p| p.rect.contains(*c)));
            }
            let (_, deep) = generate(
                seed,
                &Params {
                    biome: Biome::Deep,
                    ..silk
                },
            );
            assert!(deep.webs.is_empty());
        }
        assert!(any);
    }

    /// 뿌리 지대 굴에만 가시 덤불(0147). 문 앞 길·적 자리·소품 칸은 비우고, 같은 시드면 같은 자리.
    #[test]
    fn thorns_only_in_roots_and_off_the_lanes() {
        let roots = Params {
            biome: Biome::Roots,
            ..Params::new(1)
        };
        let mut any = false;
        for seed in 1..40 {
            let (d, room) = generate(seed, &roots);
            let (_, again) = generate(seed, &roots);
            assert_eq!(room.thorns, again.thorns, "seed {seed}");
            any |= !room.thorns.is_empty();
            for c in &room.thorns {
                let (cx, cy) = ((c.x / 16.0) as i32, (c.y / 16.0) as i32);
                let r = d
                    .rooms
                    .iter()
                    .find(|r| r.rect().contains(*c))
                    .expect("방 안");
                assert!(matches!(r.kind, RoomKind::Combat | RoomKind::Boss));
                let (x, y) = (
                    (cx - r.at.0 * ROOM_W as i32) as usize,
                    (cy - r.at.1 * ROOM_H as i32) as usize,
                );
                assert!(
                    !(8..=11).contains(&x) && !(5..=8).contains(&y),
                    "seed {seed}: 길 위"
                );
                assert!(!KEEP.contains(&(x, y)), "seed {seed}: 적 자리");
                assert!(!room.props.iter().any(|p| p.rect.contains(*c)));
            }
            let (_, deep) = generate(
                seed,
                &Params {
                    biome: Biome::Deep,
                    ..roots
                },
            );
            assert!(deep.thorns.is_empty());
        }
        assert!(any);
    }

    /// 실타래 칸(0147): 끝방 첫 적이 엘리트(생쥐 떼·포자버섯이 아님) + 졸개 둘까지.
    #[test]
    fn tangle_goal_room_starts_with_an_elite() {
        let prm = Params {
            rooms: 3,
            goal: Goal::Pack,
            elite: true,
            biome: Biome::Deep,
            ..Params::new(2)
        };
        for seed in 1..60 {
            let (d, _) = generate(seed, &prm);
            let goal = d.rooms.iter().find(|r| r.kind == RoomKind::Boss).unwrap();
            assert!(goal.elite);
            assert!(goal.enemies.len() <= 1 + t::ELITE_ESCORTS);
            assert!(!matches!(
                goal.enemies[0].0,
                Kind::HouseMouse | Kind::SporeCap
            ));
            assert!(d.rooms.iter().filter(|r| r.elite).count() == 1);
        }
    }

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
                if r.links[1] && r.hall != Some(1) {
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
                if let Layout::Template(..) = r.layout {
                    // 틀 방(0257)은 `templates_keep_lanes_spots_and_connect`가 본다. 적은 바닥에.
                    for (_, p) in r.enemies.iter().chain(r.reserve.iter().flatten()) {
                        assert_ne!(room.cell_at(*p), Some(Cell::Wall), "틀 방 적은 바닥에");
                    }
                    total += 1;
                    shaped += 1;
                    continue;
                }
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

    /// 부술 수 있는 것(0175): 모든 지대의 굴에 있고, 바닥 칸·길 밖·적 자리 밖에, 다른 소품과 겹치지 않으며,
    /// 같은 시드면 같은 자리. 생김새는 지대 것 아니면 흙더미.
    #[test]
    fn breakables_sit_on_floor_off_the_lanes_in_every_biome() {
        use crate::room::Breakable;
        for (biome, local) in [
            (Biome::Meadow, Breakable::Mound),
            (Biome::Roots, Breakable::Gall),
            (Biome::Deep, Breakable::RockPile),
            (Biome::Silk, Breakable::SilkBall),
        ] {
            let prm = Params {
                biome,
                ..Params::new(1)
            };
            for seed in 1..12 {
                let (d, room) = generate(seed, &prm);
                let (_, again) = generate(seed, &prm);
                let brk = |r: &Room| {
                    r.props
                        .iter()
                        .filter_map(|p| match p.kind {
                            PropKind::Breakable(b) => Some((p.rect, b)),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                };
                let list = brk(&room);
                assert_eq!(list, brk(&again), "seed {seed}");
                assert!(
                    list.len() >= d.rooms.len() / 2,
                    "seed {seed}: 곳곳에 — {}",
                    list.len()
                );
                for (rect, b) in &list {
                    assert!(*b == local || *b == Breakable::Mound);
                    let c = rect.center();
                    let g = room.grid.as_ref().unwrap();
                    assert_eq!(
                        g.get((c.x / 16.0) as usize, (c.y / 16.0) as usize),
                        Cell::Floor
                    );
                    let r = d
                        .rooms
                        .iter()
                        .find(|r| r.rect().contains(c))
                        .expect("방 안");
                    let (x, y) = (
                        ((c.x / 16.0) as i32 - r.at.0 * ROOM_W as i32) as usize,
                        ((c.y / 16.0) as i32 - r.at.1 * ROOM_H as i32) as usize,
                    );
                    assert!(
                        !(8..=11).contains(&x) && !(5..=8).contains(&y),
                        "seed {seed}: 길 위"
                    );
                    assert!(!KEEP.contains(&(x, y)), "seed {seed}: 적 자리");
                    assert_eq!(
                        room.props.iter().filter(|p| p.rect.contains(c)).count(),
                        1,
                        "seed {seed}: 겹침"
                    );
                    assert!(!room.thorns.contains(&c) && !room.webs.contains(&c));
                }
            }
        }
    }

    /// 넓은 방(0244): 짝은 서로를 가리키는 이어진 싸움방 둘이고, 사이 벽이 트여 있으며, 카메라 사각형은 둘을 합친 것.
    /// 길의 굴(방 여섯)이면 대부분 넓은 방이 하나 있다.
    #[test]
    fn halls_pair_two_linked_combat_rooms_with_an_open_seam() {
        let prm = Params {
            rooms: 6,
            halls: 1,
            ..Params::new(1)
        };
        let mut with_hall = 0;
        for seed in 1..60 {
            let (d, room) = generate(seed, &prm);
            let g = room.grid.as_ref().unwrap();
            let halls: Vec<_> = d.rooms.iter().filter(|r| r.hall.is_some()).collect();
            assert!(halls.len() == 0 || halls.len() == 2, "seed {seed}");
            if halls.is_empty() {
                continue;
            }
            with_hall += 1;
            for r in &halls {
                let k = r.hall.unwrap();
                assert!(r.links[k], "이어진 쌍");
                assert_eq!(r.kind, RoomKind::Combat);
                let p = d.partner(r.at).unwrap();
                assert_eq!(d.partner(p), Some(r.at), "서로를 가리킨다");
                let area = d.area(r.at);
                assert!(area.w > room_rect(r.at).w || area.h > room_rect(r.at).h);
                assert!(d
                    .area_interior(r.at)
                    .contains(d.room(p).unwrap().interior().center()));
            }
            // 사이 벽이 트였다: 한쪽 방 가운데에서 짝 방 가운데까지 바닥으로 걸어갈 수 있다(문 칸 없이).
            let (a, b) = (halls[0].at, d.partner(halls[0].at).unwrap());
            let cell = |at: (i32, i32)| (at.0 as usize * ROOM_W + 10, at.1 as usize * ROOM_H + 6);
            let (from, to) = (cell(a), cell(b));
            let mut seen = std::collections::HashSet::from([from]);
            let mut q = std::collections::VecDeque::from([from]);
            while let Some((x, y)) = q.pop_front() {
                for (dx, dy) in DIRS {
                    let n = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                    if g.get(n.0, n.1) == Cell::Floor && seen.insert(n) {
                        q.push_back(n);
                    }
                }
            }
            assert!(seen.contains(&to), "seed {seed}: 문 없이 이어진다");
        }
        assert!(with_hall * 10 >= 59 * 6, "넓은 방이 자주: {with_hall}/59");
    }

    /// 넓은 방(0244)의 열린 이음새와 접근로에는 가시·그물·막는 소품이 생기지 않는다.
    #[test]
    fn hall_passages_stay_clear_of_hazards_and_props() {
        let prm = Params {
            rooms: 6,
            halls: 1,
            biome: Biome::Roots,
            ..Params::new(1)
        };
        let mut hall_rooms = 0;
        let mut hall_thorns = 0;
        for seed in 1..100 {
            let (d, room) = generate(seed, &prm);
            for r in d.rooms.iter().filter(|r| r.hall.is_some()) {
                hall_rooms += 1;
                let local = |p: Vec2| {
                    (
                        (p.x / 16.0) as usize - r.at.0 as usize * ROOM_W,
                        (p.y / 16.0) as usize - r.at.1 as usize * ROOM_H,
                    )
                };
                for p in &room.thorns {
                    if r.rect().contains(*p) {
                        hall_thorns += 1;
                        let (x, y) = local(*p);
                        assert!(
                            !passage_lane(x, y, r.hall),
                            "seed {seed}: 가시가 넓은 방 통로에 있음 ({x}, {y})"
                        );
                    }
                }
                for p in &room.webs {
                    if r.rect().contains(*p) {
                        let (x, y) = local(*p);
                        assert!(
                            !passage_lane(x, y, r.hall),
                            "seed {seed}: 그물이 넓은 방 통로에 있음 ({x}, {y})"
                        );
                    }
                }
                for prop in &room.props {
                    let p = prop.rect.center();
                    if r.rect().contains(p) {
                        let (x, y) = local(p);
                        assert!(
                            !passage_lane(x, y, r.hall),
                            "seed {seed}: 소품이 넓은 방 통로에 있음 ({x}, {y})"
                        );
                    }
                }
            }
        }
        assert!(hall_rooms > 0);
        assert!(hall_thorns > 0);
    }

    /// 방 지형(0244): 네 가지가 다 나오고, 어느 것이든 적 자리·길은 막지 않는다(`shaped_rooms_…`가 본다).
    #[test]
    fn every_layout_appears() {
        let mut seen = std::collections::HashSet::new();
        for seed in 1..400 {
            seen.insert(Layout::roll(&mut Rng::new(seed)));
        }
        // 지형 넷 + 틀 여섯 × 좌우 뒤집기(0257).
        assert_eq!(seen.len(), 4 + TEMPLATES.len() * 2);
    }

    /// 손으로 짠 틀(0257): 18×9, 문 앞 십자(가운데 두 열·두 줄)는 비고, 적 자리는 여섯 넘게 바닥에·문 곁 밖에,
    /// 뒤집어도 모든 바닥이 한 덩어리로 이어진다.
    #[test]
    fn templates_keep_lanes_spots_and_connect() {
        for i in 0..TEMPLATES.len() {
            for row in TEMPLATES[i] {
                assert_eq!(row.len(), 18, "틀 {i} 줄 길이");
            }
            for flip in [false, true] {
                for y in 2..=10 {
                    for x in 9..=10 {
                        assert!(!template_wall(i, flip, x, y), "틀 {i} 세로 길 ({x},{y})");
                    }
                }
                for x in 1..=18 {
                    for y in 6..=7 {
                        assert!(!template_wall(i, flip, x, y), "틀 {i} 가로 길 ({x},{y})");
                    }
                }
                let spots = room_spots(Layout::Template(i as u8, flip));
                assert!(spots.len() >= 6, "틀 {i} 자리 {}", spots.len());
                for &(x, y) in &spots {
                    let near_door = ((9..=10).contains(&x) && (y <= 3 || y >= 9))
                        || ((6..=7).contains(&y) && (x <= 2 || x >= 17));
                    assert!(!near_door, "틀 {i} 자리 ({x},{y})는 문 곁");
                }
                let mut seen = std::collections::HashSet::from([(10usize, 6usize)]);
                let mut q = vec![(10usize, 6usize)];
                while let Some((x, y)) = q.pop() {
                    for (dx, dy) in DIRS {
                        let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                        if !template_wall(i, flip, nx, ny) && seen.insert((nx, ny)) {
                            q.push((nx, ny));
                        }
                    }
                }
                let floor = (2..=10)
                    .flat_map(|y| (1..=18).map(move |x| (x, y)))
                    .filter(|&(x, y)| !template_wall(i, flip, x, y))
                    .count();
                assert_eq!(
                    seen.len(),
                    floor,
                    "틀 {i} 뒤집기 {flip}: 바닥이 다 이어진다"
                );
            }
        }
    }

    /// 방 짜임(0257): 시작 바로 옆·끝방은 다 잡기, 넓은 방은 물결, 물결·버티기는 뒤 무리가 있고 보상이 늘 붙는다,
    /// 끝방엔 보상이 없다. 여러 굴을 지으면 넷이 다 나오고, 무리 표 덕에 한 방에 같은 종이 둘 넘는 방이 흔하다.
    #[test]
    fn encounters_curve_and_bounties() {
        let mut kinds = std::collections::HashSet::new();
        let mut paired = 0;
        let mut fights = 0;
        for seed in 1..80 {
            let prm = Params {
                biome: Biome::Roots,
                ..Params::classic(2)
            };
            let (d, _) = generate(seed, &prm);
            let dist = distances(&d.rooms, d.start);
            for (i, r) in d.rooms.iter().enumerate() {
                match r.kind {
                    RoomKind::Boss => {
                        assert_eq!(r.encounter, Encounter::Clear);
                        assert!(r.bounty.is_none());
                    }
                    RoomKind::Combat => {
                        kinds.insert(r.encounter);
                        if dist[i] <= 1 && r.hall.is_none() {
                            assert_eq!(
                                r.encounter,
                                Encounter::Clear,
                                "seed {seed}: 시작 옆은 다 잡기"
                            );
                        }
                        if r.hall.is_some() {
                            assert_eq!(r.encounter, Encounter::Waves);
                        }
                        if matches!(r.encounter, Encounter::Waves | Encounter::Hold) {
                            assert!(!r.reserve.is_empty() && !r.reserve[0].is_empty());
                            let half = r.hall.is_none_or(|h| h == 1 || h == 2);
                            assert_eq!(r.bounty.is_some(), half, "seed {seed}");
                        }
                        fights += 1;
                        let mut c = std::collections::HashMap::new();
                        for (k, _) in &r.enemies {
                            *c.entry(*k).or_insert(0) += 1;
                        }
                        if c.values().any(|n| *n >= 2) {
                            paired += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(kinds.len(), 4, "{kinds:?}");
        assert!(paired * 3 >= fights, "무리 짜임: {paired}/{fights}");
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
