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
    draw_buried_at(m.pos, m.time, m.fuse, t::DOTO_MINE_RADIUS, false, time);
}

/// 플레이어 도토가 묻은 도토리(0195): 같은 그림, 고리는 따뜻한 금빛(적의 붉은 것과 다르다 — 내 것은 안 아프다).
pub(super) fn draw_buried(b: &world::toss::Buried, time: f32) {
    draw_buried_at(b.pos, b.time, b.fuse, b.radius, true, time);
}

fn draw_buried_at(pos: Vec2, age: f32, fuse: f32, r: f32, ours: bool, time: f32) {
    let k = (age / fuse.max(0.01)).clamp(0.0, 1.0);
    let (x, y) = (pos.x.round(), pos.y.round());
    let late = k > 0.7 && ((time * 16.0) as i32) % 2 == 0;
    let c = match (ours, late) {
        (false, true) => Color::new(1.0, 0.85, 0.4, 0.85),
        (false, false) => Color::new(1.0, 0.55, 0.4, 0.3 + 0.4 * k),
        (true, true) => Color::new(1.0, 0.95, 0.7, 0.8),
        (true, false) => Color::new(0.95, 0.78, 0.42, 0.25 + 0.35 * k),
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

/// 공격 예고(0183 → 0184 → 0185 → 0186): 맞을 자리를 **면**으로. 면 전체는 옅게(`TeleInk::base_a`) 깔리고, 테두리(`TELE_RING`
/// 폭의 띠)와 그 안에서 자라는 채움은 **같은 색·같은 투명도**(`TeleInk::a`)라 채움이 테두리 안쪽에 닿으면 한 덩어리가 된다 —
/// **그 순간이 공격**. 원은 가운데부터 바깥으로, 띠는 끝까지 깔린 가운데 줄에서 양옆으로 폭이 넓어진다(0185).
/// 테두리와 채움은 겹쳐 그리지 않는다(반투명이 겹치면 그 자리만 진해져 다른 색으로 보인다, 0186): 테두리는 겹치지 않는
/// 삼각형으로 짓고, 채움은 테두리 안쪽까지만. 어두운 밑띠도 뺐다 — 그게 "외곽선"으로 읽혀 채움과 색이 달라 보였다(사장, 0186).
/// 색은 붉은 계열 하나(0187). 못 받아치는 것(둘레)은 가운데 X(0160, X만 어두운 같은 색).
/// 투명도는 은은하게(0192 → 0196). 밝은 바닥(들판·뿌리)에서만 짙은 빨강으로 바꾼다(0197): 밝은 빨강을 얹으면 풀밭과
/// 밝기가 같아져 흑백·색약 화면에서 사라졌다(대비 1.03~1.10). 짙은 빨강은 바닥을 살짝 어둡게 해 1.26~1.40.
/// 어두운 바닥(심층·흰 실 굴)은 원래 빨강 그대로 — 이미 1.4쯤이고, 더 밝은 분홍은 색 차이(ΔE)가 반으로 줄어 더 흐렸다.
#[derive(Clone, Copy)]
pub(super) struct TeleInk {
    pub rgb: (f32, f32, f32),
    /// 테두리 = 다 찬 채움.
    pub a: f32,
    /// 빈 안쪽.
    pub base_a: f32,
}
/// 밝은 바닥: 짙은 빨강.
pub(super) const TELE_INK_SHADE: TeleInk = TeleInk {
    rgb: (0.55, 0.10, 0.12),
    a: 0.25,
    base_a: 0.03,
};
/// 어두운 바닥: 원래 빨강(0196, 안내 그림의 어두운 판에도).
pub(super) const TELE_INK_GLOW: TeleInk = TeleInk {
    rgb: (1.0, 0.32, 0.36),
    a: 0.25,
    base_a: 0.03,
};
thread_local! {
    /// 이번 그리기의 잉크(`draw_telegraphs`가 지대로 정한다). 안내 그림은 원래 빨강.
    static TELE_INK: std::cell::Cell<TeleInk> = const { std::cell::Cell::new(TELE_INK_GLOW) };
}
/// 지대 → 잉크(0197).
pub(super) fn tele_ink_for(b: crate::biome::Biome) -> TeleInk {
    use crate::biome::Biome;
    match b {
        Biome::Meadow | Biome::Roots => TELE_INK_SHADE,
        Biome::Deep | Biome::Silk => TELE_INK_GLOW,
    }
}
/// 테두리 띠의 폭(px).
const TELE_RING: f32 = 2.0;
const TELE_MARK_SHADE: f32 = 0.42;
const TELE_MARK_A: f32 = 0.5;

/// 테두리(= 다 찬 채움)·바탕의 색.
fn tele_colors() -> (Color, Color) {
    let ink = TELE_INK.with(|c| c.get());
    let c = |a| Color::new(ink.rgb.0, ink.rgb.1, ink.rgb.2, a);
    (c(ink.a), c(ink.base_a))
}

/// 띠의 바깥 둘레(뒤 왼쪽 → 앞 왼쪽 → 화살촉 → 앞 오른쪽 → 뒤 오른쪽). 반폭 `half`, 뒤끝을 `back`만큼 앞으로,
/// 화살촉 끝을 `nose`만큼 뒤로 당길 수 있다(안쪽 둘레용).
fn lane_outline(a: Vec2, b: Vec2, half: f32, tip_len: f32, back: f32, nose: f32) -> [Vec2; 5] {
    let dir = (b - a).normalize_or(vec2(1.0, 0.0));
    let side = vec2(-dir.y, dir.x) * half;
    let a = a + dir * back;
    let tip = b + dir * (tip_len - nose);
    [a + side, b + side, tip, b - side, a - side]
}

/// 볼록 다각형을 부채꼴 삼각형으로(겹침 없음).
fn fill_poly(p: &[Vec2], col: Color) {
    for i in 1..p.len() - 1 {
        draw_triangle(p[0], p[i], p[i + 1], col);
    }
}

/// 바깥 둘레 `o`와 안쪽 둘레 `n` 사이(닫힌 띠)를 겹치지 않는 사각형으로.
fn fill_ring(o: &[Vec2], n: &[Vec2], col: Color) {
    let m = o.len();
    for i in 0..m {
        let j = (i + 1) % m;
        draw_triangle(o[i], o[j], n[j], col);
        draw_triangle(o[i], n[j], n[i], col);
    }
}

/// 곧게 오는 공격의 면(0183 → 0186): `a`에서 `b`까지 반폭 `half`의 띠 + 끝의 화살촉. 채움은 끝까지 깔린 가운데 줄에서
/// 양옆으로 폭이 넓어져 테두리 안쪽에 닿으면(테두리와 한 덩어리) 공격. 돌진 띠·던지기·혀·실 가닥·할매 당기기.
pub(super) fn draw_tele_lane(a: Vec2, b: Vec2, half: f32, k: f32) {
    let (edge, base) = tele_colors();
    let tip_len = half + 2.0;
    let ring = TELE_RING.min(half * 0.5);
    // 화살촉 비스듬한 변을 `ring`만큼 들이려면 끝점을 축 방향으로 ring / sin(반각)만큼 당긴다.
    let nose = ring * (half * half + tip_len * tip_len).sqrt() / half;
    let outer = lane_outline(a, b, half, tip_len, 0.0, 0.0);
    let inner_half = half - ring;
    let inner = lane_outline(a, b, inner_half, tip_len - ring, ring, nose - ring);
    fill_poly(&outer, base);
    fill_ring(&outer, &inner, edge);
    let w = inner_half * k.clamp(0.0, 1.0);
    if w > 0.25 {
        fill_poly(
            &lane_outline(a, b, w, tip_len - ring, ring, nose - ring),
            edge,
        );
    }
}

/// 원 둘레 점(바깥·안쪽 띠를 같은 각도로 나눠 겹치지 않게).
fn circle_pts(at: Vec2, r: f32, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            at + vec2(t.cos(), t.sin()) * r
        })
        .collect()
}

/// 둘레 공격의 면(0183 → 0186): 반지름 `r` 원 + 테두리 + 가운데 X(못 받아친다). 테두리와 같은 색의 원이 가운데부터 자라
/// 테두리 안쪽에 닿으면 공격. 내려앉기·포자·발 구르기·떨어지는 돌.
pub(super) fn draw_tele_disk(at: Vec2, r: f32, k: f32) {
    let (edge, base) = tele_colors();
    let n = ((r * 3.0) as usize).clamp(24, 96);
    let outer = circle_pts(at, r, n);
    let inner_r = (r - TELE_RING).max(0.5);
    let inner = circle_pts(at, inner_r, n);
    fill_poly(&outer, base);
    fill_ring(&outer, &inner, edge);
    let g = inner_r * k.clamp(0.0, 1.0);
    if g > 0.5 {
        if k >= 1.0 {
            fill_poly(&inner, edge);
        } else {
            fill_poly(&circle_pts(at, g, n), edge);
        }
    }
    draw_no_parry_mark(at, edge);
}

/// 못 튕겨내는 둘레 공격의 X(0160): 가운데, 테두리와 같은 색의 어두운 톤 2px(점을 겹치지 않게 한 번씩).
pub(super) fn draw_no_parry_mark(at: Vec2, edge: Color) {
    let (x, y) = (at.x.round() as i32, at.y.round() as i32);
    let k = TELE_MARK_SHADE;
    let col = Color::new(edge.r * k, edge.g * k, edge.b * k, TELE_MARK_A);
    let mut pts: Vec<(i32, i32)> = Vec::with_capacity(40);
    for d in -4..=4 {
        for (px, py) in [(d, d), (d + 1, d), (d, -d), (d + 1, -d)] {
            if !pts.contains(&(px, py)) {
                pts.push((px, py));
            }
        }
    }
    for (px, py) in pts {
        draw_rectangle((x + px) as f32, (y + py) as f32, 1.0, 1.0, col);
    }
}

/// 던지기·혀·실 가닥의 띠(0183): 적 앞에서 `reach`까지(벽에서 끊김), 탄 폭만큼. 받아칠 수 있다(띠 모양).
fn draw_aim_lane(w: &World, pos: Vec2, facing: Vec2, radius: f32, reach: f32, k: f32) {
    let dir = facing.normalize_or(vec2(1.0, 0.0));
    let start = pos + dir * (radius + 2.0);
    let len = w.room.ray_len(start, dir, t::PROJECTILE_RADIUS, reach);
    draw_tele_lane(start, start + dir * len, t::PROJECTILE_RADIUS + 1.5, k);
}

/// 공격 예고(바닥, 적·플레이어보다 먼저 그린다): 맞을 자리를 면으로, 안쪽이 진해져 테두리와 같아지는 순간 공격(0183).
/// - 돌격형(떨기): 돌진할 길(몸 폭 × 돌진 거리, 벽에서 끊김). 방향은 지금 보는 쪽(떨기 끝에 굳는다).
/// - 던지기(조준)·혀·실 가닥: 날아갈 길의 좁은 띠.
/// - 뛰기(웅크림·체공)·포자·발 구르기·돌: 내려앉을·덮을 자리의 원, 체공 중엔 가득.
pub(super) fn draw_telegraphs(w: &World, alpha: f32) {
    use crate::species::Behavior;
    TELE_INK.with(|c| c.set(tele_ink_for(w.biome)));
    // 무너지는 천장(0147): 떨어질 자리에 그림자가 커지고, 원 안쪽이 진해진다.
    for r in &w.hazards.rocks {
        let k = crate::enemy::tele_progress(r.time, t::ROCK_FALL);
        let rad = t::ROCK_R;
        draw_ellipse(
            r.pos.x,
            r.pos.y,
            rad * (0.35 + 0.65 * k),
            rad * (0.35 + 0.65 * k) * 0.7,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.18 + 0.27 * k),
        );
        draw_tele_disk(r.pos, rad, k);
    }
    for e in w.enemies.iter().filter(|e| e.alive && !e.asleep) {
        let sp = e.sp();
        let pos = lerp(e.prev_pos, e.pos, alpha);
        // 진행은 적이 쓰는 식 그대로(`Enemy::telegraph`) — 그림과 판정이 어긋나지 않게. 체공 중엔 가득.
        let k = e.telegraph().unwrap_or(1.0);
        match e.state {
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
                let dir = e.facing.normalize_or(vec2(1.0, 0.0));
                let r = e.radius();
                let len = w.room.ray_len(pos, dir, r, sp.dash_speed * sp.dash_time);
                draw_tele_lane(pos + dir * r, pos + dir * (len + r), r, k);
            }
            enemy::State::Lash { time, dir } => {
                actors::draw_tongue(w, pos, e.radius(), dir, time);
            }
            enemy::State::Aim { .. } if sp.behavior == Behavior::Spore => {
                // 포자버섯: 구름이 덮을 자리.
                draw_tele_disk(pos, t::SPORE_CLOUD_R, k);
            }
            enemy::State::Aim { .. } => {
                // 두꺼비 혀는 닿는 데까지만, 도토리는 멀리.
                let reach = if sp.behavior == Behavior::Tongue {
                    t::TOAD_TONGUE_REACH
                } else {
                    150.0
                };
                draw_aim_lane(w, pos, e.facing, e.radius(), reach, k);
            }
            enemy::State::Thump { .. } => {
                // 토끼풀 발 구르기(0165): 충격파가 닿을 둘레.
                draw_tele_disk(pos, t::CLOVER_WAVE_R, k);
            }
            enemy::State::Gather { time } if time < t::CLOVER_THREAD_AT => {
                let k = (time / t::CLOVER_THREAD_AT).min(1.0);
                draw_aim_lane(w, pos, e.facing, e.radius(), 150.0, k);
            }
            enemy::State::Crouch { target, .. } => {
                let at = e.pos + (target - e.pos).clamp_length_max(sp.trigger * 1.2);
                draw_tele_disk(at, sp.land_radius, k);
            }
            enemy::State::Leap { to, .. } => {
                draw_tele_disk(to, sp.land_radius, 1.0);
            }
            _ => {}
        }
    }
    // 가장 깊은 곳(0162): 할매의 당기기 예고 — 받아칠 수 있으니 띠.
    if let Some((c, p)) = w.deep.as_ref().and_then(|d| {
        d.pull
            .filter(|p| p.time < t::DEEP_PULL_WINDUP)
            .map(|p| (d.center, p))
    }) {
        let k = crate::enemy::tele_progress(p.time, t::DEEP_PULL_WINDUP);
        draw_tele_lane(c, c + p.dir * p.len, 2.5, k);
    }
    TELE_INK.with(|c| c.set(TELE_INK_GLOW));
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
