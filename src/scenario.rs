//! 개발용 시나리오: 원하는 방 구성으로 바로 시작한다. `dev` feature(기본 켬)에서만 타이틀에 보인다.
//! 새 시나리오는 여기 목록에 한 줄 추가. 수치 조정(손맛)은 tuning.rs, 적 구성은 여기.

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::enemy::Kind;
use crate::loot::{Loot, LootKind};
use crate::world::World;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scenario {
    /// 타이틀 "시작": 필드(모험)에서 오프닝 컷씬 → 튜토리얼 → 자유 탐험(docs/decisions/0019).
    Normal,
    /// 필드에서 바로(컷씬·튜토리얼 없이, 발톱 들고).
    Field,
    /// 필드에서 곧장 던전으로.
    Dungeon,
    /// 아레나 웨이브(예전 첫 시작). 손맛 시험장.
    Arena,
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
    /// 발톱 4종을 가방에 넣고 웨이브 1부터. 가방에서 바꿔 끼며 비교.
    AllClaws,
    /// 환경 확인: 지정 환경에서 돌격형 1 + 원거리형 1.
    InBiome(Biome),
    /// 반딧불·산딸기 줍기 확인(docs/decisions/0016).
    Loot,
    /// 장신구·퀵 아이템·필살기 확인(docs/decisions/0017, 0018). 장신구는 앱이 가방에 넣는다.
    Gear,
}

pub struct Entry {
    pub name: &'static str,
    pub desc: &'static str,
    pub scenario: Scenario,
}

pub const LIST: [Entry; 17] = [
    Entry {
        name: "필드(모험)",
        desc: "컷씬·튜토리얼 없이 필드에서. 남쪽 굴 입구로 던전, 저장 지점(초록 버섯 고리).",
        scenario: Scenario::Field,
    },
    Entry {
        name: "던전 바로",
        desc: "새로 생성한 던전 시작방에서. 전투방은 들어서면 문이 닫힌다. 보스를 잡으면 출구.",
        scenario: Scenario::Dungeon,
    },
    Entry {
        name: "아레나 웨이브",
        desc: "예전 첫 시작: 방 하나에서 웨이브 1부터(손맛 시험장).",
        scenario: Scenario::Arena,
    },
    Entry {
        name: "환경: 꽃 핀 입구",
        desc: "밝고 꽃이 많은 첫 환경. 돌격형 1 + 원거리형 1.",
        scenario: Scenario::InBiome(Biome::Meadow),
    },
    Entry {
        name: "환경: 뿌리 지대",
        desc: "꽃이 줄고 뿌리가 드러난다. 가장자리가 어두워진다.",
        scenario: Scenario::InBiome(Biome::Roots),
    },
    Entry {
        name: "환경: 어두운 심층",
        desc: "빛은 Naïl 주변뿐. 이끼와 빛나는 버섯.",
        scenario: Scenario::InBiome(Biome::Deep),
    },
    Entry {
        name: "튜토리얼",
        desc: "컷씬 없이 플레이 튜토리얼만(이동→3연타→굴파기→솟구침).",
        scenario: Scenario::Tutorial,
    },
    Entry {
        name: "발톱 4종 비교",
        desc: "새싹·두더지·바위·독 발톱이 가방에. 메뉴 → 가방에서 확인으로 바꿔 낀다.",
        scenario: Scenario::AllClaws,
    },
    Entry {
        name: "떨어지는 것 확인",
        desc: "체력 2/5. 반딧불 12마리와 산딸기 2개가 튀어나온다. 다가가 끌려오는 반딧불, 회복을 본다.",
        scenario: Scenario::Loot,
    },
    Entry {
        name: "장신구·필살기 확인",
        desc: "장신구 5개가 가방에, 산딸기 3·지렁이 2, SP 가득, 생명의 씨앗 하나. 돌격형 2마리. L·B로 드릴, F·Y로 먹기, Q·L1로 넘기기.",
        scenario: Scenario::Gear,
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
        Scenario::Normal | Scenario::Field => World::field(),
        Scenario::Dungeon => {
            let mut w = World::field();
            w.enter_dungeon_now();
            w
        }
        Scenario::Arena => World::new(),
        Scenario::Empty | Scenario::CutsceneOnly | Scenario::Tutorial => World::empty(),
        Scenario::AllClaws => World::new(),
        Scenario::Gear => {
            let mut w = build(Scenario::Chargers(2));
            w.bag.berries = 3;
            w.bag.worms = 2;
            w.player.sp = crate::tuning::SP_MAX;
            w.player.hp = 3;
            w.loot.push(Loot::place(
                LootKind::LifeSeed,
                w.room.center() + vec2(-40.0, 30.0),
            ));
            w
        }
        Scenario::Loot => {
            let mut w = World::empty();
            w.player.hp = 2;
            // 오른쪽 길목(기둥을 피해서). 오른쪽으로 걸어가면 반딧불이 끌려오고 산딸기를 밟는다.
            let c = w.room.center() + vec2(40.0, 0.0);
            for _ in 0..12 {
                w.loot
                    .push(Loot::spawn(LootKind::Glowbug, c + vec2(0.0, -30.0)));
            }
            for _ in 0..2 {
                w.loot
                    .push(Loot::place(LootKind::Berry, c + vec2(0.0, -2.0)));
            }
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
        Scenario::InBiome(b) => {
            let mut w = build(Scenario::OneEach);
            w.biome = b;
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
