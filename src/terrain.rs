//! 지도 굽기(docs/decisions/0062): 격자 지도(필드·마을·쉼터·던전)를 장소가 바뀔 때 한 장의 그림(월드 1px = 1px)으로 굽는다.
//!
//! 충돌은 그대로 16px 칸이다. 그림만 칸에서 벗어난다.
//! - 벽 윤곽: 칸 가운데 값(벽 1, 바닥 0)을 겹선형으로 잇고 잡음을 더해 0.5에서 자른다. 곧은 벽은 칸 경계를 따라 물결치고,
//!   모서리는 둥글어진다. 바닥 칸 가운데는 늘 바닥이라 걷는 곳을 가리지 않는다.
//! - 3/4 벼랑: 벽 아래쪽 16px(바로 아래가 바닥)은 흙 앞면, 그 위는 윗면(덤불·뿌리·돌).
//! - 바닥: 큰 잡음 띠(밝고 어두운 풀), 반대 재질 얼룩(흙·이끼), 던전은 문에서 방 가운데로 난 흙길.
//! - 그늘: 벼랑 앞면 아래로 몇 px, 옆벽 곁 1~2px.
//! - 장식: 칸 해시 + 저주파 밀도(무리 지어 핀 꽃, 비어 있는 풀밭).
//!
//! 잠긴 문·출구·저장 지점처럼 바뀌는 칸은 굽지 않고 매 프레임 위에 그린다(바닥으로 굽는다).

use macroquad::prelude::*;

use crate::biome::{self, Biome, Decor};
use crate::map::{Cell, Grid};

const T: i32 = 16;
/// 벼랑 앞면 높이(px). 예전 그림의 벽 앞면 한 칸과 같다.
const FACE: u16 = 16;

type Rgb = (u8, u8, u8);

struct Palette {
    floor: [Rgb; 4],
    soil: [Rgb; 3],
    /// 반대 재질 얼룩이 흙인가(아니면 이끼).
    patch_is_soil: bool,
    patch_at: f32,
    path: [Rgb; 2],
    top: [Rgb; 4],
    face: [Rgb; 4],
    roots_on_face: f32,
    roots_on_top: bool,
    stone_top: bool,
}

const OUT: Rgb = (43, 29, 42);
const ROOT: [Rgb; 2] = [(170, 130, 92), (120, 86, 60)];

fn palette(b: Biome) -> Palette {
    let grass = [
        (96, 156, 80),
        (112, 172, 88),
        (82, 138, 72),
        (136, 192, 100),
    ];
    let soil = [(150, 110, 78), (128, 92, 64), (172, 132, 94)];
    let face = [(118, 84, 62), (98, 70, 52), (142, 104, 74), (78, 56, 44)];
    match b {
        Biome::Meadow => Palette {
            floor: grass,
            soil,
            patch_is_soil: true,
            patch_at: 0.75,
            path: [(166, 128, 90), (146, 108, 76)],
            top: [(62, 112, 62), (80, 136, 72), (46, 86, 52), (106, 162, 90)],
            face,
            roots_on_face: 0.12,
            roots_on_top: false,
            stone_top: false,
        },
        Biome::Roots => Palette {
            floor: grass,
            soil,
            patch_is_soil: true,
            patch_at: 0.66,
            path: [(160, 120, 86), (138, 100, 70)],
            top: [(70, 104, 62), (88, 124, 70), (54, 80, 52), (108, 142, 82)],
            face,
            roots_on_face: 0.45,
            roots_on_top: true,
            stone_top: false,
        },
        Biome::Deep => Palette {
            floor: [(128, 94, 70), (140, 104, 78), (110, 80, 60), (156, 118, 88)],
            soil: [(74, 112, 80), (60, 94, 68), (92, 132, 92)],
            patch_is_soil: false,
            patch_at: 0.66,
            path: [(150, 116, 86), (128, 96, 70)],
            top: [
                (96, 90, 104),
                (114, 108, 120),
                (72, 66, 80),
                (136, 130, 140),
            ],
            face,
            roots_on_face: 0.7,
            roots_on_top: true,
            stone_top: true,
        },
        // 흰 실 굴(0165): 보랏빛 흙 위에 흰 실 얼룩(얼룩이 흙이 아니라 실), 연보라 돌 윗면에 뿌리.
        Biome::Silk => Palette {
            floor: [
                (118, 92, 106),
                (130, 104, 118),
                (100, 78, 92),
                (146, 118, 132),
            ],
            soil: [(226, 218, 236), (206, 196, 222), (240, 236, 246)],
            patch_is_soil: false,
            patch_at: 0.60,
            path: [(160, 136, 160), (138, 114, 140)],
            top: [
                (132, 118, 150),
                (150, 138, 168),
                (104, 92, 122),
                (176, 166, 192),
            ],
            face,
            roots_on_face: 0.5,
            roots_on_top: true,
            stone_top: true,
        },
    }
}

fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

fn rnd(x: i32, y: i32, salt: u32) -> f32 {
    (hash(x, y, salt) & 0xFFFF) as f32 / 65536.0
}

/// 값 잡음(0..1): 격자 `scale` px마다 무작위 값, 사이는 부드럽게.
fn vnoise(x: f32, y: f32, scale: f32, salt: u32) -> f32 {
    let (u, v) = (x / scale, y / scale);
    let (i, j) = (u.floor() as i32, v.floor() as i32);
    let (fu, fv) = (u - i as f32, v - j as f32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (su, sv) = (s(fu), s(fv));
    let a = rnd(i, j, salt);
    let b = rnd(i + 1, j, salt);
    let c = rnd(i, j + 1, salt);
    let d = rnd(i + 1, j + 1, salt);
    let top = a + (b - a) * su;
    let bot = c + (d - c) * su;
    top + (bot - top) * sv
}

/// 주기 `period` px로 되풀이되는 값 잡음(먼 벽을 채우는 무늬가 이음새 없이 이어지게). period는 scale의 배수.
fn vnoise_wrap(x: f32, y: f32, scale: f32, salt: u32, period: i32) -> f32 {
    let n = period / scale as i32;
    let (u, v) = (x / scale, y / scale);
    let (i, j) = (u.floor() as i32, v.floor() as i32);
    let (fu, fv) = (u - i as f32, v - j as f32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (su, sv) = (s(fu), s(fv));
    let r = |a: i32, b: i32| rnd(a.rem_euclid(n), b.rem_euclid(n), salt);
    let top = r(i, j) + (r(i + 1, j) - r(i, j)) * su;
    let bot = r(i, j + 1) + (r(i + 1, j + 1) - r(i, j + 1)) * su;
    top + (bot - top) * sv
}

/// 먼 벽(둘레에 바닥이 없는 칸)을 채우는 64×64 무늬: 윗면과 같은 덤불·돌을 테두리 없이.
const PAT: i32 = 64;
fn far_pattern(p: &Palette) -> Vec<Rgb> {
    let mut out = Vec::with_capacity((PAT * PAT) as usize);
    for y in 0..PAT {
        for x in 0..PAT {
            let (xf, yf) = (x as f32, y as f32);
            let c = if p.stone_top {
                let q = vnoise_wrap(xf, yf, 8.0, 42, PAT);
                let crack = (vnoise_wrap(xf, yf, 16.0, 43, PAT) - 0.5).abs() < 0.02;
                if crack {
                    p.top[2]
                } else if q > 0.6 {
                    p.top[1]
                } else {
                    p.top[0]
                }
            } else {
                let q = vnoise_wrap(xf, yf, 4.0, 45, PAT);
                let q2 = vnoise_wrap(xf + 2.0, yf + 2.0, 4.0, 45, PAT);
                let shade = q - q2;
                if shade > 0.09 {
                    p.top[3]
                } else if shade < -0.09 {
                    p.top[2]
                } else if q > 0.55 {
                    p.top[1]
                } else {
                    p.top[0]
                }
            };
            out.push(c);
        }
    }
    out
}

fn solid_cell(g: &Grid, cx: i32, cy: i32) -> bool {
    g.get_i(cx, cy) == Cell::Wall
}

/// 지도가 바뀌었는지 알기 위한 열쇠. 매 프레임 나오는 칸(문·출구·저장 지점)은 바닥으로 셈한다.
pub fn key(g: &Grid, b: Biome, paths: &[(Vec2, Vec2)]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    };
    eat(g.w as u64);
    eat(g.h as u64);
    eat(b as u64);
    for y in 0..g.h {
        for x in 0..g.w {
            eat(u64::from(g.get(x, y) == Cell::Wall));
        }
    }
    for (a, c) in paths {
        eat(a.x as u64 ^ ((a.y as u64) << 16) ^ ((c.x as u64) << 32) ^ ((c.y as u64) << 48));
    }
    h
}

/// 칸의 장식(해시 + 무리 밀도). 굽기와 어둠 뒤 다시 그리기(빛나는 버섯)가 같은 것을 쓴다. (장식, 칸 안 x·y 어긋남)
pub fn decor_at(b: Biome, cx: i32, cy: i32) -> Option<(Decor, f32, f32)> {
    let (roll, pick) = biome::hash01(cx, cy, 7);
    // 무리: 저주파 밀도 0.15~1.85배(꽃밭·빈 풀밭).
    let dens = 0.15 + 1.7 * vnoise(cx as f32, cy as f32, 4.5, 77);
    let d = b.decor((roll / dens).min(0.999), pick)?;
    let ox = ((pick >> 4) % 9) as f32 - 4.0;
    let oy = ((pick >> 8) % 7) as f32 - 3.0;
    Some((d, ox, oy))
}

/// 장식 타일의 칸(tiles.png). render.rs의 decor_src와 같은 표.
pub fn decor_cell(d: Decor) -> (u32, u32) {
    match d {
        Decor::FlowerWhite => (4, 0),
        Decor::FlowerYellow => (5, 0),
        Decor::FlowerPink => (6, 0),
        Decor::Tuft => (7, 0),
        Decor::Pebble => (3, 1),
        Decor::Mushroom => (4, 1),
        Decor::RootVine => (5, 1),
        Decor::Moss => (6, 1),
    }
}

/// 던전 흙길: 문 칸 가운데 → 방 가운데. (시작, 끝) 월드 px.
pub fn dungeon_paths(d: &crate::dungeon::Dungeon) -> Vec<(Vec2, Vec2)> {
    let mut out = Vec::new();
    for r in &d.rooms {
        let rect = r.rect();
        let c = rect.center() + vec2(0.0, 8.0);
        let doors = [
            vec2(rect.x + 10.0 * 16.0, rect.y + 2.0 * 16.0),
            vec2(rect.right() - 16.0, rect.y + 7.0 * 16.0),
            vec2(rect.x + 10.0 * 16.0, rect.bottom() - 16.0),
            vec2(rect.x + 16.0, rect.y + 7.0 * 16.0),
        ];
        for (i, linked) in r.links.iter().enumerate() {
            if *linked {
                out.push((doors[i], c));
            }
        }
    }
    out
}

/// 1px마다 벽인가(그림의 모양). 칸 가운데 값을 겹선형으로 잇고 잡음을 더해 0.5에서 자른다.
/// 잡음 폭(±0.23)이 0.5보다 작아 칸 가운데는 늘 제 칸 쪽이다.
#[cfg(test)]
fn shape(g: &Grid) -> Vec<bool> {
    let w = g.w as i32 * T;
    let uniform = shape_uniform(g);
    let mut solid = vec![false; (w * g.h as i32 * T) as usize];
    for y in 0..g.h as i32 * T {
        shape_row(g, &uniform, y, &mut solid);
    }
    solid
}

/// 칸마다: 둘레 3×3 칸이 다 같으면(경계가 없으면) 그 값, 아니면 None(1px마다 잰다).
fn shape_uniform(g: &Grid) -> Vec<Option<bool>> {
    (0..g.h as i32)
        .flat_map(|cy| (0..g.w as i32).map(move |cx| (cx, cy)))
        .map(|(cx, cy)| {
            let me = solid_cell(g, cx, cy);
            let same = (-1..=1).all(|dy| (-1..=1).all(|dx| solid_cell(g, cx + dx, cy + dy) == me));
            same.then_some(me)
        })
        .collect()
}

/// 모양 한 줄(y).
fn shape_row(g: &Grid, uniform: &[Option<bool>], y: i32, solid: &mut [bool]) {
    let w = g.w as i32 * T;
    for x in 0..w {
        let (cx, cy) = (x / T, y / T);
        let i = (y * w + x) as usize;
        if let Some(me) = uniform[cy as usize * g.w + cx as usize] {
            solid[i] = me;
            continue;
        }
        let u = (x as f32 + 0.5) / T as f32 - 0.5;
        let v = (y as f32 + 0.5) / T as f32 - 0.5;
        let (ci, cj) = (u.floor() as i32, v.floor() as i32);
        let (fu, fv) = (u - ci as f32, v - cj as f32);
        let s = |cx: i32, cy: i32| f32::from(u8::from(solid_cell(g, cx, cy)));
        let top = s(ci, cj) + (s(ci + 1, cj) - s(ci, cj)) * fu;
        let bot = s(ci, cj + 1) + (s(ci + 1, cj + 1) - s(ci, cj + 1)) * fu;
        let f = top + (bot - top) * fv;
        let n = (vnoise(x as f32, y as f32, 9.0, 1) - 0.5) * 0.34
            + (vnoise(x as f32, y as f32, 3.5, 2) - 0.5) * 0.12;
        solid[i] = f + n > 0.5;
    }
}

/// 굽기. `tiles`가 있으면 장식을 찍는다(시험에서는 없이). 한 번에 끝까지 굽는다(시험용, 게임은 `Baker`로 나눠 굽는다).
#[cfg(test)]
pub fn bake(
    g: &Grid,
    b: Biome,
    paths: &[(Vec2, Vec2)],
    tiles: Option<&Image>,
    sunny: Option<&Image>,
) -> Image {
    let mut k = Baker::new(g, b, paths, &[]);
    k.step(tiles, sunny, &mut || false);
    k.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Shape,
    Depth,
    Noise,
    Color,
    Decor,
    Done,
}

/// 나눠 굽기(0068): 단계마다 줄 단위로 나아가고, `out()`이 참이면 멈췄다가 다음 `step`에서 이어 간다.
/// 한 프레임에 다 구우면 브라우저에서 0.15초 멈춘다. 프레임마다 몇 ms씩 굽고 그동안 화면은 어둡게 둔다.
pub struct Baker {
    g: Grid,
    b: Biome,
    paths: Vec<(Vec2, Vec2)>,
    /// 모닥불 자리(0103): 곁의 장식 그림자는 불에서 바깥쪽으로.
    fires: Vec<Vec2>,
    w: i32,
    h: i32,
    pal: Palette,
    stage: Stage,
    row: i32,
    uniform: Vec<Option<bool>>,
    solid: Vec<bool>,
    depth: Vec<u16>,
    path: Vec<bool>,
    near: Vec<bool>,
    far: Vec<[u8; 4]>,
    band_n: Vec<f32>,
    patch_n: Vec<f32>,
    img: Image,
}

fn tint_px(c: Rgb, t: Color) -> [u8; 4] {
    [
        (c.0 as f32 * t.r) as u8,
        (c.1 as f32 * t.g) as u8,
        (c.2 as f32 * t.b) as u8,
        255,
    ]
}

impl Baker {
    pub fn new(g: &Grid, b: Biome, paths: &[(Vec2, Vec2)], fires: &[Vec2]) -> Self {
        let (w, h) = (g.w as i32 * T, g.h as i32 * T);
        let (wu, hu) = (w as usize, h as usize);
        let pal = palette(b);
        // 둘레 두 칸 안에 바닥이 없는 벽 칸은 화면에 나오지 않는다(카메라는 방 안): 한 색으로 빨리 채운다.
        let near: Vec<bool> = (0..g.h as i32)
            .flat_map(|cy| (0..g.w as i32).map(move |cx| (cx, cy)))
            .map(|(cx, cy)| (-2..=2).any(|dy| (-2..=2).any(|dx| !solid_cell(g, cx + dx, cy + dy))))
            .collect();
        let far = far_pattern(&pal)
            .into_iter()
            .map(|c| tint_px(c, b.wall_tint()))
            .collect();
        let (hw, hh) = (wu.div_ceil(2), hu.div_ceil(2));
        Self {
            g: g.clone(),
            b,
            paths: paths.to_vec(),
            fires: fires.to_vec(),
            w,
            h,
            pal,
            stage: Stage::Shape,
            row: 0,
            uniform: shape_uniform(g),
            solid: vec![false; wu * hu],
            depth: vec![u16::MAX; wu * hu],
            path: vec![false; wu * hu],
            near,
            far,
            band_n: vec![0.0; hw * hh],
            patch_n: vec![0.0; hw * hh],
            img: Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 1.0)),
        }
    }

    pub fn finish(self) -> Image {
        self.img
    }

    /// 조금 굽는다. 다 구웠으면 true. `out()`은 한 줄마다 묻는다(참이면 이번엔 그만).
    pub fn step(
        &mut self,
        tiles: Option<&Image>,
        sunny: Option<&Image>,
        out: &mut dyn FnMut() -> bool,
    ) -> bool {
        loop {
            let (stage, rows) = match self.stage {
                Stage::Shape => (Stage::Depth, self.h),
                Stage::Depth => (Stage::Noise, 1),
                Stage::Noise => (Stage::Color, (self.h as usize).div_ceil(2) as i32),
                Stage::Color => (Stage::Decor, self.h),
                Stage::Decor if tiles.is_some() => (Stage::Done, self.g.h as i32),
                Stage::Decor => (Stage::Done, 0),
                Stage::Done => return true,
            };
            if self.row >= rows {
                self.stage = stage;
                self.row = 0;
                continue;
            }
            let y = self.row;
            match self.stage {
                Stage::Shape => shape_row(&self.g, &self.uniform, y, &mut self.solid),
                Stage::Depth => self.depth_and_paths(),
                Stage::Noise => self.noise_row(y as usize),
                Stage::Color => self.color_row(y),
                Stage::Decor => self.decor_row(y, tiles.unwrap(), sunny),
                Stage::Done => {}
            }
            self.row += 1;
            if out() {
                return false;
            }
        }
    }

    /// 2. 벼랑 앞면(아래가 바닥인 벽의 아래쪽 FACE px, depth = 바닥 위로 몇 번째 줄) + 3. 흙길.
    fn depth_and_paths(&mut self) {
        let (w, h) = (self.w, self.h);
        let idx = |x: i32, y: i32| y as usize * w as usize + x as usize;
        for x in 0..w {
            let mut run: u16 = 0;
            let mut floor_below = false;
            for y in (0..h).rev() {
                if !self.solid[idx(x, y)] {
                    run = 0;
                    floor_below = true;
                    continue;
                }
                if floor_below && run < FACE {
                    self.depth[idx(x, y)] = run;
                }
                run = run.saturating_add(1);
            }
        }
        for (a, c) in &self.paths {
            let len = (*c - *a).length().max(1.0);
            let steps = len as i32;
            for k in 0..=steps {
                let t = k as f32 / steps as f32;
                let p = *a + (*c - *a) * t;
                let r = 4.0 + 3.0 * vnoise(p.x, p.y, 14.0, 21);
                let wob = (vnoise(p.x, p.y, 30.0, 22) - 0.5) * 10.0;
                let q = p + vec2(wob, wob * 0.5);
                let ri = r.ceil() as i32;
                for dy in -ri..=ri {
                    for dx in -ri..=ri {
                        if (dx * dx + dy * dy) as f32 <= r * r {
                            let (x, y) = (q.x as i32 + dx, q.y as i32 + dy);
                            if x >= 0 && y >= 0 && x < w && y < h {
                                self.path[idx(x, y)] = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// 바닥 큰 잡음(띠·얼룩)은 2×2 px마다 한 번만(도트 덩어리, 굽기 시간 ¼). 보이는 칸만.
    fn noise_row(&mut self, by: usize) {
        let hw = (self.w as usize).div_ceil(2);
        for bx in 0..hw {
            let (x, y) = (bx as i32 * 2, by as i32 * 2);
            if !self.near[(y / T) as usize * self.g.w + (x / T) as usize] {
                continue;
            }
            let (xf, yf) = (x as f32, y as f32);
            self.band_n[by * hw + bx] =
                vnoise(xf, yf, 56.0, 3) * 0.7 + vnoise(xf, yf, 17.0, 4) * 0.3;
            self.patch_n[by * hw + bx] =
                vnoise(xf, yf, 36.0, 6) * 0.75 + vnoise(xf, yf, 9.0, 7) * 0.25;
        }
    }

    /// 4. 색 한 줄.
    fn color_row(&mut self, y: i32) {
        let Baker {
            g,
            b,
            w,
            h,
            pal,
            solid,
            depth,
            path,
            near,
            far,
            band_n,
            patch_n,
            img,
            ..
        } = self;
        let (w, h) = (*w, *h);
        let wu = w as usize;
        let hw = wu.div_ceil(2);
        let idx = |x: i32, y: i32| y as usize * wu + x as usize;
        let is_solid = |x: i32, y: i32| {
            if x < 0 || y < 0 || x >= w || y >= h {
                true
            } else {
                solid[idx(x, y)]
            }
        };
        let is_face =
            |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && depth[idx(x, y)] != u16::MAX;
        let (ft, wt) = (b.floor_tint(), b.wall_tint());
        let checker = |x: i32, y: i32| (x + y) & 1 == 0;
        for x in 0..w {
            if !near[(y / T) as usize * g.w + (x / T) as usize] {
                let i = idx(x, y) * 4;
                let px = far[(y.rem_euclid(PAT) * PAT + x.rem_euclid(PAT)) as usize];
                img.bytes[i..i + 4].copy_from_slice(&px);
                continue;
            }
            let c: Rgb = if is_solid(x, y) {
                if is_face(x, y) {
                    face_px(pal, x, y, depth[idx(x, y)], &is_solid, &is_face)
                } else {
                    top_px(pal, x, y, &is_solid, &is_face)
                }
            } else {
                // 바닥: 큰 띠(밝음·어두움, 경계는 바둑판으로 섞음).
                let hb = (y as usize / 2) * hw + x as usize / 2;
                let t = band_n[hb];
                let band = |th: f32| t < th - 0.015 || (t < th + 0.015 && checker(x, y));
                let mut c = if band(0.36) {
                    pal.floor[2]
                } else if !band(0.66) {
                    pal.floor[1]
                } else {
                    pal.floor[0]
                };
                let r = rnd(x, y, 5);
                if r < 0.055 {
                    c = pal.floor[2];
                } else if r < 0.075 {
                    c = pal.floor[3];
                } else if r > 0.94 && !is_solid(x, y - 1) && rnd(x, y + 1, 5) < 0.055 {
                    c = pal.floor[1];
                }
                // 반대 재질 얼룩.
                let p = patch_n[hb];
                let edge = (p - pal.patch_at).abs() < 0.012;
                if p > pal.patch_at && !(edge && checker(x, y)) {
                    let r = rnd(x, y, 8);
                    c = if r < 0.18 {
                        pal.soil[1]
                    } else if r < 0.26 {
                        pal.soil[2]
                    } else {
                        pal.soil[0]
                    };
                    if !pal.patch_is_soil && r > 0.97 {
                        c = pal.floor[3];
                    }
                }
                if path[idx(x, y)] {
                    c = if rnd(x, y, 9) < 0.2 {
                        pal.path[1]
                    } else {
                        pal.path[0]
                    };
                    // 길 가장자리 조약돌.
                    let edge = !path[idx((x + 1).min(w - 1), y)] || !path[idx((x - 1).max(0), y)];
                    if edge && rnd(x, y, 10) < 0.2 {
                        c = (150, 146, 140);
                    }
                }
                // 그늘: 벼랑 앞면 아래로 5px, 옆벽 곁 2px.
                let mut k = 1.0;
                for (d, m) in [(1, 0.60), (2, 0.70), (3, 0.79), (4, 0.87), (5, 0.93)] {
                    if is_face(x, y - d) {
                        k = m;
                        break;
                    }
                }
                if k == 1.0 {
                    if is_solid(x - 1, y) || is_solid(x + 1, y) {
                        k = 0.82;
                    } else if is_solid(x - 2, y) || is_solid(x + 2, y) || is_solid(x, y + 1) {
                        k = 0.92;
                    }
                }
                (
                    (c.0 as f32 * k) as u8,
                    (c.1 as f32 * k) as u8,
                    (c.2 as f32 * k) as u8,
                )
            };
            let t = if is_solid(x, y) { wt } else { ft };
            let px = tint_px(c, t);
            let i = idx(x, y) * 4;
            img.bytes[i..i + 4].copy_from_slice(&px);
        }
    }

    /// 5. 장식 한 칸 줄(바닥 칸만, 벽 그림 위에는 찍지 않는다).
    fn decor_row(&mut self, cy: i32, tiles: &Image, sunny: Option<&Image>) {
        let Baker {
            g,
            b,
            w,
            h,
            solid,
            img,
            fires,
            ..
        } = self;
        let (w, h, b) = (*w, *h, *b);
        let bright = b.darkness().0 <= 0.0;
        // 장식 그림자(0102·0103): 발 자리에서 방향을 정하고, 바닥에 붙은 것일수록(꽃 < 조약돌 < 풀포기) 짧게.
        let shadow_for = |foot: Vec2, lift: f32| -> Option<crate::sunny::StampShadow> {
            let fire = fires
                .iter()
                .copied()
                .min_by(|a, c| (*a - foot).length().total_cmp(&(*c - foot).length()));
            let k = crate::shade::shadow_dir(foot, fire, bright)?;
            Some(crate::sunny::StampShadow {
                k: k * lift,
                alpha: if bright {
                    crate::tuning::SUN_SHADOW_ALPHA
                } else {
                    crate::tuning::FIRE_SHADOW_ALPHA
                },
                fade: fire
                    .filter(|_| !bright)
                    .map(|f| (f, crate::tuning::FIRE_SHADOW_REACH)),
            })
        };
        let idx = |x: i32, y: i32| y as usize * w as usize + x as usize;
        let is_solid = |x: i32, y: i32| {
            if x < 0 || y < 0 || x >= w || y >= h {
                true
            } else {
                solid[idx(x, y)]
            }
        };
        let ft = b.floor_tint();
        for cx in 0..g.w as i32 {
            if g.get_i(cx, cy) != Cell::Floor {
                continue;
            }
            let Some((d, ox, oy)) = decor_at(b, cx, cy) else {
                continue;
            };
            // 팩이 있으면 꽃·풀·조약돌은 Sunnyside 그림으로(0066). 칸 가운데 아래에 발을 둔다.
            // 빛나는 버섯은 굽지 않는다: 어둠 뒤에 팩의 파란 버섯(움직임)으로 그린다.
            if sunny.is_some() && d == Decor::Mushroom {
                continue;
            }
            if let Some(sheet) = sunny {
                let pick = hash(cx, cy, 51) as usize;
                let r = match d {
                    Decor::FlowerWhite => Some(crate::sunny::FLOWERS_WHITE[pick % 4]),
                    Decor::FlowerYellow => Some(crate::sunny::FLOWERS_BLUE[pick % 4]),
                    Decor::FlowerPink => Some(crate::sunny::FLOWERS_RED[pick % 4]),
                    Decor::Tuft => Some(crate::sunny::GRASS[pick % 8]),
                    Decor::Pebble => Some(crate::sunny::PEBBLES[pick % 3]),
                    _ => None,
                };
                if let Some(r) = r {
                    let fx = cx * T + 8 + ox as i32 - r.2 as i32 / 2;
                    let fy = cy * T + 12 + oy as i32 - r.3 as i32;
                    // 해·모닥불 그림자를 먼저(0102·0103). 바닥에 깔린 것은 높이 비율만큼 짧다.
                    let lift = match d {
                        Decor::Tuft => crate::tuning::DECOR_LIFT_TUFT,
                        Decor::Pebble => crate::tuning::DECOR_LIFT_PEBBLE,
                        _ => crate::tuning::DECOR_LIFT_FLOWER,
                    };
                    let foot = vec2((fx + r.2 as i32 / 2) as f32, (fy + r.3 as i32) as f32);
                    if let Some(sh) = shadow_for(foot, lift) {
                        crate::sunny::stamp_sun_shadow(img, sheet, r, (fx, fy), &sh, is_solid);
                    }
                    crate::sunny::stamp(img, sheet, r, fx, fy, ft, is_solid);
                    // 꽃은 무리로: 둘레에 한두 송이 더.
                    if matches!(
                        d,
                        Decor::FlowerWhite | Decor::FlowerYellow | Decor::FlowerPink
                    ) {
                        for k in 0..(pick >> 3) % 3 {
                            let set = match d {
                                Decor::FlowerWhite => crate::sunny::FLOWERS_WHITE,
                                Decor::FlowerYellow => crate::sunny::FLOWERS_BLUE,
                                _ => crate::sunny::FLOWERS_RED,
                            };
                            let q = hash(cx, cy, 52 + k as u32) as i32;
                            let rr = set[(q as usize >> 4) % 2];
                            let (dx, dy) = ((q % 11) - 5, ((q >> 8) % 7) - 3);
                            let (qx, qy) = (fx + dx + 3, fy + dy + 2);
                            let foot =
                                vec2((qx + rr.2 as i32 / 2) as f32, (qy + rr.3 as i32) as f32);
                            if let Some(sh) = shadow_for(foot, crate::tuning::DECOR_LIFT_FLOWER) {
                                crate::sunny::stamp_sun_shadow(
                                    img,
                                    sheet,
                                    rr,
                                    (qx, qy),
                                    &sh,
                                    is_solid,
                                );
                            }
                            crate::sunny::stamp(
                                img,
                                sheet,
                                rr,
                                fx + dx + 3,
                                fy + dy + 2,
                                ft,
                                is_solid,
                            );
                        }
                    }
                    continue;
                }
            }
            let (col, row) = decor_cell(d);
            let (bx, by) = (cx * T + ox as i32, cy * T + oy as i32);
            for sy in 0..T {
                for sx in 0..T {
                    let s = tiles.get_pixel(col * T as u32 + sx as u32, row * T as u32 + sy as u32);
                    if s.a < 0.5 {
                        continue;
                    }
                    let (x, y) = (bx + sx, by + sy);
                    if x < 0 || y < 0 || x >= w || y >= h || is_solid(x, y) {
                        continue;
                    }
                    let px = [
                        (s.r * ft.r * 255.0) as u8,
                        (s.g * ft.g * 255.0) as u8,
                        (s.b * ft.b * 255.0) as u8,
                        255,
                    ];
                    let i = idx(x, y) * 4;
                    img.bytes[i..i + 4].copy_from_slice(&px);
                }
            }
        }
    }
}

/// 벼랑 앞면 한 px: 흙 지층·박힌 돌·늘어진 뿌리, 맨 아래 그늘, 옆 가장자리 테두리.
fn face_px(
    p: &Palette,
    x: i32,
    y: i32,
    depth: u16,
    is_solid: &impl Fn(i32, i32) -> bool,
    is_face: &impl Fn(i32, i32) -> bool,
) -> Rgb {
    // 옆이 바닥이면 테두리.
    if !is_solid(x - 1, y) || !is_solid(x + 1, y) {
        return OUT;
    }
    if depth == 0 {
        return p.face[3];
    }
    // 윗면에서 내려온 잎·풀(1~3px, 들쭉날쭉).
    let from_top = FACE - 1 - depth;
    let above_top = is_solid(x, y - 1 - from_top as i32) && !is_face(x, y - 1 - from_top as i32);
    if above_top {
        let drip = 1 + (hash(x, 0, 31) % 3) as u16;
        if from_top < drip {
            return if from_top + 1 == drip {
                p.top[2]
            } else {
                p.top[0]
            };
        }
        if from_top == drip {
            return p.face[3];
        }
    }
    // 늘어진 뿌리.
    let col = x.div_euclid(7);
    if rnd(col, 0, 33) < p.roots_on_face {
        let x0 = col as f32 * 7.0 + 3.0 + ((y as f32) * 0.45 + col as f32).sin() * 1.2;
        let dx = x as f32 - x0;
        let len = 6 + (hash(col, 0, 34) % 10) as u16;
        if from_top < len && (0.0..1.0).contains(&dx) {
            return ROOT[0];
        }
        if from_top < len && (1.0..2.0).contains(&dx) {
            return ROOT[1];
        }
    }
    if depth == 1 {
        return p.face[1];
    }
    // 지층(물결치는 가로 줄)과 박힌 돌.
    let band = (y as f32 + (vnoise(x as f32, 0.0, 6.0, 35) - 0.5) * 4.0) / 5.0;
    if band.fract() < 0.2 {
        return p.face[1];
    }
    let (bx, by) = (x.div_euclid(3), y.div_euclid(2));
    if rnd(bx, by, 36) < 0.07 {
        return if y.rem_euclid(2) == 0 {
            p.face[2]
        } else {
            p.face[3]
        };
    }
    p.face[0]
}

/// 벽 윗면 한 px: 덤불(빛·그늘이 있는 잎 뭉치) 또는 돌, 뿌리 줄, 바닥과 닿는 테두리.
fn top_px(
    p: &Palette,
    x: i32,
    y: i32,
    is_solid: &impl Fn(i32, i32) -> bool,
    is_face: &impl Fn(i32, i32) -> bool,
) -> Rgb {
    let open = |x: i32, y: i32| !is_solid(x, y);
    if open(x - 1, y) || open(x + 1, y) || open(x, y - 1) || (open(x, y + 1) && !is_face(x, y + 1))
    {
        return OUT;
    }
    // 위가 열린 가장자리 바로 안쪽: 밝은 줄(빛 받는 쪽).
    if open(x, y - 2) || open(x - 2, y) {
        return p.top[3];
    }
    let (xf, yf) = (x as f32, y as f32);
    if p.roots_on_top {
        let ridge = (vnoise(xf, yf, 13.0, 41) - 0.5).abs();
        if ridge < 0.022 {
            return ROOT[0];
        }
        if ridge < 0.034 {
            return ROOT[1];
        }
    }
    if p.stone_top {
        // 돌: 큰 덩어리 + 금.
        let q = vnoise(xf, yf, 7.0, 42);
        let crack = (vnoise(xf, yf, 11.0, 43) - 0.5).abs() < 0.02;
        if crack {
            return p.top[2];
        }
        if rnd(x, y, 44) < 0.03 {
            return (88, 128, 90);
        }
        return if q > 0.6 { p.top[1] } else { p.top[0] };
    }
    // 덤불: 잎 뭉치의 밝기 차(왼쪽 위가 빛).
    let q = vnoise(xf, yf, 4.0, 45);
    let shade = q - vnoise(xf + 1.6, yf + 1.6, 4.0, 45);
    if shade > 0.09 {
        p.top[3]
    } else if shade < -0.09 {
        p.top[2]
    } else if q > 0.55 {
        p.top[1]
    } else {
        p.top[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room() -> Grid {
        let mut g = Grid::filled(20, 12, Cell::Wall);
        for y in 2..11 {
            for x in 1..19 {
                g.set(x, y, Cell::Floor);
            }
        }
        g
    }

    /// 칸 가운데 ±4px는 늘 제 칸 쪽이다(벽 그림이 걷는 곳을 가리지 않고, 벽은 벽으로 보인다).
    #[test]
    fn cell_centers_keep_their_side() {
        let g = room();
        let solid = shape(&g);
        let w = g.w * 16;
        for cy in 0..g.h {
            for cx in 0..g.w {
                let wall = g.get(cx, cy) == Cell::Wall;
                for (dx, dy) in [(0, 0), (-4, 0), (4, 0), (0, -4), (0, 4)] {
                    let (x, y) = ((cx * 16 + 8) as i32 + dx, (cy * 16 + 8) as i32 + dy);
                    assert_eq!(solid[y as usize * w + x as usize], wall, "칸 ({cx}, {cy})");
                }
            }
        }
        for b in [Biome::Meadow, Biome::Roots, Biome::Deep, Biome::Silk] {
            let img = bake(
                &g,
                b,
                &[(vec2(160.0, 40.0), vec2(160.0, 100.0))],
                None,
                None,
            );
            assert_eq!(
                (img.width as usize, img.height as usize),
                (g.w * 16, g.h * 16)
            );
        }
    }

    /// 나눠 구워도(한 줄마다 멈춰도) 한 번에 구운 것과 같은 그림이다(0068).
    #[test]
    fn baking_in_steps_matches_one_go() {
        let g = room();
        let tiles = Image::gen_image_color(512, 512, Color::new(0.5, 0.6, 0.4, 1.0));
        let paths = [(vec2(40.0, 60.0), vec2(250.0, 150.0))];
        let whole = bake(&g, Biome::Meadow, &paths, Some(&tiles), None);
        let mut k = Baker::new(&g, Biome::Meadow, &paths, &[]);
        let mut steps = 0;
        while !k.step(Some(&tiles), None, &mut || true) {
            steps += 1;
        }
        assert!(steps > 100, "멈춘 횟수 {steps}");
        assert_eq!(k.finish().bytes, whole.bytes);
    }

    /// 던전 크기(100×60칸) 굽기 시간. `cargo test --release bake_speed -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn bake_speed() {
        let mut g = Grid::filled(100, 60, Cell::Wall);
        for (ox, oy) in [(0, 0), (20, 0), (40, 12), (20, 12), (60, 24), (40, 24)] {
            for y in 2..11 {
                for x in 1..19 {
                    g.set(ox + x, oy + y, Cell::Floor);
                }
            }
        }
        let t = std::time::Instant::now();
        let _ = bake(&g, Biome::Roots, &[], None, None);
        println!("bake 100×60: {:?}", t.elapsed());
    }

    /// 같은 지도면 같은 열쇠, 벽 하나가 바뀌면 다른 열쇠. 출구가 생겨도(바닥 → 출구) 열쇠는 같다.
    #[test]
    fn key_ignores_dynamic_cells() {
        let g = room();
        let k = key(&g, Biome::Meadow, &[]);
        let mut e = g.clone();
        e.set(5, 5, Cell::Exit);
        assert_eq!(key(&e, Biome::Meadow, &[]), k);
        let mut wl = g.clone();
        wl.set(5, 5, Cell::Wall);
        assert_ne!(key(&wl, Biome::Meadow, &[]), k);
        assert_ne!(key(&g, Biome::Deep, &[]), k);
    }
}
