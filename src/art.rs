//! 그림 에셋. 지금은 직접 그린 그림(tools/art/*.py가 원본)만. Sunnyside World 팩은 레포 밖에서 들어온다(docs/decisions/0010).
//! 파일이 없으면 `None`이고 렌더는 도형 그리기로 돌아간다.

use macroquad::prelude::*;

pub struct Art {
    /// Naïl 시트: 16×16 칸, 줄 0 idle·walk, 줄 1 attack·hurt·dead, 줄 2 흙더미·그림자, 줄 3~5 흰 실루엣.
    pub nail: Texture2D,
    /// 털빛 1~3(tools/art/nail.py, 저널 보상 docs/decisions/0048). 없으면 기본 털.
    pub nail_furs: Vec<Option<Texture2D>>,
    /// 대체 타일셋(tools/art/tiles.py).
    pub tiles: Texture2D,
    /// 같은 타일셋의 CPU 쪽 그림: 지도 굽기(terrain.rs)가 장식을 찍는다.
    pub tiles_img: Image,
    /// Sunnyside World(레포 밖 `assets/vendor/`, 0010·0064). 없으면(공개 CI 빌드) 직접 그린 그림으로.
    pub sunny: Option<Sunny>,
    /// 어둠 마스크(가운데 투명, 바깥 검정). 코드로 만든다.
    pub dark: Texture2D,
    /// 적·이웃(tools/art/critters.py): 16×16, 줄 = species.rs의 sheet_row, 줄 6 라쿤, 줄 7 모모, 줄 8 돌아온 모모, 줄 9 도토, 줄 10 돌아온 도토, 줄 11 수수, 줄 12 콩이, 줄 13 돌아온 콩이. 아래 CRITTER_ROWS줄은 흰 실루엣.
    pub critters: Option<Texture2D>,
    /// 대화창 초상화: 88×88 칸, 한 줄 12칸(tools/art/portraits.py). 게임 픽셀의 2배 밀도. 없으면 색 네모.
    pub portraits: Option<Texture2D>,
    /// 들쥐 대장·보리(tools/art/chief.py): 32×32, 줄 0 물든 대장, 줄 1 보리, 줄 2·3 흰 실루엣.
    pub chief: Option<Texture2D>,
    /// 컷씬 일러스트(illust::Illust::ALL 순서, docs/decisions/0039). 없으면 그 장면은 어두운 바탕.
    pub illust: Vec<Option<Texture2D>>,
    /// 타이틀 바탕 640×360(2배 밀도, `tools/art/illust.py` title, 0083). 없으면 예전 선 바탕.
    pub title: Option<Texture2D>,
}

/// chief.png 한 칸.
/// Sunnyside 팩에서 쓰는 그림. 팩 원본은 공개 레포에 넣지 않는다(0010).
pub struct Sunny {
    /// 둥근 나무 4칸(32×34, 바람에 흔들림).
    pub tree: Texture2D,
    /// 16px 타일셋(1024×1024). 자리는 `sunny.rs` 표.
    pub sheet: Texture2D,
    /// 같은 타일셋의 CPU 쪽 그림: 지도 굽기가 꽃·풀·조약돌을 찍는다.
    pub sheet_img: Image,
    /// 불꽃 4칸, 파란 버섯 4칸.
    pub fire: Texture2D,
    pub mush: Texture2D,
}

/// 팩 폴더(assets 기준).
const SUNNY: &str = "vendor/sunnyside/Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets";
/// 둥근 나무 한 칸 크기와 잎(윗부분) 높이.
pub const SUNNY_TREE: (f32, f32) = (32.0, 34.0);
pub const SUNNY_CROWN_H: f32 = 22.0;

async fn load_sunny() -> Option<Sunny> {
    let tree = load_texture(&format!(
        "{SUNNY}/Elements/Plants/spr_deco_tree_01_strip4.png"
    ))
    .await
    .ok()?;
    let sheet_img = load_image(&format!(
        "{SUNNY}/Tileset/spr_tileset_sunnysideworld_16px.png"
    ))
    .await
    .ok()?;
    let sheet = Texture2D::from_image(&sheet_img);
    let fire = load_texture(&format!(
        "{SUNNY}/Elements/VFX/Fire/spr_deco_fire_02_strip4.png"
    ))
    .await
    .ok()?;
    let mush = load_texture(&format!(
        "{SUNNY}/Elements/Plants/spr_deco_mushroom_blue_01_strip4.png"
    ))
    .await
    .ok()?;
    for t in [&tree, &sheet, &fire, &mush] {
        t.set_filter(FilterMode::Nearest);
    }
    Some(Sunny {
        tree,
        sheet,
        sheet_img,
        fire,
        mush,
    })
}

pub const CHIEF_CELL: f32 = 32.0;
/// chief.png의 줄: 물든 대장 / 돌아온 보리. 실루엣은 + CHIEF_ROWS.
pub const CHIEF_TAINTED: u32 = 0;
pub const CHIEF_BORI: u32 = 1;
pub const CHIEF_ROWS: u32 = 2;

/// chief.png의 칸 사각형.
pub fn chief_cell(col: u32, row: u32) -> Rect {
    inset(Rect::new(
        col as f32 * CHIEF_CELL,
        row as f32 * CHIEF_CELL,
        CHIEF_CELL,
        CHIEF_CELL,
    ))
}

/// 초상화 한 칸(원본 px).
pub const PORTRAIT: f32 = 88.0;
/// 초상화 시트 한 줄의 칸 수. 칸이 늘어 한 줄이면 2,048px를 넘는다(오래된 폰의 텍스처 한도) → 12칸씩 줄을 바꾼다.
pub const PORTRAIT_COLS: u32 = 12;

pub const CELL: f32 = 16.0;

/// critters.png에서 실루엣 줄까지의 거리(그림 줄 수).
pub const CRITTER_ROWS: u32 = 14;
/// critters.png의 라쿤 줄.
pub const RACCOON_ROW: u32 = 6;
/// critters.png의 모모(물든 굴토끼) 줄(docs/decisions/0029).
pub const MOMO_ROW: u32 = 7;
/// 돌아온 모모(꽃이 핀 모습, docs/decisions/0046).
pub const MOMO_BLOOM_ROW: u32 = 8;
/// 도토(물든 청설모, 둘째 길 보스)와 돌아온 도토(docs/decisions/0052).
pub const DOTO_ROW: u32 = 9;
pub const DOTO_BACK_ROW: u32 = 10;
/// 땅강아지 수수(첫 쉼터의 장인, docs/decisions/0053).
pub const SUSU_ROW: u32 = 11;
/// 콩이(물든 세띠아르마딜로, 셋째 길 보스)와 돌아온 콩이(docs/decisions/0054).
pub const KONGI_ROW: u32 = 12;
pub const KONGI_BACK_ROW: u32 = 13;

impl Art {
    /// 털빛 n의 Naïl 시트(0 또는 없으면 기본).
    pub fn nail_fur(&self, n: u8) -> &Texture2D {
        (n as usize)
            .checked_sub(1)
            .and_then(|i| self.nail_furs.get(i))
            .and_then(|t| t.as_ref())
            .unwrap_or(&self.nail)
    }

    pub async fn load() -> Option<Self> {
        let nail = load_texture("art/nail.png").await.ok()?;
        let tiles_img = load_image("art/tiles.png").await.ok()?;
        let tiles = Texture2D::from_image(&tiles_img);
        nail.set_filter(FilterMode::Nearest);
        let mut nail_furs = Vec::new();
        for n in 1..crate::journal::FURS {
            let t = load_texture(&format!("art/nail_fur{n}.png")).await.ok();
            if let Some(t) = &t {
                t.set_filter(FilterMode::Nearest);
            }
            nail_furs.push(t);
        }
        tiles.set_filter(FilterMode::Nearest);
        let portraits = load_texture("art/portraits.png").await.ok();
        if let Some(p) = &portraits {
            p.set_filter(FilterMode::Nearest);
        }
        let critters = load_texture("art/critters.png").await.ok();
        if let Some(c) = &critters {
            c.set_filter(FilterMode::Nearest);
        }
        let chief = load_texture("art/chief.png").await.ok();
        if let Some(c) = &chief {
            c.set_filter(FilterMode::Nearest);
        }
        let mut illust = Vec::new();
        for il in crate::illust::Illust::ALL {
            let t = load_texture(&format!("art/illust/{}.png", il.spec().file))
                .await
                .ok();
            if let Some(t) = &t {
                t.set_filter(FilterMode::Nearest);
            }
            illust.push(t);
        }
        // 타이틀 바탕(0083). 컷씬 그림이 아니라 앨범 목록(`Illust::ALL`)에 넣지 않는다.
        let title = load_texture("art/illust/title.png").await.ok();
        if let Some(t) = &title {
            t.set_filter(FilterMode::Nearest);
        }
        Some(Self {
            title,
            nail,
            nail_furs,
            tiles,
            tiles_img,
            sunny: load_sunny().await,
            dark: dark_mask(),
            critters,
            portraits,
            chief,
            illust,
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
