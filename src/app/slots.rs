//! 저장 슬롯(docs/decisions/0078): 기록(프로필)을 셋까지 따로 둔다.
//! 타이틀 "시작하기" → 슬롯 고르기(빈 슬롯은 시작, 기록이 있으면 두 번 눌러 덮어쓰기),
//! "이어하기"(기록이 있을 때만) → 기록이 있는 슬롯 고르기(이어하기 · 두 번 눌러 지우기).

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::i18n::{self, fill};
use crate::input::{Actions, Verb};
use crate::profile::{self, Profile, SLOTS};
use crate::ui::{self, Ui, FONT_PX};

impl App {
    /// 슬롯마다 비었나(타이틀 줄 고르기). 지금 슬롯은 메모리의 기록, 나머지는 읽어 둔 것.
    pub(super) fn slot_filled(&self, i: usize) -> bool {
        if i == profile::slot() {
            !self.profile.is_empty()
        } else {
            self.slot_cache.get(i).is_some_and(|p| !p.is_empty())
        }
    }

    pub(super) fn refresh_slots(&mut self) {
        self.slot_cache = (0..SLOTS).map(profile::load_slot).collect();
    }

    /// `load`: 이어하기로 열었나(false = 시작하기).
    pub(super) fn open_slots(&mut self, load: bool) {
        // 지금 기록을 먼저 써 둬야 창의 요약이 맞는다.
        crate::profile::save(&self.profile);
        self.refresh_slots();
        let cur = profile::slot();
        self.slots_load = load;
        // 이어하기는 지금(없으면 첫) 기록 있는 슬롯, 시작하기는 첫 빈 슬롯(없으면 지금 슬롯)에서 시작한다.
        self.slots_cursor = if load {
            if self.slot_filled(cur) {
                cur
            } else {
                (0..SLOTS).find(|&i| self.slot_filled(i)).unwrap_or(cur)
            }
        } else {
            (0..SLOTS).find(|&i| !self.slot_filled(i)).unwrap_or(cur)
        };
        self.slots_col = 0;
        self.slots_armed = None;
        self.scene = Scene::Slots;
    }

    /// 슬롯 `i`의 기록으로 이어한다(튜토리얼을 마쳤으면 마을).
    fn continue_slot(&mut self, i: usize) {
        profile::set_slot(i);
        self.profile = profile::load_slot(i);
        self.apply_assist();
        self.begin_run();
    }

    /// 슬롯 `i`를 비우고 처음부터(오프닝).
    fn new_in_slot(&mut self, i: usize) {
        profile::set_slot(i);
        self.profile = Profile::default();
        profile::save(&self.profile);
        self.refresh_slots();
        self.apply_assist();
        self.begin_run();
    }

    fn erase_slot(&mut self, i: usize) {
        let empty = Profile::default();
        if i == profile::slot() {
            self.profile = empty.clone();
            self.apply_assist();
        }
        profile::save_slot(i, &empty);
        self.refresh_slots();
        self.sfx.play(crate::sfx::Id::Hurt);
    }

    /// 한 줄의 단추: (글, 탭 칸, 단추 번호). 이어하기 창의 빈 슬롯은 단추가 없다.
    fn slot_buttons(&self, i: usize) -> Vec<(&'static str, Hit, usize)> {
        let s = i18n::s();
        let armed = self.slots_armed == Some(i);
        let filled = self.slot_filled(i);
        match (self.slots_load, filled) {
            (true, true) => vec![
                (s.title_continue, Hit::SlotPlay(i), 0),
                (
                    if armed {
                        s.slot_erase_again
                    } else {
                        s.slot_erase
                    },
                    Hit::SlotErase(i),
                    1,
                ),
            ],
            (true, false) => Vec::new(),
            (false, false) => vec![(s.title_start, Hit::SlotPlay(i), 0)],
            (false, true) => vec![(
                if armed {
                    s.slot_erase_again
                } else {
                    s.slot_overwrite
                },
                Hit::SlotPlay(i),
                0,
            )],
        }
    }

    /// 위아래로 다음 슬롯. 이어하기 창은 기록 있는 슬롯만 지난다.
    fn step_slot(&self, down: bool) -> usize {
        let mut i = self.slots_cursor;
        for _ in 0..SLOTS {
            i = if down {
                (i + 1) % SLOTS
            } else {
                (i + SLOTS - 1) % SLOTS
            };
            if !self.slots_load || self.slot_filled(i) {
                return i;
            }
        }
        self.slots_cursor
    }

    pub(super) fn update_slots(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        if a.up || a.down {
            self.slots_cursor = self.step_slot(a.down);
            self.slots_col = 0;
            self.slots_armed = None;
        }
        let n = self.slot_buttons(self.slots_cursor).len();
        if (a.left || a.right) && n > 1 {
            self.slots_col = 1 - self.slots_col;
            self.slots_armed = None;
        }
        let mut go = a.confirm;
        let tapped = match tap_hit {
            Some(Hit::SlotPlay(i)) => Some((i, 0)),
            Some(Hit::SlotErase(i)) => Some((i, 1)),
            _ => None,
        };
        if let Some((i, col)) = tapped {
            if self.slots_cursor != i || self.slots_col != col {
                self.slots_armed = None;
            }
            self.slots_cursor = i;
            self.slots_col = col;
            go = true;
        }
        if go {
            let i = self.slots_cursor;
            let filled = self.slot_filled(i);
            match (self.slots_load, filled, self.slots_col) {
                (true, true, 0) => self.continue_slot(i),
                (false, false, _) => self.new_in_slot(i),
                // 지우기·덮어쓰기는 두 번 눌러야 한다.
                (true, true, _) | (false, true, _) if self.slots_armed != Some(i) => {
                    self.slots_armed = Some(i);
                }
                (true, true, _) => {
                    self.slots_armed = None;
                    self.slots_col = 0;
                    self.erase_slot(i);
                    // 이어할 슬롯이 없어지면 타이틀로(이어하기 줄도 사라진다).
                    match (0..SLOTS).find(|&k| self.slot_filled(k)) {
                        Some(k) => self.slots_cursor = k,
                        None => self.scene = Scene::Title,
                    }
                }
                (false, true, _) => {
                    self.slots_armed = None;
                    self.new_in_slot(i);
                }
                _ => {}
            }
            return;
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Title;
        }
    }

    pub(super) fn draw_slots(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(28.0, 14.0, 264.0, 144.0);
        let title = if self.slots_load {
            s.title_continue
        } else {
            s.title_new
        };
        ui.panel(panel, Some(title));
        let current = profile::slot();
        for i in 0..SLOTS {
            let r = Rect::new(
                panel.x + 6.0,
                panel.y + 22.0 + i as f32 * 38.0,
                panel.w - 12.0,
                34.0,
            );
            let on = self.slots_cursor == i;
            let filled = self.slot_filled(i);
            // 이어하기 창의 빈 슬롯은 고를 수 없다: 흐리게.
            let usable = filled || !self.slots_load;
            ui.rect(r, if on { ui::PANEL_LIGHT } else { ui::PANEL });
            ui.rect_lines(r, if on { ui::ACCENT } else { ui::BORDER });
            let name = fill(s.slot_name, &[&(i + 1)]);
            let name_c = if !usable {
                ui::TEXT_DIM
            } else if i == current && filled {
                ui::ACCENT
            } else {
                ui::TEXT
            };
            ui.text(&name, r.x + 6.0, r.y + 3.0, FONT_PX, name_c);
            let summary = if filled {
                let p = if i == current {
                    &self.profile
                } else {
                    &self.slot_cache[i]
                };
                slot_summary(p)
            } else {
                s.slot_empty.to_string()
            };
            // 단추는 첫 줄 오른쪽, 요약은 둘째 줄 전체 폭(요약이 잘리지 않게).
            let line = ui.fit(&summary, r.w - 12.0, FONT_PX);
            ui.text(&line, r.x + 6.0, r.y + 19.0, FONT_PX, ui::TEXT_DIM);
            let bw = 74.0;
            let buttons = self.slot_buttons(i);
            let n = buttons.len() as f32;
            for (k, (label, hit, col)) in buttons.into_iter().enumerate() {
                let b = Rect::new(r.right() - (n - k as f32) * (bw + 4.0), r.y + 2.0, bw, 15.0);
                let sel = on && self.slots_col == col;
                let warn = self.slots_armed == Some(i) && (col == 1 || filled && !self.slots_load);
                ui.rect(b, if sel { ui::PANEL_LIGHT } else { ui::PANEL });
                let edge = if warn {
                    Color::new(0.95, 0.45, 0.40, 1.0)
                } else if sel {
                    ui::ACCENT
                } else {
                    ui::BORDER
                };
                ui.rect_lines(b, edge);
                let label = ui.fit(label, b.w - 4.0, FONT_PX);
                let c = if sel { ui::TEXT } else { ui::TEXT_DIM };
                ui.text_center(&label, b.center().x, b.y + 2.0, FONT_PX, c);
                self.hits
                    .push((Rect::new(b.x, b.y - 2.0, b.w, b.h + 4.0), hit));
            }
        }
        self.close_button(ui, panel);
        // 시작하기 창은 슬롯마다 단추가 하나라 좌우 안내가 없다.
        let hint = if self.slots_load {
            super::device_hint(
                s.slots_hint,
                s.title_controls_touch,
                &[Verb::Pick, Verb::Change, Verb::Confirm, Verb::Back],
            )
        } else {
            super::device_hint(
                s.title_controls,
                s.title_controls_touch,
                &[Verb::Pick, Verb::Confirm, Verb::Back],
            )
        };
        let hint = ui.fit(&hint, 312.0, FONT_PX);
        ui.text_center(&hint, 160.0, 162.0, FONT_PX, ui::TEXT_DIM);
    }
}

/// 슬롯 요약 한 줄: 가장 멀리 간 쉼터 · 끝까지 간 수(두 캐릭터 합) · 내려간 수.
fn slot_summary(p: &Profile) -> String {
    use crate::equipment::Character;
    use crate::profile::key;
    let s = i18n::s();
    let deepest = p.count(key::DEEPEST);
    if deepest == 0 {
        return fill(s.slot_summary_start, &[&p.count(key::RUNS)]);
    }
    let clears = p.count(&key::clears(Character::Mole)) + p.count(&key::clears(Character::Momo));
    fill(s.slot_summary, &[&deepest, &clears, &p.count(key::RUNS)])
}
