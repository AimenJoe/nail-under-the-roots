//! 해·모닥불 그림자 모델(0102·0103): 발 자리에서 그림자 방향·길이, 눕히는 행렬, 그림자 색.
//! 실시간 그림자(`render/shadow.rs`)와 지도 굽기(`terrain.rs`·`sunny.rs`)가 같이 쓴다 — 그래서 렌더 아래에 둔다(0170 R-소).

use macroquad::prelude::*;

use crate::tuning as t;

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
pub fn sun_shear(foot: f32, k: Vec2) -> Mat4 {
    Mat4::from_cols(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(-k.x, -k.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(k.x * foot, (1.0 + k.y) * foot, 0.0, 1.0),
    )
}

/// 해 그림자 색(0102). 실루엣 재질(셰이더 안 같은 값)과 지도 굽기가 같이 쓴다. 알파 = 진하기.
pub const SUN_SHADE: Color = Color::new(0.10, 0.06, 0.14, t::SUN_SHADOW_ALPHA);
