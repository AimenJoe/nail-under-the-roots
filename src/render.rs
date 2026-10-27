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
        let shake = w
            .map(|w| (w.feel.shake_offset * shake_mul).round())
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
                draw_grid_art(w, cam, &self.terrain[0].1, sheet);
            } else {
                draw_room_art(art, w);
            }
            self.draw_sun_shadows(art, w, alpha);
            draw_drill_trail(Some(art), &w.player, alpha);
            draw_telegraphs(w, alpha);
            if self.cut.is_some() {
                draw_mushrooms(art, w, cam);
            }
            draw_cast_shadows(w, alpha, cam, self.silhouette.is_some());
            draw_sorted(art, w, alpha, None);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_falling_rocks(w);
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
            draw_save_lights(w, cam, self.light.as_ref());
            draw_world_lights(
                w,
                cam,
                alpha,
                self.light.as_ref(),
                art.sunny.as_ref().map(|s| &s.sheet),
            );
            if draw_super_dim(w, cam) {
                draw_nail(art, &w.player, w.fur, alpha, w.time);
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
                draw_enemy(None, e, alpha, w.time);
            }
            draw_drill_trail(None, &w.player, alpha);
            draw_player(&w.player, alpha, w.time);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_falling_rocks(w);
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
        if w.feel.flash > 0.0 {
            // 벽 쿵 번쩍임. 흔들림 설정을 따른다(0이면 끈다: 눈이 예민한 사람).
            let k = (w.feel.flash / t::DRILL_WALL_FLASH).min(1.0) * shake_mul.min(1.0);
            draw_rectangle(0.0, 0.0, RW, RH, Color::new(1.0, 0.97, 0.9, 0.55 * k));
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
        // 왼쪽 위 바탕판(0091): 체력·반딧불·씨앗이 밝은 바닥 위에서 묻혔다.
        {
            let gx = 3.0 + w.player.stats.max_hp as f32 * hp_step(sheet) + 5.0;
            let sx = gx + 7.0 + ui.text_w(&w.glowbugs.to_string(), crate::ui::FONT_PX) + 6.0;
            let tx = if sheet.is_some() { sx + 9.0 } else { sx + 5.0 };
            let end = tx + ui.text_w(&w.seeds.to_string(), crate::ui::FONT_PX);
            hud_plate(ui, Rect::new(0.0, -1.0, end + 3.0, 13.0));
        }
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
            &w.glowbugs.to_string(),
            x + 5.0,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        // 씨앗(상점 화폐): 반딧불 오른쪽. 팩이 있으면 새싹 아이콘(0077).
        let sx = x + 7.0 + ui.text_w(&w.glowbugs.to_string(), crate::ui::FONT_PX) + 6.0;
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
            &w.seeds.to_string(),
            tx,
            1.0,
            crate::ui::FONT_PX,
            crate::ui::TEXT,
        );
        draw_quick_slots(ui, w);
        self.draw_biome_card(ui, w);
        if let Some((text, time)) = w.toast {
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
        draw_boss_bar(ui, w);
        draw_hud_screen(w, ui.scale, ui.ox, ui.oy, gamepad_count);
    }

    /// 도감 그림(대장): chief.png 줄 0의 `frame`칸을 `r`에 정수 배율로.
    pub fn draw_chief_icon(&self, ui: &Ui, r: Rect, frame: u32) {
        let Some(tex) = self.art.as_ref().and_then(|a| a.chief.as_ref()) else {
            return;
        };
        let k = ((r.w / art::CHIEF_CELL).floor() * ui.scale)
            .max(ui.scale * 0.5)
            .floor();
        let size = art::CHIEF_CELL * k.max(1.0);
        let p = ui.px(vec2(r.x, r.y));
        let (bw, bh) = (r.w * ui.scale, r.h * ui.scale);
        draw_texture_ex(
            tex,
            (p.x + (bw - size) / 2.0).round(),
            (p.y + (bh - size) / 2.0).round(),
            WHITE,
            DrawTextureParams {
                source: Some(art::chief_cell(frame, art::CHIEF_TAINTED)),
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
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

/// 보스 체력 막대(화면 위 가운데): 이름 + 막대. 깨어 있는 보스가 있을 때만.
/// 위에 둔다: 던전 방(192px)이 화면(180px)보다 커서 방 아래쪽에 서면 아래 막대가 Naïl과 보스를 가렸다(0046).
fn draw_boss_bar(ui: &Ui, w: &World) {
    let Some(b) = w.enemies.iter().find(|e| e.is_boss() && !e.asleep) else {
        return;
    };
    let name = crate::i18n::tr(b.sp().name);
    let bar = Rect::new(90.0, 4.0 + crate::ui::FONT_PX + 4.0, 140.0, 4.0);
    let nw = ui.text_w(name, crate::ui::FONT_PX) + 10.0;
    ui.rect(
        Rect::new(160.0 - nw / 2.0, 2.0, nw, crate::ui::FONT_PX + 3.0),
        Color::new(0.07, 0.05, 0.09, 0.75),
    );
    ui.text_center(name, 160.0, 3.0, crate::ui::FONT_PX, crate::ui::ACCENT);
    ui.rect(
        Rect::new(bar.x - 1.0, bar.y - 1.0, bar.w + 2.0, bar.h + 2.0),
        OUTLINE,
    );
    ui.rect(bar, Color::new(0.20, 0.10, 0.14, 1.0));
    let k = (b.hp.max(0) as f32 / b.max_hp.max(1) as f32).clamp(0.0, 1.0);
    ui.rect(
        Rect::new(bar.x, bar.y, (bar.w * k).round(), bar.h),
        Color::new(0.85, 0.30, 0.38, 1.0),
    );
    ui.rect(
        Rect::new(bar.x, bar.y, (bar.w * k).round(), 1.0),
        Color::new(1.0, 0.6, 0.62, 1.0),
    );
}

/// 퀵 슬롯(왼쪽 아래): 산딸기·지렁이 두 칸, 고른 칸은 테두리 강조, 개수, 먹기 키 안내.
/// HUD 바탕판(0091): 풀밭·흙 위에서도 HUD가 묻히지 않게 반투명 어두운 판 + 어두운 테두리.
fn hud_plate(ui: &Ui, r: Rect) {
    ui.rect(r, Color::new(0.06, 0.04, 0.08, 0.62));
    ui.rect_lines(r, Color::new(0.02, 0.01, 0.03, 0.75));
}

/// 퀵 슬롯(0091): 칸 22px·아이콘 2배, 고른 칸은 두꺼운 금빛 테두리, 안 고른 칸은 흐리게.
/// 저절로 바뀐 칸(`Bag::flash`)은 하얗게 깜빡인다. 수는 테두리 글자로 칸 오른쪽 아래.
fn draw_quick_slots(ui: &Ui, w: &World) {
    use crate::items::Consumable;
    let (x0, y0, size, gap) = (5.0 + deny_shake(w.deny[1]), 153.0, 22.0, 3.0);
    let hint = crate::i18n::fill(
        crate::i18n::s().quick_hint,
        &[&crate::ui::key(crate::input::prompt(
            crate::input::Act::UseItem,
        ))],
    );
    let hx = x0 + 2.0 * (size + gap) + 2.0;
    let hw = ui.text_w(&hint, crate::ui::FONT_PX);
    hud_plate(
        ui,
        Rect::new(x0 - 3.0, y0 - 3.0, hx + hw + 4.0 - (x0 - 3.0), size + 6.0),
    );
    let out = ui.with_outline();
    let flash_on = w.bag.flash > 0.0 && ((w.bag.flash * 8.0) as i32) % 2 == 0;
    for (i, c) in Consumable::ALL.iter().enumerate() {
        let r = Rect::new(x0 + i as f32 * (size + gap), y0, size, size);
        let n = w.bag.count(*c);
        let selected = w.bag.selected == *c;
        ui.rect(
            r,
            if selected {
                Color::new(0.20, 0.14, 0.17, 0.95)
            } else {
                Color::new(0.07, 0.05, 0.09, 0.85)
            },
        );
        let border = if selected && w.deny[1] > 0.0 {
            deny_color(w.deny[1])
        } else if selected && flash_on {
            WHITE
        } else if selected {
            crate::ui::ACCENT
        } else {
            Color::new(0.55, 0.40, 0.28, 0.6)
        };
        ui.rect_lines(r, border);
        if selected {
            ui.rect_lines(
                Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0),
                border,
            );
        }
        let dim = if n == 0 {
            0.3
        } else if selected {
            1.0
        } else {
            0.65
        };
        let center = ui.px(vec2(r.x + size / 2.0 - 2.0, r.y + size / 2.0 - 1.0));
        let k = ui.scale * 2.0;
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
                    draw_worm(center, w.time * 6.0, k * 0.75);
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
        // 다음 먹기까지: 위에서부터 어둡게 덮였다가 걷힌다(0038).
        let cd = w.player.item_cd / (t::ITEM_COOLDOWN * w.player.eat_cd_mul);
        if cd > 0.0 {
            let h = (size * cd).round();
            ui.rect(
                Rect::new(r.x, r.y, size, h),
                Color::new(0.0, 0.0, 0.0, 0.55),
            );
        }
        let ns = n.to_string();
        out.text(
            &ns,
            r.right() - ui.text_w(&ns, crate::ui::FONT_PX) - 2.0,
            r.bottom() - 13.0,
            crate::ui::FONT_PX,
            if n == 0 {
                Color::new(0.85, 0.45, 0.45, 1.0)
            } else {
                crate::ui::TEXT
            },
        );
    }
    out.text(&hint, hx, y0 + 5.0, crate::ui::FONT_PX, crate::ui::TEXT);
    // 못 먹었다(0149·0150): 고른 칸 위에 ✕.
    if w.deny[1] > 0.0 {
        let k = Consumable::ALL
            .iter()
            .position(|c| *c == w.bag.selected)
            .unwrap_or(0) as f32;
        deny_cross(
            ui,
            vec2(x0 + k * (size + gap) + size / 2.0, y0 + size / 2.0),
            4.0,
        );
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
    let faint = Color::new(1.0, 1.0, 1.0, 0.15);
    for b in &t.buttons {
        draw_circle(b.center.x, b.center.y, b.radius, fill);
        // 기다리는 중이면 테두리가 시계 방향으로 다시 차오른다(굴파기·필살기·아이템, 0038).
        let ready = b.ready.clamp(0.0, 1.0);
        if ready < 1.0 {
            draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, faint);
            draw_arc(
                b.center.x,
                b.center.y,
                48,
                b.radius,
                -90.0,
                2.0,
                360.0 * ready,
                line,
            );
        } else {
            draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, line);
        }
        let line = if ready < 1.0 { faint } else { line };
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
    if let Some((item, k)) = p.eat_progress() {
        draw_eating(pos, p.face_left, item, k);
    }
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
                // 자국이 지나가는 쪽 = 친 발(0082): 짝수 타는 위에서 아래로, 홀수 타는 아래에서 위로, 마지막 타는 둘 다(X).
                let mark = p.moveset.mark;
                if finisher {
                    draw_claw_marks(origin, dir, hit.range, k, true, mark, 1.0);
                    draw_claw_marks(origin, dir, hit.range, k, true, mark, -1.0);
                } else {
                    let sweep = if idx % 2 == 0 { 1.0 } else { -1.0 };
                    draw_claw_marks(origin, dir, hit.range, k, false, mark, sweep);
                }
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
            // 겨누기(0139): 나갈 쪽으로 점선, 겨눈 적에는 모서리 표시. 8방향 입력 사이의 적도 맞춘다는 것을 보여 준다.
            let guide = Color::new(rage.r, rage.g, rage.b, 0.35 + 0.45 * k);
            let len = p
                .drill_lock
                .map_or(72.0, |q| ((q - pos).length() - 8.0).clamp(14.0, 240.0));
            let mut d = 14.0;
            while d < len {
                let q = (pos + dir * d).round();
                draw_rectangle(q.x, q.y, 1.0, 1.0, guide);
                d += 6.0;
            }
            if let Some(q) = p.drill_lock {
                let q = q.round();
                let r = 9.0 - 2.0 * k;
                for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let c = vec2(q.x + sx * r, q.y + sy * r);
                    // 모서리 점에서 안쪽으로 3px씩(그 점 포함 4px).
                    let x0 = if sx < 0.0 { c.x } else { c.x - 3.0 };
                    let y0 = if sy < 0.0 { c.y } else { c.y - 3.0 };
                    draw_rectangle(x0, c.y, 4.0, 1.0, guide);
                    draw_rectangle(c.x, y0, 1.0, 4.0, guide);
                }
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
            let r = 6.0 + (p.talents.emerge_radius() - 6.0) * p01;
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
/// `sweep` 1 = 호의 한쪽 끝에서, −1 = 반대쪽 끝에서 펼친다(번갈아 치는 발).
fn draw_claw_marks(origin: Vec2, dir: Vec2, r: f32, k: f32, strong: bool, mark: Color, sweep: f32) {
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
        let a_start = base - span * sweep;
        let a_end = a_start + 2.0 * span * reveal * sweep;
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
            source: Some(art::inset(src)),
            dest_size: Some(vec2(src.w.round(), src.h.round())),
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
            // 구운 지도와 같은 자리(무리 밀도·어긋남, terrain.rs).
            if let Some((d, ox, oy)) = crate::terrain::decor_at(w.biome, cx, cy) {
                out.push((cx as f32 * 16.0 + ox, cy as f32 * 16.0 + oy, d));
            }
        }
    }
    out
}

/// 잠긴 문: 문을 막고 보라 실 두 가닥이 감는다(이야기의 "실", 0037).
/// 팩이 있으면 팩의 나무 울타리(남북 문은 가로 울타리, 동서 문은 세로 울타리, 0097), 없으면 물든 뿌리 세 줄기(0094).
/// 잠그면 문이 바닥에서 솟고(뿌리는 문틀에서 자라고, `grow` 0→1) 실이 나중에 감긴다. 열면 거꾸로 걷히고 월드가 풀리는 실을 낸다.
/// 무늬는 월드 좌표로 정해 두 칸짜리 문이 이어져 보인다.
/// `glow`면 더하기 빛 패스용: 문은 빼고 실과 문틈 빛만(어두운 심층에서도 문이 보이게).
fn draw_root_gate(
    g: &crate::map::Grid,
    cx: i32,
    cy: i32,
    grow: f32,
    time: f32,
    glow: bool,
    sheet: Option<&Texture2D>,
) {
    let blocked = |c: Cell| matches!(c, Cell::Wall | Cell::Door);
    // 좌우가 막혀 있으면 위아래로 지나는 문(남북) → 세로 뿌리.
    let vertical = blocked(g.get_i(cx - 1, cy)) && blocked(g.get_i(cx + 1, cy));
    let (x0, y0) = (cx as f32 * 16.0, cy as f32 * 16.0);
    let root_dark = Color::new(0.30, 0.20, 0.16, 1.0);
    let root = Color::new(0.45, 0.31, 0.22, 1.0);
    let root_hi = Color::new(0.62, 0.46, 0.32, 1.0);
    let th = world::THREAD;
    let pulse = 0.82 + 0.18 * (time * 3.0).sin();
    // (along, across) → 월드 px. along = 뿌리가 뻗는 방향, across = 가로지르는 방향.
    let at = |along: f32, across: f32| -> Vec2 {
        if vertical {
            vec2(x0 + across, y0 + along)
        } else {
            vec2(x0 + along, y0 + across)
        }
    };
    let px = |q: Vec2, c: Color| draw_rectangle(q.x.round(), q.y.round(), 1.0, 1.0, c);
    let (base_along, base_across) = if vertical { (y0, x0) } else { (x0, y0) };
    if glow {
        let c = at(8.0, 8.0);
        draw_circle(
            c.x,
            c.y,
            11.0,
            Color::new(th.r, th.g, th.b, 0.10 * grow * pulse),
        );
    }
    // 문틈 뒤 보라 아지랑이(잠긴 동안 숨 쉰다). 가는 울타리 뒤에서는 네모로 보여 뿌리일 때만.
    if !glow && sheet.is_none() {
        draw_rectangle(
            x0,
            y0,
            16.0,
            16.0,
            Color::new(th.r * 0.5, th.g * 0.3, th.b * 0.6, 0.22 * grow * pulse),
        );
    }
    if let Some(sheet) = sheet {
        // 팩 울타리: 앞쪽 절반 동안 바닥에서 솟는다(윗줄부터 보인다).
        if !glow {
            let rise = (grow / 0.55).clamp(0.0, 1.0);
            let h = (rise * 16.0).round();
            if h > 0.0 {
                // 울타리 두 겹을 엇갈려 쌓는다(뒤 겹부터). 가로는 위아래로, 세로는 좌우로 민다.
                let r = sunny::rect(if vertical {
                    sunny::FENCE_H
                } else {
                    sunny::FENCE_V
                });
                for d in sunny::FENCE_STACK {
                    let (dx, dy) = if vertical { (0.0, d) } else { (d, 0.0) };
                    draw_texture_ex(
                        sheet,
                        x0 + dx,
                        y0 + 16.0 - h + dy,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(r.x, r.y, r.w, h)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    } else {
        let reach = (grow * 16.0).ceil() as i32;
        // 뿌리 세 줄기, 몸 3px(밝은 쪽·몸·그늘) + 외곽선. 빛 패스에서는 밝은 쪽에 흐린 빛만 더한다.
        for lane in [2.0_f32, 7.0, 12.0] {
            for r in 0..reach.min(16) {
                let wa = base_along + r as f32;
                let wob = ((wa * 0.45 + (base_across + lane) * 0.3).sin() * 1.2).round();
                let c = lane + wob;
                let rf = r as f32;
                if glow {
                    px(at(rf, c), Color::new(0.55, 0.35, 0.25, 0.18 * grow));
                    continue;
                }
                px(at(rf, c - 1.0), OUTLINE);
                px(at(rf, c), if (wa as i32) % 3 == 0 { root_hi } else { root });
                px(at(rf, c + 1.0), root);
                px(at(rf, c + 2.0), root_dark);
                px(at(rf, c + 3.0), OUTLINE);
            }
        }
    }
    // 실: 문(뿌리)이 거의 다 선 뒤 감긴다. 실은 문을 가로지르는 방향(across)으로 간다.
    let ta = ((grow - 0.55) / 0.45).clamp(0.0, 1.0);
    if ta > 0.0 {
        // 뿌리: 두 가닥이 따로 물결친다. 울타리: 한 가닥이 가로대를 사선으로 감는다(앞을 지나는 반만 보인다).
        let strands: &[(f32, f32, f32)] = if sheet.is_some() {
            let c = if vertical {
                sunny::FENCE_RAIL_Y
            } else {
                sunny::FENCE_RAIL_X
            };
            &[(c, sunny::FENCE_WRAP, 0.0)]
        } else {
            &[(4.0, 1.1, 4.0), (11.0, 1.1, 11.0)]
        };
        let fence = sheet.is_some();
        let freq = if fence { 0.6 } else { 0.55 };
        for &(lane, amp, phase) in strands {
            for k in 0..16 {
                let wc = base_across + k as f32;
                let ph = wc * freq + time * 2.4 + phase;
                // 울타리를 감는 실은 뒤로 도는 반(cos < 0)이 가로대에 가려진다.
                if fence && ph.cos() < 0.0 {
                    continue;
                }
                let a = lane + (ph.sin() * amp).round();
                let kf = k as f32;
                if glow {
                    px(at(a, kf), Color::new(th.r, th.g, th.b, 0.55 * ta * pulse));
                    px(
                        at(a + 1.0, kf),
                        Color::new(th.r, th.g, th.b, 0.25 * ta * pulse),
                    );
                } else {
                    px(at(a + 1.0, kf), Color::new(0.34, 0.20, 0.46, ta));
                    px(at(a, kf), Color::new(th.r * pulse, th.g * pulse, th.b, ta));
                }
            }
        }
    }
}

/// 격자 방(필드·던전) 그림: 구운 지도(terrain.rs, 0062)의 보이는 부분 + 매 프레임 바뀌는 칸.
/// 특별한 칸: 문(열림 = 바닥, 잠김 = 나무 울타리(팩 없으면 물든 뿌리) + 보라 실, 0094·0097), 굴 입구, 출구, 저장 지점(빛나는 버섯 고리).
fn draw_grid_art(w: &World, cam: Vec2, baked: &Texture2D, sheet: Option<&Texture2D>) {
    let Some(g) = &w.room.grid else { return };
    let view = Rect::new(cam.x.floor() - 1.0, cam.y.floor() - 1.0, RW + 3.0, RH + 3.0);
    let bounds = Rect::new(0.0, 0.0, baked.width(), baked.height());
    if let Some(src) = view.intersect(bounds) {
        draw_texture_ex(
            baked,
            src.x,
            src.y,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                ..Default::default()
            },
        );
    }
    let (x0, y0) = (
        (cam.x / 16.0).floor() as i32 - 1,
        (cam.y / 16.0).floor() as i32 - 1,
    );
    let (x1, y1) = (x0 + 23, y0 + 14);
    // 특별한 칸
    for cy in y0..y1 {
        for cx in x0..x1 {
            let c = g.get_i(cx, cy);
            let p = vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0);
            match c {
                Cell::Door if w.room.door_anim > 0.0 => {
                    draw_root_gate(g, cx, cy, w.room.door_anim, w.time, false, sheet);
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
                Cell::Gate => {
                    // 뿌리 길: 뿌리가 감싼 큰 구멍 + 팻말.
                    let root = Color::new(0.42, 0.30, 0.24, 1.0);
                    draw_ellipse(p.x, p.y + 2.0, 15.0, 8.0, 0.0, OUTLINE);
                    draw_ellipse(p.x, p.y + 2.0, 13.0, 6.5, 0.0, world::DIRT);
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        10.0,
                        4.5,
                        0.0,
                        Color::new(0.03, 0.02, 0.04, 1.0),
                    );
                    for k in 0..4 {
                        let a = std::f32::consts::PI * (0.1 + k as f32 * 0.27);
                        let q = p + vec2(a.cos() * 13.0, 2.0 - a.sin() * 6.5);
                        draw_line(q.x, q.y, q.x + a.cos() * 4.0, q.y - 3.0, 2.0, root);
                    }
                    let pulse = 0.4 + 0.3 * (w.time * 2.0).sin().abs();
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        4.0,
                        1.5,
                        0.0,
                        Color::new(0.7, 0.9, 1.0, pulse * 0.4),
                    );
                    // 팻말(왼쪽 위)
                    let s = p + vec2(-17.0, -10.0);
                    draw_rectangle(s.x, s.y + 4.0, 1.0, 9.0, root);
                    draw_rectangle(s.x - 4.0, s.y - 1.0, 10.0, 6.0, OUTLINE);
                    draw_rectangle(s.x - 3.0, s.y, 8.0, 4.0, Color::new(0.66, 0.50, 0.32, 1.0));
                    draw_triangle(
                        vec2(s.x + 6.0, s.y - 1.0),
                        vec2(s.x + 6.0, s.y + 5.0),
                        vec2(s.x + 9.0, s.y + 2.0),
                        OUTLINE,
                    );
                }
                Cell::Save => draw_save_ring(p, w.time),
                _ => {}
            }
        }
    }
}

/// 빛 더하기 재질(0076). 섞기만 다르고 셰이더는 macroquad 기본과 같다.
fn light_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec2 uv;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
    const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;
    uniform sampler2D Texture;
    void main() {
        gl_FragColor = color * texture2D(Texture, uv);
    }"#;
    use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
    load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .ok()
}

/// 저장 지점: 빛나는 버섯 고리(버섯만, 빛은 `draw_save_lights`). 고리 자국은 짙게(실제 균환처럼).
fn draw_save_ring(p: Vec2, time: f32) {
    let pulse = 0.5 + 0.5 * (time * 2.5).sin();
    let cap = Color::new(1.0, 0.97, 0.80, 1.0);
    let cap_lit = Color::new(0.80, 1.0, 0.62, 1.0);
    let stem = Color::new(0.95, 0.88, 0.70, 1.0);
    draw_ellipse_lines(
        p.x,
        p.y,
        11.0,
        6.0,
        0.0,
        2.0,
        Color::new(0.05, 0.12, 0.06, 0.30),
    );
    // 버섯 일곱: 뒤쪽(위)부터 그려 앞 버섯이 겹친다.
    let mut caps: Vec<Vec2> = (0..7)
        .map(|k| {
            let t = k as f32 / 7.0 * std::f32::consts::TAU + 0.3;
            (p + vec2(t.cos() * 11.0, t.sin() * 6.0)).round()
        })
        .collect();
    caps.sort_by(|u, v| u.y.total_cmp(&v.y));
    for q in caps {
        // 둥근 갓(위 3칸, 아래 5칸) + 짧은 대.
        draw_rectangle(q.x - 1.0, q.y - 5.0, 3.0, 1.0, OUTLINE);
        draw_rectangle(q.x - 2.0, q.y - 4.0, 5.0, 1.0, OUTLINE);
        draw_rectangle(q.x - 3.0, q.y - 3.0, 7.0, 3.0, OUTLINE);
        draw_rectangle(q.x - 1.0, q.y - 1.0, 3.0, 3.0, OUTLINE);
        draw_rectangle(q.x - 1.0, q.y - 4.0, 3.0, 1.0, cap);
        draw_rectangle(q.x - 2.0, q.y - 3.0, 5.0, 1.0, cap);
        let lit = Color::new(cap_lit.r, cap_lit.g, cap_lit.b, 0.6 + 0.4 * pulse);
        draw_rectangle(q.x - 2.0, q.y - 2.0, 5.0, 1.0, lit);
        draw_rectangle(q.x - 1.0, q.y - 4.0, 1.0, 1.0, WHITE);
        draw_rectangle(q.x, q.y - 1.0, 1.0, 2.0, stem);
    }
}

/// 저장 지점의 빛(0076). 사람 피드백 "저장 포인트가 너무 눈에 띄지 않는다": 초록 풀밭에 반투명 초록·금빛을
/// 섞으면 탁해져 묻힌다 → 빛을 더해(밝기 대비) 숨쉬는 후광 + 위로 뜨는 포자(움직임). 아직 저장하지 않은
/// 고리는 빛기둥이 높이 서서 부른다. 어두운 굴에서도 보이게 어둠 위에 그린다.
fn draw_save_lights(w: &World, cam: Vec2, light: Option<&Material>) {
    let Some(g) = &w.room.grid else { return };
    let (x0, y0) = (
        (cam.x / 16.0).floor() as i32 - 2,
        (cam.y / 16.0).floor() as i32 - 2,
    );
    let rings: Vec<Vec2> = (y0..y0 + 16)
        .flat_map(|cy| (x0..x0 + 25).map(move |cx| (cx, cy)))
        .filter(|&(cx, cy)| g.get_i(cx, cy) == Cell::Save)
        .map(|(cx, cy)| vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0))
        .collect();
    if rings.is_empty() {
        return;
    }
    if let Some(m) = light {
        gl_use_material(m);
    }
    // 더하기면 바탕이 밝아지고, 보통 섞기(재질 없음)면 덮인다 → 그때는 옅게.
    let k = if light.is_some() { 1.0 } else { 0.6 };
    let pulse = 0.5 + 0.5 * (w.time * 2.5).sin();
    let a = |r: f32, g: f32, b: f32, al: f32| Color::new(r, g, b, al * k);
    for p in rings {
        let active = (w.adv.save_pos - p).length() < 1.0;
        draw_ellipse(
            p.x,
            p.y,
            16.0,
            9.0,
            0.0,
            a(1.0, 0.85, 0.45, 0.10 + 0.08 * pulse),
        );
        draw_ellipse(
            p.x,
            p.y,
            10.0,
            5.5,
            0.0,
            a(1.0, 0.92, 0.60, 0.14 + 0.08 * pulse),
        );
        draw_ellipse(
            p.x,
            p.y,
            5.0,
            2.5,
            0.0,
            a(1.0, 1.0, 0.80, 0.22 + 0.10 * pulse),
        );
        let rise = if active { 3 } else { 8 };
        for i in 0..rise {
            let f = (rise - i) as f32 / rise as f32;
            draw_rectangle(
                p.x - 2.0,
                p.y - 4.0 - i as f32 * 4.0,
                4.0,
                4.0,
                a(1.0, 0.95, 0.70, 0.22 * f * (0.6 + 0.4 * pulse)),
            );
        }
        // 위로 떠오르는 포자 다섯.
        for j in 0..5 {
            let ph = (w.time * 0.45 + j as f32 / 5.0).fract();
            let x = p.x + (w.time * 1.7 + j as f32 * 2.1).sin() * 8.0;
            let y = p.y - 2.0 - ph * 30.0;
            let s = if ph < 0.5 { 2.0 } else { 1.0 };
            draw_rectangle(
                x.round(),
                y.round(),
                s,
                s,
                a(1.0, 0.95, 0.65, 1.0 - ph * 0.8),
            );
        }
    }
    if light.is_some() {
        gl_use_default_material();
    }
}

/// 0..1 해시(환경 입자 자리, 0081). 같은 번호면 같은 값이라 상태 없이 그린다.
/// 입자 효과 설정(0141): 번호 `i`의 입자를 그릴까. `share`(0..=1) 비율만큼 고르게 고른다(0.5면 둘에 하나, 1이면 모두).
fn keep_share(i: usize, share: f32) -> bool {
    ((i + 1) as f32 * share).floor() > (i as f32 * share).floor()
}

fn hash01(i: u32, k: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// 환경 입자(0081): 들판·마을은 떠다니는 꽃가루, 뿌리 지대는 천천히 내리는 포자, 심층은 빛 알갱이(더하기 빛 패스).
/// 상태가 없다. 번호마다 월드에 붙은 자리(카메라 둘레 타일에서 되풀이)와 흐름이 정해져 있다.
/// 분위기 입자는 작고 대비가 낮게(타격 효과보다 눈에 덜 띄게).
fn draw_ambient(w: &World, cam: Vec2, glow: bool) {
    let deep = w.biome == Biome::Deep;
    if glow != deep || w.room.grid.is_none() {
        return;
    }
    let (tw, th) = (RW + 48.0, RH + 48.0);
    let n = if deep { 18 } else { 22 };
    for i in 0..n {
        // 입자 효과 설정(0141): 그 비율만.
        if !keep_share(i as usize, w.effects) {
            continue;
        }
        let (hx, hy, hs) = (hash01(i, 1), hash01(i, 2), hash01(i, 3));
        let t = w.time;
        let (drift, color, size) = match w.biome {
            // 꽃가루: 바람 따라 옆으로, 위아래로 살랑.
            Biome::Meadow => (
                vec2(6.0 + hs * 6.0, (t * (0.8 + hs) + hs * 6.0).sin() * 3.0),
                Color::new(1.0, 0.97, 0.85, 0.55),
                1.0,
            ),
            // 포자: 천천히 내리며 좌우로 흔들.
            Biome::Roots => (
                vec2((t * (0.6 + hs) + hs * 9.0).sin() * 4.0, 5.0 + hs * 4.0),
                Color::new(0.90, 0.85, 0.72, 0.45),
                1.0,
            ),
            // 빛 알갱이: 아주 천천히 떠오른다.
            Biome::Deep => (
                vec2((t * 0.5 + hs * 7.0).sin() * 5.0, -3.0 - hs * 3.0),
                Color::new(
                    0.55,
                    0.80,
                    1.0,
                    0.35 + 0.25 * (t * 2.0 + hs * 9.0).sin().abs(),
                ),
                if hs > 0.7 { 2.0 } else { 1.0 },
            ),
        };
        // 꽃가루는 옆으로 흘러가고, 포자·빛 알갱이는 위아래로 흐른다(나머지 축은 살랑임).
        let flow = if w.biome == Biome::Meadow {
            vec2(drift.x * t, drift.y)
        } else {
            vec2(drift.x, drift.y * t)
        };
        let x = cam.x - 24.0 + (hx * tw + flow.x - cam.x).rem_euclid(tw);
        let y = cam.y - 24.0 + (hy * th + flow.y - cam.y).rem_euclid(th);
        if deep {
            draw_circle(
                x.round(),
                y.round(),
                3.0,
                Color::new(color.r, color.g, color.b, color.a * 0.3),
            );
        }
        draw_rectangle(x.round(), y.round(), size, size, color);
    }
}

/// 더하기 빛(0081): 반딧불(떨어진 것)·모닥불(바닥 빛 + 불티)·빛 버섯·심층 빛 알갱이. 어둠 위에 그린다.
fn draw_world_lights(
    w: &World,
    cam: Vec2,
    alpha: f32,
    light: Option<&Material>,
    sheet: Option<&Texture2D>,
) {
    if w.room.grid.is_none() {
        return;
    }
    if let Some(m) = light {
        gl_use_material(m);
    }
    let k = if light.is_some() { 1.0 } else { 0.5 };
    let t = w.time;
    draw_light_shafts(w, cam, k);
    for l in &w.loot {
        if l.kind != loot::LootKind::Glowbug {
            continue;
        }
        let q = lerp(l.prev_pos, l.pos, alpha);
        let pulse = 0.55 + 0.45 * ((t * 6.0 + l.phase * 10.0) * 0.8).sin().abs();
        let g = world::GLOW;
        draw_circle(
            q.x.round(),
            (q.y - l.z - 3.0).round(),
            6.0,
            Color::new(g.r, g.g, g.b, 0.16 * pulse * k),
        );
    }
    // 지대 장애물(0149): 어둠이 덮은 뒤에도 보이게. 덤불은 붉은 기운, 떨어질 돌 자리는 붉은 고리가 조여 든다.
    for c in &w.room.thorns {
        let pulse = 0.5 + 0.5 * (t * 3.0 + c.x * 0.1).sin();
        draw_circle(
            c.x.round(),
            c.y.round(),
            8.0,
            Color::new(0.85, 0.15, 0.25, (0.10 + 0.08 * pulse) * k),
        );
    }
    for r in &w.rocks {
        let q = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        let rad = t::ROCK_R * (1.5 - 0.5 * q);
        draw_circle_lines(
            r.pos.x.round(),
            r.pos.y.round(),
            rad,
            1.0,
            Color::new(1.0, 0.35, 0.30, (0.25 + 0.5 * q) * k),
        );
        draw_circle(
            r.pos.x.round(),
            r.pos.y.round(),
            t::ROCK_R * q,
            Color::new(0.9, 0.2, 0.15, 0.12 * q * k),
        );
    }
    // 포자 구름(0146)은 어둠 속에서도 옅게 빛난다(아플 때만 조금 더).
    for c in &w.clouds {
        let fade = 1.0 - ((c.time / t::SPORE_CLOUD_TIME - 0.75) / 0.25).clamp(0.0, 1.0);
        let hot = if c.time < t::SPORE_CLOUD_GRACE {
            0.5
        } else {
            1.0
        };
        draw_circle(
            c.pos.x.round(),
            c.pos.y.round(),
            t::SPORE_CLOUD_R,
            Color::new(0.55, 0.45, 0.80, 0.12 * fade * hot * k),
        );
    }
    for p in &w.room.props {
        if p.kind != PropKind::Campfire {
            continue;
        }
        let c = vec2(p.rect.center().x, p.rect.bottom() - 6.0);
        let flick = 0.5 + 0.25 * (t * 9.0).sin() + 0.25 * (t * 13.7).sin();
        // 바닥 빛: 겹을 여럿 두어 가장자리가 스티커처럼 끊기지 않게(바깥일수록 옅게).
        for (i, (rx, ry, a)) in [
            (40.0, 22.0, 0.03),
            (30.0, 16.0, 0.04),
            (20.0, 11.0, 0.05),
            (11.0, 6.0, 0.06),
        ]
        .into_iter()
        .enumerate()
        {
            let warm = 0.55 + i as f32 * 0.06;
            draw_ellipse(
                c.x,
                c.y + 2.0,
                rx + flick,
                ry + flick * 0.5,
                0.0,
                Color::new(1.0, warm, 0.25 + i as f32 * 0.04, (a + 0.02 * flick) * k),
            );
        }
        // 불티: 넷이 위로 떠오르다 사라진다.
        for i in 0..4u32 {
            let ph = (t * 0.9 + hash01(i, 7)).fract();
            let x = c.x + (t * 3.0 + i as f32 * 1.7).sin() * 3.0 + (hash01(i, 8) - 0.5) * 6.0;
            let y = c.y - 6.0 - ph * 22.0;
            draw_rectangle(
                x.round(),
                y.round(),
                1.0,
                1.0,
                Color::new(1.0, 0.75, 0.35, (1.0 - ph) * k),
            );
        }
    }
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.6 + 0.4 * ((t * 2.0 + x * 0.1).sin());
            // 동심원 세 겹으로 가장자리를 풀어 은은하게(0100).
            for r in [16.0, 11.0, 6.0] {
                draw_circle(
                    x + 8.0,
                    y + 10.0,
                    r,
                    Color::new(0.35, 0.65, 1.0, 0.035 * pulse * k),
                );
            }
        }
    }
    // 잠긴 문의 실과 빛(0094): 어둠 마스크 위에 더해 그려 어두운 심층에서도 막힌 문이 보인다.
    if w.room.door_anim > 0.0 {
        if let Some(g) = &w.room.grid {
            let (x0, y0) = (
                (cam.x / 16.0).floor() as i32 - 1,
                (cam.y / 16.0).floor() as i32 - 1,
            );
            for cy in y0..y0 + 14 {
                for cx in x0..x0 + 23 {
                    if g.get_i(cx, cy) == Cell::Door {
                        draw_root_gate(g, cx, cy, w.room.door_anim, t, true, sheet);
                    }
                }
            }
        }
    }
    draw_ambient(w, cam, true);
    if light.is_some() {
        gl_use_default_material();
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

/// 돌아온 보리(chief.png 줄 1, 32×32). 발이 `pos.y + 6` 근처.
pub fn draw_bori_at(art: &Art, pos: Vec2, face_left: bool, moving: bool, time: f32) {
    let Some(tex) = art.chief.as_ref() else {
        draw_circle(pos.x, pos.y, 10.0, Color::new(0.62, 0.45, 0.36, 1.0));
        return;
    };
    let frame = if moving {
        2 + ((time * 6.0) as u32) % 2
    } else {
        ((time * 1.5) as u32) % 2
    };
    draw_texture_ex(
        tex,
        (pos.x - 16.0).round(),
        (pos.y - 24.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(art::chief_cell(frame, art::CHIEF_BORI)),
            dest_size: Some(vec2(32.0, 32.0)),
            flip_x: face_left,
            ..Default::default()
        },
    );
}

/// 쉼터 모닥불: 돌 둘레 + 장작 + 흔들리는 불꽃 + 바닥 빛.
/// 모닥불의 돌만(불꽃·빛 없이): 그림자 패스용(0105). 그림 자리는 `draw_campfire_sunny`·`draw_campfire`와 같다.
fn draw_campfire_stones(sunny: Option<&art::Sunny>, r: Rect) {
    if let Some(sunny) = sunny {
        let c = vec2(r.center().x, r.bottom() - 4.0);
        crate::sunny::draw_at_foot(
            &sunny.sheet,
            crate::sunny::FIRE_PIT,
            c + vec2(0.0, 6.0),
            WHITE,
        );
        return;
    }
    let c = vec2(r.center().x, r.bottom() - 5.0);
    for k in 0..7 {
        let a = k as f32 / 7.0 * std::f32::consts::TAU;
        let p = c + vec2(a.cos() * 7.0, a.sin() * 3.0);
        draw_rectangle((p.x - 1.5).round(), (p.y - 1.0).round(), 3.0, 2.0, OUTLINE);
    }
}

fn draw_campfire(r: Rect, time: f32) {
    let c = vec2(r.center().x, r.bottom() - 5.0);
    let flick = (time * 9.0).sin() * 0.5 + (time * 13.7).sin() * 0.5;
    draw_circle(
        c.x,
        c.y - 2.0,
        22.0 + flick,
        Color::new(1.0, 0.65, 0.3, 0.07),
    );
    draw_circle(
        c.x,
        c.y - 2.0,
        13.0 + flick * 0.5,
        Color::new(1.0, 0.7, 0.35, 0.08),
    );
    let stone = Color::new(0.45, 0.42, 0.44, 1.0);
    for k in 0..7 {
        let a = k as f32 / 7.0 * std::f32::consts::TAU;
        let p = c + vec2(a.cos() * 7.0, a.sin() * 3.0);
        draw_rectangle((p.x - 1.5).round(), (p.y - 1.0).round(), 3.0, 2.0, OUTLINE);
        draw_rectangle((p.x - 1.0).round(), (p.y - 1.0).round(), 2.0, 1.0, stone);
    }
    let wood = Color::new(0.45, 0.30, 0.20, 1.0);
    draw_line(c.x - 5.0, c.y + 1.0, c.x + 5.0, c.y - 2.0, 2.0, wood);
    draw_line(c.x - 5.0, c.y - 2.0, c.x + 5.0, c.y + 1.0, 2.0, wood);
    let h = 7.0 + flick * 1.5;
    draw_triangle(
        vec2(c.x - 4.0, c.y - 1.0),
        vec2(c.x + 4.0, c.y - 1.0),
        vec2(c.x + flick, c.y - h - 3.0),
        Color::new(0.95, 0.45, 0.2, 1.0),
    );
    draw_triangle(
        vec2(c.x - 2.5, c.y - 1.0),
        vec2(c.x + 2.5, c.y - 1.0),
        vec2(c.x + flick * 0.5, c.y - h),
        Color::new(1.0, 0.85, 0.35, 1.0),
    );
    // 불티
    for k in 0..3 {
        let ph = (time * 0.9 + k as f32 * 0.33).fract();
        let x = c.x + ((k as f32 * 2.1 + time * 2.0).sin() * 4.0);
        draw_rectangle(
            x.round(),
            (c.y - 8.0 - ph * 16.0).round(),
            1.0,
            1.0,
            Color::new(1.0, 0.8, 0.4, 1.0 - ph),
        );
    }
}

/// 모닥불, 팩 그림판(0066): 돌 두른 자리 + 불꽃 4칸 + 따뜻한 빛·불티(직접).
fn draw_campfire_sunny(sunny: &art::Sunny, r: Rect, time: f32) {
    let c = vec2(r.center().x, r.bottom() - 4.0);
    let flick = (time * 9.0).sin() * 0.5 + (time * 13.7).sin() * 0.5;
    draw_circle(
        c.x,
        c.y - 4.0,
        24.0 + flick,
        Color::new(1.0, 0.65, 0.3, 0.07),
    );
    draw_circle(
        c.x,
        c.y - 4.0,
        14.0 + flick * 0.5,
        Color::new(1.0, 0.7, 0.35, 0.08),
    );
    crate::sunny::draw_at_foot(
        &sunny.sheet,
        crate::sunny::FIRE_PIT,
        c + vec2(0.0, 6.0),
        WHITE,
    );
    let (fw, fh) = crate::sunny::FIRE_FRAME;
    let f = ((time * 8.0) as u32 % 4) as f32;
    draw_texture_ex(
        &sunny.fire,
        (c.x - fw / 2.0).round(),
        (c.y - fh + 1.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(f * fw, 0.0, fw, fh)),
            ..Default::default()
        },
    );
    for k in 0..3 {
        let ph = (time * 0.9 + k as f32 * 0.33).fract();
        let x = c.x + ((k as f32 * 2.1 + time * 2.0).sin() * 4.0);
        draw_rectangle(
            x.round(),
            (c.y - 10.0 - ph * 16.0).round(),
            1.0,
            1.0,
            Color::new(1.0, 0.8, 0.4, 1.0 - ph),
        );
    }
}

/// 포포의 가게 좌판: 나무 상자 + 줄무늬 차양 + 산딸기·지렁이 단지.
fn draw_stall(r: Rect) {
    let wood = Color::new(0.55, 0.36, 0.22, 1.0);
    let dark = Color::new(0.36, 0.22, 0.14, 1.0);
    let (x, y) = (r.x, r.bottom() - 12.0);
    draw_rectangle(x - 1.0, y - 1.0, 18.0, 13.0, OUTLINE);
    draw_rectangle(x, y, 16.0, 11.0, wood);
    draw_rectangle(x, y + 8.0, 16.0, 3.0, dark);
    // 차양(위로 솟은 막대 둘 + 줄무늬 천)
    draw_rectangle(x, y - 14.0, 1.0, 14.0, dark);
    draw_rectangle(x + 15.0, y - 14.0, 1.0, 14.0, dark);
    for k in 0..4 {
        let c = if k % 2 == 0 {
            Color::new(0.85, 0.35, 0.35, 1.0)
        } else {
            Color::new(0.95, 0.88, 0.75, 1.0)
        };
        draw_rectangle(x - 1.0 + k as f32 * 4.5, y - 17.0, 4.5, 4.0, c);
    }
    draw_rectangle(x - 1.0, y - 13.0, 18.0, 1.0, OUTLINE);
    // 물건
    draw_circle(x + 4.0, y - 1.0, 2.0, world::BERRY);
    draw_circle(x + 7.0, y - 1.0, 2.0, world::BERRY);
    draw_rectangle(x + 10.0, y - 3.0, 4.0, 3.0, world::SEED);
}

/// 가게(팩 장터 가판, 0077): 판대 위에 산딸기·씨앗 자루는 직접 올린다(무엇을 파는지 보이게).
fn draw_stall_sunny(sunny: &art::Sunny, r: Rect) {
    use crate::sunny as sn;
    let foot = vec2((r.center().x).round(), r.bottom().round());
    draw_ellipse(foot.x, foot.y - 1.0, 13.0, 3.0, 0.0, SHADOW_SOFT);
    sn::draw_at_foot(&sunny.sheet, sn::STALL, foot, WHITE);
    let top = foot.y - sn::STALL.3 as f32 + sn::STALL_COUNTER;
    let x = foot.x - 7.0;
    draw_circle(x + 3.0, top - 1.0, 2.0, world::BERRY);
    draw_circle(x + 6.0, top - 1.0, 2.0, world::BERRY);
    draw_rectangle(x + 10.0, top - 4.0, 4.0, 4.0, OUTLINE);
    draw_rectangle(x + 10.5, top - 3.5, 3.0, 3.5, world::SEED);
}

/// 빈 자리(0046): 팩 그루터기 + 꽃잎. 꽃잎 규칙은 `draw_seat`와 같다.
fn draw_seat_sunny(sunny: &art::Sunny, r: Rect, petals: u8) {
    use crate::sunny as sn;
    let foot = vec2((r.center().x).round(), r.bottom().round());
    draw_ellipse(foot.x, foot.y - 1.0, 8.0, 2.0, 0.0, SHADOW_SOFT);
    sn::draw_at_foot(&sunny.sheet, sn::STUMPS[1], foot, WHITE);
    let (x, y) = (foot.x - 7.0, foot.y - 12.0);
    seat_petals(x, y, petals);
}

/// 빈 자리: 통나무 의자 + 시든 꽃잎 하나(모모가 돌아올 곳).
/// 모모가 숨은 흙 둔덕(0046): 따라오는 동안은 흙이 튀고, 멈춘 뒤엔 떨며 부풀어 오른다(튀어나올 예고).
fn draw_dig_mound(pos: Vec2, ut: f32, windup: f32, time: f32) {
    let hold = ((ut - t::MOMO_DIG_TRACK) / windup.max(0.01)).clamp(0.0, 1.0);
    let shake = if hold > 0.0 && ((time * 50.0) as i32) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    let (x, y) = ((pos.x + shake).round(), pos.y.round() + 4.0);
    let w = 6.0 + hold * 3.0;
    draw_ellipse(x, y + 1.0, w + 1.0, 3.5, 0.0, OUTLINE);
    draw_ellipse(x, y, w, 3.0 + hold, 0.0, world::DIRT);
    draw_ellipse(
        x - 1.0,
        y - 1.0,
        w * 0.5,
        1.2,
        0.0,
        Color::new(0.70, 0.54, 0.38, 1.0),
    );
    // 흙 부스러기
    for k in 0..3 {
        let ph = (time * 3.0 + k as f32 * 0.33).fract();
        let dx = (k as f32 - 1.0) * 4.0;
        let c = Color::new(0.55, 0.40, 0.28, 1.0 - ph);
        draw_rectangle(x + dx, y - 3.0 - ph * 5.0, 1.0, 1.0, c);
    }
    // 떨 때: 착지 고리처럼 둘레를 보여 준다.
    if hold > 0.0 {
        draw_circle_lines(
            x,
            y,
            8.0 + hold * 12.0,
            1.0,
            Color::new(1.0, 0.5, 0.5, 0.35 + 0.4 * hold),
        );
    }
}

/// 도토가 묻은 도토리(0052): 흙 둔덕 + 삐죽 나온 갓. 터질 둘레를 고리로 보이고, 고리가 좁혀 오다 끝에선 깜박인다.
fn draw_mine(m: &world::Mine, time: f32) {
    let k = (m.time / m.fuse.max(0.01)).clamp(0.0, 1.0);
    let (x, y) = (m.pos.x.round(), m.pos.y.round());
    let r = t::DOTO_MINE_RADIUS;
    let late = k > 0.7 && ((time * 16.0) as i32) % 2 == 0;
    let c = if late {
        Color::new(1.0, 0.85, 0.4, 0.85)
    } else {
        Color::new(1.0, 0.55, 0.4, 0.3 + 0.4 * k)
    };
    draw_circle_lines(x, y, r, 1.0, c);
    draw_circle_lines(x, y, (r * (1.0 - k)).max(2.0), 1.0, c);
    let swell = (k * 2.0).round();
    draw_ellipse(x, y + 1.0, 5.0 + swell, 2.5, 0.0, OUTLINE);
    draw_ellipse(x, y, 4.0 + swell, 2.0, 0.0, world::DIRT);
    // 갓(도토리 머리)
    let shake = if k > 0.5 && ((time * 40.0) as i32) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    draw_rectangle(
        x - 2.0 + shake,
        y - 2.0 - swell,
        4.0,
        2.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
    draw_rectangle(
        x + shake,
        y - 3.0 - swell,
        1.0,
        1.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
}

/// 실 고리(0055): 막대 둘에 걸린 흰 실 고리(실뜨기 모양 하나). 둥실 떠오르며 보랏빛으로 숨 쉰다.
fn draw_dream_loop(at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round());
    // 막대 둘
    for dx in [-7.0, 7.0] {
        draw_rectangle(x + dx - 1.0, y - 14.0, 2.0, 18.0, OUTLINE);
        draw_rectangle(
            x + dx - 0.5,
            y - 13.0,
            1.0,
            16.0,
            Color::new(0.55, 0.40, 0.30, 1.0),
        );
    }
    let bob = ((time * 1.6).sin() * 1.5).round();
    let glow = 0.35 + 0.25 * (time * 2.2).sin().abs();
    if !shadow_pass() {
        draw_circle(
            x,
            y - 12.0 + bob,
            9.0,
            Color::new(0.72, 0.62, 0.95, glow * 0.35),
        );
    }
    // 실뜨기 마름모 + 가로줄
    let c = Color::new(0.95, 0.93, 1.0, 1.0);
    let (cx, cy) = (x, y - 12.0 + bob);
    let pts = [
        vec2(cx - 6.0, cy),
        vec2(cx, cy - 5.0),
        vec2(cx + 6.0, cy),
        vec2(cx, cy + 5.0),
    ];
    for i in 0..4 {
        let (a, b) = (pts[i], pts[(i + 1) % 4]);
        draw_line(a.x, a.y, b.x, b.y, 1.0, c);
    }
    draw_line(
        cx - 6.0,
        cy,
        cx + 6.0,
        cy,
        1.0,
        Color::new(0.80, 0.72, 1.0, 1.0),
    );
    draw_line(x - 7.0, y - 13.0, cx - 6.0, cy, 1.0, c);
    draw_line(x + 7.0, y - 13.0, cx + 6.0, cy, 1.0, c);
    // 떠오르는 빛 알갱이
    for k in (0..2).filter(|_| !shadow_pass()) {
        let ph = (time * 0.7 + k as f32 * 0.5).fract();
        draw_rectangle(
            (cx - 3.0 + k as f32 * 6.0).round(),
            (cy - 4.0 - ph * 10.0).round(),
            1.0,
            1.0,
            Color::new(0.85, 0.80, 1.0, 1.0 - ph),
        );
    }
}

/// 마을 물웅덩이(0048): 파란 타원 + 떠다니는 반짝임 둘.
fn draw_puddle(at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round() + 3.0);
    draw_ellipse(x, y, 11.0, 5.0, 0.0, Color::new(0.20, 0.30, 0.28, 1.0));
    draw_ellipse(x, y, 10.0, 4.0, 0.0, Color::new(0.38, 0.62, 0.78, 1.0));
    draw_ellipse(
        x - 2.0,
        y - 1.0,
        5.0,
        1.5,
        0.0,
        Color::new(0.62, 0.82, 0.92, 1.0),
    );
    for k in 0..2 {
        let ph = (time * 0.7 + k as f32 * 0.5).fract();
        let c = Color::new(1.0, 1.0, 1.0, 0.8 * (1.0 - ph));
        draw_rectangle(
            (x - 5.0 + k as f32 * 7.0 + ph * 3.0).round(),
            y - 1.0 + k as f32,
            1.0,
            1.0,
            c,
        );
    }
}

/// 빈 자리(모모를 기다리는 그루터기). 꽃잎 셋 가운데 모모를 이긴 만큼 분홍(0046).
fn draw_seat(r: Rect, petals: u8) {
    let bark = Color::new(0.42, 0.28, 0.20, 1.0);
    let ring = Color::new(0.75, 0.58, 0.40, 1.0);
    let (x, y) = (r.x + 1.0, r.bottom() - 9.0);
    draw_rectangle(x - 1.0, y - 1.0, 16.0, 10.0, OUTLINE);
    draw_rectangle(x, y + 2.0, 14.0, 6.0, bark);
    draw_ellipse(x + 7.0, y + 2.0, 7.0, 2.5, 0.0, ring);
    draw_ellipse_lines(x + 7.0, y + 2.0, 4.0, 1.2, 0.0, 1.0, bark);
    seat_petals(x, y, petals);
}

/// 의자 위 꽃잎 셋: 모모를 이긴 만큼 분홍, 처음엔 시든 것 하나만.
fn seat_petals(x: f32, y: f32, petals: u8) {
    let wilted = Color::new(0.69, 0.50, 0.55, 1.0);
    let pink = Color::new(1.0, 0.66, 0.77, 1.0);
    for (i, dx) in [10.0, 3.0, 7.0].into_iter().enumerate() {
        // 처음엔 시든 꽃잎 하나만 떨어져 있다(seat_1).
        if petals == 0 && i > 0 {
            break;
        }
        let c = if (i as u8) < petals { pink } else { wilted };
        draw_rectangle(x + dx, y + 1.0 + (i % 2) as f32, 2.0, 1.0, c);
    }
}

/// 누룩 할매(마을, 0045): 흰 실 덩어리 + 버섯 갓 셋 + 감은 눈. 숨 쉬듯 살짝 오르내린다.
fn draw_halmae_at(pos: Vec2, time: f32) {
    let bob = ((time * 1.6).sin() * 1.0).round();
    let (x, y) = (pos.x.round(), pos.y.round() + 2.0 - bob);
    let pale = Color::new(0.86, 0.84, 0.93, 1.0);
    let shade = Color::new(0.66, 0.62, 0.78, 1.0);
    let cap = Color::new(0.78, 0.60, 0.38, 1.0);
    let cap_dark = Color::new(0.52, 0.38, 0.26, 1.0);
    // 바닥에 퍼진 실(바닥에 깔려 그림자 패스에서는 뺀다)
    for k in (0..5).filter(|_| !shadow_pass()) {
        let a = k as f32 * 1.2 + 0.3;
        let (dx, dy) = (a.cos() * 11.0, a.sin().abs() * 3.0 + 4.0);
        draw_line(
            x,
            y + 3.0,
            x + dx,
            y + dy,
            1.0,
            Color::new(0.90, 0.88, 0.96, 0.7),
        );
    }
    draw_ellipse(x, y, 9.0, 7.0, 0.0, OUTLINE);
    draw_ellipse(x, y, 8.0, 6.0, 0.0, shade);
    draw_ellipse(x - 1.0, y - 1.0, 7.0, 5.0, 0.0, pale);
    for (cx, cy, r) in [(-4.0, -6.0, 3.0), (1.0, -8.0, 4.0), (5.0, -5.0, 2.5)] {
        draw_rectangle(x + cx - 0.5, y + cy, 1.0, 3.0, pale);
        draw_ellipse(x + cx, y + cy, r + 0.8, r * 0.6 + 0.8, 0.0, OUTLINE);
        draw_ellipse(x + cx, y + cy, r, r * 0.6, 0.0, cap);
        draw_rectangle(x + cx - r * 0.4, y + cy - r * 0.3, 1.0, 1.0, cap_dark);
    }
    // 감은 눈 + 홍조
    for ex in [-3.0, 2.0] {
        draw_rectangle(x + ex, y - 1.0, 2.0, 1.0, Color::new(0.35, 0.28, 0.45, 1.0));
    }
    draw_rectangle(
        x - 5.0,
        y + 1.0,
        2.0,
        1.0,
        Color::new(0.93, 0.62, 0.70, 1.0),
    );
    draw_rectangle(
        x + 4.0,
        y + 1.0,
        2.0,
        1.0,
        Color::new(0.93, 0.62, 0.70, 1.0),
    );
}

/// 큰 나무 밑동의 빛(0061), 나무 뒤: 은은한 빛 웅덩이와, 밑동에서 퍼져 나가는 밝은 고리.
fn draw_tree_glow(base: Vec2, time: f32) {
    let warm = Color::new(1.0, 0.95, 0.55, 1.0);
    let k = 0.5 + 0.5 * (time * 2.5).sin();
    for (r, a) in [(26.0, 0.14), (16.0, 0.22)] {
        draw_ellipse(
            base.x,
            base.y,
            r + k * 3.0,
            (r + k * 3.0) * 0.4,
            0.0,
            Color::new(warm.r, warm.g, warm.b, a),
        );
    }
    // 고리 둘이 엇갈려 퍼진다(풀밭 초록 위에서도 선으로 읽힌다).
    for off in [0.0, 0.5] {
        let ph = (time * 0.7 + off).fract();
        let r = 8.0 + ph * 26.0;
        let a = (1.0 - ph) * 0.9;
        draw_ellipse_lines(
            base.x,
            base.y,
            r,
            r * 0.4,
            0.0,
            1.0,
            Color::new(1.0, 1.0, 0.7, a),
        );
    }
}

/// 큰 나무 밑동의 빛 알갱이(0061), 나무 앞: 줄기를 따라 떠오른다.
fn draw_tree_motes(base: Vec2, time: f32) {
    for i in 0..10 {
        let ph = (time * 0.55 + i as f32 / 10.0).fract();
        let x = base.x + ((i as f32 * 2.4 + time * 1.3).sin()) * 16.0;
        let y = base.y + 2.0 - ph * 44.0;
        let a = (1.0 - ph).min(ph * 4.0);
        let sz = if i % 3 == 0 { 2.0 } else { 1.0 };
        draw_rectangle(x.round(), y.round(), sz, sz, Color::new(0.95, 1.0, 0.6, a));
    }
}

/// 큰 나무(마을), 팩 그림판(0065): 우리 줄기·뿌리 위에 Sunnyside 둥근 나무의 잎을 무더기로 쌓는다.
/// 잎 무더기 수 = 잎 단계(1 → 3, 2 → 7, 3 → 10, 4 → 12). 3부터 꽃. 무더기마다 흔들림이 어긋난다.
fn draw_great_tree_sunny(sunny: &art::Sunny, at: Vec2, growth: u8, time: f32) {
    let bark = Color::new(0.45, 0.29, 0.22, 1.0);
    let bark_hi = Color::new(0.58, 0.39, 0.28, 1.0);
    let bark_lo = Color::new(0.32, 0.20, 0.17, 1.0);
    let (x, base) = (at.x.round(), at.y + 8.0);
    // 뿌리: 벼랑 위로 뻗는 굵은 뿌리 넷(바닥에 깔려 그림자 패스에서는 뺀다).
    for (dx, dy) in [(-30.0, 4.0), (-14.0, 6.0), (14.0, 6.0), (30.0, 4.0)]
        .into_iter()
        .filter(|_| !shadow_pass())
    {
        draw_line(x + dx * 0.3, base - 4.0, x + dx, base + dy, 6.0, OUTLINE);
        draw_line(x + dx * 0.3, base - 4.0, x + dx, base + dy, 4.0, bark);
        draw_line(
            x + dx * 0.3,
            base - 5.0,
            x + dx * 0.9,
            base + dy - 1.0,
            1.0,
            bark_hi,
        );
    }
    // 줄기: 아래가 넓다.
    // 마을 지도에서 큰 나무는 위쪽 가장자리 가까이 선다: 잎 꼭대기가 밑동 위 60px 안에 들어와야 화면에 다 보인다.
    let top = base - 22.0;
    draw_triangle(
        vec2(x - 16.0, base),
        vec2(x + 16.0, base),
        vec2(x, base - 12.0),
        OUTLINE,
    );
    draw_rectangle(x - 13.0, top, 26.0, base - top, OUTLINE);
    draw_rectangle(x - 12.0, top, 24.0, base - top, bark);
    draw_rectangle(x - 12.0, top, 5.0, base - top, bark_hi);
    draw_rectangle(x + 7.0, top, 5.0, base - top, bark_lo);
    for k in 0..2 {
        let yy = top + 8.0 + k as f32 * 6.0;
        draw_rectangle(x - 4.0 + (k % 2) as f32 * 6.0, yy, 1.0, 5.0, bark_lo);
    }
    // 잎 무더기(뒤 → 앞). (가운데에서 x, 잎 아래 끝의 줄기 꼭대기 기준 y, 나오는 단계)
    let crowns: [(f32, f32, u8); 12] = [
        (-12.0, -16.0, 4),
        (12.0, -17.0, 4),
        (-26.0, -8.0, 3),
        (0.0, -9.0, 3),
        (26.0, -8.0, 3),
        (-38.0, 1.0, 2),
        (-13.0, 0.0, 2),
        (13.0, 0.0, 2),
        (38.0, 1.0, 2),
        (-25.0, 8.0, 1),
        (0.0, 7.0, 1),
        (25.0, 8.0, 1),
    ];
    let (tw, _) = art::SUNNY_TREE;
    for (i, (dx, dy, need)) in crowns.iter().enumerate() {
        if growth < *need {
            continue;
        }
        let f = ((time * 2.5 + i as f32 * 0.37) as u32 % 4) as f32;
        let (cx, cy) = (
            (x + dx - tw / 2.0).round(),
            (top + dy - art::SUNNY_CROWN_H).round(),
        );
        draw_texture_ex(
            &sunny.tree,
            cx,
            cy,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(f * tw, 0.0, tw, art::SUNNY_CROWN_H)),
                ..Default::default()
            },
        );
        if growth >= 3 && i % 2 == 0 {
            // 꽃(분홍 셋, 직접 찍음).
            for (bx, by) in [(10.0, 6.0), (19.0, 10.0), (14.0, 14.0)] {
                draw_rectangle(cx + bx, cy + by, 2.0, 2.0, Color::new(1.0, 0.76, 0.84, 1.0));
                draw_rectangle(cx + bx, cy + by, 1.0, 1.0, WHITE);
            }
        }
    }
}

/// 큰 나무(마을, 0045): 벼랑 위로 솟은 줄기와 뿌리. 잎은 깬 횟수만큼 돌아온다(0 = 마른 잎 몇 장, 3부터 꽃).
fn draw_great_tree(at: Vec2, growth: u8, time: f32) {
    let bark = Color::new(0.40, 0.27, 0.21, 1.0);
    let bark_hi = Color::new(0.52, 0.37, 0.27, 1.0);
    let (x, base) = (at.x.round(), at.y + 8.0);
    // 뿌리(바닥에 깔려 그림자 패스에서는 뺀다)
    for (dx, len) in [(-26.0, 18.0), (-12.0, 10.0), (12.0, 10.0), (26.0, 18.0)]
        .into_iter()
        .filter(|_| !shadow_pass())
    {
        draw_line(
            x + dx * 0.4,
            base - 2.0,
            x + dx,
            base + len * 0.2,
            4.0,
            OUTLINE,
        );
        draw_line(
            x + dx * 0.4,
            base - 2.0,
            x + dx,
            base + len * 0.2,
            2.0,
            bark,
        );
    }
    // 줄기
    let top = base - 44.0;
    draw_rectangle(x - 13.0, top, 26.0, base - top, OUTLINE);
    draw_rectangle(x - 12.0, top, 24.0, base - top, bark);
    draw_rectangle(x - 9.0, top, 4.0, base - top, bark_hi);
    for k in 0..3 {
        draw_rectangle(
            x - 3.0 + k as f32 * 5.0,
            top + 6.0 + k as f32 * 9.0,
            1.0,
            10.0,
            OUTLINE,
        );
    }
    // 잎·꽃: 단계마다 더 많이(위치는 고정된 표에서)
    let spots: [(f32, f32); 15] = [
        (-30.0, -4.0),
        (28.0, -6.0),
        (0.0, -12.0),
        (-16.0, -14.0),
        (16.0, -16.0),
        (-40.0, 4.0),
        (40.0, 2.0),
        (-8.0, -22.0),
        (8.0, -24.0),
        (-26.0, -18.0),
        (26.0, -20.0),
        (0.0, -30.0),
        (-36.0, -10.0),
        (36.0, -12.0),
        (-14.0, -30.0),
    ];
    let n = (3 + growth as usize * 3).min(spots.len());
    for (i, (dx, dy)) in spots.iter().take(n).enumerate() {
        let sway = ((time * 1.2 + i as f32).sin() * 1.0).round();
        let (cx, cy) = (x + dx + sway, top + dy);
        let r = if growth == 0 { 3.0 } else { 7.0 };
        let leaf = if growth == 0 {
            Color::new(0.62, 0.48, 0.30, 1.0)
        } else {
            Color::new(0.36 + 0.03 * (i % 3) as f32, 0.58, 0.32, 1.0)
        };
        draw_circle(cx, cy, r + 1.0, OUTLINE);
        draw_circle(cx, cy, r, leaf);
        if growth >= 3 && i % 2 == 0 {
            draw_circle(cx + 2.0, cy - 2.0, 2.0, Color::new(1.0, 0.78, 0.84, 1.0));
        }
    }
}

/// 마을 꾸미기(0045): 0 꽃밭, 1 반딧불 등, 2 빛나는 버섯, 3 통나무 의자, 4 바람종, 5 깃발 줄.
/// 마을 꾸미기, 팩 그림판(0066): 0 꽃밭 = 열매·꽃 덤불 + 들꽃, 2 빛나는 버섯 = 파란 버섯 셋(움직임).
fn draw_decor_sunny(sunny: &art::Sunny, i: usize, at: Vec2, time: f32) {
    use crate::sunny as sn;
    let foot = vec2(at.x.round(), at.y.round() + 6.0);
    match i {
        0 => {
            draw_ellipse(foot.x, foot.y - 1.0, 16.0, 3.0, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(&sunny.sheet, sn::BUSH_BERRY, foot + vec2(-8.0, 0.0), WHITE);
            sn::draw_at_foot(&sunny.sheet, sn::BUSH_FLOWER, foot + vec2(9.0, 1.0), WHITE);
            for (k, dx) in [-14.0f32, -2.0, 15.0].iter().enumerate() {
                let set = [sn::FLOWERS_WHITE, sn::FLOWERS_RED, sn::FLOWERS_BLUE][k];
                sn::draw_at_foot(&sunny.sheet, set[3], foot + vec2(*dx, 4.0), WHITE);
            }
        }
        3 => {
            draw_ellipse(foot.x, foot.y - 1.0, 12.0, 2.5, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(&sunny.sheet, sn::BENCH, foot, WHITE);
        }
        7 => {
            draw_ellipse(foot.x - 2.0, foot.y - 1.0, 8.0, 2.0, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(
                &sunny.sheet,
                sn::POT_WILDFLOWER,
                foot + vec2(-3.0, 0.0),
                WHITE,
            );
            sn::draw_at_foot(&sunny.sheet, sn::WILDFLOWERS, foot + vec2(6.0, 1.0), WHITE);
        }
        _ => {
            let s = sn::MUSH_FRAME;
            for (k, dx) in [-8.0f32, 0.0, 8.0].iter().enumerate() {
                let pulse = 0.6 + 0.3 * (time * 1.8 + k as f32).sin();
                let f = ((time * 4.0 + k as f32 * 1.3) as u32 % 4) as f32;
                let (x, y) = (
                    (foot.x + dx - s / 2.0).round(),
                    (foot.y - s + (k % 2) as f32 * 2.0).round(),
                );
                if !shadow_pass() {
                    draw_circle(
                        x + 8.0,
                        y + 10.0,
                        7.0,
                        Color::new(0.5, 0.85, 1.0, 0.12 * pulse),
                    );
                }
                draw_texture_ex(
                    &sunny.mush,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(f * s, 0.0, s, s)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn draw_decor(i: usize, at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round() + 4.0);
    let wood = Color::new(0.45, 0.31, 0.22, 1.0);
    match i {
        0 => {
            draw_ellipse(x, y, 11.0, 4.0, 0.0, OUTLINE);
            draw_ellipse(x, y, 10.0, 3.0, 0.0, Color::new(0.42, 0.30, 0.22, 1.0));
            let cols = [
                Color::new(1.0, 0.70, 0.78, 1.0),
                Color::new(1.0, 0.92, 0.55, 1.0),
                Color::new(0.80, 0.70, 1.0, 1.0),
            ];
            for k in 0..5 {
                let fx = x - 8.0 + k as f32 * 4.0;
                draw_rectangle(fx, y - 4.0, 1.0, 3.0, Color::new(0.36, 0.58, 0.32, 1.0));
                draw_rectangle(fx - 1.0, y - 6.0, 3.0, 2.0, cols[k % 3]);
            }
        }
        1 => {
            let pulse = 0.55 + 0.25 * (time * 2.5).sin();
            draw_circle(x, y - 12.0, 9.0, Color::new(0.85, 1.0, 0.55, 0.15 * pulse));
            draw_rectangle(x - 1.0, y - 10.0, 2.0, 12.0, wood);
            draw_rectangle(x - 3.0, y - 15.0, 6.0, 6.0, OUTLINE);
            draw_rectangle(
                x - 2.0,
                y - 14.0,
                4.0,
                4.0,
                Color::new(0.85, 1.0, 0.55, pulse + 0.2),
            );
        }
        2 => {
            for (k, (dx, h)) in [(-5.0f32, 5.0f32), (0.0, 8.0), (5.0, 4.0)]
                .into_iter()
                .enumerate()
            {
                let pulse = 0.6 + 0.3 * (time * 1.8 + k as f32).sin();
                draw_circle(
                    x + dx,
                    y - h,
                    5.0,
                    Color::new(0.55, 0.95, 1.0, 0.12 * pulse),
                );
                draw_rectangle(
                    x + dx - 0.5,
                    y - h + 1.0,
                    1.0,
                    h,
                    Color::new(0.88, 0.90, 0.86, 1.0),
                );
                draw_ellipse(x + dx, y - h, 3.0, 2.0, 0.0, OUTLINE);
                draw_ellipse(
                    x + dx,
                    y - h,
                    2.5,
                    1.5,
                    0.0,
                    Color::new(0.55, 0.95, 1.0, pulse),
                );
            }
        }
        3 => {
            draw_rectangle(x - 13.0, y - 5.0, 26.0, 7.0, OUTLINE);
            draw_rectangle(x - 12.0, y - 4.0, 24.0, 5.0, wood);
            draw_ellipse(
                x - 12.0,
                y - 1.5,
                2.0,
                2.5,
                0.0,
                Color::new(0.75, 0.58, 0.40, 1.0),
            );
            draw_ellipse(
                x + 12.0,
                y - 1.5,
                2.0,
                2.5,
                0.0,
                Color::new(0.75, 0.58, 0.40, 1.0),
            );
        }
        4 => {
            draw_rectangle(x - 1.0, y - 16.0, 2.0, 18.0, wood);
            draw_rectangle(x - 6.0, y - 16.0, 12.0, 2.0, wood);
            for k in 0..3 {
                let sw = (time * 2.0 + k as f32).sin() * 1.5;
                let cx = x - 4.0 + k as f32 * 4.0;
                draw_line(
                    cx,
                    y - 14.0,
                    cx + sw,
                    y - 9.0,
                    1.0,
                    Color::new(0.85, 0.85, 0.80, 1.0),
                );
                draw_rectangle(
                    cx + sw - 1.0,
                    y - 9.0,
                    2.0,
                    3.0,
                    Color::new(0.70, 0.85, 0.95, 1.0),
                );
            }
        }
        // 저널 전용(0048). 6 반딧불 병: 유리병 안에서 빛이 떠다닌다.
        6 => {
            let glass = Color::new(0.70, 0.85, 0.90, 0.45);
            draw_rectangle(x - 6.0, y - 13.0, 12.0, 14.0, OUTLINE);
            draw_rectangle(x - 5.0, y - 12.0, 10.0, 12.0, glass);
            draw_rectangle(x - 3.0, y - 16.0, 6.0, 4.0, OUTLINE);
            draw_rectangle(x - 2.0, y - 15.0, 4.0, 2.0, wood);
            draw_circle(x, y - 6.0, 10.0, Color::new(0.85, 1.0, 0.55, 0.10));
            for k in 0..4 {
                let a = time * (1.2 + k as f32 * 0.3) + k as f32 * 1.7;
                let p = vec2(x + a.cos() * 3.0, y - 6.0 + (a * 1.3).sin() * 4.0);
                let c = Color::new(0.85, 1.0, 0.55, 0.6 + 0.4 * (a * 2.0).sin().abs());
                draw_rectangle(p.x.round(), p.y.round(), 1.0, 1.0, c);
            }
        }
        // 7 들꽃 화분.
        7 => {
            let pot = Color::new(0.72, 0.42, 0.30, 1.0);
            draw_rectangle(x - 6.0, y - 6.0, 12.0, 8.0, OUTLINE);
            draw_rectangle(x - 5.0, y - 5.0, 10.0, 6.0, pot);
            draw_rectangle(
                x - 5.0,
                y - 5.0,
                10.0,
                1.0,
                Color::new(0.86, 0.56, 0.40, 1.0),
            );
            let cols = [
                Color::new(1.0, 0.66, 0.77, 1.0),
                Color::new(0.80, 0.70, 1.0, 1.0),
                Color::new(1.0, 0.92, 0.55, 1.0),
            ];
            for (k, (dx, h)) in [(-3.0f32, 11.0f32), (0.0, 14.0), (3.0, 10.0)]
                .into_iter()
                .enumerate()
            {
                let sway = (time * 1.5 + k as f32).sin().round();
                draw_line(
                    x + dx,
                    y - 5.0,
                    x + dx + sway,
                    y - h,
                    1.0,
                    Color::new(0.40, 0.65, 0.30, 1.0),
                );
                draw_rectangle(x + dx + sway - 1.0, y - h - 1.0, 3.0, 3.0, cols[k]);
            }
        }
        // 8 실 그네: 두 기둥에 흰 실, 천천히 흔들린다.
        8 => {
            draw_rectangle(x - 9.0, y - 20.0, 2.0, 22.0, wood);
            draw_rectangle(x + 7.0, y - 20.0, 2.0, 22.0, wood);
            draw_rectangle(x - 9.0, y - 21.0, 18.0, 2.0, wood);
            let swing = (time * 1.2).sin() * 2.0;
            let thread = Color::new(0.92, 0.90, 0.96, 1.0);
            draw_line(x - 4.0, y - 19.0, x - 4.0 + swing, y - 6.0, 1.0, thread);
            draw_line(x + 4.0, y - 19.0, x + 4.0 + swing, y - 6.0, 1.0, thread);
            draw_rectangle(
                (x - 5.0 + swing).round(),
                y - 6.0,
                10.0,
                2.0,
                Color::new(0.78, 0.60, 0.38, 1.0),
            );
        }
        _ => {
            draw_rectangle(x - 17.0, y - 16.0, 2.0, 18.0, wood);
            draw_rectangle(x + 15.0, y - 16.0, 2.0, 18.0, wood);
            draw_line(
                x - 16.0,
                y - 15.0,
                x + 16.0,
                y - 15.0,
                1.0,
                Color::new(0.85, 0.80, 0.70, 1.0),
            );
            let cols = [
                Color::new(0.95, 0.45, 0.50, 1.0),
                Color::new(0.98, 0.85, 0.40, 1.0),
                Color::new(0.50, 0.75, 0.95, 1.0),
                Color::new(0.60, 0.85, 0.50, 1.0),
            ];
            for k in 0..6 {
                let fx = x - 13.0 + k as f32 * 5.0;
                let sag = ((k as f32 - 2.5).abs() - 2.5) * -0.6;
                draw_triangle(
                    vec2(fx, y - 15.0 + sag),
                    vec2(fx + 4.0, y - 15.0 + sag),
                    vec2(fx + 2.0, y - 10.0 + sag),
                    cols[k % 4],
                );
            }
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
    Neighbor(usize),
    /// 마을(0045): 큰 나무(자리, 잎 단계), 꾸미기(번호, 자리).
    Tree(Vec2, u8),
    Decor(usize, Vec2),
    /// 실 고리(0055).
    DreamLoop(Vec2),
}

/// 3/4 시점: 발 위치(y)가 아래인 것이 앞. 그림자는 모두 먼저.
/// `sun`이 있으면 해 그림자 패스(0102): 바닥 것(발밑 그림자·웅덩이·지뢰)은 빼고, 서 있는 것마다 발 줄을 축으로
/// 바닥에 눕혀(`sun_shear`) 그대로 다시 그린다. 색은 버리고 모양만 쓴다(`Renderer::draw_sun_shadows`).
fn draw_sorted(art: &Art, w: &World, alpha: f32, sun: Option<ShadowLight>) {
    let ground = sun.is_none();
    SHADOW_PASS.with(|c| c.set(!ground));
    let mut list: Vec<(f32, Drawable)> = Vec::new();
    for (i, p) in w.room.props.iter().enumerate() {
        list.push((p.rect.bottom(), Drawable::Pillar(i)));
    }
    for (i, p) in w.pickups.iter().enumerate() {
        list.push((p.pos.y + 4.0, Drawable::Pickup(i)));
        if ground {
            draw_ellipse(p.pos.x, p.pos.y + 4.0, 5.0, 1.8, 0.0, SHADOW_SOFT);
        }
    }
    for (i, l) in w.loot.iter().enumerate() {
        if !ground {
            continue;
        }
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
        if ground {
            draw_ellipse(q.x, q.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
        }
    }
    for (i, n) in w.neighbors.iter().enumerate() {
        let big = n.who == world::Who::Bori;
        if n.who == world::Who::Momo {
            // 모모는 빈 자리 그루터기 위에 앉는다: 그루터기(칸 아래 끝) 뒤에 그린다, 그림자 없이.
            list.push((n.pos.y + 9.0, Drawable::Neighbor(i)));
            continue;
        }
        // 보리(32px 그림)는 발이 그림 맨 아래 줄(pos.y + 7)이고 몸이 발보다 넓다: 발밑 그림자를 발 줄에 크게 두어야
        // 몸 밖으로 보인다(y + 6, 가로 12였을 때는 몸에 거의 다 가려 그림자가 없는 것처럼 보였다).
        let (foot, shadow_y, rx, ry): (f32, f32, f32, f32) = if big {
            (7.0, 7.5, 14.0, 3.0)
        } else {
            (6.0, 6.0, 6.0, 2.5)
        };
        list.push((n.pos.y + foot, Drawable::Neighbor(i)));
        if ground {
            draw_ellipse(n.pos.x, n.pos.y + shadow_y, rx, ry, 0.0, SHADOW_SOFT);
        }
    }
    if let Some(v) = &w.adv.village {
        for (c, at) in &w.adv.marks {
            match c {
                'Y' => {
                    list.push((at.y + 8.0, Drawable::Tree(*at, v.growth)));
                    // 깨울 재주가 있으면 밑동이 빛난다(그림자처럼 먼저, 0061).
                    if v.tree_ready && ground {
                        draw_tree_glow(*at + vec2(0.0, 14.0), w.time);
                    }
                }
                // 물웅덩이는 바닥: 맨 먼저.
                'W' if ground => draw_puddle(*at, w.time),
                'X' if v.halmae => list.push((at.y + 6.0, Drawable::DreamLoop(*at))),
                '1'..='9' => {
                    let i = (*c as u8 - b'1') as usize;
                    if v.decor[i] {
                        list.push((at.y + 6.0, Drawable::Decor(i, *at)));
                    }
                }
                _ => {}
            }
        }
    }
    for (i, e) in w.enemies.iter().enumerate() {
        let ep = lerp(e.prev_pos, e.pos, alpha);
        if !e.hittable() {
            // 땅속: 그림자 없이 둔덕만.
            if ground {
                list.push((ep.y + e.radius(), Drawable::Enemy(i)));
            }
            continue;
        }
        list.push((ep.y + e.radius(), Drawable::Enemy(i)));
        if !ground {
            continue;
        }
        draw_ellipse(
            ep.x,
            ep.y + e.radius(),
            e.radius() + 1.0,
            2.5,
            0.0,
            SHADOW_SOFT,
        );
    }
    // 묻힌 도토리는 바닥에(세워 그리는 것들보다 먼저).
    for m in w.mines.iter().filter(|_| ground) {
        draw_mine(m, w.time);
    }
    for c in w.clouds.iter().filter(|_| ground) {
        draw_cloud(c, w.time);
    }
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    for c in w.room.thorns.iter().filter(|_| ground) {
        let near = 1.0 - ((*c - pp).length() - 14.0).clamp(0.0, 30.0) / 30.0;
        draw_thorn(*c, w.time, near);
    }
    for (i, p) in w.projectiles.iter().enumerate().filter(|_| ground) {
        let q = lerp(p.prev_pos, p.pos, alpha);
        list.push((q.y + 3.0, Drawable::Projectile(i)));
        draw_ellipse(q.x, q.y + 6.0, 2.5, 1.2, 0.0, SHADOW_SOFT);
    }
    let standing = !matches!(
        w.player.state,
        player::State::Burrow { .. } | player::State::Dead
    );
    if standing && ground {
        draw_ellipse(pp.x, pp.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
        draw_player_halo(pp, w.time);
    }
    if !ground {
        // 해 그림자: 땅속 플레이어(둔덕)·줍는 것은 그림자를 드리우지 않는다. 모닥불은 돌만(아래).
        list.retain(|(_, d)| match d {
            Drawable::Player => standing,
            Drawable::Pickup(_) => false,
            _ => true,
        });
    }
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (foot, d) in list {
        // 해·모닥불 그림자: 발 줄(foot)을 축으로 눕힌다. 끝에서 되돌린다. 빛이 닿지 않으면 그리지 않는다.
        if let (Some(_), Drawable::Pillar(i)) = (sun, &d) {
            let p = w.room.props[*i];
            if p.kind == PropKind::Campfire {
                // 모닥불 돌(0105): 불꽃은 빛이라 빼고, 돌만 키워 그려 돌마다 바깥쪽 그림자가 되게 한다.
                // 가로는 불 한가운데를 축으로 양옆으로, 세로는 돌무더기 뒤 끝을 축으로: 3/4 시점에서 뒤(위)로 눕는
                // 그림자는 짧게 줄어 보이므로 뒤쪽 돌은 거의 늘지 않고 앞쪽 돌일수록 앞으로 길어진다.
                let cx = p.rect.center().x;
                let back = p.rect.bottom() + 2.0 - crate::sunny::FIRE_PIT.3 as f32 + 2.0;
                let k = 1.0 + t::FIRE_PIT_SPREAD;
                let m = Mat4::from_cols(
                    vec4(k, 0.0, 0.0, 0.0),
                    vec4(0.0, k, 0.0, 0.0),
                    vec4(0.0, 0.0, 1.0, 0.0),
                    vec4(cx * (1.0 - k), back * (1.0 - k), 0.0, 1.0),
                );
                let gl = unsafe { get_internal_gl() };
                gl.quad_gl.push_model_matrix(m);
                draw_campfire_stones(art.sunny.as_ref(), p.rect);
                gl.quad_gl.pop_model_matrix();
                continue;
            }
        }
        if let Some(l) = sun {
            let at = vec2(drawable_x(w, alpha, &d), foot);
            let Some(k) = shadow_dir(at, l.fire, l.bright) else {
                continue;
            };
            unsafe { get_internal_gl() }
                .quad_gl
                .push_model_matrix(sun_shear(foot, k));
        }
        match d {
            Drawable::Pillar(i) => {
                let p = w.room.props[i];
                let r = p.rect;
                match p.kind {
                    // 들판의 나무(T): 팩이 있으면 흔들리는 둥근 나무. 충돌은 밑동 한 칸 그대로.
                    PropKind::Stump
                        if w.biome == crate::biome::Biome::Meadow && art.sunny.is_some() =>
                    {
                        if let Some(sunny) = &art.sunny {
                            let (tw, th) = art::SUNNY_TREE;
                            let f = ((w.time * 3.0 + r.x * 0.07) as u32 % 4) as f32;
                            draw_texture_ex(
                                &sunny.tree,
                                (r.center().x - tw / 2.0).round(),
                                (r.bottom() - th + 3.0).round(),
                                WHITE,
                                DrawTextureParams {
                                    source: Some(Rect::new(f * tw, 0.0, tw, th)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                    // 팩이 있으면 그루터기·바위도 Sunnyside(0066). 바위는 자리 해시로 셋 중 하나.
                    PropKind::Stump | PropKind::Rock if art.sunny.is_some() => {
                        if let Some(sunny) = &art.sunny {
                            let pick = (r.x as usize / 16 * 7 + r.y as usize / 16 * 3) % 3;
                            let spr = if p.kind == PropKind::Stump {
                                crate::sunny::STUMPS[pick % 2]
                            } else {
                                crate::sunny::BOULDERS[pick]
                            };
                            let foot = vec2(r.center().x, r.bottom() + 2.0);
                            draw_ellipse(
                                foot.x,
                                foot.y - 1.0,
                                spr.2 as f32 * 0.42,
                                2.5,
                                0.0,
                                SHADOW_SOFT,
                            );
                            crate::sunny::draw_at_foot(
                                &sunny.sheet,
                                spr,
                                foot,
                                w.biome.wall_tint(),
                            );
                        }
                    }
                    PropKind::Stump | PropKind::Rock => {
                        let col = if p.kind == PropKind::Stump { 0.0 } else { 16.0 };
                        let src = Rect::new(col, 32.0, 16.0, 32.0);
                        draw_tile(&art.tiles, src, r.x, r.bottom() - 32.0, w.biome.wall_tint());
                    }
                    PropKind::Chest { opened } => draw_chest(r, opened, w.time),
                    PropKind::Campfire => match &art.sunny {
                        Some(sunny) => draw_campfire_sunny(sunny, r, w.time),
                        None => draw_campfire(r, w.time),
                    },
                    PropKind::Stall => match &art.sunny {
                        Some(sunny) => draw_stall_sunny(sunny, r),
                        None => draw_stall(r),
                    },
                    PropKind::Seat => match &art.sunny {
                        Some(sunny) => draw_seat_sunny(sunny, r, w.momo_wins),
                        None => draw_seat(r, w.momo_wins),
                    },
                }
            }
            Drawable::Pickup(i) => draw_pickup(&w.pickups[i], w.time),
            Drawable::Loot(i) => draw_loot(&w.loot[i], alpha, w.time, false),
            Drawable::Player => draw_nail(art, &w.player, w.fur, alpha, w.time),
            Drawable::Enemy(i) => draw_enemy(Some(art), &w.enemies[i], alpha, w.time),
            Drawable::Projectile(i) => draw_projectile(&w.projectiles[i], alpha),
            Drawable::Npc => {
                if let (Some(n), Some(tex)) = (&w.npc, art.critters.as_ref()) {
                    let q = lerp(n.prev_pos, n.pos, alpha);
                    let moving = (n.pos - n.prev_pos).length() > 0.05;
                    draw_critter_at(tex, art::RACCOON_ROW, q, n.face_left, moving, w.time);
                }
            }
            Drawable::Neighbor(i) => {
                let n = &w.neighbors[i];
                let left = (w.player.pos.x < n.pos.x) != n.face_left;
                match n.who {
                    world::Who::Popo => {
                        if let Some(tex) = art.critters.as_ref() {
                            draw_critter_at(tex, art::RACCOON_ROW, n.pos, left, false, w.time);
                        }
                    }
                    world::Who::Bori => draw_bori_at(art, n.pos, left, false, w.time),
                    world::Who::Halmae => draw_halmae_at(n.pos, w.time),
                    world::Who::Momo => {
                        if let Some(tex) = art.critters.as_ref() {
                            let on_seat = n.pos - vec2(0.0, 7.0);
                            draw_critter_at(tex, art::MOMO_BLOOM_ROW, on_seat, left, false, w.time);
                        }
                    }
                    world::Who::Doto => {
                        if let Some(tex) = art.critters.as_ref() {
                            draw_critter_at(tex, art::DOTO_BACK_ROW, n.pos, left, false, w.time);
                        }
                    }
                    world::Who::Susu => {
                        if let Some(tex) = art.critters.as_ref() {
                            draw_critter_at(tex, art::SUSU_ROW, n.pos, left, false, w.time);
                        }
                    }
                    world::Who::Kongi => {
                        if let Some(tex) = art.critters.as_ref() {
                            draw_critter_at(tex, art::KONGI_BACK_ROW, n.pos, left, false, w.time);
                        }
                    }
                }
            }
            Drawable::Tree(at, growth) => {
                match art.sunny.as_ref() {
                    Some(sunny) if growth > 0 => draw_great_tree_sunny(sunny, at, growth, w.time),
                    _ => draw_great_tree(at, growth, w.time),
                }
                if w.adv.village.is_some_and(|v| v.tree_ready) && !shadow_pass() {
                    draw_tree_motes(at + vec2(0.0, 14.0), w.time);
                }
            }
            Drawable::Decor(i, at) => match &art.sunny {
                Some(sunny) if matches!(i, 0 | 2 | 3 | 7) => draw_decor_sunny(sunny, i, at, w.time),
                _ => draw_decor(i, at, w.time),
            },
            Drawable::DreamLoop(at) => draw_dream_loop(at, w.time),
        }
        if sun.is_some() {
            unsafe { get_internal_gl() }.quad_gl.pop_model_matrix();
        }
    }
    SHADOW_PASS.with(|c| c.set(false));
}

thread_local! {
    /// 그림자 패스 중인가(0106): 그리기 함수 안의 빛·표시(거품·별·고리·Z·체력 점·빛 알갱이)는 그림자를 드리우지 않는다.
    static SHADOW_PASS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// 지금 그림자 모양만 모으는 중이면 참. 빛·표시·바닥에 깔린 것은 이때 그리지 않는다.
fn shadow_pass() -> bool {
    SHADOW_PASS.with(|c| c.get())
}

/// 그림자 패스의 빛(0103): 가장 가까운 모닥불 자리(불 밑동)와 밝은 곳인지.
#[derive(Clone, Copy)]
struct ShadowLight {
    fire: Option<Vec2>,
    bright: bool,
}

/// 장소의 모닥불 자리(불 밑동, 빛 자리와 같다).
pub fn campfires(w: &World) -> Vec<Vec2> {
    w.room
        .props
        .iter()
        .filter(|p| p.kind == PropKind::Campfire)
        .map(|p| vec2(p.rect.center().x, p.rect.bottom() - 6.0))
        .collect()
}

/// 발이 `foot`인 것의 그림자 방향·길이(높이 1px당 바닥 위 px, 0102·0103). 빛이 닿지 않으면 None.
/// 모닥불 곁은 불에서 바깥쪽으로, 멀수록 길게. 옆으로 눕는 그림자는 납작한 줄이 되지 않게 뒤로 조금 두께를 준다.
/// 밝은 곳은 모닥불에서 멀어질수록 해 방향으로 부드럽게 돌아간다. 실시간 그림자와 지도 굽기가 같이 쓴다.
pub fn shadow_dir(foot: Vec2, fire: Option<Vec2>, bright: bool) -> Option<Vec2> {
    let sun = vec2(t::SUN_SHADOW_X, t::SUN_SHADOW_Y);
    let Some(f) = fire else {
        return bright.then_some(sun);
    };
    let reach = t::FIRE_SHADOW_REACH;
    let d = foot - f;
    let dist = d.length();
    if !bright && dist > reach {
        return None;
    }
    let dir = if dist < 1.0 { vec2(0.0, 1.0) } else { d / dist };
    let len = t::FIRE_SHADOW_MIN + t::FIRE_SHADOW_GROW * (dist / reach).min(1.0);
    // 불이 아래(앞)에 있으면 그림자가 뒤(위)로 눕는다: 몸에 가려지는 만큼 더 길게(0104).
    let len = len * (1.0 + t::FIRE_SHADOW_BACK * (-dir.y).max(0.0));
    let radial = dir * len - vec2(0.0, t::FIRE_SHADOW_THICK * (1.0 - dir.y.abs()));
    if !bright {
        return Some(radial);
    }
    let u = ((dist - 0.6 * reach) / (0.4 * reach)).clamp(0.0, 1.0);
    let u = u * u * (3.0 - 2.0 * u);
    Some(radial.lerp(sun, u))
}

/// 정렬 그리기 한 개의 발 x(그림자 패스가 모닥불까지 거리를 잴 때).
fn drawable_x(w: &World, alpha: f32, d: &Drawable) -> f32 {
    match *d {
        Drawable::Pillar(i) => w.room.props[i].rect.center().x,
        Drawable::Pickup(i) => w.pickups[i].pos.x,
        Drawable::Loot(i) => lerp(w.loot[i].prev_pos, w.loot[i].pos, alpha).x,
        Drawable::Player => lerp(w.player.prev_pos, w.player.pos, alpha).x,
        Drawable::Enemy(i) => lerp(w.enemies[i].prev_pos, w.enemies[i].pos, alpha).x,
        Drawable::Projectile(i) => lerp(w.projectiles[i].prev_pos, w.projectiles[i].pos, alpha).x,
        Drawable::Npc => w
            .npc
            .as_ref()
            .map_or(0.0, |n| lerp(n.prev_pos, n.pos, alpha).x),
        Drawable::Neighbor(i) => w.neighbors[i].pos.x,
        Drawable::Tree(at, _) | Drawable::Decor(_, at) | Drawable::DreamLoop(at) => at.x,
    }
}

/// 해 그림자 행렬(0102): 발 줄 y = `foot`는 그대로, 그 위로 h만큼 높은 점은 바닥 위 `k * h`만큼 떨어진 자리로.
/// x' = x + k.x·(foot − y), y' = foot + k.y·(foot − y). k.y > 0이면 그림자가 앞(아래)으로 눕는다.
fn sun_shear(foot: f32, k: Vec2) -> Mat4 {
    Mat4::from_cols(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(-k.x, -k.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(k.x * foot, (1.0 + k.y) * foot, 0.0, 1.0),
    )
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

/// 먹는 중: 입 앞에 든 산딸기(베어 물수록 작아진다)·지렁이 + 머리 위 작은 진행 막대.
/// 회복은 다 먹어야 된다는 걸 보여 준다(0038). 맞으면 상태가 바뀌어 사라진다.
fn draw_eating(pos: Vec2, face_left: bool, item: crate::items::Consumable, k: f32) {
    let side = if face_left { -1.0 } else { 1.0 };
    let bob = if ((k * 8.0) as i32) % 2 == 0 {
        0.0
    } else {
        1.0
    };
    let m = vec2(pos.x + side * 6.0, pos.y - 1.0 + bob);
    match item {
        crate::items::Consumable::Berry => {
            // 세 입: 3 → 2 → 1칸 너비.
            let wpx = 3.0 - (k * 3.0).floor().min(2.0);
            // 입 쪽부터 베어 문다.
            let x = if face_left {
                m.x - 2.0
            } else {
                m.x - 1.0 + (3.0 - wpx)
            };
            draw_rectangle(x - 1.0, m.y - 2.0, wpx + 2.0, 5.0, OUTLINE);
            draw_rectangle(x, m.y - 1.0, wpx, 3.0, world::BERRY);
        }
        crate::items::Consumable::Worm => {
            draw_worm(m + vec2(side * 1.0, 0.0), k * 20.0, 0.6);
        }
    }
    let (bw, y) = (10.0, pos.y - 14.0);
    draw_rectangle(pos.x - bw / 2.0 - 1.0, y - 1.0, bw + 2.0, 3.0, OUTLINE);
    draw_rectangle(pos.x - bw / 2.0, y, (bw * k).round(), 1.0, world::BERRY);
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
/// 모모를 플레이어로(0056): 돌아온 모모 줄(critters MOMO_BLOOM_ROW). 칸 0·1 서기, 2·3 걷기, 4 웅크림, 5 차기·뛰기, 6 맞음.
/// 뛸 때는 그림만 높이만큼 위로(그림자는 바닥에 남아 착지 자리를 보인다).
fn draw_momo_player(art: &Art, p: &Player, alpha: f32, time: f32) {
    let Some(tex) = art.critters.as_ref() else {
        return;
    };
    let pos = lerp(p.prev_pos, p.pos, alpha) - vec2(0.0, p.height().round());
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    let frame = match p.state {
        player::State::Dead => 6,
        player::State::Hop { time: ht, .. } | player::State::Chain { time: ht, .. } => {
            if ht < 0.05 {
                4
            } else {
                5
            }
        }
        player::State::Attack { .. } => 5,
        player::State::Recover { .. } => 4,
        _ if p.invuln > t::PLAYER_HURT_INVULN - 0.25 => 6,
        _ if moving => 2 + ((time * 10.0) as u32) % 2,
        _ => ((time * 2.0) as u32) % 2,
    };
    let flash = p.flash > 0.0;
    let row = art::MOMO_BLOOM_ROW + if flash { art::CRITTER_ROWS } else { 0 };
    let blink = p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && !flash;
    let tint = if blink {
        Color::new(1.0, 1.0, 1.0, 0.35)
    } else {
        WHITE
    };
    draw_texture_ex(
        tex,
        (pos.x - 8.0).round(),
        (pos.y - 9.0).round(),
        tint,
        DrawTextureParams {
            source: Some(art::cell(frame, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: p.face_left,
            ..Default::default()
        },
    );
}

fn draw_nail(art: &Art, p: &Player, fur: u8, alpha: f32, time: f32) {
    if p.kit == crate::equipment::Character::Momo {
        draw_momo_player(art, p, alpha, time);
        return;
    }
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    let (col, row) = match p.state {
        player::State::Dead => (4, 1),
        player::State::Burrow { time: bt } => (((bt * 12.0) as u32) % 2, 2),
        player::State::Emerge { .. } | player::State::Drill { .. } => (2, 1),
        player::State::Recover { .. } => (3, 1),
        // 한 타 = 들기 → 치기 → (마지막 타) 거두기. 1·2타는 가까운 발·먼 발을 번갈아, 마지막 타는 두 발(0082).
        player::State::Attack { idx, time: at, .. } => {
            let (a0, a1) = p.moveset.hits[idx].active;
            let pose = if idx == p.moveset.last() {
                2
            } else {
                (idx % 2) as u32
            };
            let col = if at < a0 {
                5 + pose
            } else if pose == 2 && at > a1 + 0.04 {
                8
            } else {
                pose
            };
            (col, 1)
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
        art.nail_fur(fur),
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
    draw_dark_extras(art, w, alpha, cam);
}

/// 어둠 위에 한 번 더 그리는 것: 반딧불과 빛나는 버섯(어둠에 묻히지 않게).
fn draw_dark_extras(art: &Art, w: &World, alpha: f32, cam: Vec2) {
    for l in &w.loot {
        draw_loot(l, alpha, w.time, true);
    }
    draw_mushrooms(art, w, cam);
}

/// 빛나는 버섯. 어둠 지도(0095)가 있으면 바닥 높이(캐릭터 밑)에 그리고 버섯 빛이 둘레를 밝힌다.
/// 없으면 예전처럼 어둠 위에 한 번 더 그린다(그때는 캐릭터 위에 겹칠 수 있다).
fn draw_mushrooms(art: &Art, w: &World, cam: Vec2) {
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.65 + 0.25 * ((w.time * 2.0 + x * 0.1).sin());
            // 팩이 있으면 파란 버섯 4칸(0066). 굽기에서는 빼 두었다.
            if let Some(sunny) = &art.sunny {
                let s = crate::sunny::MUSH_FRAME;
                let f = ((w.time * 4.0 + x * 0.13) as u32 % 4) as f32;
                for r in [13.0, 9.0, 5.0] {
                    draw_circle(
                        x + 8.0,
                        y + 10.0,
                        r,
                        Color::new(0.5, 0.85, 1.0, 0.035 * pulse),
                    );
                }
                draw_texture_ex(
                    &sunny.mush,
                    x,
                    y,
                    Color::new(1.0, 1.0, 1.0, 0.75 + 0.25 * pulse),
                    DrawTextureParams {
                        source: Some(Rect::new(f * s, 0.0, s, s)),
                        ..Default::default()
                    },
                );
                continue;
            }
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
    if let enemy::State::Under { time: ut } = e.state {
        draw_dig_mound(pos, ut, e.windup(), time);
        return;
    }
    let base = sp.color;
    let mut tint = WHITE;
    // 그림자 패스(0106)에서는 몸만: 거품·별·잔상·고리·Z·체력 점은 빛·표시라 그림자를 드리우지 않는다.
    let marks = !shadow_pass();
    if e.poison > 0.0 && e.flash <= 0.0 {
        // 독: 초록빛이 돌고 거품이 오른다.
        tint = Color::new(0.7, 1.0, 0.6, 1.0);
        for k in (0..2).filter(|_| marks) {
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
            let k = (wt / e.windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.45 * k, 1.0 - 0.45 * k, 1.0);
            if matches!(e.state, enemy::State::Aim { .. })
                && !matches!(
                    sp.behavior,
                    crate::species::Behavior::Tongue | crate::species::Behavior::Spore
                )
            {
                // 도토리를 쳐든다.
                let hand = pos + e.facing * (r + 1.0) - vec2(0.0, 4.0);
                draw_acorn(hand, 0.0);
            }
        }
        enemy::State::Crouch { time: ct, .. } => {
            frame = 4;
            let k = (ct / e.windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.4 * k, 1.0 - 0.4 * k, 1.0);
        }
        enemy::State::Leap { .. } => {
            frame = 5;
            pos.y -= e.height();
        }
        enemy::State::Pop { time: pt } => {
            // 튀어나옴: 살짝 솟았다가 내려온다.
            frame = 5;
            let k = (pt / t::MOMO_POP_TIME).clamp(0.0, 1.0);
            pos.y -= (4.0 * 6.0 * k * (1.0 - k)).round();
        }
        enemy::State::Rest { .. } if sp.behavior != crate::species::Behavior::Dummy => {
            tint = Color::new(0.75, 0.75, 0.75, 1.0);
        }
        enemy::State::Stunned { time: st } => {
            frame = 6;
            // 머리 위에 노란 점이 돈다.
            let a = st * 12.0;
            let s = pos + vec2(a.cos(), a.sin() * 0.5) * (r + 3.0) - vec2(0.0, r + 4.0);
            if marks {
                draw_circle(s.x.round(), s.y.round(), 1.0, YELLOW);
            }
        }
        enemy::State::Charge { dir, .. } => {
            frame = 5;
            // 잔상
            for i in (1..4).filter(|_| marks) {
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
    if let enemy::State::Howl { time: ht } = e.state {
        if e.kind == crate::enemy::Kind::Doto {
            // 도토: 땅을 판다(웅크린 칸이 떨고, 앞에 흙이 튄다). 울음 고리는 없다.
            frame = 4;
            pos.x += if ((time * 30.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let ph = (ht * 4.0).fract();
            let front = pos + e.facing * (r + 2.0);
            draw_rectangle(
                (front.x + (ph * 7.0).sin() * 3.0).round(),
                (front.y - 2.0 - ph * 6.0).round(),
                1.0,
                1.0,
                world::DIRT,
            );
        } else {
            frame = 7;
        }
    }
    if let (enemy::State::Howl { time: ht }, false) = (e.state, e.kind == crate::enemy::Kind::Doto)
    {
        // 울음: 퍼지는 고리 둘.
        for k in (0..2).filter(|_| marks) {
            let ph = ((ht * 2.5) + k as f32 * 0.5).fract();
            draw_circle_lines(
                pos.x,
                pos.y - 4.0,
                r + 4.0 + ph * 22.0,
                1.0,
                Color::new(1.0, 0.85, 0.8, 0.6 * (1.0 - ph)),
            );
        }
    }
    // 콩이(0054): 말려 있으면 공(칸 4·5). 구를 때는 두 칸을 번갈아 띠가 돈다. 버틸 때는 등딱지에 빛이 돈다.
    if e.kind == crate::enemy::Kind::Kongi {
        match e.state {
            enemy::State::Charge { .. } => frame = 4 + ((time * 14.0) as u32) % 2,
            enemy::State::Windup { .. } => frame = 4,
            enemy::State::Guard { .. } if marks => {
                frame = 4;
                let ph = (time * 2.5).fract();
                let a = std::f32::consts::PI * (1.1 + ph * 0.8);
                let g = pos + vec2(a.cos(), a.sin() * 0.8) * (r - 1.0) - vec2(0.0, 2.0);
                draw_rectangle(g.x.round(), g.y.round(), 1.0, 1.0, world::SHELL);
                draw_circle_lines(
                    pos.x.round(),
                    (pos.y - 2.0).round(),
                    r + 3.0,
                    1.0,
                    Color::new(0.75, 0.90, 1.0, 0.35 + 0.25 * (time * 6.0).sin().abs()),
                );
            }
            enemy::State::Guard { .. } => frame = 4,
            _ => {}
        }
    }
    // 새 졸개(0146): 고슴도치는 콩이처럼 공(칸 4·5), 혀 내밀기·포자 뿜기는 칸 5.
    match (sp.behavior, e.state) {
        (crate::species::Behavior::Roller, enemy::State::Charge { .. }) => {
            frame = 4 + ((time * 14.0) as u32) % 2
        }
        (crate::species::Behavior::Roller, enemy::State::Windup { .. }) => frame = 4,
        (_, enemy::State::Lash { .. } | enemy::State::Puff { .. }) => frame = 5,
        _ => {}
    }
    if e.flying() {
        // 박쥐: 예고·내리꽂기·맞음이 아니면 늘 날갯짓(칸 0·1). 높이만큼 떠서 살랑인다(그림자는 바닥에 남는다).
        if !matches!(
            e.state,
            enemy::State::Windup { .. }
                | enemy::State::Charge { .. }
                | enemy::State::Stunned { .. }
        ) {
            frame = ((time * 10.0 + e.pos.x * 0.03) as u32) % 2;
        }
        pos.y -= (e.height() + (time * 5.0 + e.pos.x * 0.02).sin()).round();
    }
    if e.asleep {
        frame = 1;
        // 자는 중: Z가 떠오른다.
        let ph = (time * 0.8 + e.pos.x * 0.013).fract();
        let z = pos + vec2(4.0 + ph * 3.0, -r - 6.0 - ph * 6.0);
        let c = Color::new(0.9, 0.9, 1.0, 1.0 - ph);
        if marks {
            draw_line(z.x, z.y, z.x + 3.0, z.y, 1.0, c);
            draw_line(z.x + 3.0, z.y, z.x, z.y + 3.0, 1.0, c);
            draw_line(z.x, z.y + 3.0, z.x + 3.0, z.y + 3.0, 1.0, c);
        }
    }
    // 엘리트(0147): 발밑에 보라 실 고리가 숨 쉬고, 실 매듭 셋이 돈다.
    if e.elite && marks {
        let ph = time * 3.0 + e.pos.x * 0.05;
        draw_circle_lines(
            pos.x.round(),
            (pos.y + r * 0.4).round(),
            r + 3.0 + ph.sin() * 0.8,
            1.0,
            Color::new(0.75, 0.50, 0.95, 0.55),
        );
        for i in 0..3 {
            let a = ph * 0.8 + i as f32 * std::f32::consts::TAU / 3.0;
            let q = pos + vec2(a.cos() * (r + 3.0), a.sin() * (r + 3.0) * 0.6 - 2.0);
            draw_rectangle(
                q.x.round(),
                q.y.round(),
                1.0,
                1.0,
                Color::new(0.85, 0.65, 1.0, 0.9),
            );
        }
    }
    let flash = e.flash > 0.0;
    let chief = e.kind == crate::enemy::Kind::Chief;
    if let (true, Some(tex)) = (chief, art.and_then(|a| a.chief.as_ref())) {
        // 대장: 32×32(발바닥 y=30).
        let row = art::CHIEF_TAINTED + if flash { art::CHIEF_ROWS } else { 0 };
        draw_texture_ex(
            tex,
            (pos.x - 16.0).round(),
            (pos.y + r - 30.0).round(),
            if flash { WHITE } else { tint },
            DrawTextureParams {
                source: Some(art::chief_cell(frame, row)),
                dest_size: Some(vec2(32.0, 32.0)),
                flip_x: e.facing.x < 0.0,
                ..Default::default()
            },
        );
        return;
    }
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
    // 엘리트는 처음부터 작은 막대(점이 너무 많다).
    if marks && e.elite {
        let (w, y) = (18.0, (pos.y - r - 9.0).round());
        let x = (pos.x - w / 2.0).round();
        draw_rectangle(x - 1.0, y - 1.0, w + 2.0, 4.0, OUTLINE);
        draw_rectangle(x, y, w, 2.0, DARKGRAY);
        let k = (e.hp.max(0) as f32 / e.max_hp.max(1) as f32).clamp(0.0, 1.0);
        draw_rectangle(x, y, (w * k).round(), 2.0, Color::new(0.80, 0.58, 1.0, 1.0));
    }
    // 체력 점(맞은 적만: 처음부터 숫자를 늘어놓지 않는다). 보스는 화면 아래 막대.
    if marks
        && !e.is_boss()
        && !e.elite
        && (e.hp < e.max_hp || sp.behavior == crate::species::Behavior::Dummy)
    {
        let max = e.max_hp;
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

/// 내 캐릭터 표시: 발밑의 따뜻한 빛 고리(색이 아니라 모양으로도 구별되게). 바닥에 그려 다른 그림을 가리지 않는다.
/// 적이 많을 때 Naïl을 놓치지 않게(사람 피드백 2026-10-25). 아주 천천히 숨 쉬듯 밝아졌다 어두워진다.
fn draw_player_halo(p: Vec2, time: f32) {
    let pulse = 0.5 + 0.5 * (time * 2.2).sin();
    let a = 0.45 + 0.2 * pulse;
    draw_ellipse(
        p.x,
        p.y + 6.0,
        9.0,
        3.5,
        0.0,
        Color::new(1.0, 0.92, 0.65, 0.10 + 0.05 * pulse),
    );
    draw_ellipse_lines(
        p.x,
        p.y + 6.0,
        9.0,
        3.5,
        0.0,
        1.0,
        Color::new(1.0, 0.9, 0.6, a),
    );
}

/// 공격 예고의 불투명도(시작, 공격 직전). 채움은 바닥이 비치게 옅게, 테두리가 주로 읽힌다.
/// 테두리는 밝은 산호색 + 하한 0.3(0150): 1·2형 색약·흑백 시뮬레이션에서 붉은 테두리(0.15~0.5)가 어두운 바닥에 묻혔다.
/// 색이 아니라 밝기로 읽히게 올리고, 채움은 그대로 옅게 둔다("너무 샛빨갛다", 2026-10-25).
const TELEGRAPH_FILL: (f32, f32) = (0.03, 0.16);
const TELEGRAPH_EDGE: (f32, f32) = (0.3, 0.65);
const TELEGRAPH_EDGE_RGB: (f32, f32, f32) = (1.0, 0.55, 0.50);
/// 마지막 깜빡임은 거의 흰빛.
const TELEGRAPH_BLINK: Color = Color::new(1.0, 0.85, 0.80, 0.85);

/// 공격 예고(바닥, 적·플레이어보다 먼저 그린다): 맞을 자리를 붉게, 공격 순간이 가까울수록 진하게.
/// - 돌격형(떨기): 돌진할 길(몸 폭 × 돌진 거리, 벽에서 끊김)을 띠로. 방향은 지금 보는 쪽(떨기 끝에 굳는다).
/// - 던지기(조준): 도토리가 날아갈 줄.
/// - 뛰기(웅크림·체공): 내려앉을 자리 고리, 체공 중엔 가득.
///
/// 진하기 k = 예고 경과 / 예고 시간(0→1). 마지막 15%는 깜빡여 "지금"을 알린다.
fn draw_telegraphs(w: &World, alpha: f32) {
    use crate::species::Behavior;
    // 무너지는 천장(0147): 떨어질 자리에 그림자가 커지고, 붉은 고리가 조여 든다(청설모 내려앉기와 같은 말).
    for r in &w.rocks {
        let k = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        let rad = t::ROCK_R;
        draw_ellipse(
            r.pos.x,
            r.pos.y,
            rad * (0.35 + 0.65 * k),
            rad * (0.35 + 0.65 * k) * 0.7,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.18 + 0.27 * k),
        );
        let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
        let edge = if blink {
            TELEGRAPH_BLINK
        } else {
            Color::new(
                TELEGRAPH_EDGE_RGB.0,
                TELEGRAPH_EDGE_RGB.1,
                TELEGRAPH_EDGE_RGB.2,
                TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k,
            )
        };
        draw_circle_lines(r.pos.x, r.pos.y, rad * (1.5 - 0.5 * k), 1.0, edge);
    }
    for e in w.enemies.iter().filter(|e| e.alive && !e.asleep) {
        let sp = e.sp();
        let pos = lerp(e.prev_pos, e.pos, alpha);
        let (k, kind) = match e.state {
            enemy::State::Windup { time }
                if matches!(
                    sp.behavior,
                    Behavior::Charger
                        | Behavior::Chief
                        | Behavior::Kongi
                        | Behavior::Darter
                        | Behavior::Roller
                        | Behavior::Swooper
                ) =>
            {
                ((time / e.windup()).min(1.0), 0)
            }
            enemy::State::Lash { time, dir } => {
                draw_tongue(w, pos, e.radius(), dir, time);
                continue;
            }
            enemy::State::Aim { time } if sp.behavior == Behavior::Spore => {
                ((time / e.windup()).min(1.0), 3)
            }
            enemy::State::Aim { time } => ((time / e.windup()).min(1.0), 1),
            enemy::State::Crouch { time, .. } => ((time / e.windup()).min(1.0), 2),
            enemy::State::Leap { .. } => (1.0, 2),
            _ => continue,
        };
        // 투명하게: 바닥이 비쳐 보여야 화면이 붉게 덮이지 않는다(사람 피드백 "너무 샛빨갛다", 2026-10-25).
        // 채움은 아주 옅게(최대 16%), 테두리가 주로 읽힌다. 깜빡임도 흰 번쩍이 아니라 조금 더 진한 붉은빛.
        let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
        let edge_a = TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k;
        let edge = if blink {
            TELEGRAPH_BLINK
        } else {
            Color::new(
                TELEGRAPH_EDGE_RGB.0,
                TELEGRAPH_EDGE_RGB.1,
                TELEGRAPH_EDGE_RGB.2,
                edge_a,
            )
        };
        let fill = Color::new(
            0.95,
            0.35,
            0.35,
            TELEGRAPH_FILL.0 + (TELEGRAPH_FILL.1 - TELEGRAPH_FILL.0) * k,
        );
        match kind {
            0 => {
                let dir = e.facing.normalize_or(vec2(1.0, 0.0));
                let r = e.radius();
                let len = w.room.ray_len(pos, dir, r, sp.dash_speed * sp.dash_time);
                let side = vec2(-dir.y, dir.x) * r;
                let (a, b) = (pos + dir * r, pos + dir * (len + r));
                draw_triangle(a + side, a - side, b + side, fill);
                draw_triangle(a - side, b - side, b + side, fill);
                for s in [side, -side] {
                    draw_line(a.x + s.x, a.y + s.y, b.x + s.x, b.y + s.y, 1.0, edge);
                }
                // 끝 화살촉
                let tip = b + dir * 3.0;
                draw_line(b.x + side.x, b.y + side.y, tip.x, tip.y, 1.0, edge);
                draw_line(b.x - side.x, b.y - side.y, tip.x, tip.y, 1.0, edge);
            }
            3 => {
                // 포자버섯: 구름이 덮을 자리. 고리가 안에서 밖으로 부푼다.
                let r = t::SPORE_CLOUD_R;
                draw_circle(pos.x, pos.y, r, fill);
                draw_circle_lines(pos.x, pos.y, r * (0.5 + 0.5 * k), 1.0, edge);
            }
            1 => {
                let dir = e.facing.normalize_or(vec2(1.0, 0.0));
                let start = pos + dir * (e.radius() + 2.0);
                // 두꺼비 혀는 닿는 데까지만, 도토리는 멀리.
                let reach = if sp.behavior == Behavior::Tongue {
                    t::TOAD_TONGUE_REACH
                } else {
                    150.0
                };
                let len = w.room.ray_len(start, dir, t::PROJECTILE_RADIUS, reach);
                // 점선: 3px 칠하고 2px 비움, 멀수록 옅게.
                let mut d = 0.0;
                while d < len {
                    let p0 = start + dir * d;
                    let p1 = start + dir * (d + 3.0).min(len);
                    // 줄은 1px라 띠보다 잘 안 보인다: 처음부터 조금 진하게.
                    let fade = 1.0 - d / 150.0 * 0.5;
                    let c = Color::new(edge.r, edge.g, edge.b, edge.a.max(0.3) * fade);
                    draw_line(p0.x, p0.y, p1.x, p1.y, 1.0, c);
                    d += 5.0;
                }
            }
            _ => {
                let at = match e.state {
                    enemy::State::Crouch { target, .. } => {
                        e.pos + (target - e.pos).clamp_length_max(sp.trigger * 1.2)
                    }
                    enemy::State::Leap { to, .. } => to,
                    _ => pos,
                };
                let r = sp.land_radius;
                draw_circle(at.x, at.y, r, fill);
                // 고리가 바깥에서 조여 들어 내려앉을 때 딱 맞는다.
                draw_circle_lines(at.x, at.y, r * (1.5 - 0.5 * k), 1.0, edge);
                let inner = Color::new(edge.r, edge.g, edge.b, edge.a * 0.5);
                draw_circle_lines(at.x, at.y, r, 1.0, inner);
            }
        }
    }
}

/// 가시뿌리 덤불(0147·0149): 칸 가운데 낮은 덤불, 붉게 물든 가시가 사방으로 뻗는다. 발밑에 어두운 핏빛 얼룩이 숨 쉬고,
/// Naïl이 다가오면(`near` 0..1) 가시가 곤두서 길어진다. 끝이 가끔 하얗게 번득인다.
/// 예전(보라 가시 2px)은 "눈에 잘 띄지 않고 위험한 느낌이 없어"(사장).
fn draw_thorn(c: Vec2, time: f32, near: f32) {
    let seed = ((c.x * 7.0 + c.y * 13.0) as i32).rem_euclid(97) as f32;
    let pulse = 0.5 + 0.5 * (time * 3.0 + seed).sin();
    draw_ellipse(
        c.x.round(),
        (c.y + 1.0).round(),
        9.0,
        6.0,
        0.0,
        Color::new(0.30, 0.02, 0.08, 0.30 + 0.15 * pulse + 0.15 * near),
    );
    let dark = Color::new(0.16, 0.05, 0.10, 1.0);
    let body = Color::new(0.34, 0.12, 0.20, 1.0);
    // 가시는 밝게(0150): 색약 시뮬레이션(1·2형)에서 붉은 가시가 몸과 같은 어둠으로 묻혔다. 밝기 차로 "뾰족한 것"이 읽히게.
    let spike = Color::new(1.0, 0.80, 0.84, 1.0);
    for (i, (dx, dy)) in [
        (-3.0, 1.0),
        (3.0, 1.5),
        (0.0, -1.5),
        (-1.0, 3.0),
        (2.0, -3.0),
    ]
    .into_iter()
    .enumerate()
    {
        let r = 2.5 + ((seed + i as f32 * 3.0) % 3.0) * 0.5;
        draw_circle((c.x + dx).round(), (c.y + dy).round(), r + 1.0, dark);
        draw_circle((c.x + dx).round(), (c.y + dy - 0.5).round(), r, body);
    }
    let grow = 3.0 + 2.0 * near;
    for i in 0..10 {
        let a = (seed * 0.37 + i as f32 * 0.63) % std::f32::consts::TAU;
        let d = Vec2::from_angle(a);
        let p = (c + vec2(d.x * 4.0, d.y * 3.0 - 1.0)).round();
        let q = (p + vec2(d.x, d.y * 0.8 - 0.4) * grow).round();
        draw_line(p.x, p.y, q.x, q.y, 1.0, spike);
        // 끝 번득임: 가시마다 차례로 잠깐.
        if (time * 2.0 + i as f32 * 0.31 + seed * 0.1).fract() < 0.08 {
            draw_rectangle(q.x, q.y, 1.0, 1.0, WHITE);
        }
    }
}

/// 무너지는 돌(0147): 떨어지기 전 반쯤부터 위에서 내려오는 돌(땅 그림자·고리는 `draw_telegraphs`).
fn draw_falling_rocks(w: &World) {
    for r in &w.rocks {
        let k = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        if k < 0.45 {
            continue;
        }
        let h = (1.0 - k) * (1.0 - k) * 150.0;
        let p = vec2(r.pos.x, r.pos.y - 4.0 - h).round();
        draw_circle(p.x, p.y, 5.0, OUTLINE);
        draw_circle(p.x, p.y, 4.0, Color::new(0.46, 0.42, 0.44, 1.0));
        draw_rectangle(
            p.x - 2.0,
            p.y - 2.0,
            2.0,
            1.0,
            Color::new(0.68, 0.64, 0.64, 1.0),
        );
    }
}

/// 두꺼비 혀(0146): 맞는 순간 다 뻗었다가 말려 들어간다(맞았는지는 월드가 뻗은 틱에 본다).
fn draw_tongue(w: &World, pos: Vec2, r: f32, dir: Vec2, time: f32) {
    let start = pos + dir * r;
    let reach = w.room.ray_len(start, dir, 1.0, t::TOAD_TONGUE_REACH);
    let k = (time / t::TOAD_LASH_TIME).clamp(0.0, 1.0);
    let len = reach * (1.0 - k * k);
    let (a, b) = (
        (start - vec2(0.0, 2.0)).round(),
        (start + dir * len - vec2(0.0, 2.0)).round(),
    );
    draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(0.94, 0.50, 0.60, 1.0));
    draw_circle(b.x, b.y, 2.0, Color::new(0.98, 0.62, 0.70, 1.0));
}

/// 포자 구름(0146): 처음 잠깐(아프지 않을 때)은 옅게 부풀고, 끝에 흐려진다. 안의 점들이 천천히 돈다.
fn draw_cloud(c: &world::Cloud, time: f32) {
    let grow = (c.time / t::SPORE_CLOUD_GRACE).min(1.0);
    let k = c.time / t::SPORE_CLOUD_TIME;
    let fade = 1.0 - ((k - 0.75) / 0.25).clamp(0.0, 1.0);
    let a = fade
        * if c.time < t::SPORE_CLOUD_GRACE {
            0.5
        } else {
            1.0
        };
    let r = t::SPORE_CLOUD_R * (0.55 + 0.45 * grow);
    for i in 0..5 {
        let ang = i as f32 * std::f32::consts::TAU / 5.0 + time * 0.5;
        let p = c.pos + vec2(ang.cos(), ang.sin() * 0.7) * r * 0.45;
        draw_circle(p.x, p.y, r * 0.55, Color::new(0.66, 0.56, 0.84, 0.16 * a));
    }
    draw_circle_lines(
        c.pos.x,
        c.pos.y,
        r,
        1.0,
        Color::new(0.85, 0.75, 1.0, 0.7 * a),
    );
    for i in 0..8 {
        let ph = (time * 0.4 + i as f32 * 0.37).fract();
        let ang = i as f32 * 2.4 + time * 0.8;
        let p = c.pos + vec2(ang.cos(), ang.sin() * 0.7) * r * (0.2 + 0.7 * ph);
        draw_rectangle(
            p.x.round(),
            (p.y - ph * 4.0).round(),
            1.0,
            1.0,
            Color::new(0.90, 0.85, 1.0, 0.7 * a * (1.0 - ph)),
        );
    }
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
            // 먼지·연기: 정수 반지름, 천천히 옅어진다(분위기 효과는 대비를 낮게, 0081).
            FxKind::Puff { r0, r1, color } => {
                let r = (r0 + (r1 - r0) * p).round().max(1.0);
                let a = 0.55 * (1.0 - p) * (1.0 - p);
                draw_circle(pos.x, pos.y, r, Color::new(color.r, color.g, color.b, a));
            }
            // 흙덩이: 바닥 그림자 + 높이만큼 떠 있는 2px 덩이(아래 한 점 어둡게).
            FxKind::Clod { color, z, .. } => {
                let a = if p > 0.8 { (1.0 - p) * 5.0 } else { 1.0 };
                draw_rectangle(
                    pos.x,
                    pos.y + 1.0,
                    2.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.25 * a),
                );
                let y = (pos.y - z).round();
                draw_rectangle(pos.x, y, 2.0, 2.0, Color::new(color.r, color.g, color.b, a));
                draw_rectangle(
                    pos.x,
                    y + 1.0,
                    2.0,
                    1.0,
                    Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, a),
                );
            }
            // 반짝임: 십자가 커졌다 줄어든다(팔 길이 0→2→0, 가운데는 흰 점).
            FxKind::Glint { color } => {
                if f.time < 0.0 {
                    continue;
                }
                let arm = ((1.0 - (p * 2.0 - 1.0).abs()) * 2.5).round();
                let c = Color::new(color.r, color.g, color.b, 1.0 - p * 0.5);
                if arm > 0.0 {
                    draw_rectangle(pos.x - arm, pos.y, arm * 2.0 + 1.0, 1.0, c);
                    draw_rectangle(pos.x, pos.y - arm, 1.0, arm * 2.0 + 1.0, c);
                }
                draw_rectangle(pos.x, pos.y, 1.0, 1.0, WHITE);
            }
            // 풀려 흩어지는 실: 짧은 물결 선(점 몇 개), 끝으로 갈수록 옅다.
            FxKind::Thread { angle, len } => {
                let c = crate::world::THREAD;
                let d = vec2(angle.cos(), angle.sin());
                let n = vec2(-d.y, d.x);
                let n_px = len.round() as i32;
                for i in 0..n_px {
                    let t = i as f32;
                    let wave = (t * 0.9 + f.time * 9.0).sin() * 1.2;
                    let q = (pos + d * t + n * wave).round();
                    let a = (1.0 - p) * (1.0 - t / len * 0.6);
                    draw_rectangle(q.x, q.y, 1.0, 1.0, Color::new(c.r, c.g, c.b, a));
                }
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
            StrikeKind::Stomp => PINK,
        };
        match s.shape {
            Shape::Circle { center, r } => draw_circle_lines(center.x, center.y, r, 1.0, col),
            Shape::Sector { origin, dir, r } => draw_sector_lines(origin, dir, r, col),
        }
    }
}

/// 체력 핍(왼쪽 위)·굴파기 쿨다운·SP 게이지(오른쪽 위). 화면 패스에서 게임 px 격자로(ui.rect).
/// 체력 한 칸의 간격: 팩 하트(9px) 또는 직접 그린 네모(6px).
fn hp_step(sheet: Option<&Texture2D>) -> f32 {
    if sheet.is_some() {
        10.0
    } else {
        7.0
    }
}

fn draw_hud_bars(ui: &Ui, w: &World, sheet: Option<&Texture2D>) {
    let dark = Color::new(0.2, 0.15, 0.2, 1.0);
    let step = hp_step(sheet);
    for i in 0..w.player.stats.max_hp {
        let x = 3.0 + i as f32 * step;
        match sheet {
            // 빈 칸은 같은 하트를 어둡게(0077).
            Some(sheet) => {
                let tint = if i < w.player.hp {
                    WHITE
                } else {
                    Color::new(0.22, 0.17, 0.24, 1.0)
                };
                ui.sprite(sheet, crate::sunny::rect(crate::sunny::HEART), x, 0.0, tint);
            }
            None => {
                let c = if i < w.player.hp { world::MOLE } else { dark };
                ui.rect(Rect::new(x, 1.0, 6.0, 6.0), c);
            }
        }
    }
    // 오른쪽 위(0091): 굴파기 대기(위, 가늘게)와 SP 게이지(아래, 굵게). 24px·3~4px로는 눈에 안 띄었다 →
    // 34px, 바탕판, 칸마다 어두운 테두리, 가득 찬 SP는 금빛 테두리가 숨 쉰다.
    let cd = (w.player.burrow_cd / w.player.burrow_cd_full()).min(1.0);
    let bw = 34.0;
    // 필살기를 못 냈다(0149): 게이지 판이 좌우로 떨고 붉은 테두리.
    let x = t::VIEW_W - 4.0 - bw + deny_shake(w.deny[0]);
    hud_plate(ui, Rect::new(x - 3.0, -1.0, bw + 6.0, 16.0));
    let frame = Color::new(0.02, 0.01, 0.03, 1.0);
    ui.rect(Rect::new(x - 1.0, 1.0, bw + 2.0, 5.0), frame);
    ui.rect(Rect::new(x, 2.0, bw, 3.0), dark);
    let ready = cd <= 0.0;
    ui.rect(
        Rect::new(x, 2.0, (bw * (1.0 - cd).max(0.0)).round(), 3.0),
        if ready {
            Color::new(0.78, 0.60, 0.40, 1.0)
        } else {
            world::DIRT
        },
    );
    // SP 게이지(그 아래). 가득이면 깜빡인다 — 필살기를 쓸 수 있다. 색은 캐릭터의 게이지 종류(두더지 = 분노).
    let sp = w.player.sp / t::SP_MAX;
    let full = w.player.sp_full();
    ui.rect(Rect::new(x - 1.0, 7.0, bw + 2.0, 7.0), frame);
    ui.rect(Rect::new(x, 8.0, bw, 5.0), dark);
    let gc = w.character.gauge().color();
    let gold = if full && ((w.time * 6.0) as i32) % 2 == 0 {
        Color::new(1.0, 1.0, 0.85, 1.0)
    } else {
        gc
    };
    ui.rect(Rect::new(x, 8.0, (bw * sp).round(), 5.0), gold);
    if w.deny[0] > 0.0 {
        ui.rect_lines(
            Rect::new(x - 2.0, 6.0, bw + 4.0, 9.0),
            deny_color(w.deny[0]),
        );
        deny_cross(ui, vec2(x + bw / 2.0, 10.0), 3.0);
    }
    if full {
        let a = 0.5 + 0.5 * (w.time * 5.0).sin();
        let c = crate::ui::ACCENT;
        ui.rect_lines(
            Rect::new(x - 2.0, 6.0, bw + 4.0, 9.0),
            Color::new(c.r, c.g, c.b, a),
        );
    }
}

/// 바닥 위 타원(축 `g` 방향 반지름 `along`, 옆 `across`)을 3/4 시점으로(세로 ×0.5) 눌렀을 때 화면의 (가로 반지름, 세로 반지름, 각도°).
/// 눌린 타원의 공분산 M·Mᵀ(M = 누름 × 돌림 × 반지름)의 고유값·고유벡터(0150).
fn ground_ellipse(g: Vec2, along: f32, across: f32) -> (f32, f32, f32) {
    let n = vec2(-g.y, g.x);
    let (u, v) = (vec2(g.x, g.y * 0.5) * along, vec2(n.x, n.y * 0.5) * across);
    let (xx, yy, xy) = (
        u.x * u.x + v.x * v.x,
        u.y * u.y + v.y * v.y,
        u.x * u.y + v.x * v.y,
    );
    let mid = (xx + yy) * 0.5;
    let d = (((xx - yy) * 0.5).powi(2) + xy * xy).sqrt();
    let ang = 0.5 * (2.0 * xy).atan2(xx - yy);
    (
        (mid + d).sqrt(),
        (mid - d).max(0.0).sqrt(),
        ang.to_degrees(),
    )
}

/// 할 수 없는 행동(0149)의 붉은 테두리.
const DENY_RED: Color = Color::new(1.0, 0.30, 0.32, 1.0);

/// 테두리는 흰색·붉은색을 번갈아 깜빡인다(0150): 1·2형 색약 시뮬레이션에서 붉은 테두리가 고른 칸의 금빛과 같아졌다.
/// 밝기가 바뀌는 깜빡임과 ✕ 모양(`deny_cross`)은 색을 몰라도 읽힌다.
fn deny_color(left: f32) -> Color {
    if ((left * 16.0) as i32) % 2 == 0 {
        WHITE
    } else {
        DENY_RED
    }
}

/// 픽셀 ✕(반지름 `h`): 어두운 테두리 위에 흰 대각선 둘.
fn deny_cross(ui: &Ui, c: Vec2, h: f32) {
    let c = c.round();
    let n = h as i32;
    for (col, grow) in [(OUTLINE, 1), (WHITE, 0)] {
        for i in -n..=n {
            let f = i as f32;
            for (dx, dy) in [(f, f), (f, -f)] {
                let g = grow as f32;
                ui.rect(
                    Rect::new(c.x + dx - g, c.y + dy - g, 1.0 + 2.0 * g, 1.0 + 2.0 * g),
                    col,
                );
            }
        }
    }
}

/// 떨림(px): 남은 시간 동안 1px씩 좌우로.
fn deny_shake(left: f32) -> f32 {
    if left <= 0.0 {
        0.0
    } else if ((left * 40.0) as i32) % 2 == 0 {
        1.0
    } else {
        -1.0
    }
}

fn draw_hud_screen(w: &World, scale: f32, ox: f32, oy: f32, gamepad_count: usize) {
    let fs = 16.0 * (scale / 4.0).max(1.0);
    let line = fs * 1.2;
    // 월드 HUD(체력 핍 8px 높이) 아래부터 텍스트.
    let mut y = oy + 9.0 * scale + fs;
    // 웨이브·처치 수는 아레나(웨이브 시험장)에서만. 실의 꿈(0055)도 아레나지만 보이지 않는다.
    if w.adv.place == crate::adventure::Place::Arena && !w.dream {
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
        player::State::Eat { .. } => "eat",
        player::State::Hop { .. } => "hop",
        player::State::Chain { .. } => "chain",
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

/// 빛 하나(0095): 자리·반지름·세기(0..1, 어둠을 얼마나 빼나).
pub struct LightSrc {
    pub pos: Vec2,
    pub radius: f32,
    pub strength: f32,
}

/// 지금 화면 둘레의 빛. 어둠 지도와 그림자(빛 반대쪽으로 늘어남)가 같이 쓴다.
pub fn light_sources(w: &World, alpha: f32, cam: Vec2, player_radius: f32) -> Vec<LightSrc> {
    let t = w.time;
    let mut v = Vec::new();
    // 플레이어(예전 어둠 막의 구멍과 같은 크기, 가운데는 거의 다 걷힌다).
    v.push(LightSrc {
        pos: lerp(w.player.prev_pos, w.player.pos, alpha),
        radius: player_radius,
        strength: 1.0,
    });
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.85 + 0.15 * ((t * 2.0 + x * 0.1).sin());
            v.push(LightSrc {
                pos: vec2(x + 8.0, y + 10.0),
                radius: t::MUSH_LIGHT_R,
                strength: t::MUSH_LIGHT * pulse,
            });
        }
    }
    for p in &w.room.props {
        if p.kind == PropKind::Campfire {
            let flick = 0.9 + 0.1 * (t * 9.0).sin();
            v.push(LightSrc {
                pos: vec2(p.rect.center().x, p.rect.bottom() - 6.0),
                radius: 80.0,
                strength: 0.95 * flick,
            });
        }
    }
    for l in &w.loot {
        if l.kind == loot::LootKind::Glowbug {
            v.push(LightSrc {
                pos: lerp(l.prev_pos, l.pos, alpha),
                radius: 18.0,
                strength: 0.55,
            });
        }
    }
    if let Some(g) = &w.room.grid {
        let (x0, y0) = (
            (cam.x / 16.0).floor() as i32 - 2,
            (cam.y / 16.0).floor() as i32 - 2,
        );
        for cy in y0..y0 + 16 {
            for cx in x0..x0 + 25 {
                let c = vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0);
                match g.get_i(cx, cy) {
                    Cell::Save => v.push(LightSrc {
                        pos: c,
                        radius: 44.0,
                        strength: 0.7,
                    }),
                    Cell::Exit => v.push(LightSrc {
                        pos: c,
                        radius: 34.0,
                        strength: 0.65,
                    }),
                    Cell::Door if w.room.door_anim > 0.0 => v.push(LightSrc {
                        pos: c,
                        radius: 22.0,
                        strength: 0.4 * w.room.door_anim,
                    }),
                    _ => {}
                }
            }
        }
    }
    v
}

/// 해 그림자 색(0102). 실루엣 재질(셰이더 안 같은 값)과 지도 굽기가 같이 쓴다. 알파 = 진하기.
pub const SUN_SHADE: Color = Color::new(0.10, 0.06, 0.14, t::SUN_SHADOW_ALPHA);

/// 실루엣 재질(해 그림자, 0102): 그린 자리(알파가 있는 곳)를 모두 같은 그림자 색(`SUN_SHADE`)으로. 진하기는 정점 색 알파.
fn silhouette_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec2 uv;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    varying highp vec2 world;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
        world = position.xy;
    }"#;
    // Fire = (불 x, 불 y, 닿는 거리, 어두운 곳이면 1): 어두운 곳은 불에서 멀수록 옅게(0103).
    // Fade = 옅어지기 시작·다 사라지는 거리(닿는 거리의 비율, t::FIRE_SHADOW_FADE).
    const FRAGMENT: &str = r#"#version 100
    precision mediump float;
    varying lowp vec4 color;
    varying lowp vec2 uv;
    varying highp vec2 world;
    uniform sampler2D Texture;
    uniform vec4 Fire;
    uniform vec2 Fade;
    void main() {
        // 반투명(빛 원·잔상·옅은 고리)은 그림자를 드리우지 않는다: 반쯤 넘게 채운 점만 모양으로 친다(0106).
        float a = step(0.5, texture2D(Texture, uv).a);
        float d = distance(world, Fire.xy);
        float fade = mix(1.0, 1.0 - smoothstep(Fade.x * Fire.z, Fade.y * Fire.z, d), Fire.w);
        gl_FragColor = vec4(0.10, 0.06, 0.14, a * color.a * fade);
    }"#;
    load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(macroquad::miniquad::BlendState::new(
                    macroquad::miniquad::Equation::Add,
                    macroquad::miniquad::BlendFactor::Value(
                        macroquad::miniquad::BlendValue::SourceAlpha,
                    ),
                    macroquad::miniquad::BlendFactor::OneMinusValue(
                        macroquad::miniquad::BlendValue::SourceAlpha,
                    ),
                )),
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("Fire", UniformType::Float4),
                UniformDesc::new("Fade", UniformType::Float2),
            ],
            ..Default::default()
        },
    )
    .ok()
}

/// 어둠 빼기 재질(0095): 어둠 지도에서 그린 만큼 알파를 뺀다(결과 = 바탕 − 그림, 0 아래는 0).
fn cut_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec2 uv;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
    const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;
    uniform sampler2D Texture;
    void main() {
        gl_FragColor = color * texture2D(Texture, uv);
    }"#;
    use macroquad::miniquad::{BlendFactor, BlendState, Equation};
    load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::ReverseSubtract,
                    BlendFactor::One,
                    BlendFactor::One,
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::ReverseSubtract,
                    BlendFactor::One,
                    BlendFactor::One,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .ok()
}

/// 빛 반대쪽으로 늘어나는 그림자(0095). 야광버섯·모닥불·저장 지점 곁의 캐릭터는 빛에서 먼 쪽으로 그림자가 길어지고,
/// 어두운 곳에서는 플레이어의 빛이 적에게 그림자를 드리운다. 바닥 위(정렬 그리기 전)에 그려 어둠 지도가 함께 덮는다.
/// `fire_silhouettes`: 모닥불 그림자는 모양 그림자(`draw_sun_shadows`, 0103)가 맡으므로 여기서 뺀다. 밝은 곳은 통째로 해 그림자에 맡긴다(0106).
fn draw_cast_shadows(w: &World, alpha: f32, cam: Vec2, fire_silhouettes: bool) {
    let (dark, radius) = w.biome.darkness();
    // 밝은 곳은 해 그림자(0102)가 맡는다: 낮에는 버섯·저장 지점·반딧불 같은 약한 빛이 그림자를 만들지 않는다(0106).
    if fire_silhouettes && dark <= 0.0 {
        return;
    }
    let fires = if fire_silhouettes {
        campfires(w)
    } else {
        Vec::new()
    };
    let mut lights = light_sources(w, alpha, cam, radius);
    // 플레이어 빛(0번)은 자리를 지킨다: 아래 skip_player가 0번을 본다.
    let mut i = 1;
    while i < lights.len() {
        if fires.iter().any(|f| (lights[i].pos - *f).length() < 0.5) {
            lights.remove(i);
        } else {
            i += 1;
        }
    }
    // 밝은 들판에서는 플레이어 빛이 없다(어둠이 없으니 비추는 빛이 아니다).
    let skip_player = if dark > 0.0 { 0 } else { 1 };
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    let mut casters: Vec<(Vec2, f32, bool)> = vec![(pp + vec2(0.0, 6.0), 6.0, true)];
    for e in &w.enemies {
        if e.alive {
            let q = lerp(e.prev_pos, e.pos, alpha);
            casters.push((q + vec2(0.0, e.radius() * 0.8), e.radius(), false));
        }
    }
    if let Some(n) = &w.npc {
        let q = lerp(n.prev_pos, n.pos, alpha);
        casters.push((q + vec2(0.0, 6.0), 6.0, false));
    }
    for (foot, size, is_player) in casters {
        // 가장 센 빛 둘만(0149): 버섯이 많은 방에서 빛마다 하나씩 그리면 발밑에서 별처럼 뾰족하게 뻗었다.
        let mut shades: Vec<(f32, Vec2)> = lights
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= skip_player && !(is_player && *i == 0))
            .filter_map(|(_, l)| {
                let d = foot - l.pos;
                let dist = d.length();
                let reach = l.radius * 1.15;
                if dist < 3.0 || dist > reach {
                    return None;
                }
                let k = (1.0 - dist / reach) * l.strength;
                (k >= 0.05).then_some((k, d / dist))
            })
            .collect();
        shades.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (k, dir) in shades.into_iter().take(2) {
            // 바닥 위 타원을 세로 절반으로 눌러 그린다(0150): 3/4 시점에서 바닥은 세로가 반이다.
            // 예전엔 화면에서 돌리기만 해서, 남북으로 눕는 그림자는 폭 3px 그대로 길게 서 막대처럼 얇았다.
            let g = vec2(dir.x, dir.y * 2.0).normalize_or(vec2(1.0, 0.0));
            let along = size * 0.45 + 4.0 * k + 2.0;
            let across = size * 0.6 + 1.0;
            let (rx, ry, ang) = ground_ellipse(g, along, across);
            let c = foot + vec2(g.x, g.y * 0.5) * (along * 0.8);
            let a = (0.5 * k).min(0.4);
            // 가장자리를 한 겹 옅게 둘러 끝이 뾰족하지 않게.
            draw_ellipse(
                c.x.round(),
                c.y.round(),
                rx + 1.5,
                ry + 1.0,
                ang,
                Color::new(0.03, 0.01, 0.05, a * 0.45),
            );
            draw_ellipse(
                c.x.round(),
                c.y.round(),
                rx,
                ry,
                ang,
                Color::new(0.03, 0.01, 0.05, a * 0.6),
            );
        }
    }
}

// ---- 내려가는 전환: 지층 단면(0156) ---------------------------------------

/// 1m가 화면 몇 px인가(단면이 흐르는 빠르기).
const DESCENT_PX_PER_M: f32 = 12.0;
/// 파 내려가는 Naïl의 화면 높이(머리 끝).
const DESCENT_DIG_Y: f32 = 104.0;

/// 깊이(m)에 맞는 흙 색: 땅 위 겉흙 → 흙 → 뿌리 흙 → 검은 바위 → 가장 깊은 곳.
fn strata_color(m: f32) -> Color {
    const KEYS: [(f32, [f32; 3]); 6] = [
        (0.0, [0.42, 0.29, 0.18]),
        (8.0, [0.37, 0.25, 0.16]),
        (30.0, [0.29, 0.19, 0.13]),
        (55.0, [0.21, 0.15, 0.12]),
        (85.0, [0.14, 0.12, 0.16]),
        (130.0, [0.08, 0.07, 0.11]),
    ];
    let m = m.max(0.0);
    let i = KEYS.iter().rposition(|(k, _)| *k <= m).unwrap_or(0);
    let (a, ca) = KEYS[i];
    let (b, cb) = KEYS[(i + 1).min(KEYS.len() - 1)];
    let f = if b > a {
        ((m - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Color::new(
        ca[0] + (cb[0] - ca[0]) * f,
        ca[1] + (cb[1] - ca[1]) * f,
        ca[2] + (cb[2] - ca[2]) * f,
        1.0,
    )
}

fn shade(c: Color, k: f32, a: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, a)
}

/// 지층 단면을 화면(내부 해상도, 화면 고정 카메라)에 그린다. `d.depth_m`이 가운데 깊이, `clock`은 실제 시간
/// (굽는 동안에도 Naïl과 흙 부스러기는 움직인다). `cover`만큼 아래 월드를 덮는다.
fn draw_descent(
    art: Option<&Art>,
    w: &World,
    d: crate::adventure::Descent,
    clock: f32,
    cover: f32,
) {
    if cover <= 0.0 {
        return;
    }
    let a = cover;
    let px = DESCENT_PX_PER_M;
    // 화면 맨 위 줄의 절대 깊이(px). 정수로 굴려 도트가 떨리지 않게.
    let top = (d.depth_m * px - DESCENT_DIG_Y).round();
    // 지층: 4m 띠마다 색이 조금씩 다르고, 8px 기둥마다 경계가 물결친다. 2px 줄 단위.
    let col_w = 8.0;
    let cols = (RW / col_w).ceil() as i32;
    let mut y = 0.0;
    while y < RH {
        let abs_y = top + y;
        for c in 0..cols {
            let x = c as f32 * col_w;
            if abs_y < 0.0 {
                // 땅 위: 저녁 하늘.
                let k = ((abs_y + 120.0) / 120.0).clamp(0.0, 1.0);
                let sky = Color::new(0.36 + 0.30 * k, 0.24 + 0.12 * k, 0.33 - 0.05 * k, a);
                draw_rectangle(x, y, col_w, 2.0, sky);
                continue;
            }
            let wob = (hash01(c as u32, (abs_y / 48.0).floor() as u32 + 7) - 0.5) * 6.0;
            let m = (abs_y + wob) / px;
            let band = (m / 4.0).floor();
            let base = strata_color(band * 4.0);
            let k = 0.92 + 0.16 * hash01(band as u32, 3);
            draw_rectangle(x, y, col_w, 2.0, shade(base, k, a));
        }
        y += 2.0;
    }
    // 지층 경계: 4m마다 어두운 가는 줄(기둥마다 물결).
    for c in 0..cols {
        let x = c as f32 * col_w;
        let b0 = (top / (4.0 * px)).floor() as i32;
        for b in b0..=b0 + (RH / (4.0 * px)).ceil() as i32 + 1 {
            if b <= 0 {
                continue;
            }
            let wob = (hash01(c as u32, b as u32 + 7) - 0.5) * 6.0;
            let yy = (b as f32 * 4.0 * px - wob - top).round();
            if (0.0..RH).contains(&yy) {
                let base = strata_color(b as f32 * 4.0);
                draw_rectangle(x, yy, col_w, 1.0, shade(base, 0.72, a));
            }
        }
    }
    // 큰 돌: 32px 칸에 하나쯤, 반지름 3~6px 덩어리(외곽선 + 윗면 밝게).
    let big = 32.0;
    let bfirst = (top / big).floor() as i32;
    for r in 0..=(RH / big).ceil() as i32 + 1 {
        let by = bfirst + r;
        if by < 0 {
            continue;
        }
        for c in 0..(RW / big).ceil() as i32 {
            let id = (by as u32).wrapping_mul(131) ^ (c as u32).wrapping_mul(7);
            if hash01(id, 31) > 0.32 {
                continue;
            }
            let cx = c as f32 * big + 6.0 + (hash01(id, 32) * 20.0).floor();
            let cy = by as f32 * big + 6.0 + (hash01(id, 33) * 20.0).floor() - top;
            if (cx - 160.0).abs() < 16.0 && cy < DESCENT_DIG_Y + 8.0 {
                continue;
            }
            let rr = 3.0 + (hash01(id, 34) * 3.0).floor();
            let m = (cy + top) / px;
            let deep = (m / 100.0).clamp(0.0, 1.0);
            let st = Color::new(
                0.50 - 0.22 * deep,
                0.46 - 0.22 * deep,
                0.42 - 0.16 * deep,
                a,
            );
            for dy in -rr as i32..=rr as i32 {
                let half = ((rr * rr - (dy as f32) * (dy as f32)).max(0.0).sqrt() * 1.3).round();
                let y = cy + dy as f32;
                draw_rectangle(
                    cx - half - 1.0,
                    y,
                    half * 2.0 + 2.0,
                    1.0,
                    shade(st, 0.45, a),
                );
                let tone = if dy < -(rr as i32) / 3 {
                    1.18
                } else if dy > (rr as i32) / 3 {
                    0.82
                } else {
                    1.0
                };
                draw_rectangle(cx - half, y, half * 2.0, 1.0, shade(st, tone, a));
            }
        }
    }
    // 긴 뿌리: 24px 줄마다 가끔 옆으로 길게(얕은 곳은 가늘게, 뿌리 지대는 굵게, 심층은 보라 실이 감긴다).
    let lane = 24.0;
    let lfirst = (top / lane).floor() as i32;
    for r in 0..=(RH / lane).ceil() as i32 + 1 {
        let ly = lfirst + r;
        if ly < 0 {
            continue;
        }
        let id = (ly as u32).wrapping_mul(2654435761);
        let m = ly as f32 * lane / px;
        if hash01(id, 41)
            > if (30.0..90.0).contains(&m) {
                0.55
            } else {
                0.25
            }
        {
            continue;
        }
        let x0 = (hash01(id, 42) * 200.0).floor() - 20.0;
        let len = 60.0 + (hash01(id, 43) * 90.0).floor();
        let thick = if (30.0..90.0).contains(&m) { 2.0 } else { 1.0 };
        let y0 = ly as f32 * lane + (hash01(id, 44) * 12.0).floor() - top;
        let rc = Color::new(0.50, 0.36, 0.25, a);
        let threaded = m > 45.0 && hash01(id, 45) < 0.5;
        let ph = hash01(id, 46) * 6.0;
        let mut x = x0;
        while x < x0 + len {
            if (x - 160.0).abs() > 7.0 || y0 > DESCENT_DIG_Y + 6.0 {
                let yy = (y0 + ((x - x0) * 0.08 + ph).sin() * 3.0).round();
                draw_rectangle(x, yy, 1.0, thick, rc);
                draw_rectangle(x, yy + thick, 1.0, 1.0, shade(rc, 0.6, a));
                if threaded && ((x - x0) as i32) % 9 < 2 {
                    draw_rectangle(x, yy - 1.0, 1.0, 1.0, Color::new(0.70, 0.50, 0.90, a));
                }
            }
            x += 1.0;
        }
    }
    // 풀 줄(땅 위와 흙 사이).
    if top < 0.0 && top > -RH {
        let gy = -top;
        draw_rectangle(0.0, gy - 3.0, RW, 3.0, Color::new(0.36, 0.56, 0.26, a));
        draw_rectangle(0.0, gy - 4.0, RW, 1.0, Color::new(0.52, 0.72, 0.36, a));
    }
    // 장식: 8px 칸 해시. 깊이마다 나오는 것이 다르다(조약돌 늘, 잔뿌리·지렁이는 얕은 곳, 굵은 뿌리·보라 실은 가운데,
    // 빛버섯·수정은 깊은 곳). 칸은 절대 깊이에 붙어 있어 내려가면 위로 흘러간다.
    let cell = 8.0;
    let first = (top / cell).floor() as i32;
    let rows = (RH / cell).ceil() as i32 + 1;
    for r in 0..rows {
        let cy = first + r;
        if cy < 0 {
            continue;
        }
        let m = cy as f32 * cell / px;
        let y0 = cy as f32 * cell - top;
        for c in 0..cols {
            let x0 = c as f32 * cell;
            // 가운데 굴(파 온 길)에는 장식을 두지 않는다.
            if (x0 + 4.0 - 160.0).abs() < 12.0 && y0 < DESCENT_DIG_Y {
                continue;
            }
            let id = (cy as u32).wrapping_mul(97) ^ c as u32;
            let h = hash01(id, 11);
            let ox = (hash01(id, 12) * 5.0).floor();
            let oy = (hash01(id, 13) * 5.0).floor();
            let (x, y) = (x0 + ox, y0 + oy);
            let soil = strata_color(m);
            if h < 0.035 {
                // 조약돌
                let st = Color::new(0.52, 0.47, 0.42, a);
                draw_rectangle(x, y + 1.0, 4.0, 2.0, shade(st, 0.8, a));
                draw_rectangle(x + 1.0, y, 2.0, 1.0, st);
                draw_rectangle(x + 1.0, y + 1.0, 1.0, 1.0, Color::new(0.70, 0.66, 0.60, a));
            } else if h < 0.07 && m < 45.0 {
                // 잔뿌리
                let rc = Color::new(0.55, 0.40, 0.27, a * 0.8);
                for i in 0..6 {
                    let yy = y + ((i as f32 * 0.9 + hash01(id, 14) * 3.0).sin() * 1.5).round();
                    draw_rectangle(x + i as f32, yy, 1.0, 1.0, rc);
                }
            } else if h < 0.082 && (3.0..35.0).contains(&m) {
                // 지렁이
                let pink = Color::new(0.86, 0.52, 0.55, a);
                draw_rectangle(x, y + 1.0, 4.0, 1.0, pink);
                draw_rectangle(x + 4.0, y, 1.0, 1.0, pink);
            } else if h < 0.11 && (35.0..85.0).contains(&m) {
                // 굵은 뿌리
                let rc = Color::new(0.46, 0.33, 0.23, a);
                for i in 0..8 {
                    let yy = y + ((i as f32 * 0.6 + hash01(id, 15) * 6.0).sin() * 2.0).round();
                    draw_rectangle(x + i as f32 - 1.0, yy, 1.0, 2.0, rc);
                }
            } else if h < 0.13 && m > 40.0 {
                // 보라 실(물든 뿌리)
                let th = Color::new(0.66, 0.47, 0.85, a * 0.9);
                for i in 0..4 {
                    draw_rectangle(x + i as f32, y + i as f32 * 0.5, 1.0, 1.0, th);
                }
            } else if h < 0.16 && m > 78.0 {
                // 빛버섯: 갓 + 둘레 빛
                let glow = Color::new(0.35, 0.75, 0.95, a * 0.25);
                draw_rectangle(x - 1.0, y - 1.0, 5.0, 4.0, glow);
                draw_rectangle(x, y, 3.0, 1.0, Color::new(0.45, 0.85, 1.0, a));
                draw_rectangle(x + 1.0, y + 1.0, 1.0, 2.0, Color::new(0.80, 0.85, 0.80, a));
            } else if h < 0.17 && (10.0..60.0).contains(&m) {
                // 옛 조개 화석
                let fc = Color::new(0.80, 0.74, 0.60, a * 0.8);
                draw_rectangle(x, y, 3.0, 1.0, fc);
                draw_rectangle(x, y + 1.0, 1.0, 1.0, fc);
                draw_rectangle(x + 2.0, y + 1.0, 1.0, 1.0, fc);
            } else if h < 0.24 {
                // 흙 얼룩(밝고 어두운 점)
                draw_rectangle(x, y, 2.0, 1.0, shade(soil, 1.25, a * 0.7));
            }
        }
    }
    // 파 온 굴: Naïl 위로 어두운 기둥, 가장자리 한 줄은 더 어둡다.
    let tw = 12.0;
    let tx = 160.0 - tw / 2.0;
    let ty = (-top).max(0.0);
    if ty < DESCENT_DIG_Y {
        let h = DESCENT_DIG_Y - ty + 4.0;
        let soil = strata_color(d.depth_m);
        draw_rectangle(tx, ty, tw, h, shade(soil, 0.42, a));
        draw_rectangle(tx - 1.0, ty, 1.0, h, shade(soil, 0.65, a));
        draw_rectangle(tx + tw, ty, 1.0, h, shade(soil, 0.65, a));
        // 굴 벽에서 떨어지는 부스러기(위로 흘러간다 = 내려가는 중).
        for i in 0..10u32 {
            let ph = (clock * 1.3 + hash01(i, 21)).fract();
            let bx = tx + 1.0 + (hash01(i, 22) * (tw - 2.0)).floor();
            let by = DESCENT_DIG_Y - ph * (DESCENT_DIG_Y - ty);
            draw_rectangle(bx, by.round(), 1.0, 1.0, shade(soil, 1.3, a * 0.8));
        }
    }
    // 앞쪽으로 튀는 흙(머리 아래, 좌우로).
    let soil = strata_color(d.depth_m + 2.0);
    for i in 0..8u32 {
        let ph = (clock * 2.6 + i as f32 / 8.0).fract();
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let sx = 160.0 + side * (3.0 + ph * (6.0 + 6.0 * hash01(i, 23)));
        let sy = DESCENT_DIG_Y + 14.0 - ph * 10.0 + ph * ph * 12.0;
        draw_rectangle(
            sx.round(),
            sy.round(),
            1.0 + (i % 3 == 0) as u8 as f32,
            1.0,
            shade(soil, 1.35, a * (1.0 - ph)),
        );
    }
    // Naïl(또는 모모): 옆모습을 90° 돌려 머리가 아래로. 걷기 칸을 빨리 돌려 앞발로 긁는 것처럼.
    if let Some(art) = art {
        let jit = ((clock * 30.0) as i32 % 2) as f32;
        let (tex, src) = if w.player.kit == crate::equipment::Character::Momo {
            match art.critters.as_ref() {
                Some(t) => (
                    t,
                    Rect::new(
                        (2 + ((clock * 10.0) as u32) % 2) as f32 * 16.0,
                        art::MOMO_BLOOM_ROW as f32 * 16.0,
                        16.0,
                        16.0,
                    ),
                ),
                None => (&art.nail, Rect::new(32.0, 0.0, 16.0, 16.0)),
            }
        } else {
            (
                art.nail_fur(w.fur),
                Rect::new(
                    (2 + ((clock * 14.0) as u32) % 4) as f32 * 16.0,
                    0.0,
                    16.0,
                    16.0,
                ),
            )
        };
        draw_texture_ex(
            tex,
            152.0 + jit,
            DESCENT_DIG_Y - 4.0,
            Color::new(1.0, 1.0, 1.0, a),
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(16.0, 16.0)),
                rotation: std::f32::consts::FRAC_PI_2,
                ..Default::default()
            },
        );
    }
    // 깊은 곳은 어둡고 Naïl 둘레만 보인다(어둠 마스크 = 가운데 투명).
    let dark = ((d.depth_m - 35.0) / 80.0).clamp(0.0, 0.85);
    if let (Some(art), true) = (art, dark > 0.0) {
        let size = 260.0;
        let (mx, my) = (160.0, DESCENT_DIG_Y + 6.0);
        draw_texture_ex(
            &art.dark,
            mx - size / 2.0,
            my - size / 2.0,
            Color::new(1.0, 1.0, 1.0, dark * a),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
        let c = Color::new(0.0, 0.0, 0.0, dark * a);
        draw_rectangle(0.0, 0.0, RW, (my - size / 2.0).max(0.0), c);
        draw_rectangle(0.0, my + size / 2.0, RW, RH, c);
        draw_rectangle(0.0, 0.0, (mx - size / 2.0).max(0.0), RH, c);
        draw_rectangle(mx + size / 2.0, 0.0, RW, RH, c);
    }
    // 위아래 가장자리 어둠(깊을수록 더 어둡다).
    let dark = (d.depth_m / 130.0).clamp(0.0, 1.0);
    for i in 0..12 {
        let k = 1.0 - i as f32 / 12.0;
        let c = Color::new(0.0, 0.0, 0.0, a * k * (0.25 + 0.45 * dark));
        draw_rectangle(0.0, i as f32 * 2.0, RW, 2.0, c);
        draw_rectangle(0.0, RH - (i as f32 + 1.0) * 2.0, RW, 2.0, c);
    }
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
        let shown = t::BIOME_CARD_TIME - time;
        let a = (shown / 0.6).min(time / 0.8).clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let name = crate::i18n::tr(biome.name());
        let depth = crate::i18n::fill(crate::i18n::s().depth_m, &[&m]);
        let out = ui.with_outline();
        let y = 48.0;
        let nw = ui.text_w(name, crate::ui::FONT_PX);
        let dw = ui.text_w(&depth, crate::ui::FONT_PX);
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
            &depth,
            160.0,
            y + 13.0,
            crate::ui::FONT_PX,
            Color::new(0.80, 0.74, 0.66, a),
        );
    }
}

// ---- 방 안의 "위"(0156): 땅속이라는 것이 방에서도 느껴지게 ---------------------

/// 뿌리 지대: 천장 틈으로 비스듬히 내려오는 빛줄기(더하기 빛)와 그 안의 먼지. 월드 칸(112px)에 붙어 있어
/// 카메라가 움직여도 제자리다. 벽 칸에는 내리지 않는다.
fn draw_light_shafts(w: &World, cam: Vec2, k: f32) {
    if w.biome != Biome::Roots || w.effects <= 0.0 {
        return;
    }
    const CELL: f32 = 112.0;
    let t = w.time;
    let (x0, y0) = (
        (cam.x / CELL).floor() as i32 - 1,
        (cam.y / CELL).floor() as i32 - 1,
    );
    for cy in y0..=y0 + (RH / CELL).ceil() as i32 + 2 {
        for cx in x0..=x0 + (RW / CELL).ceil() as i32 + 2 {
            let id = (cx as u32).wrapping_mul(73_856_093) ^ (cy as u32).wrapping_mul(19_349_663);
            if hash01(id, 51) > 0.38 {
                continue;
            }
            let p = vec2(
                cx as f32 * CELL + 16.0 + hash01(id, 52) * (CELL - 32.0),
                cy as f32 * CELL + 16.0 + hash01(id, 53) * (CELL - 32.0),
            )
            .round();
            if w.room.is_solid(p, 6.0) {
                continue;
            }
            // 위(왼쪽 위 천장 틈)에서 바닥 p로: 위는 좁고 아래는 넓은 사다리꼴. 아주 천천히 숨쉰다.
            let breathe = 0.75 + 0.25 * (t * 0.6 + hash01(id, 54) * 6.0).sin();
            let a = 0.11 * breathe * k * w.effects.min(1.0);
            let top = p + vec2(-30.0, -78.0);
            let (tw, bw) = (4.0, 12.0);
            let warm = Color::new(1.0, 0.88, 0.62, a);
            draw_triangle(
                top - vec2(tw, 0.0),
                top + vec2(tw, 0.0),
                p + vec2(bw, 0.0),
                warm,
            );
            draw_triangle(
                top - vec2(tw, 0.0),
                p + vec2(bw, 0.0),
                p - vec2(bw, 0.0),
                warm,
            );
            // 안쪽 심(조금 더 밝게).
            let core = Color::new(1.0, 0.92, 0.70, a * 0.8);
            draw_triangle(
                top - vec2(1.5, 0.0),
                top + vec2(1.5, 0.0),
                p + vec2(5.0, 0.0),
                core,
            );
            draw_triangle(
                top - vec2(1.5, 0.0),
                p + vec2(5.0, 0.0),
                p - vec2(5.0, 0.0),
                core,
            );
            // 바닥에 닿은 빛 웅덩이(납작한 타원 = 원 둘을 겹쳐).
            let pool = Color::new(1.0, 0.86, 0.58, a * 1.6);
            draw_ellipse(p.x, p.y, 15.0, 5.0, 0.0, pool);
            // 빛줄기 안을 천천히 내려오는 먼지.
            for m in 0..4u32 {
                let ph = (t * (0.08 + 0.04 * hash01(id ^ m, 55)) + hash01(id ^ m, 56)).fract();
                let q = top + (p - top) * ph + vec2(((t * 0.9 + m as f32 * 2.1).sin()) * 2.0, 0.0);
                let tw = 0.5 + 0.5 * (t * 3.0 + m as f32).sin();
                draw_rectangle(
                    q.x.round(),
                    q.y.round(),
                    1.0,
                    1.0,
                    Color::new(1.0, 0.95, 0.80, (0.25 + 0.35 * tw) * k),
                );
            }
        }
    }
}

/// 위에서 떨어지는 것: 뿌리 지대는 흙 알갱이, 심층은 물방울. 바닥 그림자가 커지다 닿으면 부서지거나 퍼진다.
/// 상태 없이 시간으로 정한다(같은 번호·같은 주기 = 같은 자리).
fn draw_overhead_falls(w: &World, cam: Vec2) {
    let drip = match w.biome {
        Biome::Roots => false,
        Biome::Deep => true,
        Biome::Meadow => return,
    };
    if w.room.grid.is_none() {
        return;
    }
    const FALL: f32 = 0.45;
    const LAND: f32 = 0.35;
    for i in 0..6u32 {
        if !keep_share(i as usize, w.effects) {
            continue;
        }
        let period = 2.4 + hash01(i, 61) * 2.2;
        let tt = w.time + hash01(i, 62) * period;
        let cyc = (tt / period).floor() as u32;
        let since = tt - cyc as f32 * period;
        if since > FALL + LAND {
            continue;
        }
        let p = vec2(
            cam.x + 12.0 + hash01(i ^ cyc.wrapping_mul(7919), 63) * (RW - 24.0),
            cam.y + 24.0 + hash01(i ^ cyc.wrapping_mul(104_729), 64) * (RH - 40.0),
        )
        .round();
        if w.room.is_solid(p, 3.0) {
            continue;
        }
        if since < FALL {
            let f = since / FALL;
            let h = 46.0 * (1.0 - f * f);
            // 그림자: 떨어질수록 진하고 크다.
            draw_ellipse(
                p.x,
                p.y,
                1.0 + 2.0 * f,
                0.5 + 0.8 * f,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.25 * f),
            );
            let c = if drip {
                Color::new(0.70, 0.88, 1.0, 0.85)
            } else {
                Color::new(0.55, 0.42, 0.30, 0.9)
            };
            draw_rectangle(p.x, (p.y - h).round(), 1.0, if drip { 2.0 } else { 1.0 }, c);
        } else {
            let f = (since - FALL) / LAND;
            if drip {
                // 물방울이 퍼지는 고리.
                draw_ellipse_lines(
                    p.x,
                    p.y,
                    1.0 + 4.0 * f,
                    0.5 + 1.6 * f,
                    0.0,
                    1.0,
                    Color::new(0.70, 0.88, 1.0, 0.6 * (1.0 - f)),
                );
            } else {
                // 흙 알갱이가 두 조각으로 튄다.
                let c = Color::new(0.55, 0.42, 0.30, 0.8 * (1.0 - f));
                let dx = (1.0 + 3.0 * f).round();
                let dy = (-2.0 * f + 3.0 * f * f).round();
                draw_rectangle(p.x - dx, p.y + dy - 1.0, 1.0, 1.0, c);
                draw_rectangle(p.x + dx, p.y + dy - 1.0, 1.0, 1.0, c);
            }
        }
    }
}
