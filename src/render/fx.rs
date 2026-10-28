//! 효과: 입자, 환경 입자, 예고(돌진 띠·고리·받아치기 색), 충격파·고치·실 그물·보풀·꽃잎, 덤불·돌·구름·지뢰, 어둠·빛줄기·떨어지는 것.

use super::*;

/// 필살기 시동: 화면(카메라)을 어둡게 덮는다. 그 위에 플레이어를 다시 그리라고 true.
pub(super) fn draw_super_dim(w: &World, cam: Vec2) -> bool {
    let player::State::Drill { time, .. } = w.player.state else {
        return false;
    };
    if time >= t::DRILL_REV {
        return false;
    }
    let k = (time / 0.08).min(1.0);
    draw_rectangle(
        cam.x - 8.0,
        cam.y - 8.0,
        t::VIEW_W + 16.0,
        t::VIEW_H + 16.0,
        Color::new(0.04, 0.02, 0.06, 0.55 * k),
    );
    true
}

/// 길의 샘(0159): 돌 테두리의 맑은 웅덩이. 물결 고리가 퍼지고 반짝임이 떠다닌다. 마신 뒤엔 조용해진다.
/// 모모가 남긴 시든 꽃잎(0164): 바닥에 놓인 작은 꽃잎, 분홍빛이 조금 남아 가끔 반짝인다.
pub(super) fn draw_left_petal(at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round());
    let bob = ((time * 1.4).sin() * 0.5).round();
    draw_ellipse(x, y + 2.0, 4.0, 1.5, 0.0, SHADOW_SOFT);
    let c = Color::new(0.74, 0.56, 0.60, 1.0);
    let pink = Color::new(0.98, 0.70, 0.78, 1.0);
    draw_rectangle(x - 2.0, y - 1.0 + bob, 5.0, 2.0, c);
    draw_rectangle(x - 1.0, y - 2.0 + bob, 3.0, 1.0, c);
    draw_rectangle(x - 1.0, y + 1.0 + bob, 2.0, 1.0, c);
    draw_rectangle(x, y - 1.0 + bob, 1.0, 1.0, pink);
    let tw = (time * 2.3).sin();
    if tw > 0.6 {
        let a = (tw - 0.6) / 0.4;
        draw_rectangle(x + 2.0, y - 4.0, 1.0, 1.0, Color::new(1.0, 0.85, 0.9, a));
        draw_rectangle(
            x - 4.0,
            y - 2.0,
            1.0,
            1.0,
            Color::new(1.0, 0.85, 0.9, a * 0.7),
        );
    }
}

/// 컷씬의 어둠(0159): `gloom.at` 둘레 `radius`만 밝고 바깥은 `amount`만큼 보랏빛 검정. 환경 어둠 위에 한 번 더.
pub(super) fn draw_gloom(art: &Art, w: &World, cam: Vec2) {
    let g = w.juice.gloom;
    if g.amount <= 0.0 {
        return;
    }
    let c = g.at;
    let s = g.radius * 2.0;
    let (x0, y0) = (c.x - g.radius, c.y - g.radius);
    let (l, tp) = (cam.x - 8.0, cam.y - 8.0);
    let (r, b) = (cam.x + t::VIEW_W + 8.0, cam.y + t::VIEW_H + 8.0);
    // 보랏빛 장막을 먼저 옅게 깔고(빛 안도 조금 물든다), 그 위에 검은 어둠 마스크. 마스크 그림이 검정이라
    // 바깥 네모도 검정이어야 네모 경계가 안 보인다.
    draw_rectangle(
        l,
        tp,
        r - l,
        b - tp,
        Color::new(0.30, 0.12, 0.45, 0.22 * g.amount),
    );
    let col = Color::new(0.0, 0.0, 0.0, g.amount);
    draw_texture_ex(
        &art.dark,
        x0,
        y0,
        col,
        DrawTextureParams {
            dest_size: Some(vec2(s, s)),
            ..Default::default()
        },
    );
    draw_rectangle(l, tp, r - l, (y0 - tp).max(0.0), col);
    draw_rectangle(l, y0 + s, r - l, (b - (y0 + s)).max(0.0), col);
    draw_rectangle(l, y0, (x0 - l).max(0.0), s, col);
    draw_rectangle(x0 + s, y0, (r - (x0 + s)).max(0.0), s, col);
}

pub(super) fn draw_ambient(w: &World, cam: Vec2, glow: bool) {
    let deep = w.biome == Biome::Deep;
    if glow != deep || w.room.grid.is_none() {
        return;
    }
    if w.biome == Biome::Silk {
        draw_silk_fluff(w, cam);
        return;
    }
    if w.juice.petals && !glow {
        draw_wilted_petals(w, cam);
    }
    let (tw, th) = (RW + 48.0, RH + 48.0);
    let n = if deep { 18 } else { 22 };
    for i in 0..n {
        // 입자 효과 설정(0141): 그 비율만.
        if !keep_share(i as usize, w.juice.effects) {
            continue;
        }
        let (hx, hy, hs) = (hash01(i, 1), hash01(i, 2), hash01(i, 3));
        let t = w.time;
        let (drift, color, size) = match w.biome {
            // 꽃가루: 바람 따라 옆으로, 위아래로 살랑.
            Biome::Meadow => (
                vec2(6.0 + hs * 6.0, (t * (0.8 + hs) + hs * 6.0).sin() * 3.0),
                Color::new(1.0, 0.97, 0.85, 0.55),
                1.0,
            ),
            // 포자: 천천히 내리며 좌우로 흔들.
            Biome::Roots => (
                vec2((t * (0.6 + hs) + hs * 9.0).sin() * 4.0, 5.0 + hs * 4.0),
                Color::new(0.90, 0.85, 0.72, 0.45),
                1.0,
            ),
            // 흰 실 굴은 위에서 따로 그렸다(`draw_silk_fluff`).
            Biome::Silk => return,
            // 빛 알갱이: 아주 천천히 떠오른다.
            Biome::Deep => (
                vec2((t * 0.5 + hs * 7.0).sin() * 5.0, -3.0 - hs * 3.0),
                Color::new(
                    0.55,
                    0.80,
                    1.0,
                    0.35 + 0.25 * (t * 2.0 + hs * 9.0).sin().abs(),
                ),
                if hs > 0.7 { 2.0 } else { 1.0 },
            ),
        };
        // 꽃가루는 옆으로 흘러가고, 포자·빛 알갱이는 위아래로 흐른다(나머지 축은 살랑임).
        let flow = if w.biome == Biome::Meadow {
            vec2(drift.x * t, drift.y)
        } else {
            vec2(drift.x, drift.y * t)
        };
        let x = cam.x - 24.0 + (hx * tw + flow.x - cam.x).rem_euclid(tw);
        let y = cam.y - 24.0 + (hy * th + flow.y - cam.y).rem_euclid(th);
        if deep {
            draw_circle(
                x.round(),
                y.round(),
                3.0,
                Color::new(color.r, color.g, color.b, color.a * 0.3),
            );
        }
        draw_rectangle(x.round(), y.round(), size, size, color);
    }
}

/// 토끼풀의 충격파(0165): 퍼지는 두꺼운 흰 고리 + 안쪽 옅은 빛, 다 퍼지면 옅어진다. 뻗은 실 한 가닥도 여기서.
pub(super) fn draw_waves(w: &World) {
    for wv in &w.hazards.waves {
        let k = (wv.time / t::CLOVER_WAVE_TIME).min(1.0);
        let r = t::CLOVER_WAVE_R * k;
        let fade = 1.0 - ((wv.time - t::CLOVER_WAVE_TIME) / 0.25).clamp(0.0, 1.0);
        draw_circle(
            wv.pos.x.round(),
            wv.pos.y.round(),
            r,
            Color::new(0.92, 0.88, 1.0, 0.10 * fade),
        );
        draw_circle_lines(
            wv.pos.x.round(),
            wv.pos.y.round(),
            r,
            3.0,
            Color::new(0.96, 0.94, 1.0, 0.85 * fade),
        );
        draw_circle_lines(
            wv.pos.x.round(),
            wv.pos.y.round(),
            r - 3.0,
            1.0,
            Color::new(0.70, 0.55, 0.95, 0.6 * fade),
        );
    }
    if let Some((a, b, age)) = w.hazards.thread_lash {
        let k = 1.0 - (age / 0.3).clamp(0.0, 1.0);
        draw_line(
            a.x,
            a.y,
            b.x,
            b.y,
            3.0,
            Color::new(0.30, 0.15, 0.45, 0.5 * k),
        );
        draw_line(
            a.x,
            a.y,
            b.x,
            b.y,
            1.0,
            Color::new(0.96, 0.92, 1.0, 0.9 * k),
        );
    }
}

/// 아기들의 고치(0165): 흰 실에 감긴 타원, 숨 쉬듯 부풀고 분홍빛이 조금 비친다. 위에 작은 Z.
pub(super) fn draw_cocoon(c: Vec2, time: f32) {
    let (x, y) = (c.x.round(), c.y.round());
    let br = 1.0 + 0.04 * (time * 1.2 + c.x * 0.1).sin();
    draw_ellipse(x, y + 5.0, 10.0, 4.0, 0.0, SHADOW_SOFT);
    draw_ellipse(x, y, 10.0 * br, 8.0 * br, 0.0, OUTLINE);
    draw_ellipse(
        x,
        y,
        9.0 * br,
        7.0 * br,
        0.0,
        Color::new(0.90, 0.88, 0.96, 1.0),
    );
    draw_ellipse(
        x - 2.0,
        y - 2.0,
        5.0,
        3.0,
        0.0,
        Color::new(0.97, 0.95, 0.99, 1.0),
    );
    draw_ellipse(
        x + 1.0,
        y + 1.0,
        4.0,
        2.5,
        0.0,
        Color::new(0.98, 0.78, 0.84, 0.55),
    );
    for i in 0..3 {
        let ang = i as f32 * 1.1 + 0.4;
        draw_ellipse_lines(
            x + ang.cos() * 1.5,
            y + ang.sin(),
            7.5,
            4.0,
            ang * 57.3,
            1.0,
            Color::new(0.74, 0.66, 0.86, 0.8),
        );
    }
    if !shadow_pass() {
        let ph = (time * 0.7 + c.x * 0.01).fract();
        let z = vec2(x + 6.0 + ph * 3.0, y - 9.0 - ph * 6.0);
        let col = Color::new(0.95, 0.9, 1.0, 1.0 - ph);
        draw_line(z.x, z.y, z.x + 2.0, z.y, 1.0, col);
        draw_line(z.x + 2.0, z.y, z.x, z.y + 2.0, 1.0, col);
        draw_line(z.x, z.y + 2.0, z.x + 2.0, z.y + 2.0, 1.0, col);
    }
}

/// 흰 실 굴의 보풀(0165): 흰 실 보풀이 아주 천천히 떠오르며 옆으로 흔들린다.
pub(super) fn draw_silk_fluff(w: &World, cam: Vec2) {
    let (tw, th) = (RW + 48.0, RH + 48.0);
    let t = w.time;
    for i in 0..20u32 {
        if !keep_share(i as usize, w.juice.effects) {
            continue;
        }
        let (hx, hy, hs) = (hash01(i, 21), hash01(i, 22), hash01(i, 23));
        let flow = vec2(
            (t * (0.4 + hs * 0.5) + hs * 8.0).sin() * 5.0,
            -(2.0 + hs * 2.0) * t,
        );
        let x = (cam.x - 24.0 + (hx * tw + flow.x - cam.x).rem_euclid(tw)).round();
        let y = (cam.y - 24.0 + (hy * th + flow.y - cam.y).rem_euclid(th)).round();
        let a = 0.35 + 0.25 * (t * 1.5 + hs * 9.0).sin().abs();
        let c = Color::new(0.95, 0.92, 1.0, a);
        if hs > 0.6 {
            draw_rectangle(x - 1.0, y, 3.0, 1.0, c);
            draw_rectangle(x, y - 1.0, 1.0, 3.0, c);
        } else {
            draw_rectangle(x, y, 1.0, 1.0, c);
        }
    }
}

/// 흰 실 그물(0165): 바닥에 깔린 흰 실 — 가운데 매듭에서 뻗는 가닥 몇 개와 작은 고리. 가까우면 조금 밝다.
pub(super) fn draw_web(c: Vec2, time: f32, near: f32) {
    let seed = ((c.x * 11.0 + c.y * 7.0) as i32).rem_euclid(89) as f32;
    let (x, y) = (c.x.round(), c.y.round());
    let a = 0.55 + 0.25 * near;
    let silk = Color::new(0.94, 0.92, 0.99, a);
    let dim = Color::new(0.70, 0.64, 0.82, a * 0.7);
    draw_ellipse(
        x,
        y + 1.0,
        t::WEB_R + 1.0,
        (t::WEB_R + 1.0) * 0.6,
        0.0,
        Color::new(0.80, 0.74, 0.92, 0.12 + 0.08 * near),
    );
    for i in 0..6 {
        let ang = i as f32 * 1.05 + seed * 0.1 + (time * 0.7 + i as f32).sin() * 0.05;
        let len = t::WEB_R - 1.0 + ((seed + i as f32 * 5.0) % 3.0);
        let (ex, ey) = (x + ang.cos() * len, y + ang.sin() * len * 0.6);
        draw_line(x, y, ex, ey, 1.0, if i % 2 == 0 { silk } else { dim });
        draw_rectangle(ex.round(), ey.round(), 1.0, 1.0, silk);
    }
    draw_ellipse_lines(x, y, 5.0, 3.0, 0.0, 1.0, dim);
    draw_circle(x, y, 1.5, silk);
}

/// 시든 꽃잎(0159): 모모가 지키는 굴에서 방마다 떠도는 복선. 마을 빈 자리의 시든 꽃잎과 같은 색.
/// 옆으로 흘러가며 위아래로 팔랑이고, 흔들림에 따라 가로·세로로 바뀐다.
pub(super) fn draw_wilted_petals(w: &World, cam: Vec2) {
    let (tw, th) = (RW + 48.0, RH + 48.0);
    let t = w.time;
    let c = Color::new(0.74, 0.56, 0.60, 0.9);
    for i in 0..9u32 {
        if !keep_share(i as usize, w.juice.effects) {
            continue;
        }
        let (hx, hy, hs) = (hash01(i, 11), hash01(i, 12), hash01(i, 13));
        let ph = t * (1.6 + hs) + hs * 7.0;
        let flow = vec2(t * (7.0 + hs * 5.0), ph.sin() * 4.0 + t * 2.0);
        let x = (cam.x - 24.0 + (hx * tw + flow.x - cam.x).rem_euclid(tw)).round();
        let y = (cam.y - 24.0 + (hy * th + flow.y - cam.y).rem_euclid(th)).round();
        if ph.cos() > 0.0 {
            draw_rectangle(x - 1.0, y, 3.0, 1.0, c);
            draw_rectangle(x, y - 1.0, 1.0, 1.0, c);
        } else {
            draw_rectangle(x, y - 1.0, 1.0, 3.0, c);
            draw_rectangle(x - 1.0, y, 1.0, 1.0, c);
        }
    }
}

/// 도토가 묻은 도토리(0052): 흙 둔덕 + 삐죽 나온 갓. 터질 둘레를 고리로 보이고, 고리가 좁혀 오다 끝에선 깜박인다.
pub(super) fn draw_mine(m: &world::Mine, time: f32) {
    let k = (m.time / m.fuse.max(0.01)).clamp(0.0, 1.0);
    let (x, y) = (m.pos.x.round(), m.pos.y.round());
    let r = t::DOTO_MINE_RADIUS;
    let late = k > 0.7 && ((time * 16.0) as i32) % 2 == 0;
    let c = if late {
        Color::new(1.0, 0.85, 0.4, 0.85)
    } else {
        Color::new(1.0, 0.55, 0.4, 0.3 + 0.4 * k)
    };
    draw_circle_lines(x, y, r, 1.0, c);
    draw_circle_lines(x, y, (r * (1.0 - k)).max(2.0), 1.0, c);
    let swell = (k * 2.0).round();
    draw_ellipse(x, y + 1.0, 5.0 + swell, 2.5, 0.0, OUTLINE);
    draw_ellipse(x, y, 4.0 + swell, 2.0, 0.0, world::DIRT);
    // 갓(도토리 머리)
    let shake = if k > 0.5 && ((time * 40.0) as i32) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    draw_rectangle(
        x - 2.0 + shake,
        y - 2.0 - swell,
        4.0,
        2.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
    draw_rectangle(
        x + shake,
        y - 3.0 - swell,
        1.0,
        1.0,
        Color::new(0.42, 0.28, 0.18, 1.0),
    );
}

/// 공격 예고의 불투명도(시작, 공격 직전). 채움은 바닥이 비치게 옅게, 테두리가 주로 읽힌다.
/// 테두리는 밝은 산호색 + 하한 0.3(0150): 1·2형 색약·흑백 시뮬레이션에서 붉은 테두리(0.15~0.5)가 어두운 바닥에 묻혔다.
/// 색이 아니라 밝기로 읽히게 올리고, 채움은 그대로 옅게 둔다("너무 샛빨갛다", 2026-10-25).
pub(super) const TELEGRAPH_FILL: (f32, f32) = (0.03, 0.16);
pub(super) const TELEGRAPH_EDGE: (f32, f32) = (0.3, 0.65);
/// 못 튕겨내는 공격(둘레: 내려앉기·포자·돌): 진홍. 가운데 ✕도 함께(0160).
pub(super) const TELEGRAPH_EDGE_RGB: (f32, f32, f32) = (1.0, 0.32, 0.36);
pub(super) const TELEGRAPH_FILL_RGB: (f32, f32, f32) = (0.95, 0.25, 0.30);
/// 튕겨낼 수 있는 공격(곧게 오는 것: 돌진 띠·던지기·혀 점선): 황금. 진홍보다 밝아 색약·흑백에서도 갈린다(0160).
pub(super) const PARRY_EDGE_RGB: (f32, f32, f32) = (1.0, 0.88, 0.45);
pub(super) const PARRY_FILL_RGB: (f32, f32, f32) = (1.0, 0.80, 0.35);
/// 마지막 깜빡임은 거의 흰빛.
pub(super) const TELEGRAPH_BLINK: Color = Color::new(1.0, 0.95, 0.85, 0.9);

/// 못 튕겨내는 둘레 공격의 ✕(0160): 고리 가운데, 어두운 밑띠 위에 테두리 색 2px.
pub(super) fn draw_no_parry_mark(at: Vec2, edge: Color) {
    let (x, y) = (at.x.round(), at.y.round());
    let rim = Color::new(0.1, 0.02, 0.04, edge.a * 0.5);
    for d in -4..=4 {
        let d = d as f32;
        draw_rectangle(x + d - 1.0, y + d - 1.0, 4.0, 4.0, rim);
        draw_rectangle(x + d - 1.0, y - d - 1.0, 4.0, 4.0, rim);
    }
    for d in -4..=4 {
        let d = d as f32;
        draw_rectangle(x + d, y + d, 2.0, 2.0, edge);
        draw_rectangle(x + d, y - d, 2.0, 2.0, edge);
    }
}

/// 못 튕겨내는 고리(0160): 어두운 밑띠 위에 진홍 1px — 풀밭에서 갈색으로 묻히지 않게.
pub(super) fn draw_no_parry_ring(at: Vec2, r: f32, edge: Color) {
    draw_circle_lines(
        at.x,
        at.y,
        r,
        3.0,
        Color::new(0.1, 0.02, 0.04, edge.a * 0.45),
    );
    draw_circle_lines(at.x, at.y, r, 1.0, edge);
}

/// 차오름의 앞줄(0179): 띠를 가로지르는 밝은 줄(어두운 밑띠 위). `side`는 띠 반폭 벡터.
fn draw_fuse_front(f: Vec2, side: Vec2) {
    let (p, q) = (f + side, f - side);
    draw_line(p.x, p.y, q.x, q.y, 4.0, Color::new(0.1, 0.05, 0.02, 0.55));
    draw_line(p.x, p.y, q.x, q.y, 2.0, TELEGRAPH_BLINK);
}

/// 둘레 공격의 차오름(0179): 가운데부터 반지름 `r`까지 채운 원(테두리 색).
fn draw_fill_disk(at: Vec2, r: f32, edge: Color) {
    if r > 0.5 {
        draw_circle(at.x, at.y, r, Color::new(edge.r, edge.g, edge.b, 0.28));
    }
}

/// 점선 예고(0179): 3px 칠하고 2px 비움. 적 쪽부터 `k`만큼은 밝게 차오르고(어두운 밑띠 위) 그 앞은 밑띠 없이 옅게 —
/// 밝기 차로 "어디까지 왔나"가 읽힌다. 앞끝에 밝은 점.
fn draw_dots(start: Vec2, dir: Vec2, len: f32, k: f32, edge: Color) {
    let lit = len * k;
    let mut d = 0.0;
    while d < len {
        let p0 = start + dir * d;
        let p1 = start + dir * (d + 3.0).min(len);
        if d < lit {
            let rim = Color::new(0.1, 0.05, 0.02, 0.45);
            draw_line(p0.x, p0.y, p1.x, p1.y, 3.0, rim);
            let c = Color::new(edge.r, edge.g, edge.b, 1.0);
            draw_line(p0.x, p0.y, p1.x, p1.y, 1.0, c);
        } else {
            let fade = 1.0 - d / 150.0 * 0.5;
            let c = Color::new(edge.r, edge.g, edge.b, 0.28 * fade);
            draw_line(p0.x, p0.y, p1.x, p1.y, 1.0, c);
        }
        d += 5.0;
    }
    let f = start + dir * lit;
    let (x, y) = (f.x.round(), f.y.round());
    draw_rectangle(
        x - 2.0,
        y - 2.0,
        5.0,
        5.0,
        Color::new(0.1, 0.05, 0.02, 0.55),
    );
    draw_rectangle(x - 1.0, y - 1.0, 3.0, 3.0, TELEGRAPH_BLINK);
}

/// 쏘기·혀·실 가닥의 점선 예고(0179): 적 앞에서 `reach`까지(벽에서 끊김), 진행 `k`만큼 차오른다. 받아칠 수 있어 황금.
fn draw_dotted_fuse(w: &World, pos: Vec2, facing: Vec2, radius: f32, reach: f32, k: f32) {
    let dir = facing.normalize_or(vec2(1.0, 0.0));
    let start = pos + dir * (radius + 2.0);
    let len = w.room.ray_len(start, dir, t::PROJECTILE_RADIUS, reach);
    let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
    let edge = if blink {
        TELEGRAPH_BLINK
    } else {
        Color::new(
            PARRY_EDGE_RGB.0,
            PARRY_EDGE_RGB.1,
            PARRY_EDGE_RGB.2,
            TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k,
        )
    };
    draw_dots(start, dir, len, k, edge);
}

/// 공격 예고(바닥, 적·플레이어보다 먼저 그린다): 맞을 자리를 붉게, 공격 순간이 가까울수록 진하게.
/// - 돌격형(떨기): 돌진할 길(몸 폭 × 돌진 거리, 벽에서 끊김)을 띠로. 방향은 지금 보는 쪽(떨기 끝에 굳는다).
/// - 던지기(조준): 도토리가 날아갈 줄.
/// - 뛰기(웅크림·체공): 내려앉을 자리 고리, 체공 중엔 가득.
///
/// 진하기 k = 예고 경과 / 예고 시간(0→1). 마지막 15%는 깜빡여 "지금"을 알린다.
/// 언제(0179): 눈은 적이 아니라 공격이 올 길을 본다. 그래서 띠·점선은 적 쪽부터 빛이 차올라 끝에 닿는 순간 공격,
/// 둘레(내려앉기·포자·발 구르기)는 가운데부터 채워져 고리에 닿는 순간 공격. 차오른 끝에는 밝은 앞줄.
pub(super) fn draw_telegraphs(w: &World, alpha: f32) {
    use crate::species::Behavior;
    // 무너지는 천장(0147): 떨어질 자리에 그림자가 커지고, 붉은 고리가 조여 든다(청설모 내려앉기와 같은 말).
    for r in &w.hazards.rocks {
        let k = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        let rad = t::ROCK_R;
        draw_ellipse(
            r.pos.x,
            r.pos.y,
            rad * (0.35 + 0.65 * k),
            rad * (0.35 + 0.65 * k) * 0.7,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.18 + 0.27 * k),
        );
        let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
        let edge = if blink {
            TELEGRAPH_BLINK
        } else {
            Color::new(
                TELEGRAPH_EDGE_RGB.0,
                TELEGRAPH_EDGE_RGB.1,
                TELEGRAPH_EDGE_RGB.2,
                TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k,
            )
        };
        draw_no_parry_ring(r.pos, rad * (1.5 - 0.5 * k), edge);
        draw_no_parry_mark(r.pos, edge);
    }
    for e in w.enemies.iter().filter(|e| e.alive && !e.asleep) {
        let sp = e.sp();
        let pos = lerp(e.prev_pos, e.pos, alpha);
        // 진행은 적이 쓰는 식 그대로(`Enemy::telegraph`) — 그림과 판정이 어긋나지 않게. 체공 중엔 가득.
        let k = e.telegraph().unwrap_or(1.0);
        let kind = match e.state {
            enemy::State::Windup { .. }
                if matches!(
                    sp.behavior,
                    Behavior::Charger
                        | Behavior::Chief
                        | Behavior::Kongi
                        | Behavior::Darter
                        | Behavior::Roller
                        | Behavior::Swooper
                ) =>
            {
                0
            }
            enemy::State::Lash { time, dir } => {
                draw_tongue(w, pos, e.radius(), dir, time);
                continue;
            }
            enemy::State::Aim { .. } if sp.behavior == Behavior::Spore => 3,
            enemy::State::Aim { .. } => 1,
            enemy::State::Crouch { .. } => 2,
            enemy::State::Thump { .. } => 4,
            enemy::State::Gather { time } if time < t::CLOVER_THREAD_AT => {
                let k = (time / t::CLOVER_THREAD_AT).min(1.0);
                draw_dotted_fuse(w, pos, e.facing, e.radius(), 150.0, k);
                continue;
            }
            enemy::State::Leap { .. } => 2,
            _ => continue,
        };
        // 투명하게: 바닥이 비쳐 보여야 화면이 붉게 덮이지 않는다(사람 피드백 "너무 샛빨갛다", 2026-10-25).
        // 채움은 아주 옅게(최대 16%), 테두리가 주로 읽힌다. 깜빡임도 흰 번쩍이 아니라 조금 더 진한 붉은빛.
        let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
        let edge_a = TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k;
        // 띠·점선(곧게 오는 것)은 튕겨낼 수 있어 황금, 고리(둘레)는 못 튕겨 진홍(0160).
        let parry = kind == 0 || kind == 1;
        let (er, ef) = if parry {
            (PARRY_EDGE_RGB, PARRY_FILL_RGB)
        } else {
            (TELEGRAPH_EDGE_RGB, TELEGRAPH_FILL_RGB)
        };
        let edge = if blink {
            TELEGRAPH_BLINK
        } else {
            Color::new(er.0, er.1, er.2, edge_a)
        };
        let fill = Color::new(
            ef.0,
            ef.1,
            ef.2,
            TELEGRAPH_FILL.0 + (TELEGRAPH_FILL.1 - TELEGRAPH_FILL.0) * k,
        );
        match kind {
            0 => {
                let dir = e.facing.normalize_or(vec2(1.0, 0.0));
                let r = e.radius();
                let len = w.room.ray_len(pos, dir, r, sp.dash_speed * sp.dash_time);
                let side = vec2(-dir.y, dir.x) * r;
                let (a, b) = (pos + dir * r, pos + dir * (len + r));
                draw_triangle(a + side, a - side, b + side, fill);
                draw_triangle(a - side, b - side, b + side, fill);
                // 차오름(0179): 적 쪽부터 끝까지, 다 차면 돌진. 앞줄은 밝게 가로로.
                let f = a + (b - a) * k;
                let charged = Color::new(edge.r, edge.g, edge.b, 0.30 + 0.25 * k);
                draw_triangle(a + side, a - side, f + side, charged);
                draw_triangle(a - side, f - side, f + side, charged);
                draw_fuse_front(f, side);
                // 황금 테두리 밑에 어두운 띠(0160): 밝은 풀밭·색약·흑백에서도 선이 남는다.
                let tip = b + dir * 3.0;
                let rim = Color::new(0.1, 0.05, 0.02, edge.a * 0.45);
                for s in [side, -side] {
                    draw_line(a.x + s.x, a.y + s.y, b.x + s.x, b.y + s.y, 3.0, rim);
                    draw_line(b.x + s.x, b.y + s.y, tip.x, tip.y, 3.0, rim);
                }
                for s in [side, -side] {
                    draw_line(a.x + s.x, a.y + s.y, b.x + s.x, b.y + s.y, 1.0, edge);
                    draw_line(b.x + s.x, b.y + s.y, tip.x, tip.y, 1.0, edge);
                }
            }
            4 => {
                // 토끼풀 발 구르기(0165): 충격파가 닿을 둘레. 못 받아치니 진홍 고리 + ✕. 땅속으로.
                let r = t::CLOVER_WAVE_R;
                draw_circle(pos.x, pos.y, r, fill);
                draw_fill_disk(pos, r * k, edge);
                draw_no_parry_ring(pos, r, edge);
                draw_no_parry_mark(pos + vec2(0.0, -r - 6.0), edge);
            }
            3 => {
                // 포자버섯: 구름이 덮을 자리. 고리가 안에서 밖으로 부푼다.
                let r = t::SPORE_CLOUD_R;
                draw_circle(pos.x, pos.y, r, fill);
                draw_fill_disk(pos, r * k, edge);
                draw_no_parry_ring(pos, r, edge);
                draw_no_parry_mark(pos + vec2(0.0, -r - 6.0), edge);
            }
            1 => {
                // 두꺼비 혀는 닿는 데까지만, 도토리는 멀리.
                let reach = if sp.behavior == Behavior::Tongue {
                    t::TOAD_TONGUE_REACH
                } else {
                    150.0
                };
                draw_dotted_fuse(w, pos, e.facing, e.radius(), reach, k);
            }
            _ => {
                let at = match e.state {
                    enemy::State::Crouch { target, .. } => {
                        e.pos + (target - e.pos).clamp_length_max(sp.trigger * 1.2)
                    }
                    enemy::State::Leap { to, .. } => to,
                    _ => pos,
                };
                let r = sp.land_radius;
                draw_circle(at.x, at.y, r, fill);
                // 가운데부터 차오르고 고리는 바깥에서 조여 든다 — 둘이 만나는 순간 내려앉는다.
                draw_fill_disk(at, r * k, edge);
                draw_no_parry_ring(at, r * (1.5 - 0.5 * k), edge);
                let inner = Color::new(edge.r, edge.g, edge.b, edge.a * 0.5);
                draw_circle_lines(at.x, at.y, r, 1.0, inner);
                draw_no_parry_mark(at, edge);
            }
        }
    }
    // 가장 깊은 곳(0162): 할매의 당기기 예고 — 받아칠 수 있으니 황금 점선(어두운 밑띠).
    if let Some(p) = w.deep.as_ref().and_then(|d| {
        d.pull
            .filter(|p| p.time < t::DEEP_PULL_WINDUP)
            .map(|p| (d.center, p))
    }) {
        let (c, p) = p;
        let k = (p.time / t::DEEP_PULL_WINDUP).clamp(0.0, 1.0);
        let blink = k > 0.85 && ((w.time * 16.0) as i32) % 2 == 0;
        let a = TELEGRAPH_EDGE.0 + (TELEGRAPH_EDGE.1 - TELEGRAPH_EDGE.0) * k;
        let edge = if blink {
            TELEGRAPH_BLINK
        } else {
            Color::new(PARRY_EDGE_RGB.0, PARRY_EDGE_RGB.1, PARRY_EDGE_RGB.2, a)
        };
        draw_dots(c, p.dir, p.len, k, edge);
    }
}

/// 가시뿌리 덤불(0147·0149): 칸 가운데 낮은 덤불, 붉게 물든 가시가 사방으로 뻗는다. 발밑에 어두운 핏빛 얼룩이 숨 쉬고,
/// Naïl이 다가오면(`near` 0..1) 가시가 곤두서 길어진다. 끝이 가끔 하얗게 번득인다.
/// 예전(보라 가시 2px)은 "눈에 잘 띄지 않고 위험한 느낌이 없어"(사장).
pub(super) fn draw_thorn(c: Vec2, time: f32, near: f32) {
    let seed = ((c.x * 7.0 + c.y * 13.0) as i32).rem_euclid(97) as f32;
    let pulse = 0.5 + 0.5 * (time * 3.0 + seed).sin();
    draw_ellipse(
        c.x.round(),
        (c.y + 1.0).round(),
        9.0,
        6.0,
        0.0,
        Color::new(0.30, 0.02, 0.08, 0.30 + 0.15 * pulse + 0.15 * near),
    );
    let dark = Color::new(0.16, 0.05, 0.10, 1.0);
    let body = Color::new(0.34, 0.12, 0.20, 1.0);
    // 가시는 밝게(0150): 색약 시뮬레이션(1·2형)에서 붉은 가시가 몸과 같은 어둠으로 묻혔다. 밝기 차로 "뾰족한 것"이 읽히게.
    let spike = Color::new(1.0, 0.80, 0.84, 1.0);
    for (i, (dx, dy)) in [
        (-3.0, 1.0),
        (3.0, 1.5),
        (0.0, -1.5),
        (-1.0, 3.0),
        (2.0, -3.0),
    ]
    .into_iter()
    .enumerate()
    {
        let r = 2.5 + ((seed + i as f32 * 3.0) % 3.0) * 0.5;
        draw_circle((c.x + dx).round(), (c.y + dy).round(), r + 1.0, dark);
        draw_circle((c.x + dx).round(), (c.y + dy - 0.5).round(), r, body);
    }
    let grow = 3.0 + 2.0 * near;
    for i in 0..10 {
        let a = (seed * 0.37 + i as f32 * 0.63) % std::f32::consts::TAU;
        let d = Vec2::from_angle(a);
        let p = (c + vec2(d.x * 4.0, d.y * 3.0 - 1.0)).round();
        let q = (p + vec2(d.x, d.y * 0.8 - 0.4) * grow).round();
        draw_line(p.x, p.y, q.x, q.y, 1.0, spike);
        // 끝 번득임: 가시마다 차례로 잠깐.
        if (time * 2.0 + i as f32 * 0.31 + seed * 0.1).fract() < 0.08 {
            draw_rectangle(q.x, q.y, 1.0, 1.0, WHITE);
        }
    }
}

/// 무너지는 돌(0147): 떨어지기 전 반쯤부터 위에서 내려오는 돌(땅 그림자·고리는 `draw_telegraphs`).
pub(super) fn draw_falling_rocks(w: &World) {
    for r in &w.hazards.rocks {
        let k = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        if k < 0.45 {
            continue;
        }
        let h = (1.0 - k) * (1.0 - k) * 150.0;
        let p = vec2(r.pos.x, r.pos.y - 4.0 - h).round();
        draw_circle(p.x, p.y, 5.0, OUTLINE);
        draw_circle(p.x, p.y, 4.0, Color::new(0.46, 0.42, 0.44, 1.0));
        draw_rectangle(
            p.x - 2.0,
            p.y - 2.0,
            2.0,
            1.0,
            Color::new(0.68, 0.64, 0.64, 1.0),
        );
    }
}

/// 포자 구름(0146): 처음 잠깐(아프지 않을 때)은 옅게 부풀고, 끝에 흐려진다. 안의 점들이 천천히 돈다.
pub(super) fn draw_cloud(c: &world::Cloud, time: f32) {
    let grow = (c.time / t::SPORE_CLOUD_GRACE).min(1.0);
    let k = c.time / t::SPORE_CLOUD_TIME;
    let fade = 1.0 - ((k - 0.75) / 0.25).clamp(0.0, 1.0);
    let a = fade
        * if c.time < t::SPORE_CLOUD_GRACE {
            0.5
        } else {
            1.0
        };
    let r = t::SPORE_CLOUD_R * (0.55 + 0.45 * grow);
    for i in 0..5 {
        let ang = i as f32 * std::f32::consts::TAU / 5.0 + time * 0.5;
        let p = c.pos + vec2(ang.cos(), ang.sin() * 0.7) * r * 0.45;
        draw_circle(p.x, p.y, r * 0.55, Color::new(0.66, 0.56, 0.84, 0.16 * a));
    }
    draw_circle_lines(
        c.pos.x,
        c.pos.y,
        r,
        1.0,
        Color::new(0.85, 0.75, 1.0, 0.7 * a),
    );
    for i in 0..8 {
        let ph = (time * 0.4 + i as f32 * 0.37).fract();
        let ang = i as f32 * 2.4 + time * 0.8;
        let p = c.pos + vec2(ang.cos(), ang.sin() * 0.7) * r * (0.2 + 0.7 * ph);
        draw_rectangle(
            p.x.round(),
            (p.y - ph * 4.0).round(),
            1.0,
            1.0,
            Color::new(0.90, 0.85, 1.0, 0.7 * a * (1.0 - ph)),
        );
    }
}

pub(super) fn draw_fx(w: &World) {
    for f in &w.juice.fx {
        let p = (f.time / f.dur).clamp(0.0, 1.0);
        let pos = f.pos.round();
        match f.kind {
            FxKind::Ring { r0, r1, color } => {
                let r = r0 + (r1 - r0) * p;
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    r,
                    1.0,
                    Color::new(color.r, color.g, color.b, 1.0 - p),
                );
            }
            FxKind::Spark { color } => {
                let s = if p < 0.5 { 2.0 } else { 1.0 };
                draw_rectangle(
                    pos.x,
                    pos.y,
                    s,
                    s,
                    Color::new(color.r, color.g, color.b, 1.0 - p),
                );
            }
            // 먼지·연기: 정수 반지름, 천천히 옅어진다(분위기 효과는 대비를 낮게, 0081).
            FxKind::Puff { r0, r1, color } => {
                let r = (r0 + (r1 - r0) * p).round().max(1.0);
                let a = 0.55 * (1.0 - p) * (1.0 - p);
                draw_circle(pos.x, pos.y, r, Color::new(color.r, color.g, color.b, a));
            }
            // 흙덩이: 바닥 그림자 + 높이만큼 떠 있는 2px 덩이(아래 한 점 어둡게).
            FxKind::Clod { color, z, .. } => {
                let a = if p > 0.8 { (1.0 - p) * 5.0 } else { 1.0 };
                draw_rectangle(
                    pos.x,
                    pos.y + 1.0,
                    2.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.25 * a),
                );
                let y = (pos.y - z).round();
                draw_rectangle(pos.x, y, 2.0, 2.0, Color::new(color.r, color.g, color.b, a));
                draw_rectangle(
                    pos.x,
                    y + 1.0,
                    2.0,
                    1.0,
                    Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, a),
                );
            }
            // 반짝임: 십자가 커졌다 줄어든다(팔 길이 0→2→0, 가운데는 흰 점).
            FxKind::Glint { color } => {
                if f.time < 0.0 {
                    continue;
                }
                let arm = ((1.0 - (p * 2.0 - 1.0).abs()) * 2.5).round();
                let c = Color::new(color.r, color.g, color.b, 1.0 - p * 0.5);
                if arm > 0.0 {
                    draw_rectangle(pos.x - arm, pos.y, arm * 2.0 + 1.0, 1.0, c);
                    draw_rectangle(pos.x, pos.y - arm, 1.0, arm * 2.0 + 1.0, c);
                }
                draw_rectangle(pos.x, pos.y, 1.0, 1.0, WHITE);
            }
            // 꽃잎(0159): 2×2 점 둘이 흔들림에 따라 가로·세로로 바뀌며 팔랑인다.
            FxKind::Petal { color, phase } => {
                if f.time < 0.0 {
                    continue;
                }
                let a = (1.0 - p).min(1.0) * (0.5 + 0.5 * (p * 6.0).min(1.0));
                let c = Color::new(color.r, color.g, color.b, a);
                if phase.sin() > 0.0 {
                    draw_rectangle(pos.x - 1.0, pos.y, 3.0, 1.0, c);
                    draw_rectangle(pos.x, pos.y - 1.0, 1.0, 1.0, c);
                } else {
                    draw_rectangle(pos.x, pos.y - 1.0, 1.0, 3.0, c);
                    draw_rectangle(pos.x - 1.0, pos.y, 1.0, 1.0, c);
                }
            }
            // 풀려 흩어지는 실: 짧은 물결 선(점 몇 개), 끝으로 갈수록 옅다.
            FxKind::Thread { angle, len } => {
                let c = crate::world::THREAD;
                let d = vec2(angle.cos(), angle.sin());
                let n = vec2(-d.y, d.x);
                let n_px = len.round() as i32;
                for i in 0..n_px {
                    let t = i as f32;
                    let wave = (t * 0.9 + f.time * 9.0).sin() * 1.2;
                    let q = (pos + d * t + n * wave).round();
                    let a = (1.0 - p) * (1.0 - t / len * 0.6);
                    draw_rectangle(q.x, q.y, 1.0, 1.0, Color::new(c.r, c.g, c.b, a));
                }
            }
        }
    }
}

/// 뿌리 지대: 천장 틈으로 비스듬히 내려오는 빛줄기(더하기 빛)와 그 안의 먼지. 월드 칸(112px)에 붙어 있어
/// 카메라가 움직여도 제자리다. 벽 칸에는 내리지 않는다.
pub(super) fn draw_light_shafts(w: &World, cam: Vec2, k: f32) {
    if w.biome != Biome::Roots || w.juice.effects <= 0.0 {
        return;
    }
    const CELL: f32 = 112.0;
    let t = w.time;
    let (x0, y0) = (
        (cam.x / CELL).floor() as i32 - 1,
        (cam.y / CELL).floor() as i32 - 1,
    );
    for cy in y0..=y0 + (RH / CELL).ceil() as i32 + 2 {
        for cx in x0..=x0 + (RW / CELL).ceil() as i32 + 2 {
            let id = (cx as u32).wrapping_mul(73_856_093) ^ (cy as u32).wrapping_mul(19_349_663);
            if hash01(id, 51) > 0.38 {
                continue;
            }
            let p = vec2(
                cx as f32 * CELL + 16.0 + hash01(id, 52) * (CELL - 32.0),
                cy as f32 * CELL + 16.0 + hash01(id, 53) * (CELL - 32.0),
            )
            .round();
            if w.room.is_solid(p, 6.0) {
                continue;
            }
            // 위(왼쪽 위 천장 틈)에서 바닥 p로: 위는 좁고 아래는 넓은 사다리꼴. 아주 천천히 숨쉰다.
            let breathe = 0.75 + 0.25 * (t * 0.6 + hash01(id, 54) * 6.0).sin();
            let a = 0.11 * breathe * k * w.juice.effects.min(1.0);
            let top = p + vec2(-30.0, -78.0);
            let (tw, bw) = (4.0, 12.0);
            let warm = Color::new(1.0, 0.88, 0.62, a);
            draw_triangle(
                top - vec2(tw, 0.0),
                top + vec2(tw, 0.0),
                p + vec2(bw, 0.0),
                warm,
            );
            draw_triangle(
                top - vec2(tw, 0.0),
                p + vec2(bw, 0.0),
                p - vec2(bw, 0.0),
                warm,
            );
            // 안쪽 심(조금 더 밝게).
            let core = Color::new(1.0, 0.92, 0.70, a * 0.8);
            draw_triangle(
                top - vec2(1.5, 0.0),
                top + vec2(1.5, 0.0),
                p + vec2(5.0, 0.0),
                core,
            );
            draw_triangle(
                top - vec2(1.5, 0.0),
                p + vec2(5.0, 0.0),
                p - vec2(5.0, 0.0),
                core,
            );
            // 바닥에 닿은 빛 웅덩이(납작한 타원 = 원 둘을 겹쳐).
            let pool = Color::new(1.0, 0.86, 0.58, a * 1.6);
            draw_ellipse(p.x, p.y, 15.0, 5.0, 0.0, pool);
            // 빛줄기 안을 천천히 내려오는 먼지.
            for m in 0..4u32 {
                let ph = (t * (0.08 + 0.04 * hash01(id ^ m, 55)) + hash01(id ^ m, 56)).fract();
                let q = top + (p - top) * ph + vec2(((t * 0.9 + m as f32 * 2.1).sin()) * 2.0, 0.0);
                let tw = 0.5 + 0.5 * (t * 3.0 + m as f32).sin();
                draw_rectangle(
                    q.x.round(),
                    q.y.round(),
                    1.0,
                    1.0,
                    Color::new(1.0, 0.95, 0.80, (0.25 + 0.35 * tw) * k),
                );
            }
        }
    }
}

/// 위에서 떨어지는 것: 뿌리 지대는 흙 알갱이, 심층은 물방울. 바닥 그림자가 커지다 닿으면 부서지거나 퍼진다.
/// 상태 없이 시간으로 정한다(같은 번호·같은 주기 = 같은 자리).
pub(super) fn draw_overhead_falls(w: &World, cam: Vec2) {
    let drip = match w.biome {
        Biome::Roots | Biome::Silk => false,
        Biome::Deep => true,
        Biome::Meadow => return,
    };
    if w.room.grid.is_none() {
        return;
    }
    const FALL: f32 = 0.45;
    const LAND: f32 = 0.35;
    for i in 0..6u32 {
        if !keep_share(i as usize, w.juice.effects) {
            continue;
        }
        let period = 2.4 + hash01(i, 61) * 2.2;
        let tt = w.time + hash01(i, 62) * period;
        let cyc = (tt / period).floor() as u32;
        let since = tt - cyc as f32 * period;
        if since > FALL + LAND {
            continue;
        }
        let p = vec2(
            cam.x + 12.0 + hash01(i ^ cyc.wrapping_mul(7919), 63) * (RW - 24.0),
            cam.y + 24.0 + hash01(i ^ cyc.wrapping_mul(104_729), 64) * (RH - 40.0),
        )
        .round();
        if w.room.is_solid(p, 3.0) {
            continue;
        }
        if since < FALL {
            let f = since / FALL;
            let h = 46.0 * (1.0 - f * f);
            // 그림자: 떨어질수록 진하고 크다.
            draw_ellipse(
                p.x,
                p.y,
                1.0 + 2.0 * f,
                0.5 + 0.8 * f,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.25 * f),
            );
            let c = if drip {
                Color::new(0.70, 0.88, 1.0, 0.85)
            } else {
                Color::new(0.55, 0.42, 0.30, 0.9)
            };
            draw_rectangle(p.x, (p.y - h).round(), 1.0, if drip { 2.0 } else { 1.0 }, c);
        } else {
            let f = (since - FALL) / LAND;
            if drip {
                // 물방울이 퍼지는 고리.
                draw_ellipse_lines(
                    p.x,
                    p.y,
                    1.0 + 4.0 * f,
                    0.5 + 1.6 * f,
                    0.0,
                    1.0,
                    Color::new(0.70, 0.88, 1.0, 0.6 * (1.0 - f)),
                );
            } else {
                // 흙 알갱이가 두 조각으로 튄다.
                let c = Color::new(0.55, 0.42, 0.30, 0.8 * (1.0 - f));
                let dx = (1.0 + 3.0 * f).round();
                let dy = (-2.0 * f + 3.0 * f * f).round();
                draw_rectangle(p.x - dx, p.y + dy - 1.0, 1.0, 1.0, c);
                draw_rectangle(p.x + dx, p.y + dy - 1.0, 1.0, 1.0, c);
            }
        }
    }
}
