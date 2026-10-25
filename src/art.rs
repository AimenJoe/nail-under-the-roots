//! 그림 에셋. 지금은 직접 그린 그림(tools/art/*.py가 원본)만. Sunnyside World 팩은 레포 밖에서 들어온다(docs/decisions/0010).
//! 파일이 없으면 `None`이고 렌더는 도형 그리기로 돌아간다.

use macroquad::prelude::*;

pub struct Art {
    /// Naïl 시트: 16×16 칸, 줄 0 idle·walk, 줄 1 attack·hurt·dead, 줄 2 흙더미·그림자, 줄 3~5 흰 실루엣.
    pub nail: Texture2D,
    /// 대체 타일셋(tools/art/tiles.py).
    pub tiles: Texture2D,
    /// 어둠 마스크(가운데 투명, 바깥 검정). 코드로 만든다.
    pub dark: Texture2D,
    /// 적·이웃(tools/art/critters.py): 16×16, 줄 = species.rs의 sheet_row, 줄 6 라쿤. 아래 CRITTER_ROWS줄은 흰 실루엣.
    pub critters: Option<Texture2D>,
    /// 대화창 초상화: 88×88 칸 가로로(tools/art/portraits.py). 게임 픽셀의 2배 밀도. 없으면 색 네모.
    pub portraits: Option<Texture2D>,
}

/// 초상화 한 칸(원본 px).
pub const PORTRAIT: f32 = 88.0;

pub const CELL: f32 = 16.0;

/// critters.png에서 실루엣 줄까지의 거리(그림 줄 수).
pub const CRITTER_ROWS: u32 = 7;
/// critters.png의 라쿤 줄.
pub const RACCOON_ROW: u32 = 6;

impl Art {
    pub async fn load() -> Option<Self> {
        let nail = load_texture("art/nail.png").await.ok()?;
        let tiles = load_texture("art/tiles.png").await.ok()?;
        nail.set_filter(FilterMode::Nearest);
        tiles.set_filter(FilterMode::Nearest);
        let portraits = load_texture("art/portraits.png").await.ok();
        if let Some(p) = &portraits {
            p.set_filter(FilterMode::Nearest);
        }
        let critters = load_texture("art/critters.png").await.ok();
        if let Some(c) = &critters {
            c.set_filter(FilterMode::Nearest);
        }
        Some(Self {
            nail,
            tiles,
            dark: dark_mask(),
            critters,
            portraits,
        })
    }
}

/// 64×64 원형 어둠 마스크. 가운데는 투명, 가장자리로 갈수록 검정. 부드러운 빛이라 Linear로 늘린다.
fn dark_mask() -> Texture2D {
    let n = 64u16;
    let mut img = Image::gen_image_color(n, n, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let d = (dx * dx + dy * dy).sqrt();
            let t = ((d - 0.35) / 0.65).clamp(0.0, 1.0);
            let a = t * t * (3.0 - 2.0 * t);
            img.set_pixel(x as u32, y as u32, Color::new(0.0, 0.0, 0.0, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

/// 시트의 칸 사각형.
pub fn cell(col: u32, row: u32) -> Rect {
    inset(Rect::new(col as f32 * CELL, row as f32 * CELL, CELL, CELL))
}

/// 시트에서 칸을 잘라 쓸 때 가장자리를 아주 조금(0.01px) 안으로(예방). 카메라·배율의 소수점 오차로
/// 칸 경계를 표본하면 이웃 칸의 한 줄이 묻어 나올 수 있다. Nearest라 그려지는 픽셀은 같다.
/// (바위 위 "얼룩"의 실제 원인은 이게 아니라 tiles.py의 벽 앞면이 칸 밖으로 한 줄 넘친 것이었다.)
pub fn inset(r: Rect) -> Rect {
    const E: f32 = 0.01;
    Rect::new(r.x + E, r.y + E, r.w - 2.0 * E, r.h - 2.0 * E)
}
