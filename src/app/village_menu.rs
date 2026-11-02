//! 마을 창(0269 R35, 요구 2.5): 마을의 포포에게 말을 걸면 연다. 탭 표 `VILLAGE_TABS`(꾸미기 · 짐 꾸리기) — 새 탭 = 표 한 줄 +
//! 함수 둘(일시정지 `PauseTab`과 같은 꼴, 0248 R30). 데이터·저장은 합치지 않았다: 꾸미기는 깃발 `decor.<i>`, 짐은 수
//! `pack.*`, 화폐는 둘 다 마을 씨앗 — 사면 바로 저장, 짐은 새 런 시작 때 `App::unpack`이 꺼낸다(0259 그대로).
//!
//! 화면: 위 탭 + 마을 씨앗, 그 아래 왼쪽 위 포포의 초상화와 말(고른 것의 설명, 산 직후엔 한마디 — 쉼터 가게와 같은 자리),
//! 아래 왼쪽 목록(작은 그림 · 이름 · 상태), 오른쪽 큰 그림 · 이름 · 값·수.
//! 사기: 키·패드는 확인 = 산다(지금처럼 빠르게). 터치·마우스는 첫 누름 = 고르기, 고른 줄을 다시 누름 = 사기(실수로 사지 않게,
//! 확인 창을 따로 띄우지 않는다). 짐은 좌우·−+ 단추로 빼고 넣는다(뺀 것은 씨앗을 돌려준다).

use super::*;
use crate::cutscene::Face;
use crate::render::VillagePic;
use crate::village::{
    decor_name, key_decor, key_pack, PackKind, DECOR, JOURNAL_DECOR_DESC, JOURNAL_SHOWN, KEY_SEEDS,
    PACK, PACK_POUCH_SEEDS,
};

/// 마을 창의 탭 하나: 이름, 좌우를 탭 안에서 쓰나(짐: 빼기·넣기 — 그때 탭은 Q/E·누르기로), 입력·그림,
/// 포포가 위 말 칸에서 하는 말(고른 줄의 설명 — 산 직후 같은 한마디가 있으면 그게 먼저).
pub(super) struct VillageTab {
    pub name: Tr,
    pub lr: bool,
    pub update: fn(&mut App, &Actions, Option<Hit>),
    pub draw: fn(&mut App, &Ui, Rect),
    pub say: fn(&App) -> String,
}

pub(super) const VILLAGE_TABS: &[VillageTab] = &[
    VillageTab {
        name: |s| s.decor_tab,
        lr: false,
        update: |app, a, tap| app.update_decor_tab(a, tap),
        draw: |app, ui, r| app.draw_decor_tab(ui, r),
        say: |app| app.decor_say(),
    },
    VillageTab {
        name: |s| s.pack_tab,
        lr: true,
        update: |app, a, tap| app.update_pack_tab(a, tap),
        draw: |app, ui, r| app.draw_pack_tab(ui, r),
        say: |app| tr(PACK[app.decor_cursor.min(PACK.len() - 1)].desc).to_string(),
    },
];

/// 꾸미기 탭의 줄: 가게 꾸미기 여섯 + 저널 꾸미기(지운 6번 병은 빼고).
fn decor_rows() -> Vec<usize> {
    (0..DECOR.len()).chain(JOURNAL_SHOWN).collect()
}

fn decor_desc(i: usize) -> Tr {
    match i.checked_sub(DECOR.len()) {
        Some(j) => JOURNAL_DECOR_DESC[j.min(JOURNAL_DECOR_DESC.len() - 1)],
        None => DECOR[i].desc,
    }
}

/// 자세히 칸(44px)에 그릴 배율: 꾸미기는 1배(팩 그림 꽃밭이 36px쯤), 짐(산딸기·지렁이·씨앗)은 작아서 2배. 목록 칸은 이 반.
fn pic_zoom(p: VillagePic) -> f32 {
    match p {
        VillagePic::Decor(_) => 1.0,
        VillagePic::Pack(_) => 2.0,
    }
}

const PANEL: Rect = Rect {
    x: 10.0,
    y: 4.0,
    w: 300.0,
    h: 166.0,
};
const ROW_H: f32 = FONT_PX + 6.0;
/// 왼쪽 목록 폭.
const LIST_W: f32 = 150.0;

impl App {
    pub(super) fn open_village_menu(&mut self) {
        self.decor_cursor = 0;
        self.decor_tab = 0;
        // 처음 열면 포포의 인사(자세히 칸의 말 자리, 몇 초 뒤 설명으로).
        self.shop_msg = Some((|s| s.decor_hi, 3.5));
        self.village_view = Default::default();
        self.world.freeze();
        self.scene = Scene::Decor;
    }

    pub(super) fn update_village_menu(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        if let Some((_, t)) = self.shop_msg.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.shop_msg = None;
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.shop_msg = None;
            self.scene = Scene::Game;
            return;
        }
        let n = VILLAGE_TABS.len();
        let cur = self.decor_tab.min(n - 1);
        let lr = VILLAGE_TABS[cur].lr;
        let mut tab = cur;
        if a.tab_prev || (!lr && a.left) {
            tab = (tab + n - 1) % n;
        }
        if a.tab_next || (!lr && a.right) {
            tab = (tab + 1) % n;
        }
        if let Some(Hit::ShopTab(i)) = tap_hit {
            tab = i.min(n - 1);
        }
        if tab != cur {
            self.decor_tab = tab;
            self.decor_cursor = 0;
            self.village_view = Default::default();
            self.shop_msg = None;
            self.sfx.play(crate::sfx::Id::UiMove);
            return;
        }
        (VILLAGE_TABS[cur].update)(self, a, tap_hit);
    }

    /// 위아래·끌기로 줄을 옮기고, 누른 줄을 돌려준다: (고른 줄이 바뀌었나, 이미 고른 줄을 다시 눌렀나).
    fn village_pick(&mut self, a: &Actions, tap_hit: Option<Hit>, n: usize) -> (bool, bool) {
        let was = self.decor_cursor;
        if a.up {
            self.decor_cursor = (self.decor_cursor + n - 1) % n;
        }
        if a.down {
            self.decor_cursor = (self.decor_cursor + 1) % n;
        }
        self.village_view
            .scroll(self.scroll_in, &mut self.decor_cursor, n);
        let mut again = false;
        if let Some(Hit::ShopItem(i)) = tap_hit {
            again = i == self.decor_cursor;
            self.decor_cursor = i.min(n - 1);
        }
        self.decor_cursor = self.decor_cursor.min(n - 1);
        let moved = self.decor_cursor != was;
        if moved {
            self.sfx.play(crate::sfx::Id::UiMove);
        }
        (moved, again)
    }

    fn update_decor_tab(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let rows = decor_rows();
        let (moved, again) = self.village_pick(a, tap_hit, rows.len());
        let i = rows[self.decor_cursor];
        if a.confirm || again {
            self.place_decor(i);
        } else if moved && tap_hit.is_some() && self.decor_can_buy(i) {
            // 터치로 처음 고른 줄: 한 번 더 누르면 산다고 알려 준다.
            self.shop_msg = Some((|s| s.tap_again, 2.5));
        }
    }

    fn decor_can_buy(&self, i: usize) -> bool {
        i < DECOR.len()
            && !self.profile.flag(&key_decor(i))
            && self.profile.count(KEY_SEEDS) >= DECOR[i].price
    }

    fn place_decor(&mut self, i: usize) {
        let have = self.profile.count(KEY_SEEDS);
        if self.profile.flag(&key_decor(i)) {
            // 이미 놓았다: 아무 일도 없다.
        } else if i >= DECOR.len() {
            // 저널 꾸미기는 사지 않는다 — 발자국 목표의 보상.
            self.shop_msg = Some((|s| s.decor_journal_d, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
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

    fn update_pack_tab(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let (moved, again) = self.village_pick(a, tap_hit, PACK.len());
        let i = self.decor_cursor;
        let step = match tap_hit {
            Some(Hit::PackStep(k, d)) => {
                self.decor_cursor = k.min(PACK.len() - 1);
                d as i32
            }
            _ if a.right || a.confirm || again => 1,
            _ if a.left => -1,
            _ => 0,
        };
        match step {
            1 => self.pack_put(self.decor_cursor),
            -1 => self.pack_take(self.decor_cursor),
            _ if moved && tap_hit.is_some() && i < PACK.len() => {
                self.shop_msg = Some((|s| s.tap_again, 2.5));
            }
            _ => {}
        }
    }

    fn pack_put(&mut self, i: usize) {
        let p = &PACK[i];
        let have = self.profile.count(KEY_SEEDS);
        let packed = self.profile.count(key_pack(p.kind));
        if packed >= p.max {
            self.shop_msg = Some((|s| s.pack_full, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else if have < p.price {
            self.shop_msg = Some((|s| s.decor_poor, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else {
            self.profile.set_count(KEY_SEEDS, have - p.price);
            self.profile.set_count(key_pack(p.kind), packed + 1);
            crate::profile::save(&self.profile);
            self.shop_msg = Some((|s| s.pack_thanks, 2.5));
            self.sfx.play(crate::sfx::Id::Buy);
        }
    }

    /// 꾸려 둔 것 하나를 다시 꺼낸다: 씨앗을 그대로 돌려준다(아직 런에 들고 가지 않았으니, 0269).
    fn pack_take(&mut self, i: usize) {
        let p = &PACK[i];
        let packed = self.profile.count(key_pack(p.kind));
        if packed == 0 {
            self.sfx.play(crate::sfx::Id::Deny);
            return;
        }
        self.profile.set_count(key_pack(p.kind), packed - 1);
        self.profile.add(KEY_SEEDS, p.price);
        crate::profile::save(&self.profile);
        self.shop_msg = Some((|s| s.pack_back, 2.5));
        self.sfx.play(crate::sfx::Id::UiMove);
    }

    pub(super) fn draw_village_menu(&mut self, ui: &Ui) {
        let s = i18n::s();
        ui.panel(PANEL, Some(""));
        self.close_button(ui, PANEL);
        // 머리줄: 탭 + 마을 씨앗.
        let have = self.profile.count(KEY_SEEDS);
        let seeds = fill(s.village_seeds, &[&have]);
        let seeds_w = ui.text_w(&seeds, FONT_PX);
        ui.text_right(
            &seeds,
            PANEL.right() - 20.0,
            PANEL.y + 3.0,
            FONT_PX,
            ui::TEXT,
        );
        let n = VILLAGE_TABS.len();
        let tab_w = ((PANEL.w - 30.0 - seeds_w) / n as f32).clamp(30.0, 90.0);
        for (i, t) in VILLAGE_TABS.iter().enumerate() {
            let r = Rect::new(
                PANEL.x + 3.0 + i as f32 * tab_w,
                PANEL.y + 1.0,
                tab_w - 2.0,
                FONT_PX + 4.0,
            );
            self.hits.push((r, Hit::ShopTab(i)));
            let on = i == self.decor_tab;
            if on {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.rect_lines(r, ui::ACCENT);
            }
            let label = ui.fit(tr(t.name), r.w - 4.0, FONT_PX);
            ui.text_center(
                &label,
                r.center().x,
                PANEL.y + 3.0,
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT_DIM },
            );
        }
        // 왼쪽 위 포포 + 위쪽 말 칸(사장 2026-11-02: 초상화는 왼쪽 위, 대화는 위). 산 뒤에는 웃는다.
        let face = Rect::new(PANEL.x + 6.0, PANEL.y + 21.0, 36.0, 36.0);
        ui.rect(face, ui::PANEL_LIGHT);
        let happy = self
            .shop_msg
            .is_some_and(|(m, _)| tr(m) == s.decor_thanks || tr(m) == s.pack_thanks);
        let frame = cutscene::Portrait::POPO.frame(if happy { Face::Happy } else { Face::Normal });
        self.renderer.draw_portrait(ui, face, frame);
        ui.rect_lines(face, ui::BORDER);
        let say = match self.shop_msg {
            Some((m, _)) => tr(m).to_string(),
            None => (VILLAGE_TABS[self.decor_tab.min(n - 1)].say)(self),
        };
        let tx = face.right() + 6.0;
        for (k, l) in ui::wrap(ui, &say, PANEL.right() - tx - 6.0, FONT_PX)
            .iter()
            .take(3)
            .enumerate()
        {
            ui.text(l, tx, face.y + 1.0 + k as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        let top = face.bottom() + 5.0;
        let body = Rect::new(
            PANEL.x + 6.0,
            top,
            PANEL.w - 12.0,
            PANEL.bottom() - 4.0 - top,
        );
        let tab = self.decor_tab.min(n - 1);
        (VILLAGE_TABS[tab].draw)(self, ui, body);
        // 아래 안내: 탭마다 동사가 다르다.
        let hint = if VILLAGE_TABS[tab].lr {
            super::vfill(
                s.pack_hint,
                &[Verb::Pick, Verb::Change, Verb::Tabs, Verb::Back],
            )
        } else {
            super::vfill(s.decor_hint, &[Verb::Confirm, Verb::Tabs, Verb::Back])
        };
        let hint = ui.fit(&hint, PANEL.w - 8.0, FONT_PX);
        ui.text_center(&hint, 160.0, PANEL.bottom() + 1.0, FONT_PX, ui::TEXT_DIM);
    }

    /// 왼쪽 목록 한 벌: (작은 그림, 이름, 오른쪽 글, 흐리게). 고른 줄이 보이게 굴린다(`ListView`).
    fn draw_village_list(&mut self, ui: &Ui, r: Rect, rows: &[(VillagePic, String, String, bool)]) {
        let fits = ((r.h / ROW_H).floor() as usize).max(1);
        let n = rows.len();
        let first = self.village_view.window(self.decor_cursor, n, fits, ROW_H);
        let t = get_time() as f32;
        let glow = self.world.adv.village.map_or(0, |v| v.glow);
        for (k, (pic, name, right, dim)) in rows.iter().enumerate().skip(first).take(fits) {
            let row = Rect::new(r.x, r.y + (k - first) as f32 * ROW_H, r.w, ROW_H - 1.0);
            self.hits.push((row, Hit::ShopItem(k)));
            let on = k == self.decor_cursor;
            if on {
                ui.rect(row, ui::PANEL_LIGHT);
                ui.rect_lines(row, ui::ACCENT);
            }
            let icon = Rect::new(row.x + 1.0, row.y, ROW_H, ROW_H - 1.0);
            self.renderer
                .draw_village_pic(ui, *pic, icon, t, pic_zoom(*pic) * 0.5, glow);
            let c = if *dim {
                ui::TEXT_DIM
            } else if on {
                ui::TEXT
            } else {
                Color::new(0.78, 0.72, 0.64, 1.0)
            };
            let rw = ui.text_w(right, FONT_PX);
            let name = ui.fit(name, row.w - ROW_H - rw - 10.0, FONT_PX);
            ui.text(&name, icon.right() + 3.0, ui::mid_y(row), FONT_PX, c);
            ui.text_right(right, row.right() - 3.0, ui::mid_y(row), FONT_PX, c);
        }
        if first > 0 {
            ui.text("▲", r.right() - 9.0, r.y - 12.0, FONT_PX, ui::TEXT_DIM);
        }
        if first + fits < n {
            ui.text(
                "▼",
                r.right() - 9.0,
                r.y + fits as f32 * ROW_H - 3.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
    }

    /// 오른쪽 칸: 큰 그림, 이름, 그 아래 한 줄(값·놓음·얻는 곳). 그 아래 줄 `y`를 돌려준다(짐은 거기에 −·+).
    fn draw_village_detail(
        &mut self,
        ui: &Ui,
        r: Rect,
        pic: VillagePic,
        name: &str,
        info: &str,
    ) -> f32 {
        let t = get_time() as f32;
        let glow = self.world.adv.village.map_or(0, |v| v.glow);
        let art = Rect::new(r.x, r.y, 44.0, 44.0);
        ui.rect(art, Color::new(0.20, 0.30, 0.18, 1.0));
        self.renderer
            .draw_village_pic(ui, pic, art, t, pic_zoom(pic), glow);
        ui.rect_lines(art, ui::BORDER);
        let tx = art.right() + 6.0;
        let mut y = r.y + 2.0;
        for l in ui::wrap(ui, name, r.right() - tx, FONT_PX).iter().take(2) {
            ui.text(l, tx, y, FONT_PX, ui::ACCENT);
            y += 12.0;
        }
        for l in ui::wrap(ui, info, r.right() - tx, FONT_PX).iter().take(2) {
            ui.text(l, tx, y + 2.0, FONT_PX, ui::TEXT);
            y += 12.0;
        }
        art.bottom() + 4.0
    }

    /// 꾸미기 탭에서 포포가 하는 말: 고른 꾸미기의 설명(발자국 꾸미기는 얻는 곳까지).
    fn decor_say(&self) -> String {
        let s = i18n::s();
        let ids = decor_rows();
        let i = ids[self.decor_cursor.min(ids.len() - 1)];
        if i >= DECOR.len() && !self.profile.flag(&key_decor(i)) {
            format!("{} {}", tr(decor_desc(i)), s.decor_journal_d)
        } else {
            tr(decor_desc(i)).to_string()
        }
    }

    fn draw_decor_tab(&mut self, ui: &Ui, body: Rect) {
        let s = i18n::s();
        let have = self.profile.count(KEY_SEEDS);
        let ids = decor_rows();
        let rows: Vec<(VillagePic, String, String, bool)> = ids
            .iter()
            .map(|&i| {
                let owned = self.profile.flag(&key_decor(i));
                let journal = i >= DECOR.len();
                let right = if owned {
                    s.decor_owned.to_string()
                } else if journal {
                    s.decor_journal.to_string()
                } else {
                    fill(s.shop_price, &[&DECOR[i].price])
                };
                let dim = !owned && (journal || have < DECOR[i].price);
                (
                    VillagePic::Decor(i),
                    tr(decor_name(i)).to_string(),
                    right,
                    dim,
                )
            })
            .collect();
        let list = Rect::new(body.x, body.y, LIST_W, body.h - 4.0);
        self.draw_village_list(ui, list, &rows);
        let i = ids[self.decor_cursor.min(ids.len() - 1)];
        let detail = Rect::new(
            list.right() + 8.0,
            body.y,
            body.right() - list.right() - 8.0,
            body.h,
        );
        // 이름 아래 한 줄: 놓았다 / 발자국 보상 / 값.
        let info = if self.profile.flag(&key_decor(i)) {
            s.decor_owned.to_string()
        } else if i >= DECOR.len() {
            s.decor_journal.to_string()
        } else {
            fill(s.shop_price, &[&DECOR[i].price])
        };
        self.draw_village_detail(ui, detail, VillagePic::Decor(i), tr(decor_name(i)), &info);
    }

    fn pack_name(p: &crate::village::Pack) -> String {
        if p.kind == PackKind::Pouch {
            fill(tr(p.name), &[&PACK_POUCH_SEEDS])
        } else {
            tr(p.name).to_string()
        }
    }

    fn draw_pack_tab(&mut self, ui: &Ui, body: Rect) {
        let s = i18n::s();
        let have = self.profile.count(KEY_SEEDS);
        let rows: Vec<(VillagePic, String, String, bool)> = PACK
            .iter()
            .map(|p| {
                let n = self.profile.count(key_pack(p.kind));
                // 칸마다 ●○ 점: 싼 수 / 최대.
                let pips: String = (0..p.max).map(|k| if k < n { '●' } else { '○' }).collect();
                let right = format!("{pips} {}", fill(s.shop_price, &[&p.price]));
                (
                    VillagePic::Pack(p.kind),
                    Self::pack_name(p),
                    right,
                    n >= p.max || have < p.price,
                )
            })
            .collect();
        let list = Rect::new(body.x, body.y, LIST_W, body.h - 4.0);
        self.draw_village_list(ui, list, &rows);
        // 목록 아래: 다음 모험에 가져갈 것 한 줄.
        let packed: Vec<String> = PACK
            .iter()
            .filter_map(|p| {
                let n = self.profile.count(key_pack(p.kind));
                (n > 0).then(|| format!("{} ×{n}", Self::pack_name(p)))
            })
            .collect();
        let ny = list.y + PACK.len() as f32 * ROW_H + 6.0;
        ui.text(s.pack_now, list.x, ny, FONT_PX, ui::ACCENT);
        let line = if packed.is_empty() {
            "—".to_string()
        } else {
            packed.join(" · ")
        };
        for (k, l) in ui::wrap(ui, &line, list.w, FONT_PX)
            .iter()
            .take(3)
            .enumerate()
        {
            ui.text(l, list.x, ny + 13.0 + k as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        let i = self.decor_cursor.min(PACK.len() - 1);
        let p = &PACK[i];
        let detail = Rect::new(
            list.right() + 8.0,
            body.y,
            body.right() - list.right() - 8.0,
            body.h,
        );
        let y = self.draw_village_detail(
            ui,
            detail,
            VillagePic::Pack(p.kind),
            &Self::pack_name(p),
            &fill(i18n::s().shop_price, &[&p.price]),
        );
        // − 수 + 단추(터치·마우스). 키·패드는 좌우.
        let n = self.profile.count(key_pack(p.kind));
        let by = (y + 4.0).min(detail.bottom() - FONT_PX - 6.0);
        let minus = Rect::new(detail.x, by, 20.0, FONT_PX + 4.0);
        let plus = Rect::new(detail.x + 60.0, by, 20.0, FONT_PX + 4.0);
        for (r, label, d) in [(minus, "-", -1i8), (plus, "+", 1)] {
            ui.rect(r, ui::PANEL_LIGHT);
            ui.rect_lines(r, ui::BORDER);
            ui.text_center(label, r.center().x, ui::mid_y(r), FONT_PX, ui::TEXT);
            self.hits.push((r, Hit::PackStep(i, d)));
        }
        ui.text_center(
            &format!("{n}/{}", p.max),
            detail.x + 40.0,
            ui::mid_y(minus),
            FONT_PX,
            ui::TEXT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 마을 창의 꾸미기 줄: 가게 여섯 + 저널 둘, 지운 반딧불 병(6)은 없다. 설명이 다 있다.
    #[test]
    fn decor_rows_skip_the_removed_jar_and_have_descriptions() {
        let rows = decor_rows();
        assert_eq!(rows.len(), 8);
        assert!(!rows.contains(&6));
        let ko = &crate::i18n::KO;
        for i in rows {
            assert!(!decor_desc(i)(ko).is_empty(), "{i} 설명");
        }
    }
}
