//! 앨범: 본 일러스트 고르기·크게 보기. 일러스트 뒤에 실의 기억 여섯(0273 A), 그 뒤에 이야기 조각(0273 C) —
//! 칸은 그 조각의 그림, 크게 보면 조각 글(이야기 조각은 제목도).

use super::*;

pub(super) const ALBUM_COLS: usize = 5;
/// 앨범에 한 번에 보이는 줄. 그림이 더 많으면 굴린다(0085).
pub(super) const ALBUM_ROWS: usize = 2;

/// 앨범 칸 하나(0273): 그림, 본·받은 적 있나, 무슨 칸인가.
struct AlbumItem {
    illust: crate::illust::Illust,
    seen: bool,
    kind: AlbumKind,
}

#[derive(Clone, Copy)]
enum AlbumKind {
    Illust,
    /// 실의 기억(글).
    Memory(Tr),
    /// 이야기 조각(제목, 글).
    Tale(Tr, Tr),
}

impl App {
    pub(super) fn album_seen(&self, il: crate::illust::Illust) -> bool {
        self.profile
            .flag(&crate::profile::key::illust(il.spec().file))
    }

    /// 앨범 칸 i: 일러스트 → 실의 기억 → 이야기 조각.
    fn album_item(&self, i: usize) -> AlbumItem {
        use crate::illust::Illust;
        let mems = crate::world::MEMORIES as usize;
        if i < Illust::ALL.len() {
            let il = Illust::ALL[i];
            return AlbumItem {
                illust: il,
                seen: self.album_seen(il),
                kind: AlbumKind::Illust,
            };
        }
        let k = i - Illust::ALL.len();
        if k < mems {
            let n = k as u8 + 1;
            let m = crate::memories::get(n);
            return AlbumItem {
                illust: m.illust,
                seen: crate::memories::seen(&self.profile, n),
                kind: AlbumKind::Memory(m.text),
            };
        }
        let t = (k - mems).min(crate::tales::ALL.len() - 1);
        let tale = &crate::tales::ALL[t];
        AlbumItem {
            illust: tale.illust,
            seen: crate::tales::owned(&self.profile, t),
            kind: AlbumKind::Tale(tale.title, tale.text),
        }
    }

    fn album_len() -> usize {
        crate::illust::Illust::ALL.len() + crate::world::MEMORIES as usize + crate::tales::ALL.len()
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
            if self.album_item(i).seen {
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
            let item = self.album_item(self.album_cursor);
            self.renderer.draw_illust(ui, item.illust, t);
            // 실의 기억·이야기 조각은 그림 아래에 글(컷씬의 내레이션처럼 어두운 띠 위에). 이야기 조각은 제목이 한 줄 위에.
            let (title, text) = match item.kind {
                AlbumKind::Illust => (None, None),
                AlbumKind::Memory(text) => (None, Some(text)),
                AlbumKind::Tale(title, text) => (Some(title), Some(text)),
            };
            if let Some(text) = text {
                let lines = ui::wrap(ui, tr(text), 280.0, FONT_PX);
                let heads = title.is_some() as usize;
                let top = 150.0 - (lines.len() + heads) as f32 * 13.0;
                ui.rect(
                    Rect::new(0.0, top - 6.0, t::VIEW_W, t::VIEW_H - top + 6.0),
                    Color::new(0.03, 0.02, 0.05, 0.66),
                );
                let ui = &ui.with_outline();
                if let Some(title) = title {
                    let head = ui.fit(tr(title), 280.0, FONT_PX);
                    ui.text_center(&head, 160.0, top, FONT_PX, ui::ACCENT);
                }
                for (k, l) in lines.iter().enumerate() {
                    ui.text_center(
                        l,
                        160.0,
                        top + (k + heads) as f32 * 13.0,
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
        let seen = (0..n).filter(|i| self.album_item(*i).seen).count();
        for i in 0..n {
            let item = self.album_item(i);
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
            if item.seen {
                self.renderer.draw_illust_thumb(ui, item.illust, r);
                // 그림이 일러스트와 겹치니 모서리 표시: 실의 기억은 흰 실, 이야기 조각은 펼친 책.
                match item.kind {
                    AlbumKind::Illust => {}
                    AlbumKind::Memory(_) => {
                        ui.rect(Rect::new(r.right() - 7.0, r.y + 2.0, 5.0, 3.0), ui::TEXT);
                        ui.rect(
                            Rect::new(r.right() - 3.0, r.y + 5.0, 2.0, 1.0),
                            Color::new(0.70, 0.56, 0.86, 1.0),
                        );
                    }
                    AlbumKind::Tale(..) => {
                        let (bx, by) = (r.right() - 9.0, r.y + 2.0);
                        ui.rect(
                            Rect::new(bx, by, 7.0, 5.0),
                            Color::new(0.55, 0.38, 0.28, 1.0),
                        );
                        let page = Color::new(0.93, 0.88, 0.76, 1.0);
                        ui.rect(Rect::new(bx + 1.0, by, 2.0, 4.0), page);
                        ui.rect(Rect::new(bx + 4.0, by, 2.0, 4.0), page);
                    }
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
