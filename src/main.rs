//! Naïl: Under the Roots — 1단계 전투 프로토타입 + 게임 틀.
//!
//! 루프: `App::frame()` = 입력 폴링 → 장면 갱신(게임은 고정 타임스텝 60Hz 틱 N번) → 가변 렌더(보간).
//! 모듈 경계: `input`(Actions) · `player` · `enemy` · `combat` · `room` · `render` · `world`(틱 순서) ·
//! `app`(장면·메뉴) · `ui`(패널 그리기) · `i18n`(언어별 글) · `cutscene`(JRPG식 컷씬) · `loot`(반딧불·산딸기) · `scenario`(개발용 시작 구성) · `sfx` · `tuning`(수치).
//! 범위와 판정 기준: docs/phase-1-combat-prototype.md
#![allow(clippy::new_without_default)]

mod adventure;
mod app;
mod art;
mod biome;
mod camp;
mod combat;
mod cutscene;
mod daily;
mod dream;
mod dungeon;
mod enemy;
mod equipment;
mod field;
mod i18n;
mod illust;
mod input;
mod items;
mod journal;
mod knots;
mod loading;
mod loot;
mod map;
mod moveset;
mod player;
mod prefs;
mod profile;
mod render;
mod room;
mod route;
mod scenario;
mod sfx;
mod species;
mod story;
mod sunny;
mod talents;
mod terrain;
mod tuning;
mod tutorial;
mod ui;
mod village;
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

async fn load_font(path: &str) -> Option<Font> {
    match load_ttf_font(path).await {
        Ok(mut f) => {
            f.set_filter(FilterMode::Nearest);
            Some(f)
        }
        Err(e) => {
            eprintln!("폰트 로드 실패 {path}: {e:?}");
            None
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 데스크톱·웹은 "assets/…", Android는 APK assets 루트. (macroquad set_pc_assets_folder)
    set_pc_assets_folder("assets");
    // 기기 언어로 시작한다. 설정에서 바꿀 수 있다(아직 저장하지 않음, docs/decisions/0013).
    i18n::set_lang(i18n::detect());
    // 언어별 폰트를 전부 읽는다. 없으면 기본 폰트(한글·한자 없음)로라도 뜨게 한다.
    // 준비 단계마다 웹 로딩 화면의 막대를 채운다(loading::STEPS = 기본 폰트 1 + 일본어·간체 2 + 그림 1 + 소리 1).
    let mut done = 0;
    let mut fonts = render::Fonts {
        base: load_font(i18n::BASE_FONT).await,
        by_file: Vec::new(),
    };
    done += 1;
    loading::step(done);
    for l in i18n::Lang::ALL {
        let file = l.font_file();
        if file != i18n::BASE_FONT && !fonts.by_file.iter().any(|(f, _)| *f == file) {
            if let Some(f) = load_font(file).await {
                fonts.by_file.push((file, f));
            }
            done += 1;
            loading::step(done);
        }
    }
    let art = art::Art::load().await;
    if art.is_none() {
        eprintln!("그림 에셋(assets/art) 로드 실패: 도형으로 그린다");
    }
    done += 1;
    loading::step(done);
    let sfx = sfx::Sfx::load().await;
    done += 1;
    loading::step(done);
    let mut app = app::App::new(sfx, fonts, art);
    let mut first = true;
    loop {
        app.frame();
        next_frame().await;
        if first {
            // 첫 프레임이 화면에 나간 뒤 로딩 화면을 걷는다(그 전에 걷으면 검은 화면이 한 번 보인다).
            first = false;
            loading::ready();
        }
    }
}
