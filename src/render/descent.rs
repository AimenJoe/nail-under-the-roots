//! 내려가는 전환(0156): 지층 단면과 파 내려가는 Naïl.

use super::*;

/// 1m가 화면 몇 px인가(단면이 흐르는 빠르기).
pub(super) const DESCENT_PX_PER_M: f32 = 12.0;
/// 파 내려가는 Naïl의 화면 높이(머리 끝).
pub(super) const DESCENT_DIG_Y: f32 = 104.0;

/// 파 내려가기 장면의 칸(dig.png 열)과 "돌고 있나". 주기 `DIG_CYCLE`초: 긁기 → 우리 보기(윙크 한 번) → 회전.
pub(super) fn dig_col(clock: f32) -> (u32, bool) {
    const DIG_CYCLE: f32 = 1.6;
    let t = clock % DIG_CYCLE;
    if t < DIG_T_LOOK {
        (((t * 12.0) as u32) % 6, false)
    } else if t < DIG_T_SPIN {
        let wink = (DIG_T_LOOK + 0.16..DIG_T_LOOK + 0.28).contains(&t);
        (if wink { 7 } else { 6 }, false)
    } else {
        (8 + (((t - DIG_T_SPIN) * 30.0) as u32) % 8, true)
    }
}
pub(super) const DIG_T_LOOK: f32 = 0.35;
pub(super) const DIG_T_SPIN: f32 = 0.75;

/// 깊이(m)에 맞는 흙 색: 땅 위 겉흙 → 흙 → 뿌리 흙 → 검은 바위 → 가장 깊은 곳.
pub(super) fn strata_color(m: f32) -> Color {
    const KEYS: [(f32, [f32; 3]); 6] = [
        (0.0, [0.42, 0.29, 0.18]),
        (8.0, [0.37, 0.25, 0.16]),
        (30.0, [0.29, 0.19, 0.13]),
        (55.0, [0.21, 0.15, 0.12]),
        (85.0, [0.14, 0.12, 0.16]),
        (130.0, [0.08, 0.07, 0.11]),
    ];
    let m = m.max(0.0);
    let i = KEYS.iter().rposition(|(k, _)| *k <= m).unwrap_or(0);
    let (a, ca) = KEYS[i];
    let (b, cb) = KEYS[(i + 1).min(KEYS.len() - 1)];
    let f = if b > a {
        ((m - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Color::new(
        ca[0] + (cb[0] - ca[0]) * f,
        ca[1] + (cb[1] - ca[1]) * f,
        ca[2] + (cb[2] - ca[2]) * f,
        1.0,
    )
}

pub(super) fn shade(c: Color, k: f32, a: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, a)
}

/// 지층 단면을 화면(내부 해상도, 화면 고정 카메라)에 그린다. `d.depth_m`이 가운데 깊이, `clock`은 실제 시간
/// (굽는 동안에도 Naïl과 흙 부스러기는 움직인다). `cover`만큼 아래 월드를 덮는다.
pub(super) fn draw_descent(
    art: Option<&Art>,
    w: &World,
    d: crate::adventure::Descent,
    clock: f32,
    cover: f32,
) {
    if cover <= 0.0 {
        return;
    }
    let a = cover;
    let px = DESCENT_PX_PER_M;
    // 화면 맨 위 줄의 절대 깊이(px). 정수로 굴려 도트가 떨리지 않게.
    let top = (d.depth_m * px - DESCENT_DIG_Y).round();
    // 지층: 4m 띠마다 색이 조금씩 다르고, 8px 기둥마다 경계가 물결친다. 2px 줄 단위.
    let col_w = 8.0;
    let cols = (RW / col_w).ceil() as i32;
    let mut y = 0.0;
    while y < RH {
        let abs_y = top + y;
        for c in 0..cols {
            let x = c as f32 * col_w;
            if abs_y < 0.0 {
                // 땅 위: 저녁 하늘.
                let k = ((abs_y + 120.0) / 120.0).clamp(0.0, 1.0);
                let sky = Color::new(0.36 + 0.30 * k, 0.24 + 0.12 * k, 0.33 - 0.05 * k, a);
                draw_rectangle(x, y, col_w, 2.0, sky);
                continue;
            }
            let wob = (hash01(c as u32, (abs_y / 48.0).floor() as u32 + 7) - 0.5) * 6.0;
            let m = (abs_y + wob) / px;
            let band = (m / 4.0).floor();
            let base = strata_color(band * 4.0);
            let k = 0.92 + 0.16 * hash01(band as u32, 3);
            draw_rectangle(x, y, col_w, 2.0, shade(base, k, a));
        }
        y += 2.0;
    }
    // 지층 경계: 4m마다 어두운 가는 줄(기둥마다 물결).
    for c in 0..cols {
        let x = c as f32 * col_w;
        let b0 = (top / (4.0 * px)).floor() as i32;
        for b in b0..=b0 + (RH / (4.0 * px)).ceil() as i32 + 1 {
            if b <= 0 {
                continue;
            }
            let wob = (hash01(c as u32, b as u32 + 7) - 0.5) * 6.0;
            let yy = (b as f32 * 4.0 * px - wob - top).round();
            if (0.0..RH).contains(&yy) {
                let base = strata_color(b as f32 * 4.0);
                draw_rectangle(x, yy, col_w, 1.0, shade(base, 0.72, a));
            }
        }
    }
    // 큰 돌: 32px 칸에 하나쯤, 반지름 3~6px 덩어리(외곽선 + 윗면 밝게).
    let big = 32.0;
    let bfirst = (top / big).floor() as i32;
    for r in 0..=(RH / big).ceil() as i32 + 1 {
        let by = bfirst + r;
        if by < 0 {
            continue;
        }
        for c in 0..(RW / big).ceil() as i32 {
            let id = (by as u32).wrapping_mul(131) ^ (c as u32).wrapping_mul(7);
            if hash01(id, 31) > 0.32 {
                continue;
            }
            let cx = c as f32 * big + 6.0 + (hash01(id, 32) * 20.0).floor();
            let cy = by as f32 * big + 6.0 + (hash01(id, 33) * 20.0).floor() - top;
            if (cx - 160.0).abs() < 16.0 && cy < DESCENT_DIG_Y + 8.0 {
                continue;
            }
            let rr = 3.0 + (hash01(id, 34) * 3.0).floor();
            let m = (cy + top) / px;
            let deep = (m / 100.0).clamp(0.0, 1.0);
            let st = Color::new(
                0.50 - 0.22 * deep,
                0.46 - 0.22 * deep,
                0.42 - 0.16 * deep,
                a,
            );
            for dy in -rr as i32..=rr as i32 {
                let half = ((rr * rr - (dy as f32) * (dy as f32)).max(0.0).sqrt() * 1.3).round();
                let y = cy + dy as f32;
                draw_rectangle(
                    cx - half - 1.0,
                    y,
                    half * 2.0 + 2.0,
                    1.0,
                    shade(st, 0.45, a),
                );
                let tone = if dy < -(rr as i32) / 3 {
                    1.18
                } else if dy > (rr as i32) / 3 {
                    0.82
                } else {
                    1.0
                };
                draw_rectangle(cx - half, y, half * 2.0, 1.0, shade(st, tone, a));
            }
        }
    }
    // 긴 뿌리: 24px 줄마다 가끔 옆으로 길게(얕은 곳은 가늘게, 뿌리 지대는 굵게, 심층은 보라 실이 감긴다).
    let lane = 24.0;
    let lfirst = (top / lane).floor() as i32;
    for r in 0..=(RH / lane).ceil() as i32 + 1 {
        let ly = lfirst + r;
        if ly < 0 {
            continue;
        }
        let id = (ly as u32).wrapping_mul(2654435761);
        let m = ly as f32 * lane / px;
        if hash01(id, 41)
            > if (30.0..90.0).contains(&m) {
                0.55
            } else {
                0.25
            }
        {
            continue;
        }
        let x0 = (hash01(id, 42) * 200.0).floor() - 20.0;
        let len = 60.0 + (hash01(id, 43) * 90.0).floor();
        let thick = if (30.0..90.0).contains(&m) { 2.0 } else { 1.0 };
        let y0 = ly as f32 * lane + (hash01(id, 44) * 12.0).floor() - top;
        let rc = Color::new(0.50, 0.36, 0.25, a);
        let threaded = m > 45.0 && hash01(id, 45) < 0.5;
        let ph = hash01(id, 46) * 6.0;
        let mut x = x0;
        while x < x0 + len {
            if (x - 160.0).abs() > 7.0 || y0 > DESCENT_DIG_Y + 6.0 {
                let yy = (y0 + ((x - x0) * 0.08 + ph).sin() * 3.0).round();
                draw_rectangle(x, yy, 1.0, thick, rc);
                draw_rectangle(x, yy + thick, 1.0, 1.0, shade(rc, 0.6, a));
                if threaded && ((x - x0) as i32) % 9 < 2 {
                    draw_rectangle(x, yy - 1.0, 1.0, 1.0, Color::new(0.70, 0.50, 0.90, a));
                }
            }
            x += 1.0;
        }
    }
    // 풀 줄(땅 위와 흙 사이).
    if top < 0.0 && top > -RH {
        let gy = -top;
        draw_rectangle(0.0, gy - 3.0, RW, 3.0, Color::new(0.36, 0.56, 0.26, a));
        draw_rectangle(0.0, gy - 4.0, RW, 1.0, Color::new(0.52, 0.72, 0.36, a));
    }
    // 장식: 8px 칸 해시. 깊이마다 나오는 것이 다르다(조약돌 늘, 잔뿌리·지렁이는 얕은 곳, 굵은 뿌리·보라 실은 가운데,
    // 빛버섯·수정은 깊은 곳). 칸은 절대 깊이에 붙어 있어 내려가면 위로 흘러간다.
    let cell = 8.0;
    let first = (top / cell).floor() as i32;
    let rows = (RH / cell).ceil() as i32 + 1;
    for r in 0..rows {
        let cy = first + r;
        if cy < 0 {
            continue;
        }
        let m = cy as f32 * cell / px;
        let y0 = cy as f32 * cell - top;
        for c in 0..cols {
            let x0 = c as f32 * cell;
            // 가운데 굴(파 온 길)에는 장식을 두지 않는다.
            if (x0 + 4.0 - 160.0).abs() < 16.0 && y0 < DESCENT_DIG_Y {
                continue;
            }
            let id = (cy as u32).wrapping_mul(97) ^ c as u32;
            let h = hash01(id, 11);
            let ox = (hash01(id, 12) * 5.0).floor();
            let oy = (hash01(id, 13) * 5.0).floor();
            let (x, y) = (x0 + ox, y0 + oy);
            let soil = strata_color(m);
            if h < 0.035 {
                // 조약돌
                let st = Color::new(0.52, 0.47, 0.42, a);
                draw_rectangle(x, y + 1.0, 4.0, 2.0, shade(st, 0.8, a));
                draw_rectangle(x + 1.0, y, 2.0, 1.0, st);
                draw_rectangle(x + 1.0, y + 1.0, 1.0, 1.0, Color::new(0.70, 0.66, 0.60, a));
            } else if h < 0.07 && m < 45.0 {
                // 잔뿌리
                let rc = Color::new(0.55, 0.40, 0.27, a * 0.8);
                for i in 0..6 {
                    let yy = y + ((i as f32 * 0.9 + hash01(id, 14) * 3.0).sin() * 1.5).round();
                    draw_rectangle(x + i as f32, yy, 1.0, 1.0, rc);
                }
            } else if h < 0.082 && (3.0..35.0).contains(&m) {
                // 지렁이
                let pink = Color::new(0.86, 0.52, 0.55, a);
                draw_rectangle(x, y + 1.0, 4.0, 1.0, pink);
                draw_rectangle(x + 4.0, y, 1.0, 1.0, pink);
            } else if h < 0.11 && (35.0..85.0).contains(&m) {
                // 굵은 뿌리
                let rc = Color::new(0.46, 0.33, 0.23, a);
                for i in 0..8 {
                    let yy = y + ((i as f32 * 0.6 + hash01(id, 15) * 6.0).sin() * 2.0).round();
                    draw_rectangle(x + i as f32 - 1.0, yy, 1.0, 2.0, rc);
                }
            } else if h < 0.13 && m > 40.0 {
                // 보라 실(물든 뿌리)
                let th = Color::new(0.66, 0.47, 0.85, a * 0.9);
                for i in 0..4 {
                    draw_rectangle(x + i as f32, y + i as f32 * 0.5, 1.0, 1.0, th);
                }
            } else if h < 0.16 && m > 78.0 {
                // 빛버섯: 갓 + 둘레 빛
                let glow = Color::new(0.35, 0.75, 0.95, a * 0.25);
                draw_rectangle(x - 1.0, y - 1.0, 5.0, 4.0, glow);
                draw_rectangle(x, y, 3.0, 1.0, Color::new(0.45, 0.85, 1.0, a));
                draw_rectangle(x + 1.0, y + 1.0, 1.0, 2.0, Color::new(0.80, 0.85, 0.80, a));
            } else if h < 0.17 && (10.0..60.0).contains(&m) {
                // 옛 조개 화석
                let fc = Color::new(0.80, 0.74, 0.60, a * 0.8);
                draw_rectangle(x, y, 3.0, 1.0, fc);
                draw_rectangle(x, y + 1.0, 1.0, 1.0, fc);
                draw_rectangle(x + 2.0, y + 1.0, 1.0, 1.0, fc);
            } else if h < 0.24 {
                // 흙 얼룩(밝고 어두운 점)
                draw_rectangle(x, y, 2.0, 1.0, shade(soil, 1.25, a * 0.7));
            }
        }
    }
    // 파 온 굴: Naïl 위로 어두운 기둥, 가장자리 한 줄은 더 어둡다.
    let tw = 18.0;
    let tx = 160.0 - tw / 2.0;
    let ty = (-top).max(0.0);
    if ty < DESCENT_DIG_Y {
        let h = DESCENT_DIG_Y - ty + 4.0;
        let soil = strata_color(d.depth_m);
        draw_rectangle(tx, ty, tw, h, shade(soil, 0.42, a));
        draw_rectangle(tx - 1.0, ty, 1.0, h, shade(soil, 0.65, a));
        draw_rectangle(tx + tw, ty, 1.0, h, shade(soil, 0.65, a));
        // 굴 벽에서 떨어지는 부스러기(위로 흘러간다 = 내려가는 중).
        for i in 0..10u32 {
            let ph = (clock * 1.3 + hash01(i, 21)).fract();
            let bx = tx + 1.0 + (hash01(i, 22) * (tw - 2.0)).floor();
            let by = DESCENT_DIG_Y - ph * (DESCENT_DIG_Y - ty);
            draw_rectangle(bx, by.round(), 1.0, 1.0, shade(soil, 1.3, a * 0.8));
        }
    }
    // 앞쪽으로 튀는 흙(머리 아래, 좌우로).
    let soil = strata_color(d.depth_m + 2.0);
    for i in 0..8u32 {
        let ph = (clock * 2.6 + i as f32 / 8.0).fract();
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let sx = 160.0 + side * (3.0 + ph * (6.0 + 6.0 * hash01(i, 23)));
        let sy = DESCENT_DIG_Y + 18.0 - ph * 10.0 + ph * ph * 12.0;
        draw_rectangle(
            sx.round(),
            sy.round(),
            1.0 + (i % 3 == 0) as u8 as f32,
            1.0,
            shade(soil, 1.35, a * (1.0 - ph)),
        );
    }
    // Naïl(또는 모모): 이 장면 전용 32×32(dig.png, `tools/art/dig.py`). 털빛마다 줄이 있다.
    // 순서(사장 제안): 파다가 → 고개를 들어 우리를 보고 윙크 → 드릴처럼 뱅글뱅글 돌며 내려간다. 굽기가 길면 되풀이.
    if let Some(tex) = art.and_then(|a| a.dig.as_ref()) {
        let (col, spinning) = dig_col(clock);
        let row = if w.player.kit == crate::equipment::Character::Momo {
            4
        } else {
            u32::from(w.inv.fur.min(3))
        };
        if spinning {
            // 돌며 흙을 나선으로 흩뿌린다(몸 둘레를 돌다 바깥으로).
            let soil = strata_color(d.depth_m + 1.0);
            for i in 0..10u32 {
                let ph = (clock * 2.2 + hash01(i, 71)).fract();
                let ang = -clock * 22.0 + i as f32 * 0.63;
                let r = 10.0 + ph * 14.0;
                let px = 160.0 + ang.cos() * r;
                let py = DESCENT_DIG_Y + 6.0 + ang.sin() * r * 0.55;
                let wide = 1.0 + f32::from(u8::from(i % 3 == 0));
                draw_rectangle(
                    px.round(),
                    py.round(),
                    wide,
                    1.0,
                    shade(soil, 1.3, a * (1.0 - ph)),
                );
            }
        }
        draw_texture_ex(
            tex,
            144.0,
            DESCENT_DIG_Y - 12.0,
            Color::new(1.0, 1.0, 1.0, a),
            DrawTextureParams {
                source: Some(Rect::new(col as f32 * 32.0, row as f32 * 32.0, 32.0, 32.0)),
                dest_size: Some(vec2(32.0, 32.0)),
                ..Default::default()
            },
        );
    }
    // 깊은 곳은 어둡고 Naïl 둘레만 보인다(어둠 마스크 = 가운데 투명).
    let dark = ((d.depth_m - 35.0) / 80.0).clamp(0.0, 0.85);
    if let (Some(art), true) = (art, dark > 0.0) {
        let size = 260.0;
        let (mx, my) = (160.0, DESCENT_DIG_Y + 6.0);
        draw_texture_ex(
            &art.dark,
            mx - size / 2.0,
            my - size / 2.0,
            Color::new(1.0, 1.0, 1.0, dark * a),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
        let c = Color::new(0.0, 0.0, 0.0, dark * a);
        draw_rectangle(0.0, 0.0, RW, (my - size / 2.0).max(0.0), c);
        draw_rectangle(0.0, my + size / 2.0, RW, RH, c);
        draw_rectangle(0.0, 0.0, (mx - size / 2.0).max(0.0), RH, c);
        draw_rectangle(mx + size / 2.0, 0.0, RW, RH, c);
    }
    // 위아래 가장자리 어둠(깊을수록 더 어둡다).
    let dark = (d.depth_m / 130.0).clamp(0.0, 1.0);
    for i in 0..12 {
        let k = 1.0 - i as f32 / 12.0;
        let c = Color::new(0.0, 0.0, 0.0, a * k * (0.25 + 0.45 * dark));
        draw_rectangle(0.0, i as f32 * 2.0, RW, 2.0, c);
        draw_rectangle(0.0, RH - (i as f32 + 1.0) * 2.0, RW, 2.0, c);
    }
}
