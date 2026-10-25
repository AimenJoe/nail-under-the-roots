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
    /// 지금 언어의 폰트.
    pub font: Option<Font>,
    /// 한국어 폰트(Galmuri). 번역하지 않는 개발 화면은 언어와 상관없이 이걸로 찍는다.
    pub base: Option<Font>,
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

    /// 같은 배치에 기본 폰트(개발 화면용).
    pub fn with_base_font(&self) -> Ui {
        Ui {
            font: self.base.clone(),
            ..self.clone()
        }
    }

    /// 글자 폭(논리 px).
    pub fn text_w(&self, s: &str, size: f32) -> f32 {
        measure_text(s, self.font.as_ref(), (size * self.scale) as u16, 1.0).width / self.scale
    }

    /// 폭에 안 맞으면 뒤를 잘라 "…"를 붙인다. 언어마다 길이가 달라 고정 칸에서 넘칠 때의 마지막 안전장치.
    pub fn fit(&self, s: &str, max_w: f32, size: f32) -> String {
        if self.text_w(s, size) <= max_w {
            return s.to_string();
        }
        let mut cut: Vec<char> = s.chars().collect();
        while !cut.is_empty() {
            cut.pop();
            let t: String = cut.iter().collect::<String>().trim_end().to_string() + "…";
            if self.text_w(&t, size) <= max_w {
                return t;
            }
        }
        "…".to_string()
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
            let label = self.fit(it, w - 16.0, FONT_PX);
            self.text(&label, r.x + 14.0, r.y + 2.0, FONT_PX, c);
            rects.push(r);
        }
        rects
    }

    /// 가로 탭 줄(폭 `max_w` 안). 각 탭의 논리 rect를 돌려준다.
    /// 이름이 긴 언어면 여백을 줄이고, 그래도 넘치면 탭마다 같은 몫으로 자른다.
    pub fn tabs(&self, labels: &[&str], active: usize, x: f32, y: f32, max_w: f32) -> Vec<Rect> {
        let n = labels.len().max(1) as f32;
        let text: f32 = labels.iter().map(|l| self.text_w(l, FONT_PX)).sum();
        let pad = ((max_w - text - 2.0 * (n - 1.0)) / n).clamp(2.0, 10.0);
        let share = (max_w - 2.0 * (n - 1.0)) / n - pad;
        let mut rects = Vec::with_capacity(labels.len());
        let mut cx = x;
        for (i, l) in labels.iter().enumerate() {
            let l = if text + n * pad + 2.0 * (n - 1.0) > max_w {
                self.fit(l, share, FONT_PX)
            } else {
                l.to_string()
            };
            let w = self.text_w(&l, FONT_PX) + pad;
            let r = Rect::new(cx, y, w, FONT_PX + 4.0);
            if i == active {
                self.rect(r, PANEL_LIGHT);
                self.rect_lines(r, BORDER);
            }
            self.text(
                &l,
                r.x + pad / 2.0,
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
        let right = x + w - 4.0;
        let vw = self.text_w(value, FONT_PX).max(40.0);
        let arrow_w = 12.0;
        let r_arrow = Rect::new(right - arrow_w, y, arrow_w, row.h);
        let l_arrow = Rect::new(right - arrow_w * 2.0 - vw, y, arrow_w, row.h);
        let label = self.fit(label, l_arrow.x - x - 8.0, FONT_PX);
        self.text(&label, x + 4.0, y + 2.0, FONT_PX, c);
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

/// 줄바꿈. 띄어쓰기가 있는 글(한국어·영어)은 단어 단위, 띄어쓰기가 없는 글(일본어·중국어)은 글자 단위.
pub fn wrap(ui: &Ui, s: &str, max_w: f32, size: f32) -> Vec<String> {
    wrap_with(s, max_w, |t| ui.text_w(t, size))
}

/// 한자·가나·전각 문장부호: 이 글자 앞뒤에서는 띄어쓰기 없이도 줄을 바꿀 수 있다.
fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3000..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF)
        || c == '…'
}

/// 금칙: 줄 맨 앞에 오면 안 되는 글자(닫는 부호·구두점·작은 가나·장음).
fn no_line_start(c: char) -> bool {
    "、。，．！？：；）」』】〕…ー〜・ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ!?,.:;)".contains(c)
}

/// 금칙: 줄 맨 끝에 오면 안 되는 글자(여는 부호).
fn no_line_end(c: char) -> bool {
    "（「『【〔(".contains(c)
}

/// 측정 함수를 받는 줄바꿈(테스트용으로 분리).
pub fn wrap_with(s: &str, max_w: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    // 끊을 수 있는 조각: (조각, 앞에 띄어쓰기가 있었나). CJK 글자는 한 글자가 한 조각.
    let mut units: Vec<(String, bool)> = Vec::new();
    let mut space = false;
    for c in s.chars() {
        if c == ' ' {
            space = true;
            continue;
        }
        let joins_prev = !space
            && units
                .last()
                .is_some_and(|(u, _)| !is_cjk(c) && !u.chars().last().is_some_and(is_cjk));
        if joins_prev {
            units.last_mut().unwrap().0.push(c);
        } else {
            units.push((c.to_string(), space));
        }
        space = false;
    }
    // 한 조각이 혼자서도 줄보다 길면(독일어 합성어, URL 등) 글자 단위로 쪼갠다.
    let units: Vec<(String, bool)> = units
        .into_iter()
        .flat_map(|(u, sp)| {
            if measure(&u) <= max_w {
                return vec![(u, sp)];
            }
            let mut parts: Vec<(String, bool)> = Vec::new();
            for c in u.chars() {
                match parts.last_mut() {
                    Some((p, _)) if measure(&format!("{p}{c}")) <= max_w => p.push(c),
                    _ => parts.push((c.to_string(), parts.is_empty() && sp)),
                }
            }
            parts
        })
        .collect();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for (u, sp) in units {
        let joined = if cur.is_empty() {
            u.clone()
        } else if sp {
            format!("{cur} {u}")
        } else {
            format!("{cur}{u}")
        };
        let starts_badly = u.chars().next().is_some_and(no_line_start);
        if cur.is_empty() || measure(&joined) <= max_w || starts_badly {
            cur = joined;
            continue;
        }
        // 여는 부호로 끝나는 줄이면 그 부호를 다음 줄로 넘긴다.
        let mut carry = String::new();
        if let Some(last) = cur.chars().last().filter(|c| no_line_end(*c)) {
            cur.pop();
            carry.push(last);
        }
        lines.push(cur.trim_end().to_string());
        cur = carry + &u;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> f32 {
        s.chars().count() as f32
    }

    #[test]
    fn wraps_words_with_spaces() {
        assert_eq!(wrap_with("aa bb cc", 5.0, w), vec!["aa bb", "cc"]);
        // 줄보다 긴 단어는 쪼갠다.
        assert_eq!(wrap_with("abcdefgh", 3.0, w), vec!["abc", "def", "gh"]);
    }

    #[test]
    fn wraps_cjk_by_character_with_kinsoku() {
        // 5글자 폭: 。는 줄 맨 앞에 오지 않고 앞 줄에 붙는다.
        assert_eq!(
            wrap_with("あいうえお。かき", 5.0, w),
            vec!["あいうえお。", "かき"]
        );
        // 여는 괄호는 줄 끝에 남지 않는다.
        assert_eq!(
            wrap_with("あいうえ（かき）", 5.0, w),
            vec!["あいうえ", "（かき）"]
        );
        // 라틴 단어는 CJK 사이에서도 쪼개지지 않는다.
        assert_eq!(
            wrap_with("こうげき（ATK）", 6.0, w),
            vec!["こうげき", "（ATK）"]
        );
    }
}
