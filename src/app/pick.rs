//! 누구와 내려갈까(docs/decisions/0056): 마을 뿌리 길에서 Naïl·모모를 고른다. 고른 것은 기록(`char_pick`)에 남는다.

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::cutscene::{Face, Portrait};
use crate::equipment::Character;
use crate::i18n::{self, fill};
use crate::input::{Actions, Verb};
use crate::scenario::Scenario;
use crate::ui::{self, Ui, FONT_PX};

const CHARS: [Character; 2] = [Character::Mole, Character::Momo];

impl App {
    /// 기록에 남은 캐릭터의 칸 번호.
    pub(super) fn picked_index(&self) -> usize {
        usize::from(self.picked() == Character::Momo)
    }

    pub(super) fn update_char_pick(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        if a.left || a.right {
            self.char_cursor = 1 - self.char_cursor;
        }
        let mut go = a.confirm;
        if let Some(Hit::CharCard(i)) = tap_hit {
            if self.char_cursor == i {
                go = true;
            }
            self.char_cursor = i;
        }
        if go {
            self.profile
                .set_count(crate::profile::key::CHAR_PICK, self.char_cursor as u32);
            crate::profile::save(&self.profile);
            self.start(Scenario::Normal);
            return;
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Game;
        }
    }

    pub(super) fn draw_char_pick(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(36.0, 10.0, 248.0, 150.0);
        ui.panel(panel, Some(s.pick_title));
        self.close_button(ui, panel);
        let cw = (panel.w - 18.0) / 2.0;
        for (i, ch) in CHARS.iter().enumerate() {
            let r = Rect::new(
                panel.x + 6.0 + i as f32 * (cw + 6.0),
                panel.y + 20.0,
                cw,
                124.0,
            );
            self.hits.push((r, Hit::CharCard(i)));
            let on = self.char_cursor == i;
            ui.rect(r, if on { ui::PANEL_LIGHT } else { ui::PANEL });
            ui.rect_lines(r, if on { ui::ACCENT } else { ui::BORDER });
            let portrait = match ch {
                Character::Mole => Portrait::nail(crate::journal::fur(&self.profile)),
                Character::Momo => Portrait::MOMO,
            };
            let face = if on { Face::Happy } else { Face::Normal };
            let pr = Rect::new(r.center().x - 22.0, r.y + 4.0, 44.0, 44.0);
            self.renderer.draw_portrait(ui, pr, portrait.frame(face));
            let name = ch.name();
            ui.text_center(
                name,
                r.center().x,
                r.y + 49.0,
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT },
            );
            let desc = match ch {
                Character::Mole => s.pick_nail,
                Character::Momo => s.pick_momo,
            };
            for (k, l) in ui::wrap(ui, desc, r.w - 8.0, FONT_PX)
                .iter()
                .take(3)
                .enumerate()
            {
                ui.text(
                    l,
                    r.x + 4.0,
                    r.y + 63.0 + k as f32 * 12.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
            }
            let clears = self.profile.count(&crate::profile::key::clears(*ch));
            let c = fill(s.pick_clears, &[&clears]);
            let c = ui.fit(&c, r.w - 8.0, FONT_PX);
            ui.text_center(&c, r.center().x, r.bottom() - 14.0, FONT_PX, ui::TEXT_DIM);
        }
        let hint = super::vfill(s.pick_hint, &[Verb::Confirm, Verb::Back]);
        let hint = ui.fit(&hint, 300.0, FONT_PX);
        let hw = ui.text_w(&hint, FONT_PX) + 8.0;
        ui.rect(
            Rect::new(160.0 - hw / 2.0, panel.bottom() + 2.0, hw, FONT_PX + 2.0),
            ui::PANEL,
        );
        ui.text_center(&hint, 160.0, panel.bottom() + 3.0, FONT_PX, ui::TEXT_DIM);
    }
}
