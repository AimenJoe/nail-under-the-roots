//! 안내 화면의 힌트 그림(0180): 주제마다 진짜 그림(Naïl·생쥐·허수아비·소품)으로 그 기능의 한 장면을 되풀이한다.
//! 좌표는 그림 칸의 게임 픽셀(왼쪽 위 0,0). 칸 밖으로 나가지 않게 그린다(자르지 않는다).
//! 그림이 없으면(art 없음) 색 동그라미로.

use super::*;
use crate::art::frame as F;
use crate::enemy::Kind;
use crate::help::Pic;

/// 발바닥 줄(칸 위에서). 칸 높이 40 기준(`app/help.rs` PIC_H).
const FLOOR: f32 = 30.0;

fn nail(art: Option<&Art>, col: u32, row: u32, at: Vec2, left: bool) {
    draw_ellipse(at.x, at.y + 6.0, 9.0, 2.6, 0.0, SHADOW_SOFT);
    match art {
        Some(a) => draw_texture_ex(
            &a.nail,
            art::nail_origin(at).x.round(),
            art::nail_origin(at).y.round(),
            WHITE,
            DrawTextureParams {
                source: Some(art::nail_cell(col, row)),
                dest_size: Some(vec2(art::NAIL_CELL, art::NAIL_CELL)),
                flip_x: left,
                ..Default::default()
            },
        ),
        None => draw_circle(at.x, at.y, 6.0, Color::new(0.45, 0.33, 0.26, 1.0)),
    }
}

fn critter(art: Option<&Art>, kind: Kind, frame: u32, at: Vec2, left: bool, tint: Color) {
    draw_ellipse(at.x, at.y + 6.0, 5.0, 2.0, 0.0, SHADOW_SOFT);
    match art.and_then(|a| a.critter(frame, kind.species().sheet_row)) {
        Some(c) => c.draw(
            vec2(at.x, (at.y - 8.0).round() + art::CRITTER_FOOT),
            Vec2::ONE,
            left,
            tint,
        ),
        None => draw_circle(at.x, at.y, 5.0, kind.species().color),
    }
}

/// 돌아온 도토(`DOTO_BACK_ROW`, 새 그림 시트 — 칸 0·1 서기, 4 겨누기, 5 던지기). 그림이 없으면 밤색 동그라미.
fn doto(art: Option<&Art>, frame: u32, at: Vec2, lift: f32, left: bool) {
    kin(art, art::DOTO_BACK_ROW, frame, at, lift, left);
}

fn momo(art: Option<&Art>, frame: u32, at: Vec2, lift: f32, left: bool) {
    kin(art, art::MOMO_BLOOM_ROW, frame, at, lift, left);
}

/// 고를 수 있는 이웃(모모·도토)을 플레이어 그리기와 같은 발 자리로. `lift` = 뛰는 높이(그림자는 바닥에).
fn kin(art: Option<&Art>, row: u32, frame: u32, at: Vec2, lift: f32, left: bool) {
    draw_ellipse(at.x, at.y + 6.0, 5.0, 2.0, 0.0, SHADOW_SOFT);
    match art.and_then(|a| a.critter(frame, row)) {
        Some(c) => c.draw(
            vec2(at.x, (at.y - 9.0 - lift).round() + art::CRITTER_FOOT),
            Vec2::ONE,
            left,
            WHITE,
        ),
        None => draw_circle(at.x, at.y - lift, 5.0, Color::new(0.72, 0.38, 0.20, 1.0)),
    }
}

/// 도토리 한 알(2×3 + 깍정이).
fn acorn(at: Vec2) {
    let (x, y) = (at.x.round(), at.y.round());
    draw_rectangle(x - 1.0, y - 1.0, 3.0, 3.0, OUTLINE);
    draw_rectangle(x, y, 1.0, 1.0, Color::new(0.80, 0.55, 0.28, 1.0));
    draw_rectangle(
        x - 1.0,
        y - 2.0,
        3.0,
        1.0,
        Color::new(0.45, 0.30, 0.18, 1.0),
    );
}

/// 체력 하트 하나(꽉 참·빔).
fn heart(x: f32, y: f32, full: bool) {
    let c = if full {
        Color::new(0.93, 0.30, 0.38, 1.0)
    } else {
        Color::new(0.30, 0.22, 0.26, 1.0)
    };
    draw_rectangle(x - 1.0, y - 1.0, 7.0, 4.0, OUTLINE);
    draw_rectangle(x, y + 3.0, 5.0, 2.0, OUTLINE);
    draw_rectangle(x + 1.0, y + 5.0, 3.0, 1.0, OUTLINE);
    draw_rectangle(x, y, 2.0, 3.0, c);
    draw_rectangle(x + 3.0, y, 2.0, 3.0, c);
    draw_rectangle(x + 1.0, y + 3.0, 3.0, 2.0, c);
}

/// 별 셋이 머리 위를 돈다(어질어질).
fn dizzy(at: Vec2, t: f32) {
    for i in 0..3 {
        let a = t * 6.0 + i as f32 * std::f32::consts::TAU / 3.0;
        let q = at + vec2(a.cos() * 6.0, a.sin() * 2.0 - 10.0);
        draw_rectangle(
            q.x.round(),
            q.y.round(),
            2.0,
            2.0,
            Color::new(1.0, 0.95, 0.6, 1.0),
        );
    }
}

/// 힌트 그림 한 장의 바탕(0230 R18): 칸 크기·시각·바닥 줄·자주 쓰는 색. 주제마다 함수 하나(`pic_*`).
#[derive(Clone, Copy)]
struct PicCtx<'a> {
    art: Option<&'a Art>,
    w: f32,
    time: f32,
    mid: f32,
    y: f32,
    gold: Color,
    mark: Color,
}

pub(super) fn draw_pic(art: Option<&Art>, pic: Pic, w: f32, h: f32, time: f32) {
    let c = PicCtx {
        art,
        w,
        time,
        mid: w * 0.5,
        y: FLOOR.min(h - 10.0),
        gold: Color::new(1.0, 0.88, 0.45, 0.9),
        mark: Color::new(1.0, 0.95, 0.85, 1.0),
    };
    match pic {
        Pic::Move => pic_move(c),
        Pic::Combo => pic_combo(c),
        Pic::Heavy => pic_heavy(c),
        Pic::Burrow => pic_burrow(c),
        Pic::Parry => pic_parry(c),
        Pic::Special => pic_special(c),
        Pic::Eat => pic_eat(c),
        Pic::Enemy => pic_enemy(c),
        Pic::Loot => pic_loot(c),
        Pic::Camp => pic_camp(c),
        Pic::Gear => pic_gear(c),
        Pic::Break => pic_break(c),
        Pic::Doto => pic_doto(c),
        Pic::Run => pic_run(c),
        Pic::Village => pic_village(c),
    }
}

fn pic_move(c: PicCtx) {
    let PicCtx {
        art, w, time, y, ..
    } = c;
    // 좌우로 걷는다. 발밑 먼지.
    let ph = (time / 2.4).fract();
    let k = if ph < 0.5 { ph * 2.0 } else { 2.0 - ph * 2.0 };
    let x = 16.0 + (w - 32.0) * k;
    let left = ph >= 0.5;
    nail(art, 2 + ((time * 10.0) as u32) % 4, 0, vec2(x, y), left);
    for i in 0..3 {
        let d = ((time * 3.0 + i as f32 * 0.33).fract()) * 6.0;
        let px = x + if left { d + 4.0 } else { -d - 4.0 };
        draw_rectangle(
            px.round(),
            y + 5.0 - d * 0.3,
            1.0,
            1.0,
            Color::new(0.75, 0.62, 0.45, 1.0 - d / 6.0),
        );
    }
}

fn pic_combo(c: PicCtx) {
    let PicCtx {
        art,
        time,
        mid,
        y,
        mark,
        ..
    } = c;
    // 1·2·3타가 박자(`BEAT_PERIOD`)마다, 끝에 쉼 0.5초. 몸의 고리가 과녁으로 닫히고 닿는 순간 번쩍 + 똑, 그때 친다.
    let p = t::BEAT_PERIOD;
    let ph = time % (3.0 * p + 0.5);
    let at = vec2(mid - 10.0, y);
    critter(art, Kind::Dummy, F::IDLE, at + vec2(18.0, 0.0), true, WHITE);
    let i = (ph / p) as usize;
    if i < 3 {
        let k = (ph - i as f32 * p) / p;
        let hit = k < 0.45;
        let col = if hit {
            if i == 2 {
                2
            } else {
                i as u32
            }
        } else {
            5 + i.min(2) as u32
        };
        nail(art, col, 1, at, false);
        if hit {
            let sweep = if i.is_multiple_of(2) { 1.0 } else { -1.0 };
            actors::draw_claw_marks(
                at + vec2(2.0, 0.0),
                vec2(1.0, 0.0),
                14.0,
                k / 0.45,
                i == 2,
                mark,
                sweep,
            );
        }
        // 박자 고리: 다음 박자까지 닫힌다(마지막 타 뒤엔 없음).
        if i < 2 {
            let r = t::BEAT_RING_START + (t::BEAT_RING_END - t::BEAT_RING_START) * k;
            draw_circle_lines(
                at.x,
                at.y,
                t::BEAT_RING_END,
                1.0,
                Color::new(1.0, 1.0, 1.0, 0.3),
            );
            draw_circle_lines(
                at.x,
                at.y,
                r,
                1.0,
                Color::new(1.0, 1.0, 1.0, 0.45 + 0.5 * k),
            );
        }
        if k < 0.2 && i > 0 {
            draw_circle_lines(
                at.x,
                at.y,
                t::BEAT_RING_END,
                2.5,
                Color::new(1.0, 1.0, 1.0, 1.0 - k / 0.2),
            );
        }
        // 맞춘 박자 수: 머리 위 음표.
        for n in 0..i {
            let px = at.x - 2.0 + n as f32 * 4.0;
            draw_rectangle(px - 1.0, at.y - 14.0, 2.0, 2.0, WHITE);
            draw_rectangle(px + 1.0, at.y - 17.0, 1.0, 4.0, WHITE);
        }
    } else {
        nail(art, ((time * 2.0) as u32) % 2, 0, at, false);
    }
}

fn pic_heavy(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        mark,
        ..
    } = c;
    // 누르고 있으면 발톱에 빛이 모이고, 떼면 강타 → 허수아비가 벽까지 밀려 쿵.
    let ph = time % 2.0;
    let at = vec2(mid - 18.0, y);
    let wall = w - 8.0;
    draw_rectangle(wall, y - 14.0, 4.0, 22.0, Color::new(0.36, 0.27, 0.22, 1.0));
    if ph < 1.0 {
        let k = (ph / 0.8).min(1.0);
        nail(art, 7, 1, at + vec2(0.0, 1.0), false);
        let claw = at + vec2(7.0, 0.0);
        let pulse = if k >= 1.0 {
            0.7 + 0.3 * (time * 20.0).sin()
        } else {
            k * 0.6
        };
        draw_circle(
            claw.x,
            claw.y,
            2.0 + 3.0 * k,
            Color::new(1.0, 0.95, 0.8, 0.35 * pulse),
        );
        draw_rectangle(claw.x, claw.y, 1.0, 1.0, Color::new(1.0, 1.0, 1.0, pulse));
        critter(art, Kind::Dummy, F::IDLE, at + vec2(20.0, 0.0), true, WHITE);
    } else {
        let k = ((ph - 1.0) / 0.25).min(1.0);
        nail(art, if k < 1.0 { 2 } else { 8 }, 1, at, false);
        if k < 1.0 {
            actors::draw_sector(
                at + vec2(2.0, 0.0),
                vec2(1.0, 0.0),
                18.0,
                Color::new(1.0, 1.0, 1.0, 0.25 * (1.0 - k)),
            );
            actors::draw_claw_marks(
                at + vec2(2.0, 0.0),
                vec2(1.0, 0.0),
                18.0,
                k,
                true,
                mark,
                1.0,
            );
        }
        let fly = ((ph - 1.05) / 0.25).clamp(0.0, 1.0);
        let dx = at.x + 20.0 + (wall - 6.0 - at.x - 20.0) * fly;
        critter(art, Kind::Dummy, F::HURT, vec2(dx, y), true, WHITE);
        if fly >= 1.0 && ph < 1.6 {
            let r = (ph - 1.3) * 40.0;
            draw_circle_lines(
                wall - 2.0,
                y,
                3.0 + r.max(0.0),
                1.0,
                Color::new(1.0, 1.0, 1.0, 0.8),
            );
        }
    }
}

fn pic_burrow(c: PicCtx) {
    let PicCtx {
        art, w, time, y, ..
    } = c;
    // 파고들어 흙더미로 다가가고, 누른 만큼 고리가 자라고, 떼면 솟구쳐 생쥐를 밀어낸다.
    let ph = time % 2.4;
    let mouse = vec2(w - 22.0, y);
    if ph < 0.3 {
        nail(art, 0, 0, vec2(16.0, y), false);
        critter(art, Kind::FieldMouse, F::IDLE, mouse, true, WHITE);
    } else if ph < 1.5 {
        let k = (ph - 0.3) / 1.2;
        let x = 16.0 + (mouse.x - 22.0) * k.min(0.6) / 0.6;
        nail(art, ((time * 14.0) as u32) % 2, 2, vec2(x, y), false);
        if k > 0.6 {
            let c = ((k - 0.6) / 0.4).min(1.0);
            draw_circle_lines(
                x,
                y + 2.0,
                6.0 + 10.0 * c,
                1.0,
                Color::new(0.75, 0.55, 0.38, 0.5 + 0.4 * c),
            );
        }
        critter(art, Kind::FieldMouse, F::IDLE, mouse, true, WHITE);
    } else {
        let k = ((ph - 1.5) / 0.35).min(1.0);
        let x = 16.0 + (mouse.x - 22.0);
        let pop = (4.0 * (k * std::f32::consts::PI).sin()).round();
        nail(art, 2, 1, vec2(x, y - pop), false);
        draw_circle_lines(
            x,
            y,
            6.0 + 12.0 * k,
            1.5,
            Color::new(1.0, 1.0, 1.0, 1.0 - k),
        );
        let push = (6.0 * k).min(6.0);
        critter(
            art,
            Kind::FieldMouse,
            F::HURT,
            mouse + vec2(push, 0.0),
            true,
            WHITE,
        );
        if ph > 1.7 {
            dizzy(mouse + vec2(push, 0.0), time);
        }
    }
}

fn pic_parry(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        y,
        gold,
        ..
    } = c;
    // 예고 띠가 가운데 줄에서 넓어지고, 다 차기 직전에 치면 튕겨 낸다(불똥 + 어질어질).
    let ph = time % 2.2;
    let at = vec2(18.0, y);
    let start = vec2(w - 18.0, y);
    if ph < 1.0 {
        let k = ph;
        fx::draw_tele_lane(start - vec2(6.0, 0.0), at + vec2(8.0, 0.0), 5.0, k);
        let shake = if ((time * 60.0) as i32) % 2 == 0 {
            1.0
        } else {
            -1.0
        };
        critter(
            art,
            Kind::FieldMouse,
            F::CROUCH,
            start + vec2(shake, 0.0),
            true,
            Color::new(1.0, 0.6, 0.6, 1.0),
        );
        nail(art, 0, 0, at, false);
    } else if ph < 1.15 {
        let k = (ph - 1.0) / 0.15;
        critter(
            art,
            Kind::FieldMouse,
            F::STRIKE,
            start + (at + vec2(12.0, 0.0) - start) * k,
            true,
            WHITE,
        );
        nail(art, 5, 1, at, false);
    } else {
        let k = ((ph - 1.15) / 0.3).min(1.0);
        nail(art, if k < 0.5 { 0 } else { 8 }, 1, at, false);
        let hit = at + vec2(10.0, 0.0);
        if k < 1.0 {
            for i in 0..6 {
                let a = i as f32 * 1.05;
                let q = hit + vec2(a.cos(), a.sin()) * (3.0 + 8.0 * k);
                draw_rectangle(q.x.round(), q.y.round(), 1.0, 1.0, gold);
            }
        }
        let back = at + vec2(12.0, 0.0) + vec2(20.0 * k, 0.0);
        critter(art, Kind::FieldMouse, F::HURT, back, true, WHITE);
        dizzy(back, time);
    }
}

fn pic_special(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 게이지가 차면 드릴이 되어 화면을 가로지른다. 생쥐 둘이 튕겨 나간다.
    let ph = time % 2.2;
    let gx = 6.0;
    draw_rectangle(gx - 1.0, 3.0, 32.0, 4.0, OUTLINE);
    let g = (ph / 0.8).min(1.0);
    draw_rectangle(gx, 4.0, 30.0 * g, 2.0, Color::new(1.0, 0.55, 0.25, 1.0));
    let m1 = vec2(mid, y);
    let m2 = vec2(mid + 22.0, y);
    if ph < 0.9 {
        nail(art, 0, 0, vec2(14.0, y), false);
        critter(art, Kind::FieldMouse, F::IDLE, m1, true, WHITE);
        critter(art, Kind::FieldMouse, F::IDLE, m2, true, WHITE);
    } else {
        let k = ((ph - 0.9) / 0.4).min(1.0);
        let x = 14.0 + (w - 28.0) * k;
        for i in 0..4 {
            let ly = y - 4.0 + i as f32 * 3.0;
            draw_line(
                x - 6.0 - i as f32 * 3.0,
                ly,
                x - 16.0 - i as f32 * 3.0,
                ly,
                1.0,
                Color::new(1.0, 1.0, 1.0, 0.6),
            );
        }
        nail(art, 2, 1, vec2(x, y), false);
        for m in [m1, m2] {
            if x > m.x - 4.0 {
                let up = ((x - m.x) * 0.3).min(8.0);
                critter(
                    art,
                    Kind::FieldMouse,
                    F::HURT,
                    m + vec2(4.0, -up),
                    true,
                    WHITE,
                );
            } else {
                critter(art, Kind::FieldMouse, F::IDLE, m, true, WHITE);
            }
        }
    }
}

fn pic_eat(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 산딸기를 세 입에 먹으면 하트가 하나 찬다. 지렁이는 다 채운다.
    let ph = time % 2.4;
    let at = vec2(mid - 14.0, y);
    nail(art, ((time * 2.0) as u32) % 2, 0, at, false);
    let eaten = ((ph - 0.3) / 0.9).clamp(0.0, 1.0);
    if eaten < 1.0 {
        let s = 1.0 - (eaten * 3.0).floor() / 3.0;
        if s > 0.0 {
            props::draw_berry(at.x + 10.0, at.y + 1.0, s.max(0.34), 1.0);
        }
    }
    let full = if eaten >= 1.0 { 3 } else { 2 };
    for i in 0..3 {
        heart(at.x - 10.0 + i as f32 * 8.0, at.y - 18.0, i < full);
    }
    if eaten >= 1.0 && ph < 1.6 {
        let k = (ph - 1.2) / 0.4;
        draw_circle_lines(
            at.x + 9.0,
            at.y - 16.0,
            2.0 + 6.0 * k,
            1.0,
            Color::new(1.0, 1.0, 1.0, 1.0 - k),
        );
    }
    // 오른쪽: 지렁이(들고 있다가 쓰러지면 저절로).
    let wx = w - 24.0;
    let wig = ((time * 4.0).sin() * 1.0).round();
    for i in 0..5 {
        draw_rectangle(
            wx + i as f32 * 3.0,
            y + 2.0 + if i % 2 == 0 { wig } else { -wig },
            3.0,
            2.0,
            Color::new(0.95, 0.62, 0.62, 1.0),
        );
    }
    for i in 0..3 {
        heart(wx - 2.0 + i as f32 * 8.0, y - 10.0, true);
    }
}

fn pic_enemy(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 왼쪽: 돌진할 길의 띠가 넓어져 테두리와 한 덩어리가 되는 순간 돌진(띠 = 받아칠 수 있다).
    // 오른쪽: 내려앉을 자리의 원이 진해져 테두리와 같아지는 순간(진홍 X = 피한다). 둘은 같은 박자로.
    let k = (time % 1.6 / 1.3).min(1.0);
    let m = vec2(12.0, y - 6.0);
    fx::draw_tele_lane(m + vec2(7.0, 0.0), m + vec2(mid - 14.0, 0.0), 4.0, k);
    critter(
        art,
        Kind::FieldMouse,
        F::CROUCH,
        m,
        false,
        Color::new(1.0, 0.6, 0.6, 1.0),
    );
    let at = vec2(w - 22.0, y);
    let r = 10.0;
    fx::draw_tele_disk(at, r, k);
    if k >= 1.0 {
        draw_circle_lines(at.x, at.y, r, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
    }
}

/// 안내 그림의 줍는 것(반딧불·씨앗·뿌리 심장)은 게임 시계의 이 배로 — 크게 키워 혼자 보이니 게임 빠르기(날갯짓·둥실·고리)가
/// 너무 바빠 보였다(사장 "안내의 반딧불과 씨앗 애니메이션 속도가 너무 빠름").
/// 0.4로도 "아직도 깜빡이는 게 빠르다"(사장 2026-10-29) — 반딧불 날개가 0.4초마다 켜졌다 꺼졌다. 0.2로 한 번 더.
const LOOT_PACE: f32 = 0.2;

fn pic_loot(c: PicCtx) {
    let PicCtx { w, time, y, .. } = c;
    // 반딧불(재주) · 씨앗(가게) · 뿌리 심장(최대 체력).
    let xs = [w * 0.2, w * 0.5, w * 0.8];
    for (i, k) in [
        loot::LootKind::Glowbug,
        loot::LootKind::Seed,
        loot::LootKind::RootHeart,
    ]
    .into_iter()
    .enumerate()
    {
        let l = loot::Loot::place(k, vec2(xs[i], y));
        props::draw_loot(&l, 1.0, time * LOOT_PACE, false);
    }
}

fn pic_camp(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 모닥불(저장) 옆에서 쉬고, 포포의 좌판에서 산다.
    props::draw_campfire(Rect::new(mid - 8.0, y - 4.0, 16.0, 16.0), time);
    props::draw_stall(Rect::new(w - 30.0, y - 4.0, 16.0, 16.0));
    nail(art, ((time * 2.0) as u32) % 2, 0, vec2(18.0, y), false);
}

fn pic_gear(c: PicCtx) {
    let PicCtx {
        art, w, time, y, ..
    } = c;
    // 상자에 다가가면 열리고 발톱이 나온다.
    let ph = time % 2.6;
    let chest = Rect::new(w - 34.0, y - 4.0, 16.0, 16.0);
    let open = ph > 1.0;
    let x = 14.0 + (chest.x - 24.0) * (ph / 1.0).min(1.0);
    props::draw_chest(art.and_then(|a| a.props.as_ref()), chest, open, time);
    nail(
        art,
        if ph < 1.0 {
            2 + ((time * 10.0) as u32) % 4
        } else {
            0
        },
        0,
        vec2(x, y),
        false,
    );
    if open {
        let rise = ((ph - 1.0) / 0.5).min(1.0) * 12.0;
        props::draw_pickup(
            &world::Pickup {
                pos: vec2(chest.center().x, chest.y - rise + 8.0),
                item: world::Gift::Weapon(1),
                age: 1.0,
            },
            time,
        );
    }
}

fn pic_break(c: PicCtx) {
    let PicCtx {
        art,
        time,
        mid,
        y,
        mark,
        ..
    } = c;
    // 흙더미를 치면 깨지고, 운이 좋으면 산딸기가 튀어나온다.
    let ph = time % 2.2;
    let at = vec2(mid - 14.0, y);
    let thing = Rect::new(mid, y - 4.0, 16.0, 16.0);
    let k = ((ph - 0.6) / 0.25).clamp(0.0, 1.0);
    nail(
        art,
        if ph > 0.6 && ph < 0.85 { 1 } else { 0 },
        if ph > 0.6 && ph < 0.85 { 1 } else { 0 },
        at,
        false,
    );
    if ph < 0.6 {
        props::draw_breakable(
            art.and_then(|a| a.props.as_ref()),
            crate::room::Breakable::Mound,
            thing,
            time,
        );
    } else {
        if k < 1.0 {
            actors::draw_claw_marks(
                at + vec2(2.0, 0.0),
                vec2(1.0, 0.0),
                14.0,
                k,
                false,
                mark,
                1.0,
            );
        }
        let c = thing.center();
        let d = ((ph - 0.6) / 0.4).min(1.0);
        for i in 0..5 {
            let a = -0.4 - i as f32 * 0.55;
            let q = c + vec2(a.cos(), a.sin()) * 10.0 * d + vec2(0.0, 8.0 * d * d);
            if d < 1.0 {
                draw_rectangle(
                    q.x.round(),
                    q.y.round(),
                    2.0,
                    2.0,
                    Color::new(0.55, 0.40, 0.28, 1.0),
                );
            }
        }
        let b = ((ph - 0.7) / 0.4).clamp(0.0, 1.0);
        let hop = (10.0 * (b * std::f32::consts::PI).sin()).round();
        props::draw_berry(c.x + 6.0 * b, c.y + 2.0 - hop, 1.0, 1.0);
    }
}

fn pic_doto(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 셋을 던진다(셋째는 부채꼴) → 생쥐가 달려들면 뒤로 뛰어 물러난다 → 묻어 둔 도토리가 생쥐 밑에서 터진다.
    let ph = time % 3.0;
    let me = vec2(14.0, y);
    let foe_home = vec2(w - 16.0, y);
    if ph < 1.2 {
        let k = (ph % 0.4) / 0.4;
        let n = (ph / 0.4) as usize;
        doto(art, if k < 0.3 { 5 } else { 0 }, me, 0.0, false);
        let target = foe_home;
        let fan: &[f32] = if n == 2 { &[-5.0, 0.0, 5.0] } else { &[0.0] };
        for dy in fan {
            let p = me.lerp(target + vec2(0.0, *dy), k) + vec2(6.0 * (1.0 - k), -4.0);
            if k < 0.9 {
                acorn(p);
            }
        }
        let hit = k > 0.85;
        critter(
            art,
            Kind::FieldMouse,
            if hit { F::HURT } else { F::IDLE },
            foe_home,
            true,
            WHITE,
        );
    } else if ph < 2.0 {
        // 달려든다 — 뒤로 뛴다(겨눈 쪽은 그대로, 몸은 왼쪽으로).
        let k = (ph - 1.2) / 0.8;
        let foe = foe_home.lerp(vec2(mid, y), k.min(1.0));
        critter(
            art,
            Kind::FieldMouse,
            F::WALK + ((time * 10.0) as u32) % 2,
            foe,
            true,
            WHITE,
        );
        let back = (k * 1.6).min(1.0);
        let lift = (back * std::f32::consts::PI).sin() * 7.0;
        doto(
            art,
            if back < 1.0 { 5 } else { 0 },
            me + vec2(-6.0 * back, 0.0),
            lift,
            false,
        );
        // 뛰기 전에 발밑에 묻어 둔 셋.
        for i in 0..3 {
            let b = vec2(mid - 10.0 + i as f32 * 10.0, y + 4.0);
            draw_rectangle(b.x - 2.0, b.y, 4.0, 1.0, Color::new(0.45, 0.30, 0.18, 1.0));
        }
    } else {
        // 터진다: 묻은 자리에서 고리, 생쥐가 튕긴다.
        let k = ((ph - 2.0) / 0.5).min(1.0);
        doto(art, 0, me + vec2(-6.0, 0.0), 0.0, false);
        for i in 0..3 {
            let b = vec2(mid - 10.0 + i as f32 * 10.0, y + 3.0);
            draw_circle_lines(
                b.x,
                b.y,
                2.0 + 7.0 * k,
                1.0,
                Color::new(1.0, 0.75, 0.35, 1.0 - k),
            );
        }
        critter(
            art,
            Kind::FieldMouse,
            F::HURT,
            vec2(mid + 4.0 * k, y - 5.0 * k),
            true,
            WHITE,
        );
    }
}

fn pic_run(c: PicCtx) {
    let PicCtx {
        art,
        w,
        time,
        mid,
        y,
        ..
    } = c;
    // 모모가 깡충 → 쥔 채 달리고(앞 바닥에 내려앉을 자리, 0211) → 떼면 그 자리로 멀리 뛰어 생쥐 곁을 밟는다(0209).
    let ph = time % 2.8;
    let start = 12.0;
    let run_end = mid - 6.0;
    let land = w - 36.0;
    let mouse = vec2(w - 18.0, y);
    let arc = |k: f32, h: f32| (k * std::f32::consts::PI).sin() * h;
    if ph < 0.35 {
        let k = ph / 0.35;
        momo(art, 3, vec2(start + 12.0 * k, y), arc(k, 7.0), false);
        critter(art, Kind::FieldMouse, F::IDLE, mouse, true, WHITE);
    } else if ph < 1.25 {
        let k = (ph - 0.35) / 0.9;
        let x = start + 12.0 + (run_end - start - 12.0) * k;
        // 달리는 동안 내려앉을 자리(0211): 늘 같은 거리 앞, 밟기 둘레.
        let ahead = land - run_end;
        draw_circle_lines(
            (x + ahead).round(),
            y + 6.0,
            10.0,
            1.0,
            Color::new(1.0, 0.78, 0.86, if k > 0.25 { 0.9 } else { 0.35 }),
        );
        momo(art, 2 + ((time * 16.0) as u32) % 2, vec2(x, y), 0.0, false);
        critter(art, Kind::FieldMouse, F::IDLE, mouse, true, WHITE);
    } else if ph < 1.75 {
        let k = (ph - 1.25) / 0.5;
        let x = run_end + (land - run_end) * k;
        momo(art, 3, vec2(x, y), arc(k, 12.0), false);
        critter(art, Kind::FieldMouse, F::IDLE, mouse, true, WHITE);
    } else {
        let k = ((ph - 1.75) / 0.4).min(1.0);
        momo(art, 0, vec2(land, y), 0.0, false);
        draw_circle_lines(
            land,
            y + 3.0,
            4.0 + 14.0 * k,
            1.5,
            Color::new(1.0, 0.85, 0.9, 1.0 - k),
        );
        critter(
            art,
            Kind::FieldMouse,
            F::HURT,
            mouse + vec2(6.0 * k, 0.0),
            true,
            WHITE,
        );
    }
}

fn pic_village(c: PicCtx) {
    let PicCtx {
        art, w, time, y, ..
    } = c;
    // 반딧불이 큰 나무로 올라가 재주를 깨운다.
    let tx = w - 28.0;
    let trunk = Color::new(0.40, 0.27, 0.21, 1.0);
    draw_rectangle(tx - 3.0, y - 14.0, 6.0, 20.0, trunk);
    draw_circle(tx, y - 18.0, 10.0, Color::new(0.30, 0.55, 0.30, 1.0));
    draw_circle(tx - 6.0, y - 14.0, 7.0, Color::new(0.36, 0.62, 0.34, 1.0));
    draw_circle(tx + 6.0, y - 13.0, 7.0, Color::new(0.33, 0.58, 0.32, 1.0));
    nail(art, ((time * 2.0) as u32) % 2, 0, vec2(18.0, y), false);
    for i in 0..4 {
        let k = (time * 0.35 + i as f32 * 0.25).fract(); // 큰 나무로 날아가는 빠르기(줍는 것 박자 `LOOT_PACE`와 함께 늦춤)
        let p = vec2(24.0, y - 4.0)
            + (vec2(tx, y - 18.0) - vec2(24.0, y - 4.0)) * k
            + vec2(0.0, -(k * std::f32::consts::PI).sin() * 8.0);
        let l = loot::Loot::place(loot::LootKind::Glowbug, p);
        props::draw_loot(&l, 1.0, time * LOOT_PACE, false);
    }
}
