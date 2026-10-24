//! 그리기. 월드는 320×180 렌더 타깃에, HUD 텍스트는 실제 해상도에.
//! 월드 좌표는 그릴 때 반올림한다. (KB: reference/macroquad/pixel-perfect-rendering.md)

use macroquad::prelude::*;

use crate::combat::{Shape, StrikeKind};
use crate::enemy::{self, Enemy, Kind};
use crate::player::{self, Player};
use crate::tuning as t;
use crate::world::{self, FxKind, World};

const VW: u32 = t::VIEW_W as u32;
const VH: u32 = t::VIEW_H as u32;

const BG: Color = Color::new(0.12, 0.09, 0.14, 1.0);
const GRID: Color = Color::new(0.16, 0.12, 0.19, 1.0);
const WALL_COLOR: Color = Color::new(0.30, 0.22, 0.20, 1.0);
const SHADOW: Color = Color::new(0.05, 0.03, 0.06, 0.8);

pub struct Renderer {
    rt: RenderTarget,
    cam: Camera2D,
}

impl Renderer {
    pub fn new() -> Self {
        let rt = render_target(VW, VH);
        rt.texture.set_filter(FilterMode::Nearest);
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H));
        cam.render_target = Some(rt.clone());
        Self { rt, cam }
    }

    /// `alpha`: 마지막 틱 이후 지난 비율(0..1). 위치 보간에 쓴다.
    pub fn draw(&mut self, w: &World, alpha: f32, gamepad_count: usize) {
        // --- 월드 패스 (내부 해상도) ---
        let center = vec2(t::VIEW_W / 2.0, t::VIEW_H / 2.0);
        self.cam.target = center + w.feel.shake_offset;
        set_camera(&self.cam);
        clear_background(BG);
        draw_room(w);
        for p in &w.projectiles {
            let pos = lerp(p.prev_pos, p.pos, alpha);
            draw_circle(pos.x, pos.y, p.radius(), world::PROJECTILE);
            draw_circle_lines(pos.x, pos.y, p.radius() + 0.5, 1.0, WHITE);
        }
        for e in &w.enemies {
            draw_enemy(e, alpha, w.time);
        }
        draw_player(&w.player, alpha, w.time);
        draw_fx(w);
        if w.debug.hitboxes {
            draw_hitboxes(w, alpha);
        }
        draw_hud_world(w);

        // --- 화면 패스 (정수 배율 + 레터박스) ---
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

        draw_hud_screen(w, scale, x, y, gamepad_count);
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
    for p in &w.room.pillars {
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

    match p.state {
        player::State::Attack { idx, time: at, dir } => {
            let (a0, a1) = t::CLAW_ACTIVE[idx];
            let origin = pos + dir * 2.0;
            if at >= a0 && at <= a1 + 0.03 {
                let c = if idx == 2 { WHITE } else { world::MOLE };
                draw_sector(origin, dir, t::CLAW_RANGE, Color::new(c.r, c.g, c.b, 0.55));
                draw_sector_lines(origin, dir, t::CLAW_RANGE, c);
            } else if at < a0 {
                draw_sector_lines(origin, dir, t::CLAW_RANGE * 0.6, world::DIRT);
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

fn draw_enemy(e: &Enemy, alpha: f32, time: f32) {
    let mut pos = lerp(e.prev_pos, e.pos, alpha);
    let base = world::enemy_color(e.kind);
    let r = e.radius();
    let mut color = if e.flash > 0.0 { WHITE } else { base };
    match e.state {
        enemy::State::Windup { time: wt } => {
            // 예고: 떨림 + 점점 밝아짐
            pos.x += if ((time * 60.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let k = (wt / t::CHARGER_WINDUP).min(1.0);
            if e.flash <= 0.0 {
                color = Color::new(base.r + (1.0 - base.r) * k * 0.7, base.g, base.b, 1.0);
            }
        }
        enemy::State::Rest { .. } => {
            if e.flash <= 0.0 {
                color = Color::new(base.r * 0.6, base.g * 0.6, base.b * 0.6, 1.0);
            }
        }
        enemy::State::Stunned { time: st } => {
            if e.flash <= 0.0 {
                color = Color::new(base.r * 0.5, base.g * 0.5, base.b * 0.5, 1.0);
            }
            // 별 대신 작은 점이 돈다
            let a = st * 12.0;
            let s = pos + vec2(a.cos(), a.sin() * 0.5) * (r + 3.0) - vec2(0.0, r);
            draw_circle(s.x.round(), s.y.round(), 1.0, YELLOW);
        }
        enemy::State::Charge { dir, .. } => {
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
    match e.kind {
        Kind::Charger => {
            draw_rectangle(pos.x - r, pos.y - r, r * 2.0, r * 2.0, color);
        }
        Kind::Shooter => {
            draw_circle(pos.x, pos.y, r, color);
            if let enemy::State::Aim { time: at } = e.state {
                let k = at / t::SHOOTER_AIM_TIME;
                let muzzle = pos + e.facing * (r + 2.0);
                draw_circle(muzzle.x, muzzle.y, 1.0 + 2.5 * k, world::PROJECTILE);
            }
        }
    }
    // 코(방향)
    let nose = pos + e.facing * (r + 1.0);
    draw_circle(nose.x.round(), nose.y.round(), 1.5, BLACK);
    // 체력 점
    let max = match e.kind {
        Kind::Charger => t::CHARGER_HP,
        Kind::Shooter => t::SHOOTER_HP,
    };
    for i in 0..max {
        let c = if i < e.hp { WHITE } else { DARKGRAY };
        draw_rectangle(
            pos.x - max as f32 * 1.5 + i as f32 * 3.0,
            pos.y - r - 5.0,
            2.0,
            2.0,
            c,
        );
    }
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
        };
        match s.shape {
            Shape::Circle { center, r } => draw_circle_lines(center.x, center.y, r, 1.0, col),
            Shape::Sector { origin, dir, r } => draw_sector_lines(origin, dir, r, col),
        }
    }
}

fn draw_hud_world(w: &World) {
    // 체력 핍(왼쪽 위, 벽 위에)
    for i in 0..t::PLAYER_HP {
        let c = if i < w.player.hp {
            world::MOLE
        } else {
            Color::new(0.2, 0.15, 0.2, 1.0)
        };
        draw_rectangle(3.0 + i as f32 * 7.0, 1.0, 6.0, 6.0, c);
    }
    // 굴파기 쿨다운 바(오른쪽 위)
    let cd = w.player.burrow_cd / t::BURROW_COOLDOWN;
    let bw = 24.0;
    draw_rectangle(
        t::VIEW_W - 3.0 - bw,
        2.0,
        bw,
        4.0,
        Color::new(0.2, 0.15, 0.2, 1.0),
    );
    draw_rectangle(t::VIEW_W - 3.0 - bw, 2.0, bw * (1.0 - cd), 4.0, world::DIRT);
}

fn draw_hud_screen(w: &World, scale: f32, ox: f32, oy: f32, gamepad_count: usize) {
    let fs = 16.0 * (scale / 4.0).max(1.0);
    let line = fs * 1.2;
    // 월드 HUD(체력 핍 8px 높이) 아래부터 텍스트.
    let mut y = oy + 9.0 * scale + fs;
    draw_text(
        format!("wave {}  kills {}", w.wave, w.kills),
        ox + 4.0,
        y,
        fs,
        GRAY,
    );
    y += line;
    if !w.player.is_alive() {
        let msg = "DEAD  -  R / Start: restart";
        let dim = measure_text(msg, None, (fs * 1.6) as u16, 1.0);
        draw_text(
            msg,
            (screen_width() - dim.width) / 2.0,
            screen_height() / 2.0,
            fs * 1.6,
            RED,
        );
    }
    if w.paused {
        draw_text(
            "PAUSED  (Esc / B / Select)",
            ox + 8.0,
            screen_height() / 2.0,
            fs * 1.4,
            YELLOW,
        );
    }
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
