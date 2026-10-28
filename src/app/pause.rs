//! 일시정지 메뉴의 탭 일곱(메뉴·상태·가방·저널·도감·지도·안내, 안내는 0180).

use super::*;

/// 상태 탭 글 한 줄 높이 = 위아래로 한 번 내리는 양.
pub(super) const STATUS_LINE: f32 = 12.0;
pub(super) const PAUSE_TABS: [Tr; 7] = [
    |s| s.tab_menu,
    |s| s.tab_status,
    |s| s.tab_bag,
    |s| s.tab_journal,
    |s| s.tab_bestiary,
    |s| s.tab_map,
    |s| s.pm_help,
];
/// 안내 탭 번호(0180).
pub(super) const TAB_HELP: usize = 6;
/// 지도 탭 번호(지도 버튼으로 바로 연다).
pub(super) const TAB_MAP: usize = 5;
pub(super) const PAUSE_MENU: [Tr; 4] = [
    |s| s.pm_resume,
    |s| s.pm_restart,
    |s| s.pm_settings,
    |s| s.pm_title,
];
/// 가방 칸의 발톱 아이콘: 크림/무기 색 발톱 세 개.
pub(super) fn draw_claw_icon(ui: &Ui, r: Rect, color: Color) {
    let dark = Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, 1.0);
    for k in 0..3 {
        let x = r.x + 6.0 + k as f32 * 4.0;
        ui.rect(Rect::new(x, r.y + 6.0 + (k % 2) as f32, 2.0, 8.0), color);
        ui.rect(Rect::new(x, r.y + 13.0 + (k % 2) as f32, 2.0, 2.0), dark);
    }
    ui.rect(Rect::new(r.x + 5.0, r.y + 15.0, 12.0, 2.0), dark);
}

/// 가방 칸의 장신구 아이콘: 끈에 매단 둥근 돌.
pub(super) fn draw_charm_icon(ui: &Ui, r: Rect, color: Color) {
    let dark = Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, 1.0);
    ui.rect(Rect::new(r.x + 6.0, r.y + 4.0, 10.0, 1.0), ui::BORDER);
    ui.rect(Rect::new(r.x + 10.0, r.y + 5.0, 2.0, 3.0), ui::BORDER);
    ui.rect(Rect::new(r.x + 7.0, r.y + 8.0, 8.0, 9.0), dark);
    ui.rect(Rect::new(r.x + 8.0, r.y + 8.0, 6.0, 7.0), color);
    ui.rect(
        Rect::new(r.x + 8.0, r.y + 9.0, 2.0, 2.0),
        Color::new(1.0, 1.0, 1.0, 0.8),
    );
}

impl App {
    pub(super) fn update_pause(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let n_tabs = PAUSE_TABS.len();
        // 이번 입력을 받기 전 탭: 좌우로 가방에 막 들어온 프레임에 같은 좌우가 가방 끝에서 또 탭을 넘기지 않게.
        let came_in = self.pause_tab;
        if a.cancel || a.menu {
            self.scene = Scene::Game;
            return;
        }
        if a.tab_prev {
            self.pause_tab = (self.pause_tab + n_tabs - 1) % n_tabs;
        }
        if a.tab_next {
            self.pause_tab = (self.pause_tab + 1) % n_tabs;
        }
        if let Some(Hit::PauseTab(i)) = tap_hit {
            self.pause_tab = i;
        }
        // 가로 탐색이 없는 탭에서는 좌우로도 탭을 넘긴다.
        let horizontal_tab = self.pause_tab != 2;
        if horizontal_tab && a.left {
            self.pause_tab = (self.pause_tab + n_tabs - 1) % n_tabs;
        }
        if horizontal_tab && a.right {
            self.pause_tab = (self.pause_tab + 1) % n_tabs;
        }
        // 좌우로 가방에 들어오면 들어온 쪽 끝 칸(같은 줄)에서 시작한다.
        if self.pause_tab == 2 && came_in != 2 && (a.left || a.right) {
            let cols = self.inventory.cols;
            let n = self.inventory.slots.len();
            let row_start = self.inv_cursor / cols * cols;
            self.inv_cursor = if a.right {
                row_start
            } else {
                (row_start + cols - 1).min(n - 1)
            };
        }
        match self.pause_tab {
            0 => {
                let n = PAUSE_MENU.len();
                if a.up {
                    self.pause_cursor = (self.pause_cursor + n - 1) % n;
                }
                if a.down {
                    self.pause_cursor = (self.pause_cursor + 1) % n;
                }
                let chosen = if a.confirm {
                    Some(self.pause_cursor)
                } else if let Some(Hit::PauseItem(i)) = tap_hit {
                    Some(i)
                } else {
                    None
                };
                match chosen {
                    Some(0) => self.scene = Scene::Game,
                    Some(1) => {
                        // 다시 하기·타이틀로: 런이 끝난다. 들고 있던 반딧불은 마을로(0047) — 굴 안이면 잃는다(0210).
                        if self.safe_to_bank() {
                            self.bank_glowbugs();
                        }
                        self.start_game();
                    }
                    Some(2) => {
                        self.settings_cursor = 0;
                        self.scene = Scene::Settings { back: Back::Pause };
                    }
                    Some(3) => {
                        if self.safe_to_bank() {
                            self.bank_glowbugs();
                        }
                        // 쉼터에서 나가면 그 쉼터부터 이어한다(굴 안이면 마지막 쉼터, 0080).
                        self.checkpoint_run();
                        self.title_cursor = 0;
                        self.scene = Scene::Title;
                    }
                    _ => {}
                }
            }
            2 => {
                let cols = self.inventory.cols;
                let n = self.inventory.slots.len();
                // 좌우 끝에서 한 번 더 밀면 옆 탭으로(다른 탭은 좌우가 곧 탭 넘기기라 같은 손으로 빠져나온다).
                let lr = came_in == 2;
                if lr && a.left {
                    if self.inv_cursor.is_multiple_of(cols) {
                        self.pause_tab = (self.pause_tab + n_tabs - 1) % n_tabs;
                        return;
                    }
                    self.inv_cursor -= 1;
                }
                if lr && a.right {
                    if self.inv_cursor % cols == cols - 1 || self.inv_cursor + 1 >= n {
                        self.pause_tab = (self.pause_tab + 1) % n_tabs;
                        return;
                    }
                    self.inv_cursor += 1;
                }
                if a.up && self.inv_cursor >= cols {
                    self.inv_cursor -= cols;
                }
                if a.down && self.inv_cursor + cols < n {
                    self.inv_cursor += cols;
                }
                let mut equip = a.confirm;
                if let Some(Hit::InvSlot(i)) = tap_hit {
                    equip = self.inv_cursor == i;
                    self.inv_cursor = i;
                }
                if equip {
                    match self.inventory.slots[self.inv_cursor] {
                        Some(Item::Weapon(w)) => {
                            // 캐릭터가 쓸 수 있는 종류만.
                            if WEAPONS[w].kind == self.world.character.weapon_kind() {
                                self.inventory.equipped = Some(w);
                                self.apply_equipped();
                                self.sfx.play(crate::sfx::Id::Claw(2));
                            }
                        }
                        Some(Item::Charm(c)) => {
                            // 끼고 있으면 빼고, 아니면 낀다. 칸이 가득이면 먼저 낀 것과 바꾼다.
                            let charms = &mut self.world.inv.charms;
                            if let Some(i) = charms.iter().position(|x| *x == c) {
                                charms.remove(i);
                            } else {
                                if charms.len() >= CHARM_SLOTS {
                                    charms.remove(0);
                                }
                                charms.push(c);
                            }
                            self.world.refresh_stats();
                            self.sfx.play(crate::sfx::Id::Claw(1));
                        }
                        None => {}
                    }
                }
            }
            3 => {
                let n = crate::journal::visible(&self.profile).len().max(1);
                if a.up {
                    self.journal_cursor = (self.journal_cursor + n - 1) % n;
                }
                if a.down {
                    self.journal_cursor = (self.journal_cursor + 1) % n;
                }
                let mut take = a.confirm;
                self.journal_view
                    .scroll(self.scroll_in, &mut self.journal_cursor, n);
                if let Some(Hit::JournalItem(i)) = tap_hit {
                    // 고른 칸을 한 번 더 누르면 받는다.
                    take = i == self.journal_cursor;
                    self.journal_cursor = i;
                }
                if take {
                    self.claim_journal();
                }
            }
            1 => {
                // 글이 칸보다 길면 위아래로 한 줄씩 내린다(언어·재주·장신구에 따라 길이가 다르다).
                let mut step = 0.0;
                if a.up || tap_hit == Some(Hit::StatusScroll(-1)) {
                    step = -STATUS_LINE;
                }
                if a.down || tap_hit == Some(Hit::StatusScroll(1)) {
                    step = STATUS_LINE;
                }
                // 끌기는 그만큼, 휠·▲▼는 한 줄씩(0199).
                let (drag, steps) = self.scroll_in;
                step += drag + steps as f32 * STATUS_LINE;
                self.status_scroll = (self.status_scroll + step).clamp(0.0, self.status_max);
            }
            4 => {
                let n = crate::species::BESTIARY.len();
                if a.up {
                    self.bestiary_cursor = (self.bestiary_cursor + n - 1) % n;
                }
                if a.down {
                    self.bestiary_cursor = (self.bestiary_cursor + 1) % n;
                }
                self.bestiary_view
                    .scroll(self.scroll_in, &mut self.bestiary_cursor, n);
                if let Some(Hit::BestiaryItem(i)) = tap_hit {
                    self.bestiary_cursor = i;
                }
                // 확인(또는 설명을 누르기) = 설명 쪽 넘기기. 좌우는 일시정지에서 탭을 바꾼다.
                if a.confirm || tap_hit == Some(Hit::BestiaryPage) {
                    self.bestiary_page = (self.bestiary_page + 1) % 3;
                }
            }
            TAB_HELP => self.help_nav(a, tap_hit),
            _ => {}
        }
    }

    // ---- 그리기 ---------------------------------------------------------------

    pub(super) fn draw_pause(&mut self, ui: &Ui) {
        let panel = Rect::new(12.0, 8.0, 296.0, 156.0);
        ui.panel(panel, None);
        let tabs = ui.tabs(
            &trs(&PAUSE_TABS),
            self.pause_tab,
            panel.x + 6.0,
            panel.y + 5.0,
            panel.w - 12.0,
        );
        for (i, r) in tabs.into_iter().enumerate() {
            self.hits.push((r, Hit::PauseTab(i)));
        }
        ui.line(
            vec2(panel.x + 4.0, panel.y + 24.0),
            vec2(panel.right() - 4.0, panel.y + 24.0),
            ui::BORDER,
        );
        // 탭 안내는 패널 밖 오른쪽 아래. 탭 줄에 두면 탭 이름이 긴 언어에서 겹치고, 패널 안에 두면 내용과 겹친다.
        let s = i18n::s();
        ui.text_right(
            &device_hint(s.tab_hint, s.tab_hint_touch, &[Verb::Tabs]),
            panel.right() - 2.0,
            panel.bottom() + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let content = Rect::new(
            panel.x + 8.0,
            panel.y + 30.0,
            panel.w - 16.0,
            panel.h - 38.0,
        );
        match self.pause_tab {
            0 => {
                let rects = ui.menu_list(
                    &trs(&PAUSE_MENU),
                    self.pause_cursor,
                    content.x + 90.0,
                    content.y + 12.0,
                    100.0,
                );
                for (i, r) in rects.into_iter().enumerate() {
                    self.hits.push((r, Hit::PauseItem(i)));
                }
            }
            1 => self.draw_status(ui, content),
            2 => self.draw_inventory(ui, content),
            3 => self.draw_journal(ui, content),
            4 => self.draw_bestiary(ui, content),
            TAB_HELP => self.draw_help_body(ui, content),
            _ => self.draw_map(ui, content),
        }
    }

    /// 상태 탭: 캐릭터 능력치는 숫자로(두 칸 표), 그 아래 필살기·장신구·재주·화폐 설명.
    /// 글은 자르지 않고 줄을 바꾼다. 칸보다 길면 위아래(또는 ▲▼)로 내린다.
    pub(super) fn draw_status(&mut self, ui: &Ui, c: Rect) {
        let w = &self.world;
        let p = &w.player;
        let s = i18n::s();
        let st = p.stats;
        let gauge = w.character.gauge();
        let base_dmg = p.moveset.hits.first().map(|h| h.damage).unwrap_or(1);
        let weapon = match self.inventory.equipped.filter(|_| p.armed) {
            Some(i) => tr(WEAPONS[i].name).to_string(),
            None => s.bare_hands.to_string(),
        };
        let charms = if w.inv.charms.is_empty() {
            s.charms_none.to_string()
        } else {
            w.inv
                .charms
                .iter()
                .map(|c| tr(CHARMS[*c].name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let pct = |x: f32| fill(s.fmt_pct, &[&((x * 100.0).round() as i32)]);
        let left: [(&str, String); 5] = [
            (s.hp, fill(s.fmt_hp, &[&p.hp, &st.max_hp])),
            (s.stat_power, (base_dmg + st.damage).to_string()),
            (s.stat_aspd, pct(st.attack_speed)),
            (
                (w.character.def().stat_dodge_cd)(s),
                fill(s.fmt_secs, &[&format!("{:.1}", p.burrow_cd_full())]),
            ),
            (&fill(s.stat_gauge, &[&tr(gauge.name())]), pct(st.sp_gain)),
        ];
        let right: [(&str, String); 3] = [
            (s.stat_weapon, weapon),
            (s.glowbugs, w.inv.glowbugs.to_string()),
            (s.seeds, w.inv.seeds.to_string()),
        ];
        // 내린 만큼 위로 민 y. 칸 안에 다 들어오는 줄만 그린다.
        let off = self.status_scroll;
        let at = |y: f32| -> Option<f32> {
            let v = y - off;
            (v >= -0.5 && v + FONT_PX <= c.h + 0.5).then_some(c.y + v)
        };
        let row_h = FONT_PX + 1.0;
        let col_w = c.w / 2.0 - 4.0;
        // 이름 칸은 값이 남긴 만큼까지 넓힌다(55%로 고정하면 일본어 "いかりのたまり"가 잘렸다, 0089).
        let key_w_of = |rows: &[(&str, String)]| {
            let val = rows
                .iter()
                .map(|(_, v)| ui.text_w(v, FONT_PX))
                .fold(0.0, f32::max);
            rows.iter()
                .map(|(k, _)| ui.text_w(k, FONT_PX))
                .fold(0.0, f32::max)
                .min((col_w - val - 10.0).max(col_w * 0.55))
                + 6.0
        };
        for (col, rows) in [(0usize, &left[..]), (1, &right[..])] {
            let x = c.x + col as f32 * (col_w + 8.0);
            let key_w = key_w_of(rows);
            for (i, (k, v)) in rows.iter().enumerate() {
                let Some(y) = at(i as f32 * row_h) else {
                    continue;
                };
                ui.text(&ui.fit(k, key_w - 6.0, FONT_PX), x, y, FONT_PX, ui::ACCENT);
                ui.text(
                    &ui.fit(v, col_w - key_w, FONT_PX),
                    x + key_w,
                    y,
                    FONT_PX,
                    ui::TEXT,
                );
            }
        }
        // 체력 하트(숫자 옆에 그림으로도). 숫자 바로 뒤에 둔다: 고정 자리면 일본어처럼 이름 칸이 넓은 언어에서 숫자를 덮었다.
        if let Some(hy) = at(0.0) {
            let hx0 = c.x + key_w_of(&left) + ui.text_w(&left[0].1, FONT_PX) + 5.0;
            for i in 0..st.max_hp.min(10) {
                let col = if i < p.hp {
                    Color::new(0.93, 0.33, 0.40, 1.0)
                } else {
                    ui::TEXT_DIM
                };
                ui.rect(Rect::new(hx0 + i as f32 * 5.0, hy + 2.0, 4.0, 3.0), col);
                ui.rect(
                    Rect::new(hx0 + i as f32 * 5.0 + 1.0, hy + 5.0, 2.0, 1.0),
                    col,
                );
            }
        }
        let mut y = 5.0 * row_h + 4.0;
        if let Some(ry) = at(y) {
            ui.line(vec2(c.x, ry - 2.0), vec2(c.right(), ry - 2.0), ui::BORDER);
        }
        // 이름(색) + 설명(이름 뒤 칸에 줄 바꿈). 필살기·장신구가 쓴다.
        let labeled = |y: &mut f32, name: &str, color: Color, body: &str| {
            if let Some(ly) = at(*y) {
                ui.text(name, c.x, ly, FONT_PX, color);
            }
            let dx = c.x + ui.text_w(name, FONT_PX) + 6.0;
            let lines = ui::wrap(ui, body, c.right() - dx, FONT_PX);
            for l in &lines {
                if let Some(ly) = at(*y) {
                    ui.text(l, dx, ly, FONT_PX, ui::TEXT);
                }
                *y += STATUS_LINE;
            }
            if lines.is_empty() {
                *y += STATUS_LINE;
            }
            *y += 2.0;
        };
        let desc = fill(
            s.special_desc,
            &[
                &tr(gauge.name()),
                &ui::key(crate::input::prompt(crate::input::Act::Special)),
            ],
        );
        labeled(&mut y, s.special_name, gauge.color(), &desc);
        labeled(&mut y, s.charms_label, ui::ACCENT, &charms);
        // 낀 재주(0047): 자리 순서로. 없으면 어디서 깨우는지.
        let picked: Vec<&str> = crate::talents::table(self.world.character)
            .iter()
            .zip(p.talents.pick)
            .filter(|(_, side)| *side > 0)
            .map(|((_, sides), side)| tr(sides[side as usize - 1].name))
            .collect();
        let talents = if picked.is_empty() {
            s.tal_none.to_string()
        } else {
            picked.join(" · ")
        };
        let block = |y: &mut f32, text: &str, color: Color| {
            for l in ui::wrap(ui, text, c.w, FONT_PX) {
                if let Some(ly) = at(*y) {
                    ui.text(&l, c.x, ly, FONT_PX, color);
                }
                *y += STATUS_LINE;
            }
            *y += 2.0;
        };
        block(&mut y, &fill(s.stat_talents, &[&talents]), ui::ACCENT);
        // 화폐 설명.
        block(&mut y, s.wallet_note, ui::TEXT_DIM);
        // 다 그린 높이로 내릴 수 있는 끝을 잰다(언어를 바꾸거나 재주를 끼면 달라진다).
        self.status_max = (y - 2.0 - c.h).max(0.0);
        self.status_scroll = self.status_scroll.min(self.status_max);
        let ax = c.right() - 8.0;
        if off > 0.0 {
            ui.text("▲", ax, c.y - 9.0, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 4.0, c.y - 12.0, 16.0, 14.0),
                Hit::StatusScroll(-1),
            ));
        }
        if off < self.status_max {
            ui.text("▼", ax, c.bottom() - 3.0, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 4.0, c.bottom() - 6.0, 16.0, 14.0),
                Hit::StatusScroll(1),
            ));
        }
    }

    /// 도감 탭: 왼쪽 목록(만난 이웃만 이름, 아니면 ???), 오른쪽 그림 + 말로 된 능력 + 이야기·공격·요령.
    pub(super) fn draw_bestiary(&mut self, ui: &Ui, c: Rect) {
        use crate::species::{self, BESTIARY};
        let s = i18n::s();
        let seen = &self.world.seen;
        // 목록이 아래 "만난 이웃 n/m" 줄에 닿지 않게. 다 안 들어가면 고른 줄을 따라 넘긴다(▲▼).
        // 줄은 늘 글자 + 4(고른 칸 15px 안에 글자 11px이 가운데): 열다섯 종을 다 넣으려 13으로 줄이면 고른 칸 테두리가
        // 글자에 닿았다(9종에서 일본어가 겹친 것은 0054, 칸 높이는 사장 피드백 2026-10-28).
        let avail = c.h - FONT_PX - 6.0;
        let row_h = FONT_PX + 4.0;
        let fits = ((avail / row_h).floor() as usize).max(1);
        // 넘기면 목록 오른쪽 9px에 ▲▼(이름·설명과 겹쳤다). 목록 폭은 가장 긴 아는 이름에 맞춰 74~100
        // (더 넓히면 오른쪽 능력 말이 잘렸다. 아주 긴 이름은 목록에서만 "…", 오른쪽 제목엔 다 나온다)
        // (포르투갈어·프랑스어·러시아어 이름이 "Esquilo-listra…"처럼 잘렸다, 0089).
        // ▲▼ 폭은 글꼴에서 잰다(일본어·중국어 글꼴은 12px라 9px 칸 밖으로 나왔다, 0146).
        let arrow_w = ui.text_w("▼", FONT_PX).max(9.0);
        let arrows = if fits < BESTIARY.len() { arrow_w } else { 0.0 };
        let list_w = BESTIARY
            .iter()
            .filter(|k| seen.contains(k))
            .map(|k| ui.text_w(tr(k.species().name), FONT_PX) + 8.0 + arrows)
            .fold(74.0, f32::max)
            .min(100.0);
        let first = self
            .bestiary_view
            .window(self.bestiary_cursor, BESTIARY.len(), fits, row_h);
        let ax = c.x + list_w - arrow_w;
        if first > 0 {
            ui.text("▲", ax, c.y + 1.0, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 3.0, c.y - 1.0, arrow_w + 6.0, row_h + 2.0),
                Hit::Scroll(-1),
            ));
        }
        if first + fits < BESTIARY.len() {
            let y = c.y + (fits - 1) as f32 * row_h + 1.0;
            ui.text("▼", ax, y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 3.0, y - 2.0, arrow_w + 6.0, row_h + 2.0),
                Hit::Scroll(1),
            ));
        }
        for (i, k) in BESTIARY.iter().enumerate().skip(first).take(fits) {
            let r = Rect::new(c.x, c.y + (i - first) as f32 * row_h, list_w, row_h - 1.0);
            let known = seen.contains(k);
            let hinted = *k == crate::enemy::Kind::Momo && self.world.story.petal_found();
            let name = if known {
                tr(k.species().name)
            } else {
                s.best_unknown
            };
            let on = i == self.bestiary_cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
                ui.rect_lines(r, ui::ACCENT);
            }
            ui.text(
                &ui.fit(name, list_w - 6.0 - arrows, FONT_PX),
                r.x + 3.0,
                ui::mid_y(r),
                FONT_PX,
                if on {
                    ui::ACCENT
                } else if known || hinted {
                    ui::TEXT
                } else {
                    ui::TEXT_DIM
                },
            );
            self.hits.push((r, Hit::BestiaryItem(i)));
        }
        let n_seen = BESTIARY.iter().filter(|k| seen.contains(k)).count();
        ui.text(
            &fill(s.best_count, &[&n_seen, &BESTIARY.len()]),
            c.x,
            c.bottom() - FONT_PX,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let k = BESTIARY[self.bestiary_cursor.min(BESTIARY.len() - 1)];
        let x = c.x + list_w + 8.0;
        let w = c.right() - x;
        if !seen.contains(&k) {
            // 시든 꽃잎(0164)을 주웠으면 모모 칸엔 물음만 선다.
            let text = if k == crate::enemy::Kind::Momo && self.world.story.petal_found() {
                s.best_petal
            } else {
                s.best_unseen
            };
            for (i, l) in ui::wrap(ui, text, w - 6.0, FONT_PX).iter().enumerate() {
                ui.text(l, x, c.y + 20.0 + i as f32 * 12.0, FONT_PX, ui::TEXT_DIM);
            }
            return;
        }
        let sp = k.species();
        // 그림: 2배 정수 배율
        let pic = Rect::new(x, c.y, 34.0, 34.0);
        ui.rect(pic, Color::new(0.10, 0.08, 0.12, 0.9));
        ui.rect_lines(pic, ui::BORDER);
        let inner = Rect::new(pic.x + 1.0, pic.y + 1.0, 32.0, 32.0);
        let frame = ((get_time() * 2.0) as u32) % 2;
        if k == crate::enemy::Kind::Chief {
            self.renderer.draw_chief_icon(ui, inner, frame);
        } else {
            self.renderer.draw_critter(ui, inner, sp.sheet_row, frame);
        }
        let tx = pic.right() + 6.0;
        ui.text(tr(sp.name), tx, c.y, FONT_PX, ui::ACCENT);
        let (hp, pow, spd) = species::words(sp);
        let words = [(s.best_hp, hp), (s.best_pow, pow), (s.best_spd, spd)];
        for (i, (label, v)) in words.iter().enumerate() {
            let y = c.y + 13.0 + i as f32 * 12.0;
            let lw = words
                .iter()
                .map(|(l, _)| ui.text_w(l, FONT_PX))
                .fold(0.0, f32::max)
                + 6.0;
            ui.text(
                &ui.fit(label, lw - 2.0, FONT_PX),
                tx,
                y,
                FONT_PX,
                ui::TEXT_DIM,
            );
            ui.text(
                &ui.fit(tr(*v), c.right() - tx - lw, FONT_PX),
                tx + lw,
                y,
                FONT_PX,
                ui::TEXT,
            );
        }
        // 설명은 쪽 하나에 한 갈래(이야기·공격·요령)를 칸 끝까지. 그래도 넘치면 마지막 줄에 "…".
        let y0 = c.y + 49.0;
        let page = self.bestiary_page.min(2);
        let (label, text) = [
            (s.best_lore, sp.lore),
            (s.best_atk, sp.attack),
            (s.best_tip, sp.tip),
        ][page];
        ui.text(label, x, y0, FONT_PX, ui::ACCENT);
        ui.text_right(
            &format!("{}/3 · {}", page + 1, crate::input::verb(Verb::Confirm)),
            c.right(),
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let top = y0 + 13.0;
        let rows = (((c.bottom() + 2.0 - top) / 12.0).floor() as usize).max(1);
        // 줄바꿈 폭은 6px 덜: 일본어·간체 금칙이 앞 줄로 당긴 글자가 칸을 넘었다(0089).
        let lines = ui::wrap(ui, tr(text), w - 6.0, FONT_PX);
        for (i, l) in lines.iter().take(rows).enumerate() {
            let l = if i + 1 == rows && lines.len() > rows {
                ui.fit(&format!("{l}…"), w, FONT_PX)
            } else {
                l.clone()
            };
            ui.text(&l, x, top + i as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        self.hits
            .push((Rect::new(x, y0, w, c.bottom() - y0), Hit::BestiaryPage));
    }

    pub(super) fn draw_inventory(&mut self, ui: &Ui, c: Rect) {
        let cols = self.inventory.cols;
        let slot = 22.0;
        let gap = 4.0;
        let x0 = c.x + 8.0;
        let y0 = c.y + 2.0;
        for (i, item) in self.inventory.slots.iter().enumerate() {
            let (col, row) = (i % cols, i / cols);
            let r = Rect::new(
                x0 + col as f32 * (slot + gap),
                y0 + row as f32 * (slot + gap),
                slot,
                slot,
            );
            ui.rect(r, ui::PANEL_LIGHT);
            ui.rect_lines(
                r,
                if i == self.inv_cursor {
                    ui::ACCENT
                } else {
                    ui::BORDER
                },
            );
            match item {
                Some(Item::Weapon(w)) => {
                    draw_claw_icon(ui, r, WEAPONS[*w].color);
                    if self.inventory.equipped == Some(*w) {
                        ui.text("E", r.x + 2.0, r.y + 1.0, FONT_PX, ui::ACCENT);
                    }
                }
                Some(Item::Charm(c)) => {
                    draw_charm_icon(ui, r, CHARMS[*c].color);
                    if self.world.inv.charms.contains(c) {
                        ui.text("E", r.x + 2.0, r.y + 1.0, FONT_PX, ui::ACCENT);
                    }
                }
                None => {}
            }
            self.hits.push((r, Hit::InvSlot(i)));
        }
        let s = i18n::s();
        let desc_x = x0 + cols as f32 * (slot + gap) + 8.0;
        let desc_w = c.right() - desc_x;
        match &self.inventory.slots[self.inv_cursor] {
            Some(Item::Weapon(w)) => {
                let wp = &WEAPONS[*w];
                ui.text(tr(wp.name), desc_x, y0, FONT_PX, ui::ACCENT);
                let mut y = y0 + 16.0;
                for l in ui::wrap(ui, tr(wp.desc), desc_w, FONT_PX).iter() {
                    ui.text(l, desc_x, y, FONT_PX, ui::TEXT);
                    y += 14.0;
                }
                let ms = wp.moveset;
                let last = &ms.hits[ms.last()];
                // 언어마다 길이가 달라 칸 폭에서 두 줄까지(한 줄로 자르면 영어·포르투갈어에서 "마지막 타 피해"가 잘렸다, 0084·0089).
                let combo = fill(s.bag_combo, &[&ms.hits.len(), &last.damage]);
                let lines = ui::wrap(ui, &combo, desc_w, FONT_PX);
                for (k, l) in lines.iter().take(2).enumerate() {
                    ui.text(l, desc_x, y + 4.0 + k as f32 * 13.0, FONT_PX, ui::TEXT_DIM);
                }
                let hint = if self.inventory.equipped == Some(*w) {
                    s.equipped.to_string()
                } else {
                    vfill(s.equip_hint, &[Verb::Confirm])
                };
                let extra = 13.0 * (lines.len().clamp(1, 2) - 1) as f32;
                ui.text(&hint, desc_x, y + 18.0 + extra, FONT_PX, ui::ACCENT);
            }
            Some(Item::Charm(c)) => {
                let ch = &CHARMS[*c];
                ui.text(tr(ch.name), desc_x, y0, FONT_PX, ui::ACCENT);
                let mut y = y0 + 16.0;
                for l in ui::wrap(ui, tr(ch.desc), desc_w, FONT_PX).iter() {
                    ui.text(l, desc_x, y, FONT_PX, ui::TEXT);
                    y += 14.0;
                }
                ui.text(
                    &fill(s.charm_slots, &[&self.world.inv.charms.len(), &CHARM_SLOTS]),
                    desc_x,
                    y + 4.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
                let hint = if self.world.inv.charms.contains(c) {
                    s.charm_unequip
                } else {
                    s.charm_equip
                };
                let hint = vfill(hint, &[Verb::Confirm]);
                ui.text(&hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
            }
            None => {
                ui.text(s.empty_slot, desc_x, y0, FONT_PX, ui::TEXT_DIM);
                for (i, l) in ui::wrap(ui, s.empty_slot_desc, desc_w, FONT_PX)
                    .iter()
                    .enumerate()
                {
                    ui.text(
                        l,
                        desc_x,
                        y0 + 16.0 + i as f32 * 14.0,
                        FONT_PX,
                        ui::TEXT_DIM,
                    );
                }
            }
        }
        ui.text(
            &fill(
                s.slots,
                &[
                    &self.inventory.slots.iter().filter(|x| x.is_some()).count(),
                    &self.inventory.slots.len(),
                ],
            ),
            x0,
            y0 + 3.0 * (slot + gap) + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        // 퀵 아이템(가방 칸과 따로, HUD 왼쪽 아래와 같은 것)
        let bag = &self.world.inv.bag;
        ui.text(
            &ui.fit(
                &fill(
                    s.bag_items,
                    &[&bag.berries, &t::BERRY_MAX, &bag.worms, &t::WORM_MAX],
                ),
                desc_x - x0 - 4.0,
                FONT_PX,
            ),
            x0,
            y0 + 3.0 * (slot + gap) + 16.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    /// 저널 탭(0048): 왼쪽 목록(◆ 받을 수 있음, ■ 받았음, □ 하는 중), 오른쪽 갈래·글·보상·상태.
    /// 저널(0048, 모양은 0173): 왼쪽은 목록(받을 것 → 하는 중 → 받은 것, 갈래 띠·상태 표시·진행 막대), 오른쪽은 고른 목표
    /// (갈래·제목·설명·보상 아이콘·진행 막대 또는 받기 상자·어디 있나). 머리에 이룬 수와 받을 것.
    pub(super) fn draw_journal(&mut self, ui: &Ui, c: Rect) {
        use crate::journal::{visible, Reward};
        let s = i18n::s();
        let p = &self.profile;
        let goals = visible(p);
        if goals.is_empty() {
            return;
        }
        self.journal_cursor = self.journal_cursor.min(goals.len() - 1);
        let blink = ((macroquad::time::get_time() * 2.5) as i64) % 2 == 0;
        let list_w = 150.0;
        // 머리: 이룸 n/m · 받을 것 k(깜박).
        let done = goals.iter().filter(|g| g.claimed(p)).count();
        let waiting = goals.iter().filter(|g| g.ready(p)).count();
        ui.text(
            &fill(s.j_done, &[&done, &goals.len()]),
            c.x,
            c.y,
            FONT_PX,
            ui::TEXT_DIM,
        );
        if waiting > 0 {
            let done_w = ui.text_w(&fill(s.j_done, &[&done, &goals.len()]), FONT_PX);
            ui.text(
                &ui.fit(
                    &fill(s.j_waiting, &[&waiting]),
                    list_w - done_w - 22.0,
                    FONT_PX,
                ),
                c.x + done_w + 8.0,
                c.y,
                FONT_PX,
                if blink { ui::ACCENT } else { ui::TEXT },
            );
        }
        let top = c.y + 15.0;
        let row_h = FONT_PX + 4.0;
        let rows = (((c.bottom() - top) / row_h).floor() as usize).max(1);
        let first = self
            .journal_view
            .window(self.journal_cursor, goals.len(), rows, row_h);
        let bar_bg = Color::new(0.22, 0.17, 0.24, 1.0);
        let bar_w = 24.0;
        for (k, g) in goals.iter().enumerate().skip(first).take(rows) {
            let r = Rect::new(c.x, top + (k - first) as f32 * row_h, list_w, row_h);
            let selected = k == self.journal_cursor;
            let claimed = g.claimed(p);
            let ready = g.ready(p);
            if selected {
                ui.rect(r, ui::PANEL_LIGHT);
            }
            // 갈래 띠(왼쪽 2px).
            ui.rect(Rect::new(r.x, r.y + 1.0, 2.0, r.h - 2.0), g.cat.color());
            // 상태: 받을 것 = 채운 마름모(깜박), 받음 = 체크, 하는 중 = 빈 네모.
            let (cx, cy) = (r.x + 9.0, r.y + r.h / 2.0);
            if claimed {
                ui.line(vec2(cx - 3.0, cy), vec2(cx - 1.0, cy + 2.0), ui::TEXT_DIM);
                ui.line(
                    vec2(cx - 1.0, cy + 2.0),
                    vec2(cx + 3.0, cy - 2.0),
                    ui::TEXT_DIM,
                );
            } else if ready {
                let col = if blink { ui::ACCENT } else { ui::TEXT };
                ui.tri(
                    vec2(cx, cy - 4.0),
                    vec2(cx + 4.0, cy),
                    vec2(cx - 4.0, cy),
                    col,
                );
                ui.tri(
                    vec2(cx - 4.0, cy),
                    vec2(cx + 4.0, cy),
                    vec2(cx, cy + 4.0),
                    col,
                );
            } else {
                ui.rect_lines(Rect::new(cx - 3.0, cy - 3.0, 6.0, 6.0), ui::TEXT_DIM);
            }
            let col = if claimed {
                ui::TEXT_DIM
            } else if selected || ready {
                ui::TEXT
            } else {
                Color::new(0.78, 0.72, 0.64, 1.0)
            };
            let title = ui.fit(tr(g.title), r.w - 16.0 - bar_w - 8.0, FONT_PX);
            ui.text(&title, r.x + 16.0, r.y + 2.0, FONT_PX, col);
            // 진행 막대(받은 것은 없다).
            if !claimed {
                let b = Rect::new(r.right() - 3.0 - bar_w, cy - 1.0, bar_w, 3.0);
                ui.rect(b, bar_bg);
                let k = (g.progress(p) as f32 / g.goal.max(1) as f32).min(1.0);
                ui.rect(
                    Rect::new(b.x, b.y, (b.w * k).round(), b.h),
                    if ready { ui::ACCENT } else { ui::TEXT_DIM },
                );
            }
            self.hits.push((r, Hit::JournalItem(k)));
        }
        // 목록이 더 있으면 ▲▼.
        if first > 0 {
            ui.text_right("▲", c.x + list_w, top - 11.0, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(c.x + list_w - 16.0, top - 14.0, 20.0, 14.0),
                Hit::Scroll(-1),
            ));
        }
        if first + rows < goals.len() {
            ui.text_right("▼", c.x + list_w, c.bottom() - 4.0, FONT_PX, ui::TEXT_DIM);
            // 마지막 줄 칸과 겹친다: 앞에 넣어 ▼가 이긴다.
            self.hits.insert(
                0,
                (
                    Rect::new(c.x + list_w - 16.0, c.bottom() - 6.0, 20.0, 14.0),
                    Hit::Scroll(1),
                ),
            );
        }
        // 오른쪽: 고른 목표.
        let g = goals[self.journal_cursor];
        let dx = c.x + list_w + 8.0;
        let dw = c.right() - dx;
        ui.text(tr(g.cat.name()), dx, c.y, FONT_PX, g.cat.color());
        ui.text(
            &ui.fit(tr(g.title), dw, FONT_PX),
            dx,
            c.y + 13.0,
            FONT_PX,
            ui::ACCENT,
        );
        // 줄바꿈 폭은 2px 덜: 일본어 금칙(줄머리에 못 오는 글자)이 앞 줄로 당겨져 칸을 넘쳤다(0089).
        let ww = dw - 6.0;
        let what = match g.reward {
            Reward::Light(n) => fill(s.r_light, &[&n]),
            Reward::Seeds(n) => fill(s.village_seeds, &[&n]),
            Reward::Decor(i) => fill(s.r_decor, &[&tr(crate::village::decor_name(i))]),
            Reward::Fur(n) => tr(crate::journal::fur_name(n)).to_string(),
            Reward::Charm(i) => fill(s.r_charm, &[&tr(crate::equipment::CHARMS[i].name)]),
        };
        let what = fill(s.j_reward, &[&what]);
        // 아래에서부터 쌓는다: 상태(받기 상자 / 어디 있나 두 줄 / 진행 막대) 위에 보상(아이콘 + 두 줄까지), 남은 자리에 설명(세 줄까지).
        let claimed = g.claimed(p);
        let ready = g.ready(p);
        let state_l: Vec<String> = if claimed {
            let mut l = ui::wrap(ui, tr(crate::journal::reward_place(g.reward)), ww, FONT_PX);
            if l.len() > 2 {
                l.truncate(2);
                l[1] = ui.fit(&format!("{}…", l[1]), dw, FONT_PX);
            }
            l
        } else if ready {
            vec![ui.fit(&vfill(s.j_claim, &[Verb::Confirm]), ww - 6.0, FONT_PX)]
        } else {
            vec![]
        };
        let state_h = if ready {
            17.0
        } else if claimed {
            13.0 * state_l.len() as f32
        } else {
            18.0
        };
        let what_l: Vec<String> = ui::wrap(ui, &what, ww - 11.0, FONT_PX)
            .into_iter()
            .take(2)
            .collect();
        let by = c.bottom() - state_h - 13.0 * what_l.len() as f32;
        draw_reward_icon(ui, g.reward, dx, by + 2.0);
        for (k, l) in what_l.iter().enumerate() {
            ui.text(l, dx + 11.0, by + k as f32 * 13.0, FONT_PX, ui::TEXT);
        }
        let sy = by + 13.0 * what_l.len() as f32;
        if ready {
            // 받기 상자: 확인으로 받는다(깜박이는 테두리).
            let b = Rect::new(dx, sy + 1.0, dw, 15.0);
            ui.rect(b, ui::PANEL_LIGHT);
            ui.rect_lines(b, if blink { ui::ACCENT } else { ui::BORDER });
            ui.text(&state_l[0], dx + 3.0, ui::mid_y(b), FONT_PX, ui::ACCENT);
        } else if claimed {
            for (k, l) in state_l.iter().enumerate() {
                ui.text(l, dx, sy + k as f32 * 13.0, FONT_PX, ui::TEXT_DIM);
            }
        } else {
            // 진행 막대 + 숫자.
            let prog = fill(s.j_progress, &[&g.progress(p), &g.goal]);
            ui.text_right(&prog, dx + dw, sy + 1.0, FONT_PX, ui::TEXT_DIM);
            let b = Rect::new(dx, sy + 14.0, dw, 3.0);
            ui.rect(b, bar_bg);
            let k = (g.progress(p) as f32 / g.goal.max(1) as f32).min(1.0);
            ui.rect(Rect::new(b.x, b.y, (b.w * k).round(), b.h), ui::TEXT);
        }
        let top = c.y + 29.0;
        let room = (((by - 2.0 - top) / 13.0).floor().max(1.0) as usize).min(3);
        let desc = ui::wrap(ui, tr(g.desc), ww, FONT_PX);
        for (k, l) in desc.iter().take(room).enumerate() {
            let l = if k + 1 == room && desc.len() > room {
                ui.fit(&format!("{l}…"), dw, FONT_PX)
            } else {
                l.clone()
            };
            ui.text(&l, dx, top + k as f32 * 13.0, FONT_PX, ui::TEXT);
        }
    }

    pub(super) fn draw_map(&self, ui: &Ui, c: Rect) {
        use crate::adventure::Place;
        let s = i18n::s();
        let biome = tr(self.world.biome.name());
        let title = match self.world.adv.place {
            Place::Arena => fill(s.map_floor, &[&biome]),
            Place::Field => fill(s.map_field, &[&biome]),
            Place::Camp => fill(s.map_camp, &[&self.world.adv.stage]),
            Place::Village => s.village_name.to_string(),
            Place::Deep => s.node_deepest.to_string(),
            Place::Dungeon => fill(
                s.map_dungeon,
                &[
                    &(self.world.adv.dungeon.as_ref().map_or(0, |d| d.depth) + 1),
                    &biome,
                ],
            ),
        };
        ui.text(&ui.fit(&title, c.w, FONT_PX), c.x, c.y, FONT_PX, ui::ACCENT);
        let area = Rect::new(c.x, c.y + 16.0, c.w, c.h - 32.0);
        match self.world.adv.place {
            Place::Dungeon => self.draw_dungeon_map(ui, area),
            _ => self.draw_room_map(ui, area),
        }
        let note = if self.world.adv.place == Place::Arena {
            s.map_note
        } else {
            s.map_legend
        };
        ui.text(
            &ui.fit(note, c.w, FONT_PX),
            c.x,
            area.bottom() + 3.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    /// 아레나·필드: 지금 지도 전체를 칸에 맞게 줄여서. 벽·소품·특별한 칸·적·나.
    pub(super) fn draw_room_map(&self, ui: &Ui, area: Rect) {
        use crate::map::Cell;
        let room = &self.world.room;
        let b = room.inner;
        let (bw, bh) = match &room.grid {
            Some(g) => (g.w as f32 * 16.0, g.h as f32 * 16.0),
            None => (t::VIEW_W, t::VIEW_H),
        };
        let k = (area.w / bw).min(area.h / bh);
        let origin = vec2(
            area.x + (area.w - bw * k) / 2.0,
            area.y + (area.h - bh * k) / 2.0,
        );
        let m = |p: Vec2| origin + p * k;
        let wall = Color::new(0.30, 0.22, 0.20, 1.0);
        let floor = Color::new(0.12, 0.09, 0.14, 1.0);
        ui.rect(Rect::new(origin.x, origin.y, bw * k, bh * k), wall);
        match &room.grid {
            Some(g) => {
                let cell = 16.0 * k;
                for y in 0..g.h {
                    for x in 0..g.w {
                        let col = match g.get(x, y) {
                            Cell::Wall => continue,
                            Cell::Save => Color::new(0.45, 0.95, 0.5, 1.0),
                            Cell::Entrance | Cell::Exit => Color::new(0.02, 0.02, 0.02, 1.0),
                            _ => floor,
                        };
                        let p = m(vec2(x as f32 * 16.0, y as f32 * 16.0));
                        ui.rect(Rect::new(p.x, p.y, cell + 0.05, cell + 0.05), col);
                    }
                }
            }
            None => {
                ui.rect(
                    Rect::new(origin.x + b.x * k, origin.y + b.y * k, b.w * k, b.h * k),
                    floor,
                );
            }
        }
        for p in &room.solids {
            if room.grid.is_none() {
                ui.rect(
                    Rect::new(origin.x + p.x * k, origin.y + p.y * k, p.w * k, p.h * k),
                    wall,
                );
            }
        }
        for e in &self.world.enemies {
            let p = m(e.pos);
            ui.rect(
                Rect::new(p.x - 1.0, p.y - 1.0, 2.0, 2.0),
                crate::world::enemy_color(e.kind),
            );
        }
        let p = m(self.world.player.pos);
        ui.rect(Rect::new(p.x - 1.5, p.y - 1.5, 3.0, 3.0), WHITE);
    }

    /// 던전: 들른 방만 칸으로, 문은 짧은 선. 지금 방은 강조.
    pub(super) fn draw_dungeon_map(&self, ui: &Ui, area: Rect) {
        use crate::dungeon::{RoomKind, DIRS};
        let Some(d) = &self.world.adv.dungeon else {
            return;
        };
        let (cw, ch, gap) = (30.0, 18.0, 6.0);
        let origin = vec2(area.center().x - 2.5 * (cw + gap), area.y + 2.0);
        let at =
            |r: (i32, i32)| origin + vec2(r.0 as f32 * (cw + gap), r.1 as f32 * (ch + gap) * 0.62);
        for r in d.rooms.iter().filter(|r| r.visited) {
            let p = at(r.at);
            let rect = Rect::new(p.x, p.y, cw, ch * 0.62);
            let current = r.at == self.world.adv.current_room;
            let fill_c = match r.kind {
                RoomKind::Boss => Color::new(0.45, 0.15, 0.18, 1.0),
                RoomKind::Treasure => Color::new(0.45, 0.38, 0.15, 1.0),
                RoomKind::Rest => Color::new(0.18, 0.38, 0.22, 1.0),
                _ => Color::new(0.16, 0.12, 0.18, 1.0),
            };
            ui.rect(rect, fill_c);
            ui.rect_lines(rect, if current { ui::ACCENT } else { ui::BORDER });
            for (k, (dx, dy)) in DIRS.iter().enumerate() {
                if r.links[k] {
                    let c = rect.center();
                    let e = c + vec2(
                        *dx as f32 * (cw / 2.0 + gap / 2.0),
                        *dy as f32 * (ch * 0.31 + gap * 0.31),
                    );
                    ui.line(
                        c + vec2(*dx as f32 * cw / 2.0, *dy as f32 * ch * 0.31),
                        e,
                        ui::BORDER,
                    );
                }
            }
            if current {
                let c = rect.center();
                ui.rect(Rect::new(c.x - 1.5, c.y - 1.5, 3.0, 3.0), WHITE);
            }
        }
    }
}

/// 저널 보상 아이콘(0173, 7×7 게임 px): 반딧불(빛나는 점), 마을 씨앗(싹 튼 씨앗), 꾸미기(작은 집), 털빛(색 견본), 장신구(그 색 칸).
fn draw_reward_icon(ui: &Ui, r: crate::journal::Reward, x: f32, y: f32) {
    use crate::journal::Reward;
    match r {
        Reward::Light(_) => {
            let glow = Color::new(1.0, 0.92, 0.5, 1.0);
            let dim = Color::new(0.75, 0.65, 0.3, 1.0);
            ui.rect(Rect::new(x + 2.0, y + 2.0, 3.0, 3.0), glow);
            for (dx, dy) in [(3.0, 0.0), (3.0, 6.0), (0.0, 3.0), (6.0, 3.0)] {
                ui.rect(Rect::new(x + dx, y + dy, 1.0, 1.0), dim);
            }
        }
        Reward::Seeds(_) => {
            ui.rect(
                Rect::new(x + 2.0, y + 3.0, 3.0, 4.0),
                Color::new(0.55, 0.35, 0.2, 1.0),
            );
            let leaf = Color::new(0.45, 0.75, 0.4, 1.0);
            ui.rect(Rect::new(x + 3.0, y + 1.0, 1.0, 2.0), leaf);
            ui.rect(Rect::new(x + 4.0, y, 2.0, 1.0), leaf);
        }
        Reward::Decor(_) => {
            ui.rect(
                Rect::new(x + 1.0, y + 3.0, 5.0, 4.0),
                Color::new(0.6, 0.45, 0.35, 1.0),
            );
            ui.tri(
                vec2(x, y + 3.0),
                vec2(x + 7.0, y + 3.0),
                vec2(x + 3.5, y),
                Color::new(0.75, 0.32, 0.3, 1.0),
            );
        }
        Reward::Fur(n) => {
            let fur = match n {
                1 => Color::new(0.62, 0.62, 0.66, 1.0),
                2 => Color::new(0.42, 0.28, 0.22, 1.0),
                3 => Color::new(0.86, 0.76, 0.58, 1.0),
                _ => Color::new(0.55, 0.4, 0.32, 1.0),
            };
            ui.rect(Rect::new(x, y, 7.0, 7.0), fur);
            ui.rect_lines(Rect::new(x, y, 7.0, 7.0), ui::BORDER);
        }
        Reward::Charm(i) => {
            ui.rect(
                Rect::new(x + 1.0, y + 1.0, 5.0, 5.0),
                crate::equipment::CHARMS[i].color,
            );
            ui.rect_lines(Rect::new(x, y, 7.0, 7.0), ui::BORDER);
        }
    }
}
