//! 마을 텃밭(0304 G1, v1.1 "건네주는 빛"): 마을 씨앗을 쓰는 곳 하나 더(0303 A2). 꾸미기를 다 사고 나면 씨앗이 쌓이기만 했다.
//!
//! - 텃밭 칸 `PLOTS`(지도 글자 `f`)에 꽃씨나 산딸기 씨를 심는다(마을 씨앗 `Crop::price`). 한 칸에 하나.
//! - 이야기 런을 새로 떠날 때마다 한 단계 자란다(`grow`, 앱이 `unpack` 곁에서 부른다) — `GROW`번 다녀오면 다 자란다.
//!   런이 끝나는 길은 여럿이지만 떠나는 자리는 하나라 거기서 센다.
//! - 다 자라면 마을 창 텃밭 탭에서 거둔다: 산딸기는 다음 런 가방에 하나(`KEY_BERRIES`, 짐 꾸리기 한도와 따로 — 꺼내는 것은 `unpack`),
//!   꽃은 마을 곳곳에 옮겨 심는다(`KEY_FLOWERS`, 자리 `FLOWER_SPOTS` 수까지, 마을에 꾸미기로 남는다).
//! - 힘(영구 수치)은 주지 않는다. 기록 키는 새로(`plot.<칸>.crop`·`plot.<칸>.age`·`garden.*`, 0041).

use crate::i18n::Tr;
use crate::profile::Profile;
use crate::village::KEY_SEEDS;

/// 텃밭 칸 수(지도의 `f` 수와 같다 — 시험이 본다).
pub const PLOTS: usize = 3;
/// 다 자라기까지 다녀올 런 수.
pub const GROW: u32 = 2;
/// 거둔 산딸기(다음 런 가방에 넣을 것).
pub const KEY_BERRIES: &str = "garden.berries";
/// 마을에 옮겨 심은 꽃 수.
pub const KEY_FLOWERS: &str = "garden.flowers";
/// 옮겨 심은 꽃이 놓이는 마을 칸(열, 줄 — `village::ROWS`의 빈 바닥). 이 수까지 옮겨 심는다.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crop {
    Flower,
    Berry,
}

impl Crop {
    pub const ALL: [Crop; 2] = [Crop::Flower, Crop::Berry];

    /// 심는 값(마을 씨앗). 산딸기는 짐 꾸리기 값(3)보다 조금 비싸다 — 한도 밖으로 하나 더 챙기는 값.
    /// 꽃은 마을에 남으니 더 비싸다.
    pub fn price(self) -> u32 {
        match self {
            Crop::Flower => 6,
            Crop::Berry => 4,
        }
    }

    pub fn name(self) -> Tr {
        match self {
            Crop::Flower => |s| s.garden_flower,
            Crop::Berry => |s| s.garden_berry,
        }
    }

    fn code(self) -> u32 {
        match self {
            Crop::Flower => 1,
            Crop::Berry => 2,
        }
    }

    fn of(code: u32) -> Option<Crop> {
        match code {
            1 => Some(Crop::Flower),
            2 => Some(Crop::Berry),
            _ => None,
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

/// 옮겨 심은 꽃 수(마을에 그릴 수).
pub fn flowers(p: &Profile) -> u32 {
    p.count(KEY_FLOWERS).min(FLOWER_SPOTS.len() as u32)
}

/// 심기가 안 되는 까닭.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum No {
    /// 칸이 비어 있지 않다.
    Busy,
    /// 마을 씨앗이 모자라다.
    Poor,
    /// 꽃을 옮겨 심을 자리가 다 찼다(꽃만 — 자라는 꽃까지 센다).
    FlowersFull,
}

/// 심는다: 마을 씨앗을 내고 칸에 적는다.
pub fn plant(p: &mut Profile, i: usize, crop: Crop) -> Result<(), No> {
    if plot(p, i) != Plot::Empty {
        return Err(No::Busy);
    }
    if crop == Crop::Flower {
        let growing = (0..PLOTS)
            .filter(|&k| {
                matches!(
                    plot(p, k),
                    Plot::Growing {
                        crop: Crop::Flower,
                        ..
                    } | Plot::Ripe(Crop::Flower)
                )
            })
            .count() as u32;
        if flowers(p) + growing >= FLOWER_SPOTS.len() as u32 {
            return Err(No::FlowersFull);
        }
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

/// 다 자란 칸을 거둔다: 산딸기는 다음 런 가방에, 꽃은 마을에. 칸은 다시 빈다. 거둔 것.
pub fn harvest(p: &mut Profile, i: usize) -> Option<Crop> {
    let Plot::Ripe(crop) = plot(p, i) else {
        return None;
    };
    match crop {
        Crop::Berry => p.add(KEY_BERRIES, 1),
        Crop::Flower => p.add(KEY_FLOWERS, 1),
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
        assert_eq!(plant(&mut p, 0, Crop::Flower), Err(No::Busy));
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
        let mut p = seeds(Crop::Flower.price() - 1);
        assert_eq!(plant(&mut p, 1, Crop::Flower), Err(No::Poor));
        assert_eq!(p.count(KEY_SEEDS), Crop::Flower.price() - 1);
        assert_eq!(plot(&p, 1), Plot::Empty);
    }

    /// 꽃은 마을 자리 수까지 — 자라는 꽃까지 세어 넘치지 않는다.
    #[test]
    fn flowers_stop_at_the_village_spots() {
        let mut p = seeds(1000);
        p.set_count(KEY_FLOWERS, FLOWER_SPOTS.len() as u32 - 1);
        assert_eq!(plant(&mut p, 0, Crop::Flower), Ok(()));
        assert_eq!(plant(&mut p, 1, Crop::Flower), Err(No::FlowersFull));
        assert_eq!(plant(&mut p, 1, Crop::Berry), Ok(()), "산딸기는 된다");
        for _ in 0..GROW {
            grow(&mut p);
        }
        assert_eq!(harvest(&mut p, 0), Some(Crop::Flower));
        assert_eq!(flowers(&p), FLOWER_SPOTS.len() as u32);
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
