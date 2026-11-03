//! 앨범: 본 일러스트 고르기·크게 보기. 일러스트 뒤에 실의 기억 여섯(0273 A) — 칸은 그 조각의 그림, 크게 보면 조각 글.

use super::*;

pub(super) const ALBUM_COLS: usize = 5;
/// 앨범에 한 번에 보이는 줄. 그림이 더 많으면 굴린다(0085).
pub(super) const ALBUM_ROWS: usize = 2;

impl App {
    pub(super) fn album_seen(&self, il: crate::illust::Illust) -> bool {
        self.profile
            .flag(&crate::profile::key::illust(il.spec().file))
    }

    /// 앨범 칸 i(일러스트 다음에 실의 기억): 그림, 본 적 있나, 실의 기억이면 그 글.
    fn album_item(&self, i: usize) -> (crate::illust::Illust, bool, Option<Tr>) {
        use crate::illust::Illust;
        match i.checked_sub(Illust::ALL.len()) {
            None => (Illust::ALL[i], self.album_seen(Illust::ALL[i]), None),
            Some(k) => {
                let n = k as u8 + 1;
                let m = crate::memories::get(n);
                (
                    m.illust,
                    crate::memories::seen(&self.profile, n),
                    Some(m.text),
                )
            }
        }
    }

    fn album_len() -> usize {
        crate::illust::Illust::ALL.len() + crate::world::MEMORIES as usize
    }

    /// 앨범: 5×2 칸에서 고르고, 본 그림은 크게(패닝·입자 그대로). 확인·뒤로·탭으로 닫는다.
    pub(super) fn update_album(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        if let Some(t) = self.album_view.as_mut() {
            *t += dt;
            if a.confirm || a.cancel || a.attack || a.tap.is_some() {
                self.album_view = None;
            }
            return;
        }
        let n = Self::album_len();
        let c = self.album_cursor;
        if a.left {
            self.album_cursor = (c + n - 1) % n;
        }
        if a.right {
            self.album_cursor = (c + 1) % n;
        }
        // 위아래 = 한 줄. 끝에서 넘어가면 반대쪽 끝 줄의 같은 칸(없으면 그 줄 마지막 칸).
        let rows = n.div_ceil(ALBUM_COLS);
        let step_row = |c: usize, down: bool| {
            let (row, col) = (c / ALBUM_COLS, c % ALBUM_COLS);
            let row = if down {
                (row + 1) % rows
            } else {
                (row + rows - 1) % rows
            };
            (row * ALBUM_COLS + col).min(n - 1)
        };
        if a.up || matches!(tap_hit, Some(Hit::AlbumRow(false))) {
            self.album_cursor = step_row(c, false);
        }
        if a.down || matches!(tap_hit, Some(Hit::AlbumRow(true))) {
            self.album_cursor = step_row(c, true);
        }
        // 끌기·휠(0199): 줄 창을 굴리고 고른 칸을 같은 열의 보이는 줄로.
        let mut row = self.album_cursor / ALBUM_COLS;
        if self.album_rows.scroll(self.scroll_in, &mut row, rows) {
            self.album_cursor = (row * ALBUM_COLS + self.album_cursor % ALBUM_COLS).min(n - 1);
        }
        let chosen = if a.confirm {
            Some(self.album_cursor)
        } else if let Some(Hit::AlbumItem(i)) = tap_hit {
            self.album_cursor = i;
            Some(i)
        } else {
            None
        };
        if let Some(i) = chosen {
            if self.album_item(i).1 {
                self.album_view = Some(0.0);
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Title;
        }
    }

    pub(super) fn draw_album(&mut self, ui: &Ui) {
        let s = i18n::s();
        if let Some(t) = self.album_view {
            let (il, _, text) = self.album_item(self.album_cursor);
            self.renderer.draw_illust(ui, il, t);
            // 실의 기억은 그림 아래에 조각 글(컷씬의 내레이션처럼 아래서 위로 옅어지는 어둠 위에).
            if let Some(text) = text {
                let lines = ui::wrap(ui, tr(text), 280.0, FONT_PX);
                let top = 150.0 - lines.len() as f32 * 13.0;
                ui.rect(
                    Rect::new(0.0, top - 6.0, t::VIEW_W, t::VIEW_H - top + 6.0),
                    Color::new(0.03, 0.02, 0.05, 0.66),
                );
                let ui = &ui.with_outline();
                for (k, l) in lines.iter().enumerate() {
                    ui.text_center(
                        l,
                        160.0,
                        top + k as f32 * 13.0,
                        FONT_PX,
                        Color::new(0.94, 0.90, 0.82, 1.0),
                    );
                }
            }
            let hint = vfill(s.album_close, &[Verb::Back]);
            ui.text_center(&hint, 160.0, 166.0, FONT_PX, ui::TEXT_DIM);
            return;
        }
        let panel = Rect::new(16.0, 8.0, 288.0, 150.0);
        ui.panel(panel, Some(s.title_album));
        // 5칸 × 보이는 두 줄(0085). 그림이 늘면 고른 칸이 보이게 줄을 굴린다. 칸은 16:9.
        let (cw, ch, gap) = (52.0, 30.0, 5.0);
        let x0 = panel.x + (panel.w - ALBUM_COLS as f32 * cw - (ALBUM_COLS - 1) as f32 * gap) / 2.0;
        let y0 = panel.y + 38.0;
        let n = Self::album_len();
        let rows = n.div_ceil(ALBUM_COLS);
        let row = self.album_cursor / ALBUM_COLS;
        let top = self
            .album_rows
            .window(row, rows, ALBUM_ROWS, ch + gap + 10.0);
        let seen = (0..n).filter(|i| self.album_item(*i).1).count();
        for i in 0..n {
            let (il, is_seen, memory) = self.album_item(i);
            let r_row = i / ALBUM_COLS;
            if r_row < top || r_row >= top + ALBUM_ROWS {
                continue;
            }
            let r = Rect::new(
                x0 + (i % ALBUM_COLS) as f32 * (cw + gap),
                y0 + (r_row - top) as f32 * (ch + gap + 10.0),
                cw,
                ch,
            );
            if is_seen {
                self.renderer.draw_illust_thumb(ui, il, r);
                // 실의 기억 칸: 그림이 일러스트와 겹치니 모서리에 흰 실 표시.
                if memory.is_some() {
                    ui.rect(Rect::new(r.right() - 7.0, r.y + 2.0, 5.0, 3.0), ui::TEXT);
                    ui.rect(
                        Rect::new(r.right() - 3.0, r.y + 5.0, 2.0, 1.0),
                        Color::new(0.70, 0.56, 0.86, 1.0),
                    );
                }
            } else {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.text_center("?", r.center().x, r.y + 12.0, FONT_PX, ui::TEXT_DIM);
            }
            let selected = i == self.album_cursor;
            ui.rect_lines(r, if selected { ui::ACCENT } else { ui::BORDER });
            if selected {
                ui.rect_lines(
                    Rect::new(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0),
                    ui::ACCENT,
                );
            }
            self.hits.push((r, Hit::AlbumItem(i)));
        }
        // 위·아래에 더 있으면 ▲▼(누르면 한 줄 굴린다). 칸이 패널 폭을 거의 다 써서 가운데 위·아래에 둔다.
        if top > 0 {
            ui.text_center("▲", 160.0, y0 - 12.0, FONT_PX, ui::ACCENT);
            self.hits.push((
                Rect::new(146.0, y0 - 14.0, 28.0, 13.0),
                Hit::AlbumRow(false),
            ));
        }
        if top + ALBUM_ROWS < rows {
            let yb = y0 + ALBUM_ROWS as f32 * (ch + gap + 10.0) - 8.0;
            ui.text_center("▼", 160.0, yb, FONT_PX, ui::ACCENT);
            self.hits
                .push((Rect::new(146.0, yb - 2.0, 28.0, 13.0), Hit::AlbumRow(true)));
        }
        let count = fill(s.album_count, &[&seen, &n]);
        ui.text(&count, panel.x + 8.0, panel.y + 21.0, FONT_PX, ui::TEXT_DIM);
        let hint = device_hint(s.album_hint, s.album_hint, &[Verb::Confirm, Verb::Pick]);
        let hint = ui.fit(&hint, panel.w - 8.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() + 4.0, FONT_PX, ui::TEXT_DIM);
        self.close_button(ui, panel);
    }
}
