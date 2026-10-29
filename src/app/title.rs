//! 인트로·타이틀·개발 목록 그리기와 타이틀 항목.

use super::*;

pub(super) const INTRO_LINES: [Tr; 3] = [|s| s.intro_1, |s| s.intro_2, |s| s.intro_3];
pub(super) const INTRO_LINE_TIME: f32 = 2.6;
/// 타이틀 메뉴 줄. 오늘의 굴은 튜토리얼을 마친 뒤에만 보인다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TitleItem {
    /// 시작하기: 슬롯을 골라 처음부터(0078).
    New,
    /// 이어하기: 기록이 있는 슬롯을 골라 이어서(기록이 하나라도 있을 때만).
    Continue,
    Daily,
    Album,
    Settings,
    #[cfg(feature = "dev")]
    Dev,
    /// 끝내기(데스크톱만, 0240). 웹·모바일은 브라우저·OS가 닫는다.
    Quit,
}
/// 앨범 칸 배치(가로 칸 수).
/// 타이틀에 보이는 버전: Cargo.toml 버전(= 릴리스 태그 v<버전>, 0059) + CI에서 만들었으면 커밋 앞 7자.
/// 같은 버전이라도 main에 올라간 웹 빌드가 어느 커밋인지 알 수 있다. 개발판(개발 기능)이면 끝에 `dev`(0193).
pub(super) fn build_label() -> String {
    let v = env!("CARGO_PKG_VERSION");
    let dev = if cfg!(feature = "dev") { " · dev" } else { "" };
    match option_env!("GITHUB_SHA") {
        Some(sha) if sha.len() >= 7 => format!("{v} · {}{dev}", &sha[..7]),
        _ => format!("{v}{dev}"),
    }
}

impl App {
    pub(super) fn draw_intro(&self, ui: &Ui) {
        // 바탕은 타이틀 그림(0083)을 짙은 어둠 밑에 두고, 줄이 넘어갈수록 조금씩 걷어 타이틀로 이어진다.
        // 예전엔 처음 타이틀의 격자 + 뿌리 선(render::draw_title_background)이 그대로 남아 어울리지 않았다(사장 2026-10-26, 0092).
        let total = INTRO_LINE_TIME * INTRO_LINES.len() as f32;
        let lift = (self.intro_t / total).clamp(0.0, 1.0);
        self.renderer.draw_title_art(ui, get_time() as f32);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.02, 0.01, 0.03, 0.9 - 0.45 * lift),
        );
        let idx = (self.intro_t / INTRO_LINE_TIME) as usize;
        if let Some(line) = INTRO_LINES.get(idx).map(|f| tr(*f)) {
            let p = (self.intro_t - idx as f32 * INTRO_LINE_TIME) / INTRO_LINE_TIME;
            let a = (p * 4.0).min(1.0) * ((1.0 - p) * 4.0).min(1.0);
            ui.text_center(line, 160.0, 84.0, FONT_PX, Color::new(0.92, 0.86, 0.76, a));
        }
        let hint = vfill(i18n::s().skip_hint, &[Verb::Confirm]);
        ui.text_center(&hint, 160.0, 160.0, FONT_PX, ui::TEXT_DIM);
    }

    pub(super) fn title_items(&self) -> Vec<TitleItem> {
        // 시작하기 / 이어하기(기록이 있는 슬롯이 하나라도 있을 때). 둘 다 슬롯 고르기로 간다(사장 피드백 2026-10-26, 0078).
        let mut v = vec![TitleItem::New];
        if (0..crate::profile::SLOTS).any(|i| self.slot_filled(i)) {
            v.push(TitleItem::Continue);
        }
        if self.profile.flag(crate::profile::key::TUTORIAL) {
            v.push(TitleItem::Daily);
        }
        v.push(TitleItem::Album);
        v.push(TitleItem::Settings);
        #[cfg(feature = "dev")]
        v.push(TitleItem::Dev);
        if crate::DESKTOP {
            v.push(TitleItem::Quit);
        }
        v
    }

    pub(super) fn title_label(&self, it: TitleItem) -> String {
        let s = i18n::s();
        match it {
            TitleItem::New => s.title_new.to_string(),
            TitleItem::Continue => s.title_continue.to_string(),
            TitleItem::Daily => {
                let day = crate::daily::today();
                if self.profile.count(&crate::daily::key_clear(day)) > 0 {
                    s.daily_clear.to_string()
                } else {
                    match self.profile.count(&crate::daily::key_best(day)) {
                        0 => s.title_daily.to_string(),
                        n => fill(s.daily_best, &[&n]),
                    }
                }
            }
            TitleItem::Album => s.title_album.to_string(),
            TitleItem::Settings => s.title_settings.to_string(),
            #[cfg(feature = "dev")]
            TitleItem::Dev => s.title_dev.to_string(),
            TitleItem::Quit => s.title_quit.to_string(),
        }
    }

    pub(super) fn draw_title(&mut self, ui: &Ui) {
        // 뒤의 뿌리 선이 가는 글자 획과 겹치면 끊겨(깨져) 보인다(사람 피드백). 타이틀 글에는 어두운 테두리.
        let ui = &ui.with_outline();
        // 줄이 여섯(시작하기·이어하기·오늘의 굴·앨범·설정·개발)까지 들어가게 제목을 조금 올렸다(0078).
        ui.text_center("Naïl", 160.0, 16.0, FONT_PX * 2.0, ui::ACCENT);
        ui.text_center("Under the Roots", 160.0, 40.0, FONT_PX, ui::TEXT);
        let s = i18n::s();
        let labels: Vec<String> = self
            .title_items()
            .into_iter()
            .map(|it| self.title_label(it))
            .collect();
        let labels: Vec<&str> = labels.iter().map(|l| l.as_str()).collect();
        // 메뉴 뒤 그늘(0152): 줄 수만큼, 가장자리 없이 둥글게(오늘의 굴 날짜 자리까지).
        let rows = labels.len() as f32 * (FONT_PX + 4.0);
        self.renderer
            .draw_soft_shade(ui, Rect::new(96.0, 56.0, 150.0, rows), 0.55);
        // 고른 줄도 네모 막대 대신(0152): 그 줄 뒤에 부드러운 그늘 + ▶ + 밝은 글. 다른 줄은 흐린 글.
        let row_h = FONT_PX + 4.0;
        let rects: Vec<Rect> = labels
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let r = Rect::new(96.0, 56.0 + i as f32 * row_h, 128.0, row_h);
                let label = ui.fit(l, r.w - 16.0, FONT_PX);
                let on = i == self.title_cursor;
                if on {
                    let w = ui.text_w(&label, FONT_PX) + 18.0;
                    self.renderer.draw_soft_shade(
                        ui,
                        Rect::new(r.x, r.y + 1.0, w, row_h - 2.0),
                        0.75,
                    );
                    ui.text("▶", r.x + 2.0, ui::mid_y(r), FONT_PX, ui::ACCENT);
                }
                ui.text(
                    &label,
                    r.x + 14.0,
                    ui::mid_y(r),
                    FONT_PX,
                    if on { ui::ACCENT } else { ui::TEXT_DIM },
                );
                r
            })
            .collect();
        // 오늘의 굴 줄 오른쪽에 오늘 날짜(UTC): 날마다 다른 굴이라는 표시.
        if let Some(i) = self
            .title_items()
            .iter()
            .position(|t| *t == TitleItem::Daily)
        {
            let (_, m, d) = crate::daily::civil(crate::daily::today());
            let r = rects[i];
            ui.text(
                &format!("{m}/{d}"),
                r.right() + 4.0,
                r.y + 2.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        let note = self.title_note.map(|(n, _)| n);
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::TitleItem(i)));
        }
        // 타이틀에서 뒤로(Esc)는 아무 일도 하지 않아 안내에서 뺐다(0153).
        let hint = device_hint(
            s.title_pick,
            s.title_controls_touch,
            &[Verb::Pick, Verb::Confirm],
        );
        // 알림(오늘의 굴 완주)이 있으면 조작 안내 자리에.
        match note {
            Some(n) => ui.text_center(tr(n), 160.0, 153.0, FONT_PX, ui::ACCENT),
            None => ui.text_center(&hint, 160.0, 153.0, FONT_PX, ui::TEXT_DIM),
        }
        ui.text(s.copyright, 4.0, 166.0, FONT_PX, ui::TEXT_DIM);
        let room = 312.0 - ui.text_w(s.copyright, FONT_PX) - 12.0;
        let version = ui.fit(&fill(s.version, &[&build_label()]), room, FONT_PX);
        ui.text_right(&version, 316.0, 166.0, FONT_PX, ui::TEXT_DIM);
    }

    pub(super) fn draw_dev(&mut self, ui: &Ui) {
        // 개발 화면은 번역하지 않는다(한국어). 언어와 상관없이 한글 폰트로.
        let base = ui.with_base_font();
        let ui = &base;
        let panel = Rect::new(24.0, 16.0, 272.0, 148.0);
        ui.panel(panel, Some("개발 시나리오"));
        // 목록이 패널보다 길다: 고른 줄이 보이게 VISIBLE줄씩 굴린다.
        const VISIBLE: usize = 7;
        let n = scenario::LIST.len();
        let first = self
            .dev_view
            .window(self.dev_cursor, n, VISIBLE, FONT_PX + 4.0);
        let names: Vec<&str> = scenario::LIST
            .iter()
            .skip(first)
            .take(VISIBLE)
            .map(|e| e.name)
            .collect();
        let rects = ui.menu_list(
            &names,
            self.dev_cursor - first,
            panel.x + 6.0,
            panel.y + 24.0,
            110.0,
        );
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::DevItem(first + i)));
        }
        if first > 0 {
            ui.text("▲", panel.x + 112.0, panel.y + 24.0, FONT_PX, ui::TEXT_DIM);
        }
        if first + VISIBLE < n {
            ui.text(
                "▼",
                panel.x + 112.0,
                panel.y + 24.0 + 6.0 * (FONT_PX + 4.0),
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        let e = &scenario::LIST[self.dev_cursor];
        let dx = panel.x + 126.0;
        ui.text(e.name, dx, panel.y + 26.0, FONT_PX, ui::ACCENT);
        for (i, l) in ui::wrap(ui, e.desc, panel.right() - dx - 6.0, FONT_PX)
            .iter()
            .enumerate()
        {
            ui.text(l, dx, panel.y + 42.0 + i as f32 * 14.0, FONT_PX, ui::TEXT);
        }
        for (i, l) in ["확인: 시작 · 뒤로: 타이틀", "적 구성은 src/scenario.rs"]
            .iter()
            .enumerate()
        {
            ui.text(
                l,
                dx,
                panel.bottom() - 30.0 + i as f32 * 13.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
    }
}
