//! 그리기. 월드는 320×180 렌더 타깃에, HUD 텍스트는 실제 해상도에.
//! 월드 좌표는 그릴 때 반올림한다. (KB: reference/macroquad/pixel-perfect-rendering.md)

use macroquad::prelude::*;

use crate::art::{self, Art};
use crate::biome::{self, Decor};
use crate::combat::{Shape, StrikeKind};
use crate::enemy::{self, Enemy, Kind};
use crate::input::TouchOverlay;
use crate::player::{self, Player};
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

pub struct Renderer {
    rt: RenderTarget,
    cam: Camera2D,
    font: Option<Font>,
    art: Option<Art>,
}

impl Renderer {
    pub fn new(font: Option<Font>, art: Option<Art>) -> Self {
        let rt = render_target(VW, VH);
        rt.texture.set_filter(FilterMode::Nearest);
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, t::VIEW_W, t::VIEW_H));
        cam.render_target = Some(rt.clone());
        Self { rt, cam, font, art }
    }

    /// 월드 패스(내부 해상도). `w`가 없으면 타이틀 배경만 그린다.
    /// `alpha`: 마지막 틱 이후 지난 비율(0..1), 위치 보간. `shake_mul`: 설정의 흔들림 배율.
    pub fn begin_world(&mut self, w: Option<&World>, alpha: f32, shake_mul: f32) {
        let center = vec2(t::VIEW_W / 2.0, t::VIEW_H / 2.0);
        let shake = w
            .map(|w| (w.feel.shake_offset * shake_mul).round())
            .unwrap_or(Vec2::ZERO);
        self.cam.target = center + shake;
        set_camera(&self.cam);
        let Some(w) = w else {
            clear_background(BG);
            draw_title_background();
            return;
        };
        if let Some(art) = &self.art {
            clear_background(w.biome.clear());
            draw_room_art(art, w);
            draw_sorted(art, w, alpha);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_darkness(art, w, alpha);
        } else {
            clear_background(BG);
            draw_room(w);
            for p in &w.projectiles {
                draw_projectile(p, alpha);
            }
            for e in &w.enemies {
                draw_enemy(e, alpha, w.time);
            }
            draw_player(&w.player, alpha, w.time);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
        }
        if w.debug.hitboxes {
            draw_hitboxes(w, alpha);
        }
        draw_hud_world(w);
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
            font: self.font.clone(),
        }
    }

    /// 인게임 HUD(실제 해상도): 웨이브·처치, 디버그 줄.
    pub fn draw_hud(&self, ui: &Ui, w: &World, gamepad_count: usize) {
        draw_hud_screen(w, ui.scale, ui.ox, ui.oy, gamepad_count);
    }
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
                draw_claw_marks(origin, dir, hit.range, k, finisher);
            } else if at < a0 {
                draw_sector_lines(
                    origin,
                    dir,
                    hit.range * 0.5,
                    Color::new(0.6, 0.45, 0.3, 0.6),
                );
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

/// 발톱 자국: 전방 호를 따라 세 줄. 휘두르는 동안 펼쳐지고 끝에서 옅어진다.
fn draw_claw_marks(origin: Vec2, dir: Vec2, r: f32, k: f32, strong: bool) {
    let base = dir.y.atan2(dir.x);
    let span = std::f32::consts::FRAC_PI_2 * 0.95;
    let reveal = (k * 1.6).min(1.0);
    let fade = 1.0 - ((k - 0.6) / 0.4).clamp(0.0, 1.0);
    let col = if strong {
        Color::new(1.0, 1.0, 1.0, fade)
    } else {
        Color::new(0.97, 0.93, 0.85, 0.9 * fade)
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
    draw_circle(pos.x, pos.y, p.radius(), world::PROJECTILE);
    draw_circle_lines(pos.x, pos.y, p.radius() + 0.5, 1.0, WHITE);
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

/// 바닥 칸마다 (x, y, 장식). 방 그림은 칸 해시로 정해져 프레임마다 같다.
fn floor_decor(w: &World) -> Vec<(f32, f32, Decor)> {
    let r = w.room.inner;
    let mut out = Vec::new();
    let (cx0, cy0) = ((r.x / 16.0).floor() as i32, (r.y / 16.0).floor() as i32);
    let (cx1, cy1) = (
        (r.right() / 16.0).ceil() as i32,
        (r.bottom() / 16.0).ceil() as i32,
    );
    for cy in cy0..cy1 {
        for cx in cx0..cx1 {
            let (roll, pick) = biome::hash01(cx, cy, 7);
            if let Some(d) = w.biome.decor(roll, pick) {
                out.push((cx as f32 * 16.0, cy as f32 * 16.0, d));
            }
        }
    }
    out
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
    for (x, y, d) in floor_decor(w) {
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
    Pillar(usize),
    Player,
    Enemy(usize),
    Projectile(usize),
}

/// 3/4 시점: 발 위치(y)가 아래인 것이 앞. 그림자는 모두 먼저.
fn draw_sorted(art: &Art, w: &World, alpha: f32) {
    let mut list: Vec<(f32, Drawable)> = Vec::new();
    for (i, p) in w.room.pillars.iter().enumerate() {
        list.push((p.bottom(), Drawable::Pillar(i)));
    }
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    list.push((pp.y + 6.0, Drawable::Player));
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
                let p = w.room.pillars[i];
                let src = Rect::new(if i % 2 == 0 { 0.0 } else { 16.0 }, 32.0, 16.0, 32.0);
                draw_tile(&art.tiles, src, p.x, p.bottom() - 32.0, w.biome.wall_tint());
            }
            Drawable::Player => draw_nail(art, &w.player, alpha, w.time),
            Drawable::Enemy(i) => draw_enemy(&w.enemies[i], alpha, w.time),
            Drawable::Projectile(i) => draw_projectile(&w.projectiles[i], alpha),
        }
    }
}

/// Naïl 스프라이트. 상태 → 칸, 좌우 반전, 피격 플래시는 흰 실루엣 줄.
fn draw_nail(art: &Art, p: &Player, alpha: f32, time: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    let (col, row) = match p.state {
        player::State::Dead => (4, 1),
        player::State::Burrow { time: bt } => (((bt * 12.0) as u32) % 2, 2),
        player::State::Emerge { .. } => (2, 1),
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
fn draw_darkness(art: &Art, w: &World, alpha: f32) {
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
    let (vw, vh) = (t::VIEW_W + 8.0, t::VIEW_H + 8.0);
    draw_rectangle(-8.0, -8.0, vw + 8.0, y0 + 8.0, black);
    draw_rectangle(-8.0, y0 + s, vw + 8.0, vh - (y0 + s) + 8.0, black);
    draw_rectangle(-8.0, y0, x0 + 8.0, s, black);
    draw_rectangle(x0 + s, y0, vw - (x0 + s) + 8.0, s, black);
    for (x, y, d) in floor_decor(w) {
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
        Kind::Dummy => {
            // 허수아비: 말뚝 + 짚 뭉치
            draw_rectangle(
                pos.x - 1.5,
                pos.y - r,
                3.0,
                r * 2.0 + 3.0,
                Color::new(0.4, 0.3, 0.2, 1.0),
            );
            draw_circle(pos.x, pos.y - 2.0, r - 1.0, color);
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
        Kind::Dummy => t::DUMMY_HP,
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
