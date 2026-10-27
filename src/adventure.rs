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
use crate::depth;
use crate::dungeon::{self, Dungeon, Goal, Params, RoomKind};
use crate::enemy::{Enemy, Kind};
use crate::field;
use crate::loot::{Loot, LootKind};
use crate::map::Cell;
use crate::room::{PropKind, Room};
use crate::route::{NodeKind, Route};
use crate::sfx;
use crate::tuning as t;
use crate::world::{clods, puff, ring, Gift, Npc, Spring, Who, World, DIRT};

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
    /// 가장 깊은 곳(0162): 할매와 매듭 셋의 한 방. 콩이를 이기고 출구를 밟으면 온다.
    Deep,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transition {
    ExitDungeon,
    Respawn,
    /// 다음 쉼터로(필드의 굴 입구, 길의 보스를 이긴 뒤).
    EnterCamp,
    /// 길 지도에서 고른 굴로.
    EnterNode,
    /// 가장 깊은 곳으로(마지막 길의 보스 출구, 0162). 내려가는 전환.
    EnterDeep,
}

impl Transition {
    /// 더 깊이 내려가는 전환: 검정 대신 지층 단면이 흐르고(0156) 조금 더 길다.
    pub fn descends(self) -> bool {
        matches!(
            self,
            Transition::EnterCamp | Transition::EnterNode | Transition::EnterDeep
        )
    }

    /// 어두워지는(=밝아지는) 반의 길이.
    pub fn half(self) -> f32 {
        if self.descends() {
            t::DESCENT_HALF
        } else {
            t::FADE_HALF
        }
    }
}

/// 내려가는 전환의 지금 모습(렌더가 지층 단면을 그린다, 0156).
#[derive(Clone, Copy, Debug)]
pub struct Descent {
    /// 화면에 보일 깊이(m): 떠난 곳 → 닿을 곳.
    pub depth_m: f32,
    /// 단면이 월드를 덮는 정도(0..1). 처음·끝에서 월드와 섞인다.
    pub cover: f32,
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

/// 쉼터에 들어설 때 시드에 더하는 수(쉼터마다 다른 길 지도). 이어하기가 되돌릴 때도 쓴다.
const CAMP_SEED_STEP: u64 = 0x0051_A7E5;

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
    /// 첫 길 끝 방에 모모가 있다 / 모모를 이겼다(0046). 몇 번째인지는 앱이 안다.
    MomoIntro,
    MomoBeaten,
    /// 둘째 길 끝 방에 도토가 있다 / 도토를 이겼다(0052).
    DotoIntro,
    DotoBeaten,
    /// 셋째 길 끝 방에 콩이가 있다 / 콩이를 이겼다(0054).
    KongiIntro,
    KongiBeaten,
    /// 가장 깊은 곳(0162): 들어섰다(할매의 말) / 매듭 셋을 끊었다(엔딩) / 세 번 감겨 실에 잠겼다(쉼터로).
    DeepIntro,
    DeepDone,
    DeepSleep,
    /// 굴토끼 옛 굴 끝 방에 토끼풀이 있다 / 토끼풀을 이겼다(0165).
    CloverIntro,
    CloverBeaten,
    /// 모모가 남긴 시든 꽃잎을 주웠다(0164): 기록에 남기고 도감 '???' 칸.
    PetalFound,
}

/// 확인 버튼으로 하는 일(쉼터).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Interact {
    Talk(Who),
    /// 빈 자리(모모가 돌아올 곳).
    Seat,
    /// 마을의 큰 나무 밑동: 재주 깨우기(0047).
    Tree,
    /// 마을 물웅덩이: 털빛 바꾸기(0048).
    Puddle,
    /// 마을 할매 곁 실 고리: 실의 꿈(0055).
    Dream,
    Gate,
}

/// 어두워졌다(반) → 장소를 바꾸고 → 밝아진다(반). 그동안 로직은 멈춘다.
#[derive(Clone, Copy, Debug)]
pub struct Fade {
    pub kind: Transition,
    pub t: f32,
    swapped: bool,
    /// 떠난 곳·닿을 곳의 깊이(m, `depth.rs`). 닿을 곳은 장소를 바꾼 뒤 실제 값으로 고친다.
    pub from_m: u32,
    pub to_m: u32,
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
    /// 컷씬이 카메라를 잡아 둔 자리(0159): 있으면 플레이어 대신 이 점을 가운데에 둔다.
    pub cam_hold: Option<Vec2>,
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
    /// 마지막으로 쓰러진 적의 자리. 끝방 보상은 여기에, 출구는 여기서 떨어진 곳에(0060).
    pub last_kill: Vec2,
    pub events: Vec<AdvEvent>,
    /// 직전 틱에 서 있던 칸(출구를 밟는 순간에만 길 지도를 연다).
    last_cell: Option<Cell>,
    /// 이 쉼터에서 뿌리 심장을 샀나(쉼터마다 하나).
    pub heart_sold: bool,
    /// 마을이면 그 모습(이웃·꾸미기·나무)과 자리 표시(0045).
    pub village: Option<crate::village::State>,
    pub marks: Vec<(char, Vec2)>,
    /// 이번 런에서 마지막으로 이름 카드를 보인 지대(같은 지대로 내려가면 다시 띄우지 않는다, 0156).
    pub shown_biome: Option<Biome>,
    /// 지대 이름 카드: (지대, 깊이 m, 남은 시간).
    pub biome_card: Option<(Biome, u32, f32)>,
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
        w.adv.shown_biome = Some(Biome::Meadow);
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

    /// 지름길(0151): 보리가 돌아온 뒤의 런은 넓은 필드를 다 걷지 않고, 남쪽 공터 굴 입구 바로 위에서 시작한다.
    /// 쓰러지면 돌아올 자리(저장)도 여기. 필드는 첫 런·튜토리얼, 그리고 걸어서 올라가 보고 싶을 때를 위해 그대로 둔다.
    pub fn start_near_entrance(&mut self) {
        let at = self.adv.entrance_pos - vec2(0.0, t::FIELD_SHORTCUT_UP);
        self.player.pos = at;
        self.player.prev_pos = at;
        self.adv.save_pos = at;
        self.snap_cam();
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
                'M' if v.momo => w.neighbors.push(Npc::standing(Who::Momo, *at)),
                'd' if v.doto => w.neighbors.push(Npc::standing(Who::Doto, *at)),
                'g' if v.kongi => w.neighbors.push(Npc::standing(Who::Kongi, *at)),
                'j' if v.clover => w.neighbors.push(Npc::standing(Who::Clover, *at)),
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

    /// 쉼터부터 이어하기(0080): 적어 둔 쉼터 번호·시드·깊이로 그 쉼터를 다시 세운다(길 지도도 같다).
    /// `seed`는 쉼터에 들어선 뒤의 값(`adv_seed`). 쉼터 도착 알림은 앱이 버린다(도착 장면을 다시 틀지 않게).
    pub fn resume_camp(&mut self, stage: u32, seed: u64, depth: u32) {
        self.adv.boss = BossFight::Done;
        self.adv.stage = stage.saturating_sub(1);
        self.adv.seed = seed.wrapping_sub(CAMP_SEED_STEP);
        self.adv.depth = depth;
        self.enter_camp();
    }

    /// 지금 모험 시드(쉼터부터 이어하기가 적어 둔다).
    pub fn adv_seed(&self) -> u64 {
        self.adv.seed
    }

    /// 쉼터에서 시작하는 모험 월드(개발 시나리오).
    pub fn camp_start(stage: u32) -> World {
        let mut w = World::field();
        w.adv.boss = BossFight::Done;
        w.adv.stage = stage.saturating_sub(1);
        w.enter_camp();
        w
    }

    /// 개발 시나리오용: 지금 길의 보스 칸 굴로 바로(어두워지는 연출 없이).
    pub fn enter_route_boss_now(&mut self) {
        if let Some(r) = self.adv.route.as_mut() {
            r.at = Some((r.layers.len() - 1, 0));
        }
        self.enter_node();
    }

    /// 개발: 굴토끼 옛 굴(0165)로 바로. 길에 그 칸이 없으면 보스 칸.
    pub fn enter_route_warren_now(&mut self) {
        if let Some(r) = self.adv.route.as_mut() {
            let last = r.layers.len() - 1;
            let i = r.layers[last]
                .iter()
                .position(|n| n.kind == NodeKind::Warren)
                .unwrap_or(0);
            r.at = Some((last, i));
        }
        self.enter_node();
    }

    /// 개발: 길의 샘 칸(0159)으로 바로. 샘 칸이 없는 길이면 끝방 앞 층 첫 칸.
    pub fn enter_route_spring_now(&mut self) {
        if let Some(r) = self.adv.route.as_mut() {
            let found = r.layers.iter().enumerate().find_map(|(l, layer)| {
                layer
                    .iter()
                    .position(|n| n.kind == NodeKind::Spring)
                    .map(|i| (l, i))
            });
            r.at = Some(found.unwrap_or((r.layers.len().saturating_sub(2), 0)));
        }
        self.enter_node();
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
        if self.tree_base().is_some_and(near) {
            return Some(Interact::Tree);
        }
        if self.puddle().is_some_and(near) {
            return Some(Interact::Puddle);
        }
        if self.dream_loop().is_some_and(near) {
            return Some(Interact::Dream);
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

    /// 마을 큰 나무의 밑동(나무 칸 바로 아래, 걸어서 닿는 곳).
    fn tree_base(&self) -> Option<Vec2> {
        self.adv.village?;
        self.adv
            .marks
            .iter()
            .find(|(c, _)| *c == 'Y')
            .map(|(_, at)| *at + vec2(0.0, 14.0))
    }

    /// 마을 물웅덩이(W).
    /// 마을 실 고리(할매가 온 뒤에만 선다, 0055).
    pub fn dream_loop(&self) -> Option<Vec2> {
        let v = self.adv.village?;
        if !v.halmae {
            return None;
        }
        self.adv
            .marks
            .iter()
            .find(|(c, _)| *c == 'X')
            .map(|(_, at)| *at)
    }

    fn puddle(&self) -> Option<Vec2> {
        self.adv.village?;
        self.adv
            .marks
            .iter()
            .find(|(c, _)| *c == 'W')
            .map(|(_, at)| *at)
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
        if let Some(b) = self.tree_base() {
            consider(b, 14.0);
        }
        if let Some(b) = self.puddle() {
            consider(b, 8.0);
        }
        if let Some(b) = self.dream_loop() {
            consider(b, 18.0);
        }
        if let Some(g) = &self.room.grid {
            for q in g.find(Cell::Gate) {
                consider(q, 16.0);
            }
        }
        self.interactable().and(best.map(|b| b.1))
    }

    /// 틱 앞: 장소를 바꾸는 중이면 진행하고 true(이번 틱 로직은 건너뛴다).
    pub fn adventure_pre(&mut self) -> bool {
        let Some(mut f) = self.adv.fade else {
            return false;
        };
        f.t += t::TICK_DT;
        let half = f.kind.half();
        if !f.swapped && f.t >= half {
            self.swap_place(&mut f);
            self.juice.audio.play(sfx::Id::Emerge);
        }
        self.adv.fade = if f.t >= half * 2.0 { None } else { Some(f) };
        self.adv.prev_cam = self.adv.cam;
        true
    }

    /// 어두운 한가운데: 장소를 바꾸고, 닿은 깊이를 적고, 새 지대로 내려왔으면 이름 카드(0156).
    fn swap_place(&mut self, f: &mut Fade) {
        f.swapped = true;
        self.spring = None;
        self.story.petal_at = None;
        self.hazards.cocoons.clear();
        self.hazards.waves.clear();
        self.hazards.thread_lash = None;
        self.exit_at = None;
        self.exit_age = 0.0;
        self.juice.petals = false;
        match f.kind {
            Transition::ExitDungeon => self.exit_dungeon(),
            Transition::Respawn => self.respawn_at_save(),
            Transition::EnterCamp => self.enter_camp(),
            Transition::EnterNode => self.enter_node(),
            Transition::EnterDeep => self.enter_deep(),
        }
        f.to_m = self.depth_m();
        if f.kind.descends() && self.adv.shown_biome != Some(self.biome) {
            self.adv.shown_biome = Some(self.biome);
            self.adv.biome_card = Some((self.biome, f.to_m, t::BIOME_CARD_TIME));
        }
    }

    /// 틱 뒤: 칸 위의 것(입구·출구·저장), 던전 방(잠금·클리어·보상), 카메라.
    pub fn adventure_post(&mut self) {
        if self.adv.place == Place::Arena {
            return;
        }
        if self.cell_triggers() {
            self.juice.audio.play(sfx::Id::Heal);
        }
        if self.spring_tick() {
            self.juice.audio.play(sfx::Id::Heal);
        }
        if self.petal_tick() {
            self.juice.audio.play(sfx::Id::Glowbug);
        }
        if self.adv.place == Place::Field {
            if let Some(id) = self.field_boss() {
                self.juice.audio.play(id);
            }
        }
        if self.adv.place == Place::Dungeon {
            self.dungeon_rooms();
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
                // 입구·출구는 밟는 순간에만(0060): 싸움이 끝날 때 그 칸에 서 있었다면, 한 번 비켰다가 다시 밟아야 나간다.
                Some(Cell::Entrance)
                    if stepped
                        && self.adv.place == Place::Field
                        && self.adv.boss == BossFight::Done =>
                {
                    self.start_fade(Transition::EnterCamp)
                }
                Some(Cell::Exit) if stepped && self.adv.place == Place::Dungeon => {
                    match self.adv.node {
                        None => self.start_fade(Transition::ExitDungeon),
                        Some(at) => {
                            let route = self.adv.route.as_ref();
                            let boss = route.is_some_and(|r| {
                                matches!(r.node(at).kind, NodeKind::Boss | NodeKind::Warren)
                            });
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
                    }
                }
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
                self.juice.feel.add_trauma(0.35);
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
                        // 대장 보상이 굴 입구 곁에 떨어지면 줍다가 들어가 버린다: 입구에서 떨어진 곳으로 민다(0060).
                        let mut at = self.adv.boss_last;
                        let from = at - self.adv.entrance_pos;
                        if from.length() < t::EXIT_MIN_DIST {
                            at = self.adv.entrance_pos
                                + from.normalize_or(vec2(0.0, -1.0)) * t::EXIT_MIN_DIST;
                            at.x = at.x.clamp(inside.x, inside.right());
                            at.y = at.y.clamp(inside.y, inside.bottom());
                        }
                        self.loot
                            .push(Loot::place(LootKind::RootHeart, at + vec2(0.0, 18.0)));
                        for _ in 0..t::BOSS_SEEDS {
                            self.loot.push(Loot::spawn(LootKind::Seed, at));
                        }
                        self.adv.events.push(AdvEvent::BossBeaten);
                        self.juice.toast = Some((|s| s.toast_chief, 3.0));
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
            let from_m = self.depth_m();
            self.adv.fade = Some(Fade {
                kind,
                t: 0.0,
                swapped: false,
                from_m,
                to_m: self.next_depth_m(kind).unwrap_or(from_m),
            });
        }
    }

    /// 지금 있는 곳의 깊이(m, `depth.rs`).
    pub fn depth_m(&self) -> u32 {
        match self.adv.place {
            Place::Arena | Place::Field | Place::Village => 0,
            Place::Deep => depth::DEEPEST_M,
            Place::Camp => depth::camp(self.adv.stage),
            Place::Dungeon => match (self.adv.node, self.adv.route.as_ref()) {
                (Some((l, _)), Some(r)) => depth::node(r.stage, l, r.layers.len()),
                _ => depth::classic(self.adv.depth),
            },
        }
    }

    /// 전환이 닿을 곳의 깊이(미리 안다: 고른 길 칸, 다음 쉼터).
    fn next_depth_m(&self, kind: Transition) -> Option<u32> {
        match kind {
            Transition::EnterCamp => Some(depth::camp(self.adv.stage + 1)),
            Transition::EnterDeep => Some(depth::DEEPEST_M),
            Transition::EnterNode => {
                let r = self.adv.route.as_ref()?;
                let (l, _) = r.at?;
                Some(depth::node(r.stage, l, r.layers.len()))
            }
            _ => None,
        }
    }

    /// 화면을 덮는 검정(0..1). 렌더가 월드 위에 그린다. 내려가는 전환은 검정 대신 `descent`.
    pub fn fade_alpha(&self) -> f32 {
        match self.adv.fade {
            Some(f) if !f.kind.descends() => {
                let k = f.t / f.kind.half();
                if k < 1.0 {
                    k
                } else {
                    (2.0 - k).max(0.0)
                }
            }
            _ => 0.0,
        }
    }

    /// 내려가는 전환 중이면 지층 단면의 모습(0156).
    pub fn descent(&self) -> Option<Descent> {
        let f = self.adv.fade.filter(|f| f.kind.descends())?;
        let k = (f.t / (f.kind.half() * 2.0)).clamp(0.0, 1.0);
        // 처음 0.2·끝 0.2 동안 월드와 섞인다. 깊이는 가운데 0.15~0.85에서 떠난 곳 → 닿을 곳.
        let cover = (k / 0.2).min((1.0 - k) / 0.2).clamp(0.0, 1.0);
        let m = ((k - 0.15) / 0.7).clamp(0.0, 1.0);
        let m = m * m * (3.0 - 2.0 * m);
        Some(Descent {
            depth_m: f.from_m as f32 + (f.to_m as f32 - f.from_m as f32) * m,
            cover,
        })
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
        self.juice.toast = Some((|s| s.toast_save, 2.5));
        true
    }

    /// 개발 시나리오용: 어두워지는 연출 없이 바로 던전으로.
    pub fn enter_dungeon_now(&mut self) {
        self.enter_dungeon();
    }

    /// 개발 시나리오용(0147): 실타래 칸 굴(엘리트 끝방)에 바로. 뿌리 지대 = 가시 덤불, 심층 = 무너지는 천장.
    pub fn enter_tangle_now(&mut self, biome: Biome) {
        let prm = Params {
            rooms: 3,
            goal: dungeon::Goal::Pack,
            elite: true,
            ..Params::new(1)
        };
        self.adv.node = None;
        self.build_dungeon(&prm, biome);
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
        let Some((at, mut prm, biome, stage, kind)) = self.adv.route.as_ref().and_then(|r| {
            r.at.map(|at| (at, r.params(at), r.node(at).biome, r.stage, r.node(at).kind))
        }) else {
            return;
        };
        // 첫 길의 끝은 모모가 지킨다(아직 안 돌아왔고, 이번 런에 모모에게 쓰러지지 않았으면, 0046).
        if self.story.momo_fight && stage == 1 && kind == NodeKind::Boss {
            prm.boss = Some(Kind::Momo);
            // 복선(0159): 모모가 지키는 굴에는 방마다 시든 꽃잎이 떠돈다.
            self.juice.petals = true;
        }
        // 둘째 길의 끝은 도토가 지킨다(아직 기억해 내지 않았고, 이번 런에 도토에게 쓰러지지 않았으면, 0052).
        if self.story.doto_fight && stage == 2 && kind == NodeKind::Boss {
            prm.boss = Some(Kind::Doto);
        }
        // 셋째 길(마지막 길)의 끝은 콩이가 지킨다. 이기면 출구가 가장 깊은 곳으로(0054).
        if self.story.kongi_fight && stage == t::FINAL_STAGE && kind == NodeKind::Boss {
            prm.boss = Some(Kind::Kongi);
        }
        // 굴토끼 옛 굴의 끝은 토끼풀이 지킨다(0165). 이기면 출구가 가장 깊은 곳으로.
        if self.story.clover_fight && stage == t::FINAL_STAGE && kind == NodeKind::Warren {
            prm.boss = Some(Kind::Clover);
        }
        self.adv.node = Some(at);
        self.build_dungeon(&prm, biome);
    }

    fn build_dungeon(&mut self, prm: &Params, biome: Biome) {
        self.adv.seed = self.adv.seed.wrapping_add(0x9E37_79B9);
        let prm = Params { biome, ..*prm };
        let (d, room) = dungeon::generate(self.adv.seed, &prm);
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
        self.hazards.clouds.clear();
        self.hazards.rocks.clear();
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
        self.hazards.clouds.clear();
        self.hazards.rocks.clear();
        self.adv.dungeon = None;
        self.adv.node = None;
    }

    /// 가장 깊은 곳(0162): 굴은 버리고(맡겨 둔 쉼터는 그대로 — 실에 잠기면 거기로 돌아간다) 한 방을 짓는다.
    /// 가운데 할매, 앞쪽 반원에 매듭 셋, 플레이어는 아래 가운데. 들어서면 할매의 말(`DeepIntro`).
    fn enter_deep(&mut self) {
        self.adv.dungeon = None;
        self.adv.node = None;
        self.adv.place = Place::Deep;
        let mut room = Room::new();
        room.props.clear();
        self.room = room;
        self.enemies.clear();
        self.loot.clear();
        self.projectiles.clear();
        self.hazards.clouds.clear();
        self.hazards.rocks.clear();
        self.hazards.mines.clear();
        self.npc = None;
        self.neighbors.clear();
        self.biome = Biome::Deep;
        let center = vec2(t::VIEW_W / 2.0, t::WALL_TOP + 34.0);
        self.deep = Some(crate::world::Deep::new(center));
        self.player.pos = vec2(t::VIEW_W / 2.0, t::VIEW_H - 34.0);
        self.player.prev_pos = self.player.pos;
        self.player.vel = Vec2::ZERO;
        self.juice.toast = None;
        self.snap_cam();
        self.adv.events.push(AdvEvent::DeepIntro);
    }

    /// 길을 같은 시드로 다시 짠다(0165): 개발 시나리오가 쉼터에서 시작한 뒤 `warren_open`을 정했을 때.
    pub fn reroll_route(&mut self) {
        if let Some(r) = self.adv.route.as_ref() {
            let stage = r.stage;
            self.adv.route = Some(Route::generate(
                self.adv.seed,
                stage,
                self.story.warren_open,
            ));
        }
    }

    /// 개발: 가장 깊은 곳으로 바로(전환 없이).
    pub fn enter_deep_now(&mut self) {
        self.enter_deep();
    }

    /// 다음 쉼터: 필드·굴은 버리고 쉼터를 새로 짓는다. 체력 가득, 저장 고리가 다시 설 곳, 새 길.
    fn enter_camp(&mut self) {
        self.adv.field = None;
        self.adv.dungeon = None;
        self.adv.node = None;
        self.adv.stage += 1;
        let stage = self.adv.stage;
        // 쉼터마다 지도가 다르다(0151).
        let p = crate::camp::parse(stage);
        self.room = Room::from_grid(p.grid, p.props, p.spawn_points);
        self.enemies.clear();
        self.loot.clear();
        self.projectiles.clear();
        self.hazards.clouds.clear();
        self.hazards.rocks.clear();
        self.npc = None;
        self.neighbors.clear();
        for (ch, at) in &p.marks {
            match ch {
                'K' => self.neighbors.push(Npc::standing(Who::Popo, *at)),
                'b' => self.neighbors.push(Npc::standing(Who::Bori, *at)),
                // 수수는 첫 쉼터(꽃 핀 입구 바로 아래)에만 있다(0053).
                'z' if stage == 1 => self.neighbors.push(Npc::standing(Who::Susu, *at)),
                // 숨은 자리(0151): 반딧불 한 줌과 씨앗 둘(구르지 않게 제자리에).
                '*' => {
                    for k in 0..t::CAMP_NOOK_BUGS {
                        let a = k as f32 * std::f32::consts::TAU / t::CAMP_NOOK_BUGS as f32;
                        self.loot.push(Loot::place(
                            LootKind::Glowbug,
                            *at + Vec2::from_angle(a) * 5.0,
                        ));
                    }
                    for k in 0..t::CAMP_NOOK_SEEDS {
                        self.loot.push(Loot::place(
                            LootKind::Seed,
                            *at + vec2(-4.0 + 8.0 * k as f32, 7.0),
                        ));
                    }
                }
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
        self.adv.seed = self.adv.seed.wrapping_add(CAMP_SEED_STEP);
        self.adv.route = Some(Route::generate(
            self.adv.seed,
            stage,
            self.story.warren_open,
        ));
        self.adv.last_cell = None;
        self.adv.events.push(AdvEvent::CampArrive(stage));
        self.biome = crate::camp::biome(stage);
        self.juice.toast = Some((|s| s.toast_camp, 2.5));
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
        // 모모에게 쓰러지면 이번 런의 첫 길 끝은 보통 무리로(이기지 않고도 지나갈 수 있게, 0046).
        if self.enemies.iter().any(|e| e.kind == Kind::Momo) {
            self.story.momo_fight = false;
        }
        if self.enemies.iter().any(|e| e.kind == Kind::Doto) {
            self.story.doto_fight = false;
        }
        if self.enemies.iter().any(|e| e.kind == Kind::Kongi) {
            self.story.kongi_fight = false;
        }
        if self.enemies.iter().any(|e| e.kind == Kind::Clover) {
            self.story.clover_fight = false;
        }
        self.hazards.mines.clear();
        self.hazards.clouds.clear();
        self.hazards.rocks.clear();
        if self.adv.place == Place::Deep {
            // 실에 잠겼다(0162): 맡겨 둔 쉼터로 돌아간다. 매듭은 다음에 다시.
            self.deep = None;
            // 잠드는 장면(0163)이 남긴 어둠·침묵을 걷는다.
            self.juice.gloom.amount = 0.0;
            self.juice.hush = false;
            self.juice.boss_card = None;
            self.restore_field();
        }
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
            self.adv.route = Some(Route::generate(
                self.adv.seed,
                stage,
                self.story.warren_open,
            ));
        }
        self.player.revive();
        self.player.hp = self.player.stats.max_hp;
        self.player.pos = self.adv.save_pos;
        self.player.prev_pos = self.player.pos;
        self.inv.glowbugs = self.inv.glowbugs * 7 / 10;
        self.juice.toast = Some((|s| s.toast_respawn, 2.5));
        self.snap_cam();
    }

    /// 던전: 방이 바뀌면 들르고, 전투방이면 안에 들어선 순간 잠그고 적을 깨운다. 다 잡으면 열고 보상.
    fn dungeon_rooms(&mut self) {
        let at = dungeon::room_at(self.player.pos);
        let Some(d) = self.adv.dungeon.as_mut() else {
            return;
        };
        if at != self.adv.current_room && d.room(at).is_some() {
            self.adv.current_room = at;
        }
        let goal = d.goal;
        let named = d.boss;
        if let Some(m) = self
            .enemies
            .iter()
            .find(|e| matches!(e.kind, Kind::Momo | Kind::Doto | Kind::Kongi))
        {
            self.adv.boss_last = m.pos;
        }
        let Some(room) = d.room_mut(self.adv.current_room) else {
            return;
        };
        if !room.visited {
            room.visited = true;
            let c = room.rect().center();
            let spring = room.kind == RoomKind::Rest && goal == Goal::Spring;
            if spring {
                // 길의 샘(0159): 방 가운데에 샘이 있고, 발을 담가야 회복한다(`spring_tick`). 전엔 들어서면 바로였다.
                self.spring = Some(Spring {
                    pos: c,
                    drunk: false,
                });
            }
            if room.kind == RoomKind::Rest {
                // 산딸기는 방 가운데(샘이 있으면 샘 아래)에 놓는다(굴러가지 않으니 묶을 필요 없음).
                let dy = if spring { 24.0 } else { 8.0 };
                for k in 0..3 {
                    self.loot.push(Loot::place(
                        LootKind::Berry,
                        c + vec2(-20.0 + k as f32 * 20.0, dy),
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
            let elite = room.elite;
            let interior = room.interior();
            for (n, (kind, pos)) in room.enemies.drain(..).enumerate() {
                for off in crate::species::pack_offsets(kind) {
                    let mut e = Enemy::new(kind, pos + off);
                    e.wake_delay = t::ROOM_WAKE_DELAY;
                    if elite && n == 0 {
                        e.make_elite();
                    }
                    if kind == Kind::Momo {
                        e.level = self.story.momo_wins.saturating_add(1);
                        self.adv.boss_last = e.pos;
                        self.adv.events.push(AdvEvent::MomoIntro);
                    }
                    if kind == Kind::Doto {
                        self.adv.boss_last = e.pos;
                        self.adv.events.push(AdvEvent::DotoIntro);
                    }
                    if kind == Kind::Kongi {
                        self.adv.boss_last = e.pos;
                        self.adv.events.push(AdvEvent::KongiIntro);
                    }
                    if kind == Kind::Clover {
                        // 아기들의 고치 둘은 방 위쪽 가운데(0165). 토끼풀은 그 앞을 지킨다.
                        let c = vec2(interior.center().x, interior.y + 22.0);
                        self.hazards.cocoons = vec![c + vec2(-13.0, 0.0), c + vec2(13.0, 2.0)];
                        e.guard = Some(c);
                        self.adv.boss_last = e.pos;
                        self.adv.events.push(AdvEvent::CloverIntro);
                    }
                    self.enemies.push(e);
                }
            }
            self.juice.audio.play(sfx::Id::WallHit);
            self.juice.feel.add_trauma(0.25);
        }
        let mut gift_at = None;
        if self.room.doors_locked() && self.enemies.is_empty() {
            self.room.set_doors_locked(false);
            room.cleared = true;
            self.juice.audio.play(sfx::Id::Emerge);
            if room.kind == RoomKind::Boss {
                // 보상은 마지막으로 쓰러진 자리(플레이어 곁)에, 출구는 거기와 플레이어에서 떨어진 바닥 칸에.
                // 예전엔 출구가 방 한가운데라, 가운데서 끝내면 서 있던 칸이 곧 출구가 되어 보상을 못 줍고 나갔다(0060).
                let inside = room.interior();
                let at = vec2(
                    self.adv
                        .last_kill
                        .x
                        .clamp(inside.x + 8.0, inside.right() - 8.0),
                    self.adv
                        .last_kill
                        .y
                        .clamp(inside.y + 8.0, inside.bottom() - 8.0),
                );
                let (ex, ey) = exit_cell(
                    &self.room,
                    inside,
                    room.rect().center(),
                    &[at, self.player.pos],
                );
                if let Some(g) = self.room.grid.as_mut() {
                    g.set(ex, ey, Cell::Exit);
                }
                // 출구가 열린다(0159): 흙이 꺼지며 흙덩이가 튀고 빛 고리. 화면 밖이면 HUD 표시가 가리킨다.
                let ep = vec2(ex as f32 * 16.0 + 8.0, ey as f32 * 16.0 + 8.0);
                self.exit_at = Some(ep);
                self.exit_age = 0.0;
                self.juice
                    .fx
                    .push(ring(ep, 4.0, 24.0, 0.5, Color::new(1.0, 0.92, 0.6, 1.0)));
                clods(&mut self.juice.fx, ep, 10, DIRT, 70.0);
                puff(&mut self.juice.fx, ep, 3, self.biome.dust(), 7.0, 0.5);
                if goal == Goal::Boss {
                    self.loot.push(Loot::place(LootKind::RootHeart, at));
                    for _ in 0..t::BOSS_SEEDS {
                        self.loot.push(Loot::spawn(LootKind::Seed, at));
                    }
                    self.adv.boss_cleared = true;
                    if named == Some(Kind::Momo) {
                        // 모모는 달아난다(장면은 앱). 끝방 보상은 그대로.
                        self.adv.events.push(AdvEvent::MomoBeaten);
                    } else if named == Some(Kind::Doto) {
                        // 도토는 제정신이 돌아와 씨앗 자리를 기억해 낸다(장면은 앱).
                        self.adv.events.push(AdvEvent::DotoBeaten);
                    } else if named == Some(Kind::Kongi) {
                        // 콩이는 문을 열고 바위 발톱을 준다(장면은 앱).
                        self.adv.events.push(AdvEvent::KongiBeaten);
                    } else if named == Some(Kind::Clover) {
                        // 토끼풀은 맑은 눈으로 돌아오고 고치가 풀린다(장면은 앱, 0165).
                        self.adv.events.push(AdvEvent::CloverBeaten);
                    } else {
                        self.juice.toast = Some((|s| s.toast_boss, 3.0));
                    }
                } else {
                    for _ in 0..t::PACK_SEEDS {
                        self.loot.push(Loot::spawn(LootKind::Seed, at));
                    }
                    self.juice.toast = Some((|s| s.toast_goal, 2.5));
                    // 실타래 칸(0147): 엘리트를 풀어 주면 상자처럼 장신구 하나(다 있으면 반딧불).
                    if room.elite {
                        gift_at = Some(at);
                    }
                }
            }
        }
        if let Some(at) = gift_at {
            self.charm_gift(at);
        }
        // 보물 상자: 닿으면 연다 → 씨앗 한 줌 + 아직 없는 장신구 하나(다 있으면 반딧불).
        let ppos = self.player.pos;
        let near: Option<usize> = self.room.props.iter().position(|p| {
            matches!(p.kind, PropKind::Chest { opened: false })
                && (p.rect.center() - ppos).length() < 18.0
        });
        if let Some(i) = near {
            self.room.open_chest(i);
            self.juice.audio.play(sfx::Id::Chest);
            let at = self.room.props[i].rect.center();
            // 상자가 열리면 반짝임 셋(0081).
            for (k, d) in [vec2(-5.0, -8.0), vec2(4.0, -11.0), vec2(7.0, -5.0)]
                .into_iter()
                .enumerate()
            {
                self.juice.fx.push(crate::world::Fx {
                    pos: at + d,
                    vel: vec2(0.0, -8.0),
                    time: -0.08 * k as f32,
                    dur: 0.45,
                    kind: crate::world::FxKind::Glint {
                        color: Color::new(1.0, 0.92, 0.55, 1.0),
                    },
                });
            }
            for _ in 0..rand::gen_range(t::CHEST_SEEDS.0, t::CHEST_SEEDS.1 + 1) {
                self.push_loot(Loot::spawn(LootKind::Seed, at));
            }
            self.stats.chests += 1;
            self.charm_gift(at);
            self.juice.audio.play(sfx::Id::Emerge);
        }
    }

    /// 아직 없는 장신구 하나(다 있으면 반딧불 한 줌). 상자·실타래 끝방(0147).
    fn charm_gift(&mut self, at: Vec2) {
        let missing: Vec<usize> = self
            .inv
            .charm_pool
            .iter()
            .copied()
            .filter(|c| !self.inv.charms_found.contains(c))
            .collect();
        if missing.is_empty() {
            for _ in 0..12 {
                self.push_loot(Loot::spawn(LootKind::Glowbug, at));
            }
        } else {
            let c = missing[rand::gen_range(0, missing.len())];
            self.inv.charms_found.push(c);
            self.inv.gifts.push(Gift::Charm(c));
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
            Place::Arena | Place::Deep => Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H),
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
        let want = self.adv.cam_hold.unwrap_or(self.player.pos) - view / 2.0;
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

    /// `focus`를 가운데에 둔 카메라 왼쪽 위(장소 경계 안으로, 0159 컷씬 팬).
    pub fn cam_at(&mut self, focus: Vec2) -> Vec2 {
        let hold = self.adv.cam_hold.replace(focus);
        let c = self.cam_target();
        self.adv.cam_hold = hold;
        c
    }
}

/// 끝방 출구 칸: 방 안쪽의 막히지 않은 바닥 칸 가운데, `avoid`의 모든 점에서 `EXIT_MIN_DIST` 넘게 떨어진 칸 중
/// 방 가운데에 가장 가까운 칸. 그런 칸이 없으면(좁은 방) 가장 멀리 떨어진 칸(0060).
fn exit_cell(
    room: &crate::room::Room,
    inside: Rect,
    center: Vec2,
    avoid: &[Vec2],
) -> (usize, usize) {
    let Some(g) = room.grid.as_ref() else {
        return ((center.x / 16.0) as usize, (center.y / 16.0) as usize);
    };
    let far = |p: Vec2| {
        avoid
            .iter()
            .map(|a| (*a - p).length())
            .fold(f32::MAX, f32::min)
    };
    let mut best: Option<((usize, usize), f32)> = None;
    let mut fallback: Option<((usize, usize), f32)> = None;
    let (x0, x1) = ((inside.x / 16.0) as usize, (inside.right() / 16.0) as usize);
    let (y0, y1) = (
        (inside.y / 16.0) as usize,
        (inside.bottom() / 16.0) as usize,
    );
    for y in y0..y1.min(g.h) {
        for x in x0..x1.min(g.w) {
            let p = vec2(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0);
            if g.get(x, y) != Cell::Floor || !inside.contains(p) || room.is_solid(p, 7.0) {
                continue;
            }
            let d = far(p);
            if d >= t::EXIT_MIN_DIST {
                let c = (p - center).length();
                if best.is_none_or(|(_, b)| c < b) {
                    best = Some(((x, y), c));
                }
            } else if fallback.is_none_or(|(_, f)| d > f) {
                fallback = Some(((x, y), d));
            }
        }
    }
    best.or(fallback)
        .map(|(c, _)| c)
        .unwrap_or(((center.x / 16.0) as usize, (center.y / 16.0) as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 내려가는 전환(0156): 검정 대신 단면이 덮고, 깊이가 쉼터 → 고른 굴로 늘어난다. 새 지대에서만 이름 카드.
    #[test]
    fn descent_goes_deeper_and_names_new_zones() {
        let mut w = World::camp_start(1);
        w.adv.shown_biome = Some(Biome::Meadow);
        let camp_m = w.depth_m();
        assert_eq!(camp_m, depth::camp(1));
        let first = w.adv.route.as_ref().unwrap().choices()[0];
        assert!(w.choose_node(first));
        let mut f = w.adv.fade.expect("전환");
        assert!(f.kind.descends());
        assert_eq!(w.fade_alpha(), 0.0, "검정 덮개 대신 단면");
        assert!(f.to_m > f.from_m, "고른 굴이 더 깊다");
        w.adv.fade.as_mut().unwrap().t = f.kind.half();
        let mid = w.descent().expect("단면");
        assert!(mid.cover > 0.99 && mid.depth_m > camp_m as f32);
        w.swap_place(&mut f);
        assert_eq!(w.adv.place, Place::Dungeon);
        assert_eq!(f.to_m, w.depth_m());
        let card = w.adv.biome_card.expect("들판에서 뿌리 지대로 = 새 지대");
        assert_eq!(card.0, w.biome);
        // 같은 지대의 다음 굴로는 카드가 다시 뜨지 않는다.
        w.adv.biome_card = None;
        let next = w.adv.route.as_ref().unwrap().choices()[0];
        if w.adv.route.as_ref().unwrap().node(next).biome == w.biome {
            assert!(w.choose_node(next));
            let mut f = w.adv.fade.unwrap();
            w.swap_place(&mut f);
            assert!(w.adv.biome_card.is_none());
        }
        // 나가는 전환은 예전처럼 검정.
        w.adv.fade = None;
        w.start_fade(Transition::Respawn);
        w.adv.fade.as_mut().unwrap().t = t::FADE_HALF;
        assert!(w.descent().is_none() && w.fade_alpha() > 0.99);
    }

    /// 지름길(0151): 공터 안, 굴 입구 바로 위의 걸을 수 있는 칸에서 시작하고, 저장도 거기.
    #[test]
    fn veteran_shortcut_starts_in_glade_above_entrance() {
        let mut w = World::field();
        w.open_glade();
        w.skip_glade_boss();
        w.start_near_entrance();
        let g = w.adv.glade.expect("공터");
        assert!(g.contains(w.player.pos));
        assert_eq!(w.room.cell_at(w.player.pos), Some(Cell::Floor));
        assert!((w.player.pos - w.adv.entrance_pos).length() < 80.0);
        assert_eq!(w.adv.save_pos, w.player.pos);
    }

    /// 쉼터마다 지도가 다르고(0151), 숨은 자리의 반딧불·씨앗이 놓인다. 마지막 길 앞은 심층.
    #[test]
    fn each_camp_has_its_own_map_and_nook() {
        let mut w = World::camp_start(1);
        let first: Vec<Cell> = (0..26)
            .map(|x| w.room.grid.as_ref().unwrap().get(x, 2))
            .collect();
        let bugs = |w: &World| {
            w.loot
                .iter()
                .filter(|l| l.kind == LootKind::Glowbug)
                .count()
        };
        assert_eq!(bugs(&w), t::CAMP_NOOK_BUGS as usize);
        w = World::camp_start(2);
        let second: Vec<Cell> = (0..26)
            .map(|x| w.room.grid.as_ref().unwrap().get(x, 2))
            .collect();
        assert_ne!(first, second);
        assert_eq!(bugs(&w), t::CAMP_NOOK_BUGS as usize);
        w = World::camp_start(3);
        assert_eq!(w.biome, Biome::Deep);
    }

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
        w.inv.glowbugs = 10;
        w.player.hp = 0;
        w.respawn_at_save();
        assert_eq!(w.adv.place, Place::Field);
        assert!(w.adv.dungeon.is_none());
        assert_eq!(w.enemies.len(), field_enemies, "맡겨 둔 필드 적이 돌아온다");
        assert_eq!(w.player.hp, w.player.stats.max_hp);
        assert_eq!(w.player.pos, w.adv.save_pos);
        assert_eq!(w.inv.glowbugs, 7);
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

    /// 끝방 출구(0060): 마지막으로 쓰러진 자리·플레이어가 방 한가운데여도 출구는 거기서 떨어진 바닥 칸에 생긴다.
    #[test]
    fn boss_exit_appears_away_from_the_last_kill() {
        let mut w = World::camp_start(1);
        let r = w.adv.route.as_mut().unwrap();
        let boss = (r.layers.len() - 1, 0);
        r.at = Some((boss.0 - 1, 0));
        assert!(w.choose_node(boss));
        w.adv.fade = None;
        w.enter_node();
        let d = w.adv.dungeon.as_ref().unwrap();
        let room = d.rooms.iter().find(|r| r.kind == RoomKind::Boss).unwrap();
        let (inside, c) = (room.interior(), room.rect().center());
        for avoid in [vec![c], vec![c + vec2(-40.0, 10.0), c + vec2(30.0, 0.0)]] {
            let (x, y) = exit_cell(&w.room, inside, c, &avoid);
            let p = vec2(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0);
            assert_eq!(w.room.grid.as_ref().unwrap().get(x, y), Cell::Floor);
            assert!(inside.contains(p));
            for a in &avoid {
                assert!(
                    (*a - p).length() >= t::EXIT_MIN_DIST,
                    "{p} 가 {a} 에 너무 가깝다"
                );
            }
        }
    }

    /// 공터(0060): 대장이 굴 입구 위에서 쓰러져도 보상은 입구에서 떨어진 곳에, 입구는 서 있던 채로는 안 열린다.
    #[test]
    fn glade_loot_avoids_the_entrance_and_standing_on_it_does_not_enter() {
        let mut w = World::field();
        w.adv.boss = BossFight::Fight;
        w.enemies.retain(|e| e.kind != Kind::Chief);
        w.adv.boss_last = w.adv.entrance_pos;
        w.player.pos = w.adv.entrance_pos;
        w.field_boss();
        assert_eq!(w.adv.boss, BossFight::Done);
        let heart = w
            .loot
            .iter()
            .find(|l| l.kind == LootKind::RootHeart)
            .unwrap();
        assert!((heart.pos - w.adv.entrance_pos).length() >= t::EXIT_MIN_DIST * 0.7);
        // 싸움이 끝날 때 입구에 서 있었다: 그대로면 안 들어간다.
        w.adv.last_cell = w.room.cell_at(w.player.pos);
        assert_eq!(w.adv.last_cell, Some(Cell::Entrance));
        w.cell_triggers();
        assert!(w.adv.fade.is_none(), "서 있던 채로는 들어가지 않는다");
        // 비켰다가 다시 밟으면 들어간다.
        w.player.pos += vec2(0.0, -32.0);
        w.cell_triggers();
        w.player.pos += vec2(0.0, 32.0);
        w.cell_triggers();
        assert_eq!(w.adv.fade.map(|f| f.kind), Some(Transition::EnterCamp));
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

    /// 도토(0052): 둘째 길 끝 굴만 도토가 지키고(싸울 때만), 기억해 낸 뒤엔 마을 그루터기 밑에 있다.
    #[test]
    fn doto_guards_the_second_path_and_then_lives_in_the_village() {
        for (stage, fight, want) in [(2, true, true), (2, false, false), (1, true, false)] {
            let mut w = World::camp_start(stage);
            w.story.doto_fight = fight;
            w.enter_route_boss_now();
            let boss = w.adv.dungeon.as_ref().and_then(|d| d.boss);
            assert_eq!(boss == Some(Kind::Doto), want, "길 {stage}, 싸움 {fight}");
        }
        let v = World::village(crate::village::State {
            doto: true,
            ..Default::default()
        });
        assert!(v.neighbors.iter().any(|n| n.who == Who::Doto));
        let v = World::village(crate::village::State::default());
        assert!(!v.neighbors.iter().any(|n| n.who == Who::Doto));
    }

    /// 콩이(0054): 마지막 길 끝 굴만 콩이가 지키고(싸울 때만), 이긴 뒤엔 마을에 있다.
    #[test]
    fn kongi_guards_the_last_path_and_then_lives_in_the_village() {
        for (stage, fight, want) in [
            (t::FINAL_STAGE, true, true),
            (t::FINAL_STAGE, false, false),
            (2, true, false),
        ] {
            let mut w = World::camp_start(stage);
            w.story.kongi_fight = fight;
            w.enter_route_boss_now();
            let boss = w.adv.dungeon.as_ref().and_then(|d| d.boss);
            assert_eq!(boss == Some(Kind::Kongi), want, "길 {stage}, 싸움 {fight}");
        }
        let v = World::village(crate::village::State {
            kongi: true,
            ..Default::default()
        });
        assert!(v.neighbors.iter().any(|n| n.who == Who::Kongi));
    }

    /// 수수(0053)는 첫 쉼터에만 서 있고, 곁에서 말을 걸 수 있다.
    #[test]
    fn susu_stands_only_in_the_first_camp() {
        let w = World::camp_start(1);
        let susu = w
            .neighbors
            .iter()
            .find(|n| n.who == Who::Susu)
            .expect("수수");
        let mut w2 = World::camp_start(1);
        w2.player.pos = susu.pos + vec2(10.0, 0.0);
        assert_eq!(w2.interactable(), Some(Interact::Talk(Who::Susu)));
        let w = World::camp_start(2);
        assert!(!w.neighbors.iter().any(|n| n.who == Who::Susu));
    }

    /// 마을의 큰 나무 밑동에 서면 재주 창(0047), 떨어지면 아니다.
    #[test]
    fn village_tree_base_is_interactable() {
        let mut w = World::village(crate::village::State::default());
        let y = w.adv.marks.iter().find(|(c, _)| *c == 'Y').unwrap().1;
        w.player.pos = y + vec2(0.0, 16.0);
        assert_eq!(w.interactable(), Some(Interact::Tree));
        w.player.pos = y + vec2(0.0, 80.0);
        assert_ne!(w.interactable(), Some(Interact::Tree));
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
