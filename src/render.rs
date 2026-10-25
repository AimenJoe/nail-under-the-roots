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
    /// 글자를 미리 구운 (언어, 배율, dpi). 바뀌면(언어 전환·창 크기) 다시 굽는다.
    warmed: std::cell::Cell<Option<(crate::i18n::Lang, u32, u32)>>,
    /// 이번 프레임 월드 화면의 왼쪽 위(월드 좌표, 흔들림 포함). 월드 패스 뒤에는 카메라가 화면 고정이라
    /// 컷씬 배우는 이걸 빼서 그린다(`to_view`).
    view_origin: Vec2,
    /// 카메라의 소수점 부분(0..1, 게임 px). 올릴 때 이만큼 화면을 민다.
    subpixel: Vec2,
    /// 구운 지도(terrain.rs, 0062): 열쇠가 같으면 다시 굽지 않는다.
    terrain: Option<(u64, Texture2D)>,
}

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
            warmed: std::cell::Cell::new(None),
            view_origin: Vec2::ZERO,
            subpixel: Vec2::ZERO,
            terrain: None,
        }
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
                if self.terrain.as_ref().map(|(k, _)| *k) != Some(key) {
                    let img =
                        crate::terrain::bake(g, w.biome, &paths, Some(&art.tiles_img), sunny_img);
                    let tex = Texture2D::from_image(&img);
                    tex.set_filter(FilterMode::Nearest);
                    self.terrain = Some((key, tex));
                }
                if let Some((_, tex)) = &self.terrain {
                    draw_grid_art(w, cam, tex);
                }
            } else {
                draw_room_art(art, w);
            }
            draw_drill_trail(Some(art), &w.player, alpha);
            draw_telegraphs(w, alpha);
            draw_sorted(art, w, alpha);
            draw_player_fx(&w.player, alpha);
            draw_fx(w);
            draw_darkness(art, w, alpha, cam);
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

    /// 컷씬 일러스트를 게임 화면 전체에(docs/decisions/0039). 원본 1px = 화면 배율/2 px(초상화와 같은 2배 밀도).
    /// 홀수 배율에서는 원본 픽셀이 1.5배처럼 고르지 않게 늘어난다 — 그래서 임시 그림은 1px 디더를 쓰지 않는다.
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
        for p in illust::particles(spec.fx, t) {
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
        draw_hud_bars(ui, w);
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

/// 초상화(원본 88px, 대화창 칸 44 논리 px)를 화면에 찍을 크기(화면 px).
/// 배율이 짝수면 칸에 꼭 맞고(원본 1px = 화면 배율/2), 홀수(3, 5…)면 내림해 조금 작게(픽셀을 고르게, 0015).
/// 배율 1(세로로 든 폰)은 반으로 줄여 칸에 맞춘다 — 예전엔 내림이 0이 되어 1배로 올려 칸의 두 배로 넘쳤다(사람 피드백 2026-10-25).
fn portrait_px(scale: f32) -> f32 {
    let k = (scale / 2.0).floor();
    if k >= 1.0 {
        art::PORTRAIT * k
    } else {
        art::PORTRAIT * 0.5 * scale.max(1.0).floor()
    }
}

#[cfg(test)]
mod portrait_tests {
    use super::*;

    #[test]
    fn portrait_never_overflows_its_box() {
        for s in 1..=8 {
            let scale = s as f32;
            let px = portrait_px(scale);
            assert!(px <= 44.0 * scale, "배율 {s}: {px} > 칸 {}", 44.0 * scale);
            assert!(px >= 44.0 * scale * 0.6, "배율 {s}: 너무 작다 {px}");
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
        // 다음 먹기까지: 위에서부터 어둡게 덮였다가 걷힌다(0038).
        let cd = w.player.item_cd / (t::ITEM_COOLDOWN * w.player.eat_cd_mul);
        if cd > 0.0 {
            let h = (size * cd).round();
            ui.rect(
                Rect::new(r.x, r.y, size, h),
                Color::new(0.0, 0.0, 0.0, 0.55),
            );
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

/// 격자 방(필드·던전) 그림: 구운 지도(terrain.rs, 0062)의 보이는 부분 + 매 프레임 바뀌는 칸.
/// 특별한 칸: 문(열림 = 바닥, 잠김 = 뿌리 창살), 굴 입구, 출구, 저장 지점(빛나는 버섯 고리).
fn draw_grid_art(w: &World, cam: Vec2, baked: &Texture2D) {
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
    draw_circle(
        x,
        y - 12.0 + bob,
        9.0,
        Color::new(0.72, 0.62, 0.95, glow * 0.35),
    );
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
    for k in 0..2 {
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
    // 바닥에 퍼진 실
    for k in 0..5 {
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
    // 뿌리: 벼랑 위로 뻗는 굵은 뿌리 넷.
    for (dx, dy) in [(-30.0, 4.0), (-14.0, 6.0), (14.0, 6.0), (30.0, 4.0)] {
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
    // 뿌리
    for (dx, len) in [(-26.0, 18.0), (-12.0, 10.0), (12.0, 10.0), (26.0, 18.0)] {
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
        _ => {
            let s = sn::MUSH_FRAME;
            for (k, dx) in [-8.0f32, 0.0, 8.0].iter().enumerate() {
                let pulse = 0.6 + 0.3 * (time * 1.8 + k as f32).sin();
                let f = ((time * 4.0 + k as f32 * 1.3) as u32 % 4) as f32;
                let (x, y) = (
                    (foot.x + dx - s / 2.0).round(),
                    (foot.y - s + (k % 2) as f32 * 2.0).round(),
                );
                draw_circle(
                    x + 8.0,
                    y + 10.0,
                    7.0,
                    Color::new(0.5, 0.85, 1.0, 0.12 * pulse),
                );
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
    for (i, n) in w.neighbors.iter().enumerate() {
        let big = n.who == world::Who::Bori;
        if n.who == world::Who::Momo {
            // 모모는 빈 자리 그루터기 위에 앉는다: 그루터기(칸 아래 끝) 뒤에 그린다, 그림자 없이.
            list.push((n.pos.y + 9.0, Drawable::Neighbor(i)));
            continue;
        }
        list.push((n.pos.y + 6.0, Drawable::Neighbor(i)));
        let rx = if big { 12.0 } else { 6.0 };
        draw_ellipse(n.pos.x, n.pos.y + 6.0, rx, 2.5, 0.0, SHADOW_SOFT);
    }
    if let Some(v) = &w.adv.village {
        for (c, at) in &w.adv.marks {
            match c {
                'Y' => {
                    list.push((at.y + 8.0, Drawable::Tree(*at, v.growth)));
                    // 깨울 재주가 있으면 밑동이 빛난다(그림자처럼 먼저, 0061).
                    if v.tree_ready {
                        draw_tree_glow(*at + vec2(0.0, 14.0), w.time);
                    }
                }
                // 물웅덩이는 바닥: 맨 먼저.
                'W' => draw_puddle(*at, w.time),
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
        list.push((ep.y + e.radius(), Drawable::Enemy(i)));
        if !e.hittable() {
            continue; // 땅속: 그림자 없이 둔덕만.
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
    for m in &w.mines {
        draw_mine(m, w.time);
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
        draw_player_halo(pp, w.time);
    }
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, d) in list {
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
                    PropKind::Stall => draw_stall(r),
                    PropKind::Seat => draw_seat(r, w.momo_wins),
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
                if w.adv.village.is_some_and(|v| v.tree_ready) {
                    draw_tree_motes(at + vec2(0.0, 14.0), w.time);
                }
            }
            Drawable::Decor(i, at) => match &art.sunny {
                Some(sunny) if matches!(i, 0 | 2) => draw_decor_sunny(sunny, i, at, w.time),
                _ => draw_decor(i, at, w.time),
            },
            Drawable::DreamLoop(at) => draw_dream_loop(at, w.time),
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
    for l in &w.loot {
        draw_loot(l, alpha, w.time, true);
    }
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.65 + 0.25 * ((w.time * 2.0 + x * 0.1).sin());
            // 팩이 있으면 파란 버섯 4칸(0066). 굽기에서는 빼 두었다.
            if let Some(sunny) = &art.sunny {
                let s = crate::sunny::MUSH_FRAME;
                let f = ((w.time * 4.0 + x * 0.13) as u32 % 4) as f32;
                draw_circle(
                    x + 8.0,
                    y + 10.0,
                    7.0,
                    Color::new(0.5, 0.85, 1.0, 0.10 * pulse),
                );
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
            let k = (wt / e.windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.45 * k, 1.0 - 0.45 * k, 1.0);
            if matches!(e.state, enemy::State::Aim { .. }) {
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
        for k in 0..2 {
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
            enemy::State::Guard { .. } => {
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
            _ => {}
        }
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
    // 체력 점(맞은 적만: 처음부터 숫자를 늘어놓지 않는다). 보스는 화면 아래 막대.
    if !e.is_boss() && (e.hp < e.max_hp || sp.behavior == crate::species::Behavior::Dummy) {
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
const TELEGRAPH_FILL: (f32, f32) = (0.03, 0.16);
const TELEGRAPH_EDGE: (f32, f32) = (0.15, 0.5);

/// 공격 예고(바닥, 적·플레이어보다 먼저 그린다): 맞을 자리를 붉게, 공격 순간이 가까울수록 진하게.
/// - 돌격형(떨기): 돌진할 길(몸 폭 × 돌진 거리, 벽에서 끊김)을 띠로. 방향은 지금 보는 쪽(떨기 끝에 굳는다).
/// - 던지기(조준): 도토리가 날아갈 줄.
/// - 뛰기(웅크림·체공): 내려앉을 자리 고리, 체공 중엔 가득.
///
/// 진하기 k = 예고 경과 / 예고 시간(0→1). 마지막 15%는 깜빡여 "지금"을 알린다.
fn draw_telegraphs(w: &World, alpha: f32) {
    use crate::species::Behavior;
    for e in w.enemies.iter().filter(|e| e.alive && !e.asleep) {
        let sp = e.sp();
        let pos = lerp(e.prev_pos, e.pos, alpha);
        let (k, kind) = match e.state {
            enemy::State::Windup { time }
                if matches!(
                    sp.behavior,
                    Behavior::Charger | Behavior::Chief | Behavior::Kongi
                ) =>
            {
                ((time / e.windup()).min(1.0), 0)
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
            Color::new(1.0, 0.62, 0.6, TELEGRAPH_EDGE.1 + 0.12)
        } else {
            Color::new(0.95, 0.38, 0.38, edge_a)
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
            1 => {
                let dir = e.facing.normalize_or(vec2(1.0, 0.0));
                let start = pos + dir * (e.radius() + 2.0);
                let len = w.room.ray_len(start, dir, t::PROJECTILE_RADIUS, 150.0);
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
            StrikeKind::Stomp => PINK,
        };
        match s.shape {
            Shape::Circle { center, r } => draw_circle_lines(center.x, center.y, r, 1.0, col),
            Shape::Sector { origin, dir, r } => draw_sector_lines(origin, dir, r, col),
        }
    }
}

/// 체력 핍(왼쪽 위)·굴파기 쿨다운·SP 게이지(오른쪽 위). 화면 패스에서 게임 px 격자로(ui.rect).
fn draw_hud_bars(ui: &Ui, w: &World) {
    let dark = Color::new(0.2, 0.15, 0.2, 1.0);
    for i in 0..w.player.stats.max_hp {
        let c = if i < w.player.hp { world::MOLE } else { dark };
        ui.rect(Rect::new(3.0 + i as f32 * 7.0, 1.0, 6.0, 6.0), c);
    }
    let cd = (w.player.burrow_cd / w.player.burrow_cd_full()).min(1.0);
    let bw = 24.0;
    let x = t::VIEW_W - 3.0 - bw;
    ui.rect(Rect::new(x, 2.0, bw, 4.0), dark);
    ui.rect(
        Rect::new(x, 2.0, (bw * (1.0 - cd).max(0.0)).round(), 4.0),
        world::DIRT,
    );
    // SP 게이지(그 아래). 가득이면 깜빡인다 — 필살기를 쓸 수 있다. 색은 캐릭터의 게이지 종류(두더지 = 분노).
    let sp = w.player.sp / t::SP_MAX;
    let full = w.player.sp_full();
    ui.rect(Rect::new(x, 8.0, bw, 3.0), dark);
    let gc = w.character.gauge().color();
    let gold = if full && ((w.time * 6.0) as i32) % 2 == 0 {
        Color::new(1.0, 1.0, 0.85, 1.0)
    } else {
        gc
    };
    ui.rect(Rect::new(x, 8.0, (bw * sp).round(), 3.0), gold);
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
