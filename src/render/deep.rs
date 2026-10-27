//! 가장 깊은 곳(0162): 할매·실·매듭.

use super::*;

/// 가장 깊은 곳(0162): 할매(가운데의 큰 균사 더미, 감은 눈, 작은 웃음) · 매듭 셋 · 할매와 매듭을 잇는 실 ·
/// 당기는 실(뻗을 때·감았을 때). 바닥 높이에 그린다(Naïl은 늘 아래쪽이라 앞에 온다).
pub(super) fn draw_deep(d: &world::Deep, w: &World, alpha: f32) {
    let t = w.time;
    let c = d.center;
    // 실: 가운데에서 안 끊긴 매듭으로, 물결치는 보라 선 + 흰 점.
    for k in d.knots.iter().filter(|k| !k.cut) {
        draw_thread(c, k.pos, t, 1.0);
    }
    // 당기는 실.
    if let Some(p) = d.pull {
        if p.time >= t::DEEP_PULL_WINDUP {
            let k = ((p.time - t::DEEP_PULL_WINDUP) / t::DEEP_PULL_TIME).clamp(0.0, 1.0);
            let reach = if p.snapped { 0.45 } else { 1.0 };
            let end = c + p.dir * p.len * (k * 4.0).min(1.0) * reach;
            draw_thread(c, end, t * 3.0, if p.snapped { 1.0 - k } else { 1.0 });
            if p.snapped {
                // 끊긴 실이 흩날린다.
                let q = end + p.dir * 6.0;
                draw_rectangle(
                    q.x.round(),
                    q.y.round(),
                    2.0,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 1.0 - k),
                );
            }
        }
    }
    if d.wrapped > 0.0 {
        let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
        draw_thread(c, pp, t * 2.0, 1.0);
        // 감긴 고리.
        for i in 0..3 {
            let y = pp.y - 4.0 + i as f32 * 4.0;
            draw_ellipse_lines(
                pp.x,
                y,
                7.0,
                2.0,
                0.0,
                1.0,
                Color::new(0.80, 0.62, 0.98, 0.9),
            );
        }
    }
    // 할매: 숨 쉬는 큰 더미. 놓아줄 때(0163)는 빛이 커지고 눈을 뜬다.
    let rel = d.done_t.map(|x| (x / 1.6).clamp(0.0, 1.0)).unwrap_or(0.0);
    let br = 1.0 + 0.02 * (t * 1.3).sin() + 0.03 * rel;
    let glow = 0.35 + 0.15 * (t * 0.9).sin() + 1.2 * rel;
    draw_circle(
        c.x,
        c.y + 4.0,
        52.0,
        Color::new(0.85, 0.80, 1.0, 0.08 * glow),
    );
    draw_ellipse(c.x, c.y + 12.0, 50.0 * br, 16.0, 0.0, SHADOW_SOFT);
    draw_ellipse(
        c.x,
        c.y + 2.0,
        48.0 * br,
        32.0 * br,
        0.0,
        Color::new(0.62, 0.52, 0.74, 1.0),
    );
    draw_ellipse(
        c.x,
        c.y,
        46.0 * br,
        30.0 * br,
        0.0,
        Color::new(0.90, 0.86, 0.96, 1.0),
    );
    draw_ellipse(
        c.x - 4.0,
        c.y - 8.0,
        30.0 * br,
        16.0,
        0.0,
        Color::new(0.97, 0.95, 0.99, 1.0),
    );
    // 버섯 갓 무리(위쪽 둘레).
    for (i, (dx, dy, r)) in [
        (-34.0, -14.0, 6.0),
        (-18.0, -24.0, 7.0),
        (4.0, -27.0, 8.0),
        (24.0, -22.0, 6.0),
        (38.0, -10.0, 5.0),
    ]
    .into_iter()
    .enumerate()
    {
        let bob = (t * 1.1 + i as f32).sin() * 0.8;
        let (x, y) = (c.x + dx, c.y + dy + bob);
        draw_rectangle(x - 1.0, y, 2.0, 5.0, Color::new(0.80, 0.74, 0.88, 1.0));
        draw_ellipse(
            x,
            y - 1.0,
            r,
            r * 0.55,
            0.0,
            Color::new(0.55, 0.40, 0.70, 1.0),
        );
        draw_ellipse(
            x - r * 0.25,
            y - 2.0,
            r * 0.45,
            r * 0.22,
            0.0,
            Color::new(0.78, 0.66, 0.92, 1.0),
        );
    }
    // 얼굴: 감은 눈(아래로 굽은 선 둘) + 작은 웃음. 아주 천천히 눈가가 움직인다.
    let eye = Color::new(0.40, 0.30, 0.50, 1.0);
    for sx in [-11.0, 11.0] {
        let (x, y) = ((c.x + sx).round(), (c.y - 2.0).round());
        if rel > 0.6 {
            // 뜬 눈: 둥근 동공 + 빛.
            draw_rectangle(x - 2.0, y - 2.0, 4.0, 4.0, eye);
            draw_rectangle(x - 1.0, y - 2.0, 1.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.9));
        } else {
            draw_rectangle(x - 3.0, y, 7.0, 1.0, eye);
            draw_rectangle(x - 4.0, y - 1.0, 1.0, 1.0, eye);
            draw_rectangle(x + 4.0, y - 1.0, 1.0, 1.0, eye);
        }
        // 볼 홍조.
        draw_rectangle(
            x - 2.0,
            y + 4.0,
            5.0,
            2.0,
            Color::new(1.0, 0.72, 0.78, 0.45),
        );
    }
    let (mx, my) = (c.x.round(), (c.y + 7.0).round());
    draw_rectangle(mx - 2.0, my, 5.0, 1.0, eye);
    draw_rectangle(mx - 3.0, my - 1.0, 1.0, 1.0, eye);
    draw_rectangle(mx + 3.0, my - 1.0, 1.0, 1.0, eye);
    // 매듭.
    for k in &d.knots {
        draw_knot(k, t);
    }
}

/// 물결치는 실(0162): 보라 선 위에 흰 점이 흐른다. `a`는 투명도.
pub(super) fn draw_thread(a_pt: Vec2, b_pt: Vec2, t: f32, a: f32) {
    let d = b_pt - a_pt;
    let len = d.length();
    if len < 1.0 {
        return;
    }
    let dir = d / len;
    let n = vec2(-dir.y, dir.x);
    let steps = (len / 3.0).ceil() as i32;
    let mut prev = a_pt;
    for i in 1..=steps {
        let k = i as f32 / steps as f32;
        let wave = (k * 9.0 + t * 2.5).sin() * 2.0 * (1.0 - (k * 2.0 - 1.0).abs() * 0.5);
        let q = a_pt + dir * (len * k) + n * wave;
        draw_line(
            prev.x,
            prev.y,
            q.x,
            q.y,
            1.0,
            Color::new(0.68, 0.45, 0.92, 0.9 * a),
        );
        prev = q;
    }
    // 흐르는 흰 점.
    for j in 0..3 {
        let k = (t * 0.35 + j as f32 / 3.0).fract();
        let wave = (k * 9.0 + t * 2.5).sin() * 2.0 * (1.0 - (k * 2.0 - 1.0).abs() * 0.5);
        let q = (a_pt + dir * (len * k) + n * wave).round();
        draw_rectangle(q.x, q.y, 1.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.9 * a));
    }
}

/// 실매듭(0162): 보라 실을 감은 덩이. 맞을수록 흰 틈이 벌어지고, 끊기면 풀린 실 몇 가닥만 남는다.
pub(super) fn draw_knot(k: &world::Knot, t: f32) {
    let jitter = if k.shake > 0.0 {
        ((t * 50.0) as i32 % 2) as f32 * 2.0 - 1.0
    } else {
        0.0
    };
    let (x, y) = (k.pos.x.round() + jitter, k.pos.y.round());
    if k.cut {
        let c = Color::new(0.85, 0.80, 0.92, 0.8);
        draw_ellipse(x, y + 3.0, 7.0, 2.5, 0.0, SHADOW_SOFT);
        for (dx, dy, w) in [(-6.0, 0.0, 5.0), (1.0, -2.0, 6.0), (-3.0, 3.0, 4.0)] {
            draw_rectangle(x + dx, y + dy, w, 1.0, c);
        }
        return;
    }
    let r = t::DEEP_KNOT_R;
    draw_ellipse(x, y + 4.0, r + 1.0, 4.0, 0.0, SHADOW_SOFT);
    draw_circle(x, y, r, OUTLINE);
    draw_circle(x, y, r - 1.0, Color::new(0.50, 0.32, 0.72, 1.0));
    // 감긴 실의 결.
    for i in 0..4 {
        let ang = i as f32 * 0.8 + 0.3;
        let (cx, cy) = (x + ang.cos() * 3.0, y + ang.sin() * 2.0);
        draw_ellipse_lines(
            cx,
            cy,
            r - 3.0,
            (r - 3.0) * 0.55,
            ang * 57.3,
            1.0,
            Color::new(0.74, 0.55, 0.95, 0.9),
        );
    }
    draw_circle(x - 3.0, y - 3.0, 2.0, Color::new(0.90, 0.80, 1.0, 0.9));
    // 벌어진 틈: 깎인 체력만큼 흰 줄.
    let worn = (t::DEEP_KNOT_HP - k.hp).max(0);
    for i in 0..worn {
        let ang = i as f32 * 1.1 + 0.6;
        let (ax, ay) = (x + ang.cos() * 5.0, y + ang.sin() * 5.0);
        draw_line(
            ax - 2.0,
            ay,
            ax + 2.0,
            ay + 1.0,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.85),
        );
    }
    // 가운데 어두운 심.
    draw_circle(x, y, 2.0, Color::new(0.20, 0.10, 0.28, 1.0));
}
