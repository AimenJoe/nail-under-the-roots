//! 환경(생물군계). 뿌리를 따라 내려갈수록 꽃 핀 입구 → 뿌리 지대 → 어두운 심층으로 바뀐다.
//! 같은 타일에 색조·장식·어둠을 달리 입혀 표현한다(docs/decisions/0010). 층 구조가 생기기 전까지는 웨이브가 깊이를 대신한다.

use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Meadow,
    Roots,
    Deep,
}

/// 장식 타일(tiles.png 칸).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decor {
    FlowerWhite,
    FlowerYellow,
    FlowerPink,
    Tuft,
    Pebble,
    Mushroom,
    RootVine,
    Moss,
}

impl Biome {
    /// 웨이브 → 깊이. 1~2 입구, 3~4 뿌리 지대, 5~ 심층.
    pub fn for_wave(wave: u32) -> Self {
        match wave {
            0..=2 => Biome::Meadow,
            3..=4 => Biome::Roots,
            _ => Biome::Deep,
        }
    }

    pub fn name(self) -> crate::i18n::Tr {
        match self {
            Biome::Meadow => |s| s.biome_meadow,
            Biome::Roots => |s| s.biome_roots,
            Biome::Deep => |s| s.biome_deep,
        }
    }

    pub fn clear(self) -> Color {
        match self {
            Biome::Meadow => Color::new(0.16, 0.20, 0.14, 1.0),
            Biome::Roots => Color::new(0.13, 0.11, 0.12, 1.0),
            Biome::Deep => Color::new(0.05, 0.04, 0.08, 1.0),
        }
    }

    /// 발밑 먼지 색(0081): 들판은 마른 흙빛, 굴은 어두운 흙, 심층은 차가운 잿빛.
    pub fn dust(self) -> Color {
        match self {
            Biome::Meadow => Color::new(0.80, 0.72, 0.55, 1.0),
            Biome::Roots => Color::new(0.58, 0.46, 0.38, 1.0),
            Biome::Deep => Color::new(0.50, 0.50, 0.62, 1.0),
        }
    }

    pub fn floor_tint(self) -> Color {
        match self {
            Biome::Meadow => WHITE,
            Biome::Roots => Color::new(0.80, 0.76, 0.80, 1.0),
            Biome::Deep => Color::new(0.48, 0.48, 0.62, 1.0),
        }
    }

    pub fn wall_tint(self) -> Color {
        match self {
            Biome::Meadow => WHITE,
            Biome::Roots => Color::new(0.78, 0.72, 0.76, 1.0),
            Biome::Deep => Color::new(0.42, 0.40, 0.56, 1.0),
        }
    }

    /// 바닥 기본이 흙인가(심층). 아니면 풀.
    pub fn dirt_floor(self) -> bool {
        self == Biome::Deep
    }

    /// 기본 바닥 위에 반대 재질 얼룩(풀 위 흙, 흙 위 풀)을 덮을 확률. 칸 경계가 안 보이게 한다.
    pub fn patch_chance(self) -> f32 {
        match self {
            Biome::Meadow => 0.07,
            Biome::Roots => 0.34,
            Biome::Deep => 0.22,
        }
    }

    /// 화면 가장자리 어둠(0 = 없음)과 플레이어 주변 빛 반지름(px).
    pub fn darkness(self) -> (f32, f32) {
        match self {
            Biome::Meadow => (0.0, 0.0),
            Biome::Roots => (0.38, 150.0),
            Biome::Deep => (0.78, 92.0),
        }
    }

    /// 칸 해시(0..1)로 장식을 고른다. 누적 확률표.
    pub fn decor(self, roll: f32, pick: u32) -> Option<Decor> {
        let table: &[(f32, &[Decor])] = match self {
            Biome::Meadow => &[
                (
                    0.30,
                    &[Decor::FlowerWhite, Decor::FlowerYellow, Decor::FlowerPink],
                ),
                (0.42, &[Decor::Tuft]),
                (0.45, &[Decor::Pebble]),
            ],
            Biome::Roots => &[
                (0.06, &[Decor::FlowerWhite, Decor::FlowerPink]),
                (0.16, &[Decor::Tuft]),
                (0.30, &[Decor::RootVine]),
                (0.36, &[Decor::Pebble]),
                (0.40, &[Decor::Moss]),
            ],
            Biome::Deep => &[
                (0.14, &[Decor::Moss]),
                (0.23, &[Decor::Mushroom]),
                (0.38, &[Decor::RootVine]),
                (0.43, &[Decor::Pebble]),
            ],
        };
        for (limit, opts) in table {
            if roll < *limit {
                return Some(opts[pick as usize % opts.len()]);
            }
        }
        None
    }
}

/// 칸 좌표 해시 → 0..1. 방마다 같은 그림이 나온다.
pub fn hash01(x: i32, y: i32, salt: u32) -> (f32, u32) {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    ((h & 0xFFFF) as f32 / 65536.0, h >> 16)
}
