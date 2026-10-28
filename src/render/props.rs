//! 장소의 것: 바닥·격자 그림, 장식, 뿌리 문, 저장 지점, 출구, 샘, 상자, 모닥불·가게·자리·웅덩이·꿈 고리, 큰 나무, 떨어진 것·획득물, 빛 버섯, 어둠 덮개.

use super::*;

pub(super) fn draw_room(w: &World) {
    for gx in (0..VW).step_by(16) {
        draw_line(gx as f32, 0.0, gx as f32, t::VIEW_H, 1.0, GRID);
    }
    for gy in (0..VH).step_by(16) {
        draw_line(0.0, gy as f32, t::VIEW_W, gy as f32, 1.0, GRID);
    }
    let r = w.room.inner;
    draw_rectangle(0.0, 0.0, t::VIEW_W, r.y, WALL_COLOR);
    draw_rectangle(
        0.0,
        r.bottom(),
        t::VIEW_W,
        t::VIEW_H - r.bottom(),
        WALL_COLOR,
    );
    draw_rectangle(0.0, 0.0, r.x, t::VIEW_H, WALL_COLOR);
    draw_rectangle(r.right(), 0.0, t::VIEW_W - r.right(), t::VIEW_H, WALL_COLOR);
    for p in &w.room.solids {
        draw_rectangle(p.x, p.y, p.w, p.h, WALL_COLOR);
        draw_rectangle(p.x + 2.0, p.y + 2.0, p.w - 4.0, p.h - 4.0, world::DIRT);
    }
}

pub(super) fn tile_src(col: u32, row: u32) -> Rect {
    art::cell(col, row)
}

pub(super) fn draw_tile(tex: &Texture2D, src: Rect, x: f32, y: f32, tint: Color) {
    draw_texture_ex(
        tex,
        x,
        y,
        tint,
        DrawTextureParams {
            source: Some(art::inset(src)),
            dest_size: Some(vec2(src.w.round(), src.h.round())),
            ..Default::default()
        },
    );
}

pub(super) fn decor_src(d: Decor) -> Rect {
    match d {
        Decor::FlowerWhite => tile_src(4, 0),
        Decor::FlowerYellow => tile_src(5, 0),
        Decor::FlowerPink => tile_src(6, 0),
        Decor::Tuft => tile_src(7, 0),
        Decor::Pebble => tile_src(3, 1),
        Decor::Mushroom => tile_src(4, 1),
        Decor::RootVine => tile_src(5, 1),
        Decor::Moss => tile_src(6, 1),
    }
}

/// 바닥 칸마다 (x, y, 장식). 방 그림은 칸 해시로 정해져 프레임마다 같다. 격자 방은 보이는 바닥 칸만.
pub(super) fn floor_decor(w: &World, cam: Vec2) -> Vec<(f32, f32, Decor)> {
    let mut out = Vec::new();
    let (cx0, cy0, cx1, cy1) = match &w.room.grid {
        Some(_) => {
            let (x0, y0) = (
                (cam.x / 16.0).floor() as i32 - 1,
                (cam.y / 16.0).floor() as i32 - 1,
            );
            (x0, y0, x0 + 23, y0 + 14)
        }
        None => {
            let r = w.room.inner;
            (
                (r.x / 16.0).floor() as i32,
                (r.y / 16.0).floor() as i32,
                (r.right() / 16.0).ceil() as i32,
                (r.bottom() / 16.0).ceil() as i32,
            )
        }
    };
    for cy in cy0..cy1 {
        for cx in cx0..cx1 {
            if let Some(g) = &w.room.grid {
                if g.get_i(cx, cy) != Cell::Floor {
                    continue;
                }
            }
            // 구운 지도와 같은 자리(무리 밀도·어긋남, terrain.rs).
            if let Some((d, ox, oy)) = crate::terrain::decor_at(w.biome, cx, cy) {
                out.push((cx as f32 * 16.0 + ox, cy as f32 * 16.0 + oy, d));
            }
        }
    }
    out
}

/// 잠긴 문: 문을 막고 보라 실 두 가닥이 감는다(이야기의 "실", 0037).
/// 팩이 있으면 팩의 나무 울타리(남북 문은 가로 울타리, 동서 문은 세로 울타리, 0097), 없으면 물든 뿌리 세 줄기(0094).
/// 잠그면 문이 바닥에서 솟고(뿌리는 문틀에서 자라고, `grow` 0→1) 실이 나중에 감긴다. 열면 거꾸로 걷히고 월드가 풀리는 실을 낸다.
/// 무늬는 월드 좌표로 정해 두 칸짜리 문이 이어져 보인다.
/// `glow`면 더하기 빛 패스용: 문은 빼고 실과 문틈 빛만(어두운 심층에서도 문이 보이게).
pub(super) fn draw_root_gate(
    g: &crate::map::Grid,
    cx: i32,
    cy: i32,
    grow: f32,
    time: f32,
    glow: bool,
    sheet: Option<&Texture2D>,
) {
    let blocked = |c: Cell| matches!(c, Cell::Wall | Cell::Door);
    // 좌우가 막혀 있으면 위아래로 지나는 문(남북) → 세로 뿌리.
    let vertical = blocked(g.get_i(cx - 1, cy)) && blocked(g.get_i(cx + 1, cy));
    let (x0, y0) = (cx as f32 * 16.0, cy as f32 * 16.0);
    let root_dark = Color::new(0.30, 0.20, 0.16, 1.0);
    let root = Color::new(0.45, 0.31, 0.22, 1.0);
    let root_hi = Color::new(0.62, 0.46, 0.32, 1.0);
    let th = world::THREAD;
    let pulse = 0.82 + 0.18 * (time * 3.0).sin();
    // (along, across) → 월드 px. along = 뿌리가 뻗는 방향, across = 가로지르는 방향.
    let at = |along: f32, across: f32| -> Vec2 {
        if vertical {
            vec2(x0 + across, y0 + along)
        } else {
            vec2(x0 + along, y0 + across)
        }
    };
    let px = |q: Vec2, c: Color| draw_rectangle(q.x.round(), q.y.round(), 1.0, 1.0, c);
    let (base_along, base_across) = if vertical { (y0, x0) } else { (x0, y0) };
    if glow {
        let c = at(8.0, 8.0);
        draw_circle(
            c.x,
            c.y,
            11.0,
            Color::new(th.r, th.g, th.b, 0.10 * grow * pulse),
        );
    }
    // 문틈 뒤 보라 아지랑이(잠긴 동안 숨 쉰다). 가는 울타리 뒤에서는 네모로 보여 뿌리일 때만.
    if !glow && sheet.is_none() {
        draw_rectangle(
            x0,
            y0,
            16.0,
            16.0,
            Color::new(th.r * 0.5, th.g * 0.3, th.b * 0.6, 0.22 * grow * pulse),
        );
    }
    if let Some(sheet) = sheet {
        // 팩 울타리: 앞쪽 절반 동안 바닥에서 솟는다(윗줄부터 보인다).
        if !glow {
            let rise = (grow / 0.55).clamp(0.0, 1.0);
            let h = (rise * 16.0).round();
            if h > 0.0 {
                // 울타리 두 겹을 엇갈려 쌓는다(뒤 겹부터). 가로는 위아래로, 세로는 좌우로 민다.
                let r = sunny::rect(if vertical {
                    sunny::FENCE_H
                } else {
                    sunny::FENCE_V
                });
                for d in sunny::FENCE_STACK {
                    let (dx, dy) = if vertical { (0.0, d) } else { (d, 0.0) };
                    draw_texture_ex(
                        sheet,
                        x0 + dx,
                        y0 + 16.0 - h + dy,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(r.x, r.y, r.w, h)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    } else {
        let reach = (grow * 16.0).ceil() as i32;
        // 뿌리 세 줄기, 몸 3px(밝은 쪽·몸·그늘) + 외곽선. 빛 패스에서는 밝은 쪽에 흐린 빛만 더한다.
        for lane in [2.0_f32, 7.0, 12.0] {
            for r in 0..reach.min(16) {
                let wa = base_along + r as f32;
                let wob = ((wa * 0.45 + (base_across + lane) * 0.3).sin() * 1.2).round();
                let c = lane + wob;
                let rf = r as f32;
                if glow {
                    px(at(rf, c), Color::new(0.55, 0.35, 0.25, 0.18 * grow));
                    continue;
                }
                px(at(rf, c - 1.0), OUTLINE);
                px(at(rf, c), if (wa as i32) % 3 == 0 { root_hi } else { root });
                px(at(rf, c + 1.0), root);
                px(at(rf, c + 2.0), root_dark);
                px(at(rf, c + 3.0), OUTLINE);
            }
        }
    }
    // 실: 문(뿌리)이 거의 다 선 뒤 감긴다. 실은 문을 가로지르는 방향(across)으로 간다.
    let ta = ((grow - 0.55) / 0.45).clamp(0.0, 1.0);
    if ta > 0.0 {
        // 뿌리: 두 가닥이 따로 물결친다. 울타리: 한 가닥이 가로대를 사선으로 감는다(앞을 지나는 반만 보인다).
        let strands: &[(f32, f32, f32)] = if sheet.is_some() {
            let c = if vertical {
                sunny::FENCE_RAIL_Y
            } else {
                sunny::FENCE_RAIL_X
            };
            &[(c, sunny::FENCE_WRAP, 0.0)]
        } else {
            &[(4.0, 1.1, 4.0), (11.0, 1.1, 11.0)]
        };
        let fence = sheet.is_some();
        let freq = if fence { 0.6 } else { 0.55 };
        for &(lane, amp, phase) in strands {
            for k in 0..16 {
                let wc = base_across + k as f32;
                let ph = wc * freq + time * 2.4 + phase;
                // 울타리를 감는 실은 뒤로 도는 반(cos < 0)이 가로대에 가려진다.
                if fence && ph.cos() < 0.0 {
                    continue;
                }
                let a = lane + (ph.sin() * amp).round();
                let kf = k as f32;
                if glow {
                    px(at(a, kf), Color::new(th.r, th.g, th.b, 0.55 * ta * pulse));
                    px(
                        at(a + 1.0, kf),
                        Color::new(th.r, th.g, th.b, 0.25 * ta * pulse),
                    );
                } else {
                    px(at(a + 1.0, kf), Color::new(0.34, 0.20, 0.46, ta));
                    px(at(a, kf), Color::new(th.r * pulse, th.g * pulse, th.b, ta));
                }
            }
        }
    }
}

/// 격자 방(필드·던전) 그림: 구운 지도(terrain.rs, 0062)의 보이는 부분 + 매 프레임 바뀌는 칸.
/// 특별한 칸: 문(열림 = 바닥, 잠김 = 나무 울타리(팩 없으면 물든 뿌리) + 보라 실, 0094·0097), 굴 입구, 출구, 저장 지점(빛나는 버섯 고리).
pub(super) fn draw_grid_art(w: &World, cam: Vec2, baked: &Texture2D, sheet: Option<&Texture2D>) {
    let Some(g) = &w.room.grid else { return };
    let view = Rect::new(cam.x.floor() - 1.0, cam.y.floor() - 1.0, RW + 3.0, RH + 3.0);
    let bounds = Rect::new(0.0, 0.0, baked.width(), baked.height());
    if let Some(src) = view.intersect(bounds) {
        draw_texture_ex(
            baked,
            src.x,
            src.y,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                ..Default::default()
            },
        );
    }
    let (x0, y0) = (
        (cam.x / 16.0).floor() as i32 - 1,
        (cam.y / 16.0).floor() as i32 - 1,
    );
    let (x1, y1) = (x0 + 23, y0 + 14);
    // 특별한 칸
    for cy in y0..y1 {
        for cx in x0..x1 {
            let c = g.get_i(cx, cy);
            let p = vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0);
            match c {
                Cell::Door if w.room.door_anim > 0.0 => {
                    draw_root_gate(g, cx, cy, w.room.door_anim, w.time, false, sheet);
                }
                Cell::Entrance => {
                    draw_ellipse(p.x, p.y + 2.0, 12.0, 7.0, 0.0, world::DIRT);
                    draw_ellipse(
                        p.x,
                        p.y + 2.0,
                        9.0,
                        5.0,
                        0.0,
                        Color::new(0.05, 0.03, 0.06, 1.0),
                    );
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        6.0,
                        3.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 1.0),
                    );
                }
                // 출구(0159): 길의 굴에서는 더 깊이 파 내려가는 구멍, 필드의 옛 굴에서는 올라가는 구멍.
                Cell::Exit => draw_exit(p, w.time, w.adv.node.is_some()),
                Cell::Gate => {
                    // 뿌리 길: 뿌리가 감싼 큰 구멍 + 팻말.
                    let root = Color::new(0.42, 0.30, 0.24, 1.0);
                    draw_ellipse(p.x, p.y + 2.0, 15.0, 8.0, 0.0, OUTLINE);
                    draw_ellipse(p.x, p.y + 2.0, 13.0, 6.5, 0.0, world::DIRT);
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        10.0,
                        4.5,
                        0.0,
                        Color::new(0.03, 0.02, 0.04, 1.0),
                    );
                    for k in 0..4 {
                        let a = std::f32::consts::PI * (0.1 + k as f32 * 0.27);
                        let q = p + vec2(a.cos() * 13.0, 2.0 - a.sin() * 6.5);
                        draw_line(q.x, q.y, q.x + a.cos() * 4.0, q.y - 3.0, 2.0, root);
                    }
                    let pulse = 0.4 + 0.3 * (w.time * 2.0).sin().abs();
                    draw_ellipse(
                        p.x,
                        p.y + 3.0,
                        4.0,
                        1.5,
                        0.0,
                        Color::new(0.7, 0.9, 1.0, pulse * 0.4),
                    );
                    // 팻말(왼쪽 위)
                    let s = p + vec2(-17.0, -10.0);
                    draw_rectangle(s.x, s.y + 4.0, 1.0, 9.0, root);
                    draw_rectangle(s.x - 4.0, s.y - 1.0, 10.0, 6.0, OUTLINE);
                    draw_rectangle(s.x - 3.0, s.y, 8.0, 4.0, Color::new(0.66, 0.50, 0.32, 1.0));
                    draw_triangle(
                        vec2(s.x + 6.0, s.y - 1.0),
                        vec2(s.x + 6.0, s.y + 5.0),
                        vec2(s.x + 9.0, s.y + 2.0),
                        OUTLINE,
                    );
                }
                Cell::Save => draw_save_ring(p, w.time),
                _ => {}
            }
        }
    }
}

/// 저장 지점: 빛나는 버섯 고리(버섯만, 빛은 `draw_save_lights`). 고리 자국은 짙게(실제 균환처럼).
pub(super) fn draw_save_ring(p: Vec2, time: f32) {
    let pulse = 0.5 + 0.5 * (time * 2.5).sin();
    let cap = Color::new(1.0, 0.97, 0.80, 1.0);
    let cap_lit = Color::new(0.80, 1.0, 0.62, 1.0);
    let stem = Color::new(0.95, 0.88, 0.70, 1.0);
    draw_ellipse_lines(
        p.x,
        p.y,
        11.0,
        6.0,
        0.0,
        2.0,
        Color::new(0.05, 0.12, 0.06, 0.30),
    );
    // 버섯 일곱: 뒤쪽(위)부터 그려 앞 버섯이 겹친다.
    let mut caps: Vec<Vec2> = (0..7)
        .map(|k| {
            let t = k as f32 / 7.0 * std::f32::consts::TAU + 0.3;
            (p + vec2(t.cos() * 11.0, t.sin() * 6.0)).round()
        })
        .collect();
    caps.sort_by(|u, v| u.y.total_cmp(&v.y));
    for q in caps {
        // 둥근 갓(위 3칸, 아래 5칸) + 짧은 대.
        draw_rectangle(q.x - 1.0, q.y - 5.0, 3.0, 1.0, OUTLINE);
        draw_rectangle(q.x - 2.0, q.y - 4.0, 5.0, 1.0, OUTLINE);
        draw_rectangle(q.x - 3.0, q.y - 3.0, 7.0, 3.0, OUTLINE);
        draw_rectangle(q.x - 1.0, q.y - 1.0, 3.0, 3.0, OUTLINE);
        draw_rectangle(q.x - 1.0, q.y - 4.0, 3.0, 1.0, cap);
        draw_rectangle(q.x - 2.0, q.y - 3.0, 5.0, 1.0, cap);
        let lit = Color::new(cap_lit.r, cap_lit.g, cap_lit.b, 0.6 + 0.4 * pulse);
        draw_rectangle(q.x - 2.0, q.y - 2.0, 5.0, 1.0, lit);
        draw_rectangle(q.x - 1.0, q.y - 4.0, 1.0, 1.0, WHITE);
        draw_rectangle(q.x, q.y - 1.0, 1.0, 2.0, stem);
    }
}

/// 저장 지점의 빛(0076). 사람 피드백 "저장 포인트가 너무 눈에 띄지 않는다": 초록 풀밭에 반투명 초록·금빛을
/// 섞으면 탁해져 묻힌다 → 빛을 더해(밝기 대비) 숨쉬는 후광 + 위로 뜨는 포자(움직임). 아직 저장하지 않은
/// 고리는 빛기둥이 높이 서서 부른다. 어두운 굴에서도 보이게 어둠 위에 그린다.
pub(super) fn draw_save_lights(w: &World, cam: Vec2, light: Option<&Material>) {
    let Some(g) = &w.room.grid else { return };
    let (x0, y0) = (
        (cam.x / 16.0).floor() as i32 - 2,
        (cam.y / 16.0).floor() as i32 - 2,
    );
    let rings: Vec<Vec2> = (y0..y0 + 16)
        .flat_map(|cy| (x0..x0 + 25).map(move |cx| (cx, cy)))
        .filter(|&(cx, cy)| g.get_i(cx, cy) == Cell::Save)
        .map(|(cx, cy)| vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0))
        .collect();
    if rings.is_empty() {
        return;
    }
    if let Some(m) = light {
        gl_use_material(m);
    }
    // 더하기면 바탕이 밝아지고, 보통 섞기(재질 없음)면 덮인다 → 그때는 옅게.
    let k = if light.is_some() { 1.0 } else { 0.6 };
    let pulse = 0.5 + 0.5 * (w.time * 2.5).sin();
    let a = |r: f32, g: f32, b: f32, al: f32| Color::new(r, g, b, al * k);
    for p in rings {
        let active = (w.adv.save_pos - p).length() < 1.0;
        draw_ellipse(
            p.x,
            p.y,
            16.0,
            9.0,
            0.0,
            a(1.0, 0.85, 0.45, 0.10 + 0.08 * pulse),
        );
        draw_ellipse(
            p.x,
            p.y,
            10.0,
            5.5,
            0.0,
            a(1.0, 0.92, 0.60, 0.14 + 0.08 * pulse),
        );
        draw_ellipse(
            p.x,
            p.y,
            5.0,
            2.5,
            0.0,
            a(1.0, 1.0, 0.80, 0.22 + 0.10 * pulse),
        );
        let rise = if active { 3 } else { 8 };
        for i in 0..rise {
            let f = (rise - i) as f32 / rise as f32;
            draw_rectangle(
                p.x - 2.0,
                p.y - 4.0 - i as f32 * 4.0,
                4.0,
                4.0,
                a(1.0, 0.95, 0.70, 0.22 * f * (0.6 + 0.4 * pulse)),
            );
        }
        // 위로 떠오르는 포자 다섯.
        for j in 0..5 {
            let ph = (w.time * 0.45 + j as f32 / 5.0).fract();
            let x = p.x + (w.time * 1.7 + j as f32 * 2.1).sin() * 8.0;
            let y = p.y - 2.0 - ph * 30.0;
            let s = if ph < 0.5 { 2.0 } else { 1.0 };
            draw_rectangle(
                x.round(),
                y.round(),
                s,
                s,
                a(1.0, 0.95, 0.65, 1.0 - ph * 0.8),
            );
        }
    }
    if light.is_some() {
        gl_use_default_material();
    }
}

/// 0..1 해시(환경 입자 자리, 0081). 같은 번호면 같은 값이라 상태 없이 그린다.
/// 입자 효과 설정(0141): 번호 `i`의 입자를 그릴까. `share`(0..=1) 비율만큼 고르게 고른다(0.5면 둘에 하나, 1이면 모두).
pub(super) fn keep_share(i: usize, share: f32) -> bool {
    ((i + 1) as f32 * share).floor() > (i as f32 * share).floor()
}

pub(super) fn hash01(i: u32, k: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// 환경 입자(0081): 들판·마을은 떠다니는 꽃가루, 뿌리 지대는 천천히 내리는 포자, 심층은 빛 알갱이(더하기 빛 패스).
/// 상태가 없다. 번호마다 월드에 붙은 자리(카메라 둘레 타일에서 되풀이)와 흐름이 정해져 있다.
/// 분위기 입자는 작고 대비가 낮게(타격 효과보다 눈에 덜 띄게).
/// 끝방 출구(0159). `down`이면 파 내려가는 구멍: 흙 테두리 안이 아래로 갈수록 검고, 빛나는 테두리와 안으로
/// 떨어지는 ⌄ 셋이 "내려간다"를 말한다. 예전엔 위를 가리키는 ▲라 올라가는 것으로 읽혔다(사장 피드백 2026-10-27).
/// `down`이 아니면(필드에서 들어온 옛 굴) 위로 나가는 구멍: 밝은 안쪽 + 떠오르는 ⌃.
pub(super) fn draw_exit(p: Vec2, time: f32, down: bool) {
    let pulse = 0.5 + 0.5 * (time * 3.0).sin();
    let rim = Color::new(0.36, 0.25, 0.18, 1.0);
    draw_ellipse(p.x, p.y + 3.0, 13.0, 7.5, 0.0, rim);
    draw_ellipse(p.x, p.y + 2.0, 12.0, 6.5, 0.0, world::DIRT);
    if down {
        for (k, (rx, ry)) in [(10.0, 5.2), (8.0, 4.0), (6.0, 2.8)]
            .into_iter()
            .enumerate()
        {
            let d = 0.16 - 0.05 * k as f32;
            draw_ellipse(
                p.x,
                p.y + 2.5,
                rx,
                ry,
                0.0,
                Color::new(d, d * 0.7, d * 0.9, 1.0),
            );
        }
        draw_ellipse(
            p.x,
            p.y + 2.5,
            4.0,
            1.8,
            0.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );
        // 테두리 빛: 숨 쉬듯.
        draw_ellipse_lines(
            p.x,
            p.y + 2.0,
            11.0,
            5.8,
            0.0,
            1.0,
            Color::new(1.0, 0.92, 0.62, 0.45 + 0.4 * pulse),
        );
        // 떨어지는 ⌄ 셋: 위에서 구멍 속으로, 들어갈수록 옅다.
        for k in 0..3 {
            let ph = (time * 0.9 + k as f32 / 3.0).fract();
            let y = p.y - 14.0 + ph * 16.0;
            let a = (1.0 - ph) * 0.9;
            let c = Color::new(1.0, 0.95, 0.75, a);
            let (x, y) = (p.x.round(), y.round());
            draw_rectangle(x - 3.0, y - 2.0, 2.0, 1.0, c);
            draw_rectangle(x + 2.0, y - 2.0, 2.0, 1.0, c);
            draw_rectangle(x - 2.0, y - 1.0, 2.0, 1.0, c);
            draw_rectangle(x + 1.0, y - 1.0, 2.0, 1.0, c);
            draw_rectangle(x - 1.0, y, 3.0, 1.0, c);
        }
    } else {
        draw_ellipse(
            p.x,
            p.y + 2.0,
            9.0,
            4.8,
            0.0,
            Color::new(0.95, 0.90, 0.62, 0.55 + 0.3 * pulse),
        );
        for k in 0..2 {
            let ph = (time * 0.8 + k as f32 * 0.5).fract();
            let y = p.y + 4.0 - ph * 16.0;
            let a = (1.0 - ph) * 0.9;
            let c = Color::new(1.0, 1.0, 0.85, a);
            let (x, y) = (p.x.round(), y.round());
            draw_rectangle(x - 1.0, y, 3.0, 1.0, c);
            draw_rectangle(x - 2.0, y + 1.0, 2.0, 1.0, c);
            draw_rectangle(x + 1.0, y + 1.0, 2.0, 1.0, c);
            draw_rectangle(x - 3.0, y + 2.0, 2.0, 1.0, c);
            draw_rectangle(x + 2.0, y + 2.0, 2.0, 1.0, c);
        }
    }
}

pub(super) fn draw_spring(sp: &world::Spring, time: f32) {
    let (x, y) = (sp.pos.x.round(), sp.pos.y.round() + 2.0);
    let live = if sp.drunk { 0.35 } else { 1.0 };
    // 젖은 흙 + 돌 테두리.
    draw_ellipse(
        x,
        y + 1.0,
        22.0,
        12.0,
        0.0,
        Color::new(0.30, 0.24, 0.18, 1.0),
    );
    draw_ellipse(x, y, 20.0, 10.5, 0.0, Color::new(0.52, 0.50, 0.46, 1.0));
    for k in 0..9 {
        let ang = k as f32 / 9.0 * std::f32::consts::TAU + 0.3;
        let (sx, sy) = (x + ang.cos() * 19.0, y + ang.sin() * 9.5);
        let c = if k % 2 == 0 {
            Color::new(0.62, 0.60, 0.56, 1.0)
        } else {
            Color::new(0.42, 0.40, 0.38, 1.0)
        };
        draw_ellipse(sx.round(), sy.round(), 3.0, 2.0, 0.0, c);
    }
    // 물: 깊은 가운데 → 얕은 가장자리.
    draw_ellipse(x, y, 16.0, 8.0, 0.0, Color::new(0.22, 0.42, 0.52, 1.0));
    draw_ellipse(x, y, 12.0, 5.5, 0.0, Color::new(0.30, 0.58, 0.72, 1.0));
    draw_ellipse(
        x - 2.0,
        y - 1.0,
        6.0,
        2.2,
        0.0,
        Color::new(0.55, 0.80, 0.90, 1.0),
    );
    // 물결 고리: 가운데에서 퍼져 옅어진다.
    for k in 0..2 {
        let ph = (time * 0.45 + k as f32 * 0.5).fract();
        let a = (1.0 - ph) * 0.55 * live;
        draw_ellipse_lines(
            x,
            y,
            2.0 + ph * 13.0,
            1.0 + ph * 6.5,
            0.0,
            1.0,
            Color::new(0.80, 0.95, 1.0, a),
        );
    }
    // 떠다니는 반짝임.
    for k in 0..4 {
        let ph = (time * 0.6 + k as f32 * 0.25).fract();
        let ang = k as f32 * 1.7 + time * 0.3;
        let (sx, sy) = (x + ang.cos() * 8.0, y - 2.0 + ang.sin() * 3.0 - ph * 6.0);
        let a = (1.0 - ph) * 0.9 * live;
        draw_rectangle(
            sx.round(),
            sy.round(),
            1.0,
            1.0,
            Color::new(1.0, 1.0, 1.0, a),
        );
    }
}

/// 보물 상자(던전 보물방). 열리면 뚜껑이 젖혀지고 안이 빈다.
/// 보물 상자. 새 그림(`props.png`, 0222)이 있으면 그 칸 + 닫힌 상자의 반짝, 없으면 코드 그림.
pub(super) fn draw_chest(tex: Option<&Texture2D>, r: Rect, opened: bool, time: f32) {
    if let Some(t) = tex {
        let feet = vec2(r.center().x, r.bottom() - 1.0);
        let frame = if opened {
            art::PROP_CHEST_OPEN
        } else {
            art::PROP_CHEST
        };
        art::draw_prop(t, frame, feet);
        if !opened && ((time * 2.0) as i32) % 2 == 0 {
            let (sx, sy) = ((feet.x + 5.0).round(), (feet.y - 15.0).round());
            draw_rectangle(sx, sy, 1.0, 3.0, WHITE);
            draw_rectangle(sx - 1.0, sy + 1.0, 3.0, 1.0, WHITE);
        }
        return;
    }
    let wood = Color::new(0.55, 0.36, 0.22, 1.0);
    let dark = Color::new(0.36, 0.22, 0.14, 1.0);
    let gold = Color::new(0.95, 0.80, 0.35, 1.0);
    let (x, y) = (r.x + 1.0, r.bottom() - 11.0);
    draw_rectangle(x - 1.0, y - 1.0, 16.0, 12.0, OUTLINE);
    draw_rectangle(x, y + 3.0, 14.0, 7.0, wood);
    draw_rectangle(x, y + 8.0, 14.0, 2.0, dark);
    if opened {
        draw_rectangle(x, y - 3.0, 14.0, 3.0, dark);
        draw_rectangle(
            x + 1.0,
            y + 3.0,
            12.0,
            2.0,
            Color::new(0.05, 0.03, 0.05, 1.0),
        );
    } else {
        draw_rectangle(x, y, 14.0, 3.0, wood);
        draw_rectangle(x + 6.0, y + 2.0, 2.0, 3.0, gold);
        if ((time * 2.0) as i32) % 2 == 0 {
            draw_rectangle(x + 11.0, y - 3.0, 1.0, 3.0, WHITE);
            draw_rectangle(x + 10.0, y - 2.0, 3.0, 1.0, WHITE);
        }
    }
}

/// 쉼터 모닥불: 돌 둘레 + 장작 + 흔들리는 불꽃 + 바닥 빛.
/// 모닥불의 돌만(불꽃·빛 없이): 그림자 패스용(0105). 그림 자리는 `draw_campfire_sunny`·`draw_campfire`와 같다.
/// 부술 수 있는 것(0175): 칸 하나(발 사각형 `r`) 위에 지대마다 다른 모양. 바닥 그림자 + 둥근 몸통 + 무늬.
/// 넷이 같은 표시를 나눠 쓴다(0205, 사장 "부술 수 있는 게 티가 안 난다"): 어두운 외곽선(바닥 장식이 아니라 물건),
/// 밝은 금(깨진다), 2.5초마다 반짝(움직임이 먼저 눈에 든다). 한 번 배우면 지대가 바뀌어도 알아본다.
/// 새 그림(`props.png`, 0222)이 있으면 그 칸(외곽선·금이 그림에 있다) + 반짝만 코드로.
pub(super) fn draw_breakable(
    tex: Option<&Texture2D>,
    b: crate::room::Breakable,
    r: Rect,
    time: f32,
) {
    use crate::room::Breakable;
    let c = r.center();
    let (cx, foot) = (c.x.round(), (r.bottom() - 2.0).round());
    draw_ellipse(cx, foot + 1.0, 7.0, 2.5, 0.0, SHADOW_SOFT);
    let px = |x: f32, y: f32, col: Color| draw_rectangle(x, y, 1.0, 1.0, col);
    if let Some(t) = tex {
        let frame = match b {
            Breakable::Mound => 0,
            Breakable::Gall => 1,
            Breakable::RockPile => 2,
            Breakable::SilkBall => 3,
        };
        art::draw_prop(t, frame, vec2(cx, foot + 1.0));
    } else {
        draw_breakable_code(b, cx, foot, c, time);
    }
    // 반짝: 2.5초마다 0.25초, 자리마다 어긋나게. 몸통 왼쪽 위에 십자(새 그림은 더 커서 두 칸 위).
    let ph = (time / 2.5 + (c.x * 0.37 + c.y * 0.61).fract()).fract();
    if ph < 0.1 {
        let a = 1.0 - (ph / 0.1 - 0.5).abs() * 2.0;
        let (gx, gy) = (cx - 3.0, foot - if tex.is_some() { 10.0 } else { 8.0 });
        let w = Color::new(1.0, 1.0, 1.0, a);
        px(gx, gy, w);
        px(gx - 1.0, gy, Color::new(1.0, 1.0, 1.0, a * 0.7));
        px(gx + 1.0, gy, Color::new(1.0, 1.0, 1.0, a * 0.7));
        px(gx, gy - 1.0, Color::new(1.0, 1.0, 1.0, a * 0.7));
        px(gx, gy + 1.0, Color::new(1.0, 1.0, 1.0, a * 0.7));
    }
}

/// 부술 수 있는 것의 코드 그림(새 그림이 없을 때): 몸통 + 금.
fn draw_breakable_code(b: crate::room::Breakable, cx: f32, foot: f32, c: Vec2, time: f32) {
    use crate::room::Breakable;
    let px = |x: f32, y: f32, col: Color| draw_rectangle(x, y, 1.0, 1.0, col);
    // 몸통 둘레 1px 외곽선: 몸통 타원을 한 px 크게 먼저 칠한다.
    let rim =
        |x: f32, y: f32, rx: f32, ry: f32| draw_ellipse(x, y, rx + 1.0, ry + 1.0, 0.0, OUTLINE);
    match b {
        Breakable::Mound => rim(cx, foot - 3.0, 6.5, 4.5),
        Breakable::Gall => rim(cx, foot - 4.0, 5.7, 5.5),
        Breakable::RockPile => {
            rim(cx - 3.0, foot - 2.0, 3.5, 2.5);
            rim(cx + 3.0, foot - 2.0, 3.5, 2.5);
            rim(cx, foot - 5.5, 3.5, 3.0);
        }
        Breakable::SilkBall => rim(cx, foot - 4.5, 5.5, 5.0),
    }
    match b {
        Breakable::Mound => {
            // 흙더미: 갈색 둔덕, 위는 밝고 아래는 어둡다. 조약돌 둘.
            let (lo, hi) = (
                Color::new(0.42, 0.30, 0.20, 1.0),
                Color::new(0.62, 0.47, 0.32, 1.0),
            );
            draw_ellipse(cx, foot - 3.0, 6.5, 4.5, 0.0, lo);
            draw_ellipse(cx - 0.5, foot - 4.5, 5.0, 3.0, 0.0, hi);
            px(cx - 3.0, foot - 2.0, Color::new(0.5, 0.48, 0.5, 1.0));
            px(cx + 2.0, foot - 5.0, Color::new(0.5, 0.48, 0.5, 1.0));
            px(cx + 1.0, foot - 1.0, lo);
        }
        Breakable::Gall => {
            // 뿌리혹: 불그죽죽한 혹, 어두운 뿌리 줄이 감겨 있다. 살짝 숨 쉰다.
            let k = 1.0 + 0.04 * (time * 2.0 + c.x * 0.1).sin();
            let (lo, hi) = (
                Color::new(0.40, 0.22, 0.18, 1.0),
                Color::new(0.58, 0.33, 0.26, 1.0),
            );
            draw_ellipse(cx, foot - 4.0, 5.5 * k, 5.5, 0.0, lo);
            draw_ellipse(cx - 1.0, foot - 5.0, 4.0 * k, 3.5, 0.0, hi);
            let root = Color::new(0.28, 0.16, 0.12, 1.0);
            draw_line(cx - 5.0, foot - 2.0, cx + 4.0, foot - 6.0, 1.0, root);
            draw_line(cx - 3.0, foot - 8.0, cx + 5.0, foot - 3.0, 1.0, root);
        }
        Breakable::RockPile => {
            // 돌무더기: 회색 돌 셋을 쌓았다.
            let (lo, hi) = (
                Color::new(0.36, 0.33, 0.36, 1.0),
                Color::new(0.58, 0.55, 0.58, 1.0),
            );
            draw_ellipse(cx - 3.0, foot - 2.0, 3.5, 2.5, 0.0, lo);
            draw_ellipse(cx + 3.0, foot - 2.0, 3.5, 2.5, 0.0, lo);
            draw_ellipse(cx, foot - 5.5, 3.5, 3.0, 0.0, hi);
            px(cx - 4.0, foot - 3.0, hi);
            px(cx + 2.0, foot - 3.0, hi);
            px(cx - 1.0, foot - 7.0, WHITE);
        }
        Breakable::SilkBall => {
            // 실뭉치: 희끄무레한 공, 실 줄 몇 가닥. 보랏빛이 돈다.
            let (lo, hi) = (
                Color::new(0.72, 0.68, 0.80, 1.0),
                Color::new(0.92, 0.90, 0.98, 1.0),
            );
            draw_ellipse(cx, foot - 4.5, 5.5, 5.0, 0.0, lo);
            draw_ellipse(cx - 1.0, foot - 5.5, 4.0, 3.5, 0.0, hi);
            let thread = Color::new(0.60, 0.50, 0.75, 0.9);
            draw_line(cx - 5.0, foot - 5.0, cx + 5.0, foot - 4.0, 1.0, thread);
            draw_line(cx - 3.0, foot - 8.0, cx + 2.0, foot - 1.0, 1.0, thread);
            px(cx + 5.0, foot - 2.0, thread);
        }
    }
    // 금: 위에서 아래로 갈지자, 밝은 줄 + 오른쪽 그늘 한 점씩(갈라진 틈).
    let (lit, dark) = (
        Color::new(1.0, 0.95, 0.82, 0.95),
        Color::new(0.12, 0.08, 0.10, 0.75),
    );
    for (dx, dy) in [
        (1.0, -7.0),
        (0.0, -6.0),
        (1.0, -5.0),
        (2.0, -4.0),
        (1.0, -3.0),
    ] {
        px(cx + dx, foot + dy, lit);
        px(cx + dx + 1.0, foot + dy, dark);
    }
}

/// 산딸기(0178): 작은 알(드루플렛)이 모인 송이 + 초록 꽃받침. `x`·`y`는 송이 가운데, `s`는 픽셀 크기(HUD는 배율),
/// `dim`은 흐리게(가방이 비었을 때). 바닥 아이템과 가방 칸이 같은 그림을 쓴다.
pub(super) fn draw_berry(x: f32, y: f32, s: f32, dim: f32) {
    // 9×10 비트맵. O 테두리, r 빨강, R 밝은 알, d 어두운 틈, g 꽃받침, G 밝은 잎, k 꼭지.
    const ROWS: [&str; 10] = [
        "...OkO...",
        ".OGgGgGO.",
        ".OrRrRrO.",
        "ORrRrRrRO",
        "OrRrRrRrO",
        "ORrRrRrRO",
        ".OrdrdrO.",
        ".OdrRrdO.",
        "..OdrdO..",
        "...OOO...",
    ];
    let col = |ch: u8| -> Option<Color> {
        let c = match ch {
            b'O' => OUTLINE,
            b'r' => Color::new(0.86 * dim, 0.22 * dim, 0.34 * dim, 1.0),
            b'R' => Color::new(1.0 * dim, 0.48 * dim, 0.56 * dim, 1.0),
            b'd' => Color::new(0.58 * dim, 0.10 * dim, 0.24 * dim, 1.0),
            b'g' => Color::new(0.36 * dim, 0.62 * dim, 0.28 * dim, 1.0),
            b'G' => Color::new(0.55 * dim, 0.80 * dim, 0.40 * dim, 1.0),
            b'k' => Color::new(0.42 * dim, 0.30 * dim, 0.20 * dim, 1.0),
            _ => return None,
        };
        Some(c)
    };
    let (x0, y0) = (x - 4.0 * s, y - 5.0 * s);
    for (j, row) in ROWS.iter().enumerate() {
        for (i, ch) in row.bytes().enumerate() {
            if let Some(c) = col(ch) {
                draw_rectangle(x0 + i as f32 * s, y0 + j as f32 * s, s, s, c);
            }
        }
    }
}

pub(super) fn draw_campfire_stones(sunny: Option<&art::Sunny>, r: Rect) {
    if let Some(sunny) = sunny {
        let c = vec2(r.center().x, r.bottom() - 4.0);
        crate::sunny::draw_at_foot(
            &sunny.sheet,
            crate::sunny::FIRE_PIT,
            c + vec2(0.0, 6.0),
            WHITE,
        );
        return;
    }
    let c = vec2(r.center().x, r.bottom() - 5.0);
    for k in 0..7 {
        let a = k as f32 / 7.0 * std::f32::consts::TAU;
        let p = c + vec2(a.cos() * 7.0, a.sin() * 3.0);
        draw_rectangle((p.x - 1.5).round(), (p.y - 1.0).round(), 3.0, 2.0, OUTLINE);
    }
}

pub(super) fn draw_campfire(r: Rect, time: f32) {
    let c = vec2(r.center().x, r.bottom() - 5.0);
    let flick = (time * 9.0).sin() * 0.5 + (time * 13.7).sin() * 0.5;
    draw_circle(
        c.x,
        c.y - 2.0,
        22.0 + flick,
        Color::new(1.0, 0.65, 0.3, 0.07),
    );
    draw_circle(
        c.x,
        c.y - 2.0,
        13.0 + flick * 0.5,
        Color::new(1.0, 0.7, 0.35, 0.08),
    );
    let stone = Color::new(0.45, 0.42, 0.44, 1.0);
    for k in 0..7 {
        let a = k as f32 / 7.0 * std::f32::consts::TAU;
        let p = c + vec2(a.cos() * 7.0, a.sin() * 3.0);
        draw_rectangle((p.x - 1.5).round(), (p.y - 1.0).round(), 3.0, 2.0, OUTLINE);
        draw_rectangle((p.x - 1.0).round(), (p.y - 1.0).round(), 2.0, 1.0, stone);
    }
    let wood = Color::new(0.45, 0.30, 0.20, 1.0);
    draw_line(c.x - 5.0, c.y + 1.0, c.x + 5.0, c.y - 2.0, 2.0, wood);
    draw_line(c.x - 5.0, c.y - 2.0, c.x + 5.0, c.y + 1.0, 2.0, wood);
    let h = 7.0 + flick * 1.5;
    draw_triangle(
        vec2(c.x - 4.0, c.y - 1.0),
        vec2(c.x + 4.0, c.y - 1.0),
        vec2(c.x + flick, c.y - h - 3.0),
        Color::new(0.95, 0.45, 0.2, 1.0),
    );
    draw_triangle(
        vec2(c.x - 2.5, c.y - 1.0),
        vec2(c.x + 2.5, c.y - 1.0),
        vec2(c.x + flick * 0.5, c.y - h),
        Color::new(1.0, 0.85, 0.35, 1.0),
    );
    // 불티
    for k in 0..3 {
        let ph = (time * 0.9 + k as f32 * 0.33).fract();
        let x = c.x + ((k as f32 * 2.1 + time * 2.0).sin() * 4.0);
        draw_rectangle(
            x.round(),
            (c.y - 8.0 - ph * 16.0).round(),
            1.0,
            1.0,
            Color::new(1.0, 0.8, 0.4, 1.0 - ph),
        );
    }
}

/// 모닥불, 팩 그림판(0066): 돌 두른 자리 + 불꽃 4칸 + 따뜻한 빛·불티(직접).
/// 새 화덕 그림(0224 A)이 있으면 팩 돌 자리 대신 그것을(불꽃·빛·불티는 그대로).
pub(super) fn draw_campfire_sunny(sunny: &art::Sunny, hub: Option<&Texture2D>, r: Rect, time: f32) {
    let c = vec2(r.center().x, r.bottom() - 4.0);
    let flick = (time * 9.0).sin() * 0.5 + (time * 13.7).sin() * 0.5;
    draw_circle(
        c.x,
        c.y - 4.0,
        24.0 + flick,
        Color::new(1.0, 0.65, 0.3, 0.07),
    );
    draw_circle(
        c.x,
        c.y - 4.0,
        14.0 + flick * 0.5,
        Color::new(1.0, 0.7, 0.35, 0.08),
    );
    match hub {
        Some(h) => art::draw_hub(h, art::HUB_FIREPIT, c + vec2(0.0, 6.0), WHITE),
        None => crate::sunny::draw_at_foot(
            &sunny.sheet,
            crate::sunny::FIRE_PIT,
            c + vec2(0.0, 6.0),
            WHITE,
        ),
    }
    let (fw, fh) = crate::sunny::FIRE_FRAME;
    let f = ((time * 8.0) as u32 % 4) as f32;
    draw_texture_ex(
        &sunny.fire,
        (c.x - fw / 2.0).round(),
        (c.y - fh + 1.0).round(),
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(f * fw, 0.0, fw, fh)),
            ..Default::default()
        },
    );
    for k in 0..3 {
        let ph = (time * 0.9 + k as f32 * 0.33).fract();
        let x = c.x + ((k as f32 * 2.1 + time * 2.0).sin() * 4.0);
        draw_rectangle(
            x.round(),
            (c.y - 10.0 - ph * 16.0).round(),
            1.0,
            1.0,
            Color::new(1.0, 0.8, 0.4, 1.0 - ph),
        );
    }
}

/// 포포의 가게 좌판: 나무 상자 + 줄무늬 차양 + 산딸기·지렁이 단지.
pub(super) fn draw_stall(r: Rect) {
    let wood = Color::new(0.55, 0.36, 0.22, 1.0);
    let dark = Color::new(0.36, 0.22, 0.14, 1.0);
    let (x, y) = (r.x, r.bottom() - 12.0);
    draw_rectangle(x - 1.0, y - 1.0, 18.0, 13.0, OUTLINE);
    draw_rectangle(x, y, 16.0, 11.0, wood);
    draw_rectangle(x, y + 8.0, 16.0, 3.0, dark);
    // 차양(위로 솟은 막대 둘 + 줄무늬 천)
    draw_rectangle(x, y - 14.0, 1.0, 14.0, dark);
    draw_rectangle(x + 15.0, y - 14.0, 1.0, 14.0, dark);
    for k in 0..4 {
        let c = if k % 2 == 0 {
            Color::new(0.85, 0.35, 0.35, 1.0)
        } else {
            Color::new(0.95, 0.88, 0.75, 1.0)
        };
        draw_rectangle(x - 1.0 + k as f32 * 4.5, y - 17.0, 4.5, 4.0, c);
    }
    draw_rectangle(x - 1.0, y - 13.0, 18.0, 1.0, OUTLINE);
    // 물건
    draw_circle(x + 4.0, y - 1.0, 2.0, world::BERRY);
    draw_circle(x + 7.0, y - 1.0, 2.0, world::BERRY);
    draw_rectangle(x + 10.0, y - 3.0, 4.0, 3.0, world::SEED);
}

/// 가게(팩 장터 가판, 0077): 판대 위에 산딸기·씨앗 자루는 직접 올린다(무엇을 파는지 보이게).
pub(super) fn draw_stall_sunny(sunny: &art::Sunny, r: Rect) {
    use crate::sunny as sn;
    let foot = vec2((r.center().x).round(), r.bottom().round());
    draw_ellipse(foot.x, foot.y - 1.0, 13.0, 3.0, 0.0, SHADOW_SOFT);
    sn::draw_at_foot(&sunny.sheet, sn::STALL, foot, WHITE);
    let top = foot.y - sn::STALL.3 as f32 + sn::STALL_COUNTER;
    let x = foot.x - 7.0;
    draw_circle(x + 3.0, top - 1.0, 2.0, world::BERRY);
    draw_circle(x + 6.0, top - 1.0, 2.0, world::BERRY);
    draw_rectangle(x + 10.0, top - 4.0, 4.0, 4.0, OUTLINE);
    draw_rectangle(x + 10.5, top - 3.5, 3.0, 3.5, world::SEED);
}

/// 빈 자리(0046): 새 통나무 자리 그림(0224 A) + 꽃잎. 꽃잎 규칙은 `draw_seat`와 같다.
pub(super) fn draw_seat_hub(hub: &Texture2D, r: Rect, petals: u8) {
    let foot = vec2((r.center().x).round(), r.bottom().round() + 1.0);
    draw_ellipse(foot.x, foot.y - 1.0, 10.0, 2.0, 0.0, SHADOW_SOFT);
    art::draw_hub(hub, art::HUB_LOG, foot, WHITE);
    let (x, y) = (foot.x - 7.0, foot.y - 13.0);
    seat_petals(x, y, petals);
}

/// 빈 자리(0046): 팩 그루터기 + 꽃잎. 꽃잎 규칙은 `draw_seat`와 같다.
pub(super) fn draw_seat_sunny(sunny: &art::Sunny, r: Rect, petals: u8) {
    use crate::sunny as sn;
    let foot = vec2((r.center().x).round(), r.bottom().round());
    draw_ellipse(foot.x, foot.y - 1.0, 8.0, 2.0, 0.0, SHADOW_SOFT);
    sn::draw_at_foot(&sunny.sheet, sn::STUMPS[1], foot, WHITE);
    let (x, y) = (foot.x - 7.0, foot.y - 12.0);
    seat_petals(x, y, petals);
}

/// 실 고리(0055): 막대 둘에 걸린 흰 실 고리(실뜨기 모양 하나). 둥실 떠오르며 보랏빛으로 숨 쉰다.
pub(super) fn draw_dream_loop(at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round());
    // 막대 둘
    for dx in [-7.0, 7.0] {
        draw_rectangle(x + dx - 1.0, y - 14.0, 2.0, 18.0, OUTLINE);
        draw_rectangle(
            x + dx - 0.5,
            y - 13.0,
            1.0,
            16.0,
            Color::new(0.55, 0.40, 0.30, 1.0),
        );
    }
    let bob = ((time * 1.6).sin() * 1.5).round();
    let glow = 0.35 + 0.25 * (time * 2.2).sin().abs();
    if !shadow_pass() {
        draw_circle(
            x,
            y - 12.0 + bob,
            9.0,
            Color::new(0.72, 0.62, 0.95, glow * 0.35),
        );
    }
    // 실뜨기 마름모 + 가로줄
    let c = Color::new(0.95, 0.93, 1.0, 1.0);
    let (cx, cy) = (x, y - 12.0 + bob);
    let pts = [
        vec2(cx - 6.0, cy),
        vec2(cx, cy - 5.0),
        vec2(cx + 6.0, cy),
        vec2(cx, cy + 5.0),
    ];
    for i in 0..4 {
        let (a, b) = (pts[i], pts[(i + 1) % 4]);
        draw_line(a.x, a.y, b.x, b.y, 1.0, c);
    }
    draw_line(
        cx - 6.0,
        cy,
        cx + 6.0,
        cy,
        1.0,
        Color::new(0.80, 0.72, 1.0, 1.0),
    );
    draw_line(x - 7.0, y - 13.0, cx - 6.0, cy, 1.0, c);
    draw_line(x + 7.0, y - 13.0, cx + 6.0, cy, 1.0, c);
    // 떠오르는 빛 알갱이
    for k in (0..2).filter(|_| !shadow_pass()) {
        let ph = (time * 0.7 + k as f32 * 0.5).fract();
        draw_rectangle(
            (cx - 3.0 + k as f32 * 6.0).round(),
            (cy - 4.0 - ph * 10.0).round(),
            1.0,
            1.0,
            Color::new(0.85, 0.80, 1.0, 1.0 - ph),
        );
    }
}

/// 마을 물웅덩이(0048): 파란 타원 + 떠다니는 반짝임 둘.
pub(super) fn draw_puddle(at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round() + 3.0);
    draw_ellipse(x, y, 11.0, 5.0, 0.0, Color::new(0.20, 0.30, 0.28, 1.0));
    draw_ellipse(x, y, 10.0, 4.0, 0.0, Color::new(0.38, 0.62, 0.78, 1.0));
    draw_ellipse(
        x - 2.0,
        y - 1.0,
        5.0,
        1.5,
        0.0,
        Color::new(0.62, 0.82, 0.92, 1.0),
    );
    for k in 0..2 {
        let ph = (time * 0.7 + k as f32 * 0.5).fract();
        let c = Color::new(1.0, 1.0, 1.0, 0.8 * (1.0 - ph));
        draw_rectangle(
            (x - 5.0 + k as f32 * 7.0 + ph * 3.0).round(),
            y - 1.0 + k as f32,
            1.0,
            1.0,
            c,
        );
    }
}

/// 빈 자리(모모를 기다리는 그루터기). 꽃잎 셋 가운데 모모를 이긴 만큼 분홍(0046).
pub(super) fn draw_seat(r: Rect, petals: u8) {
    let bark = Color::new(0.42, 0.28, 0.20, 1.0);
    let ring = Color::new(0.75, 0.58, 0.40, 1.0);
    let (x, y) = (r.x + 1.0, r.bottom() - 9.0);
    draw_rectangle(x - 1.0, y - 1.0, 16.0, 10.0, OUTLINE);
    draw_rectangle(x, y + 2.0, 14.0, 6.0, bark);
    draw_ellipse(x + 7.0, y + 2.0, 7.0, 2.5, 0.0, ring);
    draw_ellipse_lines(x + 7.0, y + 2.0, 4.0, 1.2, 0.0, 1.0, bark);
    seat_petals(x, y, petals);
}

/// 의자 위 꽃잎 셋: 모모를 이긴 만큼 분홍, 처음엔 시든 것 하나만.
pub(super) fn seat_petals(x: f32, y: f32, petals: u8) {
    let wilted = Color::new(0.69, 0.50, 0.55, 1.0);
    let pink = Color::new(1.0, 0.66, 0.77, 1.0);
    for (i, dx) in [10.0, 3.0, 7.0].into_iter().enumerate() {
        // 처음엔 시든 꽃잎 하나만 떨어져 있다(seat_1).
        if petals == 0 && i > 0 {
            break;
        }
        let c = if (i as u8) < petals { pink } else { wilted };
        draw_rectangle(x + dx, y + 1.0 + (i % 2) as f32, 2.0, 1.0, c);
    }
}

/// 큰 나무 밑동의 빛(0061), 나무 뒤: 은은한 빛 웅덩이와, 밑동에서 퍼져 나가는 밝은 고리.
pub(super) fn draw_tree_glow(base: Vec2, time: f32) {
    let warm = Color::new(1.0, 0.95, 0.55, 1.0);
    let k = 0.5 + 0.5 * (time * 2.5).sin();
    for (r, a) in [(26.0, 0.14), (16.0, 0.22)] {
        draw_ellipse(
            base.x,
            base.y,
            r + k * 3.0,
            (r + k * 3.0) * 0.4,
            0.0,
            Color::new(warm.r, warm.g, warm.b, a),
        );
    }
    // 고리 둘이 엇갈려 퍼진다(풀밭 초록 위에서도 선으로 읽힌다).
    for off in [0.0, 0.5] {
        let ph = (time * 0.7 + off).fract();
        let r = 8.0 + ph * 26.0;
        let a = (1.0 - ph) * 0.9;
        draw_ellipse_lines(
            base.x,
            base.y,
            r,
            r * 0.4,
            0.0,
            1.0,
            Color::new(1.0, 1.0, 0.7, a),
        );
    }
}

/// 큰 나무 밑동의 빛 알갱이(0061), 나무 앞: 줄기를 따라 떠오른다.
pub(super) fn draw_tree_motes(base: Vec2, time: f32) {
    for i in 0..10 {
        let ph = (time * 0.55 + i as f32 / 10.0).fract();
        let x = base.x + ((i as f32 * 2.4 + time * 1.3).sin()) * 16.0;
        let y = base.y + 2.0 - ph * 44.0;
        let a = (1.0 - ph).min(ph * 4.0);
        let sz = if i % 3 == 0 { 2.0 } else { 1.0 };
        draw_rectangle(x.round(), y.round(), sz, sz, Color::new(0.95, 1.0, 0.6, a));
    }
}

/// 큰 나무(마을), 팩 그림판(0065): 우리 줄기·뿌리 위에 Sunnyside 둥근 나무의 잎을 무더기로 쌓는다.
/// 잎 무더기 수 = 잎 단계(1 → 3, 2 → 7, 3 → 10, 4 → 12). 3부터 꽃. 무더기마다 흔들림이 어긋난다.
pub(super) fn draw_great_tree_sunny(sunny: &art::Sunny, at: Vec2, growth: u8, time: f32) {
    let bark = Color::new(0.45, 0.29, 0.22, 1.0);
    let bark_hi = Color::new(0.58, 0.39, 0.28, 1.0);
    let bark_lo = Color::new(0.32, 0.20, 0.17, 1.0);
    let (x, base) = (at.x.round(), at.y + 8.0);
    // 뿌리: 벼랑 위로 뻗는 굵은 뿌리 넷(바닥에 깔려 그림자 패스에서는 뺀다).
    for (dx, dy) in [(-30.0, 4.0), (-14.0, 6.0), (14.0, 6.0), (30.0, 4.0)]
        .into_iter()
        .filter(|_| !shadow_pass())
    {
        draw_line(x + dx * 0.3, base - 4.0, x + dx, base + dy, 6.0, OUTLINE);
        draw_line(x + dx * 0.3, base - 4.0, x + dx, base + dy, 4.0, bark);
        draw_line(
            x + dx * 0.3,
            base - 5.0,
            x + dx * 0.9,
            base + dy - 1.0,
            1.0,
            bark_hi,
        );
    }
    // 줄기: 아래가 넓다.
    // 마을 지도에서 큰 나무는 위쪽 가장자리 가까이 선다: 잎 꼭대기가 밑동 위 60px 안에 들어와야 화면에 다 보인다.
    let top = base - 22.0;
    draw_triangle(
        vec2(x - 16.0, base),
        vec2(x + 16.0, base),
        vec2(x, base - 12.0),
        OUTLINE,
    );
    draw_rectangle(x - 13.0, top, 26.0, base - top, OUTLINE);
    draw_rectangle(x - 12.0, top, 24.0, base - top, bark);
    draw_rectangle(x - 12.0, top, 5.0, base - top, bark_hi);
    draw_rectangle(x + 7.0, top, 5.0, base - top, bark_lo);
    for k in 0..2 {
        let yy = top + 8.0 + k as f32 * 6.0;
        draw_rectangle(x - 4.0 + (k % 2) as f32 * 6.0, yy, 1.0, 5.0, bark_lo);
    }
    // 잎 무더기(뒤 → 앞). (가운데에서 x, 잎 아래 끝의 줄기 꼭대기 기준 y, 나오는 단계)
    let crowns: [(f32, f32, u8); 12] = [
        (-12.0, -16.0, 4),
        (12.0, -17.0, 4),
        (-26.0, -8.0, 3),
        (0.0, -9.0, 3),
        (26.0, -8.0, 3),
        (-38.0, 1.0, 2),
        (-13.0, 0.0, 2),
        (13.0, 0.0, 2),
        (38.0, 1.0, 2),
        (-25.0, 8.0, 1),
        (0.0, 7.0, 1),
        (25.0, 8.0, 1),
    ];
    let (tw, _) = art::SUNNY_TREE;
    for (i, (dx, dy, need)) in crowns.iter().enumerate() {
        if growth < *need {
            continue;
        }
        let f = ((time * 2.5 + i as f32 * 0.37) as u32 % 4) as f32;
        let (cx, cy) = (
            (x + dx - tw / 2.0).round(),
            (top + dy - art::SUNNY_CROWN_H).round(),
        );
        draw_texture_ex(
            &sunny.tree,
            cx,
            cy,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(f * tw, 0.0, tw, art::SUNNY_CROWN_H)),
                ..Default::default()
            },
        );
        if growth >= 3 && i % 2 == 0 {
            // 꽃(분홍 셋, 직접 찍음).
            for (bx, by) in [(10.0, 6.0), (19.0, 10.0), (14.0, 14.0)] {
                draw_rectangle(cx + bx, cy + by, 2.0, 2.0, Color::new(1.0, 0.76, 0.84, 1.0));
                draw_rectangle(cx + bx, cy + by, 1.0, 1.0, WHITE);
            }
        }
    }
}

/// 큰 나무(마을, 0045): 벼랑 위로 솟은 줄기와 뿌리. 잎은 깬 횟수만큼 돌아온다(0 = 마른 잎 몇 장, 3부터 꽃).
pub(super) fn draw_great_tree(at: Vec2, growth: u8, time: f32) {
    let bark = Color::new(0.40, 0.27, 0.21, 1.0);
    let bark_hi = Color::new(0.52, 0.37, 0.27, 1.0);
    let (x, base) = (at.x.round(), at.y + 8.0);
    // 뿌리(바닥에 깔려 그림자 패스에서는 뺀다)
    for (dx, len) in [(-26.0, 18.0), (-12.0, 10.0), (12.0, 10.0), (26.0, 18.0)]
        .into_iter()
        .filter(|_| !shadow_pass())
    {
        draw_line(
            x + dx * 0.4,
            base - 2.0,
            x + dx,
            base + len * 0.2,
            4.0,
            OUTLINE,
        );
        draw_line(
            x + dx * 0.4,
            base - 2.0,
            x + dx,
            base + len * 0.2,
            2.0,
            bark,
        );
    }
    // 줄기
    let top = base - 44.0;
    draw_rectangle(x - 13.0, top, 26.0, base - top, OUTLINE);
    draw_rectangle(x - 12.0, top, 24.0, base - top, bark);
    draw_rectangle(x - 9.0, top, 4.0, base - top, bark_hi);
    for k in 0..3 {
        draw_rectangle(
            x - 3.0 + k as f32 * 5.0,
            top + 6.0 + k as f32 * 9.0,
            1.0,
            10.0,
            OUTLINE,
        );
    }
    // 잎·꽃: 단계마다 더 많이(위치는 고정된 표에서)
    let spots: [(f32, f32); 15] = [
        (-30.0, -4.0),
        (28.0, -6.0),
        (0.0, -12.0),
        (-16.0, -14.0),
        (16.0, -16.0),
        (-40.0, 4.0),
        (40.0, 2.0),
        (-8.0, -22.0),
        (8.0, -24.0),
        (-26.0, -18.0),
        (26.0, -20.0),
        (0.0, -30.0),
        (-36.0, -10.0),
        (36.0, -12.0),
        (-14.0, -30.0),
    ];
    let n = (3 + growth as usize * 3).min(spots.len());
    for (i, (dx, dy)) in spots.iter().take(n).enumerate() {
        let sway = ((time * 1.2 + i as f32).sin() * 1.0).round();
        let (cx, cy) = (x + dx + sway, top + dy);
        let r = if growth == 0 { 3.0 } else { 7.0 };
        let leaf = if growth == 0 {
            Color::new(0.62, 0.48, 0.30, 1.0)
        } else {
            Color::new(0.36 + 0.03 * (i % 3) as f32, 0.58, 0.32, 1.0)
        };
        draw_circle(cx, cy, r + 1.0, OUTLINE);
        draw_circle(cx, cy, r, leaf);
        if growth >= 3 && i % 2 == 0 {
            draw_circle(cx + 2.0, cy - 2.0, 2.0, Color::new(1.0, 0.78, 0.84, 1.0));
        }
    }
}

/// 마을 꾸미기(0045): 0 꽃밭, 1 반딧불 등, 2 빛나는 버섯, 3 통나무 의자, 4 바람종, 5 깃발 줄.
/// 마을 꾸미기, 팩 그림판(0066): 0 꽃밭 = 열매·꽃 덤불 + 들꽃, 2 빛나는 버섯 = 파란 버섯 셋(움직임).
pub(super) fn draw_decor_sunny(sunny: &art::Sunny, i: usize, at: Vec2, time: f32) {
    use crate::sunny as sn;
    let foot = vec2(at.x.round(), at.y.round() + 6.0);
    match i {
        0 => {
            draw_ellipse(foot.x, foot.y - 1.0, 16.0, 3.0, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(&sunny.sheet, sn::BUSH_BERRY, foot + vec2(-8.0, 0.0), WHITE);
            sn::draw_at_foot(&sunny.sheet, sn::BUSH_FLOWER, foot + vec2(9.0, 1.0), WHITE);
            for (k, dx) in [-14.0f32, -2.0, 15.0].iter().enumerate() {
                let set = [sn::FLOWERS_WHITE, sn::FLOWERS_RED, sn::FLOWERS_BLUE][k];
                sn::draw_at_foot(&sunny.sheet, set[3], foot + vec2(*dx, 4.0), WHITE);
            }
        }
        3 => {
            draw_ellipse(foot.x, foot.y - 1.0, 12.0, 2.5, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(&sunny.sheet, sn::BENCH, foot, WHITE);
        }
        7 => {
            draw_ellipse(foot.x - 2.0, foot.y - 1.0, 8.0, 2.0, 0.0, SHADOW_SOFT);
            sn::draw_at_foot(
                &sunny.sheet,
                sn::POT_WILDFLOWER,
                foot + vec2(-3.0, 0.0),
                WHITE,
            );
            sn::draw_at_foot(&sunny.sheet, sn::WILDFLOWERS, foot + vec2(6.0, 1.0), WHITE);
        }
        _ => {
            let s = sn::MUSH_FRAME;
            for (k, dx) in [-8.0f32, 0.0, 8.0].iter().enumerate() {
                let pulse = 0.6 + 0.3 * (time * 1.8 + k as f32).sin();
                let f = ((time * 4.0 + k as f32 * 1.3) as u32 % 4) as f32;
                let (x, y) = (
                    (foot.x + dx - s / 2.0).round(),
                    (foot.y - s + (k % 2) as f32 * 2.0).round(),
                );
                if !shadow_pass() {
                    draw_circle(
                        x + 8.0,
                        y + 10.0,
                        7.0,
                        Color::new(0.5, 0.85, 1.0, 0.12 * pulse),
                    );
                }
                draw_texture_ex(
                    &sunny.mush,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(f * s, 0.0, s, s)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

pub(super) fn draw_decor(i: usize, at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round() + 4.0);
    let wood = Color::new(0.45, 0.31, 0.22, 1.0);
    match i {
        0 => {
            draw_ellipse(x, y, 11.0, 4.0, 0.0, OUTLINE);
            draw_ellipse(x, y, 10.0, 3.0, 0.0, Color::new(0.42, 0.30, 0.22, 1.0));
            let cols = [
                Color::new(1.0, 0.70, 0.78, 1.0),
                Color::new(1.0, 0.92, 0.55, 1.0),
                Color::new(0.80, 0.70, 1.0, 1.0),
            ];
            for k in 0..5 {
                let fx = x - 8.0 + k as f32 * 4.0;
                draw_rectangle(fx, y - 4.0, 1.0, 3.0, Color::new(0.36, 0.58, 0.32, 1.0));
                draw_rectangle(fx - 1.0, y - 6.0, 3.0, 2.0, cols[k % 3]);
            }
        }
        1 => {
            let pulse = 0.55 + 0.25 * (time * 2.5).sin();
            draw_circle(x, y - 12.0, 9.0, Color::new(0.85, 1.0, 0.55, 0.15 * pulse));
            draw_rectangle(x - 1.0, y - 10.0, 2.0, 12.0, wood);
            draw_rectangle(x - 3.0, y - 15.0, 6.0, 6.0, OUTLINE);
            draw_rectangle(
                x - 2.0,
                y - 14.0,
                4.0,
                4.0,
                Color::new(0.85, 1.0, 0.55, pulse + 0.2),
            );
        }
        2 => {
            for (k, (dx, h)) in [(-5.0f32, 5.0f32), (0.0, 8.0), (5.0, 4.0)]
                .into_iter()
                .enumerate()
            {
                let pulse = 0.6 + 0.3 * (time * 1.8 + k as f32).sin();
                draw_circle(
                    x + dx,
                    y - h,
                    5.0,
                    Color::new(0.55, 0.95, 1.0, 0.12 * pulse),
                );
                draw_rectangle(
                    x + dx - 0.5,
                    y - h + 1.0,
                    1.0,
                    h,
                    Color::new(0.88, 0.90, 0.86, 1.0),
                );
                draw_ellipse(x + dx, y - h, 3.0, 2.0, 0.0, OUTLINE);
                draw_ellipse(
                    x + dx,
                    y - h,
                    2.5,
                    1.5,
                    0.0,
                    Color::new(0.55, 0.95, 1.0, pulse),
                );
            }
        }
        3 => {
            draw_rectangle(x - 13.0, y - 5.0, 26.0, 7.0, OUTLINE);
            draw_rectangle(x - 12.0, y - 4.0, 24.0, 5.0, wood);
            draw_ellipse(
                x - 12.0,
                y - 1.5,
                2.0,
                2.5,
                0.0,
                Color::new(0.75, 0.58, 0.40, 1.0),
            );
            draw_ellipse(
                x + 12.0,
                y - 1.5,
                2.0,
                2.5,
                0.0,
                Color::new(0.75, 0.58, 0.40, 1.0),
            );
        }
        4 => {
            draw_rectangle(x - 1.0, y - 16.0, 2.0, 18.0, wood);
            draw_rectangle(x - 6.0, y - 16.0, 12.0, 2.0, wood);
            for k in 0..3 {
                let sw = (time * 2.0 + k as f32).sin() * 1.5;
                let cx = x - 4.0 + k as f32 * 4.0;
                draw_line(
                    cx,
                    y - 14.0,
                    cx + sw,
                    y - 9.0,
                    1.0,
                    Color::new(0.85, 0.85, 0.80, 1.0),
                );
                draw_rectangle(
                    cx + sw - 1.0,
                    y - 9.0,
                    2.0,
                    3.0,
                    Color::new(0.70, 0.85, 0.95, 1.0),
                );
            }
        }
        // 저널 전용(0048). 6 반딧불 병: 유리병 안에서 빛이 떠다닌다.
        6 => {
            let glass = Color::new(0.70, 0.85, 0.90, 0.45);
            draw_rectangle(x - 6.0, y - 13.0, 12.0, 14.0, OUTLINE);
            draw_rectangle(x - 5.0, y - 12.0, 10.0, 12.0, glass);
            draw_rectangle(x - 3.0, y - 16.0, 6.0, 4.0, OUTLINE);
            draw_rectangle(x - 2.0, y - 15.0, 4.0, 2.0, wood);
            draw_circle(x, y - 6.0, 10.0, Color::new(0.85, 1.0, 0.55, 0.10));
            for k in 0..4 {
                let a = time * (1.2 + k as f32 * 0.3) + k as f32 * 1.7;
                let p = vec2(x + a.cos() * 3.0, y - 6.0 + (a * 1.3).sin() * 4.0);
                let c = Color::new(0.85, 1.0, 0.55, 0.6 + 0.4 * (a * 2.0).sin().abs());
                draw_rectangle(p.x.round(), p.y.round(), 1.0, 1.0, c);
            }
        }
        // 7 들꽃 화분.
        7 => {
            let pot = Color::new(0.72, 0.42, 0.30, 1.0);
            draw_rectangle(x - 6.0, y - 6.0, 12.0, 8.0, OUTLINE);
            draw_rectangle(x - 5.0, y - 5.0, 10.0, 6.0, pot);
            draw_rectangle(
                x - 5.0,
                y - 5.0,
                10.0,
                1.0,
                Color::new(0.86, 0.56, 0.40, 1.0),
            );
            let cols = [
                Color::new(1.0, 0.66, 0.77, 1.0),
                Color::new(0.80, 0.70, 1.0, 1.0),
                Color::new(1.0, 0.92, 0.55, 1.0),
            ];
            for (k, (dx, h)) in [(-3.0f32, 11.0f32), (0.0, 14.0), (3.0, 10.0)]
                .into_iter()
                .enumerate()
            {
                let sway = (time * 1.5 + k as f32).sin().round();
                draw_line(
                    x + dx,
                    y - 5.0,
                    x + dx + sway,
                    y - h,
                    1.0,
                    Color::new(0.40, 0.65, 0.30, 1.0),
                );
                draw_rectangle(x + dx + sway - 1.0, y - h - 1.0, 3.0, 3.0, cols[k]);
            }
        }
        // 8 실 그네: 두 기둥에 흰 실, 천천히 흔들린다.
        8 => {
            draw_rectangle(x - 9.0, y - 20.0, 2.0, 22.0, wood);
            draw_rectangle(x + 7.0, y - 20.0, 2.0, 22.0, wood);
            draw_rectangle(x - 9.0, y - 21.0, 18.0, 2.0, wood);
            let swing = (time * 1.2).sin() * 2.0;
            let thread = Color::new(0.92, 0.90, 0.96, 1.0);
            draw_line(x - 4.0, y - 19.0, x - 4.0 + swing, y - 6.0, 1.0, thread);
            draw_line(x + 4.0, y - 19.0, x + 4.0 + swing, y - 6.0, 1.0, thread);
            draw_rectangle(
                (x - 5.0 + swing).round(),
                y - 6.0,
                10.0,
                2.0,
                Color::new(0.78, 0.60, 0.38, 1.0),
            );
        }
        _ => {
            draw_rectangle(x - 17.0, y - 16.0, 2.0, 18.0, wood);
            draw_rectangle(x + 15.0, y - 16.0, 2.0, 18.0, wood);
            draw_line(
                x - 16.0,
                y - 15.0,
                x + 16.0,
                y - 15.0,
                1.0,
                Color::new(0.85, 0.80, 0.70, 1.0),
            );
            let cols = [
                Color::new(0.95, 0.45, 0.50, 1.0),
                Color::new(0.98, 0.85, 0.40, 1.0),
                Color::new(0.50, 0.75, 0.95, 1.0),
                Color::new(0.60, 0.85, 0.50, 1.0),
            ];
            for k in 0..6 {
                let fx = x - 13.0 + k as f32 * 5.0;
                let sag = ((k as f32 - 2.5).abs() - 2.5) * -0.6;
                draw_triangle(
                    vec2(fx, y - 15.0 + sag),
                    vec2(fx + 4.0, y - 15.0 + sag),
                    vec2(fx + 2.0, y - 10.0 + sag),
                    cols[k % 4],
                );
            }
        }
    }
}

/// 3/4 방: 바닥(풀·흙) + 장식, 좌·우·아래 벽 윗면, 위쪽은 벽 윗면 + 벼랑 앞면.
pub(super) fn draw_room_art(art: &Art, w: &World) {
    let tex = &art.tiles;
    let r = w.room.inner;
    let ft = w.biome.floor_tint();
    let wt = w.biome.wall_tint();
    // 바닥
    let cols = (t::VIEW_W / 16.0) as i32;
    let rows = (t::VIEW_H / 16.0).ceil() as i32;
    let dirt = w.biome.dirt_floor();
    for cy in 0..rows {
        for cx in 0..cols {
            let (_, pick) = biome::hash01(cx, cy, 1);
            let src = if dirt {
                tile_src(3, 0)
            } else {
                tile_src(pick % 3, 0)
            };
            draw_tile(tex, src, cx as f32 * 16.0, cy as f32 * 16.0, ft);
        }
    }
    // 반대 재질 얼룩. 칸 가운데를 반 칸 어긋나게 놓아 격자가 덜 보인다.
    for cy in 0..rows {
        for cx in 0..cols {
            let (roll, pick) = biome::hash01(cx, cy, 2);
            if roll < w.biome.patch_chance() {
                let row = if dirt { 3 } else { 2 };
                let src = tile_src(2 + pick % 2, row);
                let ox = ((pick >> 2) % 9) as f32 - 4.0;
                let oy = ((pick >> 6) % 9) as f32 - 4.0;
                draw_tile(tex, src, cx as f32 * 16.0 + ox, cy as f32 * 16.0 + oy, ft);
            }
        }
    }
    for (x, y, d) in floor_decor(w, Vec2::ZERO) {
        // 빛나는 버섯은 어둠 뒤에 한 번 더 그린다(draw_darkness).
        draw_tile(tex, decor_src(d), x, y, ft);
    }
    // 좌·우 벽(윗면 8px 폭)
    for cy in 0..rows {
        let y = cy as f32 * 16.0;
        draw_tile(tex, Rect::new(0.0, 16.0, 8.0, 16.0), 0.0, y, wt);
        draw_tile(tex, Rect::new(8.0, 16.0, 8.0, 16.0), r.right(), y, wt);
    }
    // 아래 벽(윗면 위쪽 8px)
    for cx in 0..cols {
        draw_tile(
            tex,
            Rect::new(0.0, 16.0, 16.0, 8.0),
            cx as f32 * 16.0,
            r.bottom(),
            wt,
        );
    }
    // 위쪽: 윗면(아래 8px, 풀이 늘어진 가장자리) + 벼랑 앞면 16px
    let face_y = r.y - 16.0;
    for cx in 0..cols {
        let x = cx as f32 * 16.0;
        draw_tile(tex, Rect::new(0.0, 24.0, 16.0, 8.0), x, face_y - 8.0, wt);
        let (roll, _) = biome::hash01(cx, 0, 3);
        let roots = match w.biome {
            crate::biome::Biome::Meadow => roll < 0.15,
            crate::biome::Biome::Roots => roll < 0.55,
            crate::biome::Biome::Deep => roll < 0.8,
            crate::biome::Biome::Silk => roll < 0.6,
        };
        let src = if roots {
            tile_src(2, 1)
        } else {
            tile_src(1, 1)
        };
        draw_tile(tex, src, x, face_y, wt);
    }
    // 벼랑 아래 그늘
    draw_rectangle(r.x, r.y, r.w, 2.0, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_rectangle(r.x, r.y + 2.0, r.w, 2.0, Color::new(0.0, 0.0, 0.0, 0.12));
}

/// 반딧불(빛나는 꽁무니 + 날개 깜빡임, 둥실) / 산딸기(빨간 열매 + 잎).
/// `light_only`: 어둠 위에 빛만 다시 그린다(심층에서도 반딧불이 보이게).
pub(super) fn draw_loot(l: &loot::Loot, alpha: f32, time: f32, light_only: bool) {
    let q = lerp(l.prev_pos, l.pos, alpha);
    let ph = time * 6.0 + l.phase * 10.0;
    match l.kind {
        loot::LootKind::Glowbug => {
            // 바닥에 내려앉지 않고 늘 살짝 떠 있다.
            let hover = 3.0 + (ph * 0.5).sin() * 1.0;
            let p = vec2(q.x.round(), (q.y - l.z - hover).round());
            let pulse = 0.55 + 0.45 * (ph * 0.8).sin().abs();
            let halo = Color::new(world::GLOW.r, world::GLOW.g, world::GLOW.b, 0.18 * pulse);
            draw_circle(p.x + 0.5, p.y + 1.5, 4.0, halo);
            draw_rectangle(
                p.x,
                p.y + 1.0,
                2.0,
                2.0,
                Color::new(1.0, 1.0, 0.7, 0.6 + 0.4 * pulse),
            );
            if light_only {
                return;
            }
            // 몸(짙은 머리) + 날개 깜빡임
            draw_rectangle(p.x, p.y - 1.0, 2.0, 2.0, OUTLINE);
            if (ph as i32) % 2 == 0 {
                draw_rectangle(
                    p.x - 1.0,
                    p.y - 2.0,
                    1.0,
                    1.0,
                    Color::new(0.9, 0.95, 1.0, 0.8),
                );
                draw_rectangle(
                    p.x + 2.0,
                    p.y - 2.0,
                    1.0,
                    1.0,
                    Color::new(0.9, 0.95, 1.0, 0.8),
                );
            }
        }
        loot::LootKind::Worm => {
            if light_only {
                return;
            }
            let p = vec2(q.x.round(), (q.y - l.z).round() - 2.0);
            draw_worm(p, ph, 1.0);
        }
        loot::LootKind::RootHeart => {
            // 뿌리 심장: 뿌리가 감싼 붉은 심장, 둥실 + 은은한 빛.
            let hover = 2.0 + (ph * 0.4).sin();
            let p = vec2(q.x.round(), (q.y - l.z - hover).round() - 3.0);
            let pulse = 0.6 + 0.4 * (ph * 0.5).sin().abs();
            draw_circle(
                p.x + 0.5,
                p.y + 1.0,
                7.0,
                Color::new(1.0, 0.4, 0.45, 0.15 * pulse),
            );
            if light_only {
                return;
            }
            let red = Color::new(0.90, 0.25, 0.32, 1.0);
            let dark = Color::new(0.60, 0.12, 0.22, 1.0);
            let root = Color::new(0.45, 0.32, 0.22, 1.0);
            // 하트: 위 두 혹 + 아래 뾰족
            draw_rectangle(p.x - 4.0, p.y - 3.0, 9.0, 4.0, OUTLINE);
            draw_rectangle(p.x - 3.0, p.y + 1.0, 7.0, 2.0, OUTLINE);
            draw_rectangle(p.x - 2.0, p.y + 3.0, 5.0, 1.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y + 4.0, 3.0, 1.0, OUTLINE);
            draw_rectangle(p.x - 3.0, p.y - 2.0, 3.0, 3.0, red);
            draw_rectangle(p.x + 1.0, p.y - 2.0, 3.0, 3.0, red);
            draw_rectangle(p.x - 2.0, p.y + 1.0, 5.0, 2.0, red);
            draw_rectangle(p.x - 1.0, p.y + 3.0, 3.0, 1.0, dark);
            draw_rectangle(p.x + 2.0, p.y, 2.0, 1.0, dark);
            draw_rectangle(
                p.x - 2.0,
                p.y - 2.0,
                1.0,
                1.0,
                Color::new(1.0, 0.8, 0.8, 1.0),
            );
            // 감싼 뿌리
            draw_rectangle(p.x - 3.0, p.y + 1.0, 1.0, 1.0, root);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 1.0, 3.0, root);
            draw_rectangle(p.x + 1.0, p.y + 2.0, 2.0, 1.0, root);
        }
        loot::LootKind::Seed => {
            // 씨앗: 통통한 갈색 알 + 초록 새싹 한 잎. 살짝 둥실.
            let hover = 1.0 + (ph * 0.7).sin() * 0.5;
            let p = vec2(q.x.round(), (q.y - l.z - hover).round() - 2.0);
            if light_only {
                return;
            }
            draw_rectangle(p.x - 2.0, p.y - 1.0, 4.0, 4.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 2.0, 2.0, 6.0, OUTLINE);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 2.0, 4.0, world::SEED);
            draw_rectangle(p.x, p.y + 1.0, 1.0, 2.0, Color::new(0.62, 0.44, 0.24, 1.0));
            draw_rectangle(
                p.x - 1.0,
                p.y - 1.0,
                1.0,
                1.0,
                Color::new(1.0, 0.92, 0.72, 1.0),
            );
            draw_rectangle(p.x, p.y - 3.0, 2.0, 1.0, Color::new(0.45, 0.78, 0.35, 1.0));
        }
        loot::LootKind::Berry => {
            if light_only {
                return;
            }
            // 산딸기(0178): 알이 모인 송이 + 꽃받침. 가방 칸도 같은 그림(`draw_berry`).
            draw_berry(q.x.round(), (q.y - l.z).round() - 4.0, 1.0, 1.0);
        }
    }
}

/// 바닥 무기: 떠오르는 발톱 세 개 + 반짝임.
pub(super) fn draw_pickup(p: &world::Pickup, time: f32) {
    let c = crate::equipment::WEAPONS[p.weapon].color;
    let dark = Color::new(c.r * 0.55, c.g * 0.55, c.b * 0.55, 1.0);
    let bob = ((time * 3.0).sin() * 1.5).round();
    let base = vec2(p.pos.x.round(), p.pos.y.round() - 6.0 + bob);
    for k in 0..3 {
        let x = base.x - 4.0 + k as f32 * 3.0;
        let y = base.y + (k % 2) as f32;
        draw_rectangle(x - 0.5, y - 1.0, 3.0, 8.0, OUTLINE);
        draw_rectangle(x, y, 2.0, 6.0, c);
        draw_rectangle(x, y + 5.0, 2.0, 1.0, dark);
    }
    let ph = (time * 2.0).fract();
    let s = base + vec2(5.0, -3.0) + vec2(0.0, -ph * 4.0);
    let a = 1.0 - ph;
    draw_rectangle(s.x, s.y - 1.0, 1.0, 3.0, Color::new(1.0, 1.0, 1.0, a));
    draw_rectangle(s.x - 1.0, s.y, 3.0, 1.0, Color::new(1.0, 1.0, 1.0, a));
}

/// 환경 어둠: 플레이어 주변만 밝고 바깥은 어둡다. 빛나는 버섯은 어둠 위에 다시.
pub(super) fn draw_darkness(art: &Art, w: &World, alpha: f32, cam: Vec2) {
    let (dark, radius) = w.biome.darkness();
    if dark <= 0.0 {
        return;
    }
    let c = lerp(w.player.prev_pos, w.player.pos, alpha);
    let s = radius * 2.0;
    let (x0, y0) = (c.x - radius, c.y - radius);
    let col = Color::new(1.0, 1.0, 1.0, dark);
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
    let black = Color::new(0.0, 0.0, 0.0, dark);
    // 보이는 화면(카메라 기준, 흔들림 여유 8px)에서 빛 원 바깥을 덮는다.
    let (l, tp) = (cam.x - 8.0, cam.y - 8.0);
    let (r, b) = (cam.x + t::VIEW_W + 8.0, cam.y + t::VIEW_H + 8.0);
    draw_rectangle(l, tp, r - l, (y0 - tp).max(0.0), black);
    draw_rectangle(l, y0 + s, r - l, (b - (y0 + s)).max(0.0), black);
    draw_rectangle(l, y0, (x0 - l).max(0.0), s, black);
    draw_rectangle(x0 + s, y0, (r - (x0 + s)).max(0.0), s, black);
    draw_dark_extras(art, w, alpha, cam);
}

/// 어둠 위에 한 번 더 그리는 것: 반딧불과 빛나는 버섯(어둠에 묻히지 않게).
pub(super) fn draw_dark_extras(art: &Art, w: &World, alpha: f32, cam: Vec2) {
    for l in &w.loot {
        draw_loot(l, alpha, w.time, true);
    }
    draw_mushrooms(art, w, cam);
}

/// 빛나는 버섯. 어둠 지도(0095)가 있으면 바닥 높이(캐릭터 밑)에 그리고 버섯 빛이 둘레를 밝힌다.
/// 없으면 예전처럼 어둠 위에 한 번 더 그린다(그때는 캐릭터 위에 겹칠 수 있다).
pub(super) fn draw_mushrooms(art: &Art, w: &World, cam: Vec2) {
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.65 + 0.25 * ((w.time * 2.0 + x * 0.1).sin());
            // 팩이 있으면 파란 버섯 4칸(0066). 굽기에서는 빼 두었다.
            if let Some(sunny) = &art.sunny {
                let s = crate::sunny::MUSH_FRAME;
                let f = ((w.time * 4.0 + x * 0.13) as u32 % 4) as f32;
                for r in [13.0, 9.0, 5.0] {
                    draw_circle(
                        x + 8.0,
                        y + 10.0,
                        r,
                        Color::new(0.5, 0.85, 1.0, 0.035 * pulse),
                    );
                }
                draw_texture_ex(
                    &sunny.mush,
                    x,
                    y,
                    Color::new(1.0, 1.0, 1.0, 0.75 + 0.25 * pulse),
                    DrawTextureParams {
                        source: Some(Rect::new(f * s, 0.0, s, s)),
                        ..Default::default()
                    },
                );
                continue;
            }
            draw_tile(
                &art.tiles,
                decor_src(d),
                x,
                y,
                Color::new(1.0, 1.0, 1.0, pulse),
            );
        }
    }
}

/// 바닥 위 타원(축 `g` 방향 반지름 `along`, 옆 `across`)을 3/4 시점으로(세로 ×0.5) 눌렀을 때 화면의 (가로 반지름, 세로 반지름, 각도°).
/// 눌린 타원의 공분산 M·Mᵀ(M = 누름 × 돌림 × 반지름)의 고유값·고유벡터(0150).
pub(super) fn ground_ellipse(g: Vec2, along: f32, across: f32) -> (f32, f32, f32) {
    let n = vec2(-g.y, g.x);
    let (u, v) = (vec2(g.x, g.y * 0.5) * along, vec2(n.x, n.y * 0.5) * across);
    let (xx, yy, xy) = (
        u.x * u.x + v.x * v.x,
        u.y * u.y + v.y * v.y,
        u.x * u.y + v.x * v.y,
    );
    let mid = (xx + yy) * 0.5;
    let d = (((xx - yy) * 0.5).powi(2) + xy * xy).sqrt();
    let ang = 0.5 * (2.0 * xy).atan2(xx - yy);
    (
        (mid + d).sqrt(),
        (mid - d).max(0.0).sqrt(),
        ang.to_degrees(),
    )
}
