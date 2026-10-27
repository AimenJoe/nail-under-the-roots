//! 그림자: 해 그림자 행렬, 모닥불 그림자, 실루엣 재질, 그림자 패스 표시.

use super::*;

thread_local! {
    /// 그림자 패스 중인가(0106): 그리기 함수 안의 빛·표시(거품·별·고리·Z·체력 점·빛 알갱이)는 그림자를 드리우지 않는다.
    pub(super) static SHADOW_PASS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// 지금 그림자 모양만 모으는 중이면 참. 빛·표시·바닥에 깔린 것은 이때 그리지 않는다.
pub(super) fn shadow_pass() -> bool {
    SHADOW_PASS.with(|c| c.get())
}

/// 그림자 패스의 빛(0103): 가장 가까운 모닥불 자리(불 밑동)와 밝은 곳인지.
#[derive(Clone, Copy)]
pub(super) struct ShadowLight {
    pub(super) fire: Option<Vec2>,
    pub(super) bright: bool,
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

/// 해 그림자 행렬(0102): 발 줄 y = `foot`는 그대로, 그 위로 h만큼 높은 점은 바닥 위 `k * h`만큼 떨어진 자리로.
/// x' = x + k.x·(foot − y), y' = foot + k.y·(foot − y). k.y > 0이면 그림자가 앞(아래)으로 눕는다.
pub(super) fn sun_shear(foot: f32, k: Vec2) -> Mat4 {
    Mat4::from_cols(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(-k.x, -k.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(k.x * foot, (1.0 + k.y) * foot, 0.0, 1.0),
    )
}

/// 해 그림자 색(0102). 실루엣 재질(셰이더 안 같은 값)과 지도 굽기가 같이 쓴다. 알파 = 진하기.
pub const SUN_SHADE: Color = Color::new(0.10, 0.06, 0.14, t::SUN_SHADOW_ALPHA);

/// 실루엣 재질(해 그림자, 0102): 그린 자리(알파가 있는 곳)를 모두 같은 그림자 색(`SUN_SHADE`)으로. 진하기는 정점 색 알파.
pub(super) fn silhouette_material() -> Option<Material> {
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
pub(super) fn cut_material() -> Option<Material> {
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
pub(super) fn draw_cast_shadows(w: &World, alpha: f32, cam: Vec2, fire_silhouettes: bool) {
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
