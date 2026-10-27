//! 실의 꿈(docs/decisions/0055): 마을 실 고리에서 고르는 화면, 빈 방에서 보스 다시 싸우기, 기록, 마을로 깨어나기.
//! 고를 거리·기록 키는 `crate::dream`, 싸움은 보통 월드(아레나)를 그대로 쓴다.

use macroquad::prelude::*;

use super::{App, Hit, Scene};
use crate::dream::{self, BOSSES, ROWS};
use crate::enemy::{Enemy, Kind};
use crate::i18n::{self, tr};
use crate::input::{Actions, Verb};
use crate::scenario::Scenario;
use crate::ui::{self, Ui, FONT_PX};

/// 꾸고 있는 꿈 하나.
pub(super) struct DreamRun {
    /// 고른 줄(보스 넷 = 0~3, 이어서 꾸기 = 4).
    row: usize,
    queue: Vec<Kind>,
    at: usize,
    time: f32,
    /// 한 번도 안 맞았나(★).
    clean: bool,
    hp: i32,
    /// 이기거나 쓰러진 뒤 지난 시간(잠깐 보여 주고 깬다).
    end_t: f32,
    won: bool,
}

impl DreamRun {
    pub(super) fn new(row: usize) -> Self {
        Self {
            row,
            queue: dream::queue(row),
            at: 0,
            time: 0.0,
            clean: true,
            hp: 0,
            end_t: 0.0,
            won: false,
        }
    }
}

impl App {
    /// 마을 실 고리: 꿈 고르기 창.
    pub(super) fn open_dream(&mut self) {
        self.dream_cursor = 0;
        self.world.freeze();
        self.scene = Scene::DreamPick;
    }

    pub(super) fn update_dream_pick(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        if self.dream_lock > 0.0 {
            self.dream_lock -= dt;
            return;
        }
        let n = ROWS;
        if a.up {
            self.dream_cursor = (self.dream_cursor + n - 1) % n;
        }
        if a.down {
            self.dream_cursor = (self.dream_cursor + 1) % n;
        }
        let mut go = a.confirm;
        if let Some(Hit::DreamRow(r)) = tap_hit {
            self.dream_cursor = r;
            go = true;
        }
        if go {
            if dream::unlocked(&self.profile, self.dream_cursor) {
                self.dream = Some(DreamRun::new(self.dream_cursor));
                self.start(Scenario::Dream);
                self.sfx.play(crate::sfx::Id::Emerge);
            } else {
                self.sfx.play(crate::sfx::Id::WallHit);
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Game;
        }
    }

    /// `start(Dream)` 끝에서: 꿈 처음부터(다시 시작 포함). 월드는 빈 아레나.
    pub(super) fn begin_dream(&mut self) {
        let Some(run) = self.dream.as_mut() else {
            return;
        };
        let row = run.row;
        *run = DreamRun::new(row);
        self.world.dream = true;
        self.spawn_dream_boss();
        self.world.juice.toast = Some((|s| s.toast_dream_in, 2.0));
    }

    fn spawn_dream_boss(&mut self) {
        let Some(run) = self.dream.as_mut() else {
            return;
        };
        let k = run.queue[run.at];
        let w = &mut self.world;
        let c = w.room.center();
        w.enemies.clear();
        w.projectiles.clear();
        w.hazards.mines.clear();
        w.hazards.clouds.clear();
        w.hazards.rocks.clear();
        w.loot.clear();
        let mut e = Enemy::new(k, c - vec2(0.0, 30.0));
        e.wake_delay = 1.0;
        if k == Kind::Momo {
            e.level = dream::MOMO_LEVEL;
        }
        if !w.seen.contains(&k) {
            w.seen.push(k);
        }
        w.enemies.push(e);
        w.player.pos = c + vec2(0.0, 44.0);
        w.player.prev_pos = w.player.pos;
        w.player.facing = vec2(0.0, -1.0);
        run.hp = w.player.hp;
        w.snap_cam();
    }

    /// 게임 화면 매 프레임: 시간·맞음을 세고, 보스가 쓰러지면 다음 보스나 기록, 쓰러지면 잠깐 뒤 깨어난다.
    pub(super) fn tick_dream(&mut self, dt: f32) {
        let Some(run) = self.dream.as_mut() else {
            return;
        };
        let p = &self.world.player;
        if p.hp < run.hp {
            run.clean = false;
        }
        run.hp = p.hp;
        if !p.is_alive() {
            run.end_t += dt;
            if run.end_t >= 1.6 {
                self.wake_from_dream();
            }
            return;
        }
        if run.won {
            run.end_t += dt;
            if run.end_t >= 1.4 {
                self.wake_from_dream();
            }
            return;
        }
        run.time += dt;
        if self.world.enemies.iter().any(|e| e.is_boss()) {
            return;
        }
        // 보스가 쓰러졌다(대장이 부른 생쥐가 남았으면 같이 흩어진다).
        self.world.enemies.clear();
        self.world.hazards.mines.clear();
        self.world.hazards.clouds.clear();
        self.world.hazards.rocks.clear();
        if run.at + 1 < run.queue.len() {
            run.at += 1;
            let heal = dream::RUSH_HEAL;
            self.world.player.heal(heal);
            self.world.juice.toast = Some((|s| s.toast_dream_next, 2.0));
            self.spawn_dream_boss();
            return;
        }
        run.won = true;
        let (row, secs, clean) = (run.row, run.time, run.clean);
        let best = dream::record(&mut self.profile, row, secs, clean);
        crate::profile::save(&self.profile);
        self.sfx.play(crate::sfx::Id::Heal);
        self.world.juice.toast = Some((
            if best {
                |s| s.toast_dream_best
            } else {
                |s| s.toast_dream_win
            },
            3.0,
        ));
    }

    /// 마을로 깨어난다(꿈 고르기 창을 다시 연다).
    fn wake_from_dream(&mut self) {
        let won = self.dream.as_ref().is_some_and(|r| r.won);
        let row = self.dream.as_ref().map_or(0, |r| r.row);
        self.dream = None;
        self.start(Scenario::Village);
        if !won {
            self.world.juice.toast = Some((|s| s.toast_dream_wake, 2.5));
        }
        // 할매 곁 실 고리 앞에서 깬다.
        if let Some(at) = self.world.dream_loop() {
            self.world.player.pos = at + vec2(0.0, 18.0);
            self.world.player.prev_pos = self.world.player.pos;
            self.world.snap_cam();
        }
        self.open_dream();
        self.dream_cursor = row;
        self.dream_lock = 0.8;
    }

    /// 꿈 중 경과 시간(HUD).
    pub(super) fn draw_dream_hud(&self, ui: &Ui) {
        let Some(run) = self.dream.as_ref() else {
            return;
        };
        // 꿈 빛: 화면 전체에 옅은 보랏빛(보통 굴과 구별되게).
        ui.rect(
            Rect::new(0.0, 0.0, 320.0, 180.0),
            Color::new(0.50, 0.38, 0.78, 0.10),
        );
        let tenths = (run.time * 10.0) as u32;
        let mut t = dream::fmt_time(tenths);
        if run.queue.len() > 1 {
            t = format!("{}/{} · {t}", run.at + 1, run.queue.len());
        }
        if run.clean {
            t = format!("★ {t}");
        }
        ui.text_right(&t, 316.0, 166.0, FONT_PX, ui::TEXT_DIM);
    }

    pub(super) fn draw_dream_pick(&mut self, ui: &Ui) {
        let s = i18n::s();
        let panel = Rect::new(36.0, 8.0, 248.0, 152.0);
        ui.panel(panel, Some(s.dream_title));
        self.close_button(ui, panel);
        let mut y = panel.y + 20.0;
        for l in ui::wrap(ui, s.dream_intro, panel.w - 16.0, FONT_PX)
            .iter()
            .take(2)
        {
            ui.text(l, panel.x + 8.0, y, FONT_PX, ui::TEXT_DIM);
            y += 13.0;
        }
        y += 4.0;
        for row in 0..ROWS {
            let r = Rect::new(panel.x + 6.0, y, panel.w - 12.0, 16.0);
            self.hits.push((r, Hit::DreamRow(row)));
            let open = dream::unlocked(&self.profile, row);
            let on = row == self.dream_cursor;
            if on {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.rect_lines(r, ui::ACCENT);
            }
            let name = match BOSSES.get(row) {
                _ if !open => s.dream_locked.to_string(),
                Some(k) => tr(k.species().name).to_string(),
                None => s.dream_all.to_string(),
            };
            let best = self.profile.count(&dream::key_best(row));
            let rec = if best == 0 {
                s.dream_none.to_string()
            } else {
                dream::fmt_time(best)
            };
            let star = self.profile.flag(&dream::key_clean(row));
            let rw = ui.text_w(&rec, FONT_PX) + 14.0;
            let c = if !open {
                ui::TEXT_DIM
            } else if on {
                ui::ACCENT
            } else {
                ui::TEXT
            };
            let name = ui.fit(&name, r.w - rw - 8.0, FONT_PX);
            ui.text(&name, r.x + 4.0, r.y + 2.0, FONT_PX, c);
            ui.text_right(&rec, r.right() - 3.0, r.y + 2.0, FONT_PX, c);
            if star {
                ui.text("★", r.right() - rw + 2.0, r.y + 2.0, FONT_PX, ui::ACCENT);
            }
            y += 17.0;
        }
        let legend = ui.fit(s.dream_legend, panel.w - 16.0, FONT_PX);
        ui.text(&legend, panel.x + 8.0, y + 2.0, FONT_PX, ui::TEXT_DIM);
        let hint = super::vfill(s.dream_hint, &[Verb::Confirm, Verb::Back]);
        let hint = ui.fit(&hint, 300.0, FONT_PX);
        let hw = ui.text_w(&hint, FONT_PX) + 8.0;
        ui.rect(
            Rect::new(160.0 - hw / 2.0, panel.bottom() + 2.0, hw, FONT_PX + 2.0),
            ui::PANEL,
        );
        ui.text_center(&hint, 160.0, panel.bottom() + 3.0, FONT_PX, ui::TEXT_DIM);
    }
}
