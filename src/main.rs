//! Naïl: Under the Roots — 1단계 전투 프로토타입.
//!
//! 루프: 입력 폴링 → 고정 타임스텝(60Hz) 로직 틱 N번 → 가변 렌더(보간).
//! 모듈 경계: `input`(Actions) · `player` · `enemy` · `combat` · `room` · `render`. 수치는 `tuning`.
//! 범위와 판정 기준: docs/phase-1-combat-prototype.md
#![allow(clippy::new_without_default)]

mod combat;
mod enemy;
mod input;
mod player;
mod render;
mod room;
mod sfx;
mod tuning;
mod world;

use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Naïl: Under the Roots".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let sfx = sfx::Sfx::load().await;
    let mut input = input::Input::new();
    let mut renderer = render::Renderer::new();
    let mut world = world::World::new();
    let mut acc = 0.0f32;

    loop {
        input.poll();
        acc += get_frame_time().min(tuning::MAX_FRAME_DT);
        while acc >= tuning::TICK_DT {
            let actions = input.take();
            world.tick(&actions, &sfx);
            acc -= tuning::TICK_DT;
        }
        let alpha = acc / tuning::TICK_DT;
        renderer.draw(&world, alpha, input.gamepad_count);
        next_frame().await
    }
}
