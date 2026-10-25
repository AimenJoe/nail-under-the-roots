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

/// 도감 순서(허수아비는 빼고).
pub const BESTIARY: [Kind; 5] = [
    Kind::FieldMouse,
    Kind::HouseMouse,
    Kind::SewerRat,
    Kind::Chipmunk,
    Kind::RedSquirrel,
];

pub fn of(kind: Kind) -> &'static Species {
    match kind {
        Kind::FieldMouse => &FIELD_MOUSE,
        Kind::HouseMouse => &HOUSE_MOUSE,
        Kind::SewerRat => &SEWER_RAT,
        Kind::Chipmunk => &CHIPMUNK,
        Kind::RedSquirrel => &RED_SQUIRREL,
        Kind::Dummy => &DUMMY,
    }
}

/// 잡몹 뽑기: `r`은 0..1 난수, `depth`가 깊을수록 시궁쥐·청설모가 늘어난다. 쥐가 주로 나온다.
pub fn roll(r: f32, depth: u32) -> Kind {
    let d = depth.min(4) as f32;
    let table = [
        (Kind::FieldMouse, 40.0 - d * 4.0),
        (Kind::HouseMouse, 18.0),
        (Kind::SewerRat, 8.0 + d * 4.0),
        (Kind::Chipmunk, 20.0),
        (Kind::RedSquirrel, 10.0 + d * 2.0),
    ];
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
            .map(|i| roll(i as f32 / n as f32, 0))
            .filter(|k| matches!(k, Kind::FieldMouse | Kind::HouseMouse | Kind::SewerRat))
            .count();
        assert!(rats * 2 > n, "쥐가 절반을 넘어야 한다: {rats}");
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
