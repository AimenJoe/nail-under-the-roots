//! Sunnyside World 팩에서 쓰는 그림의 자리표(docs/decisions/0065·0066).
//!
//! 팩 원본은 레포 밖(`assets/vendor/sunnyside/`, 0010)이고, 이 표는 팩을 보고 고른 칸 좌표다(0064).
//! 좌표는 `Tileset/spr_tileset_sunnysideworld_16px.png`(1024×1024)의 픽셀 사각형 (x, y, w, h).
//! 팩이 없으면 아무것도 쓰지 않고 직접 그린 그림으로 그린다.

use macroquad::prelude::*;

pub type R = (u16, u16, u16, u16);

/// 작은 꽃: 파랑·빨강·하양 × 크기 넷.
pub const FLOWERS_BLUE: [R; 4] = [
    (524, 18, 4, 5),
    (533, 18, 5, 6),
    (540, 23, 4, 6),
    (552, 19, 7, 8),
];
pub const FLOWERS_RED: [R; 4] = [
    (524, 34, 4, 5),
    (533, 34, 5, 6),
    (540, 39, 4, 6),
    (552, 35, 7, 8),
];
pub const FLOWERS_WHITE: [R; 4] = [
    (524, 51, 4, 6),
    (532, 51, 7, 6),
    (538, 57, 6, 6),
    (551, 52, 9, 8),
];
/// 풀 포기·클로버.
pub const GRASS: [R; 8] = [
    (435, 66, 13, 9),
    (451, 65, 13, 10),
    (467, 65, 13, 10),
    (483, 66, 13, 9),
    (433, 18, 15, 13),
    (449, 19, 15, 12),
    (465, 19, 15, 12),
    (481, 18, 15, 13),
];
/// 조약돌.
pub const PEBBLES: [R; 3] = [(530, 69, 11, 8), (546, 66, 9, 6), (512, 65, 16, 11)];
/// 그루터기(바닥에 붙는 키 작은 것).
pub const STUMPS: [R; 2] = [(496, 81, 16, 14), (512, 81, 16, 14)];
/// 바위(큰 것부터).
pub const BOULDERS: [R; 3] = [(787, 340, 26, 25), (820, 343, 24, 22), (855, 349, 18, 16)];
/// 덤불: 그냥·빨간 열매·주황 꽃·작은 것.
#[allow(dead_code)] // 다음 차례: 들판 덤불
pub const BUSH: R = (788, 22, 24, 21);
pub const BUSH_BERRY: R = (788, 54, 24, 21);
pub const BUSH_FLOWER: R = (788, 86, 24, 21);
#[allow(dead_code)]
pub const BUSH_SMALL: R = (816, 65, 16, 15);
/// 장터 가판(차양 + 판대 + 붉은 천). 쉼터·마을 가게(0077).
pub const STALL: R = (604, 592, 24, 48);
/// 판대 윗면: 가판 그림 위쪽에서 몇 px 아래인가(물건을 올려놓는 줄).
pub const STALL_COUNTER: f32 = 23.0;
/// 나무 긴 의자. 마을 꾸미기 "통나무 의자"(0077).
pub const BENCH: R = (652, 611, 24, 13);
/// 흰 들꽃 화분과 들꽃 한 포기. 마을 꾸미기 "들꽃 화분"(0077).
pub const POT_WILDFLOWER: R = (853, 512, 8, 16);
pub const WILDFLOWERS: R = (869, 513, 8, 12);
/// HUD 아이콘: 하트(체력 한 칸), 새싹(씨앗).
pub const HEART: R = (707, 756, 9, 8);
pub const SPROUT: R = (660, 707, 10, 11);
/// 돌로 두른 모닥불 자리.
pub const FIRE_PIT: R = (596, 343, 23, 20);
/// 잠긴 문 울타리(0097~0099): 가로(남북 문)는 (39,2), 세로(동서 문)는 (40,3). 둘 다 가운데 칸이라
/// 가로대가 벽에서 벽까지 이어진다. 두 겹을 `FENCE_STACK`만큼 엇갈려 쌓고,
/// 보라 실은 두 겹 한가운데 줄(칸 안 px)을 `FENCE_WRAP` 폭으로 감는다.
pub const FENCE_H: R = (624, 32, 16, 16);
pub const FENCE_V: R = (640, 48, 16, 16);
pub const FENCE_STACK: [f32; 2] = [-3.0, 3.0];
pub const FENCE_RAIL_Y: f32 = 8.0;
pub const FENCE_RAIL_X: f32 = 7.5;
pub const FENCE_WRAP: f32 = 5.0;
/// 불꽃 4칸(`Elements/VFX/Fire/spr_deco_fire_02_strip4.png`, 8×12 한 칸).
pub const FIRE_FRAME: (f32, f32) = (8.0, 12.0);
/// 파란 버섯 4칸(`Elements/Plants/spr_deco_mushroom_blue_01_strip4.png`, 16×16 한 칸).
pub const MUSH_FRAME: f32 = 16.0;

pub fn rect(r: R) -> Rect {
    Rect::new(r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)
}

/// 팩 한 칸을 발(아래 가운데)이 `foot`에 오게 그린다.
pub fn draw_at_foot(sheet: &Texture2D, r: R, foot: Vec2, tint: Color) {
    let src = rect(r);
    draw_texture_ex(
        sheet,
        (foot.x - src.w / 2.0).round(),
        (foot.y - src.h).round(),
        tint,
        DrawTextureParams {
            source: Some(src),
            ..Default::default()
        },
    );
}

/// CPU 그림(지도 굽기)에 팩 한 칸을 찍는다: 왼쪽 위가 (x, y). `skip(x, y)`가 참인 px는 건너뛴다(벽 위).
pub fn stamp(
    dst: &mut Image,
    sheet: &Image,
    r: R,
    x: i32,
    y: i32,
    tint: Color,
    skip: impl Fn(i32, i32) -> bool,
) {
    let (w, h) = (dst.width as i32, dst.height as i32);
    for sy in 0..r.3 as i32 {
        for sx in 0..r.2 as i32 {
            let s = sheet.get_pixel(r.0 as u32 + sx as u32, r.1 as u32 + sy as u32);
            if s.a < 0.5 {
                continue;
            }
            let (px, py) = (x + sx, y + sy);
            if px < 0 || py < 0 || px >= w || py >= h || skip(px, py) {
                continue;
            }
            let i = (py * w + px) as usize * 4;
            dst.bytes[i] = (s.r * tint.r * 255.0) as u8;
            dst.bytes[i + 1] = (s.g * tint.g * 255.0) as u8;
            dst.bytes[i + 2] = (s.b * tint.b * 255.0) as u8;
            dst.bytes[i + 3] = 255;
        }
    }
}

/// 구운 장식 하나의 그림자(0102·0103): 방향·길이(`shade::shadow_dir` × 높이 비율)와,
/// 어두운 곳이면 옅어지는 기준(불 자리, 닿는 거리). 진하기는 `alpha`.
pub struct StampShadow {
    pub k: Vec2,
    pub alpha: f32,
    pub fade: Option<(Vec2, f32)>,
}

/// 해·모닥불 그림자를 CPU 그림(지도 굽기)에 찍는다: 팩 한 칸의 모양을 발 줄(왼쪽 위 (x, y) + 높이) 축으로 눕힌다.
/// 실시간 그림자(`shade::sun_shear`)와 같은 식. 같은 자리를 두 번 어둡게 하지 않는다. `skip`(벽 위)은 건너뛴다.
pub fn stamp_sun_shadow(
    dst: &mut Image,
    sheet: &Image,
    r: R,
    (x, y): (i32, i32),
    sh: &StampShadow,
    skip: impl Fn(i32, i32) -> bool,
) {
    let k = sh.k;
    let shade = crate::shade::SUN_SHADE;
    let (w, h) = (dst.width as i32, dst.height as i32);
    let foot = y + r.3 as i32 - 1;
    let mut spots: Vec<(i32, i32)> = Vec::new();
    for sy in 0..r.3 as i32 {
        for sx in 0..r.2 as i32 {
            if sheet
                .get_pixel(r.0 as u32 + sx as u32, r.1 as u32 + sy as u32)
                .a
                < 0.5
            {
                continue;
            }
            let up = (foot - (y + sy)) as f32;
            let px = x + sx + (k.x * up).round() as i32;
            let py = foot + (k.y * up).round() as i32;
            if px < 0 || py < 0 || px >= w || py >= h || skip(px, py) {
                continue;
            }
            spots.push((px, py));
        }
    }
    spots.sort_unstable();
    spots.dedup();
    for (px, py) in spots {
        let fade = sh.fade.map_or(1.0, |(f, reach)| {
            let d = vec2(px as f32, py as f32).distance(f);
            let u = ((d - 0.55 * reach) / (0.45 * reach)).clamp(0.0, 1.0);
            1.0 - u * u * (3.0 - 2.0 * u)
        });
        let a = sh.alpha * fade;
        if a <= 0.0 {
            continue;
        }
        let i = (py as usize * w as usize + px as usize) * 4;
        for (c, s) in dst.bytes[i..i + 3]
            .iter_mut()
            .zip([shade.r, shade.g, shade.b])
        {
            *c = (*c as f32 * (1.0 - a) + s * 255.0 * a) as u8;
        }
    }
}
