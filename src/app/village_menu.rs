//! 마을 창(0269 R35, 요구 2.5·2.7): 마을의 포포에게 말을 걸면 연다(텃밭 칸 앞에서는 텃밭 탭 하나짜리로).
//!
//! **탭 = 표 한 줄(0307 R42).** 탭은 줄 목록·자세히 칸·포포의 말을 **기록만 보고** 내고(`rows`·`detail`·`say` — `Ctx`만 받는
//! 순수 함수라 앱 없이 시험한다), 고른 줄에 한 일은 `act` 하나가 받는다. 줄 옮기기·터치 "한 번 더"·목록·자세히 칸·누를 자리는
//! 이 파일의 몰이꾼(`update_village_menu`·`draw_village_menu`)이 탭마다 같은 틀로 한다 — 입력과 그림이 같은 줄 목록을 쓴다.
//! 틀 밖의 것(짐의 −+ 단추, 텃밭의 재고 줄, 수첩의 초상화)만 `extra`로 얹는다. 새 탭 = 표 한 줄 + 순수 함수 셋 + `act`.
//!
//! 데이터·저장은 합치지 않았다: 꾸미기는 깃발 `decor.<i>`, 짐은 수 `pack.*`, 화폐는 둘 다 마을 씨앗 — 사면 바로 저장,
//! 짐은 새 런 시작 때 `App::unpack`이 꺼낸다(0259 그대로). 등불은 `lanterns.rs`, 텃밭은 `garden.rs`.
//!
//! 화면: 위 탭 + 마을 씨앗, 그 아래 왼쪽 위 포포의 초상화와 말(고른 것의 설명, 산 직후엔 한마디 — 쉼터 가게와 같은 자리),
//! 아래 왼쪽 목록(작은 그림 · 이름 · 상태), 오른쪽 큰 그림 · 이름 · 값·수.
//! 사기: 키·패드는 확인 = 산다(지금처럼 빠르게). 터치·마우스는 첫 누름 = 고르기, 고른 줄을 다시 누름 = 사기(실수로 사지 않게,
//! 확인 창을 따로 띄우지 않는다). 짐은 좌우·−+ 단추로 빼고 넣는다(뺀 것은 씨앗을 돌려준다).

use super::*;
use crate::cutscene::Face;
use crate::profile::Profile;
use crate::render::VillagePic;
use crate::village::{
    decor_desc, decor_name, decor_price, key_decor, key_pack, PackKind, DECOR, EXTRA_DECOR,
    EXTRA_FIRST, JOURNAL_SHOWN, KEY_SEEDS, PACK, PACK_POUCH_SEEDS,
};

/// 목록의 한 줄: 작은 그림, 이름, 오른쪽 글(값·단계), 흐리게(지금 못 하는 줄).
pub(super) struct Row {
    pub pic: VillagePic,
    pub name: String,
    pub right: String,
    pub dim: bool,
}

/// 오른쪽 자세히 칸: 큰 그림, 이름, 그 아래 설명(값·놓음·다음 이야기).
pub(super) struct Detail {
    pub pic: VillagePic,
    pub name: String,
    pub info: String,
}

/// 고른 줄에 한 일: 확인(또는 고른 줄을 다시 누름) / 왼쪽·− / 오른쪽·+.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Pick,
    Dec,
    Inc,
}

/// 탭의 순수한 쪽(`rows`·`detail`·`say`)이 보는 것 전부: 기록, 걷는 캐릭터, 텃밭에서 고른 씨앗.
pub(super) struct Ctx<'a> {
    pub p: &'a Profile,
    pub ch: Character,
    pub crop: usize,
}

/// 마을 창이 열려 있는 동안의 상태(0307 R42): 어느 탭 묶음(포포의 넷 / 텃밭 하나)의 어느 탭, 고른 줄, 굴림 창, 텃밭에서 고른 씨앗.
/// 새 창은 이렇게 제 묶음을 갖는다(`LabUi`·`DialogueUi`와 같게) — 화면마다의 커서를 `App`에 흩지 않는다.
pub(super) struct VillageUi {
    pub tabs: &'static [VillageTab],
    pub tab: usize,
    /// 고른 줄(탭을 바꾸면 0). 텃밭 창은 칸 번호.
    pub cursor: usize,
    pub view: crate::ui::ListView,
    /// 텃밭 탭에서 심을 씨앗(`garden::Crop::ALL` 번호).
    pub crop: usize,
    /// 포포가 지금 하는 한마디(없으면 고른 줄의 설명).
    pub say: Option<Say>,
}

impl Default for VillageUi {
    fn default() -> Self {
        VillageUi {
            tabs: VILLAGE_TABS,
            tab: 0,
            cursor: 0,
            view: Default::default(),
            crop: 0,
            say: None,
        }
    }
}

/// 마을 창의 탭 하나.
pub(super) struct VillageTab {
    pub name: Tr,
    /// 좌우를 탭 안에서 쓰나(짐: 빼기·넣기, 텃밭: 씨앗 바꾸기 — 그때 탭 넘기기는 Q/E·누르기로).
    pub lr: bool,
    /// 줄 목록. 입력(줄 수)과 그림이 같이 쓴다. 비어 있으면 탭은 아무것도 하지 않고 그리지 않는다.
    pub rows: fn(&Ctx) -> Vec<Row>,
    /// 고른 줄 `i`의 자세히 칸.
    pub detail: fn(&Ctx, usize) -> Detail,
    /// 포포가 위 말 칸에서 하는 말(고른 줄의 설명 — 산 직후 같은 한마디가 있으면 그게 먼저).
    pub say: fn(&Ctx, usize) -> String,
    /// 터치로 처음 고른 줄에 "한 번 더 누르면 …"을 알려 줄까(할 일이 있는 줄만).
    pub tap_hint: fn(&Ctx, usize) -> bool,
    /// 아래 조작 안내(탭마다 동사가 다르다).
    pub hint: fn() -> String,
    /// 고른 줄 `i`에 한 일: 사기·넣고 빼기·건네기·심고 거두기. 기록을 바꾸고 한마디·소리를 낸다.
    pub act: fn(&mut App, usize, Step),
    /// 틀 밖의 그림: (목록 칸, 자세히 칸, 설명 아래 줄 y, 고른 줄).
    pub extra: Option<Extra>,
}

type Extra = fn(&mut App, &Ui, Rect, Rect, f32, usize);

fn hint_shop() -> String {
    super::vfill(
        i18n::s().decor_hint,
        &[Verb::Confirm, Verb::Tabs, Verb::Back],
    )
}

fn hint_pack() -> String {
    super::vfill(
        i18n::s().pack_hint,
        &[Verb::Pick, Verb::Change, Verb::Tabs, Verb::Back],
    )
}

fn hint_garden() -> String {
    super::vfill(
        i18n::s().garden_hint,
        &[Verb::Pick, Verb::Change, Verb::Confirm, Verb::Back],
    )
}

pub(super) const VILLAGE_TABS: &[VillageTab] = &[
    VillageTab {
        name: |s| s.decor_tab,
        lr: false,
        rows: decor_list,
        detail: decor_detail,
        say: decor_say,
        tap_hint: |c, i| decor_can_buy(c.p, decor_rows()[i]),
        hint: hint_shop,
        act: |app, i, _| app.place_decor(decor_rows()[i]),
        extra: None,
    },
    VillageTab {
        name: |s| s.pack_tab,
        lr: true,
        rows: pack_list,
        detail: pack_detail,
        say: |_, i| tr(PACK[i].desc).to_string(),
        tap_hint: |_, _| true,
        hint: hint_pack,
        act: |app, i, step| match step {
            Step::Dec => app.pack_take(i),
            Step::Pick | Step::Inc => app.pack_put(i),
        },
        extra: Some(|app, ui, list, detail, y, i| app.draw_pack_extra(ui, list, detail, y, i)),
    },
    // 이웃 등불(0304, v1.1): 맡긴 반딧불로 등불을 건네면 그 이웃의 이야기가 하나씩.
    VillageTab {
        name: |s| s.lamp_tab,
        lr: false,
        rows: lamp_list,
        detail: lamp_detail,
        say: |_, _| i18n::s().lamp_say.to_string(),
        tap_hint: |_, _| true,
        hint: hint_shop,
        act: |app, i, _| {
            let who = crate::lanterns::rows(&app.profile, app.world.character)[i];
            app.give_lamp(who);
        },
        extra: Some(|app, ui, _, detail, y, _| app.draw_lamp_extra(ui, detail, y)),
    },
    // 마을 수첩(0269 2.7): 장소·이웃 이야기. 사는 것이 없다 — 고르면 포포가 위 말 칸에서 읽어 준다.
    VillageTab {
        name: |s| s.notes_tab,
        lr: false,
        rows: notes_list,
        detail: notes_detail,
        say: |c, i| tr(note_at(c.p, i).body).to_string(),
        tap_hint: |_, _| false,
        hint: hint_shop,
        act: |_, _, _| {},
        extra: Some(|app, ui, _, detail, _, i| app.draw_notes_extra(ui, detail, i)),
    },
];

/// 텃밭 칸 앞에서 여는 마을 창(0304 G1): 탭은 텃밭 하나. 좌우 = 심을 씨앗(꽃씨·산딸기 씨) 바꾸기.
pub(super) const GARDEN_TABS: &[VillageTab] = &[VillageTab {
    name: |s| s.garden_tab,
    lr: true,
    rows: garden_list,
    detail: garden_detail,
    say: |_, _| i18n::s().garden_say.to_string(),
    tap_hint: |_, _| true,
    hint: hint_garden,
    act: |app, i, step| app.garden_act(i, step),
    extra: Some(|app, ui, _, detail, y, _| app.draw_garden_extra(ui, detail, y)),
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
        self.village_ui.tabs = VILLAGE_TABS;
        self.village_ui.cursor = 0;
        self.village_ui.tab = 0;
        // 처음 열면 포포의 인사(자세히 칸의 말 자리, 몇 초 뒤 설명으로).
        self.village_ui.say = Some(Say::plain(|s| s.decor_hi, 3.5));
        self.village_ui.view = Default::default();
        self.world.freeze();
        self.scene = Scene::Decor;
    }

    /// 텃밭 칸 `i` 앞에서: 텃밭 탭 하나짜리 마을 창을 그 칸에 맞춰 연다(0304 G1).
    pub(super) fn open_garden(&mut self, i: usize) {
        self.village_ui.tabs = GARDEN_TABS;
        self.village_ui.tab = 0;
        self.village_ui.cursor = i.min(crate::garden::PLOTS - 1);
        self.village_ui.say = None;
        self.village_ui.view = Default::default();
        self.world.freeze();
        self.scene = Scene::Decor;
    }

    pub(super) fn update_village_menu(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        if let Some(m) = self.village_ui.say.as_mut() {
            m.secs -= dt;
            if m.secs <= 0.0 {
                self.village_ui.say = None;
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.village_ui.say = None;
            self.scene = Scene::Game;
            return;
        }
        let tabs = self.village_ui.tabs;
        let n = tabs.len();
        let cur = self.village_ui.tab.min(n - 1);
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
            self.village_ui.tab = tab;
            self.village_ui.cursor = 0;
            self.village_ui.view = Default::default();
            self.village_ui.say = None;
            self.sfx.play(crate::sfx::Id::UiMove);
            return;
        }
        // 탭 안: 줄 옮기기 → 고른 줄에 한 일. 줄 목록은 그림과 같은 것(`rows`)에서.
        let t = &tabs[cur];
        let n = (t.rows)(&self.village_ctx()).len();
        if n == 0 {
            return;
        }
        let (moved, again) = self.village_pick(a, tap_hit, n);
        let step = match tap_hit {
            Some(Hit::PackStep(k, d)) => {
                self.village_ui.cursor = k.min(n - 1);
                Some(if d > 0 { Step::Inc } else { Step::Dec })
            }
            _ if lr && a.right => Some(Step::Inc),
            _ if a.confirm || again => Some(Step::Pick),
            _ if lr && a.left => Some(Step::Dec),
            _ => None,
        };
        let i = self.village_ui.cursor.min(n - 1);
        match step {
            Some(step) => (t.act)(self, i, step),
            // 터치로 처음 고른 줄: 한 번 더 누르면 한다고 알려 준다.
            None if moved && tap_hit.is_some() && (t.tap_hint)(&self.village_ctx(), i) => {
                self.village_ui.say = Some(Say::plain(|s| s.tap_again, 2.5));
            }
            None => {}
        }
    }

    /// 탭의 순수한 쪽에 건네는 것.
    fn village_ctx(&self) -> Ctx<'_> {
        Ctx {
            p: &self.profile,
            ch: self.world.character,
            crop: self.village_ui.crop,
        }
    }

    /// 위아래·끌기로 줄을 옮기고, 누른 줄을 돌려준다: (고른 줄이 바뀌었나, 이미 고른 줄을 다시 눌렀나).
    fn village_pick(&mut self, a: &Actions, tap_hit: Option<Hit>, n: usize) -> (bool, bool) {
        let was = self.village_ui.cursor;
        if a.up {
            self.village_ui.cursor = (self.village_ui.cursor + n - 1) % n;
        }
        if a.down {
            self.village_ui.cursor = (self.village_ui.cursor + 1) % n;
        }
        self.village_ui
            .view
            .scroll(self.scroll_in, &mut self.village_ui.cursor, n);
        let mut again = false;
        if let Some(Hit::ShopItem(i)) = tap_hit {
            again = i == self.village_ui.cursor;
            self.village_ui.cursor = i.min(n - 1);
        }
        self.village_ui.cursor = self.village_ui.cursor.min(n - 1);
        let moved = self.village_ui.cursor != was;
        if moved {
            self.sfx.play(crate::sfx::Id::UiMove);
        }
        (moved, again)
    }

    fn place_decor(&mut self, i: usize) {
        let have = self.profile.count(KEY_SEEDS);
        if self.profile.flag(&key_decor(i)) {
            // 이미 놓았다: 아무 일도 없다.
        } else if let Some(price) = decor_price(i).filter(|p| have >= *p) {
            self.profile.set_count(KEY_SEEDS, have - price);
            self.profile.set(&key_decor(i));
            self.write_profile();
            self.sync_village();
            self.village_ui.say = Some(Say::thanks(|s| s.decor_thanks, 2.5));
            self.sfx.play(crate::sfx::Id::Buy);
        } else if decor_price(i).is_none() {
            // 저널 꾸미기는 사지 않는다 — 발자국 목표의 보상.
            self.village_ui.say = Some(Say::plain(|s| s.decor_journal_d, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else {
            self.village_ui.say = Some(Say::plain(|s| s.decor_poor, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        }
    }

    fn pack_put(&mut self, i: usize) {
        let p = &PACK[i];
        let have = self.profile.count(KEY_SEEDS);
        let packed = self.profile.count(key_pack(p.kind));
        if packed >= p.max {
            self.village_ui.say = Some(Say::plain(|s| s.pack_full, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else if have < p.price {
            self.village_ui.say = Some(Say::plain(|s| s.decor_poor, 3.0));
            self.sfx.play(crate::sfx::Id::Deny);
        } else {
            self.profile.set_count(KEY_SEEDS, have - p.price);
            self.profile.set_count(key_pack(p.kind), packed + 1);
            self.write_profile();
            self.village_ui.say = Some(Say::thanks(|s| s.pack_thanks, 2.5));
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
        self.write_profile();
        self.village_ui.say = Some(Say::plain(|s| s.pack_back, 2.5));
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
        let tabs = self.village_ui.tabs;
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
            let on = i == self.village_ui.tab;
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
        // 왼쪽 위 포포 + 위쪽 말 칸(사장 2026-11-02: 초상화는 왼쪽 위, 대화는 위). 고맙다는 한마디를 하는 동안 웃는다.
        let face = Rect::new(PANEL.x + 6.0, PANEL.y + 21.0, 36.0, 36.0);
        ui.rect(face, ui::PANEL_LIGHT);
        let happy = self.village_ui.say.is_some_and(|m| m.happy);
        let frame = cutscene::Portrait::POPO.frame(if happy { Face::Happy } else { Face::Normal });
        self.renderer.draw_portrait(ui, face, frame);
        ui.rect_lines(face, ui::BORDER);
        let say = match self.village_ui.say {
            Some(m) => tr(m.text).to_string(),
            None => {
                let c = self.village_ctx();
                let t = &tabs[self.village_ui.tab.min(n - 1)];
                let rows = (t.rows)(&c).len();
                (t.say)(&c, self.village_ui.cursor.min(rows.saturating_sub(1)))
            }
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
        let tab = self.village_ui.tab.min(n - 1);
        // 탭 몸: 왼쪽 목록 + 오른쪽 자세히 칸 + 탭마다 얹는 것. 줄이 없으면(등불 탭에 이웃이 없을 때) 비워 둔다.
        let t = &tabs[tab];
        let (rows, picked) = {
            let c = self.village_ctx();
            let rows = (t.rows)(&c);
            let i = self.village_ui.cursor.min(rows.len().saturating_sub(1));
            let d = (!rows.is_empty()).then(|| (t.detail)(&c, i));
            (rows, d.map(|d| (i, d)))
        };
        if let Some((i, d)) = picked {
            let list = Rect::new(body.x, body.y, LIST_W, body.h - 4.0);
            self.draw_village_list(ui, list, &rows);
            let detail = Rect::new(
                list.right() + 8.0,
                body.y,
                body.right() - list.right() - 8.0,
                body.h,
            );
            let y = self.draw_village_detail(ui, detail, d.pic, &d.name, &d.info);
            if let Some(extra) = t.extra {
                extra(self, ui, list, detail, y, i);
            }
        }
        // 아래 안내: 탭마다 동사가 다르다.
        let hint = (t.hint)();
        let hint = ui.fit(&hint, PANEL.w - 8.0, FONT_PX);
        ui.text_center(&hint, 160.0, PANEL.bottom() + 1.0, FONT_PX, ui::TEXT_DIM);
    }

    /// 왼쪽 목록 한 벌. 고른 줄이 보이게 굴린다(`ListView`).
    fn draw_village_list(&mut self, ui: &Ui, r: Rect, rows: &[Row]) {
        let fits = ((r.h / ROW_H).floor() as usize).max(1);
        let n = rows.len();
        let first = self
            .village_ui
            .view
            .window(self.village_ui.cursor, n, fits, ROW_H);
        let t = get_time() as f32;
        for (k, row) in rows.iter().enumerate().skip(first).take(fits) {
            let (pic, name, right, dim) = (&row.pic, &row.name, &row.right, &row.dim);
            let row = Rect::new(r.x, r.y + (k - first) as f32 * ROW_H, r.w, ROW_H - 1.0);
            self.hits.push((row, Hit::ShopItem(k)));
            let on = k == self.village_ui.cursor;
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

    /// 수첩: 이웃은 그림 칸에 초상화.
    fn draw_notes_extra(&mut self, ui: &Ui, detail: Rect, i: usize) {
        if let Some(who) = note_at(&self.profile, i).spot.who() {
            let art = Rect::new(detail.x + 4.0, detail.y + 4.0, 36.0, 36.0);
            let p = cutscene::Portrait::of(who);
            self.renderer.draw_portrait(ui, art, p.frame(Face::Normal));
        }
    }

    /// 등불: 가진 반딧불(큰 나무 창과 같은 글) — 설명이 길어 자리가 없으면 뺀다(마을 화면 오른쪽 위에도 보인다).
    fn draw_lamp_extra(&mut self, ui: &Ui, detail: Rect, y: f32) {
        let have = self.profile.count(crate::talents::KEY_LIGHT);
        if y + 12.0 <= detail.bottom() {
            ui.text(
                &fill(i18n::s().tree_light, &[&have]),
                detail.x,
                y,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
    }

    /// 텃밭: 거둬 둔 산딸기(다음 길 가방에), 마을에 옮겨 심은 꽃.
    fn draw_garden_extra(&mut self, ui: &Ui, detail: Rect, y: f32) {
        use crate::garden;
        let s = i18n::s();
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

    /// 짐: 목록 아래 "다음 모험에 가져갈 것" 한 줄 + 자세히 칸의 − 수 + 단추.
    fn draw_pack_extra(&mut self, ui: &Ui, list: Rect, detail: Rect, y: f32, i: usize) {
        let s = i18n::s();
        // 목록 아래: 다음 모험에 가져갈 것 한 줄.
        let packed: Vec<String> = PACK
            .iter()
            .filter_map(|p| {
                let n = self.profile.count(key_pack(p.kind));
                (n > 0).then(|| format!("{} ×{n}", pack_name(p)))
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
        let p = &PACK[i];
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

    // ---- 이웃 등불(0304) ----------------------------------------------------

    /// 등불을 건넨다: 반딧불을 내고, 창을 닫고, 그 이웃의 이야기(컷씬)를 튼다. 셋째면 곁에 등불이 걸린다.
    fn give_lamp(&mut self, i: usize) {
        use crate::lanterns::{give, next, Next, ALL};
        let ch = self.world.character;
        let have = self.profile.count(crate::talents::KEY_LIGHT);
        let stage = match next(&self.profile, i, ch) {
            Next::Ready { stage, cost } if have >= cost => stage,
            Next::Ready { .. } => {
                self.village_ui.say = Some(Say::plain(|s| s.lamp_poor, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
            Next::Done => {
                self.village_ui.say = Some(Say::plain(|s| s.lamp_done, 2.5));
                return;
            }
            Next::Later => {
                self.village_ui.say = Some(Say::plain(|s| s.lamp_later, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
            Next::Other => {
                self.village_ui.say = Some(Say::plain(|s| s.lamp_other, 2.5));
                self.sfx.play(crate::sfx::Id::Deny);
                return;
            }
        };
        // 개발 시나리오도 기록 사본에는 적는다(슬롯에는 쓰지 않는다 — sandbox, 타이틀로 돌아오면 슬롯을 다시 읽는다, 0269 B3).
        give(&mut self.profile, i, ch);
        if self.recording() {
            self.save_profile();
        }
        // 셋째면 곁에 등불, 맡긴 반딧불이 줄었으니 우물 위 반딧불·큰 나무 빛도 — 모두 기록에서(R41).
        self.sync_village();
        self.sfx.play(crate::sfx::Id::Buy);
        self.village_ui.say = None;
        self.scene = Scene::Game;
        self.play(cutscene::lamp_scene(
            &self.world,
            tr(ALL[i].name),
            ALL[i].scenes[stage].title,
            &crate::lanterns::lines(i, stage),
        ));
    }

    /// 텃밭 탭에서 고른 칸에 한 일: 빈 칸에서 좌우 = 심을 씨앗 바꾸기, 확인 = 심기 / 거두기(다 자란 칸) / 기다리라는 말(자라는 칸).
    /// 개발 시나리오는 저장하지 않는다(이번 세션의 기록 사본에만 — 다시 열면 그대로 보인다).
    fn garden_act(&mut self, i: usize, step: Step) {
        use crate::garden::{self, Crop, No, Plot};
        let plot = garden::plot(&self.profile, i);
        if step != Step::Pick {
            if plot == Plot::Empty {
                self.village_ui.crop = (self.village_ui.crop + 1) % Crop::ALL.len();
                self.village_ui.say = None;
                self.sfx.play(crate::sfx::Id::UiMove);
            }
            return;
        }
        let crop = Crop::ALL[self.village_ui.crop % Crop::ALL.len()];
        let done = match plot {
            Plot::Empty => match garden::plant(&mut self.profile, i, crop) {
                Ok(()) => Some((|s: &i18n::Strings| s.garden_planted) as Tr),
                Err(No::Poor) => {
                    self.village_ui.say = Some(Say::plain(|s| s.decor_poor, 3.0));
                    None
                }
                Err(No::Busy) => None,
            },
            Plot::Growing { .. } => {
                self.village_ui.say = Some(Say::plain(|s| s.garden_wait, 2.5));
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
        self.sync_village();
        if self.recording() {
            self.save_profile();
        }
        self.village_ui.say = Some(Say::thanks(msg, 3.0));
        self.sfx.play(crate::sfx::Id::Buy);
    }
}

// ---- 탭의 순수한 쪽: 기록 → 줄 목록·자세히 칸·말 --------------------------------

fn decor_can_buy(p: &Profile, i: usize) -> bool {
    decor_price(i).is_some_and(|price| !p.flag(&key_decor(i)) && p.count(KEY_SEEDS) >= price)
}

fn decor_list(c: &Ctx) -> Vec<Row> {
    let s = i18n::s();
    let have = c.p.count(KEY_SEEDS);
    decor_rows()
        .iter()
        .map(|&i| {
            let owned = c.p.flag(&key_decor(i));
            let price = decor_price(i);
            let right = match price {
                _ if owned => s.decor_owned.to_string(),
                None => s.decor_journal.to_string(),
                Some(p) => fill(s.shop_price, &[&p]),
            };
            let dim = !owned && price.is_none_or(|p| have < p);
            Row {
                pic: VillagePic::Decor(i),
                name: tr(decor_name(i)).to_string(),
                right,
                dim,
            }
        })
        .collect()
}

fn decor_detail(c: &Ctx, at: usize) -> Detail {
    let s = i18n::s();
    let i = decor_rows()[at];
    // 이름 아래 한 줄: 놓았다 / 발자국 보상 / 값.
    let info = if c.p.flag(&key_decor(i)) {
        s.decor_owned.to_string()
    } else if let Some(p) = decor_price(i) {
        fill(s.shop_price, &[&p])
    } else {
        s.decor_journal.to_string()
    };
    Detail {
        pic: VillagePic::Decor(i),
        name: tr(decor_name(i)).to_string(),
        info,
    }
}

/// 꾸미기 탭에서 포포가 하는 말: 고른 꾸미기의 설명(발자국 꾸미기는 얻는 곳까지).
fn decor_say(c: &Ctx, at: usize) -> String {
    let s = i18n::s();
    let i = decor_rows()[at];
    if decor_price(i).is_none() && !c.p.flag(&key_decor(i)) {
        format!("{} {}", tr(decor_desc(i)), s.decor_journal_d)
    } else {
        tr(decor_desc(i)).to_string()
    }
}

fn pack_name(p: &crate::village::Pack) -> String {
    if p.kind == PackKind::Pouch {
        fill(tr(p.name), &[&PACK_POUCH_SEEDS])
    } else {
        tr(p.name).to_string()
    }
}

fn pack_list(c: &Ctx) -> Vec<Row> {
    let s = i18n::s();
    let have = c.p.count(KEY_SEEDS);
    PACK.iter()
        .map(|p| {
            let n = c.p.count(key_pack(p.kind));
            // 오른쪽은 값만 — 싼 수는 자세히 칸의 n/최대와 목록 아래 줄에(●○ 점은 뺐다, 사장 2026-11-23).
            Row {
                pic: VillagePic::Pack(p.kind),
                name: pack_name(p),
                right: fill(s.shop_price, &[&p.price]),
                dim: n >= p.max || have < p.price,
            }
        })
        .collect()
}

fn pack_detail(_: &Ctx, i: usize) -> Detail {
    let p = &PACK[i];
    Detail {
        pic: VillagePic::Pack(p.kind),
        name: pack_name(p),
        info: fill(i18n::s().shop_price, &[&p.price]),
    }
}

/// 수첩 탭에서 고른 줄(표 번호).
fn note_at(p: &Profile, i: usize) -> &'static crate::village_notes::Note {
    let vis = crate::village_notes::visible(p);
    &crate::village_notes::NOTES[vis[i.min(vis.len() - 1)]]
}

fn notes_list(c: &Ctx) -> Vec<Row> {
    use crate::village_notes::NOTES;
    crate::village_notes::visible(c.p)
        .iter()
        .map(|&i| {
            let n = &NOTES[i];
            // 이웃 줄은 마을에 선 모습을 아이콘으로(사장 2026-11-23: 인물만 아이콘이 비어 어색했다).
            let pic = match n.spot.who() {
                Some(who) => VillagePic::Neighbor(who),
                None => VillagePic::Spot(n.spot),
            };
            Row {
                pic,
                name: tr(n.title).to_string(),
                right: String::new(),
                dim: false,
            }
        })
        .collect()
}

fn notes_detail(c: &Ctx, i: usize) -> Detail {
    let note = note_at(c.p, i);
    Detail {
        pic: VillagePic::Spot(note.spot),
        name: tr(note.title).to_string(),
        info: String::new(),
    }
}

/// 등불 탭의 줄: 마을에 서 있는 이웃(`lanterns::rows`).
fn lamp_list(c: &Ctx) -> Vec<Row> {
    use crate::lanterns::{level, next, Next, ALL};
    let have = c.p.count(crate::talents::KEY_LIGHT);
    crate::lanterns::rows(c.p, c.ch)
        .iter()
        .map(|&i| {
            let dim = match next(c.p, i, c.ch) {
                Next::Ready { cost, .. } => have < cost,
                _ => true,
            };
            Row {
                pic: VillagePic::Neighbor(ALL[i].who),
                name: tr(ALL[i].name).to_string(),
                right: format!("{}/3", level(c.p, i)),
                dim,
            }
        })
        .collect()
}

fn lamp_detail(c: &Ctx, at: usize) -> Detail {
    use crate::lanterns::{next, Next, ALL};
    let s = i18n::s();
    let i = crate::lanterns::rows(c.p, c.ch)[at];
    let info = match next(c.p, i, c.ch) {
        Next::Ready { stage, cost } => fill(s.lamp_next, &[&tr(ALL[i].scenes[stage].title), &cost]),
        Next::Done => s.lamp_done.to_string(),
        Next::Later => s.lamp_later.to_string(),
        Next::Other => s.lamp_other.to_string(),
    };
    Detail {
        pic: VillagePic::Neighbor(ALL[i].who),
        name: tr(ALL[i].name).to_string(),
        info,
    }
}

fn garden_list(c: &Ctx) -> Vec<Row> {
    use crate::garden::{self, Plot, GROW};
    let s = i18n::s();
    garden::plots(c.p)
        .iter()
        .map(|&p| {
            let (name, right) = match p {
                Plot::Empty => (s.garden_empty.to_string(), String::new()),
                Plot::Growing { crop, age } => {
                    (tr(crop.name()).to_string(), format!("{age}/{GROW}"))
                }
                Plot::Ripe(crop) => (tr(crop.name()).to_string(), s.garden_ripe.to_string()),
            };
            Row {
                pic: VillagePic::Plot(p),
                name,
                right,
                dim: false,
            }
        })
        .collect()
}

fn garden_detail(c: &Ctx, i: usize) -> Detail {
    use crate::garden::{self, Crop, Plot, GROW};
    let s = i18n::s();
    let plots = garden::plots(c.p);
    let crop = Crop::ALL[c.crop % Crop::ALL.len()];
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
    Detail { pic, name, info }
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

    fn ctx(p: &Profile) -> Ctx<'_> {
        Ctx {
            p,
            ch: Character::Mole,
            crop: 0,
        }
    }

    /// 탭마다(0307 R42): 어떤 기록에서도 줄마다 자세히 칸·말이 나온다(줄 번호가 어긋나 죽지 않는다).
    /// 줄 목록은 입력과 그림이 같이 쓰니, 줄 수와 자세히 칸의 짝이 맞아야 한다.
    #[test]
    fn every_tab_has_a_detail_and_a_line_for_each_row() {
        let mut profiles: Vec<Profile> = crate::story::presets()
            .into_iter()
            .map(|(_, p)| p)
            .collect();
        profiles.push(Profile::default());
        for p in &profiles {
            for ch in [Character::Mole, Character::Momo, Character::Doto] {
                for crop in 0..crate::garden::Crop::ALL.len() {
                    let c = Ctx { p, ch, crop };
                    for t in VILLAGE_TABS.iter().chain(GARDEN_TABS) {
                        let rows = (t.rows)(&c);
                        for (i, row) in rows.iter().enumerate() {
                            let d = (t.detail)(&c, i);
                            assert!(!row.name.is_empty() && !d.name.is_empty());
                            let _ = (t.say)(&c, i);
                            let _ = (t.tap_hint)(&c, i);
                        }
                        assert!(!(t.hint)().is_empty());
                    }
                }
            }
        }
    }

    /// 꾸미기: 놓은 것은 "놓았다"(흐리지 않다), 값이 모자라면 흐리고 살 수 없다, 발자국 꾸미기는 값이 없고 얻는 곳을 말한다.
    #[test]
    fn decor_rows_show_owned_poor_and_journal() {
        let s = i18n::s();
        let ids = decor_rows();
        let cheap = *ids
            .iter()
            .filter(|&&i| decor_price(i).is_some())
            .min_by_key(|&&i| decor_price(i))
            .unwrap();
        let journal = *ids.iter().find(|&&i| decor_price(i).is_none()).unwrap();
        let at = |i: usize| ids.iter().position(|&x| x == i).unwrap();
        let mut p = Profile::default();
        let rows = decor_list(&ctx(&p));
        assert_eq!(rows.len(), ids.len());
        assert!(rows.iter().all(|r| r.dim), "씨앗 0이면 다 흐리다");
        assert!(!decor_can_buy(&p, cheap));
        assert_eq!(rows[at(journal)].right, s.decor_journal);
        assert!(decor_say(&ctx(&p), at(journal)).contains(s.decor_journal_d));
        p.set_count(KEY_SEEDS, decor_price(cheap).unwrap());
        assert!(decor_can_buy(&p, cheap) && !decor_list(&ctx(&p))[at(cheap)].dim);
        assert!(!decor_can_buy(&p, journal), "발자국 꾸미기는 사지 못한다");
        p.set(&key_decor(cheap));
        let row = &decor_list(&ctx(&p))[at(cheap)];
        assert_eq!((row.right.as_str(), row.dim), (s.decor_owned, false));
        assert_eq!(decor_detail(&ctx(&p), at(cheap)).info, s.decor_owned);
        assert!(!decor_can_buy(&p, cheap), "놓은 것은 다시 사지 않는다");
    }

    /// 짐: 값이 모자라거나 한도까지 꾸렸으면 흐리다.
    #[test]
    fn pack_rows_dim_when_poor_or_full() {
        let mut p = Profile::default();
        assert!(pack_list(&ctx(&p)).iter().all(|r| r.dim));
        p.set_count(KEY_SEEDS, 999);
        assert!(pack_list(&ctx(&p)).iter().all(|r| !r.dim));
        p.set_count(key_pack(PACK[0].kind), PACK[0].max);
        let rows = pack_list(&ctx(&p));
        assert!(rows[0].dim && !rows[1].dim);
        assert_eq!(pack_detail(&ctx(&p), 0).name, rows[0].name);
    }

    /// 등불: 줄은 마을에 선 이웃, 오른쪽은 들은 수. 값이 모자라거나 건넬 수 없으면 흐리고, 설명은 네 경우마다 다르다.
    #[test]
    fn lamp_rows_follow_the_lantern_rules() {
        use crate::lanterns::{ALL, COST};
        use crate::profile::key;
        let s = i18n::s();
        let mut p = Profile::default();
        // 처음: 포포만 서 있다. 반딧불이 없어 흐리고, 설명은 첫 이야기와 값.
        let rows = lamp_list(&ctx(&p));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pic, VillagePic::Neighbor(crate::world::Who::Popo));
        assert_eq!((rows[0].right.as_str(), rows[0].dim), ("0/3", true));
        let info = lamp_detail(&ctx(&p), 0).info;
        assert!(info.contains(&COST[0].to_string()) && info.contains(tr(ALL[0].scenes[0].title)));
        p.set_count(crate::talents::KEY_LIGHT, COST[0]);
        assert!(!lamp_list(&ctx(&p))[0].dim, "값이 되면 밝다");
        // 둘째는 모모가 돌아와야(아직) — 흐리고 "나중에".
        p.set_count(ALL[0].key, 1);
        p.set_count(crate::talents::KEY_LIGHT, 9999);
        assert!(lamp_list(&ctx(&p))[0].dim);
        assert_eq!(lamp_detail(&ctx(&p), 0).info, s.lamp_later);
        // 모모가 돌아왔다: Naïl로는 들을 수 있고, 모모로 걸으면 제 이야기라 "다른 캐릭터로".
        p.set(key::MOMO_BACK);
        assert!(!lamp_list(&ctx(&p))[0].dim);
        let as_momo = Ctx {
            p: &p,
            ch: Character::Momo,
            crop: 0,
        };
        assert_eq!(lamp_detail(&as_momo, 0).info, s.lamp_other);
        // 셋 다 들었다.
        p.set_count(ALL[0].key, 3);
        let row = &lamp_list(&ctx(&p))[0];
        assert_eq!((row.right.as_str(), row.dim), ("3/3", true));
        assert_eq!(lamp_detail(&ctx(&p), 0).info, s.lamp_done);
    }

    /// 텃밭: 빈 칸은 고른 씨앗과 값, 자라는 칸은 남은 모험 수, 다 자란 칸은 무엇을 할지.
    #[test]
    fn garden_rows_show_the_three_plot_states() {
        use crate::garden::{self, Bloom, Crop, Plot, GROW};
        let s = i18n::s();
        let mut p = Profile::default();
        p.set_count(KEY_SEEDS, 99);
        let flower = Crop::Flower(Bloom::Lavender);
        garden::plant(&mut p, 0, flower).unwrap();
        garden::grow(&mut p);
        garden::plant(&mut p, 1, Crop::Berry).unwrap();
        garden::grow(&mut p);
        let rows = garden_list(&ctx(&p));
        assert_eq!(rows.len(), garden::PLOTS);
        assert_eq!(rows[0].right, s.garden_ripe);
        assert_eq!(rows[1].right, format!("1/{GROW}"));
        assert_eq!(
            (rows[2].name.as_str(), rows[2].right.as_str()),
            (s.garden_empty, "")
        );
        assert_eq!(garden_detail(&ctx(&p), 0).info, s.garden_ripe_flower);
        assert!(garden_detail(&ctx(&p), 1)
            .info
            .contains(&(GROW - 1).to_string()));
        // 빈 칸: 고른 씨앗(좌우로 바꾼다)의 다 자란 모습과 심는 값.
        for (k, crop) in Crop::ALL.iter().enumerate() {
            let c = Ctx {
                p: &p,
                ch: Character::Mole,
                crop: k,
            };
            let d = garden_detail(&c, 2);
            assert_eq!(d.pic, VillagePic::Plot(Plot::Ripe(*crop)));
            assert!(d.name.contains(tr(crop.name())) && d.info.contains(&crop.price().to_string()));
        }
    }

    /// 수첩: 이웃 줄은 그 이웃의 모습을 아이콘으로, 자세히 칸은 자리 그림(이웃이면 그 위에 초상화), 말은 수첩 글.
    #[test]
    fn notes_rows_use_neighbor_icons() {
        use crate::village_notes::NOTES;
        let (_, p) = crate::story::presets().pop().unwrap();
        let vis = crate::village_notes::visible(&p);
        let rows = notes_list(&ctx(&p));
        assert_eq!(rows.len(), vis.len());
        let mut neighbors = 0;
        for (i, &n) in vis.iter().enumerate() {
            let note = &NOTES[n];
            match note.spot.who() {
                Some(who) => {
                    neighbors += 1;
                    assert_eq!(rows[i].pic, VillagePic::Neighbor(who));
                }
                None => assert_eq!(rows[i].pic, VillagePic::Spot(note.spot)),
            }
            assert_eq!(notes_detail(&ctx(&p), i).pic, VillagePic::Spot(note.spot));
            assert_eq!(tr(note_at(&p, i).body), tr(note.body));
        }
        assert!(neighbors >= 3, "끝까지 간 기록이면 이웃 줄이 여럿");
    }
}
