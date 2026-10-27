//! 안내 화면(0178): 왼쪽 주제 목록(위아래), 오른쪽 설명(쪽이 넘치면 확인·터치로 다음 쪽). 일시정지 메뉴·타이틀에서 연다.

use super::*;
use crate::help::TOPICS;

const BODY_LINE: f32 = 12.0;

impl App {
    pub(super) fn update_help(&mut self, a: &Actions, tap_hit: Option<Hit>, back: Back) {
        let n = TOPICS.len();
        if a.cancel || a.menu || (a.tap.is_some() && tap_hit.is_none()) {
            self.scene = match back {
                Back::Pause => Scene::Pause,
                Back::Title => Scene::Title,
            };
            return;
        }
        let was = self.help_cursor;
        if a.up {
            self.help_cursor = (self.help_cursor + n - 1) % n;
        }
        if a.down {
            self.help_cursor = (self.help_cursor + 1) % n;
        }
        let mut flip = a.confirm || tap_hit == Some(Hit::HelpPage);
        if let Some(Hit::HelpItem(i)) = tap_hit {
            flip = i == self.help_cursor;
            self.help_cursor = i;
        }
        if self.help_cursor != was {
            self.help_page = 0;
            self.sfx.play(crate::sfx::Id::UiMove);
        } else if flip {
            self.help_page += 1;
        }
    }

    pub(super) fn draw_help(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(12.0, 8.0, 296.0, 156.0);
        ui.panel(panel, Some(s.pm_help));
        ui.text_right(
            &device_hint(
                s.help_hint,
                s.help_hint_touch,
                &[Verb::Confirm, Verb::Pick, Verb::Back],
            ),
            panel.right() - 2.0,
            panel.bottom() + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let c = Rect::new(
            panel.x + 8.0,
            panel.y + 20.0,
            panel.w - 16.0,
            panel.h - 26.0,
        );
        // 목록: 열세 줄이 다 들어가지 않으면 고른 줄을 따라 넘긴다(도감과 같은 방식).
        // 줄 간격은 글자 높이 + 2 아래로 줄이지 않는다(열셋을 다 넣으면 겹친다) — 넘치면 ▲▼로 굴린다.
        let row_h = (c.h / TOPICS.len() as f32)
            .floor()
            .clamp(FONT_PX + 2.0, FONT_PX + 3.0);
        let fits = ((c.h / row_h).floor() as usize).max(1);
        let arrow_w = ui.text_w("▼", FONT_PX).max(9.0);
        let arrows = if fits < TOPICS.len() { arrow_w } else { 0.0 };
        let list_w = TOPICS
            .iter()
            .map(|t| ui.text_w(tr(t.title), FONT_PX) + 8.0 + arrows)
            .fold(80.0, f32::max)
            .min(118.0);
        let first = self
            .help_cursor
            .saturating_sub(fits - 1)
            .min(TOPICS.len().saturating_sub(fits));
        if first > 0 {
            ui.text("▲", c.x + list_w - arrow_w, c.y, FONT_PX, ui::TEXT_DIM);
        }
        if first + fits < TOPICS.len() {
            ui.text(
                "▼",
                c.x + list_w - arrow_w,
                c.y + (fits - 1) as f32 * row_h,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        for (i, t) in TOPICS.iter().enumerate().skip(first).take(fits) {
            let r = Rect::new(c.x, c.y + (i - first) as f32 * row_h, list_w, row_h - 1.0);
            let on = i == self.help_cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
                ui.rect_lines(r, ui::ACCENT);
            }
            ui.text(
                &ui.fit(tr(t.title), list_w - 6.0 - arrows, FONT_PX),
                r.x + 3.0,
                r.y + (row_h - FONT_PX) * 0.5 - 1.0,
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT },
            );
            self.hits.push((r, Hit::HelpItem(i)));
        }
        // 설명: 제목 + 줄 바꿈한 본문. 한 쪽에 들어가는 줄 수만큼 끊고, 쪽 수는 아래 점으로.
        let t = &TOPICS[self.help_cursor.min(TOPICS.len() - 1)];
        let x = c.x + list_w + 8.0;
        let w = c.right() - x;
        ui.text(tr(t.title), x, c.y, FONT_PX, ui::ACCENT);
        let body_y = c.y + FONT_PX + 4.0;
        let lines = ui::wrap(ui, tr(t.body), w - 2.0, FONT_PX);
        let per = (((c.bottom() - 8.0 - body_y) / BODY_LINE).floor() as usize).max(1);
        let pages = lines.len().div_ceil(per).max(1);
        self.help_page %= pages;
        for (i, l) in lines
            .iter()
            .skip(self.help_page * per)
            .take(per)
            .enumerate()
        {
            ui.text(l, x, body_y + i as f32 * BODY_LINE, FONT_PX, ui::TEXT);
        }
        if pages > 1 {
            for k in 0..pages {
                let dot = Rect::new(x + k as f32 * 6.0, c.bottom() - 4.0, 3.0, 3.0);
                ui.rect(
                    dot,
                    if k == self.help_page {
                        ui::ACCENT
                    } else {
                        ui::TEXT_DIM
                    },
                );
            }
        }
        self.hits.push((Rect::new(x, c.y, w, c.h), Hit::HelpPage));
    }
}
