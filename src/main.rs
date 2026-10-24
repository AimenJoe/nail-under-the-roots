//! Naïl: Under the Roots — 뼈대.
//!
//! 지금 있는 것: 내부 해상도 320×180 렌더 타깃 + 정수 배율 출력, 입력 액션 레이어, 움직이는 사각형.
//! 다음 할 것: docs/phase-1-combat-prototype.md 참조.

use macroquad::prelude::*;

/// 내부 해상도. 720p ×4, 1080p ×6, 1440p ×8 정수 배율. (KB: reference/macroquad/pixel-perfect-rendering.md)
const VW: u32 = 320;
const VH: u32 = 180;

fn window_conf() -> Conf {
    Conf {
        window_title: "Naïl: Under the Roots".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

/// 게임 로직이 보는 유일한 입력. 키보드·게임패드·터치는 전부 여기로 번역된다.
/// 코드 어디에서도 `KeyCode`를 직접 보지 않는다. (KB: reference/macroquad/gamepad-and-android-input.md)
#[derive(Default, Debug, Clone, Copy)]
struct Actions {
    /// 정규화된 이동 방향. 입력 없으면 0.
    move_dir: Vec2,
    attack: bool,
    dodge: bool,
    confirm: bool,
    cancel: bool,
}

fn read_keyboard() -> Actions {
    let mut m = Vec2::ZERO;
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        m.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        m.x += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        m.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        m.y += 1.0;
    }
    Actions {
        move_dir: if m.length_squared() > 0.0 {
            m.normalize()
        } else {
            m
        },
        attack: is_key_pressed(KeyCode::J) || is_key_pressed(KeyCode::Z),
        dodge: is_key_pressed(KeyCode::K)
            || is_key_pressed(KeyCode::X)
            || is_key_pressed(KeyCode::Space),
        confirm: is_key_pressed(KeyCode::Enter),
        cancel: is_key_pressed(KeyCode::Escape),
    }
}

struct Player {
    pos: Vec2,
    facing: Vec2,
    speed: f32,
}

#[macroquad::main(window_conf)]
async fn main() {
    let rt = render_target(VW, VH);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VW as f32, VH as f32));
    cam.render_target = Some(rt.clone());

    let mut player = Player {
        pos: vec2(VW as f32 / 2.0, VH as f32 / 2.0),
        facing: vec2(1.0, 0.0),
        speed: 90.0,
    };

    let bg = Color::from_rgba(30, 24, 36, 255);
    let grid = Color::from_rgba(40, 32, 48, 255);
    let mole = Color::from_rgba(230, 180, 120, 255);
    let mut paused = false;

    loop {
        let actions = read_keyboard();
        if actions.cancel {
            paused = !paused;
        }
        if actions.confirm {
            player.pos = vec2(VW as f32 / 2.0, VH as f32 / 2.0);
        }
        let dt = if paused { 0.0 } else { get_frame_time() };

        // --- update -------------------------------------------------------
        if actions.move_dir != Vec2::ZERO {
            player.facing = actions.move_dir;
        }
        player.pos += actions.move_dir * player.speed * dt;
        player.pos = player
            .pos
            .clamp(vec2(8.0, 8.0), vec2(VW as f32 - 8.0, VH as f32 - 8.0));

        // --- draw world at internal resolution ----------------------------
        set_camera(&cam);
        clear_background(bg);
        for x in (0..VW).step_by(16) {
            draw_line(x as f32, 0.0, x as f32, VH as f32, 1.0, grid);
        }
        for y in (0..VH).step_by(16) {
            draw_line(0.0, y as f32, VW as f32, y as f32, 1.0, grid);
        }
        let p = player.pos.round();
        draw_rectangle(p.x - 6.0, p.y - 6.0, 12.0, 12.0, mole);
        let nose = p + player.facing * 8.0;
        draw_circle(nose.x, nose.y, 2.0, WHITE);
        if actions.attack {
            draw_circle_lines(nose.x, nose.y, 10.0, 1.0, WHITE);
        }
        if actions.dodge {
            // 굴파기 자리 표시. 실제 명세는 docs/phase-1-combat-prototype.md
            draw_circle_lines(p.x, p.y, 14.0, 1.0, mole);
        }

        // --- present with integer scaling + letterbox ---------------------
        set_default_camera();
        clear_background(BLACK);
        let scale = (screen_width() / VW as f32)
            .min(screen_height() / VH as f32)
            .floor()
            .max(1.0);
        let (dw, dh) = (VW as f32 * scale, VH as f32 * scale);
        let x = ((screen_width() - dw) / 2.0).floor();
        let y = ((screen_height() - dh) / 2.0).floor();
        draw_texture_ex(
            &rt.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                ..Default::default()
            },
        );
        draw_text(
            format!(
                "{VW}x{VH} x{scale}  fps {}  WASD/arrows move, J attack, K dodge, Enter/Esc",
                get_fps()
            ),
            8.0,
            20.0,
            20.0,
            GRAY,
        );
        if paused {
            draw_text("PAUSED (Esc)", 8.0, 44.0, 20.0, YELLOW);
        }
        draw_text(
            format!("actions: {actions:?}"),
            8.0,
            screen_height() - 10.0,
            16.0,
            DARKGRAY,
        );

        next_frame().await
    }
}
