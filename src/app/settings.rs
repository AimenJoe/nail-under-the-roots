//! 설정 창과 조작 설정(다시 붙이기).

use super::*;

pub(super) const SETTING_ROWS: [Tr; 12] = [
    |s| s.set_sound,
    |s| s.set_music,
    |s| s.set_shake,
    |s| s.set_effects,
    |s| s.set_tells,
    |s| s.set_debug,
    |s| s.set_language,
    |s| s.set_assist,
    |s| s.set_controls,
    |s| s.set_licenses,
    |s| s.set_erase,
    |s| s.set_back,
];
/// 설정의 "포포의 응원"·"조작 설정"·"기록 지우기"·"돌아가기" 줄.
pub(super) const SETTING_ASSIST: usize = 7;
pub(super) const SETTING_CONTROLS: usize = 8;
pub(super) const SETTING_LICENSES: usize = 9;
pub(super) const SETTING_ERASE: usize = 10;
pub(super) const SETTING_BACK: usize = 11;
/// 설정 창에 한 번에 보이는 줄. 줄이 더 많으면 고른 줄이 보이게 굴린다(입자 줄을 더해 열 줄이 됐다, 0141).
pub(super) const SETTING_VISIBLE: usize = 9;
/// 값이 아니라 누르는 줄(좌우로 바뀌지 않는다).
pub(super) fn setting_is_button(i: usize) -> bool {
    matches!(
        i,
        SETTING_CONTROLS | SETTING_LICENSES | SETTING_ERASE | SETTING_BACK
    )
}
/// 조작 설정의 줄: 동작들(input::ACTS) + 기본값 + 돌아가기.
pub(super) const CONTROL_NAMES: [Tr; 7] = [
    |s| s.act_attack,
    |s| s.act_dodge,
    |s| s.act_special,
    |s| s.act_use_item,
    |s| s.act_item_prev,
    |s| s.act_item_next,
    |s| s.act_map,
];
pub(super) const CONTROL_RESET: usize = 7;
pub(super) const CONTROL_BACK: usize = 8;
/// 다시 붙이기: 이만큼 아무것도 안 누르면 그만둔다.
pub(super) const REBIND_TIMEOUT: f32 = 5.0;

/// 설정. 값은 0..=10 단계. 웹에서는 브라우저에 저장한다(prefs.rs). 디버그 정보는 저장하지 않는다.
pub struct Settings {
    pub volume: u8,
    /// 음악 크기(0..=10, 효과음과 따로, 0081).
    pub music: u8,
    pub shake: u8,
    /// 입자 효과(0..=10, 0141). 0이면 먼지·불똥·환경 입자를 그리지 않는다(공격 범위 고리는 남긴다).
    pub effects: u8,
    /// 공격 예고 진하기(1..=10, 0198). 7이 0196·0197의 투명도, 기본 5.
    pub tells: u8,
    pub debug_overlay: bool,
    /// 사람이 고른 언어(None = 기기 언어).
    pub lang: Option<i18n::Lang>,
}

impl Settings {
    pub(super) fn volume_f(&self) -> f32 {
        self.volume as f32 / 10.0
    }
    pub(super) fn shake_f(&self) -> f32 {
        self.shake as f32 / 10.0
    }
    pub(super) fn effects_f(&self) -> f32 {
        self.effects as f32 / 10.0
    }
    /// 예고 잉크 투명도 배율(7 = 1).
    pub(super) fn tells_f(&self) -> f32 {
        self.tells as f32 / 7.0
    }
}

impl App {
    pub(super) fn adjust_setting(&mut self, row: usize, d: i8) {
        match row {
            0 => {
                self.settings.volume = (self.settings.volume as i8 + d).clamp(0, 10) as u8;
                self.sfx.set_volume(self.settings.volume_f());
                self.amb.set_volume(self.settings.volume_f());
                self.sfx.play(crate::sfx::Id::EnemyHit);
            }
            1 => {
                self.settings.music = (self.settings.music as i8 + d).clamp(0, 10) as u8;
                self.music.set_volume(self.settings.music as f32 / 10.0);
            }
            2 => self.settings.shake = (self.settings.shake as i8 + d).clamp(0, 10) as u8,
            3 => self.settings.effects = (self.settings.effects as i8 + d).clamp(0, 10) as u8,
            // 0은 두지 않는다: 예고는 놀이 정보라 아예 끄지는 못한다.
            4 => self.settings.tells = (self.settings.tells as i8 + d).clamp(1, 10) as u8,
            5 => {
                self.settings.debug_overlay = !self.settings.debug_overlay;
                self.world.debug.overlay = self.settings.debug_overlay;
            }
            6 => {
                let l = i18n::lang().cycle(d);
                i18n::set_lang(l);
                self.settings.lang = Some(l);
            }
            SETTING_ASSIST => {
                use crate::profile::key;
                if !self.profile.unset(key::ASSIST) {
                    self.profile.set(key::ASSIST);
                }
                crate::profile::save(&self.profile);
                self.apply_assist();
            }
            _ => {}
        }
    }

    pub(super) fn save_prefs(&self) {
        crate::prefs::save(&crate::prefs::Prefs {
            volume: self.settings.volume,
            music: self.settings.music,
            shake: self.settings.shake,
            effects: self.settings.effects,
            tells: self.settings.tells,
            lang: self.settings.lang,
            bindings: crate::input::bindings(),
        });
    }

    /// 조작 설정: 위아래로 동작, 좌우로 키보드/패드 칸, 확인으로 다시 붙이기(다음에 누르는 키·버튼).
    pub(super) fn update_controls(
        &mut self,
        a: &Actions,
        tap_hit: Option<Hit>,
        back: Back,
        dt: f32,
    ) {
        use crate::input::{self, Captured, ACTS};
        if let Some(left) = self.rebind {
            // 기다리는 중: 다른 입력은 무시. Esc로 그만두기, 시간이 다 돼도 그만.
            let left = left - dt;
            if a.menu {
                self.rebind = None;
                return;
            }
            if let Some(c) = self.input.capture() {
                let mut b = input::bindings();
                let act = ACTS[self.controls_cursor];
                let ok = match c {
                    Captured::Key(k) if self.controls_col == 0 => b.bind_key(act, k),
                    Captured::Pad(p) if self.controls_col == 1 => b.bind_pad(act, p),
                    _ => false,
                };
                if ok {
                    input::set_bindings(b);
                    self.sfx.play(crate::sfx::Id::Claw(1));
                    self.rebind = None;
                    return;
                }
            }
            self.rebind = if left <= 0.0 { None } else { Some(left) };
            return;
        }
        let n = CONTROL_BACK + 1;
        if a.up {
            self.controls_cursor = (self.controls_cursor + n - 1) % n;
        }
        if a.down {
            self.controls_cursor = (self.controls_cursor + 1) % n;
        }
        if a.left || a.right {
            self.controls_col = 1 - self.controls_col;
        }
        let mut chosen = a.confirm;
        if let Some(Hit::ControlCell(row, col)) = tap_hit {
            chosen = true;
            self.controls_cursor = row;
            self.controls_col = col;
        }
        if a.cancel || (chosen && self.controls_cursor == CONTROL_BACK) {
            self.save_prefs();
            self.scene = Scene::Settings { back };
            return;
        }
        if chosen && self.controls_cursor == CONTROL_RESET {
            input::set_bindings(input::Bindings::DEFAULT);
            self.sfx.play(crate::sfx::Id::Claw(0));
        } else if chosen {
            // 확인을 누른 그 프레임의 키(Enter·Space·J)가 바로 잡히지 않게 다음 프레임부터.
            self.rebind = Some(REBIND_TIMEOUT + dt);
            self.input.take();
        }
    }

    pub(super) fn draw_settings(&mut self, ui: &Ui) {
        // 창을 위로 붙이고 줄 간격을 1px 줄여 아래 안내 두 줄 자리를 남긴다(0081). 열 줄이 되어 아홉 줄만 보이고 굴린다(0141).
        let panel = Rect::new(56.0, 0.0, 208.0, 155.0);
        let s = i18n::s();
        ui.panel(panel, Some(s.settings));
        // 터치로도 나갈 수 있게 닫기 단추(0198): 돌아가기 줄이 가려져 있으면 손가락만으로는 나갈 길이 없었다.
        self.close_button(ui, panel);
        let assist = self.assist_resist();
        let values = [
            format!("{}%", self.settings.volume * 10),
            format!("{}%", self.settings.music * 10),
            format!("{}%", self.settings.shake * 10),
            format!("{}%", self.settings.effects * 10),
            format!("{}%", self.settings.tells * 10),
            if self.settings.debug_overlay {
                s.on
            } else {
                s.off
            }
            .to_string(),
            s.lang_name.to_string(),
            // 포포의 응원: 켜져 있으면 지금 줄여 주는 비율.
            if assist > 0.0 {
                format!("{} {}%", s.on, (assist * 100.0).round())
            } else {
                s.off.to_string()
            },
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        let row_h = FONT_PX + 3.0;
        // 줄이 창보다 많다: 고른 줄이 보이게 SETTING_VISIBLE줄씩 굴린다(개발 목록과 같은 방식).
        let n = SETTING_ROWS.len();
        let first = self
            .settings_view
            .window(self.settings_cursor, n, SETTING_VISIBLE, row_h);
        for (i, label) in trs(&SETTING_ROWS)
            .into_iter()
            .enumerate()
            .skip(first)
            .take(SETTING_VISIBLE)
        {
            let y = panel.y + 18.0 + (i - first) as f32 * row_h;
            let selected = self.settings_cursor == i;
            if setting_is_button(i) {
                let r = Rect::new(panel.x + 6.0, y, panel.w - 12.0, FONT_PX + 3.0);
                if selected {
                    ui.rect(r, ui::PANEL_LIGHT);
                    ui.text("▶", r.x + 2.0, ui::mid_y(r), FONT_PX, ui::ACCENT);
                }
                let label = if i == SETTING_CONTROLS || i == SETTING_LICENSES {
                    format!("{label}  →")
                } else if i == SETTING_ERASE && self.erase_armed {
                    s.set_erase_confirm.to_string()
                } else {
                    label.to_string()
                };
                ui.text(
                    &label,
                    r.x + 14.0,
                    ui::mid_y(r),
                    FONT_PX,
                    if selected { ui::TEXT } else { ui::TEXT_DIM },
                );
                self.hits.push((r, Hit::SettingRow(i)));
            } else {
                let (row, l, r) = ui.setting_row(
                    label,
                    &values[i],
                    panel.x + 6.0,
                    y,
                    panel.w - 12.0,
                    selected,
                );
                self.hits.push((l, Hit::SettingLeft(i)));
                self.hits.push((r, Hit::SettingRight(i)));
                self.hits.push((row, Hit::SettingRow(i)));
            }
        }
        // 가려진 줄이 있으면 위·아래 끝에 표시. 누르면 그쪽으로 한 줄 굴린다(터치, 0198). ▲는 닫기 단추 옆.
        if first > 0 {
            let at = vec2(panel.right() - 30.0, panel.y + 4.0);
            ui.text("▲", at.x, at.y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(at.x - 5.0, at.y - 4.0, 18.0, 18.0),
                Hit::Scroll(-1),
            ));
        }
        if first + SETTING_VISIBLE < n {
            let at = vec2(panel.right() - 14.0, panel.bottom() - 13.0);
            ui.text("▼", at.x, at.y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(at.x - 5.0, at.y - 4.0, 18.0, 18.0),
                Hit::Scroll(1),
            ));
        }
        let settings_hint = vfill(s.settings_hint, &[Verb::Change]);
        for (i, l) in ui::wrap(ui, &settings_hint, 300.0, FONT_PX)
            .iter()
            .take(2)
            .enumerate()
        {
            ui.text_center(
                l,
                160.0,
                panel.bottom() + 1.0 + i as f32 * 12.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
    }

    /// 조작 설정 화면: 동작 | 키보드 | 패드 표. 고른 칸 강조, 기다리는 중이면 "누르세요… 5".
    pub(super) fn draw_controls(&mut self, ui: &Ui) {
        use crate::input::{self, ACTS};
        let s = i18n::s();
        let panel = Rect::new(28.0, 2.0, 264.0, 154.0);
        ui.panel(panel, Some(s.set_controls));
        let b = input::bindings();
        let row_h = FONT_PX + 0.5;
        let name_w = 110.0;
        let col_w = (panel.w - 12.0 - name_w) / 2.0;
        let x0 = panel.x + 6.0;
        let y0 = panel.y + 19.0;
        ui.text(
            &ui.fit(s.ctl_key, col_w - 6.0, FONT_PX),
            x0 + name_w + 4.0,
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        ui.text(
            &ui.fit(s.ctl_pad, col_w - 6.0, FONT_PX),
            x0 + name_w + col_w + 4.0,
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        for row in 0..=CONTROL_BACK {
            let y = y0 + (row as f32 + 1.0) * row_h;
            // 고른 칸은 y − 1부터 한 줄(창이 꽉 차 칸을 더 키울 수 없다) — 글자는 그 칸의 세로 가운데에.
            let ty = ui::mid_y(Rect::new(0.0, y - 1.0, 1.0, row_h));
            let on = row == self.controls_cursor;
            if row < CONTROL_RESET {
                let act = ACTS[row];
                ui.text(
                    &ui.fit(tr(CONTROL_NAMES[row]), name_w - 4.0, FONT_PX),
                    x0 + 2.0,
                    ty,
                    FONT_PX,
                    if on { ui::TEXT } else { ui::TEXT_DIM },
                );
                for col in 0..2 {
                    let r = Rect::new(
                        x0 + name_w + col as f32 * col_w,
                        y - 1.0,
                        col_w - 4.0,
                        row_h,
                    );
                    let sel = on && col == self.controls_col;
                    let waiting = sel && self.rebind.is_some();
                    if sel {
                        ui.rect(r, ui::PANEL_LIGHT);
                        ui.rect_lines(r, ui::ACCENT);
                    }
                    let label = if waiting {
                        fill(s.ctl_wait, &[&(self.rebind.unwrap_or(0.0).ceil() as i32)])
                    } else if col == 0 {
                        b.key_label(act).to_string()
                    } else {
                        b.pad(act).label().to_string()
                    };
                    ui.text(
                        &ui.fit(&label, r.w - 6.0, FONT_PX),
                        r.x + 4.0,
                        ty,
                        FONT_PX,
                        if waiting { ui::ACCENT } else { ui::TEXT },
                    );
                    self.hits.push((r, Hit::ControlCell(row, col)));
                }
            } else {
                let label = if row == CONTROL_RESET {
                    s.ctl_reset
                } else {
                    s.set_back
                };
                let r = Rect::new(x0, y - 1.0, panel.w - 12.0, row_h);
                if on {
                    ui.rect(r, ui::PANEL_LIGHT);
                    ui.text("▶", r.x + 2.0, ty, FONT_PX, ui::ACCENT);
                }
                ui.text(
                    label,
                    r.x + 14.0,
                    ty,
                    FONT_PX,
                    if on { ui::TEXT } else { ui::TEXT_DIM },
                );
                self.hits.push((r, Hit::ControlCell(row, 0)));
            }
        }
        // 안내: 바꾸는 법은 패널 안 아래, 고정된 키는 패널 밖(두 줄까지).
        let hint = ui.fit(s.ctl_hint, panel.w - 4.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() - 12.0, FONT_PX, ui::TEXT_DIM);
        // 두 줄이면 둘째 줄 아래가 화면(180) 밖으로 잘렸다: 창 바로 밑에서 10px 간격(0089).
        let mut y = panel.bottom();
        for l in ui::wrap(ui, s.ctl_fixed, 312.0, FONT_PX).iter().take(2) {
            ui.text_center(l, 160.0, y, FONT_PX, ui::TEXT_DIM);
            y += 11.0;
        }
    }
}

/// 라이선스 창에 보이는 글: 배포물에 함께 싣는 제3자 고지 그대로(`tools/licenses.py`가 만든다). 앱(APK·iOS) 안의 파일은
/// 사람이 열어 볼 수 없어서, OFL "사용자가 쉽게 볼 수 있게"·MIT·Apache 고지를 여기서 보여 준다(2026-10-29).
pub(super) const LICENSE_TEXT: &str = include_str!("../../assets/THIRD_PARTY_LICENSES.txt");

impl App {
    /// 라이선스 창: 위아래 한 줄, 좌우 한 쪽, 끌기·휠, 확인·뒤로·✕로 닫는다.
    pub(super) fn update_licenses(&mut self, a: &Actions, tap_hit: Option<Hit>, back: Back) {
        let n = self.license_lines.len();
        let fits = LICENSE_ROWS;
        let max_first = n.saturating_sub(fits);
        let v = &mut self.license_view;
        if a.up {
            v.first = v.first.saturating_sub(1);
        }
        if a.down {
            v.first = (v.first + 1).min(max_first);
        }
        if a.left {
            v.first = v.first.saturating_sub(fits);
        }
        if a.right {
            v.first = (v.first + fits).min(max_first);
        }
        let mut top = v.first;
        v.scroll(self.scroll_in, &mut top, n);
        // ▲▼는 공용 굴리기(`scroll_in`)가 이미 한 줄 옮겼다: 나머지를 더해 한 쪽씩.
        if let Some(Hit::Scroll(d)) = tap_hit {
            let step = d as i32 * (fits as i32 - 1);
            v.first = (v.first as i32 + step).clamp(0, max_first as i32) as usize;
        }
        if a.cancel || a.confirm || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Settings { back };
        }
    }

    pub(super) fn draw_licenses(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(8.0, 2.0, 304.0, 156.0);
        ui.panel(panel, Some(s.set_licenses));
        self.close_button(ui, panel);
        // 고지 글은 번역하지 않는다: 언어와 상관없이 한국어·라틴 글꼴(Galmuri)로 찍는다.
        let body = ui.with_base_font();
        let w = panel.w - 16.0;
        if self.license_lines.is_empty() {
            self.license_lines = LICENSE_TEXT
                .lines()
                .flat_map(|l| {
                    if l.trim().is_empty() {
                        vec![String::new()]
                    } else {
                        ui::wrap(&body, &l.replace('\t', "    "), w, FONT_PX)
                    }
                })
                .collect();
        }
        let n = self.license_lines.len();
        let row_h = FONT_PX;
        let first = self
            .license_view
            .window(self.license_view.first, n, LICENSE_ROWS, row_h);
        for (i, l) in self
            .license_lines
            .iter()
            .skip(first)
            .take(LICENSE_ROWS)
            .enumerate()
        {
            body.text(
                l,
                panel.x + 8.0,
                panel.y + 18.0 + i as f32 * row_h,
                FONT_PX,
                ui::TEXT,
            );
        }
        // 어디쯤인지(쪽/전체)와 ▲▼(누르면 한 쪽씩).
        let pages = n.div_ceil(LICENSE_ROWS).max(1);
        let page = (first / LICENSE_ROWS + 1).min(pages);
        let pos = format!("{page}/{pages}");
        ui.text(
            &pos,
            panel.right() - 30.0 - ui.text_w(&pos, FONT_PX),
            panel.y + 3.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        if first > 0 {
            let at = vec2(panel.right() - 26.0, panel.y + 3.0);
            ui.text("▲", at.x, at.y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(at.x - 4.0, at.y - 4.0, 16.0, 18.0),
                Hit::Scroll(-1),
            ));
        }
        if first + LICENSE_ROWS < n {
            let at = vec2(panel.right() - 14.0, panel.bottom() - 13.0);
            ui.text("▼", at.x, at.y, FONT_PX, ui::TEXT_DIM);
            self.hits.push((
                Rect::new(at.x - 5.0, at.y - 4.0, 18.0, 18.0),
                Hit::Scroll(1),
            ));
        }
        let hint = vfill(s.licenses_hint, &[Verb::Pick, Verb::Back]);
        ui.hint_strip(&hint, panel, 300.0);
    }
}

/// 라이선스 창에 한 번에 보이는 줄.
const LICENSE_ROWS: usize = 11;
