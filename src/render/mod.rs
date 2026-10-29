//! 그리기. 월드는 320×180 렌더 타깃에, HUD 텍스트는 실제 해상도에.
//! 월드 좌표는 그릴 때 반올림한다. (KB: reference/macroquad/pixel-perfect-rendering.md)

use macroquad::prelude::*;

use crate::art::{self, Art};
use crate::biome::{self, Biome, Decor};
use crate::combat::{Shape, StrikeKind};
use crate::enemy::{self, Enemy};
use crate::input::TouchOverlay;
use crate::loot;
use crate::map::Cell;
use crate::player::{self, Player};
use crate::room::PropKind;
use crate::sunny;
use crate::tuning as t;
use crate::ui::Ui;
use crate::world::{self, FxKind, World};

// 0169 R3: 한 파일 6,976줄을 그리는 것별로 나눴다. 함수 몸은 그대로, 자식은 `use super::*`로 부모의 가져오기를 함께 쓴다.
mod actors;
mod deep;
mod descent;
mod fx;
mod help;
mod hud;
mod lights;
mod props;
mod shadow;
mod sorted;
pub use actors::*;
use deep::*;
use descent::*;
use fx::*;
pub use hud::*;
pub use lights::*;
use props::*;
pub use shadow::*;
use sorted::*;

const VW: u32 = t::VIEW_W as u32;
const VH: u32 = t::VIEW_H as u32;
/// 렌더 타깃은 화면보다 오른쪽·아래로 1px 크다. 카메라의 소수점 부분은 올릴 때 화면 px로 밀어서 보여 준다
/// (부드러운 스크롤 + 그림은 정수 픽셀 격자 그대로). 아래 `present`.
const RW: f32 = t::VIEW_W + 1.0;
const RH: f32 = t::VIEW_H + 1.0;

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
    /// 입자 효과 설정(0..=1, 0141): 일러스트·타이틀의 입자를 이 비율만 그린다. 앱이 프레임마다 넣는다.
    pub effects: f32,
    /// 글자를 미리 구운 (언어, 배율, dpi). 바뀌면(언어 전환·창 크기) 다시 굽는다.
    warmed: std::cell::Cell<Option<(crate::i18n::Lang, u32, u32)>>,
    /// 이번 프레임 월드 화면의 왼쪽 위(월드 좌표, 흔들림 포함). 월드 패스 뒤에는 카메라가 화면 고정이라
    /// 컷씬 배우는 이걸 빼서 그린다(`to_view`).
    view_origin: Vec2,
    /// 카메라의 소수점 부분(0..1, 게임 px). 올릴 때 이만큼 화면을 민다.
    subpixel: Vec2,
    /// 구운 지도(terrain.rs, 0062): 열쇠가 같으면 다시 굽지 않는다. 최근 장소 셋을 둔다(돌아가면 바로, 0068).
    terrain: Vec<(u64, Texture2D)>,
    /// 굽는 중인 지도(0068): 프레임마다 `BAKE_BUDGET`만큼 굽고, 그동안 월드 화면은 검정이다.
    baking: Option<(u64, crate::terrain::Baker)>,
    /// 빛 더하기(0076): 바탕에 빛을 더한다. 못 만들면(셰이더 실패) 보통 섞기로 그린다.
    light: Option<Material>,
    /// 어둠 지도(0095): 월드와 같은 크기. 어둠으로 채우고 빛 자리마다 어둠을 뺀 뒤 월드 위에 덮는다.
    dark_rt: RenderTarget,
    /// 어둠 빼기 재질(0095). 못 만들면 예전 방식(플레이어 둘레만 뚫린 막).
    cut: Option<Material>,
    /// 해 그림자(0102): 밝은 곳에서 서 있는 것들의 눕힌 모양을 모으는 렌더 타깃(월드와 같은 크기).
    sun_rt: RenderTarget,
    /// 모은 모양을 한 가지 진하기의 그림자로 얹는 재질. 못 만들면 해 그림자 없이 그린다.
    silhouette: Option<Material>,
    /// 내려가는 전환(0156)이 시작된 뒤 흐른 실제 시간. 굽는 동안(월드 시계가 멈춘 동안)에도 흙 부스러기가 움직인다.
    descent_clock: f32,
}

/// 한 프레임에 지도를 굽는 시간(초). 굽는 동안 화면은 검정이라 프레임이 조금 길어져도 보이지 않는다.
/// 짧게 자를수록 검정이 길어진다(브라우저 굽기 ≈ 0.15초 → 12ms면 13프레임쯤).
const BAKE_BUDGET: f64 = 0.012;
/// 구운 지도를 몇 장까지 둘까(장소 하나 ≈ 2~6MB 텍스처).
const TERRAIN_KEEP: usize = 3;

impl Renderer {
    pub fn new(fonts: Fonts, art: Option<Art>) -> Self {
        let rt = render_target(VW + 1, VH + 1);
        rt.texture.set_filter(FilterMode::Nearest);
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, RW, RH));
        cam.render_target = Some(rt.clone());
        Self {
            rt,
            cam,
            fonts,
            art,
            effects: 1.0,
            warmed: std::cell::Cell::new(None),
            view_origin: Vec2::ZERO,
            subpixel: Vec2::ZERO,
            terrain: Vec::new(),
            baking: None,
            light: light_material(),
            dark_rt: {
                let d = render_target(VW + 1, VH + 1);
                d.texture.set_filter(FilterMode::Nearest);
                d
            },
            cut: cut_material(),
            sun_rt: {
                let d = render_target(VW + 1, VH + 1);
                d.texture.set_filter(FilterMode::Nearest);
                d
            },
            silhouette: silhouette_material(),
            descent_clock: 0.0,
        }
    }

    /// 해·모닥불 그림자(0102·0103). 서 있는 것들을 발 줄 축으로 눕혀 `sun_rt`에 모으고, 모양만 남겨 한 번에 얹는다.
    /// 그림자끼리 겹쳐도 더 진해지지 않는다(겹쳐 그린 반투명은 겹친 곳이 진해진다).
    /// 밝은 곳: 해(오른쪽 아래), 모닥불 곁은 모닥불 바깥쪽. 어두운 곳: 모닥불 빛이 닿는 데만, 멀수록 옅게.
    fn draw_sun_shadows(&self, art: &Art, w: &World, alpha: f32) {
        let Some(m) = &self.silhouette else { return };
        let bright = w.biome.darkness().0 <= 0.0;
        let center = self.cam.target;
        let fire = campfires(w)
            .into_iter()
            .filter(|f| (*f - center).length() < RW + t::FIRE_SHADOW_REACH)
            .min_by(|a, b| (*a - center).length().total_cmp(&(*b - center).length()));
        if !bright && fire.is_none() {
            return;
        }
        let dc = Camera2D {
            target: self.cam.target,
            zoom: self.cam.zoom,
            offset: self.cam.offset,
            rotation: self.cam.rotation,
            render_target: Some(self.sun_rt.clone()),
            viewport: None,
        };
        set_camera(&dc);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        draw_sorted(art, w, alpha, Some(ShadowLight { fire, bright }));
        set_camera(&self.cam);
        // 어두운 곳은 모닥불에서 멀수록 옅게(셰이더가 그림자 자리마다 모닥불까지 거리를 잰다).
        let f = fire.unwrap_or_default();
        m.set_uniform(
            "Fire",
            vec4(
                f.x,
                f.y,
                t::FIRE_SHADOW_REACH,
                if bright { 0.0 } else { 1.0 },
            ),
        );
        m.set_uniform("Fade", vec2(t::FIRE_SHADOW_FADE.0, t::FIRE_SHADOW_FADE.1));
        gl_use_material(m);
        let a = if bright {
            t::SUN_SHADOW_ALPHA
        } else {
            t::FIRE_SHADOW_ALPHA
        };
        draw_texture_ex(
            &self.sun_rt.texture,
            self.view_origin.x,
            self.view_origin.y,
            Color::new(1.0, 1.0, 1.0, a),
            DrawTextureParams {
                dest_size: Some(vec2(RW, RH)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// 어둠 지도(0095): 어둠으로 채운 렌더 타깃에 빛 자리(플레이어·야광버섯·모닥불·반딧불·저장 지점·문·출구)마다
    /// 어둠을 빼고, 월드 위에 덮는다. 예전엔 플레이어 둘레 하나만 뚫려 버섯 곁 바닥도 깜깜했다.
    fn draw_light_map(&self, art: &Art, w: &World, alpha: f32, cam: Vec2) {
        let (dark, radius) = w.biome.darkness();
        if dark <= 0.0 {
            return;
        }
        let dc = Camera2D {
            target: self.cam.target,
            zoom: self.cam.zoom,
            offset: self.cam.offset,
            rotation: self.cam.rotation,
            render_target: Some(self.dark_rt.clone()),
            viewport: None,
        };
        set_camera(&dc);
        clear_background(Color::new(0.0, 0.0, 0.0, dark));
        if let Some(m) = &self.cut {
            gl_use_material(m);
        }
        let blob = |p: Vec2, r: f32, k: f32| {
            draw_texture_ex(
                &art.light,
                p.x - r,
                p.y - r,
                Color::new(0.0, 0.0, 0.0, k.clamp(0.0, 1.0)),
                DrawTextureParams {
                    dest_size: Some(vec2(r * 2.0, r * 2.0)),
                    ..Default::default()
                },
            );
        };
        for l in light_sources(w, alpha, cam, radius) {
            blob(l.pos, l.radius, l.strength);
        }
        gl_use_default_material();
        set_camera(&self.cam);
        draw_texture_ex(
            &self.dark_rt.texture,
            self.view_origin.x,
            self.view_origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(RW, RH)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    /// 지도를 굽는 중이다: 앱은 월드 틱을 멈춘다(화면이 검정인 동안 장소 바꾸기가 끝나 버리지 않게, 0068).
    pub fn baking(&self) -> bool {
        self.baking.is_some()
    }

    /// 월드 패스(내부 해상도). `w`가 없으면 타이틀 배경만 그린다.
    /// `alpha`: 마지막 틱 이후 지난 비율(0..1), 위치 보간. `shake_mul`: 설정의 흔들림 배율.
    pub fn begin_world(&mut self, w: Option<&World>, alpha: f32, shake_mul: f32) {
        let center = vec2(RW / 2.0, RH / 2.0);
        // 흔들림 + 카메라 킥(0168), 둘 다 설정 배율을 따른다.
        let shake = w
            .map(|w| ((w.juice.feel.shake_offset + w.juice.feel.kick) * shake_mul).round())
            .unwrap_or(Vec2::ZERO);
        // 모험: 카메라 왼쪽 위(보간, 정수 px). 아레나·타이틀은 0.
        // 카메라는 반올림하지 않는다(`lerp`는 그림 위치용으로 반올림한다). 소수점은 아래에서 따로 쓴다.
        let cam = w
            .map(|w| w.adv.prev_cam + (w.adv.cam - w.adv.prev_cam) * alpha)
            .unwrap_or(Vec2::ZERO);
        // 걷는 속도(90px/s = 틱당 1.5px)는 정수 px가 아니라, 카메라를 정수로 굴리면 화면 전체가 1·2px씩
        // 번갈아 걸으며 떨린다(사람 피드백 "화면 전체가 조금 떨려 눈이 피로"). 정수 부분으로 그리고, 소수점 부분은
        // present에서 화면 px 단위로 민다. 그림(타일·스프라이트)은 렌더 타깃의 정수 격자에 그대로 남는다.
        let whole = cam.floor();
        self.subpixel = cam - whole;
        self.cam.target = whole + center + shake;
        self.view_origin = whole + shake;
        set_camera(&self.cam);
        let Some(w) = w else {
            clear_background(BG);
            draw_title_background();
            return;
        };
        let descent = w.descent();
        if descent.is_some() {
            self.descent_clock += get_frame_time().min(0.1);
        } else {
            self.descent_clock = 0.0;
        }
        if let Some(art) = &self.art {
            clear_background(w.biome.clear());
            if let Some(g) = &w.room.grid {
                let paths = match &w.adv.dungeon {
                    Some(d) if w.adv.place == crate::adventure::Place::Dungeon => {
                        crate::terrain::dungeon_paths(d)
                    }
                    _ => Vec::new(),
                };
                let sunny_img = art.sunny.as_ref().map(|s| &s.sheet_img);
                let key = crate::terrain::key(g, w.biome, &paths) ^ u64::from(sunny_img.is_some());
                if let Some(i) = self.terrain.iter().position(|(k, _)| *k == key) {
                    let hit = self.terrain.remove(i);
                    self.terrain.insert(0, hit);
                    self.baking = None;
                } else {
                    if self.baking.as_ref().map(|(k, _)| *k) != Some(key) {
                        self.baking = Some((
                            key,
                            crate::terrain::Baker::new(g, w.biome, &paths, &campfires(w)),
                        ));
                    }
                    let (_, baker) = self.baking.as_mut().unwrap();
                    let t0 = get_time();
                    let done = baker.step(Some(&art.tiles_img), sunny_img, &mut || {
                        get_time() - t0 > BAKE_BUDGET
                    });
                    if !done {
                        // 아직 굽는 중: 화면은 검정(장소 바꾸기의 어두운 한가운데가 조금 길어진다).
                        // 내려가는 전환이면 검정 대신 지층 단면(0156).
                        clear_background(BLACK);
                        if let Some(d) = descent {
                            self.cam.target = center;
                            set_camera(&self.cam);
                            draw_descent(Some(art), w, d, self.descent_clock, 1.0);
                        }
                        return;
                    }
                    let (_, baker) = self.baking.take().unwrap();
                    let tex = Texture2D::from_image(&baker.finish());
                    tex.set_filter(FilterMode::Nearest);
                    self.terrain.insert(0, (key, tex));
                    self.terrain.truncate(TERRAIN_KEEP);
                }
                let sheet = art.sunny.as_ref().map(|s| &s.sheet);
                draw_grid_art(w, cam, &self.terrain[0].1, sheet, art.hub.as_ref());
            } else {
                draw_room_art(art, w);
            }
            if let Some(sp) = &w.spring {
                draw_spring(sp, w.time);
            }
            for at in w.story.petals() {
                draw_left_petal(*at, w.time);
            }
            for c in &w.hazards.cocoons {
                draw_cocoon(*c, w.time);
            }
            if let Some(d) = &w.deep {
                draw_deep(art, d, w, alpha);
            }
            self.draw_sun_shadows(art, w, alpha);
            draw_drill_trail(Some(art), &w.player, alpha);
            draw_charge_under(&w.player, alpha);
            draw_jump_aim(w, alpha);
            draw_telegraphs(w, alpha);
            if self.cut.is_some() {
                draw_mushrooms(art, w, cam);
            }
            draw_cast_shadows(w, alpha, cam, self.silhouette.is_some());
            draw_sorted(art, w, alpha, None);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_falling_rocks(w);
            draw_waves(w);
            draw_ambient(w, cam, false);
            draw_overhead_falls(w, cam);
            if self.cut.is_some() {
                self.draw_light_map(art, w, alpha, cam);
                for l in &w.loot {
                    draw_loot(l, alpha, w.time, true);
                }
            } else {
                draw_darkness(art, w, alpha, cam);
            }
            draw_gloom(art, w, cam);
            draw_save_lights(w, cam, self.light.as_ref());
            draw_world_lights(
                w,
                cam,
                alpha,
                self.light.as_ref(),
                art.sunny.as_ref().map(|s| &s.sheet),
            );
            if draw_super_dim(w, cam) {
                draw_nail(art, &w.player, w.inv.fur, alpha, w.time);
                draw_player_fx(&w.player, alpha);
            }
        } else {
            clear_background(BG);
            draw_room(w);
            draw_telegraphs(w, alpha);
            for p in &w.projectiles {
                draw_projectile(p, alpha);
            }
            for e in &w.enemies {
                draw_enemy(None, e, alpha, w.time, w.juice.feel.hitstop);
            }
            draw_drill_trail(None, &w.player, alpha);
            draw_charge_under(&w.player, alpha);
            draw_jump_aim(w, alpha);
            draw_player(&w.player, alpha, w.time);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_falling_rocks(w);
            draw_waves(w);
            if draw_super_dim(w, cam) {
                draw_player(&w.player, alpha, w.time);
                draw_player_fx(&w.player, alpha);
            }
        }
        if w.debug.hitboxes {
            draw_hitboxes(w, alpha);
        }
        // 화면 전체 덮개는 화면에 고정(흔들림·스크롤 없이). HUD 막대는 화면 패스(draw_hud)에서 — 렌더 타깃에
        // 그리면 소수점 밀기에 같이 흔들린다.
        self.cam.target = center;
        set_camera(&self.cam);
        if w.juice.feel.flash > 0.0 {
            // 벽 쿵 번쩍임. 흔들림 설정을 따른다(0이면 끈다: 눈이 예민한 사람).
            let k = (w.juice.feel.flash / t::DRILL_WALL_FLASH).min(1.0) * shake_mul.min(1.0);
            draw_rectangle(0.0, 0.0, RW, RH, Color::new(1.0, 0.97, 0.9, 0.55 * k));
        }
        if w.juice.feel.hurt > 0.0 {
            // 맞음 테두리: 가장자리부터 안으로 옅어지는 붉은 띠. 흔들림 설정을 따른다(번쩍임과 같다).
            let k = (w.juice.feel.hurt / t::PLAYER_HURT_EDGE).min(1.0) * shake_mul.min(1.0);
            for i in 0..8 {
                let a = 0.6 * k * (1.0 - i as f32 / 8.0);
                let inset = i as f32 * 2.0;
                draw_rectangle_lines(
                    inset,
                    inset,
                    RW - 2.0 * inset,
                    RH - 2.0 * inset,
                    4.0,
                    Color::new(0.95, 0.12, 0.14, a),
                );
            }
        }
        let fade = w.fade_alpha();
        if fade > 0.0 {
            draw_rectangle(0.0, 0.0, RW, RH, Color::new(0.0, 0.0, 0.0, fade));
        }
        if let Some(d) = w.descent() {
            draw_descent(self.art.as_ref(), w, d, self.descent_clock, d.cover);
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
        // 카메라 소수점만큼 화면 px로 민다(정수 화면 px로 반올림: 배율 4면 게임 px의 1/4 단위로 부드럽게).
        let shift = (self.subpixel * scale).round();
        draw_texture_ex(
            &self.rt.texture,
            x - shift.x,
            y - shift.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(RW * scale, RH * scale)),
                // macroquad 0.4.16: from_display_rect + render_target 조합은 상하가 뒤집혀 나온다(실행 확인 2026-10-24).
                flip_y: true,
                ..Default::default()
            },
        );
        // 1px 여분이 화면 칸 밖(오른쪽·아래 레터박스)으로 나온 것을 덮는다.
        let (sw, sh) = (screen_width(), screen_height());
        draw_rectangle(x + dw, 0.0, sw - x - dw, sh, BLACK);
        draw_rectangle(0.0, y + dh, sw, sh - y - dh, BLACK);
        let ui = Ui {
            scale,
            ox: x,
            oy: y,
            font: self.fonts.for_lang(crate::i18n::lang()),
            base: self.fonts.base.clone(),
            outline: false,
        };
        let key = (
            crate::i18n::lang(),
            scale.to_bits(),
            miniquad::window::dpi_scale().to_bits(),
        );
        if self.warmed.get() != Some(key) {
            self.warmed.set(Some(key));
            ui.warm(&crate::i18n::s().glyphs(), crate::ui::FONT_PX);
            // 두 배 크기 글(타이틀 로고·쓰러짐)도 미리 굽는다. 그리는 도중 아틀라스가 커지면 그 프레임 글이 지워진 텍스처를 본다.
            let big = format!("Naïl{}", crate::i18n::s().dead);
            ui.warm(&big, crate::ui::FONT_PX * 2.0);
        }
        ui
    }

    /// 월드 좌표 → 이번 프레임 화면 좌표(월드 패스가 끝난 뒤 그리는 컷씬 배우용).
    /// 오프닝은 카메라가 (0, 0)인 첫 화면이라 그냥 써도 맞았지만, 지도 다른 곳의 컷씬(보스 처치)은 화면 밖으로 나갔다.
    pub fn to_view(&self, p: Vec2) -> Vec2 {
        p - self.view_origin
    }

    /// 컷씬 배우(그림이 있는 이웃). 그림이 없으면 색 동그라미.
    pub fn draw_actor_sprite(&self, row: u32, pos: Vec2, facing: Vec2, moving: bool, color: Color) {
        match self.art.as_ref() {
            Some(art) if art.critter(0, row).is_some() => {
                draw_critter_at(art, row, pos, facing.x < 0.0, moving, get_time() as f32)
            }
            _ => draw_actor(pos, facing, color),
        }
    }

    /// 도감 그림: critters `row`줄 `frame`칸(새 그림 시트면 그 시트, `Art::critter`)을 논리 사각형 `r`에 —
    /// 칸이 아니라 몸 자리(`CritterCell::body`)를 가운데에 맞춰 키운다.
    pub fn draw_critter(&self, ui: &Ui, r: Rect, row: u32, frame: u32) {
        let Some(c) = self.art.as_ref().and_then(|a| a.critter(frame, row)) else {
            return;
        };
        draw_body_fit(ui, r, c.tex, c.src, c.size, c.body);
    }

    /// 안내 화면의 힌트 그림(0180): 논리 사각형 `r` 안에, 게임 픽셀을 화면 배율 그대로(정수) 찍는다.
    pub fn draw_help_pic(&self, ui: &Ui, pic: crate::help::Pic, r: Rect, t: f32) {
        ui.rect(r, Color::new(0.20, 0.30, 0.18, 1.0));
        let o = ui.px(vec2(r.x, r.y));
        let gl = unsafe { get_internal_gl() };
        gl.quad_gl
            .push_model_matrix(Mat4::from_scale_rotation_translation(
                vec3(ui.scale, ui.scale, 1.0),
                Quat::IDENTITY,
                vec3(o.x.round(), o.y.round(), 0.0),
            ));
        help::draw_pic(self.art.as_ref(), pic, r.w, r.h, t);
        let gl = unsafe { get_internal_gl() };
        gl.quad_gl.pop_model_matrix();
        ui.rect_lines(r, crate::ui::BORDER);
    }

    /// 대화창 초상화를 논리 사각형 `r`(44×44) 안에 찍는다. 원본 1px = 게임 픽셀 반(화면 배율의 절반, 정수).
    /// 배율이 홀수면 내림이라 조금 작아지지만 픽셀은 고르게 남는다. 그림이 없으면 false.
    /// 앨범 칸: 일러스트의 첫 화면(패닝 시작 자리)을 줄여서. 그림이 없으면 어두운 칸.
    pub fn draw_illust_thumb(&self, ui: &Ui, il: crate::illust::Illust, r: Rect) {
        use crate::illust::{VIEW_H, VIEW_W};
        let p = ui.px(vec2(r.x, r.y));
        let (w, h) = (r.w * ui.scale, r.h * ui.scale);
        let tex = self
            .art
            .as_ref()
            .and_then(|a| a.illust.get(il.index()))
            .and_then(|t| t.as_ref());
        match tex {
            Some(tex) => draw_texture_ex(
                tex,
                p.x,
                p.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    source: Some(Rect::new(0.0, 0.0, VIEW_W, VIEW_H)),
                    ..Default::default()
                },
            ),
            None => draw_rectangle(p.x, p.y, w, h, Color::new(0.08, 0.06, 0.10, 1.0)),
        }
    }

    /// 타이틀 바탕(docs/decisions/0083): 어둠 속 땅의 단면, 천천히 흐르는 안개 두 겹, 반딧불(굴 방·뿌리 사이),
    /// 물든 뿌리 끝에서 떠오르는 보라 알갱이, 메뉴 뒤 옅은 어둠. 그림이 없으면 아무것도 하지 않는다
    /// (월드 패스의 선 바탕이 남는다). `t`는 초(움직임만, 상태 없음).
    pub fn draw_title_art(&self, ui: &Ui, t: f32) {
        use crate::illust::{VIEW_H, VIEW_W};
        let Some(art) = self.art.as_ref() else {
            return;
        };
        let Some(tex) = art.title.as_ref() else {
            return;
        };
        let o = ui.px(vec2(0.0, 0.0));
        let k = ui.scale / 2.0;
        draw_texture_ex(
            tex,
            o.x,
            o.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIEW_W * k, VIEW_H * k)),
                ..Default::default()
            },
        );
        // 안개: 가로로 이어진 그림을 옆으로 굴린다(두 조각으로 그려 화면 밖으로 넘치지 않게). 게임 1px(원본 2px)씩.
        if let Some(fog) = art.title_fog.as_ref() {
            for (speed, alpha, flip) in [(5.0, 0.8, false), (9.0, 0.45, true)] {
                let off = ((t * speed / 2.0).floor() * 2.0).rem_euclid(VIEW_W);
                let c = Color::new(1.0, 1.0, 1.0, alpha);
                for (sx, sw, dx) in [(off, VIEW_W - off, 0.0), (0.0, off, VIEW_W - off)] {
                    if sw <= 0.0 {
                        continue;
                    }
                    draw_texture_ex(
                        fog,
                        o.x + dx * k,
                        o.y,
                        c,
                        DrawTextureParams {
                            dest_size: Some(vec2(sw * k, VIEW_H * k)),
                            source: Some(Rect::new(sx, 0.0, sw, VIEW_H)),
                            flip_y: flip,
                            ..Default::default()
                        },
                    );
                }
            }
        }
        // 원본 px(640×360) 좌표로 작은 네모 하나.
        let dot = |x: f32, y: f32, size: f32, c: Color| {
            draw_rectangle(
                (o.x + x * k).round(),
                (o.y + y * k).round(),
                (size * k).max(1.0).round(),
                (size * k).max(1.0).round(),
                c,
            );
        };
        // 반딧불: 자리마다 느린 8자 + 깜빡임. 둘은 굴 방 안, 하나는 굴길, 나머지는 뿌리 사이 어둠.
        const FLIES: [(f32, f32, f32); 7] = [
            (126.0, 256.0, 0.0),
            (176.0, 266.0, 1.7),
            (88.0, 176.0, 3.1),
            (330.0, 196.0, 4.4),
            (430.0, 250.0, 2.2),
            (600.0, 220.0, 5.3),
            (270.0, 292.0, 0.9),
        ];
        for (i, &(x, y, ph)) in FLIES.iter().enumerate() {
            if !keep_share(i, self.effects) {
                continue;
            }
            let s = 0.35 + 0.07 * i as f32;
            let px = x + (t * s + ph).sin() * 10.0;
            let py = y + (t * s * 2.0 + ph).sin() * 5.0;
            let a = 0.35 + 0.65 * (t * 2.3 + ph * 1.7).sin().max(0.0);
            dot(
                px - 2.0,
                py - 2.0,
                6.0,
                Color::new(1.0, 0.92, 0.55, 0.16 * a),
            );
            dot(px, py, 2.0, Color::new(1.0, 0.97, 0.75, a));
        }
        // 물든 뿌리 끝(오른쪽 아래)에서 보라 알갱이가 천천히 떠올라 사라진다.
        for i in 0..9 {
            if !keep_share(i, self.effects) {
                continue;
            }
            let f = i as f32;
            let period = 7.0 + (f * 1.37) % 3.0;
            let p = ((t + f * 2.9) / period).fract();
            let x = 470.0 + (f * 53.0) % 160.0 + (p * 6.0 + f).sin() * 6.0;
            let y = 352.0 - p * 120.0;
            let a = (p * 4.0).min(1.0) * (1.0 - p) * 0.8;
            dot(x, y, 2.0, Color::new(0.78, 0.6, 1.0, a));
        }
        // 메뉴 뒤 어둠은 타이틀 화면이 메뉴 줄 수에 맞춰 둥글게 깐다(`draw_soft_shade`, 0152).
        // 예전엔 여기서 크기가 정해진 네모 두 겹(166×98)을 늘 깔아, 줄이 넷뿐이면 아래가 비고 하늘 위에 모서리가 보였다.
    }

    /// 가장자리 없는 둥근 그늘(0152): 빛 얼룩 텍스처(가운데 진하고 바깥으로 부드럽게 0)를 어둡게 `r` 둘레에 깐다.
    /// 그림 위 글 뒤에 네모를 두면 UI 상자가 그림 위에 뜬 것처럼 보인다(사장 "메뉴 뒤쪽 음영 배경 때문에 거슬린다").
    pub fn draw_soft_shade(&self, ui: &Ui, r: Rect, alpha: f32) {
        let Some(art) = self.art.as_ref() else {
            ui.rect(r, Color::new(0.03, 0.02, 0.04, alpha * 0.6));
            return;
        };
        // 얼룩은 가장자리로 갈수록 옅어지니, 글 자리가 진한 안쪽에 들어오게 넉넉히 키운다.
        let (w, h) = (r.w * 1.6 + 24.0, r.h * 1.9 + 24.0);
        let c = r.center();
        let o = ui.px(vec2(c.x - w / 2.0, c.y - h / 2.0));
        draw_texture_ex(
            &art.light,
            o.x,
            o.y,
            Color::new(0.03, 0.02, 0.04, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(w * ui.scale, h * ui.scale)),
                ..Default::default()
            },
        );
    }

    /// 컷씬 일러스트를 게임 화면 전체에(docs/decisions/0039). 원본 1px = 화면 배율/2 px(초상화와 같은 2배 밀도).
    /// 홀수 배율에서는 원본 픽셀이 1.5배처럼 고르지 않게 늘어난다 — 그래서 그림은 2×2 점으로 만들고 1px 디더를 쓰지 않는다.
    /// `t`는 그림이 뜬 뒤 흐른 시간(패닝·입자).
    pub fn draw_illust(&self, ui: &Ui, il: crate::illust::Illust, t: f32) {
        use crate::illust::{self, VIEW_H, VIEW_W};
        let o = ui.px(vec2(0.0, 0.0));
        let k = ui.scale / 2.0;
        let (w, h) = (VIEW_W * k, VIEW_H * k);
        let spec = il.spec();
        let tex = self
            .art
            .as_ref()
            .and_then(|a| a.illust.get(il.index()))
            .and_then(|t| t.as_ref());
        match tex {
            Some(tex) => {
                let pan = illust::pan_at(&spec, t);
                draw_texture_ex(
                    tex,
                    o.x,
                    o.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(w, h)),
                        source: Some(Rect::new(pan.x, pan.y, VIEW_W, VIEW_H)),
                        ..Default::default()
                    },
                );
            }
            None => draw_rectangle(o.x, o.y, w, h, Color::new(0.08, 0.06, 0.10, 1.0)),
        }
        for (i, p) in illust::particles(spec.fx, t).into_iter().enumerate() {
            if !keep_share(i, self.effects) {
                continue;
            }
            draw_rectangle(
                (o.x + p.pos.x * k).round(),
                (o.y + p.pos.y * k).round(),
                (p.size * k).max(1.0).round(),
                (p.size * k).max(1.0).round(),
                p.color,
            );
        }
    }

    pub fn draw_portrait(&self, ui: &Ui, r: Rect, frame: u32) -> bool {
        let Some(tex) = self.art.as_ref().and_then(|a| a.portraits.as_ref()) else {
            return false;
        };
        let size = portrait_px(ui.scale);
        let p = ui.px(vec2(r.x, r.y));
        let (bw, bh) = (r.w * ui.scale, r.h * ui.scale);
        draw_texture_ex(
            tex,
            (p.x + (bw - size) / 2.0).round(),
            (p.y + (bh - size)).round(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                source: Some(art::inset(Rect::new(
                    (frame % art::PORTRAIT_COLS) as f32 * art::PORTRAIT,
                    (frame / art::PORTRAIT_COLS) as f32 * art::PORTRAIT,
                    art::PORTRAIT,
                    art::PORTRAIT,
                ))),
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
        // 내려가는 전환(0156): 단면이 화면을 덮는 동안은 HUD 대신 깊이 숫자만.
        if let Some(d) = w.descent() {
            if d.cover > 0.5 {
                let a = ((d.cover - 0.5) * 2.0).min(1.0);
                let m = d.depth_m.round() as u32;
                let s = crate::i18n::fill(crate::i18n::s().depth_m, &[&m]);
                let out = ui.with_outline();
                out.text_center(
                    &s,
                    160.0,
                    150.0,
                    crate::ui::FONT_PX,
                    Color::new(0.95, 0.88, 0.72, a),
                );
                return;
            }
        }
        let sheet = self
            .art
            .as_ref()
            .and_then(|a| a.sunny.as_ref())
            .map(|s| &s.sheet);
        // 왼쪽 위 바탕판(0091): 체력·반딧불·씨앗이 밝은 바닥 위에서 묻혔다. 오른쪽 끝은 가운데 이름표가 피한다(0253).
        let left_end = {
            let gx = 3.0 + w.player.stats.max_hp as f32 * hp_step(sheet) + 5.0;
            let sx = gx + 7.0 + ui.text_w(&w.inv.glowbugs.to_string(), crate::ui::FONT_PX) + 6.0;
            let tx = if sheet.is_some() { sx + 9.0 } else { sx + 5.0 };
            let end = tx + ui.text_w(&w.inv.seeds.to_string(), crate::ui::FONT_PX);
            hud_plate(ui, Rect::new(0.0, -1.0, end + 3.0, 13.0));
            end + 3.0
        };
        draw_hud_bars(ui, w, sheet);
        let out = ui.with_outline();
        // 반딧불: 체력 칸 오른쪽. 숫자는 언어 폰트(숫자는 모든 폰트에 있다). 글자 윗부분이 화면 밖으로 잘리지 않게 y ≥ 1.
        let x = 3.0 + w.player.stats.max_hp as f32 * hp_step(sheet) + 5.0;
        let pulse = 0.6 + 0.4 * (w.time * 3.0).sin().abs();
        // 반딧불 아이콘: 숨쉬는 빛 + 밝은 몸 + 머리 한 점(가운데 줄을 그으면 "="로 읽혔다).
        ui.rect(
            Rect::new(x - 1.0, 1.0, 5.0, 5.0),
            Color::new(0.85, 1.0, 0.45, 0.25 * pulse),
        );
        ui.rect(
            Rect::new(x, 2.0, 3.0, 3.0),
            Color::new(0.85, 1.0, 0.45, 0.5 + 0.5 * pulse),
        );
        ui.rect(Rect::new(x + 1.0, 1.0, 1.0, 1.0), OUTLINE);
        out.text(
            &w.inv.glowbugs.to_string(),
            x + 5.0,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        // 씨앗(상점 화폐): 반딧불 오른쪽. 팩이 있으면 새싹 아이콘(0077).
        let sx = x + 7.0 + ui.text_w(&w.inv.glowbugs.to_string(), crate::ui::FONT_PX) + 6.0;
        let tx = if let Some(sheet) = sheet {
            ui.sprite(
                sheet,
                crate::sunny::rect(crate::sunny::SPROUT),
                sx - 2.0,
                0.0,
                WHITE,
            );
            sx + 9.0
        } else {
            ui.rect(Rect::new(sx, 1.0, 3.0, 4.0), OUTLINE);
            ui.rect(Rect::new(sx + 0.5, 1.5, 2.0, 3.0), world::SEED);
            ui.rect(
                Rect::new(sx + 1.0, 0.0, 2.0, 1.0),
                Color::new(0.45, 0.78, 0.35, 1.0),
            );
            sx + 5.0
        };
        out.text(
            &w.inv.seeds.to_string(),
            tx,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        draw_quick_slots(ui, w);
        self.draw_biome_card(ui, w);
        self.draw_boss_card(ui, w);
        draw_exit_mark(ui, w);
        if let Some((text, time)) = w.juice.toast {
            let s = crate::i18n::tr(text);
            let a = (time / 0.4).min(1.0);
            // 긴 알림(물웅덩이 등)이 한 줄로 화면 폭을 넘어 오른쪽 위 HUD(게이지·마을 씨앗)와 겹쳤다(사장 피드백
            // 2026-10-26). 가운데 TOAST_W 안에서 줄을 바꾼다. 양옆 HUD 자리는 비운다.
            const TOAST_W: f32 = 196.0;
            let lines = crate::ui::wrap(ui, s, TOAST_W - 14.0, crate::ui::FONT_PX);
            let tw = lines
                .iter()
                .map(|l| ui.text_w(l, crate::ui::FONT_PX))
                .fold(0.0, f32::max)
                + 14.0;
            let line_h = crate::ui::FONT_PX + 2.0;
            // 마을은 오른쪽 위에 마을 씨앗·맡긴 반딧불 두 줄(y 14~39)이 있어 그 아래로.
            let top = if w.adv.place == crate::adventure::Place::Village {
                44.0
            } else {
                26.0
            };
            let r = Rect::new(
                (160.0 - tw / 2.0).round(),
                top,
                tw.round(),
                lines.len() as f32 * line_h + 4.0,
            );
            ui.rect(r, Color::new(0.07, 0.05, 0.09, 0.9 * a));
            ui.rect_lines(r, Color::new(0.90, 0.71, 0.47, a));
            for (i, l) in lines.iter().enumerate() {
                ui.text_center(
                    l,
                    160.0,
                    r.y + 3.0 + i as f32 * line_h,
                    crate::ui::FONT_PX,
                    Color::new(0.92, 0.86, 0.76, a),
                );
            }
        }
        draw_boss_bar(ui, w, left_end);
        draw_deep_bar(ui, w, left_end);
        draw_hud_screen(w, ui.scale, ui.ox, ui.oy, gamepad_count);
    }

    /// 도감 그림(대장): chief.png 물든 대장 줄의 `frame`칸을 `r`에 몸 자리 맞춤으로.
    pub fn draw_chief_icon(&self, ui: &Ui, r: Rect, frame: u32) {
        let Some(art) = self.art.as_ref() else {
            return;
        };
        let Some(c) = art.chief(frame, true, false) else {
            return;
        };
        draw_body_fit(ui, r, c.tex, c.src, c.size, c.body);
    }

    /// 컷씬 배우: 돌아온 보리.
    pub fn draw_bori_actor(&self, pos: Vec2, facing: Vec2, moving: bool) {
        if let Some(art) = &self.art {
            draw_bori_at(art, pos, facing.x < 0.0, moving, get_time() as f32);
        }
    }
}

/// 초상화(원본 88px, 대화창 칸 44 논리 px)를 화면에 찍을 크기(화면 px): 늘 칸에 꼭 맞게.
/// 배율이 짝수면 원본 1px = 화면 배율/2 px로 고르고, 홀수(3, 5…)면 2·3px(배율 5) 또는 1·2px(배율 3)이 번갈아 찍힌다.
/// 예전엔 홀수 배율에서 내림해 점을 고르게 두었는데(0015), 1920×1080 브라우저(배율 5)에서 칸의 80%만 차 둘레가 비어 보였다(사장, 0111).
/// 배율은 `present()`가 1 이상의 정수로 정한다(세로로 든 폰은 1: 원본 2점이 화면 1점).
fn portrait_px(scale: f32) -> f32 {
    art::PORTRAIT * 0.5 * scale.max(1.0).floor()
}

#[cfg(test)]
mod portrait_tests {
    use super::*;

    /// 눌린 그림자 타원(0150): 동서로 눕으면 가로로 길고, 남북으로 눕으면 폭이 줄지 않고 길이가 반이 된다.
    #[test]
    fn ground_shadow_keeps_width_north_south() {
        let (rx, ry, _) = ground_ellipse(vec2(1.0, 0.0), 8.0, 4.0);
        assert!((rx - 8.0).abs() < 1e-3 && (ry - 2.0).abs() < 1e-3);
        let (rx, ry, _) = ground_ellipse(vec2(0.0, 1.0), 8.0, 4.0);
        assert!(
            (rx - 4.0).abs() < 1e-3 && (ry - 4.0).abs() < 1e-3,
            "{rx} {ry}"
        );
    }

    #[test]
    fn portrait_fills_its_box() {
        for s in 1..=8 {
            let scale = s as f32;
            let px = portrait_px(scale);
            assert_eq!(px, 44.0 * scale, "배율 {s}: 칸을 꽉 채우지 않는다");
        }
        assert_eq!(portrait_px(1.0), 44.0);
        assert_eq!(portrait_px(4.0), 176.0);
    }
}

fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    (a + (b - a) * t).round()
}

impl Renderer {
    /// 지대 이름 카드(0156): 새 지대로 내려오면 화면 위쪽 가운데에 이름과 깊이가 떴다가 사라진다.
    fn draw_biome_card(&self, ui: &Ui, w: &World) {
        let Some((biome, m, time)) = w.adv.biome_card else {
            return;
        };
        if w.adv.fade.is_some() {
            return;
        }
        let name = crate::i18n::tr(biome.name());
        let depth = crate::i18n::fill(crate::i18n::s().depth_m, &[&m]);
        self.draw_card(ui, name, &depth, t::BIOME_CARD_TIME - time, time, 48.0);
    }

    /// 보스 이름 카드(0159): 컷씬이 띄운다(모모의 '???'). HUD와 컷씬 그리기 둘 다 부른다.
    pub fn draw_boss_card(&self, ui: &Ui, w: &World) {
        let Some((name, sub, left)) = w.juice.boss_card else {
            return;
        };
        // 카메라가 보스를 가운데에 두니 더 위에(건너뛰기 안내 아래).
        self.draw_card(ui, name, crate::i18n::tr(sub), 3.0 - left, left, 30.0);
    }

    /// 화면 위쪽 가운데의 이름 카드: 이름 + 한 줄. `shown`초 동안 보였고 `time`초 남았다. `y`는 이름 줄.
    fn draw_card(&self, ui: &Ui, name: &str, depth: &str, shown: f32, time: f32, y: f32) {
        let a = (shown / 0.6).min(time / 0.8).clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let out = ui.with_outline();
        let nw = ui.text_w(name, crate::ui::FONT_PX);
        let dw = ui.text_w(depth, crate::ui::FONT_PX);
        // 밝은 바닥에서도 읽히게 둥근 그늘(0152).
        let bw = nw.max(dw) + 16.0;
        self.draw_soft_shade(ui, Rect::new(160.0 - bw / 2.0, y - 2.0, bw, 28.0), 0.55 * a);
        // 이름 양옆의 가는 줄(흙 띠처럼), 나타날 때 바깥으로 뻗는다.
        let reach = 28.0 * (shown / 0.6).min(1.0);
        let lc = Color::new(0.90, 0.71, 0.47, 0.8 * a);
        let half = nw / 2.0 + 6.0;
        ui.rect(
            Rect::new((160.0 - half - reach).round(), y + 6.0, reach.round(), 1.0),
            lc,
        );
        ui.rect(
            Rect::new((160.0 + half).round(), y + 6.0, reach.round(), 1.0),
            lc,
        );
        out.text_center(
            name,
            160.0,
            y,
            crate::ui::FONT_PX,
            Color::new(1.0, 0.92, 0.72, a),
        );
        out.text_center(
            depth,
            160.0,
            y + 13.0,
            crate::ui::FONT_PX,
            Color::new(0.80, 0.74, 0.66, a),
        );
    }
}

// ---- 방 안의 "위"(0156): 땅속이라는 것이 방에서도 느껴지게 ---------------------

/// 도감 그림 한 칸: 칸(`size`) 안의 몸 자리 `body`가 논리 사각형 `r` 가운데에 오게, 화면 점 정수 배율로(게임 점의 2배까지).
/// 칸 전체로 맞추면 큰 칸(32·40)은 반 배율로 작아지고, 칸 아래쪽에 앉은 몸이 아래로 쏠렸다.
fn draw_body_fit(ui: &Ui, r: Rect, tex: &Texture2D, src: Rect, size: f32, body: Rect) {
    let fit = (r.w / body.w).min(r.h / body.h).min(2.0);
    let k = (fit * ui.scale).floor().max(1.0);
    let p = ui.px(vec2(r.x, r.y));
    let (cx, cy) = (p.x + r.w * ui.scale / 2.0, p.y + r.h * ui.scale / 2.0);
    let x = (cx - (body.x + body.w / 2.0) * k).round();
    let y = (cy - (body.y + body.h / 2.0) * k).round();
    draw_texture_ex(
        tex,
        x,
        y,
        WHITE,
        DrawTextureParams {
            source: Some(src),
            dest_size: Some(vec2(size, size) * k),
            ..Default::default()
        },
    );
}
