//! 설정 창과 조작 설정(다시 붙이기).

use super::*;

/// 설정 줄(0269: 번호 대신 이름 — 줄이 플랫폼마다 달라 번호를 상수로 맞추던 것이 줄을 더할 때마다 밀렸다).
// 빌드마다 없는 줄이 있다(진동은 Android·iOS 앱에 없고, 전체 화면은 데스크톱·웹만) — 그 빌드에선 만들지 않는 갈래.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Row {
    Sound,
    Music,
    Shake,
    /// 번쩍임·맞음 테두리(0269, 흔들림과 따로).
    Flash,
    /// 패드 진동(0269, 웹·PC만 — Android 패드는 패치한 miniquad에 진동이 없다, iOS는 패드가 아직 없다).
    Rumble,
    Effects,
    Tells,
    Debug,
    Language,
    /// 포포의 응원 창(0256).
    Assist,
    /// 전체 화면(데스크톱 0240, 웹은 쓸 수 있는 브라우저만 0269).
    Fullscreen,
    /// 홈 화면에 추가 안내(0269, 웹 — iPhone·iPad의 브라우저 탭에서만): 누르는 줄, 고르면 아래 설명.
    HomeScreen,
    Controls,
    Licenses,
    Erase,
}

/// 이 빌드의 설정 줄 후보, 보이는 순서대로. 값 줄 | 다른 화면으로 가는 줄 | 기록 지우기. 실제로 보이는 줄은 `rows()`.
const ROWS: &[Row] = &[
    Row::Sound,
    Row::Music,
    Row::Shake,
    Row::Flash,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    Row::Rumble,
    Row::Effects,
    Row::Tells,
    Row::Debug,
    Row::Language,
    Row::Assist,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    Row::Fullscreen,
    #[cfg(target_arch = "wasm32")]
    Row::HomeScreen,
    Row::Controls,
    Row::Licenses,
    Row::Erase,
];

/// 지금 보이는 설정 줄(0269): 웹의 전체 화면은 쓸 수 있는 브라우저만, 홈 화면 안내는 iPhone·iPad 브라우저 탭만.
/// 브라우저 성질은 켜는 동안 바뀌지 않아 처음 한 번 정한다.
pub(super) fn rows() -> &'static [Row] {
    static R: std::sync::OnceLock<Vec<Row>> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        ROWS.iter()
            .copied()
            .filter(|r| match r {
                Row::Fullscreen => crate::web::fullscreen_supported(),
                Row::HomeScreen => crate::web::suggest_home_screen(),
                _ => true,
            })
            .collect()
    })
}

impl Row {
    pub(super) fn label(self) -> Tr {
        match self {
            Row::Sound => |s| s.set_sound,
            Row::Music => |s| s.set_music,
            Row::Shake => |s| s.set_shake,
            Row::Flash => |s| s.set_flash,
            Row::Rumble => |s| s.set_rumble,
            Row::Effects => |s| s.set_effects,
            Row::Tells => |s| s.set_tells,
            Row::Debug => |s| s.set_debug,
            Row::Language => |s| s.set_language,
            Row::Assist => |s| s.set_assist,
            Row::Fullscreen => |s| s.set_fullscreen,
            Row::HomeScreen => |s| s.set_home,
            Row::Controls => |s| s.set_controls,
            Row::Licenses => |s| s.set_licenses,
            Row::Erase => |s| s.set_erase,
        }
    }

    /// 값이 아니라 누르는 줄(좌우로 바뀌지 않는다).
    pub(super) fn is_button(self) -> bool {
        matches!(
            self,
            Row::Assist | Row::HomeScreen | Row::Controls | Row::Licenses | Row::Erase
        )
    }

    /// 몇째 묶음인가: 0 = 값(전체 화면·응원 포함), 1 = 다른 화면(조작·라이선스), 2 = 기록 지우기.
    fn group(self) -> usize {
        match self {
            Row::Erase => 2,
            Row::Controls | Row::Licenses => 1,
            _ => 0,
        }
    }
}

/// 설정 줄 `i`(없으면 마지막 줄).
pub(super) fn row_at(i: usize) -> Row {
    let r = rows();
    r[i.min(r.len() - 1)]
}

/// 줄 `r`의 번호(이 빌드에 없으면 0).
pub(super) fn row_index(r: Row) -> usize {
    rows().iter().position(|x| *x == r).unwrap_or(0)
}

/// 설정 창에 한 번에 보이는 줄. 줄이 더 많으면 고른 줄이 보이게 굴린다(입자 줄을 더해 열 줄이 됐다, 0141).
pub(super) const SETTING_VISIBLE: usize = 9;
/// 줄 간격과 묶음 사이 틈(0239): 값 줄 | 다른 화면으로 가는 줄 | 기록 지우기. 아홉 줄 + 틈 둘이 창 안에 든다.
const SETTING_ROW_H: f32 = FONT_PX + 2.0;
const SETTING_GAP: f32 = 4.0;
/// 묶음 사이 가는 선(창 제목 밑줄과 같은 색, 반만 진하게).
const GROUP_LINE: Color = Color::new(0.55, 0.40, 0.28, 0.5);
/// 더 굴릴 수 없는 쪽의 ▲·▼.
const SCROLL_SHUT: Color = Color::new(0.52, 0.48, 0.46, 0.3);
/// 기록 지우기(되돌릴 수 없는 줄)의 글자색. 색만으로 알리지 않는다 — 맨 끝 따로 묶음 + 한 번 더 묻기.
const ERASE_INK: Color = Color::new(0.86, 0.47, 0.42, 1.0);
/// 조작 설정의 줄: 동작들(input::ACTS) + 기본값 + 돌아가기.
pub(super) const CONTROL_NAMES: [Tr; 7] = [
    |s| s.act_attack,
    // 지금 캐릭터의 회피 이름(0265).
    |_| crate::input::dodge_name(),
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
    /// 번쩍임·맞음 테두리(0..=10, 0269 — 흔들림 0이어도 맞은 표시는 남길 수 있게).
    pub flash: u8,
    /// 패드 진동(0..=10, 0269, 웹·PC).
    pub rumble: u8,
    /// 입자 효과(0..=10, 0141). 0이면 먼지·불똥·환경 입자를 그리지 않는다(공격 범위 고리는 남긴다).
    pub effects: u8,
    /// 공격 예고 진하기(1..=10, 0198). 7이 0196·0197의 투명도, 기본 5.
    pub tells: u8,
    pub debug_overlay: bool,
    /// 사람이 고른 언어(None = 기기 언어).
    pub lang: Option<i18n::Lang>,
    /// 전체 화면(데스크톱, 0240). F11로도 바꾼다.
    pub fullscreen: bool,
}

impl Settings {
    pub(super) fn volume_f(&self) -> f32 {
        self.volume as f32 / 10.0
    }
    pub(super) fn shake_f(&self) -> f32 {
        self.shake as f32 / 10.0
    }
    pub(super) fn flash_f(&self) -> f32 {
        self.flash as f32 / 10.0
    }
    pub(super) fn rumble_f(&self) -> f32 {
        self.rumble as f32 / 10.0
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
        let step = |v: u8, lo: i8| (v as i8 + d).clamp(lo, 10) as u8;
        match row_at(row) {
            Row::Sound => {
                self.settings.volume = step(self.settings.volume, 0);
                self.sfx.set_volume(self.settings.volume_f());
                self.amb.set_volume(self.settings.volume_f());
                self.sfx.play(crate::sfx::Id::EnemyHit);
            }
            Row::Music => {
                self.settings.music = step(self.settings.music, 0);
                self.music.set_volume(self.settings.music as f32 / 10.0);
            }
            Row::Shake => self.settings.shake = step(self.settings.shake, 0),
            Row::Flash => self.settings.flash = step(self.settings.flash, 0),
            Row::Rumble => {
                self.settings.rumble = step(self.settings.rumble, 0);
                // 바꾼 세기를 바로 느끼게 한 번 떤다.
                self.input.rumble(0.4 * self.settings.rumble_f(), 120);
            }
            Row::Effects => self.settings.effects = step(self.settings.effects, 0),
            // 0은 두지 않는다: 예고는 놀이 정보라 아예 끄지는 못한다.
            Row::Tells => self.settings.tells = step(self.settings.tells, 1),
            Row::Debug => {
                self.settings.debug_overlay = !self.settings.debug_overlay;
                self.world.debug.overlay = self.settings.debug_overlay;
            }
            Row::Language => {
                let l = i18n::lang().cycle(d);
                i18n::set_lang(l);
                self.settings.lang = Some(l);
            }
            Row::Fullscreen => self.toggle_fullscreen(),
            Row::Assist | Row::HomeScreen | Row::Controls | Row::Licenses | Row::Erase => {}
        }
    }

    /// 전체 화면 ↔ 창(데스크톱 0240): 설정 줄과 F11이 같이 쓴다. 바로 저장한다.
    /// 웹(0269)은 브라우저에 맡긴다(누른 입력에서만, 문서 전체를 올리고 가로 잠금을 시도) — 저장하지 않는다(다음에 켤 때
    /// 브라우저가 스스로 전체 화면을 열어 주지 않는다). 켜졌는지는 `sync_web_fullscreen`이 브라우저를 따라간다.
    pub(super) fn toggle_fullscreen(&mut self) {
        if cfg!(target_arch = "wasm32") {
            crate::web::toggle_fullscreen();
            return;
        }
        self.settings.fullscreen = !self.settings.fullscreen;
        macroquad::window::set_fullscreen(self.settings.fullscreen);
        self.save_prefs();
    }

    /// 웹: 설정의 전체 화면 표시를 브라우저 상태에 맞춘다(Esc·뒤로로 나가면 브라우저가 끈다).
    pub(super) fn sync_web_fullscreen(&mut self) {
        if cfg!(target_arch = "wasm32") {
            self.settings.fullscreen = crate::web::fullscreen_active();
        }
    }

    pub(super) fn save_prefs(&self) {
        crate::prefs::save(&crate::prefs::Prefs {
            volume: self.settings.volume,
            music: self.settings.music,
            shake: self.settings.shake,
            flash: self.settings.flash,
            rumble: self.settings.rumble,
            effects: self.settings.effects,
            tells: self.settings.tells,
            lang: self.settings.lang,
            fullscreen: self.settings.fullscreen,
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
        let pct = |v: u8| format!("{}%", v * 10);
        let on_off = |b: bool| if b { s.on } else { s.off }.to_string();
        let values: Vec<String> = rows()
            .iter()
            .map(|r| match r {
                Row::Sound => pct(self.settings.volume),
                Row::Music => pct(self.settings.music),
                Row::Shake => pct(self.settings.shake),
                Row::Flash => pct(self.settings.flash),
                Row::Rumble => pct(self.settings.rumble),
                Row::Effects => pct(self.settings.effects),
                Row::Tells => pct(self.settings.tells),
                Row::Debug => on_off(self.settings.debug_overlay),
                Row::Language => s.lang_name.to_string(),
                Row::Fullscreen => on_off(self.settings.fullscreen),
                // 다른 창으로 가는 줄·누르는 줄은 값이 없다.
                Row::Assist | Row::HomeScreen | Row::Controls | Row::Licenses | Row::Erase => {
                    String::new()
                }
            })
            .collect();
        let row_h = SETTING_ROW_H;
        // 줄이 창보다 많다: 고른 줄이 보이게 SETTING_VISIBLE줄씩 굴린다(개발 목록과 같은 방식).
        let n = rows().len();
        let first = self
            .settings_view
            .window(self.settings_cursor, n, SETTING_VISIBLE, row_h);
        // 모든 줄은 같은 왼쪽 선에서 시작한다(0239): 예전에는 누르는 줄만 ▶ 자리만큼 들여 썼다.
        let x = panel.x + 6.0;
        let w = panel.w - 12.0;
        let mut y = panel.y + 18.0;
        for (i, row) in rows().iter().enumerate().skip(first).take(SETTING_VISIBLE) {
            let label = tr(row.label());
            if i > first && row.group() != rows()[i - 1].group() {
                let ly = (y + SETTING_GAP * 0.5).round();
                ui.line(vec2(x + 4.0, ly), vec2(x + w - 4.0, ly), GROUP_LINE);
                y += SETTING_GAP;
            }
            let selected = self.settings_cursor == i;
            if row.is_button() {
                let r = Rect::new(x, y, w, FONT_PX + 4.0);
                if selected {
                    ui.rect(r, ui::PANEL_LIGHT);
                }
                let ty = ui::mid_y(r);
                let erase = *row == Row::Erase;
                let label = if erase && self.erase_armed {
                    s.set_erase_confirm
                } else {
                    label
                };
                let ink = match (erase, selected) {
                    (true, _) => ERASE_INK,
                    (false, true) => ui::TEXT,
                    (false, false) => ui::TEXT_DIM,
                };
                // 다른 화면으로 가는 줄은 값 칸의 ▶ 자리에 →(가는 곳이 있다는 표시). 기록 지우기는 표시 없음.
                let arrow_w = if erase { 0.0 } else { 12.0 };
                if !erase {
                    let ac = if selected { ui::ACCENT } else { ui::TEXT_DIM };
                    ui.text("→", x + w - 4.0 - arrow_w, ty, FONT_PX, ac);
                }
                let label = ui.fit(label, w - 8.0 - arrow_w - 4.0, FONT_PX);
                ui.text(&label, x + 4.0, ty, FONT_PX, ink);
                self.hits.push((r, Hit::SettingRow(i)));
            } else {
                let (row, l, r) = ui.setting_row(label, &values[i], x, y, w, selected);
                self.hits.push((l, Hit::SettingLeft(i)));
                self.hits.push((r, Hit::SettingRight(i)));
                self.hits.push((row, Hit::SettingRow(i)));
            }
            y += row_h;
        }
        // 줄이 창보다 많으면 제목 줄의 닫기 단추 옆에 ▲▼ 한 쌍(0239). 더 갈 수 없는 쪽은 흐리게, 누르면 한 줄 굴린다(터치, 0198).
        // 예전에는 ▼가 창 오른쪽 아래에 있어 마지막 줄의 ▶·→와 겹쳤다.
        if n > SETTING_VISIBLE {
            for (glyph, dx, step, open) in [
                ("▲", 44.0, -1, first > 0),
                ("▼", 30.0, 1, first + SETTING_VISIBLE < n),
            ] {
                let at = vec2(panel.right() - dx, panel.y + 4.0);
                let c = if open { ui::TEXT_DIM } else { SCROLL_SHUT };
                ui.text(glyph, at.x, at.y, FONT_PX, c);
                if open {
                    self.hits.push((
                        Rect::new(at.x - 2.0, at.y - 4.0, 14.0, 18.0),
                        Hit::Scroll(step),
                    ));
                }
            }
        }
        // 홈 화면 줄을 고르면 아래 안내 자리에 그 설명(0269 D4: 홈 화면 앱은 Safari 기록이 따라오지 않는다).
        let settings_hint = if row_at(self.settings_cursor) == Row::HomeScreen {
            s.set_home_d.to_string()
        } else {
            vfill(s.settings_hint, &[Verb::Change, Verb::Back])
        };
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

    /// 포포의 응원 창(0256): 위아래로 항목, 좌우로 값, 바꾸면 바로 기록에 쓰고 월드에 넣는다. 뒤로·닫기로 설정.
    pub(super) fn update_assist(&mut self, a: &Actions, tap_hit: Option<Hit>, back: Back) {
        let n = crate::assist::ROWS;
        if a.up {
            self.assist_cursor = (self.assist_cursor + n - 1) % n;
        }
        if a.down {
            self.assist_cursor = (self.assist_cursor + 1) % n;
        }
        let mut adjust = 0i8;
        if a.left {
            adjust = -1;
        }
        if a.right || a.confirm {
            adjust = 1;
        }
        match tap_hit {
            Some(Hit::SettingLeft(i)) => {
                self.assist_cursor = i;
                adjust = -1;
            }
            Some(Hit::SettingRight(i)) | Some(Hit::SettingRow(i)) => {
                self.assist_cursor = i;
                adjust = 1;
            }
            _ => {}
        }
        if adjust != 0 {
            let mut v = self.assist();
            v.step(self.assist_cursor, adjust);
            if v != self.assist() {
                v.write(&mut self.profile);
                crate::profile::save(&self.profile);
                self.apply_assist();
                self.sfx.play(crate::sfx::Id::Claw(0));
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Settings { back };
        }
    }

    pub(super) fn draw_assist(&mut self, ui: &Ui) {
        use crate::assist::{PARRY, RESIST_FIXED, RESIST_SCALING, TELLS};
        let panel = Rect::new(56.0, 0.0, 208.0, 155.0);
        let s = i18n::s();
        ui.panel(panel, Some(s.set_assist));
        self.close_button(ui, panel);
        let v = self.assist();
        let more = |k: f32| {
            if k <= 1.0 {
                s.off.to_string()
            } else {
                format!("+{}%", ((k - 1.0) * 100.0).round())
            }
        };
        let resist = if v.resist >= RESIST_SCALING {
            let pct = (self.assist_resist() * 100.0).round() as i32;
            fill(s.ast_scaling, &[&pct])
        } else if v.resist == 0 {
            s.off.to_string()
        } else {
            format!("-{}%", (RESIST_FIXED[v.resist] * 100.0).round())
        };
        let rows: [(&str, String, &str); crate::assist::ROWS] = [
            (s.ast_resist, resist, s.ast_resist_d),
            (
                s.ast_go_on,
                if v.go_on { s.on } else { s.off }.to_string(),
                s.ast_go_on_d,
            ),
            (s.ast_tells, more(TELLS[v.tells]), s.ast_tells_d),
            (s.ast_parry, more(PARRY[v.parry]), s.ast_parry_d),
        ];
        let x = panel.x + 6.0;
        let w = panel.w - 12.0;
        let mut y = panel.y + 18.0;
        for (i, (label, value, _)) in rows.iter().enumerate() {
            let (row, l, r) = ui.setting_row(label, value, x, y, w, self.assist_cursor == i);
            self.hits.push((l, Hit::SettingLeft(i)));
            self.hits.push((r, Hit::SettingRight(i)));
            self.hits.push((row, Hit::SettingRow(i)));
            y += SETTING_ROW_H;
        }
        // 고른 항목이 하는 일(한 줄 설명, 창 안에서 줄바꿈).
        let ly = (y + SETTING_GAP * 0.5).round();
        ui.line(vec2(x + 4.0, ly), vec2(x + w - 4.0, ly), GROUP_LINE);
        y += SETTING_GAP + 2.0;
        let desc = rows[self.assist_cursor.min(rows.len() - 1)].2;
        let lines = ui::wrap(ui, desc, w - 8.0, FONT_PX);
        let room = ((panel.bottom() - 3.0 - y) / (FONT_PX + 1.0))
            .floor()
            .max(1.0) as usize;
        for (k, l) in lines.iter().take(room).enumerate() {
            ui.text(
                l,
                x + 4.0,
                y + k as f32 * (FONT_PX + 1.0),
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        let hint = vfill(s.settings_hint, &[Verb::Change, Verb::Back]);
        for (i, l) in ui::wrap(ui, &hint, 300.0, FONT_PX)
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
        // 걷는 키 이름은 지금 배열(AZERTY면 ZQSD, 0282).
        let fixed = fill(s.ctl_fixed, &[&crate::input::walk_keys()]);
        for l in ui::wrap(ui, &fixed, 312.0, FONT_PX).iter().take(2) {
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
