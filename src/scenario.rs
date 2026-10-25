//! 개발용 시나리오: 원하는 방 구성으로 바로 시작한다. `dev` feature(기본 켬)에서만 타이틀에 보인다.
//! 새 시나리오는 여기 목록에 한 줄 추가. 수치 조정(손맛)은 tuning.rs, 적 구성은 여기.

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::world::World;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scenario {
    /// 타이틀 "시작": 오프닝 컷씬 뒤 웨이브 1부터.
    Normal,
    Empty,
    Chargers(u32),
    Shooters(u32),
    /// 지정 웨이브부터 자동 웨이브.
    FromWave(u32),
    /// 돌격형 1 + 원거리형 1, 자동 웨이브 없음.
    OneEach,
    /// 오프닝 컷씬만 보고 빈 방.
    CutsceneOnly,
    /// 컷씬 없이 튜토리얼만.
    Tutorial,
    /// 실험용 무거운 무기(2타 콤보)로 웨이브 1부터.
    Heavy,
}

pub struct Entry {
    pub name: &'static str,
    pub desc: &'static str,
    pub scenario: Scenario,
}

pub const LIST: [Entry; 9] = [
    Entry {
        name: "튜토리얼",
        desc: "컷씬 없이 플레이 튜토리얼만(이동→3연타→굴파기→솟구침).",
        scenario: Scenario::Tutorial,
    },
    Entry {
        name: "무거운 무기(실험)",
        desc: "2타 콤보 무브셋으로 웨이브 1부터. 무기가 바뀌면 손맛이 얼마나 달라지나.",
        scenario: Scenario::Heavy,
    },
    Entry {
        name: "빈 방",
        desc: "적 없음. 이동·굴파기 손맛만 본다.",
        scenario: Scenario::Empty,
    },
    Entry {
        name: "돌격형 1마리",
        desc: "예고→돌진→멈춤 타이밍 확인.",
        scenario: Scenario::Chargers(1),
    },
    Entry {
        name: "원거리형 1마리",
        desc: "거리 유지·투사체. 솟구침으로 투사체 지우기.",
        scenario: Scenario::Shooters(1),
    },
    Entry {
        name: "돌격형 3마리",
        desc: "판정 2: 위치 선정을 생각하게 되는가.",
        scenario: Scenario::Chargers(3),
    },
    Entry {
        name: "1+1",
        desc: "돌격형 1 + 원거리형 1. 다음 웨이브 없음.",
        scenario: Scenario::OneEach,
    },
    Entry {
        name: "웨이브 3부터",
        desc: "2C+2S부터 자동 웨이브.",
        scenario: Scenario::FromWave(3),
    },
    Entry {
        name: "오프닝 컷씬",
        desc: "컷씬 재생 확인. 끝나면 빈 방.",
        scenario: Scenario::CutsceneOnly,
    },
];

/// 시나리오에 맞는 월드를 만든다.
pub fn build(s: Scenario) -> World {
    match s {
        Scenario::Normal => World::new(),
        Scenario::Empty | Scenario::CutsceneOnly | Scenario::Tutorial => World::empty(),
        Scenario::Heavy => {
            let mut w = World::new();
            w.player.moveset = &crate::moveset::HEAVY;
            w
        }
        Scenario::Chargers(n) => {
            let mut w = World::empty();
            for i in 0..n {
                let p = spawn_point(&w, i as usize);
                w.spawn_enemy(Kind::Charger, p);
            }
            w
        }
        Scenario::Shooters(n) => {
            let mut w = World::empty();
            for i in 0..n {
                let p = spawn_point(&w, i as usize + 1);
                w.spawn_enemy(Kind::Shooter, p);
            }
            w
        }
        Scenario::OneEach => {
            let mut w = World::empty();
            let a = spawn_point(&w, 0);
            let b = spawn_point(&w, 3);
            w.spawn_enemy(Kind::Charger, a);
            w.spawn_enemy(Kind::Shooter, b);
            w
        }
        Scenario::FromWave(n) => {
            let mut w = World::empty();
            w.auto_waves = true;
            w.wave = n.saturating_sub(1);
            w.next_wave();
            w
        }
    }
}

/// 플레이어에서 먼 스폰 지점을 순서대로.
fn spawn_point(w: &World, i: usize) -> Vec2 {
    let far: Vec<Vec2> = w
        .room
        .spawn_points
        .iter()
        .copied()
        .filter(|p| (*p - w.player.pos).length() >= crate::tuning::SPAWN_MIN_DIST)
        .collect();
    far[i % far.len()]
}
