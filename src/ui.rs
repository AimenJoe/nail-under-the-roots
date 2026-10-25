//! 메뉴·패널 그리기 도우미. 좌표는 내부 해상도(320×180) 단위, 실제 그리기는 화면 패스(정수 배율)에서.
//! 텍스트는 실제 해상도로 찍어 선명하다. 커서 없이 동작한다(상하좌우 + 확인). 탭/클릭은 보조.

use macroquad::prelude::*;

/// 폰트 기본 크기(논리 px). Galmuri11은 12px 격자.
pub const FONT_PX: f32 = 12.0;

pub const PANEL: Color = Color::new(0.07, 0.05, 0.09, 0.94);
pub const PANEL_LIGHT: Color = Color::new(0.14, 0.10, 0.15, 1.0);
pub const BORDER: Color = Color::new(0.55, 0.40, 0.28, 1.0);
pub const TEXT: Color = Color::new(0.92, 0.86, 0.76, 1.0);
pub const TEXT_DIM: Color = Color::new(0.52, 0.48, 0.46, 1.0);
pub const ACCENT: Color = Color::new(0.90, 0.71, 0.47, 1.0);
pub const DIM_OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.55);

/// 한 프레임의 화면 배치. 렌더러의 `present()`가 만든다.
#[derive(Clone)]
pub struct Ui {
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
    pub font: Option<Font>,
}

impl Ui {
    /// 논리 좌표 → 화면 px.
    pub fn px(&self, p: Vec2) -> Vec2 {
        vec2(self.ox + p.x * self.scale, self.oy + p.y * self.scale)
    }

    pub fn rect(&self, r: Rect, c: Color) {
        let p = self.px(vec2(r.x, r.y));
        draw_rectangle(p.x, p.y, r.w * self.scale, r.h * self.scale, c);
    }

    pub fn rect_lines(&self, r: Rect, c: Color) {
        let p = self.px(vec2(r.x, r.y));
        draw_rectangle_lines(p.x, p.y, r.w * self.scale, r.h * self.scale, self.scale, c);
    }

    pub fn line(&self, a: Vec2, b: Vec2, c: Color) {
        let (a, b) = (self.px(a), self.px(b));
        draw_line(a.x, a.y, b.x, b.y, self.scale, c);
    }

    fn params(&self, size: f32, c: Color) -> TextParams<'_> {
        TextParams {
            font: self.font.as_ref(),
            font_size: (size * self.scale) as u16,
            color: c,
            ..Default::default()
        }
    }

    /// 왼쪽 위 기준으로 글자를 찍는다. `size`는 논리 px(12 또는 24).
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        let p = self.px(vec2(x, y + size * 0.8));
        draw_text_ex(s, p.x.round(), p.y.round(), self.params(size, c));
    }

    /// 글자 폭(논리 px).
    pub fn text_w(&self, s: &str, size: f32) -> f32 {
        measure_text(s, self.font.as_ref(), (size * self.scale) as u16, 1.0).width / self.scale
    }

    pub fn text_center(&self, s: &str, cx: f32, y: f32, size: f32, c: Color) {
        let w = self.text_w(s, size);
        self.text(s, cx - w / 2.0, y, size, c);
    }

    pub fn text_right(&self, s: &str, right: f32, y: f32, size: f32, c: Color) {
        let w = self.text_w(s, size);
        self.text(s, right - w, y, size, c);
    }

    /// 테두리 있는 패널. 제목이 있으면 위에 찍는다.
    pub fn panel(&self, r: Rect, title: Option<&str>) {
        self.rect(r, PANEL);
        self.rect_lines(r, BORDER);
        if let Some(t) = title {
            self.text(t, r.x + 6.0, r.y + 3.0, FONT_PX, ACCENT);
            self.line(
                vec2(r.x + 4.0, r.y + FONT_PX + 6.0),
                vec2(r.right() - 4.0, r.y + FONT_PX + 6.0),
                BORDER,
            );
        }
    }

    /// 세로 목록. 커서 행을 강조. 각 행의 논리 rect를 돌려준다(탭 판정용).
    pub fn menu_list(&self, items: &[&str], cursor: usize, x: f32, y: f32, w: f32) -> Vec<Rect> {
        let row_h = FONT_PX + 4.0;
        let mut rects = Vec::with_capacity(items.len());
        for (i, it) in items.iter().enumerate() {
            let r = Rect::new(x, y + i as f32 * row_h, w, row_h);
            if i == cursor {
                self.rect(r, PANEL_LIGHT);
                self.text("▶", r.x + 2.0, r.y + 2.0, FONT_PX, ACCENT);
            }
            let c = if i == cursor { TEXT } else { TEXT_DIM };
            self.text(it, r.x + 14.0, r.y + 2.0, FONT_PX, c);
            rects.push(r);
        }
        rects
    }

    /// 가로 탭 줄. 각 탭의 논리 rect를 돌려준다.
    pub fn tabs(&self, labels: &[&str], active: usize, x: f32, y: f32) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(labels.len());
        let mut cx = x;
        for (i, l) in labels.iter().enumerate() {
            let w = self.text_w(l, FONT_PX) + 10.0;
            let r = Rect::new(cx, y, w, FONT_PX + 4.0);
            if i == active {
                self.rect(r, PANEL_LIGHT);
                self.rect_lines(r, BORDER);
            }
            self.text(
                l,
                r.x + 5.0,
                r.y + 2.0,
                FONT_PX,
                if i == active { ACCENT } else { TEXT_DIM },
            );
            rects.push(r);
            cx += w + 2.0;
        }
        rects
    }

    /// 설정 한 줄: 라벨 ◀ 값 ▶. (행, 왼쪽 화살표, 오른쪽 화살표) rect.
    pub fn setting_row(
        &self,
        label: &str,
        value: &str,
        x: f32,
        y: f32,
        w: f32,
        selected: bool,
    ) -> (Rect, Rect, Rect) {
        let row = Rect::new(x, y, w, FONT_PX + 4.0);
        if selected {
            self.rect(row, PANEL_LIGHT);
        }
        let c = if selected { TEXT } else { TEXT_DIM };
        self.text(label, x + 4.0, y + 2.0, FONT_PX, c);
        let right = x + w - 4.0;
        let vw = self.text_w(value, FONT_PX).max(40.0);
        let arrow_w = 12.0;
        let r_arrow = Rect::new(right - arrow_w, y, arrow_w, row.h);
        let l_arrow = Rect::new(right - arrow_w * 2.0 - vw, y, arrow_w, row.h);
        let ac = if selected { ACCENT } else { TEXT_DIM };
        self.text("◀", l_arrow.x + 2.0, y + 2.0, FONT_PX, ac);
        self.text_center(
            value,
            (l_arrow.right() + r_arrow.x) / 2.0,
            y + 2.0,
            FONT_PX,
            c,
        );
        self.text("▶", r_arrow.x + 2.0, y + 2.0, FONT_PX, ac);
        (row, l_arrow, r_arrow)
    }

    /// 화면 전체를 어둡게.
    pub fn dim(&self) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), DIM_OVERLAY);
    }
}

/// 단어 단위로 줄바꿈(한글은 글자 단위로도 끊긴다).
pub fn wrap(ui: &Ui, s: &str, max_w: f32, size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        let cand = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if ui.text_w(&cand, size) <= max_w || cur.is_empty() {
            cur = cand;
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
