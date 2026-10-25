//! 모험 구조(docs/decisions/0019): 아레나 / 필드 / 던전 사이를 오가고, 카메라를 움직이고, 저장 지점에서 다시 선다.
//!
//! - 필드: 연속 스크롤. 카메라가 Naïl을 따라가고 지도 끝에서 멈춘다. 적은 자다가 가까이 오면 깬다.
//! - 던전: 굴 입구(D)에 닿으면 어두워졌다가 새로 생성한 던전 시작방으로. 카메라는 지금 방 안에만 있고, 문을 지나면
//!   옆 방으로 미끄러진다. 전투방은 안에 들어서면 문이 닫히고 다 잡으면 열린다. 출구(U)로 필드의 입구 앞에 돌아온다.
//! - 쓰러지고 목숨(지렁이)도 없으면: 앱이 확인을 받으면 마지막 저장 지점(필드)에서 체력 가득으로. 반딧불 30%를 잃는다.
//!   던전은 버린다(다음에 새로). 장신구·씨앗·무기·가방은 남는다.
//! - 아레나: 화면 하나짜리 시험장(웨이브·개발 시나리오). 카메라는 움직이지 않는다.
//!
//! 스테이지 고리(docs/decisions/0030~0032):
//!   필드(튜토리얼) → 남쪽 공터의 들쥐 대장 → 굴 입구 → 쉼터 1 → 길 지도에서 고른 굴 … → 그 길의 보스 → 쉼터 2 → …
//! - 공터: 튜토리얼이 끝나야 문이 열린다. 안으로 들어서면 문이 닫히고 대장이 깨어난다. 이기면 문이 열리고 입구가 산다.
//! - 쉼터: 들어오면 체력 가득, 저장 고리가 다시 설 곳(home)이 된다. 뿌리 길 곁에서 확인 → 길 지도(앱 화면).
//! - 길의 굴: 끝방을 마치면 출구(U). 출구를 밟으면 다음 갈래를 고르고(보스 굴이면 다음 쉼터로) 곧장 내려간다.
//!   쉼터는 맡겨 두고 굴만 바꾼다. 쓰러지면 쉼터로, 길은 새로 짠다(로그라이트).

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::dungeon::{self, Dungeon, Goal, Params, RoomKind};
use crate::enemy::{Enemy, Kind};
use crate::field;
use crate::loot::{Loot, LootKind};
use crate::map::Cell;
use crate::room::{PropKind, Room};
use crate::route::{NodeKind, Route};
use crate::sfx::{self, Sfx};
use crate::tuning as t;
use crate::world::{Gift, Npc, Who, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Place {
    #[default]
    Arena,
    Field,
    Dungeon,
    /// 쉼터(camp.rs).
    Camp,
    /// 큰 나무 아래 마을(village.rs, docs/decisions/0045). 런과 런 사이.
    Village,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transition {
    ExitDungeon,
    Respawn,
    /// 다음 쉼터로(필드의 굴 입구, 길의 보스를 이긴 뒤).
    EnterCamp,
    /// 길 지도에서 고른 굴로.
    EnterNode,
}

/// 필드 공터의 보스(들쥐 대장).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BossFight {
    /// 공터 문이 닫혀 있다(튜토리얼 중).
    #[default]
    Locked,
    /// 문이 열렸고 대장이 잔다.
    Waiting,
    Fight,
    Done,
}

/// 앱에 알릴 일(컷씬·화면). 앱이 비운다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdvEvent {
    BossIntro,
    BossBeaten,
    /// 길 지도를 연다(쉼터의 뿌리 길, 굴의 출구).
    RouteOpen,
    /// 쉼터에 처음 닿음(번호).
    CampArrive(u32),
    /// 마지막 길의 주인을 이기고 출구를 밟았다: 가장 깊은 곳(엔딩은 앱, 0042).
    Deepest,
}

/// 확인 버튼으로 하는 일(쉼터).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Interact {
    Talk(Who),
    /// 빈 자리(모모가 돌아올 곳).
    Seat,
    Gate,
}

/// 어두워졌다(반) → 장소를 바꾸고 → 밝아진다(반). 그동안 로직은 멈춘다.
#[derive(Clone, Copy, Debug)]
pub struct Fade {
    pub kind: Transition,
    pub t: f32,
    swapped: bool,
}

/// 던전에 있는 동안 맡겨 둔 필드·쉼터.
struct Stash {
    room: Room,
    enemies: Vec<Enemy>,
    loot: Vec<Loot>,
    neighbors: Vec<Npc>,
    place: Place,
    biome: Biome,
}

#[derive(Default)]
pub struct Adventure {
    pub place: Place,
    pub dungeon: Option<Dungeon>,
    field: Option<Stash>,
    /// 다시 설 곳(필드 좌표). 저장 지점에 닿으면 바뀐다.
    pub save_pos: Vec2,
    /// 필드의 굴 입구(던전에서 나오면 그 앞).
    pub entrance_pos: Vec2,
    /// 다음에 만들 던전의 깊이. 보스를 잡고 나오면 +1.
    pub depth: u32,
    pub current_room: (i32, i32),
    pub fade: Option<Fade>,
    /// 카메라 왼쪽 위(월드 좌표)와 직전 틱 값(보간).
    pub cam: Vec2,
    pub prev_cam: Vec2,
    boss_cleared: bool,
    seed: u64,
    /// 몇 번째 쉼터까지 왔나(0 = 아직 필드).
    pub stage: u32,
    /// 쓰러지면 돌아갈 곳(필드 또는 쉼터). save_pos는 이 장소의 좌표.
    pub home: Place,
    /// 지금 쉼터에서 다음 쉼터까지의 길.
    pub route: Option<Route>,
    /// 지금 들어와 있는 길의 칸(없으면 필드에서 들어온 예전 던전).
    pub node: Option<(usize, usize)>,
    pub boss: BossFight,
    /// 공터(던전 방과 같은 20×12칸 사각형)와 대장이 자던 곳·마지막으로 있던 곳.
    pub glade: Option<Rect>,
    boss_home: Vec2,
    pub boss_last: Vec2,
    pub events: Vec<AdvEvent>,
    /// 직전 틱에 서 있던 칸(출구를 밟는 순간에만 길 지도를 연다).
    last_cell: Option<Cell>,
    /// 이 쉼터에서 뿌리 심장을 샀나(쉼터마다 하나).
    pub heart_sold: bool,
    /// 마을이면 그 모습(이웃·꾸미기·나무)과 자리 표시(0045).
    pub village: Option<crate::village::State>,
    pub marks: Vec<(char, Vec2)>,
}

impl World {
    /// 필드에서 시작하는 모험 월드.
    pub fn field() -> World {
        let p = field::parse();
        let mut w = World::empty();
        let entrance = p.grid.find(Cell::Entrance).first().copied();
        w.room = Room::from_grid(p.grid, p.props, p.spawn_points);
        w.player.pos = p.start;
        w.player.prev_pos = p.start;
        for (kind, pos) in p.enemies {
            for off in crate::species::pack_offsets(kind) {
                let mut e = Enemy::new(kind, pos + off);
                e.asleep = true;
                w.enemies.push(e);
            }
        }
        w.adv.place = Place::Field;
        w.adv.home = Place::Field;
        w.adv.save_pos = p.start;
        w.adv.entrance_pos = entrance.unwrap_or(p.start);
        w.adv.seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
        // 공터: 문 칸(두 칸 폭)의 왼쪽 위에서 9칸 왼쪽이 방의 왼쪽 벽(던전 방과 같은 모양).
        let doors = w
            .room
            .grid
            .as_ref()
            .map(|g| g.find(Cell::Door))
            .unwrap_or_default();
        if let Some(d) = doors.iter().copied().reduce(|a, b| a.min(b)) {
            let tl = d - vec2(8.0, 8.0);
            w.adv.glade = Some(Rect::new(
                tl.x - 9.0 * 16.0,
                tl.y,
                dungeon::ROOM_W as f32 * 16.0,
                dungeon::ROOM_H as f32 * 16.0,
            ));
        }
        if let Some(c) = w.enemies.iter().find(|e| e.kind == Kind::Chief) {
            w.adv.boss_home = c.pos;
            w.adv.boss_last = c.pos;
        }
        w.adv.boss = BossFight::Locked;
        w.room.set_doors_locked(true);
        w.biome = Biome::Meadow;
        w.snap_cam();
        w
    }

    /// 튜토리얼이 끝났다(또는 튜토리얼 없이 시작): 공터 문을 연다.
    pub fn open_glade(&mut self) {
        if self.adv.boss == BossFight::Locked {
            self.adv.boss = BossFight::Waiting;
            if self.adv.place == Place::Field {
                self.room.set_doors_locked(false);
            }
        }
    }

    /// 보리가 이미 돌아왔다(기록, docs/decisions/0041): 공터는 열려 있고 대장은 없다. 굴 입구도 산다.
    pub fn skip_glade_boss(&mut self) {
        self.enemies.retain(|e| e.kind != Kind::Chief);
        self.adv.boss = BossFight::Done;
        if self.adv.place == Place::Field {
            self.room.set_doors_locked(false);
        }
    }

    /// 큰 나무 아래 마을(0045). 이웃·꾸미기·나무는 `v`(앱이 기록에서 만든 것)를 따른다.
    pub fn village(v: crate::village::State) -> World {
        let p = crate::village::parse();
        let mut w = World::empty();
        w.room = Room::from_grid(p.grid, p.props, p.spawn_points);
        for (ch, at) in &p.marks {
            match ch {
                'K' => w.neighbors.push(Npc::standing(Who::Popo, *at)),
                'b' if v.bori => w.neighbors.push(Npc::standing(Who::Bori, *at)),
                'H' if v.halmae => w.neighbors.push(Npc::standing(Who::Halmae, *at)),
                _ => {}
            }
        }
        w.adv.marks = p.marks.clone();
        w.adv.village = Some(v);
        w.player.pos = p.start;
        w.player.prev_pos = p.start;
        w.adv.place = Place::Village;
        w.adv.home = Place::Village;
        w.adv.save_pos = p.start;
        w.biome = Biome::Meadow;
        w.snap_cam();
        w
    }

    /// 정한 시드로 첫 쉼터에서 시작(오늘의 굴, docs/decisions/0044). 길·굴은 이 시드에서 이어진다.
    pub fn camp_start_seeded(seed: u64) -> World {
        let mut w = World::field();
        w.adv.boss = BossFight::Done;
        w.adv.stage = 0;
        w.adv.seed = seed;
        w.enter_camp();
        w
    }

    /// 쉼터에서 시작하는 모험 월드(개발 시나리오).
    pub fn camp_start(stage: u32) -> World {
        let mut w = World::field();
        w.adv.boss = BossFight::Done;
        w.adv.stage = stage.saturating_sub(1);
        w.enter_camp();
        w
    }

    /// 앱이 부른다: 길 지도에서 칸을 골랐다. 고를 수 있는 칸이면 어두워졌다가 그 굴로.
    pub fn choose_node(&mut self, at: (usize, usize)) -> bool {
        let ok = self.adv.route.as_mut().is_some_and(|r| r.pick(at));
        if ok {
            self.start_fade(Transition::EnterNode);
        }
        ok
    }

    /// 확인 버튼이 지금 할 일(쉼터에서 이웃·빈 자리·뿌리 길 곁).
    pub fn interactable(&self) -> Option<Interact> {
        let hub = matches!(self.adv.place, Place::Camp | Place::Village);
        if !hub || !self.player.is_alive() || self.adv.fade.is_some() {
            return None;
        }
        let p = self.player.pos;
        let near = |q: Vec2| (q - p).length() < t::INTERACT_RADIUS;
        if let Some(n) = self.neighbors.iter().find(|n| near(n.pos)) {
            return Some(Interact::Talk(n.who));
        }
        if self
            .room
            .props
            .iter()
            .any(|pr| pr.kind == PropKind::Seat && near(pr.rect.center()))
        {
            return Some(Interact::Seat);
        }
        let gates = self
            .room
            .grid
            .as_ref()
            .map(|g| g.find(Cell::Gate))
            .unwrap_or_default();
        if gates.iter().any(|g| near(*g)) {
            return Some(Interact::Gate);
        }
        None
    }

    /// 확인 말풍선을 띄울 자리(이웃·자리·길 위).
    pub fn interact_anchor(&self) -> Option<Vec2> {
        let p = self.player.pos;
        let mut best: Option<(f32, Vec2)> = None;
        let mut consider = |q: Vec2, up: f32| {
            let d = (q - p).length();
            if d < t::INTERACT_RADIUS && best.is_none_or(|b| d < b.0) {
                best = Some((d, q - vec2(0.0, up)));
            }
        };
        for n in &self.neighbors {
            consider(n.pos, if n.who == Who::Bori { 26.0 } else { 16.0 });
        }
        for pr in &self.room.props {
            if pr.kind == PropKind::Seat {
                consider(pr.rect.center(), 12.0);
            }
        }
        if let Some(g) = &self.room.grid {
            for q in g.find(Cell::Gate) {
                consider(q, 16.0);
            }
        }
        self.interactable().and(best.map(|b| b.1))
    }

    /// 틱 앞: 장소를 바꾸는 중이면 진행하고 true(이번 틱 로직은 건너뛴다).
    pub fn adventure_pre(&mut self, sfx: &Sfx) -> bool {
        let Some(mut f) = self.adv.fade else {
            return false;
        };
        f.t += t::TICK_DT;
        if !f.swapped && f.t >= t::FADE_HALF {
            f.swapped = true;
            match f.kind {
                Transition::ExitDungeon => self.exit_dungeon(),
                Transition::Respawn => self.respawn_at_save(),
                Transition::EnterCamp => self.enter_camp(),
                Transition::EnterNode => self.enter_node(),
            }
            sfx.play(sfx::Id::Emerge);
        }
        self.adv.fade = if f.t >= t::FADE_HALF * 2.0 {
            None
        } else {
            Some(f)
        };
        self.adv.prev_cam = self.adv.cam;
        true
    }

    /// 틱 뒤: 칸 위의 것(입구·출구·저장), 던전 방(잠금·클리어·보상), 카메라.
    pub fn adventure_post(&mut self, sfx: &Sfx) {
        if self.adv.place == Place::Arena {
            return;
        }
        if self.cell_triggers() {
            sfx.play(sfx::Id::Heal);
        }
        if self.adv.place == Place::Field {
            if let Some(id) = self.field_boss() {
                sfx.play(id);
            }
        }
        if self.adv.place == Place::Dungeon {
            self.dungeon_rooms(sfx);
        }
        self.update_cam();
    }

    /// 발밑 칸: 입구·출구(장소 바꾸기, 길 지도), 저장 지점. 저장 지점을 새로 밟았으면 true(소리).
    fn cell_triggers(&mut self) -> bool {
        let cell = self.room.cell_at(self.player.pos);
        let stepped = cell != self.adv.last_cell;
        self.adv.last_cell = cell;
        if self.player.is_alive() && self.adv.fade.is_none() {
            match cell {
                // 굴 입구는 공터의 대장을 이긴 뒤에야 산다. 그 아래는 쉼터.
                Some(Cell::Entrance)
                    if self.adv.place == Place::Field && self.adv.boss == BossFight::Done =>
                {
                    self.start_fade(Transition::EnterCamp)
                }
                Some(Cell::Exit) if self.adv.place == Place::Dungeon => match self.adv.node {
                    None => self.start_fade(Transition::ExitDungeon),
                    Some(at) => {
                        let route = self.adv.route.as_ref();
                        let boss = route.is_some_and(|r| r.node(at).kind == NodeKind::Boss);
                        let last = route.is_some_and(|r| r.is_final());
                        if boss && last {
                            if stepped {
                                self.adv.events.push(AdvEvent::Deepest);
                            }
                        } else if boss {
                            self.start_fade(Transition::EnterCamp);
                        } else if stepped {
                            self.adv.events.push(AdvEvent::RouteOpen);
                        }
                    }
                },
                Some(Cell::Save) => return self.touch_save(),
                _ => {}
            }
        }
        false
    }

    /// 공터의 대장: 안으로 들어서면 문을 닫고 깨운다. 쓰러지면 문을 열고 보리가 돌아온다(컷씬은 앱). 낼 소리를 돌려준다.
    fn field_boss(&mut self) -> Option<sfx::Id> {
        let glade = self.adv.glade?;
        let inside = Rect::new(
            glade.x + 24.0,
            glade.y + 40.0,
            glade.w - 48.0,
            glade.h - 64.0,
        );
        match self.adv.boss {
            BossFight::Waiting if inside.contains(self.player.pos) && self.player.is_alive() => {
                self.room.set_doors_locked(true);
                for e in self.enemies.iter_mut().filter(|e| e.kind == Kind::Chief) {
                    e.asleep = false;
                    e.wake_delay = t::ROOM_WAKE_DELAY * 1.5;
                }
                self.adv.boss = BossFight::Fight;
                self.adv.events.push(AdvEvent::BossIntro);
                self.feel.add_trauma(0.35);
                return Some(sfx::Id::WallHit);
            }
            BossFight::Fight => {
                match self.enemies.iter().find(|e| e.kind == Kind::Chief) {
                    Some(c) => self.adv.boss_last = c.pos,
                    None => {
                        // 공터에 남은 부하도 정신이 든다(흩어진다).
                        self.enemies.retain(|e| !glade.contains(e.pos));
                        self.room.set_doors_locked(false);
                        self.adv.boss = BossFight::Done;
                        let at = self.adv.boss_last;
                        self.loot
                            .push(Loot::place(LootKind::RootHeart, at + vec2(0.0, 18.0)));
                        for _ in 0..t::BOSS_SEEDS {
                            self.loot.push(Loot::spawn(LootKind::Seed, at));
                        }
                        self.adv.events.push(AdvEvent::BossBeaten);
                        self.toast = Some((|s| s.toast_chief, 3.0));
                        return Some(sfx::Id::Emerge);
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// 쓰러져서 다시 서면 공터 싸움을 처음부터(대장은 다시 잔다, 부하는 사라진다).
    fn reset_field_boss(&mut self) {
        if self.adv.boss != BossFight::Fight {
            return;
        }
        if let Some(glade) = self.adv.glade {
            self.enemies
                .retain(|e| !glade.contains(e.pos) && e.kind != Kind::Chief);
        }
        let mut c = Enemy::new(Kind::Chief, self.adv.boss_home);
        c.asleep = true;
        self.enemies.push(c);
        self.room.set_doors_locked(false);
        self.adv.boss = BossFight::Waiting;
    }

    pub fn start_fade(&mut self, kind: Transition) {
        if self.adv.fade.is_none() {
            self.adv.fade = Some(Fade {
                kind,
                t: 0.0,
                swapped: false,
            });
        }
    }

    /// 화면을 덮는 검정(0..1). 렌더가 월드 위에 그린다.
    pub fn fade_alpha(&self) -> f32 {
        match self.adv.fade {
            Some(f) => {
                let k = f.t / t::FADE_HALF;
                if k < 1.0 {
                    k
                } else {
                    (2.0 - k).max(0.0)
                }
            }
            None => 0.0,
        }
    }

    /// 앱이 부른다: 쓰러진 뒤 확인을 누르면 저장 지점으로(모험일 때).
    pub fn begin_respawn(&mut self) {
        self.start_fade(Transition::Respawn);
    }

    fn touch_save(&mut self) -> bool {
        // 저장 지점 칸 가운데.
        let at = (self.player.pos / 16.0).floor() * 16.0 + vec2(8.0, 8.0);
        if (self.adv.save_pos - at).length() < 1.0 {
            return false;
        }
        self.adv.save_pos = at;
        self.player.heal(self.player.stats.max_hp);
        self.toast = Some((|s| s.toast_save, 2.5));
        true
    }

    /// 개발 시나리오용: 어두워지는 연출 없이 바로 던전으로.
    pub fn enter_dungeon_now(&mut self) {
        self.enter_dungeon();
    }

    fn enter_dungeon(&mut self) {
        let prm = Params::classic(self.adv.depth);
        let biome = if self.adv.depth == 0 {
            Biome::Roots
        } else {
            Biome::Deep
        };
        self.adv.node = None;
        self.build_dungeon(&prm, biome);
    }

    /// 길에서 고른 칸의 굴로. 쉼터에서 오면 쉼터를 맡기고, 굴에서 오면 굴만 바꾼다.
    fn enter_node(&mut self) {
        let Some((at, prm, biome)) = self
            .adv
            .route
            .as_ref()
            .and_then(|r| r.at.map(|at| (at, r.params(at), r.node(at).biome)))
        else {
            return;
        };
        self.adv.node = Some(at);
        self.build_dungeon(&prm, biome);
    }

    fn build_dungeon(&mut self, prm: &Params, biome: Biome) {
        self.adv.seed = self.adv.seed.wrapping_add(0x9E37_79B9);
        let (d, room) = dungeon::generate(self.adv.seed, prm);
        let old_room = std::mem::replace(&mut self.room, room);
        if self.adv.place != Place::Dungeon {
            self.adv.field = Some(Stash {
                room: old_room,
                enemies: std::mem::take(&mut self.enemies),
                loot: std::mem::take(&mut self.loot),
                neighbors: std::mem::take(&mut self.neighbors),
                place: self.adv.place,
                biome: self.biome,
            });
        } else {
            self.enemies.clear();
            self.loot.clear();
        }
        self.projectiles.clear();
        self.player.pos = d.start_pos();
        self.player.prev_pos = self.player.pos;
        self.adv.current_room = d.start;
        self.adv.dungeon = Some(d);
        self.adv.place = Place::Dungeon;
        self.adv.boss_cleared = false;
        self.adv.last_cell = None;
        self.biome = biome;
        self.snap_cam();
    }

    /// 맡겨 둔 필드·쉼터로 돌아온다.
    fn restore_field(&mut self) {
        if let Some(s) = self.adv.field.take() {
            self.room = s.room;
            self.enemies = s.enemies;
            self.loot = s.loot;
            self.neighbors = s.neighbors;
            self.adv.place = s.place;
            self.biome = s.biome;
        }
        self.projectiles.clear();
        self.adv.dungeon = None;
        self.adv.node = None;
    }

    /// 다음 쉼터: 필드·굴은 버리고 쉼터를 새로 짓는다. 체력 가득, 저장 고리가 다시 설 곳, 새 길.
    fn enter_camp(&mut self) {
        let p = crate::camp::parse();
        self.adv.field = None;
        self.adv.dungeon = None;
        self.adv.node = None;
        self.adv.stage += 1;
        let stage = self.adv.stage;
        self.room = Room::from_grid(p.grid, p.props, p.spawn_points);
        self.enemies.clear();
        self.loot.clear();
        self.projectiles.clear();
        self.npc = None;
        self.neighbors.clear();
        for (ch, at) in &p.marks {
            match ch {
                'K' => self.neighbors.push(Npc::standing(Who::Popo, *at)),
                'b' => self.neighbors.push(Npc::standing(Who::Bori, *at)),
                _ => {}
            }
        }
        self.player.pos = p.start;
        self.player.prev_pos = p.start;
        self.player.hp = self.player.stats.max_hp;
        self.adv.save_pos = self
            .room
            .grid
            .as_ref()
            .and_then(|g| g.find(Cell::Save).first().copied())
            .unwrap_or(p.start);
        self.adv.home = Place::Camp;
        self.adv.place = Place::Camp;
        self.adv.heart_sold = false;
        self.adv.seed = self.adv.seed.wrapping_add(0x0051_A7E5);
        self.adv.route = Some(Route::generate(self.adv.seed, stage));
        self.adv.last_cell = None;
        self.adv.events.push(AdvEvent::CampArrive(stage));
        self.biome = Biome::Roots;
        self.toast = Some((|s| s.toast_camp, 2.5));
        self.snap_cam();
    }

    fn exit_dungeon(&mut self) {
        if self.adv.boss_cleared {
            self.adv.depth += 1;
        }
        self.restore_field();
        self.player.pos = self.adv.entrance_pos + vec2(0.0, 22.0);
        self.player.prev_pos = self.player.pos;
        self.snap_cam();
    }

    fn respawn_at_save(&mut self) {
        if self.adv.place == Place::Dungeon {
            self.restore_field();
        }
        if self.adv.place == Place::Field {
            self.reset_field_boss();
        }
        if self.adv.home == Place::Camp {
            // 길은 새로 짠다(같은 쉼터에서 다시 고른다).
            let stage = self.adv.stage;
            self.adv.seed = self.adv.seed.wrapping_add(0x7F4A_7C15);
            self.adv.route = Some(Route::generate(self.adv.seed, stage));
        }
        self.player.revive();
        self.player.hp = self.player.stats.max_hp;
        self.player.pos = self.adv.save_pos;
        self.player.prev_pos = self.player.pos;
        self.glowbugs = self.glowbugs * 7 / 10;
        self.toast = Some((|s| s.toast_respawn, 2.5));
        self.snap_cam();
    }

    /// 던전: 방이 바뀌면 들르고, 전투방이면 안에 들어선 순간 잠그고 적을 깨운다. 다 잡으면 열고 보상.
    fn dungeon_rooms(&mut self, sfx: &Sfx) {
        let at = dungeon::room_at(self.player.pos);
        let Some(d) = self.adv.dungeon.as_mut() else {
            return;
        };
        if at != self.adv.current_room && d.room(at).is_some() {
            self.adv.current_room = at;
        }
        let goal = d.goal;
        let Some(room) = d.room_mut(self.adv.current_room) else {
            return;
        };
        if !room.visited {
            room.visited = true;
            if room.kind == RoomKind::Rest && goal == Goal::Spring {
                // 길의 샘: 체력을 가득 채운다.
                self.player.heal(self.player.stats.max_hp);
                sfx.play(sfx::Id::Heal);
                self.toast = Some((|s| s.toast_spring, 2.5));
            }
            if room.kind == RoomKind::Rest {
                let c = room.rect().center();
                // 방 가운데에 놓는다(굴러가지 않으니 묶을 필요 없음).
                for k in 0..3 {
                    self.loot.push(Loot::place(
                        LootKind::Berry,
                        c + vec2(-20.0 + k as f32 * 20.0, 8.0),
                    ));
                }
            }
        }
        let fights = matches!(room.kind, RoomKind::Combat | RoomKind::Boss);
        if fights
            && !room.cleared
            && !self.room.doors_locked()
            && room.interior().contains(self.player.pos)
        {
            self.room.set_doors_locked(true);
            for (kind, pos) in room.enemies.drain(..) {
                for off in crate::species::pack_offsets(kind) {
                    let mut e = Enemy::new(kind, pos + off);
                    e.wake_delay = t::ROOM_WAKE_DELAY;
                    self.enemies.push(e);
                }
            }
            sfx.play(sfx::Id::WallHit);
            self.feel.add_trauma(0.25);
        }
        if self.room.doors_locked() && self.enemies.is_empty() {
            self.room.set_doors_locked(false);
            room.cleared = true;
            sfx.play(sfx::Id::Emerge);
            if room.kind == RoomKind::Boss {
                let c = room.rect().center();
                if let Some(g) = self.room.grid.as_mut() {
                    g.set((c.x / 16.0) as usize, (c.y / 16.0) as usize, Cell::Exit);
                }
                let at = c + vec2(0.0, 24.0);
                if goal == Goal::Boss {
                    self.loot.push(Loot::place(LootKind::RootHeart, at));
                    for _ in 0..t::BOSS_SEEDS {
                        self.loot.push(Loot::spawn(LootKind::Seed, at));
                    }
                    self.adv.boss_cleared = true;
                    self.toast = Some((|s| s.toast_boss, 3.0));
                } else {
                    for _ in 0..t::PACK_SEEDS {
                        self.loot.push(Loot::spawn(LootKind::Seed, at));
                    }
                    self.toast = Some((|s| s.toast_goal, 2.5));
                }
            }
        }
        // 보물 상자: 닿으면 연다 → 씨앗 한 줌 + 아직 없는 장신구 하나(다 있으면 반딧불).
        let ppos = self.player.pos;
        let near: Option<usize> = self.room.props.iter().position(|p| {
            matches!(p.kind, PropKind::Chest { opened: false })
                && (p.rect.center() - ppos).length() < 18.0
        });
        if let Some(i) = near {
            self.room.open_chest(i);
            let at = self.room.props[i].rect.center();
            for _ in 0..rand::gen_range(t::CHEST_SEEDS.0, t::CHEST_SEEDS.1 + 1) {
                self.push_loot(Loot::spawn(LootKind::Seed, at));
            }
            let missing: Vec<usize> = (0..crate::equipment::CHARMS.len())
                .filter(|c| !self.charms_found.contains(c))
                .collect();
            if missing.is_empty() {
                for _ in 0..12 {
                    self.push_loot(Loot::spawn(
                        LootKind::Glowbug,
                        self.room.props[i].rect.center(),
                    ));
                }
            } else {
                let c = missing[rand::gen_range(0, missing.len())];
                self.charms_found.push(c);
                self.gifts.push(Gift::Charm(c));
            }
            sfx.play(sfx::Id::Emerge);
        }
    }

    /// 떨어진 것이 굴러갈 수 있는 곳: 던전이면 지금 방의 안쪽(벽 칸과 문 칸을 뺀 곳), 그 밖은 제한 없음.
    pub fn loot_bounds(&self) -> Option<Rect> {
        match self.adv.place {
            Place::Dungeon => {
                let r = dungeon::room_rect(self.adv.current_room);
                Some(Rect::new(r.x + 22.0, r.y + 38.0, r.w - 44.0, r.h - 60.0))
            }
            _ => None,
        }
    }

    pub fn push_loot(&mut self, l: Loot) {
        let b = self.loot_bounds();
        self.loot.push(l.within(b));
    }

    /// 카메라가 머무는 사각형.
    pub fn cam_bounds(&self) -> Rect {
        match self.adv.place {
            Place::Arena => Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H),
            Place::Field => match (self.adv.boss, self.adv.glade) {
                // 대장과 싸우는 동안은 공터를 한 방처럼 보여 준다.
                (BossFight::Fight, Some(g)) => g,
                _ => self.room.inner,
            },
            Place::Camp | Place::Village => self.room.inner,
            Place::Dungeon => dungeon::room_rect(self.adv.current_room),
        }
    }

    fn cam_target(&self) -> Vec2 {
        let b = self.cam_bounds();
        let view = vec2(t::VIEW_W, t::VIEW_H);
        let want = self.player.pos - view / 2.0;
        let axis = |w: f32, lo: f32, size: f32, v: f32| {
            if size <= w {
                lo + (size - w) / 2.0
            } else {
                v.clamp(lo, lo + size - w)
            }
        };
        vec2(
            axis(view.x, b.x, b.w, want.x),
            axis(view.y, b.y, b.h, want.y),
        )
    }

    fn update_cam(&mut self) {
        self.adv.prev_cam = self.adv.cam;
        let target = self.cam_target();
        let k = 1.0 - (-t::CAM_FOLLOW * t::TICK_DT).exp();
        self.adv.cam += (target - self.adv.cam) * k;
        if (target - self.adv.cam).length() < 0.25 {
            self.adv.cam = target;
        }
    }

    pub fn snap_cam(&mut self) {
        self.adv.cam = self.cam_target();
        self.adv.prev_cam = self.adv.cam;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_round_trip_and_respawn() {
        let mut w = World::field();
        assert_eq!(w.adv.place, Place::Field);
        let field_enemies = w.enemies.len();
        assert!(field_enemies > 0 && w.enemies.iter().all(|e| e.asleep));
        w.enter_dungeon_now();
        assert_eq!(w.adv.place, Place::Dungeon);
        assert!(w.enemies.is_empty(), "던전 시작방엔 적이 없다");
        assert_eq!(w.room.cell_at(w.player.pos), Some(Cell::Floor));
        // 쓰러져서 저장 지점으로: 필드가 돌아오고, 체력 가득, 반딧불 30% 잃음.
        w.glowbugs = 10;
        w.player.hp = 0;
        w.respawn_at_save();
        assert_eq!(w.adv.place, Place::Field);
        assert!(w.adv.dungeon.is_none());
        assert_eq!(w.enemies.len(), field_enemies, "맡겨 둔 필드 적이 돌아온다");
        assert_eq!(w.player.hp, w.player.stats.max_hp);
        assert_eq!(w.player.pos, w.adv.save_pos);
        assert_eq!(w.glowbugs, 7);
        // 다시 들어갔다 나오면 굴 입구 앞.
        w.enter_dungeon_now();
        w.exit_dungeon();
        assert_eq!(w.adv.place, Place::Field);
        assert!((w.player.pos - w.adv.entrance_pos).length() < 40.0);
        assert_eq!(w.adv.depth, 0, "보스를 안 잡았으면 깊이 그대로");
    }

    /// 공터: 튜토리얼 전엔 문이 닫혀 있고, 들어서면 싸움, 대장이 쓰러지면 입구 → 쉼터.
    #[test]
    fn glade_boss_then_camp_then_route() {
        let mut w = World::field();
        let glade = w.adv.glade.expect("공터");
        assert!(w.room.doors_locked() && w.adv.boss == BossFight::Locked);
        w.open_glade();
        assert!(!w.room.doors_locked());
        w.player.pos = glade.center();
        w.field_boss();
        assert_eq!(w.adv.boss, BossFight::Fight);
        assert!(w.room.doors_locked());
        assert!(w.adv.events.contains(&AdvEvent::BossIntro));
        // 쓰러져서 다시 서면 처음부터.
        w.respawn_at_save();
        assert_eq!(w.adv.boss, BossFight::Waiting);
        assert!(!w.room.doors_locked());
        assert_eq!(
            w.enemies.iter().filter(|e| e.kind == Kind::Chief).count(),
            1
        );
        // 다시 들어가 이긴다.
        w.player.pos = glade.center();
        w.field_boss();
        w.enemies.retain(|e| e.kind != Kind::Chief);
        w.field_boss();
        assert_eq!(w.adv.boss, BossFight::Done);
        assert!(w.adv.events.contains(&AdvEvent::BossBeaten));
        assert!(glade.contains(w.adv.boss_last));
        // 입구 → 쉼터.
        w.enter_camp();
        assert_eq!(
            (w.adv.place, w.adv.home, w.adv.stage),
            (Place::Camp, Place::Camp, 1)
        );
        assert!(w.enemies.is_empty());
        assert!(w.neighbors.iter().any(|n| n.who == Who::Popo));
        // 길: 첫 칸을 고르면 굴, 쓰러지면 쉼터로(길은 새로).
        let first = w.adv.route.as_ref().unwrap().choices()[0];
        assert!(w.choose_node(first));
        w.enter_node();
        assert_eq!(w.adv.place, Place::Dungeon);
        assert!(w.neighbors.is_empty(), "이웃은 쉼터에 맡겨 둔다");
        w.player.hp = 0;
        w.respawn_at_save();
        assert_eq!(w.adv.place, Place::Camp);
        assert!(w.adv.route.as_ref().unwrap().at.is_none());
        assert_eq!(w.player.pos, w.adv.save_pos);
        assert!(!w.neighbors.is_empty());
    }

    /// 보리가 이미 돌아왔으면(기록): 대장 없이 공터가 열려 있고, 들어가도 싸움이 시작되지 않는다.
    #[test]
    fn bori_back_skips_the_glade_boss() {
        let mut w = World::field();
        w.skip_glade_boss();
        assert!(!w.room.doors_locked());
        assert!(w.enemies.iter().all(|e| e.kind != Kind::Chief));
        w.player.pos = w.adv.glade.expect("공터").center();
        w.field_boss();
        assert_eq!(w.adv.boss, BossFight::Done);
        assert!(!w.adv.events.contains(&AdvEvent::BossIntro));
    }

    /// 길의 굴: 출구를 밟는 순간 길 지도를 한 번 연다. 보스 칸의 출구는 다음 쉼터로.
    #[test]
    fn route_exit_opens_map_then_boss_exit_goes_to_next_camp() {
        let mut w = World::camp_start(1);
        w.adv.events.clear();
        let first = w.adv.route.as_ref().unwrap().choices()[0];
        assert!(w.choose_node(first));
        w.adv.fade = None;
        w.enter_node();
        // 출구 칸 하나를 시작방에 놓고 밟는다(끝방을 깬 것과 같다).
        let p = w.player.pos;
        if let Some(g) = w.room.grid.as_mut() {
            g.set((p.x / 16.0) as usize, (p.y / 16.0) as usize, Cell::Exit);
        }
        w.cell_triggers();
        assert_eq!(w.adv.events, vec![AdvEvent::RouteOpen]);
        w.cell_triggers();
        assert_eq!(w.adv.events.len(), 1, "서 있는 동안 다시 열지 않는다");
        // 보스 칸까지 건너뛰고 출구 → 다음 쉼터.
        let r = w.adv.route.as_mut().unwrap();
        let boss = (r.layers.len() - 1, 0);
        r.at = Some((boss.0 - 1, r.layers[boss.0 - 1].len() - 1));
        assert!(w.choose_node(boss));
        w.adv.fade = None;
        w.enter_node();
        let p = w.player.pos;
        if let Some(g) = w.room.grid.as_mut() {
            g.set((p.x / 16.0) as usize, (p.y / 16.0) as usize, Cell::Exit);
        }
        w.cell_triggers();
        assert_eq!(w.adv.fade.map(|f| f.kind), Some(Transition::EnterCamp));
        w.enter_camp();
        assert_eq!((w.adv.place, w.adv.stage), (Place::Camp, 2));
    }

    /// 마지막 길의 주인을 이기고 출구를 밟으면 쉼터가 아니라 가장 깊은 곳(한 번만).
    #[test]
    fn final_route_boss_exit_reaches_the_deepest_place() {
        let mut w = World::camp_start(t::FINAL_STAGE);
        assert!(w.adv.route.as_ref().unwrap().is_final());
        w.adv.events.clear();
        let r = w.adv.route.as_mut().unwrap();
        let boss = (r.layers.len() - 1, 0);
        r.at = Some((boss.0 - 1, 0));
        assert!(w.choose_node(boss));
        w.adv.fade = None;
        w.enter_node();
        let p = w.player.pos;
        if let Some(g) = w.room.grid.as_mut() {
            g.set((p.x / 16.0) as usize, (p.y / 16.0) as usize, Cell::Exit);
        }
        w.cell_triggers();
        w.cell_triggers();
        assert_eq!(w.adv.events, vec![AdvEvent::Deepest]);
        assert!(w.adv.fade.is_none(), "쉼터로 가지 않는다");
    }

    /// 오늘의 굴: 같은 시드면 같은 첫 길.
    #[test]
    fn seeded_camp_start_gives_the_same_route() {
        let a = World::camp_start_seeded(42);
        let b = World::camp_start_seeded(42);
        let kinds = |w: &World| {
            let r = w.adv.route.as_ref().unwrap();
            r.layers
                .iter()
                .map(|l| l.iter().map(|n| n.kind).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        assert_eq!(kinds(&a), kinds(&b));
        assert_eq!((a.adv.place, a.adv.stage), (Place::Camp, 1));
    }

    #[test]
    fn camera_stays_in_bounds() {
        let mut w = World::field();
        for p in [vec2(0.0, 0.0), vec2(10_000.0, 10_000.0), w.room.center()] {
            w.player.pos = p;
            w.snap_cam();
            let b = w.cam_bounds();
            assert!(w.adv.cam.x >= b.x && w.adv.cam.x + t::VIEW_W <= b.right() + 0.01);
            assert!(w.adv.cam.y >= b.y && w.adv.cam.y + t::VIEW_H <= b.bottom() + 0.01);
        }
    }
}
