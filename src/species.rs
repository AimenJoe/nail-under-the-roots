//! 적 종류 표. 적은 뿌리에 물든 땅속 이웃(진짜 동물)이다. 잡몹은 쥐가 주로, 다람쥐·청설모가 섞인다(docs/decisions/0021).
//!
//! 한 종 = 행동 하나(돌격·던지기·뛰어 덮치기) + 수치 + 도감 글. 수치는 이 표 한 곳에(손맛 튜닝은 여기서).
//! 도감은 숫자를 보이지 않는다: `words()`가 수치를 말("튼튼함", "아주 빠름")로 바꾼다.

use macroquad::prelude::Color;

use crate::enemy::Kind;
use crate::i18n::Tr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behavior {
    /// 다가가서 떨다가(예고) 곧게 돌진.
    Charger,
    /// 거리를 두고 조준했다가 던진다.
    Shooter,
    /// 웅크렸다가(착지 자리 표시) 뛰어 덮친다. 착지할 때 둘레에 피해.
    Leaper,
    /// 허수아비.
    Dummy,
    /// 뒤쥐(0146): 짧게 떨고 두 번 연달아 문다(돌진 둘). 작고 빠르다.
    Darter,
    /// 고슴도치(0146): 웅크려 공으로 굴러 온다. 말려 있는 동안 발톱은 가시에 튕긴다(콩이와 같은 규칙). 벽에 박으면 어질어질.
    Roller,
    /// 두꺼비(0146): 거리를 두고 겨눴다가 혀를 내민다. 맞으면 끌려간다. 받아치면(패링) 멈칫.
    Tongue,
    /// 박쥐(0146): 날아다니며 옆으로 돌다가 내리꽂는다. 날고 있어 솟구치기가 닿지 않는다.
    Swooper,
    /// 실버섯(0146): 움직이지 않는다. 가까이 오면 부풀었다가 포자 구름을 터뜨린다(쓰러져도). 땅속에선 안 맞는다.
    Spore,
    /// 들쥐 대장(튜토리얼 보스): 돌진 ↔ 뛰어 찍기, 체력 반 아래면 생쥐를 부른다. 공격 중엔 안 멈춘다(enemy.rs).
    Chief,
    /// 모모(첫 길의 보스, 0046): 높이 뛰기 ↔ 뒷발 차기, 두 번째부터 굴로 숨었다 튀어나오기, 세 번째엔 이어 뛰기.
    /// 대장처럼 공격 중엔 안 멈춘다. 이길 때마다 한 수씩 는다(`Enemy::level`).
    Momo,
    /// 도토(둘째 길의 보스, 0052): 도토리 부채꼴 던지기(되받아칠 수 있다) → 높이 뛰기 → 도토리 묻기(터지기 전에 비키거나 파낸다).
    /// 거리를 두려 한다. 체력이 반 아래면 던지는 수·묻는 수가 는다.
    Doto,
    /// 콩이(셋째 길의 보스, 0054): 공으로 굴러 벽에 튕기기 ↔ 웅크려 버티기(발톱은 튕긴다, 치면 바로 굴러 온다 — 패링 거울).
    /// 솟구치기·드릴은 등딱지를 뚫고 오래 어질어질하게 한다. 구르기 뒤 어질어질할 때가 틈. 반 아래면 더 많이 튕긴다.
    Kongi,
}

pub struct Species {
    pub name: Tr,
    /// 도감: 원래 어떤 이웃이었나, 지금은.
    pub lore: Tr,
    /// 도감: 어떻게 공격하나 / 어떻게 상대하나.
    pub attack: Tr,
    pub tip: Tr,
    pub behavior: Behavior,
    pub hp: i32,
    pub radius: f32,
    /// 걷기(거리 맞추기) 속도.
    pub speed: f32,
    pub damage: i32,
    /// 받는 넉백 배율. 무거우면 작다.
    pub knock_mul: f32,
    /// 돌격·뛰기를 시작하는 거리.
    pub trigger: f32,
    /// 예고(떨림·웅크림·조준) 시간.
    pub windup: f32,
    /// 돌진 속도(돌격형).
    pub dash_speed: f32,
    /// 돌진·체공 시간.
    pub dash_time: f32,
    pub rest: f32,
    pub wall_rest: f32,
    /// 던지기: 유지 거리(가까우면 물러나고 멀면 다가간다), 발사 간격, 탄속.
    pub keep: (f32, f32),
    pub fire_every: f32,
    pub shot_speed: f32,
    /// 뛰어 덮치기: 착지 피해 반지름.
    pub land_radius: f32,
    /// 떨어뜨리는 반딧불(최소, 최대)과 씨앗 확률.
    pub glowbugs: (u32, u32),
    pub seed_chance: f32,
    /// 그림이 없을 때 도형 색, 그림 시트(assets/art/critters.png) 줄.
    pub color: Color,
    pub sheet_row: u32,
}

const BASE: Species = Species {
    name: |s| s.mob_field_mouse,
    lore: |s| s.mob_field_mouse_lore,
    attack: |s| s.mob_field_mouse_atk,
    tip: |s| s.mob_field_mouse_tip,
    behavior: Behavior::Charger,
    hp: 3,
    radius: 6.0,
    speed: 38.0,
    damage: 1,
    knock_mul: 1.0,
    trigger: 75.0,
    windup: 0.45,
    dash_speed: 230.0,
    dash_time: 0.35,
    rest: 0.7,
    wall_rest: 1.1,
    keep: (60.0, 110.0),
    fire_every: 1.6,
    shot_speed: 70.0,
    land_radius: 0.0,
    glowbugs: (2, 3),
    seed_chance: 0.08,
    color: Color::new(0.85, 0.30, 0.30, 1.0),
    sheet_row: 0,
};

pub static FIELD_MOUSE: Species = BASE;

pub static HOUSE_MOUSE: Species = Species {
    name: |s| s.mob_house_mouse,
    lore: |s| s.mob_house_mouse_lore,
    attack: |s| s.mob_house_mouse_atk,
    tip: |s| s.mob_house_mouse_tip,
    hp: 1,
    radius: 4.0,
    speed: 62.0,
    trigger: 42.0,
    windup: 0.25,
    dash_speed: 200.0,
    dash_time: 0.22,
    rest: 0.45,
    wall_rest: 0.8,
    glowbugs: (1, 1),
    seed_chance: 0.03,
    color: Color::new(0.80, 0.62, 0.62, 1.0),
    sheet_row: 1,
    ..BASE
};

pub static SEWER_RAT: Species = Species {
    name: |s| s.mob_sewer_rat,
    lore: |s| s.mob_sewer_rat_lore,
    attack: |s| s.mob_sewer_rat_atk,
    tip: |s| s.mob_sewer_rat_tip,
    hp: 7,
    radius: 8.0,
    speed: 30.0,
    damage: 2,
    knock_mul: 0.45,
    trigger: 90.0,
    windup: 0.7,
    dash_speed: 260.0,
    dash_time: 0.55,
    rest: 1.0,
    wall_rest: 1.6,
    glowbugs: (4, 6),
    seed_chance: 0.3,
    color: Color::new(0.55, 0.35, 0.45, 1.0),
    sheet_row: 2,
    ..BASE
};

pub static CHIPMUNK: Species = Species {
    name: |s| s.mob_chipmunk,
    lore: |s| s.mob_chipmunk_lore,
    attack: |s| s.mob_chipmunk_atk,
    tip: |s| s.mob_chipmunk_tip,
    behavior: Behavior::Shooter,
    hp: 2,
    radius: 6.0,
    speed: 42.0,
    windup: 0.4,
    fire_every: 1.7,
    shot_speed: 72.0,
    glowbugs: (3, 4),
    seed_chance: 0.12,
    color: Color::new(0.60, 0.40, 0.85, 1.0),
    sheet_row: 3,
    ..BASE
};

pub static RED_SQUIRREL: Species = Species {
    name: |s| s.mob_red_squirrel,
    lore: |s| s.mob_red_squirrel_lore,
    attack: |s| s.mob_red_squirrel_atk,
    tip: |s| s.mob_red_squirrel_tip,
    behavior: Behavior::Leaper,
    hp: 3,
    radius: 6.0,
    speed: 55.0,
    trigger: 100.0,
    windup: 0.55,
    dash_time: 0.45,
    rest: 0.8,
    keep: (50.0, 90.0),
    land_radius: 15.0,
    glowbugs: (3, 5),
    seed_chance: 0.15,
    color: Color::new(0.75, 0.42, 0.25, 1.0),
    sheet_row: 4,
    ..BASE
};

pub static DUMMY: Species = Species {
    name: |s| s.mob_dummy,
    lore: |s| s.mob_dummy_lore,
    attack: |s| s.mob_dummy_atk,
    tip: |s| s.mob_dummy_tip,
    behavior: Behavior::Dummy,
    hp: 6,
    radius: 7.0,
    speed: 0.0,
    glowbugs: (0, 0),
    seed_chance: 0.0,
    color: Color::new(0.75, 0.62, 0.40, 1.0),
    sheet_row: 5,
    ..BASE
};

/// 뒤쥐(0146): 실제 뒤쥐(북방짧은꼬리뒤쥐·물뒤쥐)는 침샘의 독으로 지렁이·벌레를 마비시켜 산 채로 저장한다.
/// 물든 뒤쥐는 그 버릇대로 짧게 두 번 문다. 체력 낮고 작아 한 번 맞히면 끝나지만, 쫓아가면 피한다.
pub static SHREW: Species = Species {
    name: |s| s.mob_shrew,
    lore: |s| s.mob_shrew_lore,
    attack: |s| s.mob_shrew_atk,
    tip: |s| s.mob_shrew_tip,
    behavior: Behavior::Darter,
    hp: 2,
    radius: 4.5,
    speed: 66.0,
    knock_mul: 1.1,
    trigger: 55.0,
    windup: 0.22,
    dash_speed: 270.0,
    dash_time: 0.16,
    rest: 0.6,
    wall_rest: 0.8,
    glowbugs: (2, 3),
    seed_chance: 0.06,
    color: Color::new(0.48, 0.40, 0.44, 1.0),
    sheet_row: crate::art::SHREW_ROW,
    ..BASE
};

/// 고슴도치(0146): 위협받으면 등의 고리 근육으로 가시를 세우고 공처럼 말린다(실제 습성, 지렁이·딱정벌레를 먹는다).
/// 손맛: 콩이를 작게 만든 졸개. 말린 공에 발톱은 튕기고, 솟구치기·드릴은 연다. 벽에 박은 뒤가 틈.
pub static HEDGEHOG: Species = Species {
    name: |s| s.mob_hedgehog,
    lore: |s| s.mob_hedgehog_lore,
    attack: |s| s.mob_hedgehog_atk,
    tip: |s| s.mob_hedgehog_tip,
    behavior: Behavior::Roller,
    hp: 5,
    radius: 6.5,
    speed: 30.0,
    knock_mul: 0.6,
    trigger: 95.0,
    windup: 0.6,
    dash_speed: 190.0,
    dash_time: 1.1,
    rest: 0.7,
    wall_rest: 1.8,
    glowbugs: (3, 5),
    seed_chance: 0.15,
    color: Color::new(0.52, 0.44, 0.36, 1.0),
    sheet_row: crate::art::HEDGEHOG_ROW,
    ..BASE
};

/// 두꺼비(0146): 밤에 천천히 다가가 끈끈한 혀로 벌레·지렁이를 잡는다(실제 습성). 느리고 단단하다.
pub static TOAD: Species = Species {
    name: |s| s.mob_toad,
    lore: |s| s.mob_toad_lore,
    attack: |s| s.mob_toad_atk,
    tip: |s| s.mob_toad_tip,
    behavior: Behavior::Tongue,
    hp: 4,
    radius: 6.5,
    speed: 22.0,
    knock_mul: 0.5,
    windup: 0.55,
    rest: 0.9,
    keep: (36.0, 66.0),
    fire_every: 2.0,
    glowbugs: (3, 5),
    seed_chance: 0.15,
    color: Color::new(0.46, 0.52, 0.34, 1.0),
    sheet_row: crate::art::TOAD_ROW,
    ..BASE
};

/// 박쥐(0146): 깊은 굴의 천장에 매달려 사는 이웃. 물든 박쥐는 날며 맴돌다 내리꽂는다.
pub static BAT: Species = Species {
    name: |s| s.mob_bat,
    lore: |s| s.mob_bat_lore,
    attack: |s| s.mob_bat_atk,
    tip: |s| s.mob_bat_tip,
    behavior: Behavior::Swooper,
    hp: 2,
    radius: 5.0,
    speed: 58.0,
    knock_mul: 1.2,
    windup: 0.45,
    dash_speed: 240.0,
    dash_time: 0.4,
    rest: 0.6,
    wall_rest: 0.9,
    keep: (46.0, 84.0),
    fire_every: 1.8,
    glowbugs: (2, 4),
    seed_chance: 0.1,
    color: Color::new(0.42, 0.34, 0.50, 1.0),
    sheet_row: crate::art::BAT_ROW,
    ..BASE
};

/// 실버섯(0146): 이웃이 아니라 할매의 실이 맺은 버섯(뽕나무버섯류처럼 땅속 균사가 솟아 맺는다, KB animal-behavior). 포자를 터뜨린다.
pub static SPORE_CAP: Species = Species {
    name: |s| s.mob_spore,
    lore: |s| s.mob_spore_lore,
    attack: |s| s.mob_spore_atk,
    tip: |s| s.mob_spore_tip,
    behavior: Behavior::Spore,
    hp: 3,
    radius: 6.0,
    speed: 0.0,
    knock_mul: 0.0,
    trigger: 46.0,
    windup: 0.7,
    rest: 0.8,
    keep: (0.0, 0.0),
    fire_every: 2.6,
    glowbugs: (2, 4),
    seed_chance: 0.12,
    color: Color::new(0.80, 0.74, 0.86, 1.0),
    sheet_row: crate::art::SPORE_ROW,
    ..BASE
};

/// 들쥐 대장(튜토리얼 보스, docs/decisions/0032). 원래는 들쥐 마을의 대장 보리. 뿌리에 물들어 남쪽 공터를 지킨다.
/// 손맛: 예고가 길고(0.75초) 크게 보이는 대신, 맞아도 공격을 멈추지 않는다. 벽에 박으면 2초 넘게 어질어질(때릴 때).
/// 그림은 32×32 따로(assets/art/chief.png). `sheet_row`는 쓰지 않는다.
pub static CHIEF: Species = Species {
    name: |s| s.mob_chief,
    lore: |s| s.mob_chief_lore,
    attack: |s| s.mob_chief_atk,
    tip: |s| s.mob_chief_tip,
    behavior: Behavior::Chief,
    hp: 36,
    radius: 11.0,
    speed: 34.0,
    damage: 1,
    knock_mul: 0.15,
    trigger: 120.0,
    windup: 0.75,
    dash_speed: 250.0,
    dash_time: 0.6,
    rest: 0.9,
    wall_rest: 2.2,
    land_radius: 26.0,
    glowbugs: (20, 24),
    seed_chance: 1.0,
    color: Color::new(0.62, 0.45, 0.36, 1.0),
    sheet_row: 0,
    ..BASE
};

/// 모모(물든 굴토끼, 0029·0046). 첫 길 끝의 보스로 세 번 만난다. 대장보다 가볍고 빠르다: 넉백은 조금 받고,
/// 체공(0.5초)이 길어 착지 자리를 보고 피할 수 있다. 차기는 짧고 빠르다(tuning MOMO_KICK_TIME).
pub static MOMO: Species = Species {
    name: |s| s.mob_momo,
    lore: |s| s.mob_momo_lore,
    attack: |s| s.mob_momo_atk,
    tip: |s| s.mob_momo_tip,
    behavior: Behavior::Momo,
    hp: 28,
    radius: 8.0,
    speed: 50.0,
    damage: 1,
    knock_mul: 0.25,
    trigger: 70.0,
    windup: 0.6,
    dash_speed: 320.0,
    dash_time: 0.5,
    rest: 0.85,
    wall_rest: 1.8,
    land_radius: 20.0,
    glowbugs: (16, 20),
    seed_chance: 1.0,
    color: Color::new(0.55, 0.40, 0.56, 1.0),
    sheet_row: crate::art::MOMO_ROW,
    ..BASE
};

/// 도토(물든 청설모, 0037·0052). 둘째 길 "뿌리 엉킨 곳"의 끝에서 한 번 만난다. 무엇이든 묻고 지킨다.
/// 손맛: 멀리서 던지는 보스. 도토리는 패링으로 되받아쳐 맞힐 수 있다(그게 가장 빠른 길). 뛰기는 청설모 졸개보다 넓게 찍는다.
pub static DOTO: Species = Species {
    name: |s| s.mob_doto,
    lore: |s| s.mob_doto_lore,
    attack: |s| s.mob_doto_atk,
    tip: |s| s.mob_doto_tip,
    behavior: Behavior::Doto,
    hp: 34,
    radius: 8.0,
    speed: 46.0,
    damage: 1,
    knock_mul: 0.25,
    trigger: 80.0,
    windup: 0.55,
    dash_time: 0.5,
    rest: 0.8,
    wall_rest: 1.6,
    keep: (50.0, 110.0),
    shot_speed: 80.0,
    land_radius: 18.0,
    glowbugs: (20, 24),
    seed_chance: 1.0,
    color: Color::new(0.70, 0.40, 0.26, 1.0),
    sheet_row: crate::art::DOTO_ROW,
    ..BASE
};

/// 콩이(물든 세띠아르마딜로, 0037·0054). 셋째 길 끝 "깊은 곳의 문"을 지킨다. 아무도 더 내려가지 못하게 몸을 만다.
/// 손맛: 무겁고 느리게 걷지만 구르면 빠르다. 넉백을 거의 안 받는다. 벽에 다 튕기고 나면 길게 어질어질(때릴 때).
pub static KONGI: Species = Species {
    name: |s| s.mob_kongi,
    lore: |s| s.mob_kongi_lore,
    attack: |s| s.mob_kongi_atk,
    tip: |s| s.mob_kongi_tip,
    behavior: Behavior::Kongi,
    hp: 40,
    radius: 9.0,
    speed: 30.0,
    damage: 1,
    knock_mul: 0.12,
    trigger: 130.0,
    windup: 0.7,
    dash_speed: 230.0,
    dash_time: 2.4,
    rest: 0.8,
    wall_rest: 1.8,
    glowbugs: (24, 28),
    seed_chance: 1.0,
    color: Color::new(0.55, 0.50, 0.46, 1.0),
    sheet_row: crate::art::KONGI_ROW,
    ..BASE
};

/// 도감 순서(허수아비는 빼고).
pub const BESTIARY: [Kind; 14] = [
    Kind::FieldMouse,
    Kind::HouseMouse,
    Kind::SewerRat,
    Kind::Chipmunk,
    Kind::RedSquirrel,
    Kind::Shrew,
    Kind::Hedgehog,
    Kind::Toad,
    Kind::Bat,
    Kind::SporeCap,
    Kind::Chief,
    Kind::Momo,
    Kind::Doto,
    Kind::Kongi,
];

pub fn of(kind: Kind) -> &'static Species {
    match kind {
        Kind::FieldMouse => &FIELD_MOUSE,
        Kind::HouseMouse => &HOUSE_MOUSE,
        Kind::SewerRat => &SEWER_RAT,
        Kind::Chipmunk => &CHIPMUNK,
        Kind::RedSquirrel => &RED_SQUIRREL,
        Kind::Shrew => &SHREW,
        Kind::Hedgehog => &HEDGEHOG,
        Kind::Toad => &TOAD,
        Kind::Bat => &BAT,
        Kind::SporeCap => &SPORE_CAP,
        Kind::Dummy => &DUMMY,
        Kind::Chief => &CHIEF,
        Kind::Momo => &MOMO,
        Kind::Doto => &DOTO,
        Kind::Kongi => &KONGI,
    }
}

/// 잡몹 뽑기: `r`은 0..1 난수, `depth`가 깊을수록 시궁쥐·청설모가 늘어난다. 쥐가 주로 나온다.
/// 바이옴마다 대표 적이 섞인다(0146): 들판 = 뒤쥐·고슴도치, 뿌리 = 두꺼비(+ 뒤쥐·고슴도치·실버섯 조금), 심층 = 박쥐·실버섯.
pub fn roll(r: f32, depth: u32, biome: crate::biome::Biome) -> Kind {
    use crate::biome::Biome;
    let d = depth.min(4) as f32;
    let table: Vec<(Kind, f32)> = match biome {
        Biome::Meadow => vec![
            (Kind::FieldMouse, 36.0 - d * 4.0),
            (Kind::HouseMouse, 16.0),
            (Kind::SewerRat, 6.0 + d * 3.0),
            (Kind::Chipmunk, 18.0),
            (Kind::RedSquirrel, 9.0 + d * 2.0),
            (Kind::Shrew, 10.0),
            (Kind::Hedgehog, 4.0 + d * 2.0),
        ],
        Biome::Roots => vec![
            (Kind::FieldMouse, 26.0 - d * 2.0),
            (Kind::HouseMouse, 12.0),
            (Kind::SewerRat, 10.0 + d * 3.0),
            (Kind::Chipmunk, 12.0),
            (Kind::RedSquirrel, 9.0 + d * 2.0),
            (Kind::Shrew, 8.0),
            (Kind::Hedgehog, 6.0 + d * 2.0),
            (Kind::Toad, 10.0 + d),
            (Kind::SporeCap, 3.0),
        ],
        Biome::Deep => vec![
            (Kind::FieldMouse, 16.0),
            (Kind::HouseMouse, 8.0),
            (Kind::SewerRat, 14.0 + d * 3.0),
            (Kind::Chipmunk, 6.0),
            (Kind::RedSquirrel, 8.0),
            (Kind::Hedgehog, 4.0),
            (Kind::Toad, 6.0),
            (Kind::Bat, 14.0 + d * 2.0),
            (Kind::SporeCap, 10.0 + d * 2.0),
        ],
        // 흰 실 굴(0165): 새 잡몹 없이 섞는다. 실버섯(할매의 균사)과 박쥐가 대표.
        Biome::Silk => vec![
            (Kind::FieldMouse, 10.0),
            (Kind::SewerRat, 14.0 + d * 3.0),
            (Kind::Shrew, 10.0),
            (Kind::Hedgehog, 6.0),
            (Kind::Toad, 6.0),
            (Kind::Bat, 16.0 + d * 2.0),
            (Kind::SporeCap, 16.0 + d * 2.0),
        ],
    };
    let total: f32 = table.iter().map(|e| e.1).sum();
    let mut x = r.clamp(0.0, 0.9999) * total;
    for (k, w) in table {
        if x < w {
            return k;
        }
        x -= w;
    }
    Kind::FieldMouse
}

/// 한 자리에 놓을 때의 위치들(생쥐는 셋이 둘러서 떼로).
pub fn pack_offsets(kind: Kind) -> Vec<macroquad::prelude::Vec2> {
    use macroquad::prelude::*;
    if kind == Kind::HouseMouse {
        (0..3)
            .map(|i| Vec2::from_angle(i as f32 * std::f32::consts::TAU / 3.0) * 10.0)
            .collect()
    } else {
        vec![Vec2::ZERO]
    }
}

/// 도감용 말: (체력, 힘, 빠르기). 숫자를 보이지 않고 종끼리 비교되는 말로.
pub fn words(s: &Species) -> (Tr, Tr, Tr) {
    let hp: Tr = match s.hp {
        ..=1 => |s| s.w_hp_1,
        2..=3 => |s| s.w_hp_2,
        4..=5 => |s| s.w_hp_3,
        _ => |s| s.w_hp_4,
    };
    let power: Tr = match s.damage {
        ..=1 => |s| s.w_pow_1,
        _ => |s| s.w_pow_2,
    };
    // 빠르기: 걷는 속도(쫓아오는 느낌). 돌진·뛰기는 "공격" 글이 말한다.
    let speed: Tr = match s.speed {
        x if x < 35.0 => |s| s.w_spd_1,
        x if x < 45.0 => |s| s.w_spd_2,
        x if x < 60.0 => |s| s.w_spd_3,
        _ => |s| s.w_spd_4,
    };
    (hp, power, speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_are_mostly_rats() {
        let n = 1000;
        let rats = (0..n)
            .map(|i| roll(i as f32 / n as f32, 0, crate::biome::Biome::Meadow))
            .filter(|k| matches!(k, Kind::FieldMouse | Kind::HouseMouse | Kind::SewerRat))
            .count();
        assert!(rats * 2 > n, "쥐가 절반을 넘어야 한다: {rats}");
    }

    /// 바이옴마다 대표 적(0146): 들판엔 박쥐·실버섯이 없고, 심층엔 박쥐·실버섯이 나오며 쥐도 남는다.
    #[test]
    fn each_biome_has_its_own_mobs() {
        use crate::biome::Biome;
        let n = 2000;
        let kinds =
            |b: Biome| -> Vec<Kind> { (0..n).map(|i| roll(i as f32 / n as f32, 2, b)).collect() };
        let meadow = kinds(Biome::Meadow);
        assert!(!meadow
            .iter()
            .any(|k| matches!(k, Kind::Bat | Kind::SporeCap | Kind::Toad)));
        assert!(meadow.contains(&Kind::Shrew) && meadow.contains(&Kind::Hedgehog));
        let roots = kinds(Biome::Roots);
        assert!(roots.contains(&Kind::Toad) && !roots.contains(&Kind::Bat));
        let deep = kinds(Biome::Deep);
        assert!(deep.contains(&Kind::Bat) && deep.contains(&Kind::SporeCap));
        let rats = deep
            .iter()
            .filter(|k| matches!(k, Kind::FieldMouse | Kind::HouseMouse | Kind::SewerRat))
            .count();
        assert!(rats * 3 > n, "심층에도 쥐가 남는다: {rats}");
    }

    #[test]
    fn words_tell_species_apart() {
        let tough = words(&SEWER_RAT);
        let weak = words(&HOUSE_MOUSE);
        let s = &crate::i18n::KO;
        assert_ne!((tough.0)(s), (weak.0)(s), "체력 말이 달라야 한다");
        assert_ne!((tough.1)(s), (weak.1)(s), "힘 말이 달라야 한다");
        for k in BESTIARY {
            assert!(of(k).hp > 0 && of(k).radius > 0.0);
        }
    }
}
