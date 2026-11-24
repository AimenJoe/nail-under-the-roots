//! 마을 창(0269 R35, 요구 2.5·2.7): 마을의 포포에게 말을 걸면 연다. 탭 표 `VILLAGE_TABS`(꾸미기 · 짐 꾸리기 · 마을 수첩) — 새 탭 = 표 한 줄 +
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
    decor_desc, decor_name, decor_price, key_decor, key_pack, PackKind, DECOR, EXTRA_DECOR,
    EXTRA_FIRST, JOURNAL_SHOWN, KEY_SEEDS, PACK, PACK_POUCH_SEEDS,
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
    // 이웃 등불(0304, v1.1): 맡긴 반딧불로 등불을 건네면 그 이웃의 이야기가 하나씩.
    VillageTab {
        name: |s| s.lamp_tab,
        lr: false,
        update: |app, a, tap| app.update_lamp_tab(a, tap),
        draw: |app, ui, r| app.draw_lamp_tab(ui, r),
        say: |_| i18n::s().lamp_say.to_string(),
    },
    // 마을 수첩(0269 2.7): 장소·이웃 이야기. 사는 것이 없다 — 고르면 포포가 위 말 칸에서 읽어 준다.
    VillageTab {
        name: |s| s.notes_tab,
        lr: false,
        update: |app, a, tap| {
            let n = crate::village_notes::visible(&app.profile).len();
            app.village_pick(a, tap, n);
        },
        draw: |app, ui, r| app.draw_notes_tab(ui, r),
        say: |app| app.note_say(),
    },
];

/// 텃밭 칸 앞에서 여는 마을 창(0304 G1): 탭은 텃밭 하나. 좌우 = 심을 씨앗(꽃씨·산딸기 씨) 바꾸기.
pub(super) const GARDEN_TABS: &[VillageTab] = &[VillageTab {
    name: |s| s.garden_tab,
    lr: true,
    update: |app, a, tap| app.update_garden_tab(a, tap),
    draw: |app, ui, r| app.draw_garden_tab(ui, r),
    say: |_| i18n::s().garden_say.to_string(),
}];

/// 꾸미기 탭의 줄: 가게 꾸미기(더한 것 포함) + 저널 꾸미기, 지운 1번 등·6번 병은 빼고(0269).
fn decor_rows() -> Vec<usize> {
    (0..DECOR.len())
        .chain(EXTRA_FIRST..EXTRA_FIRST + EXTRA_DECOR.len())
        .chain(JOURNAL_SHOWN)
        .filter(|i| !crate::village::REMOVED_DECOR.contains(i))
        .collect()
}

/// 자세히 칸(44px)에 그릴 배율: 꾸미기는 1배(팩 그림 꽃밭이 36px쯤), 짐(산딸기·지렁이·씨앗)은 작아서 2배. 목록 칸은 이 반.
fn pic_zoom(p: VillagePic) -> f32 {
    match p {
        VillagePic::Decor(_) => 1.0,
        VillagePic::Pack(_) => 2.0,
        VillagePic::Spot(crate::village_notes::Spot::Tree) => 0.5,
        VillagePic::Spot(_) => 1.5,
        // 목록 줄에서는 이 반(0.5배): 32칸 몸이 줄 높이(17px) 안에 다 든다. 화면 배율이 곱해지니 점은 그대로 또렷하다.
        VillagePic::Neighbor(_) => 1.0,
        VillagePic::Plot(_) => 2.0,
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
        self.village_tabs = VILLAGE_TABS;
        self.decor_cursor = 0;
        self.decor_tab = 0;
        // 처음 열면 포포의 인사(자세히 칸의 말 자리, 몇 초 뒤 설명으로).
        self.shop_msg = Some((|s| s.decor_hi, 3.5));
        self.village_view = Default::default();
        self.world.freeze();
        self.scene = Scene::Decor;
    }

    /// 텃밭 칸 `i` 앞에서: 텃밭 탭 하나짜리 마을 창을 그 칸에 맞춰 연다(0304 G1).
    pub(super) fn open_garden(&mut self, i: usize) {
        self.village_tabs = GARDEN_TABS;
        self.decor_tab = 0;
        self.decor_cursor = i.min(crate::garden::PLOTS - 1);
        self.shop_msg = None;
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
        let tabs = self.village_tabs;
        let n = tabs.len();
        let cur = self.decor_tab.min(n - 1);
        let lr = tabs[cur].lr;
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
        (tabs[cur].update)(self, a, tap_hit);
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
        decor_price(i).is_some_and(|p| {
            !self.profile.flag(&key_decor(i)) && self.profile.count(KEY_SEEDS) >= p
        })
    }

    fn place_decor(&mut self, i: usize) {
        let have = self.profile.count(KEY_SEEDS);
        if self.profile.flag(&key_decor(i)) {
            // 이미 놓았다: 아무 일도 없다.
        } else if let Some(price) = decor_price(i).filter(|p| have >= *p) {
            self.profile.set_count(KEY_SEEDS, have - price);
            self.profile.set(&key_decor(i));
            crate::profile::save(&self.profile);
            if let Some(v) = self.world.adv.village.as_mut() {
                v.decor[i] = true;
            }
            self.shop_msg = Some((|s| s.decor_thanks, 2.5));
            self.sfx.play(crate::sfx::Id::Buy);
        } else if decor_price(i).is_none() {
            // 저널 꾸미기는 사지 않는다 — 발자국 목표의 보상.
            self.shop_msg = Some((|s| s.decor_journal_d, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else {
            self.shop_msg = Some((|s| s.decor_poor, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
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
        let tabs = self.village_tabs;
        let n = tabs.len();
        let tab_w = ((PANEL.w - 30.0 - seeds_w) / n as f32).clamp(30.0, 90.0);
        for (i, t) in tabs.iter().enumerate() {
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
            None => (tabs[self.decor_tab.min(n - 1)].say)(self),
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
        (tabs[tab].draw)(self, ui, body);
        // 아래 안내: 탭마다 동사가 다르다.
        let hint = if std::ptr::eq(tabs, GARDEN_TABS) {
            super::vfill(
                s.garden_hint,
                &[Verb::Pick, Verb::Change, Verb::Confirm, Verb::Back],
            )
        } else if tabs[tab].lr {
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
                .draw_village_pic(ui, *pic, icon, t, pic_zoom(*pic) * 0.5);
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
        let art = Rect::new(r.x, r.y, 44.0, 44.0);
        ui.rect(art, Color::new(0.20, 0.30, 0.18, 1.0));
        self.renderer
            .draw_village_pic(ui, pic, art, t, pic_zoom(pic));
        ui.rect_lines(art, ui::BORDER);
        let tx = art.right() + 6.0;
        let mut y = r.y + 2.0;
        let names = ui::wrap(ui, name, r.right() - tx, FONT_PX);
        let names = &names[..names.len().min(2)];
        for l in names {
            ui.text(l, tx, y, FONT_PX, ui::ACCENT);
            y += 12.0;
        }
        // 그림 오른쪽은 세 줄 자리(44px). 이름과 함께 넘치면 설명은 그림 아래에 칸 폭 그대로 — 등불 다음 이야기·텃밭 자라는 중처럼
        // 긴 글이 잘려 값이 안 보이던 것(0306 뒤 검토). 아래로 간 줄 수만큼 돌려주는 y가 내려간다.
        let side = ui::wrap(ui, info, r.right() - tx, FONT_PX);
        if names.len() + side.len() <= 3 {
            for l in &side {
                ui.text(l, tx, y + 2.0, FONT_PX, ui::TEXT);
                y += 12.0;
            }
            return art.bottom() + 4.0;
        }
        let mut y = art.bottom() + 4.0;
        let fits = ((r.bottom() - y) / 12.0).floor().max(1.0) as usize;
        for l in ui::wrap(ui, info, r.w, FONT_PX).iter().take(fits) {
            ui.text(l, r.x, y, FONT_PX, ui::TEXT);
            y += 12.0;
        }
        y
    }

    /// 꾸미기 탭에서 포포가 하는 말: 고른 꾸미기의 설명(발자국 꾸미기는 얻는 곳까지).
    fn decor_say(&self) -> String {
        let s = i18n::s();
        let ids = decor_rows();
        let i = ids[self.decor_cursor.min(ids.len() - 1)];
        if decor_price(i).is_none() && !self.profile.flag(&key_decor(i)) {
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
                let price = decor_price(i);
                let right = match price {
                    _ if owned => s.decor_owned.to_string(),
                    None => s.decor_journal.to_string(),
                    Some(p) => fill(s.shop_price, &[&p]),
                };
                let dim = !owned && price.is_none_or(|p| have < p);
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
        } else if let Some(p) = decor_price(i) {
            fill(s.shop_price, &[&p])
        } else {
            s.decor_journal.to_string()
        };
        self.draw_village_detail(ui, detail, VillagePic::Decor(i), tr(decor_name(i)), &info);
    }

    /// 수첩 탭에서 고른 줄(표 번호).
    fn note_at(&self) -> &'static crate::village_notes::Note {
        let vis = crate::village_notes::visible(&self.profile);
        &crate::village_notes::NOTES[vis[self.decor_cursor.min(vis.len() - 1)]]
    }

    fn note_say(&self) -> String {
        tr(self.note_at().body).to_string()
    }

    fn draw_notes_tab(&mut self, ui: &Ui, body: Rect) {
        use crate::village_notes::{Spot, NOTES};
        let rows: Vec<(VillagePic, String, String, bool)> =
            crate::village_notes::visible(&self.profile)
                .iter()
                .map(|&i| {
                    let n = &NOTES[i];
                    // 이웃 줄은 마을에 선 모습을 아이콘으로(사장 2026-11-23: 인물만 아이콘이 비어 어색했다).
                    let pic = if n.spot.neighbor() {
                        VillagePic::Neighbor(n.spot)
                    } else {
                        VillagePic::Spot(n.spot)
                    };
                    (pic, tr(n.title).to_string(), String::new(), false)
                })
                .collect();
        let list = Rect::new(body.x, body.y, LIST_W, body.h - 4.0);
        self.draw_village_list(ui, list, &rows);
        let note = self.note_at();
        let detail = Rect::new(
            list.right() + 8.0,
            body.y,
            body.right() - list.right() - 8.0,
            body.h,
        );
        self.draw_village_detail(ui, detail, VillagePic::Spot(note.spot), tr(note.title), "");
        // 이웃은 그림 칸에 초상화.
        let portrait = match note.spot {
            Spot::Stall => Some(cutscene::Portrait::POPO),
            Spot::Bori => Some(cutscene::Portrait::BORI),
            Spot::Momo => Some(cutscene::Portrait::MOMO),
            Spot::Doto => Some(cutscene::Portrait::DOTO),
            Spot::Kongi => Some(cutscene::Portrait::KONGI),
            Spot::Clover => Some(cutscene::Portrait::CLOVER),
            Spot::Halmae => Some(cutscene::Portrait::HALMAE),
            _ => None,
        };
        if let Some(p) = portrait.filter(|_| note.spot.neighbor()) {
            let art = Rect::new(detail.x + 4.0, detail.y + 4.0, 36.0, 36.0);
            self.renderer.draw_portrait(ui, art, p.frame(Face::Normal));
        }
    }

    // ---- 이웃 등불(0304) ----------------------------------------------------

    /// 등불 탭의 줄: 마을에 서 있는 이웃(`lanterns::rows`).
    fn lamp_rows(&self) -> Vec<usize> {
        crate::lanterns::rows(&self.profile, self.world.character)
    }

    fn update_lamp_tab(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let rows = self.lamp_rows();
        if rows.is_empty() {
            return;
        }
        let (moved, again) = self.village_pick(a, tap_hit, rows.len());
        let i = rows[self.decor_cursor.min(rows.len() - 1)];
        if a.confirm || again {
            self.give_lamp(i);
        } else if moved && tap_hit.is_some() {
            self.shop_msg = Some((|s| s.tap_again, 2.5));
        }
    }

    /// 등불을 건넨다: 반딧불을 내고, 창을 닫고, 그 이웃의 이야기(컷씬)를 튼다. 셋째면 곁에 등불이 걸린다.
    fn give_lamp(&mut self, i: usize) {
        use crate::lanterns::{give, next, Next, ALL};
        let ch = self.world.character;
        let have = self.profile.count(crate::talents::KEY_LIGHT);
        let stage = match next(&self.profile, i, ch) {
            Next::Ready { stage, cost } if have >= cost => stage,
            Next::Ready { .. } => {
                self.shop_msg = Some((|s| s.lamp_poor, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
            Next::Done => {
                self.shop_msg = Some((|s| s.lamp_done, 2.5));
                return;
            }
            Next::Later => {
                self.shop_msg = Some((|s| s.lamp_later, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
            Next::Other => {
                self.shop_msg = Some((|s| s.lamp_other, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
        };
        // 개발 시나리오는 기록 없이 이번에만 듣는다(부탁과 같게).
        if self.recording() {
            give(&mut self.profile, i, ch);
            self.save_profile();
        }
        if stage == 2 {
            if let Some(v) = self.world.adv.village.as_mut() {
                v.lamps[i] = true;
            }
        }
        // 맡긴 반딧불이 줄었다: 우물 위 반딧불·큰 나무 빛도 따라서.
        let (glow, ready) = (
            crate::village::glow_level(self.profile.count(crate::talents::KEY_LIGHT)),
            self.tree_affordable(),
        );
        if let Some(v) = self.world.adv.village.as_mut() {
            v.glow = glow;
            v.tree_ready = ready;
        }
        self.sfx.play(crate::sfx::Id::Buy);
        self.shop_msg = None;
        self.scene = Scene::Game;
        self.play(cutscene::lamp_scene(
            &self.world,
            tr(ALL[i].name),
            ALL[i].scenes[stage].title,
            &crate::lanterns::lines(i, stage),
        ));
    }

    /// 텃밭 탭: 위아래 = 칸, 빈 칸에서 좌우 = 심을 씨앗, 확인 = 심기 / 거두기(다 자란 칸) / 기다리라는 말(자라는 칸).
    /// 개발 시나리오는 저장하지 않는다(이번 세션의 기록 사본에만 — 다시 열면 그대로 보인다).
    fn update_garden_tab(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        use crate::garden::{self, Crop, No, Plot, PLOTS};
        let (moved, again) = self.village_pick(a, tap_hit, PLOTS);
        let i = self.decor_cursor.min(PLOTS - 1);
        let plot = garden::plot(&self.profile, i);
        if (a.left || a.right) && plot == Plot::Empty {
            self.garden_crop = (self.garden_crop + 1) % Crop::ALL.len();
            self.shop_msg = None;
            self.sfx.play(crate::sfx::Id::UiMove);
            return;
        }
        if !(a.confirm || again) {
            if moved && tap_hit.is_some() {
                self.shop_msg = Some((|s| s.tap_again, 2.5));
            }
            return;
        }
        let crop = Crop::ALL[self.garden_crop % Crop::ALL.len()];
        let done = match plot {
            Plot::Empty => match garden::plant(&mut self.profile, i, crop) {
                Ok(()) => Some((|s: &i18n::Strings| s.garden_planted) as Tr),
                Err(No::Poor) => {
                    self.shop_msg = Some((|s| s.decor_poor, 3.0));
                    None
                }
                Err(No::Busy) => None,
            },
            Plot::Growing { .. } => {
                self.shop_msg = Some((|s| s.garden_wait, 2.5));
                None
            }
            Plot::Ripe(_) => {
                // 화단이 다 찼으면 가장 오래된 자리가 바뀐다 — 그렇다고 알려 준다.
                let (_, replaces) = garden::next_spot(&self.profile);
                match garden::harvest(&mut self.profile, i) {
                    Some(Crop::Berry) => Some((|s: &i18n::Strings| s.garden_picked) as Tr),
                    Some(Crop::Flower(_)) if replaces => {
                        Some((|s: &i18n::Strings| s.garden_replant) as Tr)
                    }
                    Some(Crop::Flower(_)) => Some((|s: &i18n::Strings| s.garden_moved) as Tr),
                    None => None,
                }
            }
        };
        let Some(msg) = done else {
            self.sfx.play(crate::sfx::Id::Deny);
            return;
        };
        let (plots, blooms) = (garden::plots(&self.profile), garden::spots(&self.profile));
        if let Some(v) = self.world.adv.village.as_mut() {
            v.plots = plots;
            v.blooms = blooms;
        }
        if self.recording() {
            self.save_profile();
        }
        self.shop_msg = Some((msg, 3.0));
        self.sfx.play(crate::sfx::Id::Buy);
    }

    fn draw_garden_tab(&mut self, ui: &Ui, body: Rect) {
        use crate::garden::{self, Crop, Plot, GROW, PLOTS};
        let s = i18n::s();
        let plots = garden::plots(&self.profile);
        let rows: Vec<(VillagePic, String, String, bool)> = plots
            .iter()
            .map(|&p| {
                let (name, right) = match p {
                    Plot::Empty => (s.garden_empty.to_string(), String::new()),
                    Plot::Growing { crop, age } => {
                        (tr(crop.name()).to_string(), format!("{age}/{GROW}"))
                    }
                    Plot::Ripe(crop) => (tr(crop.name()).to_string(), s.garden_ripe.to_string()),
                };
                (VillagePic::Plot(p), name, right, false)
            })
            .collect();
        let list = Rect::new(body.x, body.y, LIST_W, body.h - 4.0);
        self.draw_village_list(ui, list, &rows);
        let i = self.decor_cursor.min(PLOTS - 1);
        let detail = Rect::new(
            list.right() + 8.0,
            body.y,
            body.right() - list.right() - 8.0,
            body.h,
        );
        let crop = Crop::ALL[self.garden_crop % Crop::ALL.len()];
        let (pic, name, info) = match plots[i] {
            Plot::Empty => (
                VillagePic::Plot(Plot::Ripe(crop)),
                format!("◀ {} ▶", tr(crop.name())),
                fill(s.garden_plant, &[&crop.price()]),
            ),
            Plot::Growing { crop, age } => (
                VillagePic::Plot(plots[i]),
                tr(crop.name()).to_string(),
                fill(s.garden_growing, &[&(GROW - age)]),
            ),
            Plot::Ripe(Crop::Berry) => (
                VillagePic::Plot(plots[i]),
                tr(Crop::Berry.name()).to_string(),
                s.garden_ripe_berry.to_string(),
            ),
            Plot::Ripe(Crop::Flower(b)) => (
                VillagePic::Plot(plots[i]),
                tr(b.name()).to_string(),
                s.garden_ripe_flower.to_string(),
            ),
        };
        let y = self.draw_village_detail(ui, detail, pic, &name, &info);
        // 거둬 둔 산딸기(다음 길 가방에), 마을에 옮겨 심은 꽃.
        let p = &self.profile;
        let stock = fill(
            s.garden_stock,
            &[
                &p.count(garden::KEY_BERRIES),
                &garden::flowers(p),
                &garden::FLOWER_SPOTS.len(),
            ],
        );
        // 남은 자리만큼(설명이 그림 아래로 내려오면 한 줄로 줄어든다).
        let room = ((detail.bottom() - FONT_PX - 2.0 - y) / 12.0)
            .floor()
            .max(0.0) as usize
            + 1;
        for (k, l) in ui::wrap(ui, &stock, detail.w, FONT_PX)
            .iter()
            .take(room.min(2))
            .enumerate()
        {
            ui.text(l, detail.x, y + k as f32 * 12.0, FONT_PX, ui::TEXT_DIM);
        }
    }

    fn draw_lamp_tab(&mut self, ui: &Ui, body: Rect) {
        use crate::lanterns::{level, next, Next, ALL};
        let s = i18n::s();
        let ch = self.world.character;
        let have = self.profile.count(crate::talents::KEY_LIGHT);
        let ids = self.lamp_rows();
        if ids.is_empty() {
            return;
        }
        let rows: Vec<(VillagePic, String, String, bool)> = ids
            .iter()
            .map(|&i| {
                let n = next(&self.profile, i, ch);
                let dim = match n {
                    Next::Ready { cost, .. } => have < cost,
                    _ => true,
                };
                (
                    VillagePic::Neighbor(lamp_spot(ALL[i].who)),
                    tr(ALL[i].name).to_string(),
                    format!("{}/3", level(&self.profile, i)),
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
        let info = match next(&self.profile, i, ch) {
            Next::Ready { stage, cost } => {
                fill(s.lamp_next, &[&tr(ALL[i].scenes[stage].title), &cost])
            }
            Next::Done => s.lamp_done.to_string(),
            Next::Later => s.lamp_later.to_string(),
            Next::Other => s.lamp_other.to_string(),
        };
        let y = self.draw_village_detail(
            ui,
            detail,
            VillagePic::Neighbor(lamp_spot(ALL[i].who)),
            tr(ALL[i].name),
            &info,
        );
        // 가진 반딧불(큰 나무 창과 같은 글) — 설명이 길어 자리가 없으면 뺀다(마을 화면 오른쪽 위에도 보인다).
        if y + 12.0 <= detail.bottom() {
            ui.text(
                &fill(s.tree_light, &[&have]),
                detail.x,
                y,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
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
                // 오른쪽은 값만 — 싼 수는 자세히 칸의 n/최대와 목록 아래 줄에(●○ 점은 뺐다, 사장 2026-11-23).
                let right = fill(s.shop_price, &[&p.price]);
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

/// 등불 탭의 이웃 그림: 수첩과 같은 이웃 자리(포포는 좌판).
pub(super) fn lamp_spot(who: crate::world::Who) -> crate::village_notes::Spot {
    use crate::village_notes::Spot;
    use crate::world::Who;
    match who {
        Who::Popo => Spot::Stall,
        Who::Bori => Spot::Bori,
        Who::Momo => Spot::Momo,
        Who::Doto => Spot::Doto,
        Who::Kongi => Spot::Kongi,
        Who::Clover => Spot::Clover,
        Who::Halmae => Spot::Halmae,
        Who::Susu | Who::Nail => Spot::Stall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 마을 창의 꾸미기 줄: 가게 다섯 + 더한 셋 + 저널 둘, 지운 반딧불 등(1)·병(6)은 없다. 설명이 다 있다.
    #[test]
    fn decor_rows_skip_removed_decor_and_have_descriptions() {
        let rows = decor_rows();
        assert_eq!(rows.len(), 10);
        assert!(!rows.contains(&1) && !rows.contains(&6));
        let ko = &crate::i18n::KO;
        for i in rows {
            assert!(!decor_desc(i)(ko).is_empty(), "{i} 설명");
        }
    }
}
