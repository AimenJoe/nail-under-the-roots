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
    /// 아직 게임에 자리가 없는 이야기 일러스트 장면(모모·엔딩, docs/decisions/0039).
    StoryPreview,
    /// 마지막 쉼터(길 지도의 보스 칸이 "가장 깊은 곳", docs/decisions/0042).
    LastCamp,
    /// 오늘의 굴: 날짜 시드로 첫 쉼터에서(타이틀에서 고른다, docs/decisions/0044).
    Daily,
    /// 큰 나무 아래 마을(모습은 기록에서, docs/decisions/0045).
    Village,
    /// 가장 깊은 곳에 바로 닿는다: 반쪽 엔딩 → 타이틀(기록은 남기지 않는다).
    Deepest,
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
    /// 적 다섯 종을 한 마리씩(생쥐는 떼). 도감·그림 확인(docs/decisions/0021).
    AllSpecies,
    /// 튜토리얼 보스: 필드 공터 앞에서, 문은 열려 있다(docs/decisions/0032).
    Boss,
    /// 첫 쉼터에서 씨앗 40개로(가게·길 지도, docs/decisions/0030, 0031).
    Camp,
    /// 첫 길 끝 굴(모모가 지킨다)에 바로. 숫자 = 이미 이긴 수(0 → 첫 만남, 2 → 세 번째, docs/decisions/0046).
    Momo(u8),
    /// 둘째 길 끝 굴(도토가 지킨다)에 바로(docs/decisions/0052).
    Doto,
}

pub struct Entry {
    pub name: &'static str,
    pub desc: &'static str,
    pub scenario: Scenario,
}

pub const LIST: [Entry; 27] = [
    Entry {
        name: "튜토리얼 보스",
        desc: "필드 남쪽 공터 앞. 들어서면 문이 닫히고 들쥐 대장이 깬다. 이기면 굴 입구 → 쉼터.",
        scenario: Scenario::Boss,
    },
    Entry {
        name: "쉼터",
        desc: "첫 쉼터, 씨앗 40. 포포 가게·보리·빈 자리, 아래 뿌리 길에서 길 지도.",
        scenario: Scenario::Camp,
    },
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
        desc: "장신구 5개가 가방에, 산딸기 3·지렁이 2, 분노 가득, 뿌리 심장 하나. 들쥐 2마리. L/B 드릴, I/Y 먹기, Q·E/LB·RB 넘기기.",
        scenario: Scenario::Gear,
    },
    Entry {
        name: "적 다섯 종",
        desc: "들쥐·생쥐 떼·시궁쥐·다람쥐·청설모. 도감 탭에 차례로 적힌다.",
        scenario: Scenario::AllSpecies,
    },
    Entry {
        name: "빈 방",
        desc: "적 없음. 이동·굴파기 손맛만 본다.",
        scenario: Scenario::Empty,
    },
    Entry {
        name: "들쥐 1마리",
        desc: "예고→돌진→멈춤 타이밍 확인.",
        scenario: Scenario::Chargers(1),
    },
    Entry {
        name: "다람쥐 1마리",
        desc: "거리 유지·투사체. 솟구침으로 투사체 지우기.",
        scenario: Scenario::Shooters(1),
    },
    Entry {
        name: "들쥐 3마리",
        desc: "판정 2: 위치 선정을 생각하게 되는가.",
        scenario: Scenario::Chargers(3),
    },
    Entry {
        name: "1+1",
        desc: "들쥐 1 + 다람쥐 1. 다음 웨이브 없음.",
        scenario: Scenario::OneEach,
    },
    Entry {
        name: "웨이브 3부터",
        desc: "4마리(종류는 무작위)부터 자동 웨이브.",
        scenario: Scenario::FromWave(3),
    },
    Entry {
        name: "오프닝 컷씬",
        desc: "컷씬 재생 확인. 끝나면 빈 방.",
        scenario: Scenario::CutsceneOnly,
    },
    Entry {
        name: "이야기 일러스트",
        desc: "모모가 돌아옴 → 반쪽 엔딩 → 참 엔딩·에필로그. 아직 게임에 자리가 없는 장면(임시 그림).",
        scenario: Scenario::StoryPreview,
    },
    Entry {
        name: "마지막 쉼터",
        desc: "마지막 길(길 지도의 끝이 가장 깊은 곳). 씨앗 40.",
        scenario: Scenario::LastCamp,
    },
    Entry {
        name: "가장 깊은 곳",
        desc: "런의 끝: 반쪽 엔딩을 보고 타이틀로.",
        scenario: Scenario::Deepest,
    },
    Entry {
        name: "마을",
        desc: "큰 나무 아래 마을(지금 기록의 모습).",
        scenario: Scenario::Village,
    },
    Entry {
        name: "모모 첫 만남",
        desc: "첫 길 끝 굴에서 시작. 끝방의 모모: 높이 뛰기·뒷발 차기. 이기면 달아나는 장면.",
        scenario: Scenario::Momo(0),
    },
    Entry {
        name: "모모 세 번째",
        desc: "굴로 숨기·이어 뛰기까지. 이기면 모모가 돌아오는 장면(기록은 남기지 않는다).",
        scenario: Scenario::Momo(2),
    },
    Entry {
        name: "도토",
        desc: "둘째 길 끝 굴. 끝방의 도토: 도토리 부채꼴·높이 뛰기·도토리 묻기. 이기면 기억해 내는 장면(기록은 남기지 않는다).",
        scenario: Scenario::Doto,
    },
];

/// 시나리오에 맞는 월드를 만든다.
pub fn build(s: Scenario) -> World {
    match s {
        Scenario::Normal => World::field(),
        Scenario::Field => {
            let mut w = World::field();
            w.open_glade();
            w
        }
        Scenario::Boss => {
            let mut w = World::field();
            w.open_glade();
            if let Some(g) = w.adv.glade {
                w.player.pos = vec2(g.center().x, g.y - 40.0);
                w.player.prev_pos = w.player.pos;
            }
            w.snap_cam();
            w
        }
        Scenario::Daily => {
            let mut w = World::camp_start_seeded(crate::daily::seed(crate::daily::today()));
            w.adv.events.clear();
            w
        }
        Scenario::LastCamp | Scenario::Deepest => {
            let mut w = World::camp_start(crate::tuning::FINAL_STAGE);
            w.seeds = 40;
            w.adv.events.clear();
            w
        }
        Scenario::Momo(_) | Scenario::Doto => {
            let doto = s == Scenario::Doto;
            let mut w = World::camp_start(if doto { 2 } else { 1 });
            w.seeds = 40;
            w.adv.events.clear();
            if let Scenario::Momo(wins) = s {
                w.momo_fight = true;
                w.momo_wins = wins;
            }
            w.doto_fight = doto;
            w.enter_route_boss_now();
            // 끝방 문 안쪽에 바로 선다(들어서면 잠기고 모모가 말한다).
            let boss = w.adv.dungeon.as_ref().and_then(|d| {
                d.rooms
                    .iter()
                    .find(|r| r.kind == crate::dungeon::RoomKind::Boss)
                    .map(|r| r.interior().center())
            });
            if let Some(c) = boss {
                w.player.pos = c + vec2(0.0, 40.0);
                w.player.prev_pos = w.player.pos;
                w.snap_cam();
            }
            w
        }
        Scenario::Camp => {
            let mut w = World::camp_start(1);
            w.seeds = 40;
            // 첫 쉼터 안내 컷씬은 건너뛴다(바로 확인).
            w.adv.events.clear();
            w
        }
        Scenario::Dungeon => {
            let mut w = World::field();
            w.enter_dungeon_now();
            w
        }
        Scenario::Arena => World::new(),
        Scenario::Empty
        | Scenario::CutsceneOnly
        | Scenario::StoryPreview
        | Scenario::Village
        | Scenario::Tutorial => World::empty(),
        Scenario::AllClaws => World::new(),
        Scenario::Gear => {
            let mut w = build(Scenario::Chargers(2));
            w.bag.berries = 3;
            w.bag.worms = 2;
            w.player.sp = crate::tuning::SP_MAX;
            w.player.hp = 3;
            w.loot.push(Loot::place(
                LootKind::RootHeart,
                w.room.center() + vec2(-40.0, 30.0),
            ));
            w
        }
        Scenario::AllSpecies => {
            let mut w = World::empty();
            let mobs = crate::species::BESTIARY
                .iter()
                .filter(|k| **k != Kind::Chief);
            for (i, k) in mobs.enumerate() {
                let p = spawn_point(&w, i);
                w.spawn_mob(*k, p);
            }
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
                w.spawn_enemy(Kind::FieldMouse, p);
            }
            w
        }
        Scenario::Shooters(n) => {
            let mut w = World::empty();
            for i in 0..n {
                let p = spawn_point(&w, i as usize + 1);
                w.spawn_enemy(Kind::Chipmunk, p);
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
            w.spawn_enemy(Kind::FieldMouse, a);
            w.spawn_enemy(Kind::Chipmunk, b);
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
