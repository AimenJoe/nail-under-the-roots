//! 안내(0178 → 0180): 왼쪽 주제 목록(위아래), 오른쪽 힌트 그림 + 설명(쪽이 넘치면 확인·터치로 다음 쪽).
//! 일시정지의 "안내" 탭에서만 연다(타이틀의 "안내"는 뺐다, 0182).
//! 목록은 기록이 닿은 주제만(`help::visible`, 0269 R36). 잠겼다 열린 주제는 "새" 점, 고르면 `help.<id>`가 선다.

use super::*;
use crate::help::{self, TOPICS};

const BODY_LINE: f32 = 12.0;
/// 힌트 그림 칸(게임 픽셀). 그림의 발바닥 줄은 30.
const PIC_W: f32 = 132.0;
const PIC_H: f32 = 40.0;

impl App {
    /// 주제 고르기(위아래·터치)와 쪽 넘기기(확인·설명 터치).
    pub(super) fn help_nav(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let vis = help::visible(&self.profile);
        let n = vis.len().max(1);
        self.help_cursor = self.help_cursor.min(n - 1);
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
        self.help_view
            .scroll(self.scroll_in, &mut self.help_cursor, n);
        if self.help_cursor != was {
            self.help_page = 0;
            self.sfx.play(crate::sfx::Id::UiMove);
        } else if flip {
            self.help_page += 1;
        }
        // 본 주제는 "새" 점을 지운다. 개발 시나리오는 기록을 건드리지 않는다(0269 B3).
        if let Some(&i) = vis.get(self.help_cursor) {
            if TOPICS[i].gate.is_some()
                && self.recording()
                && self.profile.set(&help::seen_key(TOPICS[i].id))
            {
                crate::profile::save(&self.profile);
            }
        }
    }

    /// 목록 + 그림 + 설명을 사각형 `c` 안에.
    pub(super) fn draw_help_body(&mut self, ui: &Ui, c: Rect) {
        // 목록: 열세 줄이 다 들어가지 않으면 고른 줄을 따라 넘긴다(도감과 같은 방식).
        // 줄 간격은 글자 높이 + 2 아래로 줄이지 않는다(열셋을 다 넣으면 겹친다) — 넘치면 ▲▼로 굴린다.
        let vis = help::visible(&self.profile);
        let n = vis.len();
        if n == 0 {
            return;
        }
        let row_h = FONT_PX + 4.0;
        let fits = ((c.h / row_h).floor() as usize).max(1);
        let arrow_w = ui.text_w("▼", FONT_PX).max(9.0);
        let arrows = if fits < n { arrow_w } else { 0.0 };
        // "새" 점 자리(제목 오른쪽 끝).
        let dot_w = 5.0;
        let list_w = vis
            .iter()
            .map(|&i| ui.text_w(tr(TOPICS[i].title), FONT_PX) + 8.0 + arrows + dot_w)
            .fold(80.0, f32::max)
            .min(118.0);
        let cursor = self.help_cursor.min(n - 1);
        let first = self.help_view.window(cursor, n, fits, row_h);
        let ax = c.x + list_w - arrow_w;
        // ▲▼는 누르면 한 칸 굴린다(0199). 목록 칸보다 먼저 넣어 그 자리를 이긴다.
        if first > 0 {
            ui.text("▲", ax, c.y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 3.0, c.y - 2.0, arrow_w + 6.0, row_h + 2.0),
                Hit::Scroll(-1),
            ));
        }
        if first + fits < n {
            let y = c.y + (fits - 1) as f32 * row_h;
            ui.text("▼", ax, y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(ax - 3.0, y - 2.0, arrow_w + 6.0, row_h + 2.0),
                Hit::Scroll(1),
            ));
        }
        for (row, &ti) in vis.iter().enumerate().skip(first).take(fits) {
            let t = &TOPICS[ti];
            let r = Rect::new(c.x, c.y + (row - first) as f32 * row_h, list_w, row_h - 1.0);
            let on = row == cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
                ui.rect_lines(r, ui::ACCENT);
            }
            let title = ui.fit(tr(t.title), list_w - 6.0 - arrows - dot_w, FONT_PX);
            ui.text(
                &title,
                r.x + 3.0,
                ui::mid_y(r),
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT },
            );
            // 새로 열린 주제: 제목 바로 뒤 금빛 점(글이 아니라 모양 — 언어와 상관없다).
            if t.fresh(&self.profile) {
                let dx = r.x + 3.0 + ui.text_w(&title, FONT_PX) + 3.0;
                ui.rect(Rect::new(dx, ui::mid_y(r) - 1.5, 3.0, 3.0), ui::ACCENT);
            }
            self.hits.push((r, Hit::HelpItem(row)));
        }
        // 설명: 제목 + 줄 바꿈한 본문. 한 쪽에 들어가는 줄 수만큼 끊고, 쪽 수는 아래 점으로.
        let t = &TOPICS[vis[cursor]];
        let x = c.x + list_w + 8.0;
        let w = c.right() - x;
        ui.text(tr(t.title), x, c.y, FONT_PX, ui::ACCENT);
        // 힌트 그림(0180): 제목 아래, 설명 위. 그 기능의 한 장면이 되풀이된다.
        let pic = Rect::new(x, c.y + FONT_PX + 2.0, (w - 2.0).min(PIC_W), PIC_H);
        self.renderer
            .draw_help_pic(ui, t.pic, pic, get_time() as f32);
        let body_y = pic.bottom() + 3.0;
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
