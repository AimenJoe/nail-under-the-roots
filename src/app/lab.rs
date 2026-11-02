//! 일시정지의 "Lab" 탭(0269 R33, 요구 2.8A, 개발판만): 손잡이(`crate::lab::KNOBS`)를 보고 바꾼다.
//! 위아래 = 줄 고르기, 좌우 = 한 칸, 확인 = 프리셋 줄에서 실행. 탭을 닫고 이어 하면 바뀐 값으로 바로 돈다.
//! 개발 화면이라 번역하지 않는다(0013) — 한국어 + 상수 이름.

use super::*;
use crate::lab::{self, KNOBS};

/// 탭 상태.
#[derive(Default)]
pub struct LabUi {
    cursor: usize,
    view: crate::ui::ListView,
    /// 프리셋 칸(0..LAB_SLOTS).
    slot: usize,
    /// 마지막 한 일(화면 아래 한 줄).
    note: String,
}

/// 맨 위 실행 줄: 프리셋 불러오기·저장·모두 기본값.
const ACTIONS: [&str; 3] = ["불러오기", "저장", "모두 기본값"];

impl App {
    pub(super) fn update_lab(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let n = ACTIONS.len() + KNOBS.len();
        let l = &mut self.lab;
        if a.up {
            l.cursor = (l.cursor + n - 1) % n;
        }
        if a.down {
            l.cursor = (l.cursor + 1) % n;
        }
        if let Some(Hit::LabItem(i)) = tap_hit {
            l.cursor = i;
        }
        l.view.scroll(self.scroll_in, &mut l.cursor, n);
        let dir = (a.right as i32 - a.left as i32) as f32;
        let tapped = |h: Hit| tap_hit == Some(h);
        let i = l.cursor;
        if i < ACTIONS.len() {
            // 프리셋 칸은 좌우로 고른다.
            if dir != 0.0 {
                let k = crate::profile::LAB_SLOTS as i32;
                l.slot = (l.slot as i32 + dir as i32).rem_euclid(k) as usize;
            }
            if a.confirm || tapped(Hit::LabItem(i)) {
                l.note = match i {
                    0 => {
                        let text = crate::profile::lab_read(l.slot);
                        let got = lab::dev::load(&text);
                        format!("칸 {} 불러옴 — {got}줄", l.slot + 1)
                    }
                    1 => {
                        let text = lab::encode(&lab::dev::snapshot());
                        let ok = crate::profile::lab_write(l.slot, &text);
                        format!(
                            "칸 {} {}",
                            l.slot + 1,
                            if ok { "저장함" } else { "저장 실패" }
                        )
                    }
                    _ => {
                        lab::dev::reset_all();
                        "모두 상수 값으로".to_string()
                    }
                };
            }
        } else if dir != 0.0 {
            lab::dev::nudge(i - ACTIONS.len(), dir);
        } else if let Some(Hit::LabNudge(k, d)) = tap_hit {
            self.lab.cursor = ACTIONS.len() + k;
            lab::dev::nudge(k, d as f32);
        }
    }

    pub(super) fn draw_lab(&mut self, ui: &Ui, c: Rect) {
        let base = ui.with_base_font();
        let ui = &base;
        let row_h = FONT_PX + 3.0;
        let fits = (((c.h - 14.0) / row_h).floor() as usize).max(1);
        let n = ACTIONS.len() + KNOBS.len();
        let l = &mut self.lab;
        let first = l.view.window(l.cursor, n, fits, row_h);
        for (row, i) in (first..n).take(fits).enumerate() {
            let r = Rect::new(c.x, c.y + row as f32 * row_h, c.w, row_h - 1.0);
            let on = i == l.cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
            }
            let col = if on { ui::ACCENT } else { ui::TEXT };
            if i < ACTIONS.len() {
                let label = if i < 2 {
                    format!("프리셋 ◀ {} ▶  {}", l.slot + 1, ACTIONS[i])
                } else {
                    ACTIONS[i].to_string()
                };
                ui.text(&label, r.x + 3.0, ui::mid_y(r), FONT_PX, col);
            } else {
                let k = i - ACTIONS.len();
                let knob = &KNOBS[k];
                // 묶음 첫 줄이면 묶음 이름을 앞에.
                let head = k == 0 || KNOBS[k - 1].group != knob.group;
                let name = if head {
                    format!("[{}] {}", knob.group.name(), knob.name)
                } else {
                    format!("    {}", knob.name)
                };
                ui.text(
                    &ui.fit(&name, c.w * 0.62, FONT_PX),
                    r.x + 3.0,
                    ui::mid_y(r),
                    FONT_PX,
                    col,
                );
                let v = lab::dev::value(k);
                let changed = lab::dev::changed(k);
                let val = if changed {
                    format!("{v:.3} (기본 {:.3})", knob.default)
                } else {
                    format!("{v:.3}")
                };
                let vx = c.x + c.w * 0.64;
                ui.text(
                    &val,
                    vx,
                    ui::mid_y(r),
                    FONT_PX,
                    if changed { ui::ACCENT } else { ui::TEXT_DIM },
                );
                // 터치: 값 왼쪽 반 = 한 칸 내림, 오른쪽 반 = 한 칸 올림.
                let vw = c.right() - vx;
                self.hits
                    .push((Rect::new(vx, r.y, vw / 2.0, r.h), Hit::LabNudge(k, -1)));
                self.hits.push((
                    Rect::new(vx + vw / 2.0, r.y, vw / 2.0, r.h),
                    Hit::LabNudge(k, 1),
                ));
            }
            self.hits.push((r, Hit::LabItem(i)));
        }
        let foot = if l.note.is_empty() {
            "위아래 고르기 · 좌우 한 칸 · 확인 실행 · 운영판엔 없음".to_string()
        } else {
            l.note.clone()
        };
        ui.text(&foot, c.x + 3.0, c.bottom() - 6.0, FONT_PX, ui::TEXT_DIM);
    }
}
