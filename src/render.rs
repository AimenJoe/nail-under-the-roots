//! 그리기. 월드는 320×180 렌더 타깃에, HUD 텍스트는 실제 해상도에.
//! 월드 좌표는 그릴 때 반올림한다. (KB: reference/macroquad/pixel-perfect-rendering.md)

use macroquad::prelude::*;

use crate::art::{self, Art};
use crate::biome::{self, Decor};
use crate::combat::{Shape, StrikeKind};
use crate::enemy::{self, Enemy};
use crate::input::TouchOverlay;
use crate::loot;
use crate::map::Cell;
use crate::player::{self, Player};
use crate::room::PropKind;
use crate::tuning as t;
use crate::ui::Ui;
use crate::world::{self, FxKind, World};

const VW: u32 = t::VIEW_W as u32;
const VH: u32 = t::VIEW_H as u32;

const BG: Color = Color::new(0.12, 0.09, 0.14, 1.0);
const GRID: Color = Color::new(0.16, 0.12, 0.19, 1.0);
const WALL_COLOR: Color = Color::new(0.30, 0.22, 0.20, 1.0);
const SHADOW: Color = Color::new(0.05, 0.03, 0.06, 0.8);

const SHADOW_SOFT: Color = Color::new(0.05, 0.03, 0.08, 0.35);

/// 언어별 폰트(i18n::Lang::font_file). 시작할 때 전부 읽는다(일본어·중국어는 서브셋이라 100KB 안쪽).
pub struct Fonts {
    pub base: Option<Font>,
    pub by_file: Vec<(&'static str, Font)>,
}

impl Fonts {
    fn for_lang(&self, l: crate::i18n::Lang) -> Option<Font> {
        let file = l.font_file();
        self.by_file
            .iter()
            .find(|(f, _)| *f == file)
            .map(|(_, font)| font.clone())
            .or_else(|| self.base.clone())
    }
}

pub struct Renderer {
    rt: RenderTarget,
    cam: Camera2D,
    fonts: Fonts,
    art: Option<Art>,
}

impl Renderer {
    pub fn new(fonts: Fonts, art: Option<Art>) -> Self {
        let rt = render_target(VW, VH);
        rt.texture.set_filter(FilterMode::Nearest);
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H));
        cam.render_target = Some(rt.clone());
        Self {
            rt,
            cam,
            fonts,
            art,
        }
    }

    /// 월드 패스(내부 해상도). `w`가 없으면 타이틀 배경만 그린다.
    /// `alpha`: 마지막 틱 이후 지난 비율(0..1), 위치 보간. `shake_mul`: 설정의 흔들림 배율.
    pub fn begin_world(&mut self, w: Option<&World>, alpha: f32, shake_mul: f32) {
        let center = vec2(t::VIEW_W / 2.0, t::VIEW_H / 2.0);
        let shake = w
            .map(|w| (w.feel.shake_offset * shake_mul).round())
            .unwrap_or(Vec2::ZERO);
        // 모험: 카메라 왼쪽 위(보간, 정수 px). 아레나·타이틀은 0.
        let cam = w
            .map(|w| lerp(w.adv.prev_cam, w.adv.cam, alpha))
            .unwrap_or(Vec2::ZERO);
        self.cam.target = cam + center + shake;
        set_camera(&self.cam);
        let Some(w) = w else {
            clear_background(BG);
            draw_title_background();
            return;
        };
        if let Some(art) = &self.art {
            clear_background(w.biome.clear());
            if w.room.grid.is_some() {
                draw_grid_art(art, w, cam);
            } else {
                draw_room_art(art, w);
            }
            draw_drill_trail(Some(art), &w.player, alpha);
            draw_sorted(art, w, alpha);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_darkness(art, w, alpha, cam);
            if draw_super_dim(w, cam) {
                draw_nail(art, &w.player, alpha, w.time);
                draw_player_fx(&w.player, alpha);
            }
        } else {
            clear_background(BG);
            draw_room(w);
            for p in &w.projectiles {
                draw_projectile(p, alpha);
            }
            for e in &w.enemies {
                draw_enemy(None, e, alpha, w.time);
            }
            draw_drill_trail(None, &w.player, alpha);
            draw_player(&w.player, alpha, w.time);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            if draw_super_dim(w, cam) {
                draw_player(&w.player, alpha, w.time);
                draw_player_fx(&w.player, alpha);
            }
        }
        if w.debug.hitboxes {
            draw_hitboxes(w, alpha);
        }
        // HUD는 화면에 고정(흔들림·스크롤 없이).
        self.cam.target = center;
        set_camera(&self.cam);
        draw_hud_world(w);
        if w.feel.flash > 0.0 {
            // 벽 쿵 번쩍임. 흔들림 설정을 따른다(0이면 끈다: 눈이 예민한 사람).
            let k = (w.feel.flash / t::DRILL_WALL_FLASH).min(1.0) * shake_mul.min(1.0);
            draw_rectangle(
                0.0,
                0.0,
                t::VIEW_W,
                t::VIEW_H,
                Color::new(1.0, 0.97, 0.9, 0.55 * k),
            );
        }
        let fade = w.fade_alpha();
        if fade > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                t::VIEW_W,
                t::VIEW_H,
                Color::new(0.0, 0.0, 0.0, fade),
            );
        }
    }

    /// 화면 패스: 렌더 타깃을 정수 배율 + 레터박스로 올리고, 그 배치를 돌려준다.
    pub fn present(&self) -> Ui {
        set_default_camera();
        clear_background(BLACK);
        let scale = (screen_width() / t::VIEW_W)
            .min(screen_height() / t::VIEW_H)
            .floor()
            .max(1.0);
        let (dw, dh) = (t::VIEW_W * scale, t::VIEW_H * scale);
        let x = ((screen_width() - dw) / 2.0).floor();
        let y = ((screen_height() - dh) / 2.0).floor();
        draw_texture_ex(
            &self.rt.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                // macroquad 0.4.16: from_display_rect + render_target 조합은 상하가 뒤집혀 나온다(실행 확인 2026-10-24).
                flip_y: true,
                ..Default::default()
            },
        );
        Ui {
            scale,
            ox: x,
            oy: y,
            font: self.fonts.for_lang(crate::i18n::lang()),
            base: self.fonts.base.clone(),
        }
    }

    /// 컷씬 배우(그림이 있는 이웃). 그림이 없으면 색 동그라미.
    pub fn draw_actor_sprite(&self, row: u32, pos: Vec2, facing: Vec2, moving: bool, color: Color) {
        match self.art.as_ref().and_then(|a| a.critters.as_ref()) {
            Some(tex) => draw_critter_at(tex, row, pos, facing.x < 0.0, moving, get_time() as f32),
            None => draw_actor(pos, facing, color),
        }
    }

    /// 도감 그림: critters.png의 `row`줄 `frame`칸을 논리 사각형 `r`에 정수 배율로(가운데 맞춤).
    pub fn draw_critter(&self, ui: &Ui, r: Rect, row: u32, frame: u32) {
        let Some(tex) = self.art.as_ref().and_then(|a| a.critters.as_ref()) else {
            return;
        };
        let k = ((r.w / art::CELL).floor() * ui.scale).max(1.0);
        let size = art::CELL * k;
        let p = ui.px(vec2(r.x, r.y));
        let (bw, bh) = (r.w * ui.scale, r.h * ui.scale);
        let x = (p.x + (bw - size) / 2.0).round();
        let y = (p.y + (bh - size) / 2.0).round();
        draw_texture_ex(
            tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(art::cell(frame, row)),
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }

    /// 대화창 초상화를 논리 사각형 `r`(44×44) 안에 찍는다. 원본 1px = 게임 픽셀 반(화면 배율의 절반, 정수).
    /// 배율이 홀수면 내림이라 조금 작아지지만 픽셀은 고르게 남는다. 그림이 없으면 false.
    pub fn draw_portrait(&self, ui: &Ui, r: Rect, frame: u32) -> bool {
        let Some(tex) = self.art.as_ref().and_then(|a| a.portraits.as_ref()) else {
            return false;
        };
        let k = (ui.scale / 2.0).floor().max(1.0);
        let size = art::PORTRAIT * k;
        let p = ui.px(vec2(r.x, r.y));
        let (bw, bh) = (r.w * ui.scale, r.h * ui.scale);
        draw_texture_ex(
            tex,
            (p.x + (bw - size) / 2.0).round(),
            (p.y + (bh - size)).round(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                source: Some(Rect::new(
                    frame as f32 * art::PORTRAIT,
                    0.0,
                    art::PORTRAIT,
                    art::PORTRAIT,
                )),
                ..Default::default()
            },
        );
        true
    }

    /// 화면 가장자리 비네트(컷씬). 게임 화면 위에 타원형으로 덮는다. 그림 에셋이 없으면 생략.
    pub fn draw_vignette(&self, ui: &Ui, amount: f32) {
        let Some(art) = &self.art else { return };
        if amount <= 0.0 {
            return;
        }
        let (w, h) = (t::VIEW_W * ui.scale, t::VIEW_H * ui.scale);
        // 마스크는 가운데 35% 반지름까지 투명. 화면을 조금 넘게 늘려 모서리가 가장 어둡게.
        let (dw, dh) = (w * 1.25, h * 1.45);
        draw_texture_ex(
            &art.dark,
            ui.ox + (w - dw) / 2.0,
            ui.oy + (h - dh) / 2.0,
            Color::new(1.0, 1.0, 1.0, amount),
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                ..Default::default()
            },
        );
    }

    /// 인게임 HUD(실제 해상도): 반딧불 수, 웨이브·처치, 디버그 줄.
    pub fn draw_hud(&self, ui: &Ui, w: &World, gamepad_count: usize) {
        // 반딧불: 체력 핍 오른쪽. 숫자는 언어 폰트(숫자는 모든 폰트에 있다). 글자 윗부분이 화면 밖으로 잘리지 않게 y ≥ 1.
        let x = 3.0 + w.player.stats.max_hp as f32 * 7.0 + 5.0;
        let pulse = 0.6 + 0.4 * (w.time * 3.0).sin().abs();
        ui.rect(
            Rect::new(x, 2.0, 3.0, 3.0),
            Color::new(0.85, 1.0, 0.45, pulse),
        );
        ui.rect(Rect::new(x, 3.0, 3.0, 1.0), OUTLINE);
        ui.text(
            &w.glowbugs.to_string(),
            x + 5.0,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        // 씨앗(상점 화폐): 반딧불 오른쪽.
        let sx = x + 7.0 + ui.text_w(&w.glowbugs.to_string(), crate::ui::FONT_PX) + 6.0;
        ui.rect(Rect::new(sx, 1.0, 3.0, 4.0), OUTLINE);
        ui.rect(Rect::new(sx + 0.5, 1.5, 2.0, 3.0), world::SEED);
        ui.rect(
            Rect::new(sx + 1.0, 0.0, 2.0, 1.0),
            Color::new(0.45, 0.78, 0.35, 1.0),
        );
        ui.text(
            &w.seeds.to_string(),
            sx + 5.0,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        draw_quick_slots(ui, w);
        if let Some((text, time)) = w.toast {
            let s = crate::i18n::tr(text);
            let a = (time / 0.4).min(1.0);
            let tw = ui.text_w(s, crate::ui::FONT_PX) + 14.0;
            let r = Rect::new(160.0 - tw / 2.0, 26.0, tw, crate::ui::FONT_PX + 6.0);
            ui.rect(r, Color::new(0.07, 0.05, 0.09, 0.9 * a));
            ui.rect_lines(r, Color::new(0.90, 0.71, 0.47, a));
            ui.text_center(
                s,
                160.0,
                r.y + 3.0,
                crate::ui::FONT_PX,
                Color::new(0.92, 0.86, 0.76, a),
            );
        }
        draw_hud_screen(w, ui.scale, ui.ox, ui.oy, gamepad_count);
    }
}

/// 퀵 슬롯(왼쪽 아래): 산딸기·지렁이 두 칸, 고른 칸은 테두리 강조, 개수, 먹기 키 안내.
fn draw_quick_slots(ui: &Ui, w: &World) {
    use crate::items::Consumable;
    let (x0, y0, size) = (4.0, 156.0, 18.0);
    for (i, c) in Consumable::ALL.iter().enumerate() {
        let r = Rect::new(x0 + i as f32 * (size + 3.0), y0, size, size);
        let n = w.bag.count(*c);
        let selected = w.bag.selected == *c;
        ui.rect(r, Color::new(0.07, 0.05, 0.09, 0.8));
        ui.rect_lines(
            r,
            if selected {
                crate::ui::ACCENT
            } else {
                crate::ui::BORDER
            },
        );
        let dim = if n == 0 { 0.35 } else { 1.0 };
        let center = ui.px(vec2(r.x + size / 2.0, r.y + size / 2.0 - 1.0));
        let k = ui.scale;
        match c {
            Consumable::Berry => {
                let s = k;
                draw_rectangle(
                    center.x - 2.0 * s,
                    center.y - 2.0 * s,
                    4.0 * s,
                    5.0 * s,
                    OUTLINE,
                );
                draw_rectangle(
                    center.x - 1.5 * s,
                    center.y - 1.5 * s,
                    3.0 * s,
                    4.0 * s,
                    Color::new(0.93 * dim, 0.30 * dim, 0.38 * dim, 1.0),
                );
                draw_rectangle(
                    center.x - 1.0 * s,
                    center.y - 3.0 * s,
                    2.0 * s,
                    1.0 * s,
                    Color::new(0.45 * dim, 0.75 * dim, 0.35 * dim, 1.0),
                );
            }
            Consumable::Worm => {
                if n > 0 {
                    draw_worm(center, w.time * 6.0, k);
                } else {
                    draw_rectangle(
                        center.x - 4.0 * k,
                        center.y - 0.5 * k,
                        8.0 * k,
                        2.0 * k,
                        Color::new(0.4, 0.3, 0.35, 1.0),
                    );
                }
            }
        }
        ui.text(
            &n.to_string(),
            r.right() - 6.0,
            r.bottom() - 11.0,
            crate::ui::FONT_PX,
            if n == 0 {
                crate::ui::TEXT_DIM
            } else {
                crate::ui::TEXT
            },
        );
    }
    // 풀밭 위에서도 읽히게 어두운 바탕.
    let hint = crate::i18n::fill(
        crate::i18n::s().quick_hint,
        &[&crate::input::prompt(crate::input::Act::UseItem)],
    );
    let hint = hint.as_str();
    let hx = x0 + 2.0 * (size + 3.0) + 2.0;
    let hw = ui.text_w(hint, crate::ui::FONT_PX) + 6.0;
    ui.rect(
        Rect::new(hx - 3.0, y0 + 3.0, hw, crate::ui::FONT_PX + 2.0),
        Color::new(0.07, 0.05, 0.09, 0.6),
    );
    ui.text(hint, hx, y0 + 4.0, crate::ui::FONT_PX, crate::ui::TEXT);
}

/// 컷씬 배우(월드 패스 안에서, begin_world 뒤 present 전에).
pub fn draw_actor(pos: Vec2, facing: Vec2, color: Color) {
    let p = pos.round();
    draw_ellipse(p.x, p.y + 6.0, 7.0, 3.0, 0.0, SHADOW);
    draw_rectangle(p.x - 6.0, p.y - 6.0, 12.0, 12.0, color);
    let nose = p + facing * 8.0;
    draw_circle(nose.x.round(), nose.y.round(), 2.0, WHITE);
}

/// 타이틀 배경: 격자 위에 뿌리 몇 가닥.
fn draw_title_background() {
    for gx in (0..VW).step_by(16) {
        draw_line(gx as f32, 0.0, gx as f32, t::VIEW_H, 1.0, GRID);
    }
    for gy in (0..VH).step_by(16) {
        draw_line(0.0, gy as f32, t::VIEW_W, gy as f32, 1.0, GRID);
    }
    let root = Color::new(0.30, 0.22, 0.20, 1.0);
    for i in 0..5 {
        let x0 = 40.0 + i as f32 * 60.0;
        let mut prev = vec2(x0, 0.0);
        for s in 1..=12 {
            let y = s as f32 * 15.0;
            let x = x0 + ((y * 0.05 + i as f32).sin() * 14.0).round();
            draw_line(prev.x, prev.y, x, y, 2.0 + (12 - s) as f32 * 0.15, root);
            prev = vec2(x, y);
        }
    }
    draw_rectangle(0.0, t::VIEW_H - 8.0, t::VIEW_W, 8.0, WALL_COLOR);
}

/// 터치 조작 오버레이(실제 해상도). 터치가 한 번이라도 있었을 때만.
pub fn draw_touch_overlay(t: &TouchOverlay) {
    let line = Color::new(1.0, 1.0, 1.0, 0.35);
    let fill = Color::new(1.0, 1.0, 1.0, 0.12);
    if let Some((origin, pos)) = t.stick {
        draw_circle_lines(origin.x, origin.y, t.stick_radius, 2.0, line);
        let d = pos - origin;
        let knob = origin + d.clamp_length_max(t.stick_radius);
        draw_circle(knob.x, knob.y, t.stick_radius * 0.4, fill);
        draw_circle_lines(knob.x, knob.y, t.stick_radius * 0.4, 2.0, line);
    }
    for b in &t.buttons {
        draw_circle(b.center.x, b.center.y, b.radius, fill);
        draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, line);
        let fs = b.radius * 0.6;
        let dim = measure_text(b.label, None, fs as u16, 1.0);
        draw_text(
            b.label,
            b.center.x - dim.width / 2.0,
            b.center.y + dim.height / 2.0,
            fs,
            line,
        );
    }
}

fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    (a + (b - a) * t).round()
}

fn draw_room(w: &World) {
    for gx in (0..VW).step_by(16) {
        draw_line(gx as f32, 0.0, gx as f32, t::VIEW_H, 1.0, GRID);
    }
    for gy in (0..VH).step_by(16) {
        draw_line(0.0, gy as f32, t::VIEW_W, gy as f32, 1.0, GRID);
    }
    let r = w.room.inner;
    draw_rectangle(0.0, 0.0, t::VIEW_W, r.y, WALL_COLOR);
    draw_rectangle(
        0.0,
        r.bottom(),
        t::VIEW_W,
        t::VIEW_H - r.bottom(),
        WALL_COLOR,
    );
    draw_rectangle(0.0, 0.0, r.x, t::VIEW_H, WALL_COLOR);
    draw_rectangle(r.right(), 0.0, t::VIEW_W - r.right(), t::VIEW_H, WALL_COLOR);
    for p in &w.room.solids {
        draw_rectangle(p.x, p.y, p.w, p.h, WALL_COLOR);
        draw_rectangle(p.x + 2.0, p.y + 2.0, p.w - 4.0, p.h - 4.0, world::DIRT);
    }
}

fn draw_player(p: &Player, alpha: f32, time: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let body = if p.flash > 0.0 { WHITE } else { world::MOLE };
    match p.state {
        player::State::Dead => {
            draw_rectangle(pos.x - 8.0, pos.y - 2.0, 16.0, 5.0, GRAY);
            return;
        }
        player::State::Burrow { time: bt } => {
            // 그림자(흙더미)만. 작게 꿈틀.
            let wob = ((bt * 40.0).sin() * 1.0).round();
            draw_ellipse(pos.x, pos.y + 2.0, 8.0 + wob, 4.0, 0.0, SHADOW);
            draw_ellipse(pos.x, pos.y, 6.0, 3.0, 0.0, world::DIRT);
            return;
        }
        _ => {}
    }
    // 무적 중 깜빡임.
    if p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && p.flash <= 0.0 {
        draw_rectangle_lines(pos.x - 6.0, pos.y - 6.0, 12.0, 12.0, 1.0, world::MOLE);
    } else {
        draw_rectangle(pos.x - 6.0, pos.y - 6.0, 12.0, 12.0, body);
    }
    let nose = pos + p.facing * 8.0;
    draw_circle(nose.x.round(), nose.y.round(), 2.0, WHITE);
}

/// 공격 이펙트(발톱 자국 세 줄)와 솟구침 고리. 무브셋 수치를 따른다.
fn draw_player_fx(p: &Player, alpha: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    match p.state {
        player::State::Attack { idx, time: at, dir } => {
            let hit = p.moveset.hits[idx];
            let (a0, a1) = hit.active;
            let origin = pos + dir * 2.0;
            let finisher = idx == p.moveset.last();
            if at >= a0 && at <= a1 + 0.04 {
                let k = ((at - a0) / (a1 - a0 + 0.04)).clamp(0.0, 1.0);
                if finisher {
                    draw_sector(
                        origin,
                        dir,
                        hit.range,
                        Color::new(1.0, 1.0, 1.0, 0.25 * (1.0 - k)),
                    );
                }
                draw_claw_marks(origin, dir, hit.range, k, finisher, p.moveset.mark);
            } else if at < a0 {
                draw_sector_lines(
                    origin,
                    dir,
                    hit.range * 0.5,
                    Color::new(0.6, 0.45, 0.3, 0.6),
                );
            }
        }
        player::State::Drill { time: dt, dir, .. } if dt < t::DRILL_REV => {
            // 시동: 몸이 떨리고, 앞에 작은 드릴이 점점 커지며 돈다. 주변에 기운 고리가 조여 든다.
            let k = dt / t::DRILL_REV;
            let side = vec2(-dir.y, dir.x);
            let base = pos + dir * 3.0;
            let tip = pos + dir * (6.0 + 9.0 * k);
            let claw = Color::new(0.97, 0.93, 0.85, 1.0);
            draw_triangle(base + side * 5.0, base - side * 5.0, tip, OUTLINE);
            draw_triangle(base + side * 4.0, base - side * 4.0, tip - dir, claw);
            let rage = world::RAGE;
            let spin = dt * 40.0;
            for i in 0..6 {
                let a = spin + i as f32 * std::f32::consts::TAU / 6.0;
                let r = 18.0 * (1.0 - k) + 8.0;
                let q = pos + Vec2::from_angle(a) * r;
                let q2 = pos + Vec2::from_angle(a - 0.5) * (r + 4.0);
                draw_line(
                    q.x,
                    q.y,
                    q2.x,
                    q2.y,
                    1.0,
                    Color::new(rage.r, rage.g, rage.b, 0.5 + 0.5 * k),
                );
            }
        }
        player::State::Drill { time: dt, dir, .. } => {
            // 앞쪽 원뿔 드릴: 줄무늬가 돌아간다.
            let dt = dt - t::DRILL_REV;
            let side = vec2(-dir.y, dir.x);
            let base = pos + dir * 3.0;
            let tip = pos + dir * 17.0;
            let claw = Color::new(0.97, 0.93, 0.85, 1.0);
            let shade = Color::new(0.62, 0.55, 0.48, 1.0);
            draw_triangle(base + side * 7.0, base - side * 7.0, tip, OUTLINE);
            draw_triangle(base + side * 6.0, base - side * 6.0, tip - dir, claw);
            for k in 0..3 {
                let f = ((dt * 22.0 + k as f32 / 3.0).fract() * 0.9) + 0.05;
                let a = base + (tip - base) * f;
                let w = 5.0 * (1.0 - f);
                draw_line(
                    a.x + side.x * w,
                    a.y + side.y * w,
                    a.x - side.x * w * 0.3,
                    a.y - side.y * w * 0.3,
                    1.0,
                    shade,
                );
            }
            // 바람 줄: 몸 양옆으로 길게, 깜빡이며.
            for k in 0..4 {
                let off = [8.0, -8.0, 12.0, -12.0][k];
                let len = 10.0 + 8.0 * (((dt * 30.0) as usize + k) % 3) as f32;
                let a = pos + side * off - dir * 2.0;
                let b = a - dir * len;
                draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
            // 화면을 가로지르는 속도선(카메라 둘레에 흩뿌린 고정 무늬가 반대로 흐른다).
            for i in 0..10 {
                let h = (i as f32 * 37.7).sin() * 0.5 + 0.5;
                let lane = (h - 0.5) * 200.0;
                let flow = ((dt * 3.0 + i as f32 * 0.137).fract() - 0.5) * 360.0;
                let a = pos + side * lane - dir * flow;
                let b = a - dir * 24.0;
                draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(1.0, 1.0, 1.0, 0.25));
            }
        }
        player::State::Emerge { time: et } => {
            let p01 = et / t::EMERGE_TIME;
            let r = 6.0 + (t::EMERGE_RADIUS - 6.0) * p01;
            draw_circle_lines(pos.x, pos.y, r, 1.5, Color::new(1.0, 1.0, 1.0, 1.0 - p01));
        }
        _ => {}
    }
}

/// 드릴 돌진 잔상: 지나온 길에 분노색 그림자 셋. 그림이 있으면 Naïl 실루엣, 없으면 원.
fn draw_drill_trail(art: Option<&Art>, p: &Player, alpha: f32) {
    let player::State::Drill { time, dir, .. } = p.state else {
        return;
    };
    if time < t::DRILL_REV {
        return;
    }
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let rage = world::RAGE;
    for i in 1..4 {
        let q = pos - dir * (i as f32 * 9.0);
        let a = 0.45 - i as f32 * 0.12;
        let c = Color::new(rage.r, rage.g, rage.b, a);
        match art {
            Some(art) => draw_texture_ex(
                &art.nail,
                (q.x - 8.0).round(),
                (q.y - 8.0).round(),
                c,
                DrawTextureParams {
                    source: Some(art::cell(2, 4)),
                    dest_size: Some(vec2(16.0, 16.0)),
                    flip_x: p.face_left,
                    ..Default::default()
                },
            ),
            None => draw_circle(q.x, q.y, t::PLAYER_RADIUS, c),
        }
    }
}

/// 필살기 시동: 화면(카메라)을 어둡게 덮는다. 그 위에 플레이어를 다시 그리라고 true.
fn draw_super_dim(w: &World, cam: Vec2) -> bool {
    let player::State::Drill { time, .. } = w.player.state else {
        return false;
    };
    if time >= t::DRILL_REV {
        return false;
    }
    let k = (time / 0.08).min(1.0);
    draw_rectangle(
        cam.x - 8.0,
        cam.y - 8.0,
        t::VIEW_W + 16.0,
        t::VIEW_H + 16.0,
        Color::new(0.04, 0.02, 0.06, 0.55 * k),
    );
    true
}

/// 발톱 자국: 전방 호를 따라 세 줄. 휘두르는 동안 펼쳐지고 끝에서 옅어진다.
fn draw_claw_marks(origin: Vec2, dir: Vec2, r: f32, k: f32, strong: bool, mark: Color) {
    let base = dir.y.atan2(dir.x);
    let span = std::f32::consts::FRAC_PI_2 * 0.95;
    let reveal = (k * 1.6).min(1.0);
    let fade = 1.0 - ((k - 0.6) / 0.4).clamp(0.0, 1.0);
    let col = if strong {
        Color::new(
            (mark.r + 1.0) * 0.5,
            (mark.g + 1.0) * 0.5,
            (mark.b + 1.0) * 0.5,
            fade,
        )
    } else {
        Color::new(mark.r, mark.g, mark.b, 0.9 * fade)
    };
    let thick = if strong { 2.0 } else { 1.0 };
    let steps = 10;
    for line in 0..3 {
        let rr = r - line as f32 * 3.0;
        let a_start = base - span;
        let a_end = base - span + 2.0 * span * reveal;
        let mut prev = origin + vec2(a_start.cos(), a_start.sin()) * rr;
        for i in 1..=steps {
            let a = a_start + (a_end - a_start) * i as f32 / steps as f32;
            let q = origin + vec2(a.cos(), a.sin()) * rr;
            draw_line(prev.x, prev.y, q.x, q.y, thick, col);
            prev = q;
        }
    }
}

fn draw_projectile(p: &enemy::Projectile, alpha: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    // 되돌린 투사체는 하얗게 + 꼬리.
    let c = if p.friendly { WHITE } else { world::PROJECTILE };
    if p.friendly {
        let tail = pos - p.vel.normalize_or_zero() * 5.0;
        draw_line(
            pos.x,
            pos.y,
            tail.x,
            tail.y,
            2.0,
            Color::new(1.0, 1.0, 1.0, 0.5),
        );
    }
    if !p.friendly {
        // 다람쥐의 도토리. 날아가며 돈다.
        draw_acorn(pos, (pos.x + pos.y) * 0.25);
        return;
    }
    draw_circle(pos.x, pos.y, p.radius(), c);
    draw_circle_lines(pos.x, pos.y, p.radius() + 0.5, 1.0, world::PROJECTILE);
}

// ---- 그림 에셋 경로 -----------------------------------------------------------

fn tile_src(col: u32, row: u32) -> Rect {
    art::cell(col, row)
}

fn draw_tile(tex: &Texture2D, src: Rect, x: f32, y: f32, tint: Color) {
    draw_texture_ex(
        tex,
        x,
        y,
        tint,
        DrawTextureParams {
            source: Some(src),
            dest_size: Some(vec2(src.w, src.h)),
            ..Default::default()
        },
    );
}

fn decor_src(d: Decor) -> Rect {
    match d {
        Decor::FlowerWhite => tile_src(4, 0),
        Decor::FlowerYellow => tile_src(5, 0),
        Decor::FlowerPink => tile_src(6, 0),
        Decor::Tuft => tile_src(7, 0),
        Decor::Pebble => tile_src(3, 1),
        Decor::Mushroom => tile_src(4, 1),
        Decor::RootVine => tile_src(5, 1),
        Decor::Moss => tile_src(6, 1),
    }
}

/// 바닥 칸마다 (x, y, 장식). 방 그림은 칸 해시로 정해져 프레임마다 같다. 격자 방은 보이는 바닥 칸만.
fn floor_decor(w: &World, cam: Vec2) -> Vec<(f32, f32, Decor)> {
    let mut out = Vec::new();
    let (cx0, cy0, cx1, cy1) = match &w.room.grid {
        Some(_) => {
            let (x0, y0) = (
                (cam.x / 16.0).floor() as i32 - 1,
                (cam.y / 16.0).floor() as i32 - 1,
            );
            (x0, y0, x0 + 23, y0 + 14)
        }
        None => {
            let r = w.room.inner;
            (
                (r.x / 16.0).floor() as i32,
                (r.y / 16.0).floor() as i32,
                (r.right() / 16.0).ceil() as i32,
                (r.bottom() / 16.0).ceil() as i32,
            )
        }
    };
    for cy in cy0..cy1 {
        for cx in cx0..cx1 {
            if let Some(g) = &w.room.grid {
                if g.get_i(cx, cy) != Cell::Floor {
                    continue;
                }
            }
            let (roll, pick) = biome::hash01(cx, cy, 7);
            if let Some(d) = w.biome.decor(roll, pick) {
                out.push((cx as f32 * 16.0, cy as f32 * 16.0, d));
            }
        }
    }
    out
}

/// 격자 방(필드·던전) 그림: 보이는 칸만. 벽 칸은 아래가 바닥이면 벼랑 앞면, 아니면 윗면(3/4 시점).
/// 특별한 칸: 문(열림 = 바닥, 잠김 = 뿌리 창살), 굴 입구, 출구, 저장 지점(빛나는 버섯 고리).
fn draw_grid_art(art: &Art, w: &World, cam: Vec2) {
    let Some(g) = &w.room.grid else { return };
    let tex = &art.tiles;
    let ft = w.biome.floor_tint();
    let wt = w.biome.wall_tint();
    let dirt = w.biome.dirt_floor();
    let (x0, y0) = (
        (cam.x / 16.0).floor() as i32 - 1,
        (cam.y / 16.0).floor() as i32 - 1,
    );
    let (x1, y1) = (x0 + 23, y0 + 14);
    let solid = |c: Cell| c == Cell::Wall || (c == Cell::Door && w.room.doors_locked());
    for cy in y0..y1 {
        for cx in x0..x1 {
            let c = g.get_i(cx, cy);
            let (px, py) = (cx as f32 * 16.0, cy as f32 * 16.0);
            if c == Cell::Wall {
                if g.get_i(cx, cy + 1) != Cell::Wall {
                    let (roll, _) = biome::hash01(cx, cy, 3);
                    let roots = match w.biome {
                        crate::biome::Biome::Meadow => roll < 0.15,
                        crate::biome::Biome::Roots => roll < 0.55,
                        crate::biome::Biome::Deep => roll < 0.8,
                    };
                    let src = if roots {
                        tile_src(2, 1)
                    } else {
                        tile_src(1, 1)
                    };
                    draw_tile(tex, src, px, py, wt);
                } else {
                    draw_tile(tex, tile_src(0, 1), px, py, wt);
                }
                continue;
            }
            let (_, pick) = biome::hash01(cx, cy, 1);
            let src = if dirt {
                tile_src(3, 0)
            } else {
                tile_src(pick % 3, 0)
            };
            draw_tile(tex, src, px, py, ft);
            // 벼랑 아래 그늘
            if solid(g.get_i(cx, cy - 1)) {
                draw_rectangle(px, py, 16.0, 2.0, Color::new(0.0, 0.0, 0.0, 0.25));
                draw_rectangle(px, py + 2.0, 16.0, 2.0, Color::new(0.0, 0.0, 0.0, 0.12));
            }
        }
    }
    // 반대 재질 얼룩(바닥만)
    for cy in y0..y1 {
        for cx in x0..x1 {
            if g.get_i(cx, cy) != Cell::Floor {
                continue;
            }
            let (roll, pick) = biome::hash01(cx, cy, 2);
            if roll < w.biome.patch_chance() {
                let row = if dirt { 3 } else { 2 };
                let src = tile_src(2 + pick % 2, row);
                let ox = ((pick >> 2) % 9) as f32 - 4.0;
                let oy = ((pick >> 6) % 9) as f32 - 4.0;
                draw_tile(tex, src, cx as f32 * 16.0 + ox, cy as f32 * 16.0 + oy, ft);
            }
        }
    }
    for (x, y, d) in floor_decor(w, cam) {
        draw_tile(tex, decor_src(d), x, y, ft);
    }
    // 특별한 칸
    for cy in y0..y1 {
        for cx in x0..x1 {
            let c = g.get_i(cx, cy);
            let p = vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0);
            match c {
                Cell::Door if w.room.doors_locked() => {
                    let wood = Color::new(0.42, 0.28, 0.20, 1.0);
                    for k in 0..3 {
                        let x = p.x - 6.0 + k as f32 * 5.0;
                        draw_rectangle(x - 1.0, p.y - 8.0, 4.0, 16.0, OUTLINE);
                        draw_rectangle(x, p.y - 8.0, 2.0, 16.0, wood);
                    }
                }
                Cell::Entrance => {
                    draw_ellipse(p.x, p.y + 2.0, 12.0, 7.0, 0.0, world::DIRT);
                    draw_ellipse(
                        p.x,
                        p.y + 2.0,
                        9.0,
                        5.0,
                        0.0,
                        Color::new(0.05, 0.03, 0.06, 1.0),
                    );
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        6.0,
                        3.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 1.0),
                    );
                }
                Cell::Exit => {
                    let pulse = 0.5 + 0.3 * (w.time * 3.0).sin();
                    draw_ellipse(p.x, p.y + 2.0, 10.0, 6.0, 0.0, world::DIRT);
                    draw_ellipse(
                        p.x,
                        p.y + 2.0,
                        8.0,
                        4.5,
                        0.0,
                        Color::new(0.95, 0.9, 0.6, pulse),
                    );
                    draw_triangle(
                        vec2(p.x, p.y - 10.0),
                        vec2(p.x - 4.0, p.y - 5.0),
                        vec2(p.x + 4.0, p.y - 5.0),
                        Color::new(1.0, 1.0, 0.8, pulse),
                    );
                }
                Cell::Save => {
                    let pulse = 0.55 + 0.35 * (w.time * 2.0).sin().abs();
                    draw_circle(p.x, p.y, 9.0, Color::new(0.55, 1.0, 0.6, 0.12 * pulse));
                    for k in 0..5 {
                        let a = k as f32 / 5.0 * std::f32::consts::TAU + 0.3;
                        let q = p + vec2(a.cos() * 7.0, a.sin() * 4.0);
                        draw_rectangle(
                            q.x - 1.0,
                            q.y - 2.0,
                            3.0,
                            2.0,
                            Color::new(0.6, 1.0, 0.65, pulse),
                        );
                        draw_rectangle(q.x, q.y, 1.0, 2.0, Color::new(0.9, 0.85, 0.7, 1.0));
                    }
                }
                _ => {}
            }
        }
    }
}

/// 보물 상자(던전 보물방). 열리면 뚜껑이 젖혀지고 안이 빈다.
fn draw_chest(r: Rect, opened: bool, time: f32) {
    let wood = Color::new(0.55, 0.36, 0.22, 1.0);
    let dark = Color::new(0.36, 0.22, 0.14, 1.0);
    let gold = Color::new(0.95, 0.80, 0.35, 1.0);
    let (x, y) = (r.x + 1.0, r.bottom() - 11.0);
    draw_rectangle(x - 1.0, y - 1.0, 16.0, 12.0, OUTLINE);
    draw_rectangle(x, y + 3.0, 14.0, 7.0, wood);
    draw_rectangle(x, y + 8.0, 14.0, 2.0, dark);
    if opened {
        draw_rectangle(x, y - 3.0, 14.0, 3.0, dark);
        draw_rectangle(
            x + 1.0,
            y + 3.0,
            12.0,
            2.0,
            Color::new(0.05, 0.03, 0.05, 1.0),
        );
    } else {
        draw_rectangle(x, y, 14.0, 3.0, wood);
        draw_rectangle(x + 6.0, y + 2.0, 2.0, 3.0, gold);
        if ((time * 2.0) as i32) % 2 == 0 {
            draw_rectangle(x + 11.0, y - 3.0, 1.0, 3.0, WHITE);
            draw_rectangle(x + 10.0, y - 2.0, 3.0, 1.0, WHITE);
        }
    }
}

/// 3/4 방: 바닥(풀·흙) + 장식, 좌·우·아래 벽 윗면, 위쪽은 벽 윗면 + 벼랑 앞면.
fn draw_room_art(art: &Art, w: &World) {
    let tex = &art.tiles;
    let r = w.room.inner;
    let ft = w.biome.floor_tint();
    let wt = w.biome.wall_tint();
    // 바닥
    let cols = (t::VIEW_W / 16.0) as i32;
    let rows = (t::VIEW_H / 16.0).ceil() as i32;
    let dirt = w.biome.dirt_floor();
    for cy in 0..rows {
        for cx in 0..cols {
            let (_, pick) = biome::hash01(cx, cy, 1);
            let src = if dirt {
                tile_src(3, 0)
            } else {
                tile_src(pick % 3, 0)
            };
            draw_tile(tex, src, cx as f32 * 16.0, cy as f32 * 16.0, ft);
        }
    }
    // 반대 재질 얼룩. 칸 가운데를 반 칸 어긋나게 놓아 격자가 덜 보인다.
    for cy in 0..rows {
        for cx in 0..cols {
            let (roll, pick) = biome::hash01(cx, cy, 2);
            if roll < w.biome.patch_chance() {
                let row = if dirt { 3 } else { 2 };
                let src = tile_src(2 + pick % 2, row);
                let ox = ((pick >> 2) % 9) as f32 - 4.0;
                let oy = ((pick >> 6) % 9) as f32 - 4.0;
                draw_tile(tex, src, cx as f32 * 16.0 + ox, cy as f32 * 16.0 + oy, ft);
            }
        }
    }
    for (x, y, d) in floor_decor(w, Vec2::ZERO) {
        // 빛나는 버섯은 어둠 뒤에 한 번 더 그린다(draw_darkness).
        draw_tile(tex, decor_src(d), x, y, ft);
    }
    // 좌·우 벽(윗면 8px 폭)
    for cy in 0..rows {
        let y = cy as f32 * 16.0;
        draw_tile(tex, Rect::new(0.0, 16.0, 8.0, 16.0), 0.0, y, wt);
        draw_tile(tex, Rect::new(8.0, 16.0, 8.0, 16.0), r.right(), y, wt);
    }
    // 아래 벽(윗면 위쪽 8px)
    for cx in 0..cols {
        draw_tile(
            tex,
            Rect::new(0.0, 16.0, 16.0, 8.0),
            cx as f32 * 16.0,
            r.bottom(),
            wt,
        );
    }
    // 위쪽: 윗면(아래 8px, 풀이 늘어진 가장자리) + 벼랑 앞면 16px
    let face_y = r.y - 16.0;
    for cx in 0..cols {
        let x = cx as f32 * 16.0;
        draw_tile(tex, Rect::new(0.0, 24.0, 16.0, 8.0), x, face_y - 8.0, wt);
        let (roll, _) = biome::hash01(cx, 0, 3);
        let roots = match w.biome {
            crate::biome::Biome::Meadow => roll < 0.15,
            crate::biome::Biome::Roots => roll < 0.55,
            crate::biome::Biome::Deep => roll < 0.8,
        };
        let src = if roots {
            tile_src(2, 1)
        } else {
            tile_src(1, 1)
        };
        draw_tile(tex, src, x, face_y, wt);
    }
    // 벼랑 아래 그늘
    draw_rectangle(r.x, r.y, r.w, 2.0, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_rectangle(r.x, r.y + 2.0, r.w, 2.0, Color::new(0.0, 0.0, 0.0, 0.12));
}

#[derive(Clone, Copy)]
enum Drawable {
    Pickup(usize),
    Loot(usize),
    Pillar(usize),
    Player,
    Enemy(usize),
    Projectile(usize),
    Npc,
}

/// 3/4 시점: 발 위치(y)가 아래인 것이 앞. 그림자는 모두 먼저.
fn draw_sorted(art: &Art, w: &World, alpha: f32) {
    let mut list: Vec<(f32, Drawable)> = Vec::new();
    for (i, p) in w.room.props.iter().enumerate() {
        list.push((p.rect.bottom(), Drawable::Pillar(i)));
    }
    for (i, p) in w.pickups.iter().enumerate() {
        list.push((p.pos.y + 4.0, Drawable::Pickup(i)));
        draw_ellipse(p.pos.x, p.pos.y + 4.0, 5.0, 1.8, 0.0, SHADOW_SOFT);
    }
    for (i, l) in w.loot.iter().enumerate() {
        let q = lerp(l.prev_pos, l.pos, alpha);
        list.push((q.y + 2.0, Drawable::Loot(i)));
        let r = if l.kind == loot::LootKind::Berry {
            3.0
        } else {
            2.0
        };
        draw_ellipse(q.x, q.y + 2.0, r, 1.0, 0.0, SHADOW_SOFT);
    }
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    list.push((pp.y + 6.0, Drawable::Player));
    if let Some(n) = &w.npc {
        let q = lerp(n.prev_pos, n.pos, alpha);
        list.push((q.y + 6.0, Drawable::Npc));
        draw_ellipse(q.x, q.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
    }
    for (i, e) in w.enemies.iter().enumerate() {
        let ep = lerp(e.prev_pos, e.pos, alpha);
        list.push((ep.y + e.radius(), Drawable::Enemy(i)));
        draw_ellipse(
            ep.x,
            ep.y + e.radius(),
            e.radius() + 1.0,
            2.5,
            0.0,
            SHADOW_SOFT,
        );
    }
    for (i, p) in w.projectiles.iter().enumerate() {
        let q = lerp(p.prev_pos, p.pos, alpha);
        list.push((q.y + 3.0, Drawable::Projectile(i)));
        draw_ellipse(q.x, q.y + 6.0, 2.5, 1.2, 0.0, SHADOW_SOFT);
    }
    if !matches!(
        w.player.state,
        player::State::Burrow { .. } | player::State::Dead
    ) {
        draw_ellipse(pp.x, pp.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
    }
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, d) in list {
        match d {
            Drawable::Pillar(i) => {
                let p = w.room.props[i];
                let r = p.rect;
                match p.kind {
                    PropKind::Stump | PropKind::Rock => {
                        let col = if p.kind == PropKind::Stump { 0.0 } else { 16.0 };
                        let src = Rect::new(col, 32.0, 16.0, 32.0);
                        draw_tile(&art.tiles, src, r.x, r.bottom() - 32.0, w.biome.wall_tint());
                    }
                    PropKind::Chest { opened } => draw_chest(r, opened, w.time),
                }
            }
            Drawable::Pickup(i) => draw_pickup(&w.pickups[i], w.time),
            Drawable::Loot(i) => draw_loot(&w.loot[i], alpha, w.time, false),
            Drawable::Player => draw_nail(art, &w.player, alpha, w.time),
            Drawable::Enemy(i) => draw_enemy(Some(art), &w.enemies[i], alpha, w.time),
            Drawable::Projectile(i) => draw_projectile(&w.projectiles[i], alpha),
            Drawable::Npc => {
                if let (Some(n), Some(tex)) = (&w.npc, art.critters.as_ref()) {
                    let q = lerp(n.prev_pos, n.pos, alpha);
                    let moving = (n.pos - n.prev_pos).length() > 0.05;
                    draw_critter_at(tex, art::RACCOON_ROW, q, n.face_left, moving, w.time);
                }
            }
        }
    }
}

/// 반딧불(빛나는 꽁무니 + 날개 깜빡임, 둥실) / 산딸기(빨간 열매 + 잎).
/// `light_only`: 어둠 위에 빛만 다시 그린다(심층에서도 반딧불이 보이게).
fn draw_loot(l: &loot::Loot, alpha: f32, time: f32, light_only: bool) {
    let q = lerp(l.prev_pos, l.pos, alpha);
    let ph = time * 6.0 + l.phase * 10.0;
    match l.kind {
        loot::LootKind::Glowbug => {
            // 바닥에 내려앉지 않고 늘 살짝 떠 있다.
            let hover = 3.0 + (ph * 0.5).sin() * 1.0;
            let p = vec2(q.x.round(), (q.y - l.z - hover).round());
            let pulse = 0.55 + 0.45 * (ph * 0.8).sin().abs();
            let halo = Color::new(world::GLOW.r, world::GLOW.g, world::GLOW.b, 0.18 * pulse);
            draw_circle(p.x + 0.5, p.y + 1.5, 4.0, halo);
            draw_rectangle(
                p.x,
                p.y + 1.0,
                2.0,
                2.0,
                Color::new(1.0, 1.0, 0.7, 0.6 + 0.4 * pulse),
            );
            if light_only {
                return;
            }
            // 몸(짙은 머리) + 날개 깜빡임
            draw_rectangle(p.x, p.y - 1.0, 2.0, 2.0, OUTLINE);
            if (ph as i32) % 2 == 0 {
                draw_rectangle(
                    p.x - 1.0,
                    p.y - 2.0,
                    1.0,
                    1.0,
                    Color::new(0.9, 0.95, 1.0, 0.8),
                );
                draw_rectangle(
                    p.x + 2.0,
                    p.y - 2.0,
                    1.0,
                    1.0,
                    Color::new(0.9, 0.95, 1.0, 0.8),
                );
            }
        }
        loot::LootKind::Worm => {
            if light_only {
                return;
            }
            let p = vec2(q.x.round(), (q.y - l.z).round() - 2.0);
            draw_worm(p, ph, 1.0);
        }
        loot::LootKind::RootHeart => {
            // 뿌리 심장: 뿌리가 감싼 붉은 심장, 둥실 + 은은한 빛.
            let hover = 2.0 + (ph * 0.4).sin();
            let p = vec2(q.x.round(), (q.y - l.z - hover).round() - 3.0);
            let pulse = 0.6 + 0.4 * (ph * 0.5).sin().abs();
            draw_circle(
                p.x + 0.5,
                p.y + 1.0,
                7.0,
                Color::new(1.0, 0.4, 0.45, 0.15 * pulse),
            );
            if light_only {
                return;
            }
            let red = Color::new(0.90, 0.25, 0.32, 1.0);
            let dark = Color::new(0.60, 0.12, 0.22, 1.0);
            let root = Color::new(0.45, 0.32, 0.22, 1.0);
            // 하트: 위 두 혹 + 아래 뾰족
            draw_rectangle(p.x - 4.0, p.y - 3.0, 9.0, 4.0, OUTLINE);
            draw_rectangle(p.x - 3.0, p.y + 1.0, 7.0, 2.0, OUTLINE);
            draw_rectangle(p.x - 2.0, p.y + 3.0, 5.0, 1.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y + 4.0, 3.0, 1.0, OUTLINE);
            draw_rectangle(p.x - 3.0, p.y - 2.0, 3.0, 3.0, red);
            draw_rectangle(p.x + 1.0, p.y - 2.0, 3.0, 3.0, red);
            draw_rectangle(p.x - 2.0, p.y + 1.0, 5.0, 2.0, red);
            draw_rectangle(p.x - 1.0, p.y + 3.0, 3.0, 1.0, dark);
            draw_rectangle(p.x + 2.0, p.y, 2.0, 1.0, dark);
            draw_rectangle(
                p.x - 2.0,
                p.y - 2.0,
                1.0,
                1.0,
                Color::new(1.0, 0.8, 0.8, 1.0),
            );
            // 감싼 뿌리
            draw_rectangle(p.x - 3.0, p.y + 1.0, 1.0, 1.0, root);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 1.0, 3.0, root);
            draw_rectangle(p.x + 1.0, p.y + 2.0, 2.0, 1.0, root);
        }
        loot::LootKind::Seed => {
            // 씨앗: 통통한 갈색 알 + 초록 새싹 한 잎. 살짝 둥실.
            let hover = 1.0 + (ph * 0.7).sin() * 0.5;
            let p = vec2(q.x.round(), (q.y - l.z - hover).round() - 2.0);
            if light_only {
                return;
            }
            draw_rectangle(p.x - 2.0, p.y - 1.0, 4.0, 4.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 2.0, 2.0, 6.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 2.0, 4.0, world::SEED);
            draw_rectangle(p.x, p.y + 1.0, 1.0, 2.0, Color::new(0.62, 0.44, 0.24, 1.0));
            draw_rectangle(
                p.x - 1.0,
                p.y - 1.0,
                1.0,
                1.0,
                Color::new(1.0, 0.92, 0.72, 1.0),
            );
            draw_rectangle(p.x, p.y - 3.0, 2.0, 1.0, Color::new(0.45, 0.78, 0.35, 1.0));
        }
        loot::LootKind::Berry => {
            if light_only {
                return;
            }
            let p = vec2(q.x.round(), (q.y - l.z).round() - 3.0);
            let dark = Color::new(0.62, 0.14, 0.24, 1.0);
            draw_rectangle(p.x - 2.0, p.y - 1.0, 5.0, 4.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 2.0, 3.0, 6.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 3.0, 4.0, world::BERRY);
            draw_rectangle(p.x - 1.0, p.y + 2.0, 3.0, 1.0, dark);
            draw_rectangle(
                p.x - 1.0,
                p.y - 1.0,
                1.0,
                1.0,
                Color::new(1.0, 0.85, 0.85, 1.0),
            );
            // 잎
            draw_rectangle(p.x, p.y - 3.0, 2.0, 1.0, Color::new(0.45, 0.75, 0.35, 1.0));
        }
    }
}

/// 쥬시 지렁이: 꿈틀대는 분홍 마디 + 반들거리는 하이라이트. `k`는 크기 배율(1 = 월드, HUD는 더 크게).
pub fn draw_worm(p: Vec2, ph: f32, k: f32) {
    let body = Color::new(0.95, 0.55, 0.62, 1.0);
    let dark = Color::new(0.70, 0.32, 0.42, 1.0);
    for i in 0..5 {
        let x = p.x + (i as f32 - 2.0) * 1.6 * k;
        let y = p.y + ((ph + i as f32 * 1.2).sin() * 1.0 * k).round();
        draw_rectangle(x - 1.0 * k, y - 1.0 * k, 2.4 * k, 3.0 * k, OUTLINE);
    }
    for i in 0..5 {
        let x = p.x + (i as f32 - 2.0) * 1.6 * k;
        let y = p.y + ((ph + i as f32 * 1.2).sin() * 1.0 * k).round();
        draw_rectangle(
            x - 0.5 * k,
            y - 0.5 * k,
            1.6 * k,
            2.0 * k,
            if i == 4 { dark } else { body },
        );
    }
    // 반들반들(쥬시)
    let y = p.y + (ph.sin() * 1.0 * k).round();
    draw_rectangle(
        p.x - 3.0 * k,
        y - 0.5 * k,
        0.8 * k,
        0.8 * k,
        Color::new(1.0, 0.95, 0.95, 1.0),
    );
}

/// 바닥 무기: 떠오르는 발톱 세 개 + 반짝임.
fn draw_pickup(p: &world::Pickup, time: f32) {
    let c = crate::equipment::WEAPONS[p.weapon].color;
    let dark = Color::new(c.r * 0.55, c.g * 0.55, c.b * 0.55, 1.0);
    let bob = ((time * 3.0).sin() * 1.5).round();
    let base = vec2(p.pos.x.round(), p.pos.y.round() - 6.0 + bob);
    for k in 0..3 {
        let x = base.x - 4.0 + k as f32 * 3.0;
        let y = base.y + (k % 2) as f32;
        draw_rectangle(x - 0.5, y - 1.0, 3.0, 8.0, OUTLINE);
        draw_rectangle(x, y, 2.0, 6.0, c);
        draw_rectangle(x, y + 5.0, 2.0, 1.0, dark);
    }
    let ph = (time * 2.0).fract();
    let s = base + vec2(5.0, -3.0) + vec2(0.0, -ph * 4.0);
    let a = 1.0 - ph;
    draw_rectangle(s.x, s.y - 1.0, 1.0, 3.0, Color::new(1.0, 1.0, 1.0, a));
    draw_rectangle(s.x - 1.0, s.y, 3.0, 1.0, Color::new(1.0, 1.0, 1.0, a));
}

const OUTLINE: Color = Color::new(0.17, 0.11, 0.16, 1.0);

/// Naïl 스프라이트. 상태 → 칸, 좌우 반전, 피격 플래시는 흰 실루엣 줄.
fn draw_nail(art: &Art, p: &Player, alpha: f32, time: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    let (col, row) = match p.state {
        player::State::Dead => (4, 1),
        player::State::Burrow { time: bt } => (((bt * 12.0) as u32) % 2, 2),
        player::State::Emerge { .. } | player::State::Drill { .. } => (2, 1),
        player::State::Recover { .. } => (3, 1),
        player::State::Attack { idx, .. } => {
            let last = p.moveset.last();
            (if idx == last { 2 } else { idx.min(1) as u32 }, 1)
        }
        _ if p.invuln > t::PLAYER_HURT_INVULN - 0.25 => (3, 1),
        _ if moving => (2 + ((time * 10.0) as u32) % 4, 0),
        _ => (((time * 2.0) as u32) % 2, 0),
    };
    let flash = p.flash > 0.0 && !matches!(p.state, player::State::Burrow { .. });
    let row = if flash { row + 3 } else { row };
    let blink = p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && !flash;
    let tint = if blink {
        Color::new(1.0, 1.0, 1.0, 0.35)
    } else {
        WHITE
    };
    draw_texture_ex(
        &art.nail,
        pos.x - 8.0,
        pos.y - 8.0,
        tint,
        DrawTextureParams {
            source: Some(art::cell(col, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: p.face_left,
            ..Default::default()
        },
    );
}

/// 환경 어둠: 플레이어 주변만 밝고 바깥은 어둡다. 빛나는 버섯은 어둠 위에 다시.
fn draw_darkness(art: &Art, w: &World, alpha: f32, cam: Vec2) {
    let (dark, radius) = w.biome.darkness();
    if dark <= 0.0 {
        return;
    }
    let c = lerp(w.player.prev_pos, w.player.pos, alpha);
    let s = radius * 2.0;
    let (x0, y0) = (c.x - radius, c.y - radius);
    let col = Color::new(1.0, 1.0, 1.0, dark);
    draw_texture_ex(
        &art.dark,
        x0,
        y0,
        col,
        DrawTextureParams {
            dest_size: Some(vec2(s, s)),
            ..Default::default()
        },
    );
    let black = Color::new(0.0, 0.0, 0.0, dark);
    // 보이는 화면(카메라 기준, 흔들림 여유 8px)에서 빛 원 바깥을 덮는다.
    let (l, tp) = (cam.x - 8.0, cam.y - 8.0);
    let (r, b) = (cam.x + t::VIEW_W + 8.0, cam.y + t::VIEW_H + 8.0);
    draw_rectangle(l, tp, r - l, (y0 - tp).max(0.0), black);
    draw_rectangle(l, y0 + s, r - l, (b - (y0 + s)).max(0.0), black);
    draw_rectangle(l, y0, (x0 - l).max(0.0), s, black);
    draw_rectangle(x0 + s, y0, (r - (x0 + s)).max(0.0), s, black);
    for l in &w.loot {
        draw_loot(l, alpha, w.time, true);
    }
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.65 + 0.25 * ((w.time * 2.0 + x * 0.1).sin());
            draw_tile(
                &art.tiles,
                decor_src(d),
                x,
                y,
                Color::new(1.0, 1.0, 1.0, pulse),
            );
        }
    }
}

/// 적: 그림(critters.png)이 있으면 종류의 줄에서 상태에 맞는 칸, 없으면 색 원.
/// 예고(떨림·웅크림·조준)는 칸 4 + 떨림/밝아짐, 공격은 칸 5, 맞음·경직은 칸 6. 번쩍일 때는 흰 실루엣 줄.
fn draw_enemy(art: Option<&Art>, e: &Enemy, alpha: f32, time: f32) {
    let sp = e.sp();
    let mut pos = lerp(e.prev_pos, e.pos, alpha);
    let r = e.radius();
    let base = sp.color;
    let mut tint = WHITE;
    if e.poison > 0.0 && e.flash <= 0.0 {
        // 독: 초록빛이 돌고 거품이 오른다.
        tint = Color::new(0.7, 1.0, 0.6, 1.0);
        for k in 0..2 {
            let ph = (time * 1.6 + k as f32 * 0.5 + e.pos.x * 0.01).fract();
            let bx = pos.x + (k as f32 * 2.0 - 1.0) * (r * 0.5) + (ph * 9.0).sin();
            let by = pos.y - r - 4.0 - ph * 8.0;
            draw_circle(
                bx.round(),
                by.round(),
                1.0,
                Color::new(0.55, 0.9, 0.35, 1.0 - ph),
            );
        }
    }
    let moving = (e.pos - e.prev_pos).length() > 0.05;
    let walk = ((time * 8.0 + e.pos.x * 0.05) as u32) % 2;
    let mut frame = if moving {
        2 + walk
    } else {
        walk.min(1) * ((time * 2.0) as u32 % 2)
    };
    match e.state {
        enemy::State::Windup { time: wt } | enemy::State::Aim { time: wt } => {
            frame = 4;
            // 예고: 떨림 + 점점 붉어짐
            pos.x += if ((time * 60.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let k = (wt / sp.windup).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.45 * k, 1.0 - 0.45 * k, 1.0);
            if matches!(e.state, enemy::State::Aim { .. }) {
                // 도토리를 쳐든다.
                let hand = pos + e.facing * (r + 1.0) - vec2(0.0, 4.0);
                draw_acorn(hand, 0.0);
            }
        }
        enemy::State::Crouch { time: ct, target } => {
            frame = 4;
            let k = (ct / sp.windup).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.4 * k, 1.0 - 0.4 * k, 1.0);
            draw_land_mark(target, sp.land_radius, k);
        }
        enemy::State::Leap { time: lt, to, .. } => {
            frame = 5;
            draw_land_mark(to, sp.land_radius, 1.0);
            let _ = lt;
            pos.y -= e.height();
        }
        enemy::State::Rest { .. } if sp.behavior != crate::species::Behavior::Dummy => {
            tint = Color::new(0.75, 0.75, 0.75, 1.0);
        }
        enemy::State::Stunned { time: st } => {
            frame = 6;
            // 머리 위에 노란 점이 돈다.
            let a = st * 12.0;
            let s = pos + vec2(a.cos(), a.sin() * 0.5) * (r + 3.0) - vec2(0.0, r + 4.0);
            draw_circle(s.x.round(), s.y.round(), 1.0, YELLOW);
        }
        enemy::State::Charge { dir, .. } => {
            frame = 5;
            // 잔상
            for i in 1..4 {
                let back = pos - dir * (i as f32 * 4.0);
                let a = 0.4 - i as f32 * 0.1;
                draw_circle(
                    back.x,
                    back.y,
                    r - i as f32,
                    Color::new(base.r, base.g, base.b, a),
                );
            }
        }
        _ => {}
    }
    if e.asleep {
        frame = 1;
        // 자는 중: Z가 떠오른다.
        let ph = (time * 0.8 + e.pos.x * 0.013).fract();
        let z = pos + vec2(4.0 + ph * 3.0, -r - 6.0 - ph * 6.0);
        let c = Color::new(0.9, 0.9, 1.0, 1.0 - ph);
        draw_line(z.x, z.y, z.x + 3.0, z.y, 1.0, c);
        draw_line(z.x + 3.0, z.y, z.x, z.y + 3.0, 1.0, c);
        draw_line(z.x, z.y + 3.0, z.x + 3.0, z.y + 3.0, 1.0, c);
    }
    let flash = e.flash > 0.0;
    match art.and_then(|a| a.critters.as_ref()) {
        Some(tex) => {
            let row = sp.sheet_row + if flash { art::CRITTER_ROWS } else { 0 };
            draw_texture_ex(
                tex,
                (pos.x - 8.0).round(),
                (pos.y + r - 15.0).round(),
                if flash { WHITE } else { tint },
                DrawTextureParams {
                    source: Some(art::cell(frame, row)),
                    dest_size: Some(vec2(16.0, 16.0)),
                    flip_x: e.facing.x < 0.0,
                    ..Default::default()
                },
            );
        }
        None => {
            let c = if flash {
                WHITE
            } else {
                Color::new(base.r * tint.r, base.g * tint.g, base.b * tint.b, 1.0)
            };
            draw_circle(pos.x, pos.y, r, c);
            let nose = pos + e.facing * (r + 1.0);
            draw_circle(nose.x.round(), nose.y.round(), 1.5, BLACK);
        }
    }
    // 체력 점(맞은 적만: 처음부터 숫자를 늘어놓지 않는다)
    if e.hp < sp.hp || sp.behavior == crate::species::Behavior::Dummy {
        let max = sp.hp;
        for i in 0..max {
            let c = if i < e.hp { WHITE } else { DARKGRAY };
            draw_rectangle(
                (pos.x - max as f32 * 1.5 + i as f32 * 3.0).round(),
                (pos.y - r - 9.0).round(),
                2.0,
                2.0,
                c,
            );
        }
    }
}

/// 이웃 그림 한 칸(발이 `pos.y + 6` 근처). 걸으면 칸 2·3, 서 있으면 0·1.
fn draw_critter_at(tex: &Texture2D, row: u32, pos: Vec2, face_left: bool, moving: bool, time: f32) {
    let frame = if moving {
        2 + ((time * 8.0) as u32) % 2
    } else {
        ((time * 2.0) as u32) % 2
    };
    draw_texture_ex(
        tex,
        (pos.x - 8.0).round(),
        (pos.y - 9.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(art::cell(frame, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: face_left,
            ..Default::default()
        },
    );
}

/// 청설모가 내려앉을 자리: 바닥에 점점 진해지는 붉은 고리.
fn draw_land_mark(at: Vec2, r: f32, k: f32) {
    let a = 0.25 + 0.55 * k;
    draw_circle(at.x, at.y, r, Color::new(1.0, 0.3, 0.35, 0.12 * k));
    draw_circle_lines(
        at.x,
        at.y,
        r * (1.3 - 0.3 * k),
        1.0,
        Color::new(1.0, 0.35, 0.4, a),
    );
}

/// 도토리(다람쥐가 던지는 것): 갈색 열매 + 짙은 모자.
fn draw_acorn(p: Vec2, spin: f32) {
    let p = p.round();
    let up = Vec2::from_angle(spin - std::f32::consts::FRAC_PI_2);
    draw_circle(p.x, p.y, 2.5, OUTLINE);
    draw_circle(p.x, p.y + 0.5, 2.0, Color::new(0.78, 0.52, 0.28, 1.0));
    let cap = p + up * 1.5;
    draw_rectangle(
        cap.x - 2.0,
        cap.y - 1.0,
        4.0,
        2.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
    let stem = p + up * 3.0;
    draw_rectangle(
        stem.x,
        stem.y - 1.0,
        1.0,
        1.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
}

fn draw_fx(w: &World) {
    for f in &w.fx {
        let p = (f.time / f.dur).clamp(0.0, 1.0);
        let pos = f.pos.round();
        match f.kind {
            FxKind::Ring { r0, r1, color } => {
                let r = r0 + (r1 - r0) * p;
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    r,
                    1.0,
                    Color::new(color.r, color.g, color.b, 1.0 - p),
                );
            }
            FxKind::Spark { color } => {
                let s = if p < 0.5 { 2.0 } else { 1.0 };
                draw_rectangle(
                    pos.x,
                    pos.y,
                    s,
                    s,
                    Color::new(color.r, color.g, color.b, 1.0 - p),
                );
            }
        }
    }
}

fn draw_hitboxes(w: &World, alpha: f32) {
    let c = Color::new(0.2, 1.0, 0.2, 0.8);
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    draw_circle_lines(pp.x, pp.y, w.player.radius(), 1.0, c);
    for e in &w.enemies {
        let p = lerp(e.prev_pos, e.pos, alpha);
        let col = if e.contact_damage().is_some() { RED } else { c };
        draw_circle_lines(p.x, p.y, e.radius(), 1.0, col);
    }
    for p in &w.projectiles {
        let q = lerp(p.prev_pos, p.pos, alpha);
        draw_circle_lines(q.x, q.y, p.radius(), 1.0, RED);
    }
    if let Some(s) = w.last_strike {
        let col = match s.kind {
            StrikeKind::Claw(_) => YELLOW,
            StrikeKind::Emerge => ORANGE,
            StrikeKind::Drill => SKYBLUE,
            StrikeKind::DrillWall => RED,
        };
        match s.shape {
            Shape::Circle { center, r } => draw_circle_lines(center.x, center.y, r, 1.0, col),
            Shape::Sector { origin, dir, r } => draw_sector_lines(origin, dir, r, col),
        }
    }
}

fn draw_hud_world(w: &World) {
    // 체력 핍(왼쪽 위, 벽 위에)
    for i in 0..w.player.stats.max_hp {
        let c = if i < w.player.hp {
            world::MOLE
        } else {
            Color::new(0.2, 0.15, 0.2, 1.0)
        };
        draw_rectangle(3.0 + i as f32 * 7.0, 1.0, 6.0, 6.0, c);
    }
    // 굴파기 쿨다운 바(오른쪽 위)
    let cd = w.player.burrow_cd / (t::BURROW_COOLDOWN * w.player.stats.burrow_cd);
    let bw = 24.0;
    draw_rectangle(
        t::VIEW_W - 3.0 - bw,
        2.0,
        bw,
        4.0,
        Color::new(0.2, 0.15, 0.2, 1.0),
    );
    draw_rectangle(
        t::VIEW_W - 3.0 - bw,
        2.0,
        bw * (1.0 - cd).max(0.0),
        4.0,
        world::DIRT,
    );
    // SP 게이지(그 아래). 가득이면 깜빡인다 — 필살기를 쓸 수 있다.
    let sp = w.player.sp / t::SP_MAX;
    let full = w.player.sp_full();
    draw_rectangle(
        t::VIEW_W - 3.0 - bw,
        8.0,
        bw,
        3.0,
        Color::new(0.2, 0.15, 0.2, 1.0),
    );
    // 색은 캐릭터의 게이지 종류(두더지 = 분노, 주황).
    let gc = w.character.gauge().color();
    let gold = if full && ((w.time * 6.0) as i32) % 2 == 0 {
        Color::new(1.0, 1.0, 0.85, 1.0)
    } else {
        gc
    };
    draw_rectangle(t::VIEW_W - 3.0 - bw, 8.0, bw * sp, 3.0, gold);
}

fn draw_hud_screen(w: &World, scale: f32, ox: f32, oy: f32, gamepad_count: usize) {
    let fs = 16.0 * (scale / 4.0).max(1.0);
    let line = fs * 1.2;
    // 월드 HUD(체력 핍 8px 높이) 아래부터 텍스트.
    let mut y = oy + 9.0 * scale + fs;
    // 웨이브·처치 수는 아레나(웨이브 시험장)에서만.
    if w.adv.place == crate::adventure::Place::Arena {
        draw_text(
            format!("wave {}  kills {}", w.wave, w.kills),
            ox + 4.0,
            y,
            fs,
            GRAY,
        );
    }
    y += line;
    if !w.debug.overlay {
        return;
    }
    let dbg = Color::new(0.6, 0.6, 0.6, 0.9);
    let x = ox + 4.0;
    let lines = [
        format!(
            "fps {}  x{}  pads {}  F1 debug  F2 hitboxes",
            get_fps(),
            scale,
            gamepad_count
        ),
        format!(
            "player {:?}  hp {}  inv {:.2}  cd {:.2}  hitstop {:.2}  trauma {:.2}",
            short_state(&w.player.state),
            w.player.hp,
            w.player.invuln,
            w.player.burrow_cd,
            w.feel.hitstop,
            w.feel.trauma
        ),
        "move WASD/stick  attack J/Z/X-btn  burrow K/X/Space/A-btn  restart R/Start".to_string(),
    ];
    for l in lines {
        draw_text(&l, x, y, fs, dbg);
        y += line;
    }
    for e in &w.enemies {
        draw_text(
            format!("{:?} {} hp {}", e.kind, e.state_name(), e.hp),
            x,
            y,
            fs,
            Color::new(0.5, 0.5, 0.5, 0.8),
        );
        y += line;
    }
}

fn short_state(s: &player::State) -> &'static str {
    match s {
        player::State::Free => "free",
        player::State::Attack { .. } => "attack",
        player::State::Burrow { .. } => "burrow",
        player::State::Emerge { .. } => "emerge",
        player::State::Drill { .. } => "drill",
        player::State::Recover { .. } => "recover",
        player::State::Dead => "dead",
    }
}

/// 전방 반원(채움). 삼각형 부채꼴.
fn draw_sector(origin: Vec2, dir: Vec2, r: f32, color: Color) {
    let base = dir.y.atan2(dir.x);
    let steps = 12;
    let half = std::f32::consts::FRAC_PI_2;
    for i in 0..steps {
        let a0 = base - half + (2.0 * half) * i as f32 / steps as f32;
        let a1 = base - half + (2.0 * half) * (i + 1) as f32 / steps as f32;
        draw_triangle(
            origin,
            origin + vec2(a0.cos(), a0.sin()) * r,
            origin + vec2(a1.cos(), a1.sin()) * r,
            color,
        );
    }
}

fn draw_sector_lines(origin: Vec2, dir: Vec2, r: f32, color: Color) {
    let base = dir.y.atan2(dir.x);
    let steps = 12;
    let half = std::f32::consts::FRAC_PI_2;
    let mut prev = origin + vec2((base - half).cos(), (base - half).sin()) * r;
    draw_line(origin.x, origin.y, prev.x, prev.y, 1.0, color);
    for i in 1..=steps {
        let a = base - half + (2.0 * half) * i as f32 / steps as f32;
        let p = origin + vec2(a.cos(), a.sin()) * r;
        draw_line(prev.x, prev.y, p.x, p.y, 1.0, color);
        prev = p;
    }
    draw_line(origin.x, origin.y, prev.x, prev.y, 1.0, color);
}
