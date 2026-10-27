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
/// 글자 테두리(배경색보다 어둡게).
pub const OUTLINE_TEXT: Color = Color::new(0.05, 0.03, 0.06, 1.0);
pub const DIM_OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.55);

/// 키캡(0153): 글 속 키·버튼 이름을 이 두 표식 사이에 두면(`key`) 테두리 있는 작은 칸 + 밝은 글로 그린다.
/// 표식은 사용 영역 글자라 폰트에 없고 그리지 않는다. `text`·`text_w`가 알아보니 가운데·오른쪽 맞춤·`fit`·`wrap`이 그대로 된다.
pub const KEY_OPEN: char = '\u{E000}';
pub const KEY_CLOSE: char = '\u{E001}';
/// 키캡 안쪽 여백(좌우, 논리 px)과 칸 사이 틈.
const KEY_PAD: f32 = 2.0;
const KEY_GAP: f32 = 1.0;
pub const KEY_TEXT: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const KEY_FILL: Color = Color::new(0.10, 0.08, 0.12, 0.88);
const KEY_EDGE: Color = Color::new(0.62, 0.50, 0.38, 1.0);

/// 키·버튼 이름을 키캡 표식으로 감싼다.
pub fn key(name: &str) -> String {
    format!("{KEY_OPEN}{name}{KEY_CLOSE}")
}

/// 표식을 뺀 글(소리·검사·저장용).
pub fn plain(s: &str) -> String {
    s.chars()
        .filter(|c| *c != KEY_OPEN && *c != KEY_CLOSE)
        .collect()
}

/// (조각, 키캡인가). 닫는 표식이 없으면(잘린 글) 끝까지 키캡.
fn key_segments(s: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut cur = String::new();
    let mut in_key = false;
    for c in s.chars() {
        match c {
            KEY_OPEN if !in_key => {
                if !cur.is_empty() {
                    out.push((std::mem::take(&mut cur), false));
                }
                in_key = true;
            }
            KEY_CLOSE if in_key => {
                out.push((std::mem::take(&mut cur), true));
                in_key = false;
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push((cur, in_key));
    }
    out
}

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
    /// 글자에 1게임 px 어두운 테두리(`with_outline`). 바쁜 배경 위의 글(타이틀)용.
    pub outline: bool,
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

    /// 텍스처 한 칸을 논리 좌표 (x, y)에 원래 크기 × 배율로 찍는다.
    pub fn sprite(&self, tex: &Texture2D, src: Rect, x: f32, y: f32, tint: Color) {
        let p = self.px(vec2(x, y));
        draw_texture_ex(
            tex,
            p.x,
            p.y,
            tint,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(src.w * self.scale, src.h * self.scale)),
                ..Default::default()
            },
        );
    }

    pub fn rect_lines(&self, r: Rect, c: Color) {
        #[cfg(feature = "dev")]
        check::frame(r);
        let p = self.px(vec2(r.x, r.y));
        draw_rectangle_lines(p.x, p.y, r.w * self.scale, r.h * self.scale, self.scale, c);
    }

    pub fn line(&self, a: Vec2, b: Vec2, c: Color) {
        let (a, b) = (self.px(a), self.px(b));
        draw_line(a.x, a.y, b.x, b.y, self.scale, c);
    }

    fn params(&self, size: f32, c: Color) -> TextParams<'_> {
        let (font_size, font_scale) = glyph_plan(size, self.scale, miniquad::window::dpi_scale());
        TextParams {
            font: self.font.as_ref(),
            font_size,
            font_scale,
            color: c,
            ..Default::default()
        }
    }

    /// 왼쪽 위 기준으로 글자를 찍는다. `size`는 논리 px(12 또는 24).
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        #[cfg(feature = "dev")]
        if check::on() && c.a > 0.05 {
            check::text(&plain(s), Rect::new(x, y, self.text_w(s, size), size));
        }
        if s.contains(KEY_OPEN) {
            // 키캡(0153): 보통 글은 받은 색, 키 이름은 테두리 칸 + 밝은 글.
            let mut cx = x;
            for (seg, is_key) in key_segments(s) {
                if is_key {
                    let w = self.plain_w(&seg, size) + 2.0 * KEY_PAD;
                    let r = Rect::new(cx, y - 1.0, w, size + 1.0);
                    self.rect(
                        r,
                        Color::new(KEY_FILL.r, KEY_FILL.g, KEY_FILL.b, KEY_FILL.a * c.a),
                    );
                    // 테두리는 넘침 검사의 틀로 세지 않는다(글 한 줄 속 작은 칸이라, 줄 전체가 "틀 밖"으로 잡혔다).
                    let p = self.px(vec2(r.x, r.y));
                    draw_rectangle_lines(
                        p.x,
                        p.y,
                        r.w * self.scale,
                        r.h * self.scale,
                        self.scale,
                        Color::new(KEY_EDGE.r, KEY_EDGE.g, KEY_EDGE.b, c.a),
                    );
                    let kc = Color::new(KEY_TEXT.r, KEY_TEXT.g, KEY_TEXT.b, c.a);
                    self.plain_text(&seg, cx + KEY_PAD, y, size, kc);
                    cx += w + KEY_GAP;
                } else {
                    self.plain_text(&seg, cx, y, size, c);
                    cx += self.plain_w(&seg, size);
                }
            }
            return;
        }
        self.plain_text(s, x, y, size, c);
    }

    /// 표식 없는 글 한 줄(테두리 켜졌으면 테두리).
    fn plain_text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        let p = self.px(vec2(x, y + size * 0.8));
        if self.outline {
            // 상하좌우 1게임 px에 어두운 글자를 먼저: 가는 획(Galmuri Condensed)이 뒤의 선과 겹쳐 끊겨 보이지 않게.
            let k = self.scale;
            let dark = Color::new(OUTLINE_TEXT.r, OUTLINE_TEXT.g, OUTLINE_TEXT.b, c.a);
            for (dx, dy) in [(-k, 0.0), (k, 0.0), (0.0, -k), (0.0, k)] {
                draw_text_ex(
                    s,
                    (p.x + dx).round(),
                    (p.y + dy).round(),
                    self.params(size, dark),
                );
            }
        }
        draw_text_ex(s, p.x.round(), p.y.round(), self.params(size, c));
    }

    /// 같은 배치에 글자 테두리를 켠 것.
    pub fn with_outline(&self) -> Ui {
        Ui {
            outline: true,
            ..self.clone()
        }
    }

    /// 같은 배치에 기본 폰트(개발 화면용).
    pub fn with_base_font(&self) -> Ui {
        Ui {
            font: self.base.clone(),
            ..self.clone()
        }
    }

    /// 글자 아틀라스를 미리 굽는다: 이 언어 글자 전부를 지금 크기로 캐시(measure_text가 굽는다, 올리는 건 다음 그리기에서 한 번).
    /// 안 하면 대화창이 한 글자씩 찍힐 때마다 새 글자 → 아틀라스 텍스처 전체(1024², 4MB)를 다시 올린다.
    pub fn warm(&self, glyphs: &str, size: f32) {
        let (font_size, font_scale) = glyph_plan(size, self.scale, miniquad::window::dpi_scale());
        measure_text(glyphs, self.font.as_ref(), font_size, font_scale);
    }

    /// 글자 폭(논리 px).
    pub fn text_w(&self, s: &str, size: f32) -> f32 {
        if !s.contains(KEY_OPEN) {
            return self.plain_w(s, size);
        }
        key_segments(s)
            .iter()
            .map(|(seg, is_key)| {
                let w = self.plain_w(seg, size);
                if *is_key {
                    w + 2.0 * KEY_PAD + KEY_GAP
                } else {
                    w
                }
            })
            .sum()
    }

    fn plain_w(&self, s: &str, size: f32) -> f32 {
        let (font_size, font_scale) = glyph_plan(size, self.scale, miniquad::window::dpi_scale());
        measure_text(s, self.font.as_ref(), font_size, font_scale).width / self.scale
    }

    /// 폭에 안 맞으면 뒤를 잘라 "…"를 붙인다. 언어마다 길이가 달라 고정 칸에서 넘칠 때의 마지막 안전장치.
    pub fn fit(&self, s: &str, max_w: f32, size: f32) -> String {
        if self.text_w(s, size) <= max_w {
            return s.to_string();
        }
        #[cfg(feature = "dev")]
        check::cut(s, max_w);
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
        // 값 양옆 2px: 값이 40px보다 길면 ◀▶가 글자에 붙었다(간체 "◀简体中文▶", 0089).
        let vw = self.text_w(value, FONT_PX).max(40.0) + 4.0;
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

/// 글자를 몇 px로 굽고(래스터화) 몇 배로 키워 그릴까: (font_size, font_scale).
///
/// 폰트가 12px 픽셀 폰트라, 원래 크기의 정수배로 구워 정수배로 키우면 화면 크기로 바로 굽는 것과 모양이 같다.
/// macroquad는 `ceil(font_size × dpi)` 물리 px로 굽고(text.rs), 새 글자가 생길 때마다 글자 아틀라스 텍스처를 통째로 다시 올린다.
/// 화면 크기로 구우면 레티나(dpi 2)·배율 4에서 글자 하나가 96px → 아틀라스가 금방 커지고,
/// 대화창이 한 글자씩 찍힐 때마다 큰 텍스처를 다시 올린다(Safari에서 특히 버벅임).
/// 아틀라스가 4096²까지 크면 Android Chrome에서 글자가 검게 뭉개졌다(dpi 2.625, 0058): macroquad 셰이더의
/// `varying lowp vec2 uv`가 큰 텍스처에서 엉뚱한 곳을 집는다. 그래서
/// 1. 굽는 크기 = size × m(m은 정수, 4 이하)이고 키우는 배가 정수인 가장 작은 m.
/// 2. 없으면(dpi가 어중간) size × m으로 정확히 구워지는 가장 작은 m으로 굽고 정수가 아닌 배로 키운다.
///    픽셀 폭이 조금 고르지 않지만(화면 전체가 이미 그렇다) 선명하고 아틀라스가 작다.
/// 3. 그것도 없으면 예전처럼 화면 크기로 굽는다.
pub fn glyph_plan(size: f32, scale: f32, dpi: f32) -> (u16, f32) {
    let whole = |x: f32| (x - x.round()).abs() < 1e-3;
    let target = size * scale * dpi;
    // raster 물리 px로 정확히 구워지는 font_size(ceil(k × dpi) = raster인 k).
    let bake = |raster: f32| -> Option<u16> {
        let k = (raster / dpi + 1e-4).floor();
        (k >= 1.0 && ((k * dpi - 1e-4).ceil() - raster).abs() < 1e-3).then_some(k as u16)
    };
    for m in 1..=4 {
        let raster = size * m as f32;
        let mag = target / raster;
        if let Some(k) = bake(raster).filter(|_| whole(mag) && mag >= 1.0) {
            return (k, mag.round());
        }
    }
    for m in 1..=8 {
        let raster = size * m as f32;
        if raster > target + 1e-3 {
            break;
        }
        if let Some(k) = bake(raster) {
            return (k, target / raster);
        }
    }
    ((size * scale) as u16, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 키캡 조각(0153): 표식 사이가 키, 잘려 닫는 표식이 없으면 끝까지 키.
    #[test]
    fn key_segments_split_keys() {
        let t = format!("{} 고르기 · {} 확인", key("↑↓"), key("Enter"));
        let segs = key_segments(&t);
        assert_eq!(segs[0], ("↑↓".to_string(), true));
        assert_eq!(segs[1], (" 고르기 · ".to_string(), false));
        assert_eq!(segs[2], ("Enter".to_string(), true));
        assert_eq!(segs.len(), 4);
        assert_eq!(
            key_segments(&format!("A {KEY_OPEN}En")),
            vec![("A ".into(), false), ("En".into(), true)]
        );
        assert_eq!(plain(&t), "↑↓ 고르기 · Enter 확인");
    }

    /// 굽는 크기는 원래 12 물리 px(가능하면), 키우는 배는 정수. 레티나(dpi 2)·배율 4: 12px로 구워 8배 = 96px.
    #[test]
    fn glyphs_bake_small_and_scale_by_whole_numbers() {
        assert_eq!(glyph_plan(12.0, 4.0, 1.0), (12, 4.0));
        assert_eq!(glyph_plan(12.0, 4.0, 2.0), (6, 8.0));
        assert_eq!(glyph_plan(12.0, 3.0, 3.0), (4, 9.0));
        assert_eq!(
            glyph_plan(12.0, 4.0, 1.5),
            (8, 6.0),
            "12px로 구워 6배 = 72px"
        );
        // macroquad는 ceil(9 × 1.25) = 12px로 굽는다: 12px로 구워 3.75배.
        assert_eq!(glyph_plan(12.0, 3.0, 1.25), (9, 3.75));
        // Android Chrome dpi 2.625: 12px로는 정확히 못 구워(ceil(4 × 2.625) = 11) 24px로 굽고 2.625배.
        // 예전에는 화면 크기(63px)로 구워 아틀라스가 4096²까지 컸다(0058).
        assert_eq!(glyph_plan(12.0, 2.0, 2.625), (9, 2.625));
        // dpi 2.75: 36px(13 × 2.75 = 35.75 → 36)로 구워 66/36배.
        let (k, mag) = glyph_plan(12.0, 2.0, 2.75);
        assert_eq!(k, 13);
        assert!((mag - 66.0 / 36.0).abs() < 1e-4);
        // 굽는 크기는 macroquad가 실제로 굽는 크기와 같다.
        for dpi in [1.0f32, 1.25, 1.5, 1.75, 2.0, 2.25, 2.625, 2.75, 3.0, 3.5] {
            for scale in [1.0f32, 2.0, 3.0, 4.0, 6.0] {
                let (k, mag) = glyph_plan(12.0, scale, dpi);
                let raster = (k as f32 * dpi - 1e-4).ceil();
                assert!(
                    (raster * mag - 12.0 * scale * dpi).abs() < 0.01,
                    "dpi {dpi} scale {scale}: {raster} × {mag}"
                );
            }
        }
    }

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

/// 개발 빌드의 글자 넘침 검사(F3로 켬/끔). 그린 글자 칸을 프레임마다 모아, 화면 밖·테두리 밖·겹침·`fit`으로 잘림을
/// 콘솔에 한 번씩 찍는다(웹은 브라우저 콘솔). 헤드리스로 화면 × 언어를 돌며 콘솔을 모으면 넘침을 사람 눈 전에 거른다.
#[cfg(feature = "dev")]
pub mod check {
    use macroquad::logging::warn;
    use macroquad::prelude::Rect;
    use std::cell::RefCell;
    use std::collections::HashSet;

    #[derive(Default)]
    struct State {
        on: bool,
        texts: Vec<(String, Rect)>,
        frames: Vec<Rect>,
        seen: HashSet<String>,
    }

    thread_local! {
        static S: RefCell<State> = RefCell::new(State::default());
    }

    pub fn on() -> bool {
        S.with(|s| s.borrow().on)
    }

    pub fn toggle() {
        S.with(|s| {
            let mut s = s.borrow_mut();
            s.on = !s.on;
            s.seen.clear();
            warn!("[ui-check] {}", if s.on { "on" } else { "off" });
        });
    }

    pub(super) fn text(t: &str, r: Rect) {
        if t.trim().is_empty() {
            return;
        }
        S.with(|s| s.borrow_mut().texts.push((t.to_string(), r)));
    }

    pub(super) fn frame(r: Rect) {
        if on() {
            S.with(|s| s.borrow_mut().frames.push(r));
        }
    }

    pub(super) fn cut(t: &str, max_w: f32) {
        if on() {
            report(format!("cut ({max_w:.0}px) {t}"));
        }
    }

    fn report(msg: String) {
        S.with(|s| {
            let mut s = s.borrow_mut();
            if s.seen.insert(msg.clone()) {
                warn!("[ui-check] {}", msg);
            }
        });
    }

    /// 두 칸이 1px 넘게 겹치는가(맞닿은 줄은 겹침이 아니다).
    fn overlap(a: Rect, b: Rect) -> bool {
        let w = a.right().min(b.right()) - a.x.max(b.x);
        let h = a.bottom().min(b.bottom()) - a.y.max(b.y);
        w > 1.5 && h > 1.5
    }

    /// 프레임 끝에 부른다: 모은 칸을 살피고 비운다.
    pub fn end_frame(screen_w: f32, screen_h: f32) {
        if !on() {
            return;
        }
        let (texts, frames) = S.with(|s| {
            let mut s = s.borrow_mut();
            (std::mem::take(&mut s.texts), std::mem::take(&mut s.frames))
        });
        for (i, (t, r)) in texts.iter().enumerate() {
            if r.x < -0.5 || r.y < -0.5 || r.right() > screen_w + 0.5 || r.bottom() > screen_h + 0.5
            {
                report(format!(
                    "off-screen x{:.0} y{:.0} w{:.0} {t}",
                    r.x, r.y, r.w
                ));
            }
            // 글의 왼쪽 위를 품은 가장 작은 테두리 밖으로 나가면 넘침.
            let inside = frames
                .iter()
                .filter(|f| f.contains(r.point() + macroquad::prelude::vec2(0.5, 0.5)))
                .min_by(|a, b| (a.w * a.h).total_cmp(&(b.w * b.h)));
            if let Some(f) = inside {
                if r.right() > f.right() + 0.5 || r.bottom() > f.bottom() + 1.5 {
                    report(format!(
                        "past-frame ({:.0},{:.0} {:.0}x{:.0}) by {:.0}→ {:.0}↓ {t}",
                        f.x,
                        f.y,
                        f.w,
                        f.h,
                        (r.right() - f.right()).max(0.0),
                        (r.bottom() - f.bottom()).max(0.0)
                    ));
                }
            }
            for (u, q) in texts.iter().skip(i + 1) {
                if u != t && overlap(*r, *q) {
                    report(format!("overlap {t} <> {u}"));
                }
            }
        }
    }
}
