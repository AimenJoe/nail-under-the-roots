//! 스테이지 고리의 앱 쪽(docs/decisions/0030~0032): 모험이 알린 일(컷씬·길 지도), 쉼터에서 말 걸기,
//! 길 지도 화면, 포포의 가게 화면. 장면 상태는 app.rs의 `Scene::Route`·`Scene::Shop`.
//!
//! 둘 다 커서 없이 동작한다. 길 지도는 고를 수 있는 칸만 좌우(상하)로 돌고 확인으로 들어간다.
//! 가게는 위아래로 고르고 확인으로 산다. 탭/클릭은 보조.

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::adventure::{AdvEvent, Interact};
use crate::cutscene::{self, Cutscene, Face, Speaker};
use crate::equipment::CHARMS;
use crate::i18n::{self, fill, tr, Tr};
use crate::input::{self, Act, Actions, Verb};
use crate::items::Consumable;
use crate::route::{NodeKind, Route};
use crate::tuning as t;
use crate::ui::{self, Ui, FONT_PX};
use crate::world::{Gift, Who};

/// 가게 물건.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Ware {
    Berry,
    Worm,
    /// 뿌리 심장(최대 체력 +1). 쉼터마다 하나.
    Heart,
    /// 아직 없는 장신구 하나(equipment::CHARMS 번호).
    Charm(usize),
    /// 길에서 찾는 무기(0246, WEAPONS 번호): 이 캐릭터 것 중 아직 없는 것.
    Weapon(usize),
}

impl Ware {
    fn price(self) -> u32 {
        match self {
            Ware::Berry => t::PRICE_BERRY,
            Ware::Worm => t::PRICE_WORM,
            Ware::Heart => t::PRICE_HEART,
            Ware::Charm(_) => t::PRICE_CHARM,
            Ware::Weapon(_) => t::PRICE_WEAPON,
        }
    }

    fn name(self) -> Tr {
        match self {
            Ware::Berry => |s| s.it_berry,
            Ware::Worm => |s| s.it_worm,
            Ware::Heart => |s| s.shop_heart,
            Ware::Charm(c) => CHARMS[c].name,
            Ware::Weapon(w) => crate::equipment::WEAPONS[w].name,
        }
    }

    fn desc(self) -> Tr {
        match self {
            Ware::Berry => |s| s.it_berry_desc,
            Ware::Worm => |s| s.it_worm_desc,
            Ware::Heart => |s| s.shop_heart_desc,
            Ware::Charm(c) => CHARMS[c].desc,
            Ware::Weapon(w) => crate::equipment::WEAPONS[w].desc,
        }
    }
}

/// 길 칸 종류의 이름·설명(모르는 칸은 "?").
fn node_text(kind: Option<NodeKind>, last: bool) -> (Tr, Tr) {
    match kind {
        Some(NodeKind::Boss) if last => (|s| s.node_deepest, |s| s.node_deepest_desc),
        Some(NodeKind::Burrow) => (|s| s.node_burrow, |s| s.node_burrow_desc),
        Some(NodeKind::Nest) => (|s| s.node_nest, |s| s.node_nest_desc),
        Some(NodeKind::Cache) => (|s| s.node_cache, |s| s.node_cache_desc),
        Some(NodeKind::Spring) => (|s| s.node_spring, |s| s.node_spring_desc),
        Some(NodeKind::Elite) => (|s| s.node_elite, |s| s.node_elite_desc),
        Some(NodeKind::Boss) => (|s| s.node_boss, |s| s.node_boss_desc),
        Some(NodeKind::Warren) => (|s| s.node_warren, |s| s.node_warren_desc),
        None => (|s| s.node_unknown, |s| s.node_unknown_desc),
    }
}

/// 길 지도 칸의 화면 위치(논리 px). 0열 = 쉼터, 1열부터 층.
/// 왼쪽 위(쉼터)에서 오른쪽 아래(보스)로 비스듬히 내려간다(0156): 층의 가운데 높이가 층마다 낮아지고,
/// 같은 층의 칸은 그 둘레에 위아래로 늘어선다.
fn node_pos(r: &Route, area: Rect, layer: usize, i: usize) -> Vec2 {
    let cols = r.layers.len() as f32;
    let f = (layer as f32 + 1.0) / cols;
    let x = area.x + 18.0 + f * (area.w - 36.0);
    let n = r.layers[layer].len() as f32;
    let node = &r.layers[layer][i];
    let mid = area.y + ROUTE_TOP + f * (area.h - ROUTE_TOP - ROUTE_BOTTOM);
    // 층 전체(맨 위 칸 ~ 맨 아래 칸 + 흔들림)가 위아래 여백 안에 들어오게 층의 가운데를 옮긴다.
    // 칸마다 따로 자르면 끝 칸이 가장자리에 붙거나 두 칸이 겹쳤다(사장 피드백 2026-10-27).
    let half = (n - 1.0) / 2.0 * ROUTE_GAP + 4.0;
    let (lo, hi) = (
        area.y + ROUTE_EDGE + half,
        area.bottom() - ROUTE_EDGE - half,
    );
    let mid = if lo <= hi {
        mid.clamp(lo, hi)
    } else {
        area.center().y
    };
    let y = mid + (i as f32 - (n - 1.0) / 2.0) * ROUTE_GAP + node.jy * 4.0;
    vec2((x + node.jx * 6.0).round(), y.round())
}

/// 비스듬한 길 지도: 쉼터 높이(위에서)·보스 높이(아래에서)·같은 층 칸 사이·칸 가운데와 위아래 끝 사이 최소 여백
/// (큰 칸 반지름 8 + 테두리·고른 표시 3).
const ROUTE_TOP: f32 = 14.0;
const ROUTE_BOTTOM: f32 = 16.0;
const ROUTE_GAP: f32 = 22.0;
const ROUTE_EDGE: f32 = 13.0;

fn camp_pos(area: Rect) -> Vec2 {
    vec2(area.x + 18.0, area.y + ROUTE_TOP)
}

/// 길 지도 바탕(0156): 아래로 갈수록 짙어지는 흙 띠와 물결치는 경계. 위가 쉼터, 아래가 다음 깊이.
fn draw_route_strata(ui: &Ui, area: Rect) {
    const BANDS: usize = 6;
    let bh = area.h / BANDS as f32;
    for b in 0..BANDS {
        let k = b as f32 / (BANDS - 1) as f32;
        let c = Color::new(0.30 - 0.19 * k, 0.21 - 0.13 * k, 0.15 - 0.03 * k, 0.55);
        let y0 = area.y + b as f32 * bh;
        ui.rect(Rect::new(area.x, y0.round(), area.w, bh.ceil() + 1.0), c);
        if b > 0 {
            // 경계: 4px마다 1px 위아래로 물결.
            let line = Color::new(0.08, 0.05, 0.06, 0.45);
            let mut x = area.x;
            while x < area.right() {
                let wob = ((x * 0.21 + b as f32 * 1.7).sin() * 1.5).round();
                ui.rect(Rect::new(x, (y0 + wob).round(), 4.0, 1.0), line);
                x += 4.0;
            }
        }
    }
}

fn circle(ui: &Ui, p: Vec2, r: f32, c: Color) {
    let q = ui.px(p);
    draw_circle(q.x, q.y, r * ui.scale, c);
}

fn circle_lines(ui: &Ui, p: Vec2, r: f32, c: Color) {
    let q = ui.px(p);
    draw_circle_lines(q.x, q.y, r * ui.scale, ui.scale, c);
}

/// 뿌리처럼 점점이 이은 길.
fn dotted(ui: &Ui, a: Vec2, b: Vec2, c: Color) {
    let len = (b - a).length();
    let dir = (b - a).normalize_or_zero();
    let side = vec2(-dir.y, dir.x);
    let mut d = 7.0;
    while d < len - 7.0 {
        // 살짝 구불구불(뿌리).
        let wob = (d * 0.25 + a.x * 0.1).sin() * 1.2;
        let p = (a + dir * d + side * wob).round();
        ui.rect(Rect::new(p.x, p.y, 2.0, 1.0), c);
        d += 3.5;
    }
}

/// 칸 그림: 종류마다 다른 작은 기호(글자 없이 읽히게).
fn draw_node_icon(ui: &Ui, p: Vec2, kind: Option<NodeKind>, dim: bool) {
    let a = if dim { 0.45 } else { 1.0 };
    let col = |r: f32, g: f32, b: f32| Color::new(r, g, b, a);
    let big = matches!(kind, Some(NodeKind::Boss | NodeKind::Warren));
    let r = if big { 8.0 } else { 6.0 };
    circle(ui, p, r + 1.0, col(0.17, 0.11, 0.16));
    circle(ui, p, r, col(0.30, 0.22, 0.20));
    match kind {
        Some(NodeKind::Burrow) => {
            ui.rect(
                Rect::new(p.x - 3.0, p.y - 1.0, 6.0, 3.0),
                col(0.05, 0.03, 0.06),
            );
            ui.rect(
                Rect::new(p.x - 4.0, p.y + 2.0, 8.0, 1.0),
                col(0.55, 0.40, 0.28),
            );
        }
        Some(NodeKind::Nest) => {
            ui.rect(
                Rect::new(p.x - 3.0, p.y - 1.0, 2.0, 2.0),
                col(1.0, 0.25, 0.4),
            );
            ui.rect(
                Rect::new(p.x + 1.0, p.y - 1.0, 2.0, 2.0),
                col(1.0, 0.25, 0.4),
            );
            ui.rect(
                Rect::new(p.x - 4.0, p.y - 4.0, 2.0, 1.0),
                col(0.6, 0.35, 0.7),
            );
            ui.rect(
                Rect::new(p.x + 2.0, p.y - 4.0, 2.0, 1.0),
                col(0.6, 0.35, 0.7),
            );
        }
        Some(NodeKind::Cache) => {
            ui.rect(
                Rect::new(p.x - 3.0, p.y - 2.0, 6.0, 5.0),
                col(0.55, 0.36, 0.22),
            );
            ui.rect(
                Rect::new(p.x - 3.0, p.y - 2.0, 6.0, 1.0),
                col(0.36, 0.22, 0.14),
            );
            ui.rect(Rect::new(p.x - 1.0, p.y, 2.0, 1.0), col(0.95, 0.8, 0.35));
        }
        Some(NodeKind::Spring) => {
            circle(ui, p + vec2(0.0, 1.0), 2.5, col(0.45, 0.75, 1.0));
            ui.rect(
                Rect::new(p.x - 1.0, p.y - 3.0, 2.0, 2.0),
                col(0.45, 0.75, 1.0),
            );
            ui.rect(Rect::new(p.x - 1.0, p.y, 1.0, 1.0), col(0.9, 0.97, 1.0));
        }
        Some(NodeKind::Elite) => {
            // 실타래(0147): 엇갈려 감긴 보라 실 + 가운데 붉은 눈 하나.
            circle_lines(ui, p, 4.0, col(0.75, 0.50, 0.95));
            for (dx, dy) in [(-3.0, -1.0), (-2.0, 1.0), (1.0, -3.0), (2.0, 2.0)] {
                ui.rect(
                    Rect::new(p.x + dx, p.y + dy, 2.0, 1.0),
                    col(0.75, 0.50, 0.95),
                );
            }
            ui.rect(
                Rect::new(p.x - 1.0, p.y - 1.0, 2.0, 2.0),
                col(1.0, 0.25, 0.4),
            );
        }
        Some(NodeKind::Boss) => {
            // 뿌리 왕관 + 붉은 눈.
            for dx in [-4.0, -1.0, 2.0] {
                ui.rect(
                    Rect::new(p.x + dx, p.y - 5.0, 2.0, 3.0),
                    col(0.6, 0.35, 0.7),
                );
            }
            ui.rect(Rect::new(p.x - 3.0, p.y, 2.0, 2.0), col(1.0, 0.25, 0.4));
            ui.rect(Rect::new(p.x + 1.0, p.y, 2.0, 2.0), col(1.0, 0.25, 0.4));
        }
        Some(NodeKind::Warren) => {
            // 흰 실이 감긴 굴 구멍 + 시든 꽃잎 하나(0165).
            ui.rect(
                Rect::new(p.x - 4.0, p.y - 2.0, 8.0, 5.0),
                col(0.05, 0.03, 0.06),
            );
            for (dx, dy, w) in [
                (-5.0, -3.0, 4.0),
                (1.0, -4.0, 4.0),
                (-2.0, 3.0, 5.0),
                (3.0, 1.0, 3.0),
            ] {
                ui.rect(Rect::new(p.x + dx, p.y + dy, w, 1.0), col(0.94, 0.92, 0.99));
            }
            ui.rect(
                Rect::new(p.x - 1.0, p.y - 1.0, 2.0, 1.0),
                col(0.74, 0.56, 0.60),
            );
            ui.rect(Rect::new(p.x, p.y - 1.0, 1.0, 1.0), col(0.98, 0.70, 0.78));
        }
        None => ui.text_center("?", p.x, p.y - 6.0, FONT_PX, col(0.75, 0.70, 0.66)),
    }
}

impl App {
    /// 모험이 알린 일(게임 갱신 뒤): 보스·쉼터 컷씬, 길 지도.
    pub(super) fn take_adventure_events(&mut self) {
        let events = std::mem::take(&mut self.world.adv.events);
        for e in events {
            match e {
                AdvEvent::BossIntro => self.play(cutscene::boss_intro(&self.world)),
                AdvEvent::BossBeaten => {
                    if self.recording() {
                        self.profile.set(crate::profile::key::BORI_BACK);
                        self.save_profile();
                    }
                    let (at, ent) = (self.world.adv.boss_last, self.world.adv.entrance_pos);
                    self.play(cutscene::boss_beaten(&self.world, at, ent));
                }
                AdvEvent::CampArrive(n) => {
                    if self.current == crate::scenario::Scenario::Daily {
                        let k = crate::daily::key_best(crate::daily::today());
                        if self.profile.raise(&k, n) {
                            crate::profile::save(&self.profile);
                        }
                    }
                    if self.recording() {
                        self.profile.raise(crate::profile::key::DEEPEST, n);
                        self.save_profile();
                    }
                    // 쉼터에 닿으면 들고 온 반딧불은 마을로 간다(쓰러져 잃는 건 그 뒤에 주운 것만, 0047).
                    if self.bank_glowbugs() > 0 {
                        self.world.juice.toast = Some((|s| s.toast_camp_bank, 3.0));
                    }
                    self.checkpoint_run();
                    // 도착 한마디는 내려가는 단면이 다 흐른 뒤에(0225): 가운데서 대화가 뜨면 월드가 멈춰 단면이 중간 깊이에 섰다.
                    self.after.camp(n);
                }
                AdvEvent::RouteOpen => self.open_route(false),
                // 오늘의 굴은 싸움 없이 기록만(0162). 이야기 런은 가장 깊은 곳의 실매듭 끊기로.
                AdvEvent::Deepest => {
                    if self.current == crate::scenario::Scenario::Daily {
                        self.reach_deepest();
                    } else {
                        self.world
                            .start_fade(crate::adventure::Transition::EnterDeep);
                    }
                }
                AdvEvent::DeepIntro => self.play(cutscene::halmae_meet(&self.world)),
                AdvEvent::DeepDone => self.reach_deepest(),
                AdvEvent::PetalFound => {
                    if self.recording() {
                        self.profile.set(crate::profile::key::PETAL);
                        self.save_profile();
                    }
                }
                AdvEvent::DeepSleep => {
                    // 잠드는 장면(0163) 뒤 맡겨 둔 쉼터로.
                    self.after_cutscene = super::AfterCut::Respawn;
                    self.play(cutscene::deep_sleep(&self.world));
                }
                AdvEvent::KeeperIntro(k) => self.play(cutscene::keeper_meet(k, &self.world)),
                AdvEvent::KeeperBeaten(k) => self.keeper_beaten(k),
            }
        }
    }

    /// 길의 주인을 이겼다(bosses.rs): 이번 런의 이야기에 적고(`Story::beat`), 첫 시작이면 기록에 남기고, 장면을 튼다
    /// (모모는 세 번째에야 돌아오고 그 전엔 달아난다, 0046; 도토는 기억해 내고 0052, 콩이는 문을 열고 바위 발톱을 주고 0054,
    /// 토끼풀은 가족과 돌아온다 0165). 개발 시나리오는 기록 없이 장면만.
    fn keeper_beaten(&mut self, kind: crate::enemy::Kind) {
        let r = self.world.story.beat(kind);
        if self.recording() {
            if let Some(b) = crate::bosses::of(kind) {
                b.record(&mut self.profile, r);
            }
            self.save_profile();
        }
        self.play(cutscene::keeper_beaten(kind, &self.world, r));
    }

    /// 가장 깊은 곳(0042): 클리어를 남기고 엔딩을 튼다. 끝나면 타이틀로(런이 끝났다).
    pub(super) fn reach_deepest(&mut self) {
        use crate::profile::key;
        self.bank_glowbugs();
        // 오늘의 굴은 이야기 엔딩 없이 기록만 남기고 타이틀로.
        if self.current == crate::scenario::Scenario::Daily {
            self.profile
                .add(&crate::daily::key_clear(crate::daily::today()), 1);
            self.profile
                .add(crate::village::KEY_SEEDS, t::VILLAGE_SEEDS_DAILY);
            self.profile.add(crate::journal::stat::DAILY_CLEARS, 1);
            crate::profile::save(&self.profile);
            self.title_cursor = 0;
            self.title_note = Some((|s| s.toast_daily_clear, 5.0));
            self.scene = Scene::Title;
            return;
        }
        // 런이 끝났다: 쉼터 이어하기를 지운다(0080).
        self.clear_run();
        let ch = self.world.character;
        let momo = ch == crate::equipment::Character::Momo;
        let true_end = self.profile.flag(key::MOMO_BACK) && self.profile.flag(key::DOTO_MEMORY);
        if self.recording() {
            self.profile.add(&key::clears(ch), 1);
            // 실뜨기: 열린 가장 높은 단계로 닿았으면 다음 단계를 연다(첫 클리어가 1단계를 연다, 0043).
            // 캐릭터마다 따로 연다(0056).
            let open_key = key::knots_open(ch);
            let open = self.profile.count(&open_key);
            let level = self.world.knots.level as u32;
            if level >= open && open < crate::knots::MAX_LEVEL as u32 {
                self.profile.add(&open_key, 1);
                self.profile.set(key::KNOT_NEW);
                self.profile
                    .add(crate::village::KEY_SEEDS, t::VILLAGE_SEEDS_KNOT);
            }
            // 남은 씨앗 + 보너스가 마을 씨앗이 된다(0045).
            self.profile.add(
                crate::village::KEY_SEEDS,
                self.world.inv.seeds + t::VILLAGE_SEEDS_CLEAR,
            );
            // 모모·도토의 끝은 Naïl의 이야기 엔딩을 대신하지 않는다(0056).
            self.profile.set(if let Some(f) = ch.def().deep_flag {
                f
            } else if true_end {
                key::TRUE_ENDING
            } else {
                key::HALF_ENDING
            });
            self.save_profile();
        }
        let c = if momo {
            cutscene::ending_momo(&self.world)
        } else if ch == crate::equipment::Character::Doto {
            // 도토의 끝(0201): 씨앗 자리를 찾지만 실이 안고 있다 — 묻는 것은 나일의 참 엔딩에 남긴다.
            cutscene::ending_doto(&self.world)
        } else if true_end {
            cutscene::ending_true(&self.world)
        } else {
            cutscene::ending_half(&self.world)
        };
        self.cutscene = Some(c);
        self.scene = Scene::Cutscene;
        // 런이 끝났다: 기록하는 런이면 마을로, 아니면(개발) 타이틀로.
        self.after_cutscene = if self.recording() {
            super::AfterCut::Village
        } else {
            super::AfterCut::Title
        };
    }

    pub(super) fn play(&mut self, c: Cutscene) {
        if self.cutscene.is_none() {
            self.cutscene = Some(c);
            self.scene = Scene::Cutscene;
        }
    }

    /// 쉼터에서 확인 버튼: 이웃에게 말 걸기·가게·빈 자리·뿌리 길.
    pub(super) fn interact(&mut self, i: Interact) {
        let village = self.world.adv.place == crate::adventure::Place::Village;
        match i {
            Interact::Chest(n) => self.world.open_chest(n),
            // 마을의 포포: 마을 꾸미기 가게(0045).
            Interact::Talk(Who::Popo) if village => {
                self.decor_cursor = 0;
                self.shop_msg = None;
                self.world.freeze();
                self.scene = Scene::Decor;
            }
            Interact::Talk(Who::Popo) => {
                self.shop_cursor = 0;
                self.shop_ask = None;
                self.shop_msg = None;
                self.world.freeze();
                self.scene = Scene::Shop;
            }
            // 보리·모모: 진행에 맞는 한 번짜리 말이 있으면 그것부터(0051).
            Interact::Talk(Who::Bori) if self.say_story(crate::story::When::TalkBori) => {}
            Interact::Talk(Who::Momo) if self.say_story(crate::story::When::TalkMomo) => {}
            Interact::Talk(Who::Doto) if self.say_story(crate::story::When::TalkDoto) => {}
            Interact::Talk(Who::Bori) => {
                let lines: [Tr; 3] = [|s| s.bori_1, |s| s.bori_2, |s| s.bori_3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Bori, text, Face::Normal)],
                ));
            }
            // 빈 자리: 모모를 이긴 만큼 꽃잎이 물들어 있다(0046). 모모로 왔으면 제 자리라 1인칭으로, 나일이 있는 곳도 말한다(0067).
            Interact::Seat => {
                let ch = self.world.character;
                let momo = ch == crate::equipment::Character::Momo;
                let doto = ch == crate::equipment::Character::Doto;
                let (line, face): (Tr, Face) = match self.world.story.momo_wins() {
                    _ if momo => (|s| s.seat_momo, Face::Happy),
                    // 도토(0201): 남의 자리 곁에 도토리 하나 — 잊지 않게 묻는 버릇.
                    _ if doto => (|s| s.seat_doto, Face::Normal),
                    0 => (|s| s.seat_1, Face::Worried),
                    1 => (|s| s.seat_p1, Face::Worried),
                    2 => (|s| s.seat_p2, Face::Worried),
                    _ => (|s| s.seat_p3, Face::Worried),
                };
                // Speaker::Nail = 이번 런의 플레이어 배우(모모 런이면 모모).
                self.play(cutscene::talk(&self.world, &[(Speaker::Nail, line, face)]));
            }
            // 수수가 도토에게 호두를 다듬어 준다(0246). 받은 건 기록에 남는다(`take_gifts`).
            Interact::Talk(Who::Susu)
                if self.world.character == crate::equipment::Character::Doto
                    && !self.owns_weapon(crate::equipment::WALNUT) =>
            {
                self.play(cutscene::talk_give(
                    &self.world,
                    &[(Speaker::Susu, |s| s.susu_walnut, Face::Happy)],
                    crate::equipment::WALNUT,
                ));
            }
            // 수수: 처음엔 새싹 발톱을 만들어 준다(0053). 개발 시나리오는 기록 없이 이번 런에만.
            Interact::Talk(Who::Susu)
                if !self.profile.flag(crate::profile::key::SPROUT)
                    && !self.inventory.has(crate::equipment::SPROUT_CLAW) =>
            {
                if self.recording() {
                    self.profile.set(crate::profile::key::SPROUT);
                    self.save_profile();
                }
                self.play(cutscene::susu_meet(&self.world));
            }
            Interact::Talk(Who::Susu) => {
                let lines: [Tr; 3] = [|s| s.susu_v1, |s| s.susu_v2, |s| s.susu_v3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Susu, text, Face::Happy)],
                ));
            }
            // 콩이가 모모에게 조약돌 덧신을 만들어 준다(0246).
            Interact::Talk(Who::Kongi)
                if self.world.character == crate::equipment::Character::Momo
                    && !self.owns_weapon(crate::equipment::PEBBLE) =>
            {
                self.play(cutscene::talk_give(
                    &self.world,
                    &[(Speaker::Kongi, |s| s.kongi_pebble, Face::Happy)],
                    crate::equipment::PEBBLE,
                ));
            }
            Interact::Talk(Who::Kongi) => {
                let lines: [Tr; 3] = [|s| s.kongi_v1, |s| s.kongi_v2, |s| s.kongi_v3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Kongi, text, Face::Happy)],
                ));
            }
            Interact::Talk(Who::Clover) => {
                let lines: [Tr; 3] = [|s| s.clover_v1, |s| s.clover_v2, |s| s.clover_v3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Clover, text, Face::Happy)],
                ));
            }
            Interact::Talk(Who::Doto) => {
                let lines: [Tr; 3] = [|s| s.doto_v1, |s| s.doto_v2, |s| s.doto_v3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Doto, text, Face::Happy)],
                ));
            }
            Interact::Talk(Who::Momo) => {
                let lines: [Tr; 3] = [|s| s.momo_v1, |s| s.momo_v2, |s| s.momo_v3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Momo, text, Face::Happy)],
                ));
            }
            // 누룩 할매: 실뜨기 고르기(마을, 0043·0045).
            Interact::Talk(Who::Halmae) => {
                let open = self.knots_open();
                self.knot_level = self.knot_level.min(open);
                self.world.freeze();
                self.scene = Scene::KnotPick;
            }
            // 물웅덩이: 열린 털빛을 차례로 비춰 본다(저널 보상, 0048).
            // 실 고리: 실의 꿈 고르기(0055).
            Interact::Dream => self.open_dream(),
            Interact::Puddle => {
                let n = crate::journal::next_fur(&self.profile, self.world.inv.fur);
                let none = n == self.world.inv.fur;
                self.world.inv.fur = n;
                self.profile.set_count(crate::journal::KEY_FUR, n as u32);
                crate::profile::save(&self.profile);
                self.world.juice.toast = Some((
                    if none {
                        |s| s.puddle_none
                    } else {
                        crate::journal::fur_name(n)
                    },
                    2.0,
                ));
                self.sfx.play(crate::sfx::Id::Glowbug);
            }
            // 큰 나무: 반딧불로 재주를 깨운다(0047).
            Interact::Tree => {
                self.tree_cursor = (0, 0);
                self.tree_char = self.world.character;
                self.shop_msg = None;
                self.world.freeze();
                self.scene = Scene::Tree;
            }
            // 마을의 뿌리 길: 내려가면 런이 시작된다.
            // 고를 수 있는 캐릭터가 둘 이상이면 누구와 내려갈지 고른다(0056·0195).
            Interact::Gate if village && self.pick_list().len() > 1 => {
                self.char_cursor = self.picked_index();
                self.world.freeze();
                self.scene = Scene::CharPick;
            }
            Interact::Gate if village => self.start(crate::scenario::Scenario::Normal),
            Interact::Gate => self.open_route(true),
        }
    }

    /// 길 지도를 연다. 쉼터의 뿌리 길에서 열면 뒤로 닫을 수 있고, 굴의 출구에서 열면 골라야 한다.
    pub(super) fn open_route(&mut self, can_cancel: bool) {
        if self.world.adv.route.is_none() {
            return;
        }
        self.route_cursor = 0;
        self.route_cancel = can_cancel;
        self.world.freeze();
        self.scene = Scene::Route;
    }

    pub(super) fn update_route(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let choices = self
            .world
            .adv
            .route
            .as_ref()
            .map(|r| r.choices())
            .unwrap_or_default();
        if choices.is_empty() {
            self.scene = Scene::Game;
            return;
        }
        let n = choices.len();
        if a.up || a.left {
            self.route_cursor = (self.route_cursor + n - 1) % n;
        }
        if a.down || a.right {
            self.route_cursor = (self.route_cursor + 1) % n;
        }
        self.route_cursor = self.route_cursor.min(n - 1);
        let mut go = a.confirm;
        if let Some(Hit::RouteNode(i)) = tap_hit {
            go = self.route_cursor == i;
            self.route_cursor = i;
        }
        if go && self.world.choose_node(choices[self.route_cursor]) {
            self.sfx.play(crate::sfx::Id::Burrow);
            self.scene = Scene::Game;
        } else if (a.cancel || a.menu || tap_hit == Some(Hit::Close)) && self.route_cancel {
            self.scene = Scene::Game;
        }
    }

    pub(super) fn draw_route(&mut self, ui: &Ui) {
        let Some(r) = self.world.adv.route.as_ref() else {
            return;
        };
        let s = i18n::s();
        let panel = Rect::new(8.0, 4.0, 304.0, 160.0);
        ui.panel(panel, Some(&fill(s.route_title, &[&r.stage])));
        let can_close = self.route_cancel;
        let area = Rect::new(panel.x + 6.0, panel.y + 22.0, panel.w - 12.0, 88.0);
        let choices = r.choices();
        let sel = choices.get(self.route_cursor.min(choices.len().saturating_sub(1)));
        let root = Color::new(0.55, 0.40, 0.28, 0.55);
        let lit = Color::new(0.90, 0.71, 0.47, 1.0);
        let walked =
            |l: usize, i: usize| r.at.is_some_and(|(al, ai)| l < al || (l == al && i == ai));
        draw_route_strata(ui, area);
        // 길(뒤에서 앞으로): 쉼터 → 첫 층, 층 → 다음 층.
        for i in 0..r.layers[0].len() {
            let on = r.at.is_none() || walked(0, i);
            dotted(
                ui,
                camp_pos(area),
                node_pos(r, area, 0, i),
                if on { lit } else { root },
            );
        }
        for l in 0..r.layers.len() - 1 {
            for (i, node) in r.layers[l].iter().enumerate() {
                for &j in &node.next {
                    let from_here = r.at == Some((l, i));
                    let c = if from_here || (walked(l, i) && walked(l + 1, j)) {
                        lit
                    } else {
                        root
                    };
                    dotted(ui, node_pos(r, area, l, i), node_pos(r, area, l + 1, j), c);
                }
            }
        }
        // 쉼터(모닥불)
        let cp = camp_pos(area);
        circle(ui, cp, 6.0, Color::new(0.17, 0.11, 0.16, 1.0));
        circle(ui, cp, 3.0, Color::new(1.0, 0.6, 0.25, 1.0));
        circle(
            ui,
            cp + vec2(0.0, -1.0),
            1.5,
            Color::new(1.0, 0.9, 0.5, 1.0),
        );
        // 깊이(0156): 오른쪽 위(비어 있는 쪽)에 지금 쉼터의 깊이, 보스 위에 길이 끝나는 깊이.
        let label = |m: u32| fill(s.depth_m, &[&m]);
        ui.text_right(
            &label(crate::depth::camp(r.stage)),
            area.right() - 2.0,
            area.y + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let last = r.layers.len() - 1;
        let bp = node_pos(r, area, last, 0);
        // 두 깊이 사이 오른쪽 끝에 점선 자(아래로 내려간다).
        let (y0, y1) = (area.y + 14.0, bp.y - 25.0);
        let mut y = y0;
        while y < y1 - 3.0 {
            ui.rect(Rect::new(area.right() - 6.0, y, 1.0, 2.0), ui::TEXT_DIM);
            y += 4.0;
        }
        ui.rect(
            Rect::new(area.right() - 8.0, y1 - 3.0, 5.0, 1.0),
            ui::TEXT_DIM,
        );
        ui.rect(
            Rect::new(area.right() - 7.0, y1 - 2.0, 3.0, 1.0),
            ui::TEXT_DIM,
        );
        ui.rect(
            Rect::new(area.right() - 6.0, y1 - 1.0, 1.0, 1.0),
            ui::TEXT_DIM,
        );
        // 아래쪽은 마지막 층 칸과 가까워 짧은 꼴("48m", 긴 언어에서 칸과 겹쳤다).
        ui.text_right(
            &fill(s.depth_short, &[&crate::depth::stage_end(r.stage)]),
            area.right() - 2.0,
            bp.y - 22.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        // 칸
        let pulse = 0.5 + 0.5 * (get_time() as f32 * 5.0).sin();
        for l in 0..r.layers.len() {
            for (i, node) in r.layers[l].iter().enumerate() {
                let p = node_pos(r, area, l, i);
                let kind = r.revealed(l).then_some(node.kind);
                let choosable = choices.contains(&(l, i));
                let passed = r.at.is_some_and(|(al, _)| l <= al) && !walked(l, i);
                draw_node_icon(ui, p, kind, passed);
                if choosable {
                    let rr = if matches!(node.kind, NodeKind::Boss | NodeKind::Warren) {
                        10.0
                    } else {
                        8.0
                    };
                    let hit = Rect::new(p.x - rr, p.y - rr, rr * 2.0, rr * 2.0);
                    let k = choices.iter().position(|c| *c == (l, i)).unwrap_or(0);
                    self.hits.push((hit, Hit::RouteNode(k)));
                    if sel == Some(&(l, i)) {
                        circle_lines(ui, p, rr + 1.0 + pulse, Color::new(1.0, 0.85, 0.5, 1.0));
                    } else {
                        circle_lines(ui, p, rr, Color::new(0.90, 0.71, 0.47, 0.6));
                    }
                }
            }
        }
        // 지금 자리: Naïl 머리(작은 흙빛 동그라미 + 분홍 코).
        let here = match r.at {
            Some((l, i)) => node_pos(r, area, l, i),
            None => cp,
        } + vec2(0.0, -11.0);
        circle(ui, here, 3.5, Color::new(0.17, 0.11, 0.16, 1.0));
        circle(ui, here, 2.5, crate::world::MOLE);
        ui.rect(
            Rect::new(here.x + 1.0, here.y, 1.0, 1.0),
            Color::new(0.94, 0.59, 0.64, 1.0),
        );
        // 고른 칸 설명
        let info = Rect::new(panel.x + 6.0, area.bottom() + 4.0, panel.w - 12.0, 40.0);
        ui.rect(info, ui::PANEL_LIGHT);
        ui.rect_lines(info, ui::BORDER);
        if let Some(&at) = sel {
            let (name, desc) = node_text(Some(r.node(at).kind), r.is_final());
            ui.text(tr(name), info.x + 6.0, info.y + 3.0, FONT_PX, ui::ACCENT);
            for (k, l) in ui::wrap(ui, tr(desc), info.w - 12.0, FONT_PX)
                .iter()
                .take(2)
                .enumerate()
            {
                ui.text(
                    l,
                    info.x + 6.0,
                    info.y + 15.0 + k as f32 * 11.0,
                    FONT_PX,
                    ui::TEXT,
                );
            }
        }
        if can_close {
            self.close_button(ui, panel);
        }
        let hint = if self.route_cancel {
            super::device_hint(
                s.route_hint_back,
                s.route_hint_back_touch,
                &[Verb::Confirm, Verb::Pick, Verb::Back],
            )
        } else {
            super::device_hint(
                s.route_hint,
                s.route_hint_touch,
                &[Verb::Confirm, Verb::Pick],
            )
        };
        let hint = ui.fit(&hint, panel.w - 8.0, FONT_PX);
        ui.text_right(
            &hint,
            panel.right() - 2.0,
            panel.bottom() + 1.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    /// 가게 물건(지금 살 수 있는 것만).
    pub(super) fn wares(&self) -> Vec<Ware> {
        let w = &self.world;
        let mut v = vec![Ware::Berry, Ware::Worm];
        if !w.adv.heart_sold && w.inv.root_hearts < t::ROOT_HEART_MAX {
            v.push(Ware::Heart);
        }
        // 장신구는 이 런의 풀에서(캐릭터 전용 `Charm::only`를 거른 것, 상자·실타래와 같다). 전에는 CHARMS 전체라
        // 도토 런에서 모모 장신구를 팔았다(사장 피드백 2026-10-28).
        let missing: Vec<usize> = w
            .inv
            .charm_pool
            .iter()
            .copied()
            .filter(|c| !w.inv.charms_found.contains(c))
            .collect();
        if !missing.is_empty() {
            v.push(Ware::Charm(missing[w.adv.stage as usize % missing.len()]));
        }
        if let Some(&wp) = w.inv.road_weapons.first() {
            v.push(Ware::Weapon(wp));
        }
        v
    }

    pub(super) fn update_shop(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        if let Some((_, time)) = self.shop_msg.as_mut() {
            *time -= dt;
            if *time <= 0.0 {
                self.shop_msg = None;
            }
        }
        if self.shop_ask.is_some() {
            self.update_shop_ask(a, tap_hit);
            return;
        }
        if a.cancel || a.menu || tap_hit == Some(Hit::Close) {
            self.scene = Scene::Game;
            // 산 것까지 쉼터 이어하기에 남긴다(0080).
            self.checkpoint_run();
            return;
        }
        let wares = self.wares();
        let n = wares.len();
        if a.up {
            self.shop_cursor = (self.shop_cursor + n - 1) % n;
        }
        if a.down {
            self.shop_cursor = (self.shop_cursor + 1) % n;
        }
        self.shop_cursor = self.shop_cursor.min(n - 1);
        let mut pick = a.confirm;
        if let Some(Hit::ShopItem(i)) = tap_hit {
            pick = true;
            self.shop_cursor = i;
        }
        if pick {
            let ware = wares[self.shop_cursor];
            match self.cant_buy(ware) {
                // 살 수 없으면 묻지 않고 바로 포포가 말한다.
                Some(why) => self.shop_msg = Some((why, 2.0)),
                None => self.shop_ask = Some((self.shop_cursor, true)),
            }
        }
    }

    /// 사기 전에 묻는 창: 좌우로 사기/그만두기, 확인으로 고른다. 뒤로·바깥 탭 = 그만두기(가게는 그대로).
    fn update_shop_ask(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let Some((i, yes)) = self.shop_ask else {
            return;
        };
        if a.left || a.right {
            self.shop_ask = Some((i, !yes));
        }
        let choice = match tap_hit {
            Some(Hit::ShopYes) => Some(true),
            Some(Hit::ShopNo) | Some(Hit::Close) => Some(false),
            _ if a.cancel || a.menu => Some(false),
            _ if a.confirm => self.shop_ask.map(|(_, y)| y),
            _ => None,
        };
        if let Some(buy) = choice {
            self.shop_ask = None;
            if buy {
                if let Some(&ware) = self.wares().get(i) {
                    self.buy(ware);
                }
            }
        }
    }

    /// 물건 값: 실뜨기 배율 × 큰 나무 "흥정"(0246).
    fn ware_cost(&self, ware: Ware) -> u32 {
        let p = self.world.knots.price(ware.price());
        ((p as f32 * self.world.player.stats.bargain).round() as u32).max(1)
    }

    /// 지금 이 물건을 살 수 없는 이유(포포의 말). 살 수 있으면 None.
    fn cant_buy(&self, ware: Ware) -> Option<Tr> {
        let w = &self.world;
        if w.inv.seeds < self.ware_cost(ware) {
            return Some(|s| s.shop_poor);
        }
        let full = match ware {
            Ware::Berry => !w.inv.bag.has_room(Consumable::Berry),
            Ware::Worm => !w.inv.bag.has_room(Consumable::Worm),
            Ware::Heart | Ware::Charm(_) | Ware::Weapon(_) => false,
        };
        full.then_some(|s| s.shop_full)
    }

    fn buy(&mut self, ware: Ware) {
        let price = self.ware_cost(ware);
        let w = &mut self.world;
        let msg: Tr = if w.inv.seeds < price {
            self.sfx.play(crate::sfx::Id::Deny);
            |s| s.shop_poor
        } else {
            let ok = match ware {
                Ware::Berry => {
                    w.inv.bag.has_room(Consumable::Berry) && {
                        w.inv.bag.add(Consumable::Berry);
                        true
                    }
                }
                Ware::Worm => {
                    w.inv.bag.has_room(Consumable::Worm) && {
                        w.inv.bag.add(Consumable::Worm);
                        true
                    }
                }
                Ware::Heart => {
                    w.inv.root_hearts += 1;
                    w.refresh_stats();
                    w.player.heal(w.player.stats.max_hp);
                    w.adv.heart_sold = true;
                    true
                }
                Ware::Charm(c) => {
                    w.inv.charms_found.push(c);
                    w.inv.gifts.push(Gift::Charm(c));
                    true
                }
                Ware::Weapon(wp) => {
                    w.inv.road_weapons.retain(|&x| x != wp);
                    w.inv.gifts.push(Gift::Weapon(wp));
                    true
                }
            };
            if ok {
                w.inv.seeds -= price;
                self.sfx.play(crate::sfx::Id::Buy);
                |s| s.shop_thanks
            } else {
                self.sfx.play(crate::sfx::Id::Deny);
                |s| s.shop_full
            }
        };
        self.shop_msg = Some((msg, 2.0));
    }

    pub(super) fn draw_shop(&mut self, ui: &Ui) {
        let s = i18n::s();
        // 물건 넷 + 설명 두 줄이 창 아래로 넘쳐 HUD와 겹쳤다: 창을 위로 늘리고 줄 간격 15(0089).
        let panel = Rect::new(44.0, 6.0, 232.0, 160.0);
        ui.panel(panel, Some(s.shop_title));
        self.close_button(ui, panel);
        // 가진 씨앗(오른쪽 위, 닫기 단추 왼쪽)
        let seeds = fill(s.shop_seeds, &[&self.world.inv.seeds]);
        ui.text_right(
            &seeds,
            panel.right() - 20.0,
            panel.y + 3.0,
            FONT_PX,
            ui::TEXT,
        );
        // 포포 초상화 + 인사(또는 방금 한 일)
        // 길 무기(0246)까지 물건이 다섯이 되면 넘쳐서 초상화 칸을 44 → 36으로(인사 세 줄 = 36px).
        let face = Rect::new(panel.x + 6.0, panel.y + 22.0, 36.0, 36.0);
        ui.rect(face, ui::PANEL_LIGHT);
        let happy = self.shop_msg.is_some_and(|(m, _)| tr(m) == s.shop_thanks);
        let frame = cutscene::Portrait::POPO.frame(if happy { Face::Happy } else { Face::Normal });
        self.renderer.draw_portrait(ui, face, frame);
        ui.rect_lines(face, ui::BORDER);
        let say = self.shop_msg.map(|(m, _)| tr(m)).unwrap_or(s.shop_hi);
        let tx = face.right() + 6.0;
        for (k, l) in ui::wrap(ui, say, panel.right() - tx - 6.0, FONT_PX)
            .iter()
            .take(3)
            .enumerate()
        {
            ui.text(l, tx, face.y + 2.0 + k as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        // 물건 줄: 이름 … 값
        let wares = self.wares();
        let list_y = face.bottom() + 4.0;
        let row_h = if wares.len() > 4 {
            FONT_PX + 2.0
        } else {
            FONT_PX + 3.0
        };
        for (i, ware) in wares.iter().enumerate() {
            let r = Rect::new(
                panel.x + 6.0,
                list_y + i as f32 * row_h,
                panel.w - 12.0,
                row_h,
            );
            self.hits.push((r, Hit::ShopItem(i)));
            let on = i == self.shop_cursor;
            if on {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.text("▶", r.x + 2.0, ui::mid_y(r), FONT_PX, ui::ACCENT);
            }
            let afford = self.world.inv.seeds >= self.ware_cost(*ware);
            let c = match (on, afford) {
                (_, false) => ui::TEXT_DIM,
                (true, true) => ui::TEXT,
                (false, true) => Color::new(0.78, 0.72, 0.64, 1.0),
            };
            let price = fill(s.shop_price, &[&self.ware_cost(*ware)]);
            let pw = ui.text_w(&price, FONT_PX);
            let name = ui.fit(tr(ware.name()), r.w - pw - 26.0, FONT_PX);
            ui.text(&name, r.x + 14.0, ui::mid_y(r), FONT_PX, c);
            ui.text_right(&price, r.right() - 4.0, ui::mid_y(r), FONT_PX, c);
        }
        // 고른 물건 설명
        if let Some(ware) = wares.get(self.shop_cursor) {
            let y = list_y + wares.len() as f32 * row_h + 3.0;
            for (k, l) in ui::wrap(ui, tr(ware.desc()), panel.w - 16.0, FONT_PX)
                .iter()
                .take(2)
                .enumerate()
            {
                ui.text(l, panel.x + 8.0, y + k as f32 * 11.0, FONT_PX, ui::TEXT_DIM);
            }
        }
        let hint = super::vfill(s.shop_hint, &[Verb::Confirm, Verb::Back]);
        let hint = ui.fit(&hint, panel.w, FONT_PX);
        ui.text_right(
            &hint,
            panel.right() - 2.0,
            panel.bottom() + 1.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        self.draw_shop_ask(ui, panel);
    }

    /// 사기 전에 묻는 창(가게 패널 가운데). 열려 있으면 탭은 이 창만 받는다.
    fn draw_shop_ask(&mut self, ui: &Ui, panel: Rect) {
        let Some((i, yes)) = self.shop_ask else {
            return;
        };
        let Some(&ware) = self.wares().get(i) else {
            return;
        };
        let s = i18n::s();
        ui.rect(panel, ui::DIM_OVERLAY);
        let b = Rect::new(panel.x + 26.0, panel.y + 44.0, panel.w - 52.0, 62.0);
        ui.panel(b, None);
        let q = fill(s.shop_confirm, &[&tr(ware.name()), &self.ware_cost(ware)]);
        for (k, l) in ui::wrap(ui, &q, b.w - 12.0, FONT_PX)
            .iter()
            .take(2)
            .enumerate()
        {
            ui.text_center(
                l,
                b.center().x,
                b.y + 6.0 + k as f32 * 12.0,
                FONT_PX,
                ui::TEXT,
            );
        }
        // 이 창이 열려 있는 동안 다른 칸(물건·✕)은 탭을 받지 않는다.
        self.hits.clear();
        let bw = (b.w - 18.0) / 2.0;
        for (k, (label, hit, on)) in [
            (s.shop_yes, Hit::ShopYes, yes),
            (s.shop_no, Hit::ShopNo, !yes),
        ]
        .into_iter()
        .enumerate()
        {
            let r = Rect::new(
                b.x + 6.0 + k as f32 * (bw + 6.0),
                b.bottom() - 22.0,
                bw,
                16.0,
            );
            ui.rect(r, if on { ui::PANEL_LIGHT } else { ui::PANEL });
            ui.rect_lines(r, if on { ui::ACCENT } else { ui::BORDER });
            let label = ui.fit(label, r.w - 4.0, FONT_PX);
            let c = if on { ui::TEXT } else { ui::TEXT_DIM };
            ui.text_center(&label, r.center().x, ui::mid_y(r), FONT_PX, c);
            // 누르는 칸은 위아래로 조금 넉넉히.
            self.hits
                .push((Rect::new(r.x, r.y - 4.0, r.w, r.h + 8.0), hit));
        }
        self.hits
            .push((Rect::new(-2000.0, -2000.0, 4000.0, 4000.0), Hit::Close));
    }

    /// 닫기 단추(패널 오른쪽 위 ✕)와 패널 바깥 탭 = 닫기. 터치에는 뒤로 버튼이 없어서(사람 피드백 2026-10-25).
    /// 그림은 작게(10px), 누르는 칸은 크게(24px = 가로 폰에서 48pt 이상). 패널 안 빈 곳 탭은 닫지 않는다(실수 방지).
    /// 탭 판정은 먼저 넣은 것이 이기므로, 항목 칸을 넣은 뒤에 부른다.
    pub(super) fn close_button(&mut self, ui: &Ui, panel: Rect) {
        let c = vec2(panel.right() - 9.0, panel.y + 9.0);
        let x = Color::new(0.85, 0.78, 0.70, 1.0);
        ui.rect(Rect::new(c.x - 6.0, c.y - 6.0, 12.0, 12.0), ui::PANEL_LIGHT);
        ui.rect_lines(Rect::new(c.x - 6.0, c.y - 6.0, 12.0, 12.0), ui::BORDER);
        ui.line(c + vec2(-3.0, -3.0), c + vec2(3.0, 3.0), x);
        ui.line(c + vec2(-3.0, 3.0), c + vec2(3.0, -3.0), x);
        self.hits
            .push((Rect::new(c.x - 12.0, c.y - 12.0, 24.0, 24.0), Hit::Close));
        // 패널 바깥 네 쪽(화면 밖까지 넉넉히).
        let big = 2000.0;
        for r in [
            Rect::new(-big, -big, big * 3.0, big + panel.y),
            Rect::new(-big, panel.bottom(), big * 3.0, big),
            Rect::new(-big, panel.y, big + panel.x, panel.h),
            Rect::new(panel.right(), panel.y, big, panel.h),
        ] {
            self.hits.push((r, Hit::Close));
        }
    }

    /// 쉼터: 말 걸 수 있는 것 위에 "버튼 + 할 일" 말풍선.
    pub(super) fn draw_interact_bubble(&self, ui: &Ui) {
        let w = &self.world;
        let (Some(i), Some(anchor)) = (w.interactable(), w.interact_anchor()) else {
            return;
        };
        // 알림(토스트)이 떠 있는 동안은 숨긴다: 방금 한 일의 대답이 알림이고, 둘이 위쪽에서 겹쳐 읽히지 않았다.
        if w.juice.toast.is_some() {
            return;
        }
        let s = i18n::s();
        let village = w.adv.place == crate::adventure::Place::Village;
        let verb = match i {
            Interact::Talk(Who::Popo) if village => s.ia_decor,
            Interact::Talk(Who::Popo) => s.ia_shop,
            Interact::Talk(
                Who::Bori | Who::Momo | Who::Doto | Who::Susu | Who::Kongi | Who::Clover,
            ) => s.ia_talk,
            Interact::Talk(Who::Halmae) => s.ia_knots,
            Interact::Seat => s.ia_look,
            Interact::Tree => s.ia_tree,
            Interact::Puddle => s.ia_puddle,
            Interact::Dream => s.ia_dream,
            Interact::Gate if village => s.ia_descend,
            Interact::Gate => s.ia_route,
            Interact::Chest(_) => s.ia_open,
        };
        let text = fill(verb, &[&crate::ui::key(input::prompt(Act::Attack))]);
        let p = (anchor - w.adv.cam).round();
        let tw = ui.text_w(&text, FONT_PX) + 8.0;
        let mut r = Rect::new(
            (p.x - tw / 2.0).clamp(2.0, 318.0 - tw),
            (p.y - FONT_PX - 8.0).max(2.0),
            tw,
            FONT_PX + 4.0,
        );
        // 마을 오른쪽 위의 숫자 두 줄(y 14~39)과 겹치면 말풍선을 그 왼쪽으로 민다(할매·실 고리 곁, 0089).
        if village && r.y < 40.0 {
            let seeds = fill(
                s.village_seeds,
                &[&self.profile.count(crate::village::KEY_SEEDS)],
            );
            let light = fill(
                s.tree_light,
                &[&self.profile.count(crate::talents::KEY_LIGHT)],
            );
            let hud_x = 316.0
                - ui.text_w(&seeds, FONT_PX)
                    .max(ui.text_w(&light, FONT_PX) + 6.0)
                - 4.0;
            if r.right() > hud_x {
                r.x = (hud_x - tw).max(2.0);
            }
        }
        ui.rect(r, ui::PANEL);
        ui.rect_lines(r, ui::BORDER);
        ui.text_center(&text, r.center().x, r.y + 2.0, FONT_PX, ui::TEXT);
    }

    /// 마을 꾸미기 가게(마을의 포포, 0045): 위아래로 고르고 확인으로 놓는다. 닫기: 뒤로·✕·바깥.
    pub(super) fn update_decor(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        use crate::village::{key_decor, DECOR, KEY_SEEDS};
        if let Some((_, t)) = self.shop_msg.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.shop_msg = None;
            }
        }
        let n = DECOR.len();
        if a.up {
            self.decor_cursor = (self.decor_cursor + n - 1) % n;
        }
        if a.down {
            self.decor_cursor = (self.decor_cursor + 1) % n;
        }
        let chosen = if a.confirm {
            Some(self.decor_cursor)
        } else if let Some(Hit::ShopItem(i)) = tap_hit {
            self.decor_cursor = i;
            Some(i)
        } else {
            None
        };
        if let Some(i) = chosen {
            let owned = self.profile.flag(&key_decor(i));
            let have = self.profile.count(KEY_SEEDS);
            if owned {
                // 이미 놓았다: 아무 일도 없다.
            } else if have < DECOR[i].price {
                self.shop_msg = Some((|s| s.decor_poor, 3.0));
                self.sfx.play(crate::sfx::Id::Deny);
            } else {
                self.profile.set_count(KEY_SEEDS, have - DECOR[i].price);
                self.profile.set(&key_decor(i));
                crate::profile::save(&self.profile);
                if let Some(v) = self.world.adv.village.as_mut() {
                    v.decor[i] = true;
                }
                self.shop_msg = Some((|s| s.decor_thanks, 2.5));
                self.sfx.play(crate::sfx::Id::Buy);
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.shop_msg = None;
            self.scene = Scene::Game;
        }
    }

    pub(super) fn draw_decor(&mut self, ui: &Ui) {
        use crate::village::{key_decor, DECOR, KEY_SEEDS};
        let s = i18n::s();
        // 꾸미기 여섯 줄이 창 아래로 넘쳐 조작 안내와 겹쳤다: 창을 위로 늘림(0089).
        let panel = Rect::new(44.0, 6.0, 232.0, 160.0);
        ui.panel(panel, Some(s.decor_title));
        self.close_button(ui, panel);
        let have = self.profile.count(KEY_SEEDS);
        let seeds = fill(s.village_seeds, &[&have]);
        ui.text_right(
            &seeds,
            panel.right() - 20.0,
            panel.y + 3.0,
            FONT_PX,
            ui::TEXT,
        );
        // 길 무기(0246)까지 물건이 다섯이 되면 넘쳐서 초상화 칸을 44 → 36으로(인사 세 줄 = 36px).
        let face = Rect::new(panel.x + 6.0, panel.y + 22.0, 36.0, 36.0);
        ui.rect(face, ui::PANEL_LIGHT);
        let happy = self.shop_msg.is_some_and(|(m, _)| tr(m) == s.decor_thanks);
        let frame = cutscene::Portrait::POPO.frame(if happy { Face::Happy } else { Face::Normal });
        self.renderer.draw_portrait(ui, face, frame);
        ui.rect_lines(face, ui::BORDER);
        let say = self.shop_msg.map(|(m, _)| tr(m)).unwrap_or(s.decor_hi);
        let tx = face.right() + 6.0;
        for (k, l) in ui::wrap(ui, say, panel.right() - tx - 6.0, FONT_PX)
            .iter()
            .take(3)
            .enumerate()
        {
            ui.text(l, tx, face.y + 2.0 + k as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        let list_y = face.bottom() + 6.0;
        let row_h = FONT_PX + 2.0;
        for (i, d) in DECOR.iter().enumerate() {
            let r = Rect::new(
                panel.x + 6.0,
                list_y + i as f32 * row_h,
                panel.w - 12.0,
                row_h,
            );
            self.hits.push((r, Hit::ShopItem(i)));
            let on = i == self.decor_cursor;
            if on {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.text("▶", r.x + 2.0, ui::mid_y(r), FONT_PX, ui::ACCENT);
            }
            let owned = self.profile.flag(&key_decor(i));
            let right = if owned {
                s.decor_owned.to_string()
            } else {
                fill(s.shop_price, &[&d.price])
            };
            let c = if owned || have < d.price {
                ui::TEXT_DIM
            } else if on {
                ui::TEXT
            } else {
                Color::new(0.78, 0.72, 0.64, 1.0)
            };
            let pw = ui.text_w(&right, FONT_PX);
            let name = ui.fit(tr(d.name), r.w - pw - 26.0, FONT_PX);
            ui.text(&name, r.x + 14.0, ui::mid_y(r), FONT_PX, c);
            ui.text_right(&right, r.right() - 4.0, ui::mid_y(r), FONT_PX, c);
        }
        let hint = super::vfill(s.decor_hint, &[Verb::Confirm, Verb::Back]);
        let hint = ui.fit(&hint, panel.w - 8.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() + 1.0, FONT_PX, ui::TEXT_DIM);
    }

    /// 그만둘 때(다시 하기·타이틀로) 들고 있는 반딧불을 마을로 보내도 되나(0210): 굴·가장 깊은 곳 안이면 아니다.
    /// 쓰러지면 잃는 반딧불을, 쓰러지기 직전에 그만둬서 지키지 못하게. 도움 모드는 늘 보낸다.
    pub(super) fn safe_to_bank(&self) -> bool {
        use crate::adventure::Place;
        self.world.assist || !matches!(self.world.adv.place, Place::Dungeon | Place::Deep)
    }

    /// 들고 있는 반딧불을 마을(큰 나무)로 보낸다: 쉼터에 닿을 때·런이 끝날 때(0047). 첫 시작만. 보낸 수.
    /// 실뜨기 단계만큼 더 보낸다(0247).
    pub(super) fn bank_glowbugs(&mut self) -> u32 {
        let n = self.world.inv.glowbugs;
        if !self.recording() || n == 0 {
            return 0;
        }
        self.world.inv.glowbugs = 0;
        let n = self.world.knots.glow(n);
        self.profile.add(crate::talents::KEY_LIGHT, n);
        self.profile.add(crate::journal::stat::LIGHT, n);
        crate::profile::save(&self.profile);
        n
    }

    /// 런 통계를 기록에 쌓고(첫 시작만), 받을 수 있는 저널 목표가 새로 생기면 알린다(0048). 저장은 다른 저장 때 같이.
    pub(super) fn sync_journal(&mut self, dt: f32) {
        let now = super::Tally::of(&self.world);
        let mut changed = false;
        if self.recording() && now != self.tally {
            for (i, k) in super::Tally::KEYS.iter().enumerate() {
                let d = now.0[i].saturating_sub(self.tally.0[i]);
                if d > 0 {
                    self.profile.add(k, d);
                    changed = true;
                }
            }
        }
        self.tally = now;
        // 깃발(이야기)로 이루는 목표도 있으니 가끔은 그냥 센다.
        self.journal_check -= dt;
        if changed || self.journal_check <= 0.0 {
            self.journal_check = 1.0;
            let ready = crate::journal::ready_count(&self.profile);
            if ready > self.journal_ready {
                self.world.juice.toast = Some((|s| s.j_toast, 3.0));
                self.sfx.play(crate::sfx::Id::Heal);
            }
            self.journal_ready = ready;
        }
    }

    /// 저널 탭에서 고른 목표의 보상을 받는다. 월드에 바로 보이는 것(꾸미기·털빛·장신구 풀)도 고친다.
    pub(super) fn claim_journal(&mut self) {
        use crate::journal::{claim, visible, Reward};
        let Some(g) = visible(&self.profile).get(self.journal_cursor).copied() else {
            return;
        };
        let Some(r) = claim(&mut self.profile, g) else {
            return;
        };
        crate::profile::save(&self.profile);
        match r {
            Reward::Decor(i) => {
                if let Some(v) = self.world.adv.village.as_mut() {
                    v.decor[i] = true;
                }
            }
            Reward::Fur(_) => self.world.inv.fur = crate::journal::fur(&self.profile),
            // 장신구는 바로 가방에(0050). 이번 런의 상자에서는 다시 안 나온다.
            Reward::Charm(c) => {
                self.world.inv.charm_pool =
                    crate::journal::charm_pool(&self.profile, self.world.character);
                self.inventory.add_charm(c);
                if !self.world.inv.charms_found.contains(&c) {
                    self.world.inv.charms_found.push(c);
                }
            }
            // 무기는 기록 깃발이 섰으니(claim) 이번 런 가방에도(주인 것만 받는다).
            Reward::Weapon(w) => {
                self.inventory.add(w);
            }
            Reward::Light(_) | Reward::Seeds(_) => {}
        }
        self.journal_ready = crate::journal::ready_count(&self.profile);
        self.sfx.play(crate::sfx::Id::Heal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 길 지도(0156): 어떤 시드·길이든 칸이 위아래 끝에 붙지 않고, 같은 층 칸끼리 겹치지 않는다.
    #[test]
    fn route_nodes_keep_off_the_edges() {
        let area = Rect::new(14.0, 26.0, 292.0, 88.0);
        for seed in 0..200u64 {
            for stage in 1..=3 {
                let r = Route::generate(seed, stage, seed % 2 == 0);
                for (l, layer) in r.layers.iter().enumerate() {
                    let ys: Vec<f32> = (0..layer.len())
                        .map(|i| node_pos(&r, area, l, i).y)
                        .collect();
                    for y in &ys {
                        assert!(*y - 8.0 >= area.y + 4.0, "위 {seed}/{stage}/{l}: {y}");
                        assert!(
                            *y + 8.0 <= area.bottom() - 4.0,
                            "아래 {seed}/{stage}/{l}: {y}"
                        );
                    }
                    for w in ys.windows(2) {
                        assert!(w[1] - w[0] >= 14.0, "겹침 {seed}/{stage}/{l}: {ys:?}");
                    }
                }
            }
        }
    }
}
