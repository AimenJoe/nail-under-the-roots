//! 누구와 내려갈까(docs/decisions/0056): 마을 뿌리 길에서 고를 수 있는 캐릭터(표 순서)를 고른다. 고른 것은 기록(`char_pick`)에 남는다.

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::characters::CharacterDef;
use crate::cutscene::Face;
use crate::i18n::{self, fill};
use crate::input::{Actions, Verb};
use crate::scenario::Scenario;
use crate::ui::{self, Ui, FONT_PX};

impl App {
    /// 고르기 창의 칸들: 이 기록으로 고를 수 있는 캐릭터만(잠긴 캐릭터는 칸이 없다).
    pub(super) fn pick_list(&self) -> Vec<&'static CharacterDef> {
        crate::characters::available(&self.profile).collect()
    }

    /// 기록에 남은 캐릭터의 칸 번호.
    pub(super) fn picked_index(&self) -> usize {
        let ch = self.picked();
        self.pick_list()
            .iter()
            .position(|d| d.id == ch)
            .unwrap_or(0)
    }

    pub(super) fn update_char_pick(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let list = self.pick_list();
        let n = list.len().max(1);
        if a.left {
            self.char_cursor = (self.char_cursor + n - 1) % n;
        } else if a.right {
            self.char_cursor = (self.char_cursor + 1) % n;
        }
        let mut go = a.confirm;
        if let Some(Hit::CharCard(i)) = tap_hit {
            if self.char_cursor == i {
                go = true;
            }
            self.char_cursor = i;
        }
        if go {
            self.profile.set_count(
                crate::profile::key::CHAR_PICK,
                list[self.char_cursor.min(list.len() - 1)].code as u32,
            );
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
        // 둘이면 예전 크기, 셋부터 화면 폭 가까이·조금 높게(소개 다섯째 줄까지, 0195).
        let list = self.pick_list();
        let n = list.len().max(1);
        let (w, h) = if n <= 2 {
            (248.0, 150.0)
        } else {
            (316.0, 162.0)
        };
        let panel = Rect::new(160.0 - w / 2.0, if n <= 2 { 10.0 } else { 2.0 }, w, h);
        // (소개 줄 수, 소개 첫 줄, 카드 아래 여백, 끝까지 줄의 아래에서 거리)
        let (lines, desc_y, foot, clears_up) = if n <= 2 {
            (3, 63.0, 26.0, 14.0)
        } else {
            (5, 62.0, 23.0, 12.0)
        };
        // 일본어·중국어는 금칙으로 줄 끝에 매단 구두점(。、)이 한 글자 더 나간다 — 좁은 셋 칸에서는 그만큼 좁게 끊는다.
        let cjk = matches!(
            i18n::lang(),
            i18n::Lang::Ja | i18n::Lang::ZhHans | i18n::Lang::ZhHant
        );
        let hang = if n > 2 && cjk { FONT_PX } else { 0.0 };
        ui.panel(panel, Some(s.pick_title));
        self.close_button(ui, panel);
        let cw = (panel.w - 6.0 * (n as f32 + 1.0)) / n as f32;
        for (i, d) in list.iter().enumerate() {
            let r = Rect::new(
                panel.x + 6.0 + i as f32 * (cw + 6.0),
                panel.y + 20.0,
                cw,
                panel.h - foot,
            );
            self.hits.push((r, Hit::CharCard(i)));
            let on = self.char_cursor == i;
            ui.rect(r, if on { ui::PANEL_LIGHT } else { ui::PANEL });
            ui.rect_lines(r, if on { ui::ACCENT } else { ui::BORDER });
            let portrait = (d.portrait)(crate::journal::fur(&self.profile));
            let face = if on { Face::Happy } else { Face::Normal };
            let pr = Rect::new(r.center().x - 22.0, r.y + 4.0, 44.0, 44.0);
            self.renderer.draw_portrait(ui, pr, portrait.frame(face));
            let name = d.id.name();
            ui.text_center(
                name,
                r.center().x,
                r.y + 49.0,
                FONT_PX,
                if on { ui::ACCENT } else { ui::TEXT },
            );
            let desc = (d.pick_text)(s);
            for (k, l) in ui::wrap(ui, desc, r.w - 8.0 - hang, FONT_PX)
                .iter()
                .take(lines)
                .enumerate()
            {
                ui.text(
                    l,
                    r.x + 4.0,
                    r.y + desc_y + k as f32 * 12.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
            }
            let clears = self.profile.count(&crate::profile::key::clears(d.id));
            let c = fill(s.pick_clears, &[&clears]);
            let c = ui.fit(&c, r.w - 8.0, FONT_PX);
            ui.text_center(
                &c,
                r.center().x,
                r.bottom() - clears_up,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        let hint = super::vfill(s.pick_hint, &[Verb::Confirm, Verb::Back]);
        ui.hint_strip(&hint, panel, 300.0);
    }
}
