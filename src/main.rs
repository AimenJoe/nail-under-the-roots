//! Naïl: Under the Roots — 1단계 전투 프로토타입 + 게임 틀.
//!
//! 루프: `App::frame()` = 입력 폴링 → 장면 갱신(게임은 고정 타임스텝 60Hz 틱 N번) → 가변 렌더(보간).
//! 모듈 경계: `input`(Actions) · `player` · `enemy` · `combat` · `room` · `render` · `world`(틱 순서) ·
//! `app`(장면·메뉴) · `ui`(패널 그리기) · `i18n`(언어별 글) · `cutscene`(JRPG식 컷씬) · `scenario`(개발용 시작 구성) · `sfx` · `tuning`(수치).
//! 범위와 판정 기준: docs/phase-1-combat-prototype.md
#![allow(clippy::new_without_default)]

mod app;
mod art;
mod biome;
mod combat;
mod cutscene;
mod enemy;
mod equipment;
mod i18n;
mod input;
mod moveset;
mod player;
mod render;
mod room;
mod scenario;
mod sfx;
mod tuning;
mod tutorial;
mod ui;
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
    // 데스크톱·웹은 "assets/…", Android는 APK assets 루트. (macroquad set_pc_assets_folder)
    set_pc_assets_folder("assets");
    // 기기 언어로 시작한다. 설정에서 바꿀 수 있다(아직 저장하지 않음, docs/decisions/0013).
    i18n::set_lang(i18n::detect());
    let font = match load_ttf_font("fonts/Galmuri11-Condensed.ttf").await {
        Ok(mut f) => {
            f.set_filter(FilterMode::Nearest);
            Some(f)
        }
        Err(e) => {
            // 폰트가 없으면 기본 폰트(한글 없음)로라도 뜨게 한다.
            eprintln!("폰트 로드 실패: {e:?}");
            None
        }
    };
    let art = art::Art::load().await;
    if art.is_none() {
        eprintln!("그림 에셋(assets/art) 로드 실패: 도형으로 그린다");
    }
    let sfx = sfx::Sfx::load().await;
    let mut app = app::App::new(sfx, font, art);
    loop {
        app.frame();
        next_frame().await
    }
}
