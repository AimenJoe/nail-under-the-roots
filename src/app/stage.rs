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
}

impl Ware {
    fn price(self) -> u32 {
        match self {
            Ware::Berry => t::PRICE_BERRY,
            Ware::Worm => t::PRICE_WORM,
            Ware::Heart => t::PRICE_HEART,
            Ware::Charm(_) => t::PRICE_CHARM,
        }
    }

    fn name(self) -> Tr {
        match self {
            Ware::Berry => |s| s.it_berry,
            Ware::Worm => |s| s.it_worm,
            Ware::Heart => |s| s.shop_heart,
            Ware::Charm(c) => CHARMS[c].name,
        }
    }

    fn desc(self) -> Tr {
        match self {
            Ware::Berry => |s| s.it_berry_desc,
            Ware::Worm => |s| s.it_worm_desc,
            Ware::Heart => |s| s.shop_heart_desc,
            Ware::Charm(c) => CHARMS[c].desc,
        }
    }
}

/// 길 칸 종류의 이름·설명(모르는 칸은 "?").
fn node_text(kind: Option<NodeKind>) -> (Tr, Tr) {
    match kind {
        Some(NodeKind::Burrow) => (|s| s.node_burrow, |s| s.node_burrow_desc),
        Some(NodeKind::Nest) => (|s| s.node_nest, |s| s.node_nest_desc),
        Some(NodeKind::Cache) => (|s| s.node_cache, |s| s.node_cache_desc),
        Some(NodeKind::Spring) => (|s| s.node_spring, |s| s.node_spring_desc),
        Some(NodeKind::Boss) => (|s| s.node_boss, |s| s.node_boss_desc),
        None => (|s| s.node_unknown, |s| s.node_unknown_desc),
    }
}

/// 길 지도 칸의 화면 위치(논리 px). 0열 = 쉼터, 1열부터 층.
fn node_pos(r: &Route, area: Rect, layer: usize, i: usize) -> Vec2 {
    let cols = r.layers.len() as f32;
    let x = area.x + 18.0 + (layer as f32 + 1.0) * (area.w - 36.0) / cols;
    let n = r.layers[layer].len() as f32;
    let node = &r.layers[layer][i];
    let y = area.y + (i as f32 + 0.5) / n * area.h;
    vec2((x + node.jx * 6.0).round(), (y + node.jy * 5.0).round())
}

fn camp_pos(area: Rect) -> Vec2 {
    vec2(area.x + 18.0, area.y + area.h / 2.0)
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
    let big = kind == Some(NodeKind::Boss);
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
                    let (at, ent) = (self.world.adv.boss_last, self.world.adv.entrance_pos);
                    self.play(cutscene::boss_beaten(&self.world, at, ent));
                }
                AdvEvent::CampArrive(n) => self.play(cutscene::camp_arrive(&self.world, n)),
                AdvEvent::RouteOpen => self.open_route(false),
            }
        }
    }

    fn play(&mut self, c: Cutscene) {
        if self.cutscene.is_none() {
            self.cutscene = Some(c);
            self.scene = Scene::Cutscene;
        }
    }

    /// 쉼터에서 확인 버튼: 이웃에게 말 걸기·가게·빈 자리·뿌리 길.
    pub(super) fn interact(&mut self, i: Interact) {
        match i {
            Interact::Talk(Who::Popo) => {
                self.shop_cursor = 0;
                self.shop_ask = None;
                self.shop_msg = None;
                self.world.freeze();
                self.scene = Scene::Shop;
            }
            Interact::Talk(Who::Bori) => {
                let lines: [Tr; 3] = [|s| s.bori_1, |s| s.bori_2, |s| s.bori_3];
                let text = lines[self.talk_n as usize % lines.len()];
                self.talk_n += 1;
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Bori, text, Face::Normal)],
                ));
            }
            Interact::Seat => {
                self.play(cutscene::talk(
                    &self.world,
                    &[(Speaker::Nail, |s| s.seat_1, Face::Worried)],
                ));
            }
            Interact::Gate => self.open_route(true),
        }
    }

    /// 길 지도를 연다. 쉼터의 뿌리 길에서 열면 뒤로 닫을 수 있고, 굴의 출구에서 열면 골라야 한다.
    fn open_route(&mut self, can_cancel: bool) {
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
                    let rr = if node.kind == NodeKind::Boss {
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
            let (name, desc) = node_text(Some(r.node(at).kind));
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
        if !w.adv.heart_sold && w.root_hearts < t::ROOT_HEART_MAX {
            v.push(Ware::Heart);
        }
        let missing: Vec<usize> = (0..CHARMS.len())
            .filter(|c| !w.charms_found.contains(c))
            .collect();
        if !missing.is_empty() {
            v.push(Ware::Charm(missing[w.adv.stage as usize % missing.len()]));
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

    /// 지금 이 물건을 살 수 없는 이유(포포의 말). 살 수 있으면 None.
    fn cant_buy(&self, ware: Ware) -> Option<Tr> {
        let w = &self.world;
        if w.seeds < ware.price() {
            return Some(|s| s.shop_poor);
        }
        let full = match ware {
            Ware::Berry => !w.bag.has_room(Consumable::Berry),
            Ware::Worm => !w.bag.has_room(Consumable::Worm),
            Ware::Heart | Ware::Charm(_) => false,
        };
        full.then_some(|s| s.shop_full)
    }

    fn buy(&mut self, ware: Ware) {
        let w = &mut self.world;
        let price = ware.price();
        let msg: Tr = if w.seeds < price {
            |s| s.shop_poor
        } else {
            let ok = match ware {
                Ware::Berry => {
                    w.bag.has_room(Consumable::Berry) && {
                        w.bag.add(Consumable::Berry);
                        true
                    }
                }
                Ware::Worm => {
                    w.bag.has_room(Consumable::Worm) && {
                        w.bag.add(Consumable::Worm);
                        true
                    }
                }
                Ware::Heart => {
                    w.root_hearts += 1;
                    w.refresh_stats();
                    w.player.heal(w.player.stats.max_hp);
                    w.adv.heart_sold = true;
                    true
                }
                Ware::Charm(c) => {
                    w.charms_found.push(c);
                    w.gifts.push(Gift::Charm(c));
                    true
                }
            };
            if ok {
                w.seeds -= price;
                self.sfx.play(crate::sfx::Id::Glowbug);
                |s| s.shop_thanks
            } else {
                |s| s.shop_full
            }
        };
        self.shop_msg = Some((msg, 2.0));
    }

    pub(super) fn draw_shop(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(44.0, 14.0, 232.0, 150.0);
        ui.panel(panel, Some(s.shop_title));
        self.close_button(ui, panel);
        // 가진 씨앗(오른쪽 위, 닫기 단추 왼쪽)
        let seeds = fill(s.shop_seeds, &[&self.world.seeds]);
        ui.text_right(
            &seeds,
            panel.right() - 20.0,
            panel.y + 3.0,
            FONT_PX,
            ui::TEXT,
        );
        // 포포 초상화 + 인사(또는 방금 한 일)
        let face = Rect::new(panel.x + 6.0, panel.y + 22.0, 44.0, 44.0);
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
        let list_y = face.bottom() + 6.0;
        let row_h = FONT_PX + 4.0;
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
                ui.text("▶", r.x + 2.0, r.y + 2.0, FONT_PX, ui::ACCENT);
            }
            let afford = self.world.seeds >= ware.price();
            let c = match (on, afford) {
                (_, false) => ui::TEXT_DIM,
                (true, true) => ui::TEXT,
                (false, true) => Color::new(0.78, 0.72, 0.64, 1.0),
            };
            let price = fill(s.shop_price, &[&ware.price()]);
            let pw = ui.text_w(&price, FONT_PX);
            let name = ui.fit(tr(ware.name()), r.w - pw - 26.0, FONT_PX);
            ui.text(&name, r.x + 14.0, r.y + 2.0, FONT_PX, c);
            ui.text_right(&price, r.right() - 4.0, r.y + 2.0, FONT_PX, c);
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
            panel.bottom() + 2.0,
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
        let q = fill(s.shop_confirm, &[&tr(ware.name()), &ware.price()]);
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
            ui.text_center(&label, r.center().x, r.y + 2.0, FONT_PX, c);
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
    fn close_button(&mut self, ui: &Ui, panel: Rect) {
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
        let s = i18n::s();
        let verb = match i {
            Interact::Talk(Who::Popo) => s.ia_shop,
            Interact::Talk(Who::Bori) => s.ia_talk,
            Interact::Seat => s.ia_look,
            Interact::Gate => s.ia_route,
        };
        let text = fill(verb, &[&input::prompt(Act::Attack)]);
        let p = (anchor - w.adv.cam).round();
        let tw = ui.text_w(&text, FONT_PX) + 8.0;
        let r = Rect::new(
            (p.x - tw / 2.0).clamp(2.0, 318.0 - tw),
            (p.y - FONT_PX - 8.0).max(2.0),
            tw,
            FONT_PX + 4.0,
        );
        ui.rect(r, ui::PANEL);
        ui.rect_lines(r, ui::BORDER);
        ui.text_center(&text, r.center().x, r.y + 2.0, FONT_PX, ui::TEXT);
    }
}
