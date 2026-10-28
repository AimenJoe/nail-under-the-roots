//! 배우: Naïl·모모(플레이어), 적, 이웃(보리·할매), 발톱 자국·먹기·지렁이·굴 둔덕·혀·투사체·도토리, 부채꼴.

use super::*;

/// 컷씬 배우(월드 패스 안에서, begin_world 뒤 present 전에).
pub fn draw_actor(pos: Vec2, facing: Vec2, color: Color) {
    let p = pos.round();
    draw_ellipse(p.x, p.y + 6.0, 7.0, 3.0, 0.0, SHADOW);
    draw_rectangle(p.x - 6.0, p.y - 6.0, 12.0, 12.0, color);
    let nose = p + facing * 8.0;
    draw_circle(nose.x.round(), nose.y.round(), 2.0, WHITE);
}

pub(super) fn draw_player(p: &Player, alpha: f32, time: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let body = if p.flash > 0.0 { WHITE } else { world::MOLE };
    match p.state {
        player::State::Dead => {
            draw_rectangle(pos.x - 8.0, pos.y - 2.0, 16.0, 5.0, GRAY);
            return;
        }
        player::State::Burrow { time: bt } => {
            // 그림자(흙더미)만. 작게 꿈틀. 힘을 모으면(0158) 더 빨리·크게 들썩이고 더미가 부푼다.
            let charge = p.burrow_charge();
            let wob = ((bt * (40.0 + 40.0 * charge)).sin() * (1.0 + charge)).round();
            let grow = 2.0 * charge;
            draw_ellipse(
                pos.x,
                pos.y + 2.0,
                8.0 + wob + grow,
                4.0 + grow * 0.5,
                0.0,
                SHADOW,
            );
            draw_ellipse(pos.x, pos.y, 6.0 + grow, 3.0 + grow * 0.5, 0.0, world::DIRT);
            return;
        }
        _ => {}
    }
    // 무적 중 깜빡임.
    if p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && p.flash <= 0.0 {
        draw_rectangle_lines(pos.x - 6.0, pos.y - 6.0, 12.0, 12.0, 1.0, world::MOLE);
    } else {
        draw_rectangle(pos.x - 6.0, pos.y - 6.0, 12.0, 12.0, body);
    }
    let nose = pos + p.facing * 8.0;
    draw_circle(nose.x.round(), nose.y.round(), 2.0, WHITE);
}

/// 공격 이펙트(발톱 자국 세 줄)와 솟구침 고리. 무브셋 수치를 따른다.
pub(super) fn draw_player_fx(p: &Player, alpha: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    if let Some((item, k)) = p.eat_progress() {
        draw_eating(pos, p.face_left, item, k);
    }
    match p.state {
        player::State::Attack { idx, time: at, dir } => {
            let hit = p.moveset.hits[idx];
            let (a0, a1) = hit.active;
            let origin = pos + dir * 2.0;
            let finisher = idx == p.moveset.last();
            if at >= a0 && at <= a1 + 0.04 {
                let k = ((at - a0) / (a1 - a0 + 0.04)).clamp(0.0, 1.0);
                if finisher {
                    draw_sector(
                        origin,
                        dir,
                        hit.range,
                        Color::new(1.0, 1.0, 1.0, 0.25 * (1.0 - k)),
                    );
                }
                // 자국이 지나가는 쪽 = 친 발(0082): 짝수 타는 위에서 아래로, 홀수 타는 아래에서 위로, 마지막 타는 둘 다(X).
                let mark = p.moveset.mark;
                if finisher {
                    draw_claw_marks(origin, dir, hit.range, k, true, mark, 1.0);
                    draw_claw_marks(origin, dir, hit.range, k, true, mark, -1.0);
                } else {
                    let sweep = if idx % 2 == 0 { 1.0 } else { -1.0 };
                    draw_claw_marks(origin, dir, hit.range, k, false, mark, sweep);
                }
            } else if at < a0 {
                draw_sector_lines(
                    origin,
                    dir,
                    hit.range * 0.5,
                    Color::new(0.6, 0.45, 0.3, 0.6),
                );
            }
        }
        // 강공격 모으기(0161): 발톱 끝에 빛이 모이고, 다 모이면 밝게 맥박친다.
        player::State::Charging { time: ct, dir, .. } => {
            let k = (ct / t::HEAVY_CHARGE).clamp(0.0, 1.0);
            let claw = pos + dir * 7.0;
            let pulse = if k >= 1.0 {
                0.7 + 0.3 * (ct * 20.0).sin()
            } else {
                k * 0.6
            };
            draw_circle(
                claw.x.round(),
                claw.y.round(),
                2.0 + 3.0 * k,
                Color::new(1.0, 0.95, 0.8, 0.35 * pulse),
            );
            draw_rectangle(
                claw.x.round(),
                claw.y.round(),
                1.0,
                1.0,
                Color::new(1.0, 1.0, 1.0, pulse),
            );
            if k >= 1.0 {
                draw_sector_lines(
                    pos + dir * 2.0,
                    dir,
                    p.moveset.heavy().range * 0.5,
                    Color::new(1.0, 0.9, 0.6, 0.5),
                );
            }
        }
        player::State::Heavy { time: at, dir, .. } => {
            let hit = p.moveset.heavy();
            let (a0, a1) = hit.active;
            let origin = pos + dir * 2.0;
            if at >= a0 && at <= a1 + 0.04 {
                let k = ((at - a0) / (a1 - a0 + 0.04)).clamp(0.0, 1.0);
                draw_sector(
                    origin,
                    dir,
                    hit.range,
                    Color::new(1.0, 0.95, 0.8, 0.35 * (1.0 - k)),
                );
                let mark = p.moveset.mark;
                draw_claw_marks(origin, dir, hit.range, k, true, mark, 1.0);
                draw_claw_marks(origin, dir, hit.range, k, true, mark, -1.0);
                draw_sector_lines(
                    origin,
                    dir,
                    hit.range,
                    Color::new(1.0, 0.9, 0.6, 0.8 * (1.0 - k)),
                );
            } else if at < a0 {
                draw_sector_lines(
                    origin,
                    dir,
                    hit.range * 0.6,
                    Color::new(1.0, 0.85, 0.5, 0.7),
                );
            }
        }
        player::State::Drill { time: dt, dir, .. } if dt < t::DRILL_REV => {
            // 시동: 몸이 떨리고, 앞에 작은 드릴이 점점 커지며 돈다. 주변에 기운 고리가 조여 든다.
            let k = dt / t::DRILL_REV;
            let side = vec2(-dir.y, dir.x);
            let base = pos + dir * 3.0;
            let tip = pos + dir * (6.0 + 9.0 * k);
            let claw = Color::new(0.97, 0.93, 0.85, 1.0);
            draw_triangle(base + side * 5.0, base - side * 5.0, tip, OUTLINE);
            draw_triangle(base + side * 4.0, base - side * 4.0, tip - dir, claw);
            let rage = world::RAGE;
            let spin = dt * 40.0;
            for i in 0..6 {
                let a = spin + i as f32 * std::f32::consts::TAU / 6.0;
                let r = 18.0 * (1.0 - k) + 8.0;
                let q = pos + Vec2::from_angle(a) * r;
                let q2 = pos + Vec2::from_angle(a - 0.5) * (r + 4.0);
                draw_line(
                    q.x,
                    q.y,
                    q2.x,
                    q2.y,
                    1.0,
                    Color::new(rage.r, rage.g, rage.b, 0.5 + 0.5 * k),
                );
            }
            // 겨누기(0139): 나갈 쪽으로 점선, 겨눈 적에는 모서리 표시. 8방향 입력 사이의 적도 맞춘다는 것을 보여 준다.
            let guide = Color::new(rage.r, rage.g, rage.b, 0.35 + 0.45 * k);
            let len = p
                .drill_lock
                .map_or(72.0, |q| ((q - pos).length() - 8.0).clamp(14.0, 240.0));
            let mut d = 14.0;
            while d < len {
                let q = (pos + dir * d).round();
                draw_rectangle(q.x, q.y, 1.0, 1.0, guide);
                d += 6.0;
            }
            if let Some(q) = p.drill_lock {
                let q = q.round();
                let r = 9.0 - 2.0 * k;
                for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let c = vec2(q.x + sx * r, q.y + sy * r);
                    // 모서리 점에서 안쪽으로 3px씩(그 점 포함 4px).
                    let x0 = if sx < 0.0 { c.x } else { c.x - 3.0 };
                    let y0 = if sy < 0.0 { c.y } else { c.y - 3.0 };
                    draw_rectangle(x0, c.y, 4.0, 1.0, guide);
                    draw_rectangle(c.x, y0, 1.0, 4.0, guide);
                }
            }
        }
        player::State::Drill { time: dt, dir, .. } => {
            // 앞쪽 원뿔 드릴: 줄무늬가 돌아간다.
            let dt = dt - t::DRILL_REV;
            let side = vec2(-dir.y, dir.x);
            let base = pos + dir * 3.0;
            let tip = pos + dir * 17.0;
            let claw = Color::new(0.97, 0.93, 0.85, 1.0);
            let shade = Color::new(0.62, 0.55, 0.48, 1.0);
            draw_triangle(base + side * 7.0, base - side * 7.0, tip, OUTLINE);
            draw_triangle(base + side * 6.0, base - side * 6.0, tip - dir, claw);
            for k in 0..3 {
                let f = ((dt * 22.0 + k as f32 / 3.0).fract() * 0.9) + 0.05;
                let a = base + (tip - base) * f;
                let w = 5.0 * (1.0 - f);
                draw_line(
                    a.x + side.x * w,
                    a.y + side.y * w,
                    a.x - side.x * w * 0.3,
                    a.y - side.y * w * 0.3,
                    1.0,
                    shade,
                );
            }
            // 바람 줄: 몸 양옆으로 길게, 깜빡이며.
            for k in 0..4 {
                let off = [8.0, -8.0, 12.0, -12.0][k];
                let len = 10.0 + 8.0 * (((dt * 30.0) as usize + k) % 3) as f32;
                let a = pos + side * off - dir * 2.0;
                let b = a - dir * len;
                draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
            // 화면을 가로지르는 속도선(카메라 둘레에 흩뿌린 고정 무늬가 반대로 흐른다).
            for i in 0..10 {
                let h = (i as f32 * 37.7).sin() * 0.5 + 0.5;
                let lane = (h - 0.5) * 200.0;
                let flow = ((dt * 3.0 + i as f32 * 0.137).fract() - 0.5) * 360.0;
                let a = pos + side * lane - dir * flow;
                let b = a - dir * 24.0;
                draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(1.0, 1.0, 1.0, 0.25));
            }
        }
        player::State::Burrow { time: bt } if p.burrow_charge() > 0.0 => {
            // 힘 모으기(0158): 솟구칠 반지름까지 자라는 흙빛 고리가 맥박친다. 다 모이면 밝게.
            let charge = p.burrow_charge();
            let r = p.emerge_radius_for(charge);
            let pulse = 0.5 + 0.5 * (bt * 18.0).sin();
            let a = 0.25 + 0.35 * charge + 0.15 * pulse;
            let c = if charge >= 1.0 {
                Color::new(1.0, 0.95, 0.8, a)
            } else {
                Color::new(world::DIRT.r, world::DIRT.g, world::DIRT.b, a)
            };
            draw_circle_lines(pos.x, pos.y, r, 1.0, c);
        }
        player::State::Emerge { time: et, charge } => {
            let p01 = et / t::EMERGE_TIME;
            let r = 6.0 + (p.emerge_radius_for(charge) - 6.0) * p01;
            draw_circle_lines(
                pos.x,
                pos.y,
                r,
                1.5 + charge,
                Color::new(1.0, 1.0, 1.0, 1.0 - p01),
            );
        }
        _ => {}
    }
    draw_beat_cue(p, pos);
}

/// 박자 고리와 음표(0174): 스윙이 시작되면 큰 고리가 과녁 고리로 닫히고, 닿는 순간이 박자. 창 안이면 둘 다 밝게,
/// 창 전에 눌러 기다리는 중이면 호박색. 박자가 지나면 흐려진다. 맞춘 박자 수는 머리 위 음표.
fn draw_beat_cue(p: &Player, pos: Vec2) {
    let (x, y) = (pos.x.round(), pos.y.round());
    if let Some(c) = p.beat_cue() {
        let k = c.k.min(1.0);
        let r = t::BEAT_RING_START + (t::BEAT_RING_END - t::BEAT_RING_START) * k;
        // 박자가 지난 뒤는 창 하나만큼에 걸쳐 사라진다.
        let fade = if c.k > 1.0 {
            (1.0 - (c.k - 1.0) * t::BEAT_PERIOD / (t::BEAT_WINDOW * 2.0)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let (col, w) = if c.window {
            (Color::new(1.0, 1.0, 1.0, 0.95 * fade), 1.5)
        } else if c.waiting {
            (Color::new(1.0, 0.8, 0.35, 0.8 * fade), 1.0)
        } else {
            (Color::new(1.0, 1.0, 1.0, 0.4 * fade), 1.0)
        };
        draw_circle_lines(
            x,
            y,
            t::BEAT_RING_END,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.3 * fade),
        );
        draw_circle_lines(x, y, r, w, col);
        // 박자 순간(0180): 과녁 고리가 한 번 굵고 하얗게 번쩍인다(박자 앞뒤 반 창 동안). "지금"이 모양으로도 보이게.
        let d = (c.k - 1.0).abs() * p.beat_period();
        if d <= t::BEAT_WINDOW * 0.5 {
            let a = 1.0 - d / (t::BEAT_WINDOW * 0.5);
            draw_circle_lines(x, y, t::BEAT_RING_END, 2.5, Color::new(1.0, 1.0, 1.0, a));
        }
    }
    let (n, show) = p.beat_pips();
    if n > 0 && show > 0.0 {
        let a = (show / 0.3).min(1.0);
        let n = n.min(6);
        let x0 = x - 2.0 * (n as f32 - 1.0);
        for i in 0..n {
            let px = x0 + 4.0 * i as f32;
            let py = y - 15.0;
            // 작은 음표: 머리(2×2)와 꼬리(1×3).
            draw_rectangle(px - 1.0, py + 1.0, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, a));
            draw_rectangle(px + 1.0, py - 2.0, 1.0, 4.0, Color::new(1.0, 1.0, 1.0, a));
        }
    }
}

/// 드릴 돌진 잔상: 지나온 길에 분노색 그림자 셋. 그림이 있으면 Naïl 실루엣, 없으면 원.
pub(super) fn draw_drill_trail(art: Option<&Art>, p: &Player, alpha: f32) {
    let player::State::Drill { time, dir, .. } = p.state else {
        return;
    };
    if time < t::DRILL_REV {
        return;
    }
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let rage = world::RAGE;
    for i in 1..4 {
        let q = pos - dir * (i as f32 * 9.0);
        let a = 0.45 - i as f32 * 0.12;
        let c = Color::new(rage.r, rage.g, rage.b, a);
        match art {
            Some(art) => draw_texture_ex(
                &art.nail,
                (q.x - 8.0).round(),
                (q.y - 8.0).round(),
                c,
                DrawTextureParams {
                    source: Some(art::cell(2, 4)),
                    dest_size: Some(vec2(16.0, 16.0)),
                    flip_x: p.face_left,
                    ..Default::default()
                },
            ),
            None => draw_circle(q.x, q.y, t::PLAYER_RADIUS, c),
        }
    }
}

/// 발톱 자국: 전방 호를 따라 세 줄. 휘두르는 동안 펼쳐지고 끝에서 옅어진다.
/// `sweep` 1 = 호의 한쪽 끝에서, −1 = 반대쪽 끝에서 펼친다(번갈아 치는 발).
pub(super) fn draw_claw_marks(
    origin: Vec2,
    dir: Vec2,
    r: f32,
    k: f32,
    strong: bool,
    mark: Color,
    sweep: f32,
) {
    let base = dir.y.atan2(dir.x);
    let span = std::f32::consts::FRAC_PI_2 * 0.95;
    let reveal = (k * 1.6).min(1.0);
    let fade = 1.0 - ((k - 0.6) / 0.4).clamp(0.0, 1.0);
    let col = if strong {
        Color::new(
            (mark.r + 1.0) * 0.5,
            (mark.g + 1.0) * 0.5,
            (mark.b + 1.0) * 0.5,
            fade,
        )
    } else {
        Color::new(mark.r, mark.g, mark.b, 0.9 * fade)
    };
    let thick = if strong { 2.0 } else { 1.0 };
    let steps = 10;
    for line in 0..3 {
        let rr = r - line as f32 * 3.0;
        let a_start = base - span * sweep;
        let a_end = a_start + 2.0 * span * reveal * sweep;
        let mut prev = origin + vec2(a_start.cos(), a_start.sin()) * rr;
        for i in 1..=steps {
            let a = a_start + (a_end - a_start) * i as f32 / steps as f32;
            let q = origin + vec2(a.cos(), a.sin()) * rr;
            draw_line(prev.x, prev.y, q.x, q.y, thick, col);
            prev = q;
        }
    }
}

pub(super) fn draw_projectile(p: &enemy::Projectile, alpha: f32) {
    let pos = lerp(p.prev_pos, p.pos, alpha);
    // 되돌린 투사체는 하얗게 + 꼬리.
    let c = if p.friendly { WHITE } else { world::PROJECTILE };
    if p.friendly {
        let tail = pos - p.vel.normalize_or_zero() * 5.0;
        draw_line(
            pos.x,
            pos.y,
            tail.x,
            tail.y,
            2.0,
            Color::new(1.0, 1.0, 1.0, 0.5),
        );
    }
    if !p.friendly {
        // 다람쥐의 도토리. 날아가며 돈다.
        draw_acorn(pos, (pos.x + pos.y) * 0.25);
        return;
    }
    draw_circle(pos.x, pos.y, p.radius(), c);
    draw_circle_lines(pos.x, pos.y, p.radius() + 0.5, 1.0, world::PROJECTILE);
}

/// 도토가 던진 도토리(0195): 적의 도토리와 같은 그림, 큰 도토리는 두 배 크기. 날아가며 돈다.
pub(super) fn draw_shot(s: &world::toss::Shot, alpha: f32) {
    let pos = lerp(s.prev, s.pos, alpha);
    let spin = (pos.x + pos.y) * 0.25;
    if s.splash > 0.0 {
        let p = pos.round();
        draw_circle(p.x, p.y, 4.5, OUTLINE);
        draw_circle(p.x, p.y + 0.5, 3.8, Color::new(0.82, 0.56, 0.30, 1.0));
        let up = Vec2::from_angle(spin - std::f32::consts::FRAC_PI_2);
        let cap = (p + up * 2.5).round();
        draw_rectangle(
            cap.x - 3.0,
            cap.y - 1.5,
            6.0,
            3.0,
            Color::new(0.42, 0.28, 0.18, 1.0),
        );
    } else {
        draw_acorn(pos, spin);
    }
}

// ---- 그림 에셋 경로 -----------------------------------------------------------

/// 돌아온 보리(chief.png 줄 1, 32×32). 발이 `pos.y + 6` 근처.
pub fn draw_bori_at(art: &Art, pos: Vec2, face_left: bool, moving: bool, time: f32) {
    let Some(tex) = art.chief.as_ref() else {
        draw_circle(pos.x, pos.y, 10.0, Color::new(0.62, 0.45, 0.36, 1.0));
        return;
    };
    let frame = if moving {
        2 + ((time * 6.0) as u32) % 2
    } else {
        ((time * 1.5) as u32) % 2
    };
    draw_texture_ex(
        tex,
        (pos.x - 16.0).round(),
        (pos.y - 24.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(art::chief_cell(frame, art::CHIEF_BORI)),
            dest_size: Some(vec2(32.0, 32.0)),
            flip_x: face_left,
            ..Default::default()
        },
    );
}

/// 빈 자리: 통나무 의자 + 시든 꽃잎 하나(모모가 돌아올 곳).
/// 모모가 숨은 흙 둔덕(0046): 따라오는 동안은 흙이 튀고, 멈춘 뒤엔 떨며 부풀어 오른다(튀어나올 예고).
pub(super) fn draw_dig_mound(pos: Vec2, ut: f32, windup: f32, time: f32) {
    let hold = ((ut - t::MOMO_DIG_TRACK) / windup.max(0.01)).clamp(0.0, 1.0);
    let shake = if hold > 0.0 && ((time * 50.0) as i32) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    let (x, y) = ((pos.x + shake).round(), pos.y.round() + 4.0);
    let w = 6.0 + hold * 3.0;
    draw_ellipse(x, y + 1.0, w + 1.0, 3.5, 0.0, OUTLINE);
    draw_ellipse(x, y, w, 3.0 + hold, 0.0, world::DIRT);
    draw_ellipse(
        x - 1.0,
        y - 1.0,
        w * 0.5,
        1.2,
        0.0,
        Color::new(0.70, 0.54, 0.38, 1.0),
    );
    // 흙 부스러기
    for k in 0..3 {
        let ph = (time * 3.0 + k as f32 * 0.33).fract();
        let dx = (k as f32 - 1.0) * 4.0;
        let c = Color::new(0.55, 0.40, 0.28, 1.0 - ph);
        draw_rectangle(x + dx, y - 3.0 - ph * 5.0, 1.0, 1.0, c);
    }
    // 떨 때: 착지 고리처럼 둘레를 보여 준다.
    if hold > 0.0 {
        draw_circle_lines(
            x,
            y,
            8.0 + hold * 12.0,
            1.0,
            Color::new(1.0, 0.5, 0.5, 0.35 + 0.4 * hold),
        );
    }
}

/// 누룩 할매(마을, 0045): 흰 실 덩어리 + 버섯 갓 셋 + 감은 눈. 숨 쉬듯 살짝 오르내린다.
pub(super) fn draw_halmae_at(pos: Vec2, time: f32) {
    let bob = ((time * 1.6).sin() * 1.0).round();
    let (x, y) = (pos.x.round(), pos.y.round() + 2.0 - bob);
    let pale = Color::new(0.86, 0.84, 0.93, 1.0);
    let shade = Color::new(0.66, 0.62, 0.78, 1.0);
    let cap = Color::new(0.78, 0.60, 0.38, 1.0);
    let cap_dark = Color::new(0.52, 0.38, 0.26, 1.0);
    // 바닥에 퍼진 실(바닥에 깔려 그림자 패스에서는 뺀다)
    for k in (0..5).filter(|_| !shadow_pass()) {
        let a = k as f32 * 1.2 + 0.3;
        let (dx, dy) = (a.cos() * 11.0, a.sin().abs() * 3.0 + 4.0);
        draw_line(
            x,
            y + 3.0,
            x + dx,
            y + dy,
            1.0,
            Color::new(0.90, 0.88, 0.96, 0.7),
        );
    }
    draw_ellipse(x, y, 9.0, 7.0, 0.0, OUTLINE);
    draw_ellipse(x, y, 8.0, 6.0, 0.0, shade);
    draw_ellipse(x - 1.0, y - 1.0, 7.0, 5.0, 0.0, pale);
    for (cx, cy, r) in [(-4.0, -6.0, 3.0), (1.0, -8.0, 4.0), (5.0, -5.0, 2.5)] {
        draw_rectangle(x + cx - 0.5, y + cy, 1.0, 3.0, pale);
        draw_ellipse(x + cx, y + cy, r + 0.8, r * 0.6 + 0.8, 0.0, OUTLINE);
        draw_ellipse(x + cx, y + cy, r, r * 0.6, 0.0, cap);
        draw_rectangle(x + cx - r * 0.4, y + cy - r * 0.3, 1.0, 1.0, cap_dark);
    }
    // 감은 눈 + 홍조
    for ex in [-3.0, 2.0] {
        draw_rectangle(x + ex, y - 1.0, 2.0, 1.0, Color::new(0.35, 0.28, 0.45, 1.0));
    }
    draw_rectangle(
        x - 5.0,
        y + 1.0,
        2.0,
        1.0,
        Color::new(0.93, 0.62, 0.70, 1.0),
    );
    draw_rectangle(
        x + 4.0,
        y + 1.0,
        2.0,
        1.0,
        Color::new(0.93, 0.62, 0.70, 1.0),
    );
}

/// 먹는 중: 입 앞에 든 산딸기(베어 물수록 작아진다)·지렁이 + 머리 위 작은 진행 막대.
/// 회복은 다 먹어야 된다는 걸 보여 준다(0038). 맞으면 상태가 바뀌어 사라진다.
pub(super) fn draw_eating(pos: Vec2, face_left: bool, item: crate::items::Consumable, k: f32) {
    let side = if face_left { -1.0 } else { 1.0 };
    let bob = if ((k * 8.0) as i32) % 2 == 0 {
        0.0
    } else {
        1.0
    };
    let m = vec2(pos.x + side * 6.0, pos.y - 1.0 + bob);
    match item {
        crate::items::Consumable::Berry => {
            // 세 입: 3 → 2 → 1칸 너비.
            let wpx = 3.0 - (k * 3.0).floor().min(2.0);
            // 입 쪽부터 베어 문다.
            let x = if face_left {
                m.x - 2.0
            } else {
                m.x - 1.0 + (3.0 - wpx)
            };
            draw_rectangle(x - 1.0, m.y - 2.0, wpx + 2.0, 5.0, OUTLINE);
            draw_rectangle(x, m.y - 1.0, wpx, 3.0, world::BERRY);
        }
        crate::items::Consumable::Worm => {
            draw_worm(m + vec2(side * 1.0, 0.0), k * 20.0, 0.6);
        }
    }
    let (bw, y) = (10.0, pos.y - 14.0);
    draw_rectangle(pos.x - bw / 2.0 - 1.0, y - 1.0, bw + 2.0, 3.0, OUTLINE);
    draw_rectangle(pos.x - bw / 2.0, y, (bw * k).round(), 1.0, world::BERRY);
}

/// 쥬시 지렁이: 꿈틀대는 분홍 마디 + 반들거리는 하이라이트. `k`는 크기 배율(1 = 월드, HUD는 더 크게).
pub fn draw_worm(p: Vec2, ph: f32, k: f32) {
    let body = Color::new(0.95, 0.55, 0.62, 1.0);
    let dark = Color::new(0.70, 0.32, 0.42, 1.0);
    for i in 0..5 {
        let x = p.x + (i as f32 - 2.0) * 1.6 * k;
        let y = p.y + ((ph + i as f32 * 1.2).sin() * 1.0 * k).round();
        draw_rectangle(x - 1.0 * k, y - 1.0 * k, 2.4 * k, 3.0 * k, OUTLINE);
    }
    for i in 0..5 {
        let x = p.x + (i as f32 - 2.0) * 1.6 * k;
        let y = p.y + ((ph + i as f32 * 1.2).sin() * 1.0 * k).round();
        draw_rectangle(
            x - 0.5 * k,
            y - 0.5 * k,
            1.6 * k,
            2.0 * k,
            if i == 4 { dark } else { body },
        );
    }
    // 반들반들(쥬시)
    let y = p.y + (ph.sin() * 1.0 * k).round();
    draw_rectangle(
        p.x - 3.0 * k,
        y - 0.5 * k,
        0.8 * k,
        0.8 * k,
        Color::new(1.0, 0.95, 0.95, 1.0),
    );
}

pub(super) const OUTLINE: Color = Color::new(0.17, 0.11, 0.16, 1.0);

/// Naïl 스프라이트. 상태 → 칸, 좌우 반전, 피격 플래시는 흰 실루엣 줄.
/// 모모·도토를 플레이어로(0056·0195): 돌아온 모습 줄(critters `MOMO_BLOOM_ROW`·`DOTO_BACK_ROW`, 임시 그림 — 0191 C).
/// 칸 0·1 서기, 2·3 걷기, 4 웅크림, 5 차기·뛰기·던지기, 6 맞음.
/// 뛸 때는 그림만 높이만큼 위로(그림자는 바닥에 남아 착지 자리를 보인다).
pub(super) fn draw_kin_player(art: &Art, p: &Player, row: u32, alpha: f32, time: f32) {
    let Some(tex) = art.critters.as_ref() else {
        return;
    };
    let pos = lerp(p.prev_pos, p.pos, alpha) - vec2(0.0, p.height().round());
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    let frame = match p.state {
        player::State::Dead => 6,
        player::State::Hop { time: ht, .. }
        | player::State::Leap { time: ht, .. }
        | player::State::Chain { time: ht, .. } => {
            if ht < 0.05 {
                4
            } else {
                5
            }
        }
        player::State::Attack { .. } | player::State::Heavy { .. } => 5,
        player::State::Charging { .. } => 5,
        player::State::Recover { .. } => 4,
        _ if p.invuln > t::PLAYER_HURT_INVULN - 0.25 => 6,
        _ if moving => 2 + ((time * 10.0) as u32) % 2,
        _ => ((time * 2.0) as u32) % 2,
    };
    let flash = p.flash > 0.0;
    let row = row + if flash { art::CRITTER_ROWS } else { 0 };
    let blink = p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && !flash;
    let tint = if blink {
        Color::new(1.0, 1.0, 1.0, 0.35)
    } else {
        WHITE
    };
    // 모으는 동안 한 줄 웅크린다(0168, Naïl과 같게 — 0169 R8).
    let crouch = if matches!(p.state, player::State::Charging { .. }) {
        1.0
    } else {
        0.0
    };
    draw_texture_ex(
        tex,
        (pos.x - 8.0).round(),
        (pos.y - 9.0).round() + crouch,
        tint,
        DrawTextureParams {
            source: Some(art::cell(frame, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: p.face_left,
            ..Default::default()
        },
    );
}

pub(super) fn draw_nail(art: &Art, p: &Player, fur: u8, alpha: f32, time: f32) {
    match p.kit {
        crate::equipment::Character::Momo => {
            return draw_kin_player(art, p, art::MOMO_BLOOM_ROW, alpha, time)
        }
        crate::equipment::Character::Doto => {
            return draw_kin_player(art, p, art::DOTO_BACK_ROW, alpha, time)
        }
        crate::equipment::Character::Mole => {}
    }
    let pos = lerp(p.prev_pos, p.pos, alpha);
    let moving = (p.pos - p.prev_pos).length() > 0.05;
    // 파고들기(0160): 첫 BURROW_DIVE_TIME 동안 서 있는 그림이 땅속으로 가라앉고 흙더미가 솟는다.
    if let player::State::Burrow { time: bt } = p.state {
        if bt < t::BURROW_DIVE_TIME {
            let k = (bt / t::BURROW_DIVE_TIME).clamp(0.0, 1.0);
            let sink = (16.0 * k).round();
            if sink < 16.0 {
                draw_texture_ex(
                    art.nail_fur(fur),
                    pos.x - 8.0,
                    pos.y - 8.0 + sink,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, 0.0, art::CELL, art::CELL - sink)),
                        dest_size: Some(vec2(16.0, 16.0 - sink)),
                        flip_x: p.face_left,
                        ..Default::default()
                    },
                );
            }
            // 흙더미는 아래서부터 올라온다.
            let rise = (16.0 * (1.0 - k)).round();
            draw_texture_ex(
                art.nail_fur(fur),
                pos.x - 8.0,
                pos.y - 8.0 + rise,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, art::CELL * 2.0, art::CELL, art::CELL - rise)),
                    dest_size: Some(vec2(16.0, 16.0 - rise)),
                    flip_x: p.face_left,
                    ..Default::default()
                },
            );
            return;
        }
    }
    // 솟구치기(0160): 폴짝 튀어 올랐다 내려앉는다.
    let pop = match p.state {
        player::State::Emerge { time: et, .. } => {
            let k = (et / t::EMERGE_TIME).clamp(0.0, 1.0);
            (t::EMERGE_POP_HEIGHT * (k * std::f32::consts::PI).sin()).round()
        }
        _ => 0.0,
    };
    let (col, row) = match p.state {
        player::State::Dead => (4, 1),
        // 힘을 모으면(0158) 흙더미가 더 빨리 들썩인다.
        player::State::Burrow { time: bt } => {
            (((bt * (12.0 + 16.0 * p.burrow_charge())) as u32) % 2, 2)
        }
        player::State::Emerge { .. } | player::State::Drill { .. } => (2, 1),
        player::State::Recover { .. } => (3, 1),
        // 한 타 = 들기 → 치기 → (마지막 타) 거두기. 1·2타는 가까운 발·먼 발을 번갈아, 마지막 타는 두 발(0082).
        player::State::Attack { idx, time: at, .. } => {
            let (a0, a1) = p.moveset.hits[idx].active;
            let pose = if idx == p.moveset.last() {
                2
            } else {
                (idx % 2) as u32
            };
            let col = if at < a0 {
                5 + pose
            } else if pose == 2 && at > a1 + 0.04 {
                8
            } else {
                pose
            };
            (col, 1)
        }
        // 강공격(0161): 모으는 동안 마지막 타의 "들기" 자세(다 모이면 떨린다), 스윙은 마지막 타 그림.
        player::State::Charging { .. } => (7, 1),
        player::State::Heavy { time: at, .. } => {
            let (a0, a1) = p.moveset.heavy().active;
            let col = if at < a0 {
                7
            } else if at > a1 + 0.04 {
                8
            } else {
                2
            };
            (col, 1)
        }
        _ if p.invuln > t::PLAYER_HURT_INVULN - 0.25 => (3, 1),
        _ if moving => (2 + ((time * 10.0) as u32) % 4, 0),
        _ => (((time * 2.0) as u32) % 2, 0),
    };
    let flash = p.flash > 0.0 && !matches!(p.state, player::State::Burrow { .. });
    let row = if flash { row + 3 } else { row };
    let blink = p.invuln > 0.0 && ((time * 24.0) as i32) % 2 == 0 && !flash;
    let tint = if blink {
        Color::new(1.0, 1.0, 1.0, 0.35)
    } else {
        WHITE
    };
    if pop > 0.0 {
        draw_ellipse(pos.x, pos.y + 6.0, 7.0 - pop * 0.25, 2.5, 0.0, SHADOW_SOFT);
    }
    // 다 모이면 떨린다(0161).
    let jitter = match p.state {
        player::State::Charging { time: ct, .. } if ct >= t::HEAVY_CHARGE => {
            ((time * 60.0) as i32 % 2) as f32
        }
        _ => 0.0,
    };
    // 모으는 동안 한 줄 웅크린다(0168).
    let crouch = if matches!(p.state, player::State::Charging { .. }) {
        1.0
    } else {
        0.0
    };
    draw_texture_ex(
        art.nail_fur(fur),
        pos.x - 8.0 + jitter,
        pos.y - 8.0 - pop + crouch,
        tint,
        DrawTextureParams {
            source: Some(art::cell(col, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: p.face_left,
            ..Default::default()
        },
    );
}

/// 적: 그림(critters.png)이 있으면 종류의 줄에서 상태에 맞는 칸, 없으면 색 원.
/// 예고(떨림·웅크림·조준)는 칸 4 + 떨림/밝아짐, 공격은 칸 5, 맞음·경직은 칸 6. 번쩍일 때는 흰 실루엣 줄.
pub(super) fn draw_enemy(art: Option<&Art>, e: &Enemy, alpha: f32, time: f32, stop: f32) {
    let sp = e.sp();
    let mut pos = lerp(e.prev_pos, e.pos, alpha);
    let r = e.radius();
    // 맞은 쪽 진동·찌그러짐(0168, 사쿠라이): 히트스톱 동안 맞은 축으로 ±1px(센 타격은 2px) 번갈아,
    // 첫 순간은 맞은 축으로 눌린다. 렌더만, 상태 없음.
    let mut body = vec2(16.0, 16.0);
    if e.flash > 0.0 && stop > 0.0 && !shadow_pass() {
        let axis = e.vel.normalize_or(vec2(1.0, 0.0));
        let amp = if e.vel.length() > 300.0 { 2.0 } else { 1.0 };
        let sign = if ((macroquad::time::get_time() * 30.0) as i64) % 2 == 0 {
            1.0
        } else {
            -1.0
        };
        pos += axis * amp * sign;
        if e.flash > t::HIT_FLASH - 0.035 {
            body = if axis.x.abs() >= axis.y.abs() {
                vec2(14.0, 18.0)
            } else {
                vec2(18.0, 14.0)
            };
        }
    }
    if let enemy::State::Under { time: ut } = e.state {
        draw_dig_mound(pos, ut, e.windup(), time);
        return;
    }
    let base = sp.color;
    let mut tint = WHITE;
    // 그림자 패스(0106)에서는 몸만: 거품·별·잔상·고리·Z·체력 점은 빛·표시라 그림자를 드리우지 않는다.
    let marks = !shadow_pass();
    if e.poison > 0.0 && e.flash <= 0.0 {
        // 독: 초록빛이 돌고 거품이 오른다.
        tint = Color::new(0.7, 1.0, 0.6, 1.0);
        for k in (0..2).filter(|_| marks) {
            let ph = (time * 1.6 + k as f32 * 0.5 + e.pos.x * 0.01).fract();
            let bx = pos.x + (k as f32 * 2.0 - 1.0) * (r * 0.5) + (ph * 9.0).sin();
            let by = pos.y - r - 4.0 - ph * 8.0;
            draw_circle(
                bx.round(),
                by.round(),
                1.0,
                Color::new(0.55, 0.9, 0.35, 1.0 - ph),
            );
        }
    }
    let moving = (e.pos - e.prev_pos).length() > 0.05;
    let walk = ((time * 8.0 + e.pos.x * 0.05) as u32) % 2;
    let mut frame = if moving {
        2 + walk
    } else {
        walk.min(1) * ((time * 2.0) as u32 % 2)
    };
    match e.state {
        enemy::State::Windup { time: wt } | enemy::State::Aim { time: wt } => {
            frame = 4;
            // 예고: 떨림 + 점점 붉어짐
            pos.x += if ((time * 60.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let k = (wt / e.windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.45 * k, 1.0 - 0.45 * k, 1.0);
            if matches!(e.state, enemy::State::Aim { .. })
                && !matches!(
                    sp.behavior,
                    crate::species::Behavior::Tongue | crate::species::Behavior::Spore
                )
            {
                // 도토리를 쳐든다.
                let hand = pos + e.facing * (r + 1.0) - vec2(0.0, 4.0);
                draw_acorn(hand, 0.0);
            }
        }
        enemy::State::Crouch { time: ct, .. } => {
            frame = 4;
            let k = (ct / e.windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.4 * k, 1.0 - 0.4 * k, 1.0);
        }
        enemy::State::Leap { .. } => {
            frame = 5;
            pos.y -= e.height();
        }
        enemy::State::Pop { time: pt } => {
            // 튀어나옴: 살짝 솟았다가 내려온다.
            frame = 5;
            let k = (pt / t::MOMO_POP_TIME).clamp(0.0, 1.0);
            pos.y -= (4.0 * 6.0 * k * (1.0 - k)).round();
        }
        enemy::State::Rest { .. } if sp.behavior != crate::species::Behavior::Dummy => {
            tint = Color::new(0.75, 0.75, 0.75, 1.0);
        }
        enemy::State::Stunned { time: st } => {
            frame = 6;
            // 머리 위에 노란 점이 돈다.
            let a = st * 12.0;
            let s = pos + vec2(a.cos(), a.sin() * 0.5) * (r + 3.0) - vec2(0.0, r + 4.0);
            if marks {
                draw_circle(s.x.round(), s.y.round(), 1.0, YELLOW);
            }
        }
        enemy::State::Charge { dir, .. } => {
            frame = 5;
            // 잔상
            for i in (1..4).filter(|_| marks) {
                let back = pos - dir * (i as f32 * 4.0);
                let a = 0.4 - i as f32 * 0.1;
                draw_circle(
                    back.x,
                    back.y,
                    r - i as f32,
                    Color::new(base.r, base.g, base.b, a),
                );
            }
        }
        _ => {}
    }
    if let enemy::State::Howl { time: ht } = e.state {
        if e.kind == crate::enemy::Kind::Doto {
            // 도토: 땅을 판다(웅크린 칸이 떨고, 앞에 흙이 튄다). 울음 고리는 없다.
            frame = 4;
            pos.x += if ((time * 30.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let ph = (ht * 4.0).fract();
            let front = pos + e.facing * (r + 2.0);
            draw_rectangle(
                (front.x + (ph * 7.0).sin() * 3.0).round(),
                (front.y - 2.0 - ph * 6.0).round(),
                1.0,
                1.0,
                world::DIRT,
            );
        } else {
            frame = 7;
        }
    }
    // 토끼풀(0165): 발 구르기 예고는 웅크려 떨고(점점 붉게), 품기는 보랏빛으로 실을 감는다.
    match e.state {
        enemy::State::Thump { time: tt } => {
            frame = 4;
            pos.x += if ((time * 40.0) as i32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let k = (tt / e.thump_windup()).min(1.0);
            tint = Color::new(1.0, 1.0 - 0.45 * k, 1.0 - 0.45 * k, 1.0);
        }
        enemy::State::Gather { time: gt } => {
            frame = 4;
            tint = Color::new(0.88, 0.82, 1.0, 1.0);
            if marks {
                for k in 0..3 {
                    let ph = (gt * 1.4 + k as f32 / 3.0).fract();
                    draw_circle_lines(
                        pos.x.round(),
                        (pos.y - 2.0).round(),
                        r + 2.0 + ph * 10.0,
                        1.0,
                        Color::new(0.80, 0.62, 0.98, 0.6 * (1.0 - ph)),
                    );
                }
            }
        }
        _ => {}
    }
    if let (enemy::State::Howl { time: ht }, false) = (e.state, e.kind == crate::enemy::Kind::Doto)
    {
        // 울음: 퍼지는 고리 둘.
        for k in (0..2).filter(|_| marks) {
            let ph = ((ht * 2.5) + k as f32 * 0.5).fract();
            draw_circle_lines(
                pos.x,
                pos.y - 4.0,
                r + 4.0 + ph * 22.0,
                1.0,
                Color::new(1.0, 0.85, 0.8, 0.6 * (1.0 - ph)),
            );
        }
    }
    // 콩이(0054): 말려 있으면 공(칸 4·5). 구를 때는 두 칸을 번갈아 띠가 돈다. 버틸 때는 등딱지에 빛이 돈다.
    if e.kind == crate::enemy::Kind::Kongi {
        match e.state {
            enemy::State::Charge { .. } => frame = 4 + ((time * 14.0) as u32) % 2,
            enemy::State::Windup { .. } => frame = 4,
            enemy::State::Guard { .. } if marks => {
                frame = 4;
                let ph = (time * 2.5).fract();
                let a = std::f32::consts::PI * (1.1 + ph * 0.8);
                let g = pos + vec2(a.cos(), a.sin() * 0.8) * (r - 1.0) - vec2(0.0, 2.0);
                draw_rectangle(g.x.round(), g.y.round(), 1.0, 1.0, world::SHELL);
                draw_circle_lines(
                    pos.x.round(),
                    (pos.y - 2.0).round(),
                    r + 3.0,
                    1.0,
                    Color::new(0.75, 0.90, 1.0, 0.35 + 0.25 * (time * 6.0).sin().abs()),
                );
            }
            enemy::State::Guard { .. } => frame = 4,
            _ => {}
        }
    }
    // 새 졸개(0146): 고슴도치는 콩이처럼 공(칸 4·5), 혀 내밀기·포자 뿜기는 칸 5.
    match (sp.behavior, e.state) {
        (crate::species::Behavior::Roller, enemy::State::Charge { .. }) => {
            frame = 4 + ((time * 14.0) as u32) % 2
        }
        (crate::species::Behavior::Roller, enemy::State::Windup { .. }) => frame = 4,
        (_, enemy::State::Lash { .. } | enemy::State::Puff { .. }) => frame = 5,
        _ => {}
    }
    if e.flying() {
        // 박쥐: 예고·내리꽂기·맞음이 아니면 늘 날갯짓(칸 0·1). 높이만큼 떠서 살랑인다(그림자는 바닥에 남는다).
        if !matches!(
            e.state,
            enemy::State::Windup { .. }
                | enemy::State::Charge { .. }
                | enemy::State::Stunned { .. }
        ) {
            frame = ((time * 10.0 + e.pos.x * 0.03) as u32) % 2;
        }
        pos.y -= (e.height() + (time * 5.0 + e.pos.x * 0.02).sin()).round();
    }
    if e.asleep {
        frame = 1;
        // 자는 중: Z가 떠오른다.
        let ph = (time * 0.8 + e.pos.x * 0.013).fract();
        let z = pos + vec2(4.0 + ph * 3.0, -r - 6.0 - ph * 6.0);
        let c = Color::new(0.9, 0.9, 1.0, 1.0 - ph);
        if marks {
            draw_line(z.x, z.y, z.x + 3.0, z.y, 1.0, c);
            draw_line(z.x + 3.0, z.y, z.x, z.y + 3.0, 1.0, c);
            draw_line(z.x, z.y + 3.0, z.x + 3.0, z.y + 3.0, 1.0, c);
        }
    }
    // 엘리트(0147): 발밑에 보라 실 고리가 숨 쉬고, 실 매듭 셋이 돈다.
    if e.elite && marks {
        let ph = time * 3.0 + e.pos.x * 0.05;
        draw_circle_lines(
            pos.x.round(),
            (pos.y + r * 0.4).round(),
            r + 3.0 + ph.sin() * 0.8,
            1.0,
            Color::new(0.75, 0.50, 0.95, 0.55),
        );
        for i in 0..3 {
            let a = ph * 0.8 + i as f32 * std::f32::consts::TAU / 3.0;
            let q = pos + vec2(a.cos() * (r + 3.0), a.sin() * (r + 3.0) * 0.6 - 2.0);
            draw_rectangle(
                q.x.round(),
                q.y.round(),
                1.0,
                1.0,
                Color::new(0.85, 0.65, 1.0, 0.9),
            );
        }
    }
    let flash = e.flash > 0.0;
    let chief = e.kind == crate::enemy::Kind::Chief;
    if let (true, Some(tex)) = (chief, art.and_then(|a| a.chief.as_ref())) {
        // 대장: 32×32(발바닥 y=30).
        let row = art::CHIEF_TAINTED + if flash { art::CHIEF_ROWS } else { 0 };
        let big = body * 2.0;
        draw_texture_ex(
            tex,
            (pos.x - big.x / 2.0).round(),
            (pos.y + r - 30.0 - (big.y - 32.0)).round(),
            if flash { WHITE } else { tint },
            DrawTextureParams {
                source: Some(art::chief_cell(frame, row)),
                dest_size: Some(big),
                flip_x: e.facing.x < 0.0,
                ..Default::default()
            },
        );
        return;
    }
    match art.and_then(|a| a.critters.as_ref()) {
        Some(tex) => {
            let row = sp.sheet_row + if flash { art::CRITTER_ROWS } else { 0 };
            draw_texture_ex(
                tex,
                (pos.x - body.x / 2.0).round(),
                (pos.y + r - 15.0 - (body.y - 16.0)).round(),
                if flash { WHITE } else { tint },
                DrawTextureParams {
                    source: Some(art::cell(frame, row)),
                    dest_size: Some(body),
                    flip_x: e.facing.x < 0.0,
                    ..Default::default()
                },
            );
        }
        None => {
            let c = if flash {
                WHITE
            } else {
                Color::new(base.r * tint.r, base.g * tint.g, base.b * tint.b, 1.0)
            };
            draw_circle(pos.x, pos.y, r, c);
            let nose = pos + e.facing * (r + 1.0);
            draw_circle(nose.x.round(), nose.y.round(), 1.5, BLACK);
        }
    }
    // 엘리트는 처음부터 작은 막대(점이 너무 많다).
    if marks && e.elite {
        let (w, y) = (18.0, (pos.y - r - 9.0).round());
        let x = (pos.x - w / 2.0).round();
        draw_rectangle(x - 1.0, y - 1.0, w + 2.0, 4.0, OUTLINE);
        draw_rectangle(x, y, w, 2.0, DARKGRAY);
        let k = (e.hp.max(0) as f32 / e.max_hp.max(1) as f32).clamp(0.0, 1.0);
        draw_rectangle(x, y, (w * k).round(), 2.0, Color::new(0.80, 0.58, 1.0, 1.0));
    }
    // 체력 점(맞은 적만: 처음부터 숫자를 늘어놓지 않는다). 보스는 화면 아래 막대.
    if marks
        && !e.is_boss()
        && !e.elite
        && (e.hp < e.max_hp || sp.behavior == crate::species::Behavior::Dummy)
    {
        let max = e.max_hp;
        for i in 0..max {
            let c = if i < e.hp { WHITE } else { DARKGRAY };
            draw_rectangle(
                (pos.x - max as f32 * 1.5 + i as f32 * 3.0).round(),
                (pos.y - r - 9.0).round(),
                2.0,
                2.0,
                c,
            );
        }
    }
}

/// 이웃 그림 한 칸(발이 `pos.y + 6` 근처). 걸으면 칸 2·3, 서 있으면 0·1.
pub(super) fn draw_critter_at(
    tex: &Texture2D,
    row: u32,
    pos: Vec2,
    face_left: bool,
    moving: bool,
    time: f32,
) {
    let frame = if moving {
        2 + ((time * 8.0) as u32) % 2
    } else {
        ((time * 2.0) as u32) % 2
    };
    draw_texture_ex(
        tex,
        (pos.x - 8.0).round(),
        (pos.y - 9.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(art::cell(frame, row)),
            dest_size: Some(vec2(16.0, 16.0)),
            flip_x: face_left,
            ..Default::default()
        },
    );
}

/// 내 캐릭터 표시: 발밑의 따뜻한 빛 고리(색이 아니라 모양으로도 구별되게). 바닥에 그려 다른 그림을 가리지 않는다.
/// 적이 많을 때 Naïl을 놓치지 않게(사람 피드백 2026-10-25). 아주 천천히 숨 쉬듯 밝아졌다 어두워진다.
pub(super) fn draw_player_halo(p: Vec2, time: f32) {
    let pulse = 0.5 + 0.5 * (time * 2.2).sin();
    let a = 0.45 + 0.2 * pulse;
    draw_ellipse(
        p.x,
        p.y + 6.0,
        9.0,
        3.5,
        0.0,
        Color::new(1.0, 0.92, 0.65, 0.10 + 0.05 * pulse),
    );
    draw_ellipse_lines(
        p.x,
        p.y + 6.0,
        9.0,
        3.5,
        0.0,
        1.0,
        Color::new(1.0, 0.9, 0.6, a),
    );
}

/// 두꺼비 혀(0146): 맞는 순간 다 뻗었다가 말려 들어간다(맞았는지는 월드가 뻗은 틱에 본다).
pub(super) fn draw_tongue(w: &World, pos: Vec2, r: f32, dir: Vec2, time: f32) {
    let start = pos + dir * r;
    let reach = w.room.ray_len(start, dir, 1.0, t::TOAD_TONGUE_REACH);
    let k = (time / t::TOAD_LASH_TIME).clamp(0.0, 1.0);
    let len = reach * (1.0 - k * k);
    let (a, b) = (
        (start - vec2(0.0, 2.0)).round(),
        (start + dir * len - vec2(0.0, 2.0)).round(),
    );
    draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(0.94, 0.50, 0.60, 1.0));
    draw_circle(b.x, b.y, 2.0, Color::new(0.98, 0.62, 0.70, 1.0));
}

/// 도토리(다람쥐가 던지는 것): 갈색 열매 + 짙은 모자.
pub(super) fn draw_acorn(p: Vec2, spin: f32) {
    let p = p.round();
    let up = Vec2::from_angle(spin - std::f32::consts::FRAC_PI_2);
    draw_circle(p.x, p.y, 2.5, OUTLINE);
    draw_circle(p.x, p.y + 0.5, 2.0, Color::new(0.78, 0.52, 0.28, 1.0));
    let cap = p + up * 1.5;
    draw_rectangle(
        cap.x - 2.0,
        cap.y - 1.0,
        4.0,
        2.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
    let stem = p + up * 3.0;
    draw_rectangle(
        stem.x,
        stem.y - 1.0,
        1.0,
        1.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
}

/// 전방 반원(채움). 삼각형 부채꼴.
pub(super) fn draw_sector(origin: Vec2, dir: Vec2, r: f32, color: Color) {
    let base = dir.y.atan2(dir.x);
    let steps = 12;
    let half = std::f32::consts::FRAC_PI_2;
    for i in 0..steps {
        let a0 = base - half + (2.0 * half) * i as f32 / steps as f32;
        let a1 = base - half + (2.0 * half) * (i + 1) as f32 / steps as f32;
        draw_triangle(
            origin,
            origin + vec2(a0.cos(), a0.sin()) * r,
            origin + vec2(a1.cos(), a1.sin()) * r,
            color,
        );
    }
}

pub(super) fn draw_sector_lines(origin: Vec2, dir: Vec2, r: f32, color: Color) {
    let base = dir.y.atan2(dir.x);
    let steps = 12;
    let half = std::f32::consts::FRAC_PI_2;
    let mut prev = origin + vec2((base - half).cos(), (base - half).sin()) * r;
    draw_line(origin.x, origin.y, prev.x, prev.y, 1.0, color);
    for i in 1..=steps {
        let a = base - half + (2.0 * half) * i as f32 / steps as f32;
        let p = origin + vec2(a.cos(), a.sin()) * r;
        draw_line(prev.x, prev.y, p.x, p.y, 1.0, color);
        prev = p;
    }
    draw_line(origin.x, origin.y, prev.x, prev.y, 1.0, color);
}
