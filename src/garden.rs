//! 마을 텃밭(0304 G1, v1.1 "건네주는 빛"): 마을 씨앗을 쓰는 곳 하나 더(0303 A2). 꾸미기를 다 사고 나면 씨앗이 쌓이기만 했다.
//!
//! - 텃밭 칸 `PLOTS`(지도 글자 `f`)에 꽃(`Bloom` 일곱 가지)이나 산딸기를 심는다(마을 씨앗 `Crop::price`). 한 칸에 하나.
//! - 이야기 런을 새로 떠날 때마다 한 단계 자란다(`grow`, 앱이 `unpack` 곁에서 부른다) — `GROW`번 다녀오면 다 자란다.
//!   런이 끝나는 길은 여럿이지만 떠나는 자리는 하나라 거기서 센다.
//! - 다 자라면 마을 창 텃밭 탭에서 거둔다: 산딸기는 다음 런 가방에 하나(`KEY_BERRIES`, 짐 꾸리기 한도와 따로 — 꺼내는 것은 `unpack`),
//!   꽃은 마을 화단 자리(`FLOWER_SPOTS`)에 옮겨 심는다. 자리가 다 차면 **가장 오래된 자리부터 새 꽃으로 바뀐다**(0306) —
//!   꾸밀 때마다 씨앗을 내는, 끝이 없는 쓸 곳(0305 검토: 한 번 사면 끝인 쓸 곳으로는 후반에 남는 씨앗을 못 빨아들인다).
//! - 힘(영구 수치)은 주지 않는다. 기록 키는 새로(`plot.<칸>.crop`·`plot.<칸>.age`·`garden.*`, 0041).
//!   `garden.flowers`는 지금껏 옮겨 심은 수(다음 자리 = 수 % 자리 수), 자리마다 꽃은 `garden.spot.<자리>`(없으면 옛 기록의 들꽃).

use crate::i18n::Tr;
use crate::profile::Profile;
use crate::village::KEY_SEEDS;

/// 텃밭 칸 수(지도의 `f` 수와 같다 — 시험이 본다).
pub const PLOTS: usize = 3;
/// 다 자라기까지 다녀올 런 수.
pub const GROW: u32 = 2;
/// 거둔 산딸기(다음 런 가방에 넣을 것).
pub const KEY_BERRIES: &str = "garden.berries";
/// 지금껏 마을에 옮겨 심은 꽃 수(돌려 심기 차례).
pub const KEY_FLOWERS: &str = "garden.flowers";
/// 옮겨 심은 꽃이 놓이는 마을 화단 자리(열, 줄 — `village::ROWS`의 빈 바닥). 다 차면 오래된 자리부터 바뀐다.
pub const FLOWER_SPOTS: [(i32, i32); 10] = [
    (2, 5),
    (7, 8),
    (14, 10),
    (3, 11),
    (17, 6),
    (10, 12),
    (24, 9),
    (23, 11),
    (5, 6),
    (19, 12),
];

/// 화단 꽃(0306): 들꽃은 Sunnyside 작은 꽃 무더기(옛 꽃씨), 나머지 여섯은 CC0 Idylwild 그림(`assets/art/plants.png` 0줄).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bloom {
    Meadow,
    Blue,
    Violet,
    Pink,
    Lavender,
    Marigold,
    Yellow,
}

impl Bloom {
    pub const ALL: [Bloom; 7] = [
        Bloom::Meadow,
        Bloom::Blue,
        Bloom::Violet,
        Bloom::Pink,
        Bloom::Lavender,
        Bloom::Marigold,
        Bloom::Yellow,
    ];

    /// 식물 시트 0줄의 칸(들꽃은 팩 그림이라 없음).
    pub fn sheet(self) -> Option<u32> {
        Bloom::ALL
            .iter()
            .position(|b| *b == self)
            .and_then(|i| i.checked_sub(1))
            .map(|i| i as u32)
    }

    pub fn name(self) -> Tr {
        match self {
            Bloom::Meadow => |s| s.garden_flower,
            Bloom::Blue => |s| s.bloom_blue,
            Bloom::Violet => |s| s.bloom_violet,
            Bloom::Pink => |s| s.bloom_pink,
            Bloom::Lavender => |s| s.bloom_lavender,
            Bloom::Marigold => |s| s.bloom_marigold,
            Bloom::Yellow => |s| s.bloom_yellow,
        }
    }

    /// 기록에 적는 수(들꽃 1은 옛 `Crop::Flower` 그대로, 2는 산딸기라 건너뛴다).
    fn code(self) -> u32 {
        match self {
            Bloom::Meadow => 1,
            b => 2 + Bloom::ALL.iter().position(|x| *x == b).unwrap_or(0) as u32,
        }
    }

    fn of(code: u32) -> Option<Bloom> {
        match code {
            1 => Some(Bloom::Meadow),
            3..=8 => Some(Bloom::ALL[(code - 2) as usize]),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crop {
    Berry,
    Flower(Bloom),
}

impl Crop {
    /// 텃밭 탭의 좌우 차례: 산딸기 → 꽃 일곱.
    pub const ALL: [Crop; 8] = [
        Crop::Berry,
        Crop::Flower(Bloom::Meadow),
        Crop::Flower(Bloom::Blue),
        Crop::Flower(Bloom::Violet),
        Crop::Flower(Bloom::Pink),
        Crop::Flower(Bloom::Lavender),
        Crop::Flower(Bloom::Marigold),
        Crop::Flower(Bloom::Yellow),
    ];

    /// 심는 값(마을 씨앗). 산딸기는 짐 꾸리기 값(3)보다 조금 비싸다 — 한도 밖으로 하나 더 챙기는 값.
    /// 꽃은 마을에 남으니 더 비싸다(0305: 꽃씨 6 — 돌려 심기로 끝없이 쓰는 곳).
    pub fn price(self) -> u32 {
        match self {
            Crop::Flower(_) => 6,
            Crop::Berry => 4,
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Crop::Flower(b) => b.name(),
            Crop::Berry => |s| s.garden_berry,
        }
    }

    fn code(self) -> u32 {
        match self {
            Crop::Flower(b) => b.code(),
            Crop::Berry => 2,
        }
    }

    fn of(code: u32) -> Option<Crop> {
        match code {
            2 => Some(Crop::Berry),
            c => Bloom::of(c).map(Crop::Flower),
        }
    }
}

/// 텃밭 칸 하나의 모습.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Plot {
    #[default]
    Empty,
    /// 심었고 `age`번 다녀왔다(0..GROW).
    Growing {
        crop: Crop,
        age: u32,
    },
    Ripe(Crop),
}

fn key_crop(i: usize) -> String {
    format!("plot.{i}.crop")
}

fn key_age(i: usize) -> String {
    format!("plot.{i}.age")
}

fn key_spot(k: usize) -> String {
    format!("garden.spot.{k}")
}

pub fn plot(p: &Profile, i: usize) -> Plot {
    let Some(crop) = Crop::of(p.count(&key_crop(i))) else {
        return Plot::Empty;
    };
    let age = p.count(&key_age(i));
    if age >= GROW {
        Plot::Ripe(crop)
    } else {
        Plot::Growing { crop, age }
    }
}

pub fn plots(p: &Profile) -> [Plot; PLOTS] {
    std::array::from_fn(|i| plot(p, i))
}

/// 마을 화단 자리마다 핀 꽃. 자리 기록이 없는 자리는 옛 기록(돌려 심기 전 `garden.flowers` 수만큼)의 들꽃.
pub fn spots(p: &Profile) -> [Option<Bloom>; FLOWER_SPOTS.len()] {
    let planted = p.count(KEY_FLOWERS) as usize;
    std::array::from_fn(|k| {
        Bloom::of(p.count(&key_spot(k))).or((k < planted).then_some(Bloom::Meadow))
    })
}

/// 핀 화단 자리 수(마을 창에 보이는 수).
pub fn flowers(p: &Profile) -> u32 {
    spots(p).iter().filter(|b| b.is_some()).count() as u32
}

/// 다음에 옮겨 심을 자리와, 그 자리에 이미 꽃이 있어 바뀌는지.
pub fn next_spot(p: &Profile) -> (usize, bool) {
    let k = p.count(KEY_FLOWERS) as usize % FLOWER_SPOTS.len();
    (k, spots(p)[k].is_some())
}

/// 심기가 안 되는 까닭.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum No {
    /// 칸이 비어 있지 않다.
    Busy,
    /// 마을 씨앗이 모자라다.
    Poor,
}

/// 심는다: 마을 씨앗을 내고 칸에 적는다.
pub fn plant(p: &mut Profile, i: usize, crop: Crop) -> Result<(), No> {
    if plot(p, i) != Plot::Empty {
        return Err(No::Busy);
    }
    let have = p.count(KEY_SEEDS);
    if have < crop.price() {
        return Err(No::Poor);
    }
    p.set_count(KEY_SEEDS, have - crop.price());
    p.set_count(&key_crop(i), crop.code());
    p.set_count(&key_age(i), 0);
    Ok(())
}

/// 이야기 런을 새로 떠난다: 심어 둔 칸이 한 단계씩 자란다. 자란 칸이 있었나.
pub fn grow(p: &mut Profile) -> bool {
    let mut any = false;
    for i in 0..PLOTS {
        if let Plot::Growing { age, .. } = plot(p, i) {
            p.set_count(&key_age(i), age + 1);
            any = true;
        }
    }
    any
}

/// 다 자란 칸을 거둔다: 산딸기는 다음 런 가방에, 꽃은 마을 화단 다음 자리에(다 찼으면 가장 오래된 자리를 바꾼다).
/// 칸은 다시 빈다. 거둔 것.
pub fn harvest(p: &mut Profile, i: usize) -> Option<Crop> {
    let Plot::Ripe(crop) = plot(p, i) else {
        return None;
    };
    match crop {
        Crop::Berry => p.add(KEY_BERRIES, 1),
        Crop::Flower(b) => {
            // 옛 기록의 들꽃을 자리 기록으로 옮겨 둔다(차례가 돌아 자리 수를 넘으면 "옛 수만큼"이 틀어진다).
            let now = spots(p);
            for (k, s) in now.iter().enumerate() {
                if let Some(old) = s {
                    p.set_count(&key_spot(k), old.code());
                }
            }
            let (k, _) = next_spot(p);
            p.set_count(&key_spot(k), b.code());
            p.add(KEY_FLOWERS, 1);
        }
    }
    p.set_count(&key_crop(i), 0);
    p.set_count(&key_age(i), 0);
    Some(crop)
}

/// 새 런 가방에 넣을 거둔 산딸기: 가방 빈자리 `room`만큼 꺼내고, 넘치는 것은 다음 런에 넣게 남긴다(조용히 버리지 않는다).
pub fn take_berries(p: &mut Profile, room: u32) -> u32 {
    let n = p.count(KEY_BERRIES);
    let take = n.min(room);
    if take > 0 {
        p.set_count(KEY_BERRIES, n - take);
    }
    take
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds(n: u32) -> Profile {
        let mut p = Profile::default();
        p.set_count(KEY_SEEDS, n);
        p
    }

    /// 심으면 씨앗이 빠지고, 런 `GROW`번이면 다 자라고, 거두면 칸이 빈다.
    #[test]
    fn plots_grow_one_step_per_run() {
        let mut p = seeds(100);
        assert_eq!(plant(&mut p, 0, Crop::Berry), Ok(()));
        assert_eq!(p.count(KEY_SEEDS), 100 - Crop::Berry.price());
        assert_eq!(plant(&mut p, 0, Crop::Flower(Bloom::Blue)), Err(No::Busy));
        assert_eq!(harvest(&mut p, 0), None, "아직 자라는 중");
        for run in 0..GROW {
            assert_eq!(
                plot(&p, 0),
                Plot::Growing {
                    crop: Crop::Berry,
                    age: run
                }
            );
            assert!(grow(&mut p));
        }
        assert_eq!(plot(&p, 0), Plot::Ripe(Crop::Berry));
        grow(&mut p);
        assert_eq!(plot(&p, 0), Plot::Ripe(Crop::Berry), "다 자란 뒤엔 그대로");
        assert_eq!(harvest(&mut p, 0), Some(Crop::Berry));
        assert_eq!(plot(&p, 0), Plot::Empty);
        assert_eq!(take_berries(&mut p, 0), 0, "가방이 가득이면 남긴다");
        assert_eq!(take_berries(&mut p, 5), 1);
        assert_eq!(take_berries(&mut p, 5), 0, "한 번만 꺼낸다");
        assert!(!grow(&mut p), "빈 텃밭은 자라지 않는다");
    }

    #[test]
    fn planting_needs_seeds() {
        let rose = Crop::Flower(Bloom::Pink);
        let mut p = seeds(rose.price() - 1);
        assert_eq!(plant(&mut p, 1, rose), Err(No::Poor));
        assert_eq!(p.count(KEY_SEEDS), rose.price() - 1);
        assert_eq!(plot(&p, 1), Plot::Empty);
    }

    fn grow_flower(p: &mut Profile, b: Bloom) {
        assert_eq!(plant(p, 0, Crop::Flower(b)), Ok(()));
        for _ in 0..GROW {
            grow(p);
        }
        assert_eq!(harvest(p, 0), Some(Crop::Flower(b)));
    }

    /// 화단 자리를 차례로 채우고, 다 차면 가장 오래된 자리부터 새 꽃으로 바뀐다(끝없이 쓰는 곳, 0306).
    #[test]
    fn flowers_fill_spots_then_replace_the_oldest() {
        let mut p = seeds(10_000);
        let n = FLOWER_SPOTS.len();
        for k in 0..n {
            assert_eq!(next_spot(&p), (k, false));
            grow_flower(&mut p, Bloom::Blue);
        }
        assert_eq!(flowers(&p), n as u32);
        assert_eq!(next_spot(&p), (0, true), "다 차면 첫 자리가 바뀐다");
        grow_flower(&mut p, Bloom::Yellow);
        let s = spots(&p);
        assert_eq!(s[0], Some(Bloom::Yellow));
        assert_eq!(s[1], Some(Bloom::Blue));
        assert_eq!(flowers(&p), n as u32);
        assert_eq!(next_spot(&p), (1, true));
    }

    /// 돌려 심기 전 기록(꽃 수만)은 그 수만큼 들꽃으로 읽고, 다음 꽃은 그 뒤 자리에.
    #[test]
    fn old_flower_count_reads_as_meadow_flowers() {
        let mut p = seeds(100);
        p.set_count(KEY_FLOWERS, 3);
        let s = spots(&p);
        assert_eq!(
            s[..4],
            [
                Some(Bloom::Meadow),
                Some(Bloom::Meadow),
                Some(Bloom::Meadow),
                None
            ]
        );
        grow_flower(&mut p, Bloom::Violet);
        let s = spots(&p);
        assert_eq!(s[2], Some(Bloom::Meadow));
        assert_eq!(s[3], Some(Bloom::Violet));
    }

    /// 기록 수 ↔ 작물이 되돌아온다(들꽃 1·산딸기 2는 옛 기록과 같다).
    #[test]
    fn crop_codes_round_trip() {
        for c in Crop::ALL {
            assert_eq!(Crop::of(c.code()), Some(c));
        }
        assert_eq!(Crop::Flower(Bloom::Meadow).code(), 1);
        assert_eq!(Crop::Berry.code(), 2);
        let sheet: Vec<_> = Bloom::ALL.iter().map(|b| b.sheet()).collect();
        assert_eq!(sheet[0], None);
        assert_eq!(
            sheet[1..],
            [Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
    }

    /// 지도의 텃밭 칸 수 = `PLOTS`, 꽃 자리는 걸을 수 있는 빈 바닥이고 겹치지 않는다.
    #[test]
    fn garden_fits_the_village_map() {
        let rows = crate::village::ROWS;
        let plots = rows.iter().map(|r| r.matches('f').count()).sum::<usize>();
        assert_eq!(plots, PLOTS);
        let mut seen = std::collections::HashSet::new();
        for &(x, y) in &FLOWER_SPOTS {
            let c = rows[y as usize].as_bytes()[x as usize] as char;
            assert_eq!(c, '.', "꽃 자리 ({x},{y})는 빈 바닥");
            assert!(seen.insert((x, y)));
        }
    }
}
