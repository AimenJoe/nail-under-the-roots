//! 개발용 시나리오: 원하는 방 구성으로 바로 시작한다. `dev` feature(기본 켬)에서만 타이틀에 보인다.
//! 새 시나리오는 여기 목록에 한 줄 추가. 수치 조정(손맛)은 tuning.rs, 적 구성은 여기.

use macroquad::prelude::*;

use crate::biome::Biome;
use crate::enemy::{Enemy, Kind};
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
    /// 부술 수 있는 것 넷(0177): 흙더미·뿌리혹·돌무더기·실뭉치가 플레이어 둘레에.
    Breakables,
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
    /// 실의 꿈(마을의 실 고리에서, docs/decisions/0055): 빈 방에서 보스 다시 싸우기. 개발 목록에는 없다.
    Dream,
    /// 큰 나무 아래 마을(모습은 기록에서, docs/decisions/0045).
    Village,
    /// 엔딩 뒤처럼 가장 깊은 곳에서 마을로 올라온다(0204). 앱이 마을로 바꿔 시작한다.
    Ascent,
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
    /// 새 졸개 다섯(0146): 땃쥐·고슴도치·두꺼비·박쥐·포자버섯, 어두운 심층에서.
    NewMobs,
    /// 도토로 싸우기(0195): 아레나 웨이브, 플레이어는 도토(도토리 들고).
    DotoArena,
    /// 도토로 새 졸개 다섯(0195): 박쥐·고슴도치(등딱지)·두꺼비 혀를 멀리서.
    DotoMobs,
    /// 모모로 싸우기(0208): 아레나 웨이브, 플레이어는 모모. 보통 런은 모모가 돌아온 기록이 있어야 고를 수 있다.
    MomoArena,
    /// 실타래 칸 굴(0147): 방 셋, 끝방에 엘리트. 지대가 장애물을 정한다(뿌리 = 가시 덤불, 심층 = 무너지는 천장).
    Tangle(Biome),
    /// 튜토리얼 보스: 필드 공터 앞에서, 문은 열려 있다(docs/decisions/0032).
    Boss,
    /// 첫 쉼터에서 씨앗 40개로(가게·길 지도, docs/decisions/0030, 0031).
    Camp,
    /// 둘째 쉼터(뿌리 동굴 지도, 0151).
    Camp2,
    /// 첫 쉼터, 실뿌리 둘이 감긴 채 체력 1, 곁에 들쥐(0225): 런이 끝나는 쓰러짐 연출을 바로 본다.
    LastRootlet,
    /// 첫 길 끝 굴(모모가 지킨다)에 바로. 숫자 = 이미 이긴 수(0 → 첫 만남, 2 → 세 번째, docs/decisions/0046).
    Momo(u8),
    /// 둘째 길 끝 굴(도토가 지킨다)에 바로(docs/decisions/0052).
    Doto,
    /// 셋째 길 끝 굴(콩이가 지킨다)에 바로(docs/decisions/0054).
    Kongi,
    /// 굴토끼 옛 굴 끝의 토끼풀(0165).
    Clover,
    /// 길의 샘 칸(0159): 쉼방의 샘 곁에서 시작(체력은 깎인 채).
    Spring,
    /// 출구 확인(0159): 적 없는 한 방짜리 굴 — 들어서면 바로 출구가 열린다.
    Exit,
    /// 보스 2단계 확인(0233): 빈 방에 보스 하나, 체력 반 — 깨자마자 2단계로 깨어난다.
    BossHalf(Kind),
    /// 넓은 방(0244): 길의 보통 굴에서 넓은 방 안에 바로(들어서면 두 칸의 적이 함께 깬다).
    Hall,
}

pub struct Entry {
    pub name: &'static str,
    pub desc: &'static str,
    pub scenario: Scenario,
}

pub const LIST: [Entry; 46] = [
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
        name: "쉼터: 둘째",
        desc: "둘째 쉼터 지도(뿌리 동굴). 오른쪽 아래 바위 너머 숨은 자리에 반딧불·씨앗. 셋째는 \"마지막 쉼터\".",
        scenario: Scenario::Camp2,
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
        desc: "빛은 나일 주변뿐. 이끼와 빛나는 버섯.",
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
        desc: "체력 2/5. 반딧불 12마리와 산딸기 2개가 튀어나온다. 왼쪽엔 산딸기·지렁이·씨앗·뿌리 심장이 놓여 있다(주울 수 있다 표시).",
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
        name: "새 졸개 다섯",
        desc: "땃쥐(두 번 물기)·고슴도치(가시 공)·두꺼비(혀)·박쥐(솟구치기 안 닿음)·포자버섯(구름). 심층에서.",
        scenario: Scenario::NewMobs,
    },
    Entry {
        name: "실타래 굴: 뿌리",
        desc: "방 셋, 싸움방마다 가시뿌리 덤불. 끝방에 엘리트 + 졸개 둘, 이기면 장신구.",
        scenario: Scenario::Tangle(Biome::Roots),
    },
    Entry {
        name: "실타래 굴: 심층",
        desc: "방 셋, 싸움하는 동안 천장에서 돌이 떨어진다. 끝방에 엘리트 + 졸개 둘.",
        scenario: Scenario::Tangle(Biome::Deep),
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
        desc: "런의 끝: 할매와 실매듭 셋(0162). 셋 다 끊으면 반쪽 엔딩을 보고 타이틀로. 세 번 감기면 쉼터로.",
        scenario: Scenario::Deepest,
    },
    Entry {
        name: "마을",
        desc: "큰 나무 아래 마을(지금 기록의 모습).",
        scenario: Scenario::Village,
    },
    Entry {
        name: "마을로 올라오기",
        desc: "엔딩 뒤처럼 가장 깊은 곳(130m)에서 땅 위까지 올라와 뿌리 길 곁으로 튀어나온다(0204).",
        scenario: Scenario::Ascent,
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
    Entry {
        name: "길의 샘",
        desc: "길의 샘 칸. 쉼방 가운데 샘에 발을 담그면 가득 회복(0159).",
        scenario: Scenario::Spring,
    },
    Entry {
        name: "출구",
        desc: "적 없는 한 방 굴. 들어서면 내려가는 출구가 열린다(0159, 화면 밖이면 가장자리 표시).",
        scenario: Scenario::Exit,
    },
    Entry {
        name: "콩이",
        desc: "셋째 길 끝 굴. 끝방의 콩이: 굴러 튕기기·웅크려 버티기(발톱은 튕김, 솟구치기·드릴은 뚫음). 이기면 바위 발톱.",
        scenario: Scenario::Kongi,
    },
    Entry {
        name: "환경: 흰 실 굴",
        desc: "굴토끼 옛 굴의 지대(0165). 보랏빛 흙에 흰 실 얼룩, 보풀.",
        scenario: Scenario::InBiome(Biome::Silk),
    },
    Entry {
        name: "실타래 굴: 흰 실",
        desc: "흰 실 굴 3방 + 흰 실 그물(걸으면 느림, 발톱으로 끊김).",
        scenario: Scenario::Tangle(Biome::Silk),
    },
    Entry {
        name: "토끼풀",
        desc: "굴토끼 옛 굴(흰 실 굴). 끝방의 토끼풀: 발 구르기(충격파·실 그물)·두 번 차기·내려앉기, 반 아래면 품기(실 한 가닥). 고치는 못 때린다.",
        scenario: Scenario::Clover,
    },
    Entry {
        name: "부술 수 있는 것 넷",
        desc: "흙더미·뿌리혹·돌무더기·실뭉치. 쳐서 깨면 운에 따라 씨앗·산딸기·반딧불·지렁이(0177).",
        scenario: Scenario::Breakables,
    },
    Entry {
        name: "도토: 아레나",
        desc: "도토로 웨이브(0195). 던지기 3타·강타(큰 도토리)·뒤로 뛰기·묻기(게이지 가득). 받아치기는 던지는 순간.",
        scenario: Scenario::DotoArena,
    },
    Entry {
        name: "도토: 새 졸개",
        desc: "도토로 새 졸개 다섯(어두운 심층). 박쥐도 도토리는 닿는다. 고슴도치 가시는 큰 도토리·묻은 도토리로.",
        scenario: Scenario::DotoMobs,
    },
    Entry {
        name: "모모: 아레나",
        desc: "모모로 웨이브(0056). 뒷발 차기 콤보·깡충 회피·이어 뛰기(게이지 가득). 기록과 상관없이 바로.",
        scenario: Scenario::MomoArena,
    },
    Entry {
        name: "쉼터: 실뿌리 둘",
        desc: "실뿌리 둘이 감긴 채 체력 1, 곁에 들쥐(0225). 맞으면 셋째가 감기며 런이 끝나는 연출 → 확인으로 마을.",
        scenario: Scenario::LastRootlet,
    },
    Entry {
        name: "보스 2단계: 대장",
        desc: "빈 방에 들쥐 대장, 체력 반(0233). 깨자마자 2단계 — 붉은 발밑 고리, 빨라진 예고·걸음, 덜 멈칫.",
        scenario: Scenario::BossHalf(Kind::Chief),
    },
    Entry {
        name: "보스 2단계: 모모",
        desc: "첫 만남 모모, 체력 반(0233). 2단계면 첫 만남에도 굴로 숨고 뛰기를 두 번 잇는다.",
        scenario: Scenario::BossHalf(Kind::Momo),
    },
    Entry {
        name: "넓은 방",
        desc: "길의 보통 굴, 화면 두 칸짜리 넓은 방 안(0244). 카메라가 방 안을 따라가고 두 칸의 적이 함께 깬다. 방마다 지형(기둥·골짜기·돌밭).",
        scenario: Scenario::Hall,
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
        Scenario::Dream => {
            let mut w = World::empty();
            w.biome = crate::biome::Biome::Roots;
            w
        }
        Scenario::Daily => {
            let mut w = World::camp_start_seeded(crate::daily::seed(crate::daily::today()));
            w.adv.events.clear();
            w
        }
        Scenario::LastCamp | Scenario::Deepest => {
            let mut w = World::camp_start(crate::tuning::FINAL_STAGE);
            w.inv.seeds = 40;
            w.adv.events.clear();
            w
        }
        Scenario::Momo(_) | Scenario::Doto | Scenario::Kongi | Scenario::Clover => {
            let doto = s == Scenario::Doto;
            let stage = match s {
                Scenario::Doto => 2,
                Scenario::Kongi | Scenario::Clover => crate::tuning::FINAL_STAGE,
                _ => 1,
            };
            let mut w = World::camp_start(stage);
            w.inv.seeds = 40;
            w.adv.events.clear();
            w.story.seed(crate::world::StorySeed {
                momo_fight: matches!(s, Scenario::Momo(_)),
                momo_wins: if let Scenario::Momo(wins) = s {
                    wins
                } else {
                    0
                },
                doto_fight: doto,
                kongi_fight: s == Scenario::Kongi,
                clover_fight: s == Scenario::Clover,
                warren_open: s == Scenario::Clover,
                ..Default::default()
            });
            if s == Scenario::Clover {
                w.reroll_route();
                w.enter_route_warren_now();
            } else {
                w.enter_route_boss_now();
            }
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
        Scenario::LastRootlet => {
            let mut w = World::camp_start(1);
            w.adv.events.clear();
            w.adv.falls = crate::tuning::RUN_LIVES - 1;
            w.player.hp = 1;
            let at = w.player.pos + vec2(26.0, 0.0);
            w.spawn_mob(Kind::FieldMouse, at);
            w
        }
        Scenario::Camp2 => {
            let mut w = World::camp_start(2);
            w.inv.seeds = 40;
            w.adv.events.clear();
            w
        }
        Scenario::Spring => {
            let mut w = World::camp_start(1);
            w.adv.events.clear();
            w.enter_route_spring_now();
            // 샘 방(쉼방) 문 안쪽에, 체력은 반으로.
            let rest = w.adv.dungeon.as_ref().and_then(|d| {
                d.rooms
                    .iter()
                    .find(|r| r.kind == crate::dungeon::RoomKind::Rest)
                    .map(|r| r.interior().center())
            });
            if let Some(c) = rest {
                w.player.pos = c + vec2(0.0, 48.0);
                w.player.prev_pos = w.player.pos;
                w.snap_cam();
            }
            w.player.hp = (w.player.stats.max_hp / 2).max(1);
            w
        }
        Scenario::Hall => {
            let mut w = World::camp_start(1);
            w.adv.events.clear();
            w.enter_route_hall_now();
            w
        }
        Scenario::Exit => {
            let mut w = World::camp_start(1);
            w.adv.events.clear();
            w.enter_route_boss_now();
            // 끝방의 적을 비운다: 들어서면 바로 클리어 → 출구.
            if let Some(d) = w.adv.dungeon.as_mut() {
                for r in &mut d.rooms {
                    r.enemies.clear();
                }
            }
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
            w.inv.seeds = 40;
            // 첫 쉼터 안내 컷씬은 건너뛴다(바로 확인).
            w.adv.events.clear();
            w
        }
        Scenario::Dungeon => {
            let mut w = World::field();
            w.enter_dungeon_now();
            w
        }
        Scenario::Tangle(b) => {
            let mut w = World::field();
            w.enter_tangle_now(b);
            w
        }
        Scenario::Arena | Scenario::DotoArena | Scenario::MomoArena => World::new(),
        Scenario::Empty
        | Scenario::CutsceneOnly
        | Scenario::StoryPreview
        | Scenario::Village
        | Scenario::Ascent
        | Scenario::Tutorial => World::empty(),
        Scenario::AllClaws => World::new(),
        Scenario::Breakables => {
            use crate::room::{Breakable, Prop, PropKind};
            let mut w = World::empty();
            let c = w.player.pos;
            for (i, b) in [
                Breakable::Mound,
                Breakable::Gall,
                Breakable::RockPile,
                Breakable::SilkBall,
            ]
            .into_iter()
            .enumerate()
            {
                // 플레이어 아래 줄(위 줄은 빈 방의 나무가 흙더미를 가렸다).
                let at = c + vec2(-48.0 + 32.0 * i as f32, 36.0);
                w.room.add_prop(Prop {
                    rect: Rect::new(at.x - 8.0, at.y - 8.0, 16.0, 16.0),
                    kind: PropKind::Breakable(b),
                });
            }
            w
        }
        Scenario::Gear => {
            let mut w = build(Scenario::Chargers(2));
            w.inv.bag.berries = 3;
            w.inv.bag.worms = 2;
            w.player.sp = crate::tuning::SP_MAX;
            w.player.hp = 3;
            w.loot.push(Loot::place(
                LootKind::RootHeart,
                w.room.center() + vec2(-40.0, 30.0),
            ));
            w
        }
        Scenario::BossHalf(kind) => {
            let mut w = World::empty();
            let c = w.room.center();
            let mut e = Enemy::new(kind, c - vec2(0.0, 30.0));
            e.wake_delay = 1.0;
            e.knotted = true; // 실뜨기가 체력을 다시 채우지 않게(개발 확인용)
            e.hp = e.max_hp / 2;
            w.enemies.push(e);
            w.player.pos = c + vec2(0.0, 44.0);
            w.player.prev_pos = w.player.pos;
            w.player.facing = vec2(0.0, -1.0);
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
        Scenario::NewMobs | Scenario::DotoMobs => {
            let mut w = World::empty();
            w.biome = Biome::Deep;
            let mobs = [
                Kind::Shrew,
                Kind::Hedgehog,
                Kind::Toad,
                Kind::Bat,
                Kind::SporeCap,
            ];
            for (i, k) in mobs.into_iter().enumerate() {
                let p = spawn_point(&w, i);
                w.spawn_mob(k, p);
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
            // 왼쪽에 줍는 것 한 줄(판정 9의 "주울 수 있다" 표시를 보려고, 닿지 않는 거리).
            let left = w.room.center() - vec2(90.0, 0.0);
            for (i, k) in [
                LootKind::Berry,
                LootKind::Worm,
                LootKind::Seed,
                LootKind::RootHeart,
            ]
            .into_iter()
            .enumerate()
            {
                w.loot
                    .push(Loot::place(k, left + vec2(i as f32 * 18.0, 0.0)));
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

/// 시나리오를 통째로 돌리는 시험(0169 R1): 소리가 큐(`sfx::Queue`)라 월드를 틱 단위로 돌릴 수 있다.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Actions;
    use crate::tuning as t;
    use macroquad::math::Vec2;

    /// 개발 시나리오 전부: 가만히 서서 600틱(10초)을 돌려도 panic이 없고 플레이어가 살아 있거나 쓰러졌거나 둘 중 하나다.
    #[test]
    fn every_dev_scenario_runs_ten_seconds_without_panicking() {
        let idle = Actions::default();
        for e in LIST.iter() {
            let mut w = build(e.scenario);
            for _ in 0..600 {
                w.tick(&idle);
            }
            assert!(w.time > 0.0, "{} 시나리오가 돌았다", e.name);
        }
    }

    /// 들쥐 한 마리 방: 다가가 연타하면 쓰러진다 — 공격·피격·쓰러짐·떨어뜨림·소리가 한 틱 흐름 안에서 이어진다.
    #[test]
    fn mashing_at_a_field_mouse_kills_it_and_queues_the_sounds() {
        let mut w = build(Scenario::Chargers(1));
        w.player.armed = true;
        let mut tick = 0;
        while tick < 60 * 20 && w.stats.kills == 0 && w.player.is_alive() {
            let target = w
                .enemies
                .iter()
                .filter(|e| e.alive)
                .map(|e| e.pos)
                .min_by(|a, b| {
                    (*a - w.player.pos)
                        .length()
                        .total_cmp(&(*b - w.player.pos).length())
                });
            let Some(target) = target else {
                break;
            };
            let to = target - w.player.pos;
            let near = to.length() < 14.0;
            let dir = to.normalize_or(Vec2::X);
            let a = Actions {
                move_dir: if near { Vec2::ZERO } else { dir },
                aim: dir,
                attack: near && tick % 2 == 0,
                ..Default::default()
            };
            w.tick(&a);
            tick += 1;
        }
        assert!(w.stats.kills >= 1, "{}틱 안에 들쥐를 쓰러뜨린다", tick);
        assert!(
            w.juice.audio.played(crate::sfx::Id::EnemyHit)
                || w.juice.audio.played(crate::sfx::Id::EnemyDie)
        );
        assert!(w.time >= t::TICK_DT);
    }
}
