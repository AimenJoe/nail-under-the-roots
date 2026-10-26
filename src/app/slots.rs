//! 저장 슬롯(docs/decisions/0078): 기록(프로필)을 셋까지 따로 둔다. 타이틀의 "이어하기"는 마지막 슬롯,
//! "저장 슬롯" 창에서 슬롯을 골라 이어하거나, 빈 슬롯에서 새로 시작하거나, 지운다(두 번 눌러야 지워진다).

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

    pub(super) fn open_slots(&mut self) {
        // 지금 기록을 먼저 써 둬야 창의 요약이 맞는다.
        crate::profile::save(&self.profile);
        self.refresh_slots();
        self.slots_cursor = profile::slot();
        self.slots_col = 0;
        self.slots_armed = None;
        self.scene = Scene::Slots;
    }

    /// 슬롯 `i`로 바꾸고 그 기록으로 시작한다(비었으면 오프닝부터).
    pub(super) fn play_slot(&mut self, i: usize) {
        profile::set_slot(i);
        self.profile = profile::load_slot(i);
        self.apply_assist();
        self.begin_run();
    }

    fn erase_slot(&mut self, i: usize) {
        let empty = Profile::default();
        if i == profile::slot() {
            self.profile = empty.clone();
            self.apply_assist();
            profile::save(&empty);
        } else {
            profile::save_slot(i, &empty);
        }
        self.refresh_slots();
        self.sfx.play(crate::sfx::Id::Hurt);
    }

    pub(super) fn update_slots(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let filled = self.slot_filled(self.slots_cursor);
        if a.up || a.down {
            self.slots_cursor = if a.up {
                (self.slots_cursor + SLOTS - 1) % SLOTS
            } else {
                (self.slots_cursor + 1) % SLOTS
            };
            self.slots_col = 0;
            self.slots_armed = None;
        }
        if (a.left || a.right) && filled {
            self.slots_col = 1 - self.slots_col;
            self.slots_armed = None;
        }
        let mut go = a.confirm;
        match tap_hit {
            Some(Hit::SlotPlay(i)) => {
                if self.slots_cursor != i || self.slots_col != 0 {
                    self.slots_armed = None;
                }
                self.slots_cursor = i;
                self.slots_col = 0;
                go = true;
            }
            Some(Hit::SlotErase(i)) => {
                if self.slots_cursor != i || self.slots_col != 1 {
                    self.slots_armed = None;
                }
                self.slots_cursor = i;
                self.slots_col = 1;
                go = true;
            }
            _ => {}
        }
        if go {
            let i = self.slots_cursor;
            if self.slots_col == 1 && self.slot_filled(i) {
                if self.slots_armed == Some(i) {
                    self.slots_armed = None;
                    self.slots_col = 0;
                    self.erase_slot(i);
                } else {
                    self.slots_armed = Some(i);
                }
            } else {
                self.play_slot(i);
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
        ui.panel(panel, Some(s.title_slots));
        let current = profile::slot();
        for i in 0..SLOTS {
            let r = Rect::new(
                panel.x + 6.0,
                panel.y + 22.0 + i as f32 * 38.0,
                panel.w - 12.0,
                34.0,
            );
            let on = self.slots_cursor == i;
            ui.rect(r, if on { ui::PANEL_LIGHT } else { ui::PANEL });
            ui.rect_lines(r, if on { ui::ACCENT } else { ui::BORDER });
            let name = fill(s.slot_name, &[&(i + 1)]);
            let name_c = if i == current { ui::ACCENT } else { ui::TEXT };
            ui.text(&name, r.x + 6.0, r.y + 3.0, FONT_PX, name_c);
            let filled = self.slot_filled(i);
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
            let bw = 74.0;
            let line = ui.fit(&summary, r.w - 12.0, FONT_PX);
            ui.text(&line, r.x + 6.0, r.y + 19.0, FONT_PX, ui::TEXT_DIM);
            // 오른쪽 단추: 이어하기(빈 슬롯은 시작) · 지우기.
            let mut buttons = vec![(
                if filled {
                    s.title_continue
                } else {
                    s.title_start
                },
                Hit::SlotPlay(i),
                0,
            )];
            if filled {
                let erase = if self.slots_armed == Some(i) {
                    s.slot_erase_again
                } else {
                    s.slot_erase
                };
                buttons.push((erase, Hit::SlotErase(i), 1));
            }
            let n = buttons.len() as f32;
            for (k, (label, hit, col)) in buttons.into_iter().enumerate() {
                let b = Rect::new(
                    r.right() - 4.0 - (n - k as f32) * (bw + 4.0) + 4.0,
                    r.y + 2.0,
                    bw,
                    15.0,
                );
                let sel = on && self.slots_col == col;
                let warn = col == 1 && self.slots_armed == Some(i);
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
        let hint = super::device_hint(
            s.slots_hint,
            s.title_controls_touch,
            &[Verb::Pick, Verb::Change, Verb::Confirm, Verb::Back],
        );
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
