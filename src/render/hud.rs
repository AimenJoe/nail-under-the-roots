//! HUD·화면 덮개: 보스·할매 막대, 출구 표시, 퀵 슬롯, 터치 오버레이, 타이틀 바탕, 체력 막대, 거절 표시, 디버그 오버레이·히트박스.

use super::*;

use crate::characters::{CharacterDef, Icon};
use crate::input::TouchKind;

/// 보스 체력 막대(화면 위 가운데): 이름 + 막대. 깨어 있는 보스가 있을 때만.
/// 위에 둔다: 던전 방(192px)이 화면(180px)보다 커서 방 아래쪽에 서면 아래 막대가 Naïl과 보스를 가렸다(0046).
pub(super) fn draw_boss_bar(ui: &Ui, w: &World) {
    let Some(b) = w.enemies.iter().find(|e| e.is_boss() && !e.asleep) else {
        return;
    };
    let name = crate::i18n::tr(b.sp().name);
    let bar = Rect::new(90.0, 4.0 + crate::ui::FONT_PX + 4.0, 140.0, 4.0);
    let nw = ui.text_w(name, crate::ui::FONT_PX) + 10.0;
    ui.rect(
        Rect::new(160.0 - nw / 2.0, 2.0, nw, crate::ui::FONT_PX + 3.0),
        Color::new(0.07, 0.05, 0.09, 0.75),
    );
    ui.text_center(name, 160.0, 3.0, crate::ui::FONT_PX, crate::ui::ACCENT);
    ui.rect(
        Rect::new(bar.x - 1.0, bar.y - 1.0, bar.w + 2.0, bar.h + 2.0),
        OUTLINE,
    );
    ui.rect(bar, Color::new(0.20, 0.10, 0.14, 1.0));
    let k = (b.hp.max(0) as f32 / b.max_hp.max(1) as f32).clamp(0.0, 1.0);
    // 2단계(0233): 깨어나면 막대가 짙은 붉은 보라로 숨 쉰다.
    let (fill, top) = if b.roused {
        let p = 0.5 + 0.5 * (get_time() as f32 * 6.0).sin();
        (
            Color::new(0.78 + 0.12 * p, 0.22, 0.42, 1.0),
            Color::new(1.0, 0.55, 0.75, 1.0),
        )
    } else {
        (
            Color::new(0.85, 0.30, 0.38, 1.0),
            Color::new(1.0, 0.6, 0.62, 1.0),
        )
    };
    ui.rect(Rect::new(bar.x, bar.y, (bar.w * k).round(), bar.h), fill);
    ui.rect(Rect::new(bar.x, bar.y, (bar.w * k).round(), 1.0), top);
    // 반 자리 눈금: 여기 아래에서 보스가 2단계로 깨어난다.
    if !b.roused {
        ui.rect(
            Rect::new((bar.x + bar.w / 2.0).round(), bar.y - 1.0, 1.0, bar.h + 2.0),
            OUTLINE,
        );
    }
}

/// 가장 깊은 곳(0162)의 위쪽 표시: 할매 이름 + 매듭 셋(남은 것은 보라, 끊은 것은 풀린 흰 실) + 감긴 수(실 고리).
pub(super) fn draw_deep_bar(ui: &Ui, w: &World) {
    let Some(d) = &w.deep else {
        return;
    };
    let name = crate::i18n::s().name_halmae;
    let nw = ui.text_w(name, crate::ui::FONT_PX) + 10.0;
    ui.rect(
        Rect::new(160.0 - nw / 2.0, 2.0, nw, crate::ui::FONT_PX + 3.0),
        Color::new(0.07, 0.05, 0.09, 0.75),
    );
    ui.text_center(name, 160.0, 3.0, crate::ui::FONT_PX, crate::ui::ACCENT);
    let y = 4.0 + crate::ui::FONT_PX + 5.0;
    for (i, k) in d.knots.iter().enumerate() {
        let x = 160.0 - 16.0 + i as f32 * 16.0;
        if k.cut {
            ui.rect(
                Rect::new(x - 4.0, y + 2.0, 8.0, 1.0),
                Color::new(0.85, 0.80, 0.92, 0.9),
            );
            ui.rect(
                Rect::new(x - 2.0, y, 5.0, 1.0),
                Color::new(0.85, 0.80, 0.92, 0.9),
            );
        } else {
            ui.rect(Rect::new(x - 4.0, y - 1.0, 8.0, 6.0), OUTLINE);
            ui.rect(
                Rect::new(x - 3.0, y, 6.0, 4.0),
                Color::new(0.55, 0.35, 0.78, 1.0),
            );
            let worn = (t::DEEP_KNOT_HP - k.hp).max(0) as f32 / t::DEEP_KNOT_HP as f32;
            ui.rect(
                Rect::new(x - 3.0, y, (6.0 * worn).round(), 4.0),
                Color::new(0.90, 0.85, 1.0, 0.8),
            );
        }
    }
    // 감긴 수: 오른쪽에 작은 고리, 감긴 만큼 보라로 찬다.
    for i in 0..t::DEEP_WRAPS {
        let x = 160.0 + 34.0 + i as f32 * 7.0;
        let c = if i < d.wraps {
            Color::new(0.80, 0.62, 0.98, 1.0)
        } else {
            Color::new(0.35, 0.28, 0.42, 0.9)
        };
        ui.rect(Rect::new(x, y, 4.0, 4.0), c);
        ui.rect(
            Rect::new(x + 1.0, y + 1.0, 2.0, 2.0),
            Color::new(0.07, 0.05, 0.09, 1.0),
        );
    }
}

/// 퀵 슬롯(왼쪽 아래): 산딸기·지렁이 두 칸, 고른 칸은 테두리 강조, 개수, 먹기 키 안내.
/// HUD 바탕판(0091): 풀밭·흙 위에서도 HUD가 묻히지 않게 반투명 어두운 판 + 어두운 테두리.
/// 끝방 출구가 화면 밖일 때(0159) 가장자리에 구멍 표시와 그쪽을 가리키는 화살을 띄운다. 보상을 줍는 동안
/// (`EXIT_MARK_DELAY`)은 조용히. 화면 안이면 출구 자체가 보이니 아무것도 안 그린다.
pub(super) fn draw_exit_mark(ui: &Ui, w: &World) {
    let Some(e) = w.exit_at else {
        return;
    };
    if w.adv.fade.is_some() || w.exit_age < t::EXIT_MARK_DELAY {
        return;
    }
    let v = e - w.adv.cam;
    let inside = Rect::new(-8.0, -8.0, t::VIEW_W + 16.0, t::VIEW_H + 16.0);
    if inside.contains(v) {
        return;
    }
    // 화면 가운데에서 출구 쪽으로 나가는 선이 안쪽 테두리와 만나는 점.
    let c = vec2(t::VIEW_W / 2.0, t::VIEW_H / 2.0 + 4.0);
    let d = v - c;
    let bounds = Rect::new(14.0, 18.0, t::VIEW_W - 28.0, t::VIEW_H - 40.0);
    let k = {
        let kx = if d.x.abs() < 1e-3 {
            f32::MAX
        } else if d.x > 0.0 {
            (bounds.right() - c.x) / d.x
        } else {
            (bounds.x - c.x) / d.x
        };
        let ky = if d.y.abs() < 1e-3 {
            f32::MAX
        } else if d.y > 0.0 {
            (bounds.bottom() - c.y) / d.y
        } else {
            (bounds.y - c.y) / d.y
        };
        kx.min(ky).min(1.0)
    };
    let p = (c + d * k).round();
    let pulse = 0.55 + 0.45 * (w.time * 4.0).sin();
    let dir = d.normalize_or(vec2(1.0, 0.0));
    // 작은 구멍 아이콘 + 화살.
    ui.rect(
        Rect::new(p.x - 6.0, p.y - 4.0, 12.0, 8.0),
        Color::new(0.07, 0.05, 0.09, 0.6),
    );
    ui.rect(
        Rect::new(p.x - 4.0, p.y - 2.0, 8.0, 4.0),
        Color::new(0.16, 0.11, 0.14, 1.0),
    );
    ui.rect(
        Rect::new(p.x - 2.0, p.y - 1.0, 4.0, 2.0),
        Color::new(0.0, 0.0, 0.0, 1.0),
    );
    let tip = p + dir * 11.0;
    let n = vec2(-dir.y, dir.x);
    let col = Color::new(1.0, 0.92, 0.62, pulse);
    ui.tri(tip, p + dir * 5.0 + n * 4.0, p + dir * 5.0 - n * 4.0, col);
}

pub(super) fn hud_plate(ui: &Ui, r: Rect) {
    ui.rect(r, Color::new(0.06, 0.04, 0.08, 0.62));
    ui.rect_lines(r, Color::new(0.02, 0.01, 0.03, 0.75));
}

/// 퀵 슬롯(0091): 칸 22px·아이콘 2배, 고른 칸은 두꺼운 금빛 테두리, 안 고른 칸은 흐리게.
/// 저절로 바뀐 칸(`Bag::flash`)은 하얗게 깜빡인다. 수는 테두리 글자로 칸 오른쪽 아래.
pub(super) fn draw_quick_slots(ui: &Ui, w: &World) {
    use crate::items::Consumable;
    let (x0, y0, size, gap) = (5.0 + deny_shake(w.juice.deny[1]), 153.0, 22.0, 3.0);
    let hint = crate::i18n::fill(
        crate::i18n::s().quick_hint,
        &[&crate::ui::key(crate::input::prompt(
            crate::input::Act::UseItem,
        ))],
    );
    let hx = x0 + 2.0 * (size + gap) + 2.0;
    let hw = ui.text_w(&hint, crate::ui::FONT_PX);
    hud_plate(
        ui,
        Rect::new(x0 - 3.0, y0 - 3.0, hx + hw + 4.0 - (x0 - 3.0), size + 6.0),
    );
    let out = ui.with_outline();
    let flash_on = w.inv.bag.flash > 0.0 && ((w.inv.bag.flash * 8.0) as i32) % 2 == 0;
    for (i, c) in Consumable::ALL.iter().enumerate() {
        let r = Rect::new(x0 + i as f32 * (size + gap), y0, size, size);
        let n = w.inv.bag.count(*c);
        let selected = w.inv.bag.selected == *c;
        ui.rect(
            r,
            if selected {
                Color::new(0.20, 0.14, 0.17, 0.95)
            } else {
                Color::new(0.07, 0.05, 0.09, 0.85)
            },
        );
        let border = if selected && w.juice.deny[1] > 0.0 {
            deny_color(w.juice.deny[1])
        } else if selected && flash_on {
            WHITE
        } else if selected {
            crate::ui::ACCENT
        } else {
            Color::new(0.55, 0.40, 0.28, 0.6)
        };
        ui.rect_lines(r, border);
        if selected {
            ui.rect_lines(
                Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0),
                border,
            );
        }
        let dim = if n == 0 {
            0.3
        } else if selected {
            1.0
        } else {
            0.65
        };
        let center = ui.px(vec2(r.x + size / 2.0 - 2.0, r.y + size / 2.0 - 1.0));
        let k = ui.scale * 2.0;
        match c {
            Consumable::Berry => {
                // 산딸기(0178): 바닥 아이템과 같은 송이 그림, 칸 크기에 맞춰 반 배율.
                super::props::draw_berry(center.x, center.y + k * 0.5, k * 0.5, dim);
            }
            Consumable::Worm => {
                if n > 0 {
                    draw_worm(center, w.time * 6.0, k * 0.75);
                } else {
                    draw_rectangle(
                        center.x - 4.0 * k,
                        center.y - 0.5 * k,
                        8.0 * k,
                        2.0 * k,
                        Color::new(0.4, 0.3, 0.35, 1.0),
                    );
                }
            }
        }
        // 다음 먹기까지: 위에서부터 어둡게 덮였다가 걷힌다(0038).
        let cd = w.player.item_cd / (t::ITEM_COOLDOWN * w.player.eat_cd_mul);
        if cd > 0.0 {
            let h = (size * cd).round();
            ui.rect(
                Rect::new(r.x, r.y, size, h),
                Color::new(0.0, 0.0, 0.0, 0.55),
            );
        }
        let ns = n.to_string();
        out.text(
            &ns,
            r.right() - ui.text_w(&ns, crate::ui::FONT_PX) - 2.0,
            r.bottom() - 13.0,
            crate::ui::FONT_PX,
            if n == 0 {
                Color::new(0.85, 0.45, 0.45, 1.0)
            } else {
                crate::ui::TEXT
            },
        );
    }
    out.text(&hint, hx, y0 + 5.0, crate::ui::FONT_PX, crate::ui::TEXT);
    // 못 먹었다(0149·0150): 고른 칸 위에 ✕.
    if w.juice.deny[1] > 0.0 {
        let k = Consumable::ALL
            .iter()
            .position(|c| *c == w.inv.bag.selected)
            .unwrap_or(0) as f32;
        deny_cross(
            ui,
            vec2(x0 + k * (size + gap) + size / 2.0, y0 + size / 2.0),
            4.0,
        );
    }
}

/// 타이틀 배경: 격자 위에 뿌리 몇 가닥.
pub(super) fn draw_title_background() {
    for gx in (0..VW).step_by(16) {
        draw_line(gx as f32, 0.0, gx as f32, t::VIEW_H, 1.0, GRID);
    }
    for gy in (0..VH).step_by(16) {
        draw_line(0.0, gy as f32, t::VIEW_W, gy as f32, 1.0, GRID);
    }
    let root = Color::new(0.30, 0.22, 0.20, 1.0);
    for i in 0..5 {
        let x0 = 40.0 + i as f32 * 60.0;
        let mut prev = vec2(x0, 0.0);
        for s in 1..=12 {
            let y = s as f32 * 15.0;
            let x = x0 + ((y * 0.05 + i as f32).sin() * 14.0).round();
            draw_line(prev.x, prev.y, x, y, 2.0 + (12 - s) as f32 * 0.15, root);
            prev = vec2(x, y);
        }
    }
    draw_rectangle(0.0, t::VIEW_H - 8.0, t::VIEW_W, 8.0, WALL_COLOR);
}

/// 터치 조작 오버레이(실제 해상도). 지금 장치가 터치일 때만(0172). 버튼 그림은 캐릭터 표에서.
pub fn draw_touch_overlay(t: &TouchOverlay, ch: &CharacterDef) {
    let line = Color::new(1.0, 1.0, 1.0, 0.35);
    let fill = Color::new(1.0, 1.0, 1.0, 0.12);
    if let Some((origin, pos)) = t.stick {
        draw_circle_lines(origin.x, origin.y, t.stick_radius, 2.0, line);
        let d = pos - origin;
        let knob = origin + d.clamp_length_max(t.stick_radius);
        draw_circle(knob.x, knob.y, t.stick_radius * 0.4, fill);
        draw_circle_lines(knob.x, knob.y, t.stick_radius * 0.4, 2.0, line);
    }
    let faint = Color::new(1.0, 1.0, 1.0, 0.15);
    for b in &t.buttons {
        draw_circle(b.center.x, b.center.y, b.radius, fill);
        // 기다리는 중이면 테두리가 시계 방향으로 다시 차오른다(굴파기·필살기·아이템, 0038).
        let ready = b.ready.clamp(0.0, 1.0);
        if ready < 1.0 {
            draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, faint);
            draw_arc(
                b.center.x,
                b.center.y,
                48,
                b.radius,
                -90.0,
                2.0,
                360.0 * ready,
                line,
            );
        } else {
            draw_circle_lines(b.center.x, b.center.y, b.radius, 2.0, line);
        }
        let line = if ready < 1.0 { faint } else { line };
        let icon = match b.kind {
            TouchKind::Attack => ch.touch[0],
            TouchKind::Dodge => ch.touch[1],
            TouchKind::Special => ch.touch[2],
            TouchKind::Item => Icon::Berry,
            TouchKind::ItemSwitch => Icon::Switch,
            TouchKind::Menu => Icon::Menu,
        };
        draw_touch_icon(icon, b.center, b.radius, line);
    }
}

/// 터치 버튼의 픽토그램(0172): 글자 대신 — 글자는 읽어야 하고 언어를 탄다. 판정 11(사장, 2026-10-29: "터치 아이콘이 아직
/// 다듬어지지 않은 느낌")로 선 그림을 12×12 도트 그림으로 바꿨다 — 흰 몸 + 어두운 1칸 테두리, 버튼 크기에 맞춘 정수 배율.
/// 그림 원본은 아래 `icon_mask`(`#` = 칠한 칸). 고치면 시험 `touch_icons_are_12_by_12`.
fn draw_touch_icon(icon: Icon, c: Vec2, r: f32, col: Color) {
    let mask = icon_mask(icon);
    let px = ((r * 1.15) / 12.0).floor().max(1.0);
    let size = 12.0 * px;
    let (x0, y0) = ((c.x - size / 2.0).round(), (c.y - size / 2.0).round());
    // `col`의 투명도(쓸 수 있음 0.35 / 기다리는 중 0.15)를 몸·테두리 진하기로.
    let fill = Color::new(1.0, 1.0, 1.0, (col.a * 2.5).min(0.92));
    let edge = Color::new(OUTLINE.r, OUTLINE.g, OUTLINE.b, (col.a * 1.6).min(0.6));
    let cells = || {
        mask.iter().enumerate().flat_map(|(y, row)| {
            row.bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'#')
                .map(move |(x, _)| vec2(x as f32, y as f32))
        })
    };
    for q in cells() {
        draw_rectangle(
            x0 + (q.x - 1.0) * px,
            y0 + (q.y - 1.0) * px,
            3.0 * px,
            3.0 * px,
            edge,
        );
    }
    for q in cells() {
        draw_rectangle(x0 + q.x * px, y0 + q.y * px, px, px, fill);
    }
}

/// 터치 아이콘 12×12(`#` = 칠함). 캐릭터 표(`characters::Icon`)가 버튼마다 고른다.
fn icon_mask(icon: Icon) -> [&'static str; 12] {
    match icon {
        Icon::Claw => [
            "##...##...##",
            ".##...##...#",
            ".##...##...#",
            "..##...##...",
            "..##...##...",
            "...##...##..",
            "...##...##..",
            "....##...##.",
            "....##...##.",
            ".....#....##",
            ".....#.....#",
            "............",
        ],
        Icon::Paw => [
            "..##....##..",
            ".####..####.",
            ".####..####.",
            "..##....##..",
            "##...##...##",
            "##..####..##",
            "...######...",
            "..########..",
            "..########..",
            "..########..",
            "...######...",
            "............",
        ],
        Icon::Mound => [
            "....####....",
            "....####....",
            "....####....",
            ".##########.",
            "..########..",
            "...######...",
            "....####....",
            ".....##.....",
            "............",
            "...######...",
            ".##########.",
            "############",
        ],
        Icon::Hop => [
            ".....####...",
            "...##....##.",
            "..#.......##",
            ".#.......###",
            ".#......####",
            "#...........",
            "#...........",
            "#...........",
            "............",
            "............",
            "############",
            "############",
        ],
        Icon::Drill => [
            "............",
            "##..........",
            "###.###.....",
            "###..####...",
            "###.#..#####",
            "#####.#..###",
            "#####..#.###",
            "###.#..#####",
            "###..####...",
            "###.###.....",
            "##..........",
            "............",
        ],
        Icon::Chain => [
            "............",
            ".........###",
            "........#...",
            "........#..#",
            "....##..#...",
            "...#..#.....",
            "...#..#.....",
            ".##.........",
            "#..#........",
            "#..#........",
            "............",
            "############",
        ],
        Icon::Acorn => [
            ".....##.....",
            "..########..",
            ".##########.",
            "############",
            "############",
            "............",
            ".##########.",
            ".##########.",
            "..########..",
            "...######...",
            "....####....",
            ".....##.....",
        ],
        Icon::Leap => [
            "...####.....",
            ".##....##...",
            "##.......#..",
            "###.......#.",
            "####......#.",
            "...........#",
            "...........#",
            "...........#",
            "............",
            "............",
            "############",
            "############",
        ],
        Icon::Bury => [
            ".....##.....",
            "...######...",
            "..########..",
            "..########..",
            "............",
            "..########..",
            "############",
            "############",
            ".#..#..#..#.",
            "............",
            "..#..#..#...",
            "............",
        ],
        Icon::Berry => [
            "....#.###...",
            ".....###....",
            "...######...",
            "..##.##.##..",
            ".##########.",
            ".##.##.##.#.",
            ".##########.",
            "..#.##.##.#.",
            "..########..",
            "...######...",
            "....####....",
            "............",
        ],
        Icon::Switch => [
            "............",
            "...#....#...",
            "..##....##..",
            ".##......##.",
            "##........##",
            "###......###",
            "##........##",
            ".##......##.",
            "..##....##..",
            "...#....#...",
            "............",
            "............",
        ],
        Icon::Menu => [
            "............",
            "............",
            "############",
            "############",
            "............",
            "............",
            "############",
            "############",
            "............",
            "............",
            "############",
            "############",
        ],
    }
}

pub(super) fn draw_hitboxes(w: &World, alpha: f32) {
    let c = Color::new(0.2, 1.0, 0.2, 0.8);
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    draw_circle_lines(pp.x, pp.y, w.player.radius(), 1.0, c);
    for e in &w.enemies {
        let p = lerp(e.prev_pos, e.pos, alpha);
        let col = if e.contact_damage().is_some() { RED } else { c };
        draw_circle_lines(p.x, p.y, e.radius(), 1.0, col);
    }
    for p in &w.projectiles {
        let q = lerp(p.prev_pos, p.pos, alpha);
        draw_circle_lines(q.x, q.y, p.radius(), 1.0, RED);
    }
    if let Some(s) = w.last_strike {
        let col = match s.kind {
            StrikeKind::Claw(_) => YELLOW,
            StrikeKind::Heavy => GOLD,
            StrikeKind::Emerge => ORANGE,
            StrikeKind::Drill => SKYBLUE,
            StrikeKind::DrillWall => RED,
            StrikeKind::Stomp => PINK,
            StrikeKind::Toss => LIGHTGRAY,
            StrikeKind::Burst => BROWN,
            StrikeKind::Shove => WHITE,
        };
        match s.shape {
            Shape::Circle { center, r } => draw_circle_lines(center.x, center.y, r, 1.0, col),
            Shape::Sector { origin, dir, r } => draw_sector_lines(origin, dir, r, col),
        }
    }
}

/// 체력 핍(왼쪽 위)·굴파기 쿨다운·SP 게이지(오른쪽 위). 화면 패스에서 게임 px 격자로(ui.rect).
/// 체력 한 칸의 간격: 팩 하트(9px) 또는 직접 그린 네모(6px).
pub(super) fn hp_step(sheet: Option<&Texture2D>) -> f32 {
    if sheet.is_some() {
        10.0
    } else {
        7.0
    }
}

pub(super) fn draw_hud_bars(ui: &Ui, w: &World, sheet: Option<&Texture2D>) {
    let dark = Color::new(0.2, 0.15, 0.2, 1.0);
    let step = hp_step(sheet);
    for i in 0..w.player.stats.max_hp {
        let x = 3.0 + i as f32 * step;
        match sheet {
            // 빈 칸은 같은 하트를 어둡게(0077).
            Some(sheet) => {
                let tint = if i < w.player.hp {
                    WHITE
                } else {
                    Color::new(0.22, 0.17, 0.24, 1.0)
                };
                ui.sprite(sheet, crate::sunny::rect(crate::sunny::HEART), x, 0.0, tint);
            }
            None => {
                let c = if i < w.player.hp { world::MOLE } else { dark };
                ui.rect(Rect::new(x, 1.0, 6.0, 6.0), c);
            }
        }
    }
    // 실뿌리(0210): 체력 아래 셋. 쓰러져 쉼터에서 깨어날 때마다 하나씩 감긴다(밝은 갈색). 다 감긴 뒤 또 쓰러지면 런 끝.
    // 쉼터를 지난 런에서만(도움 모드·필드에서는 세지 않는다). 모양(꼬인 뿌리 vs 빈 칸 점)으로도 갈린다.
    if w.strands_shown() {
        let y = if sheet.is_some() { 12.0 } else { 10.0 };
        hud_plate(
            ui,
            Rect::new(1.0, y - 2.0, 3.0 + t::RUN_LIVES as f32 * 9.0, 8.0),
        );
        // 물결 모양 실뿌리 한 가닥(6×3).
        const WAVE: [(f32, f32); 6] = [
            (0.0, 2.0),
            (1.0, 1.0),
            (2.0, 0.0),
            (3.0, 1.0),
            (4.0, 2.0),
            (5.0, 1.0),
        ];
        // 런을 끝내는 쓰러짐(셋째)이면 쓰러진 동안 셋째도 감긴 것으로.
        let wound = w.adv.falls + u32::from(!w.player.is_alive() && w.run_over());
        for i in 0..t::RUN_LIVES {
            let x = 4.0 + i as f32 * 9.0;
            let (fill, edge) = if i < wound {
                (
                    Color::new(0.86, 0.62, 0.38, 1.0),
                    Color::new(0.20, 0.12, 0.08, 1.0),
                )
            } else {
                // 아직 안 감긴 자리: 같은 모양을 어둡게.
                (
                    Color::new(0.36, 0.30, 0.32, 1.0),
                    Color::new(0.10, 0.07, 0.10, 0.0),
                )
            };
            for (dx, dy) in WAVE {
                ui.rect(Rect::new(x + dx - 1.0, y + dy - 1.0, 3.0, 3.0), edge);
            }
            for (dx, dy) in WAVE {
                ui.rect(Rect::new(x + dx, y + dy, 1.0, 2.0), fill);
            }
        }
    }
    // 오른쪽 위(0091): 굴파기 대기(위, 가늘게)와 SP 게이지(아래, 굵게). 24px·3~4px로는 눈에 안 띄었다 →
    // 34px, 바탕판, 칸마다 어두운 테두리, 가득 찬 SP는 금빛 테두리가 숨 쉰다.
    let cd = (w.player.burrow_cd / w.player.burrow_cd_full()).min(1.0);
    let bw = 34.0;
    // 필살기를 못 냈다(0149): 게이지 판이 좌우로 떨고 붉은 테두리.
    let x = t::VIEW_W - 4.0 - bw + deny_shake(w.juice.deny[0]);
    hud_plate(ui, Rect::new(x - 3.0, -1.0, bw + 6.0, 16.0));
    let frame = Color::new(0.02, 0.01, 0.03, 1.0);
    ui.rect(Rect::new(x - 1.0, 1.0, bw + 2.0, 5.0), frame);
    ui.rect(Rect::new(x, 2.0, bw, 3.0), dark);
    let ready = cd <= 0.0;
    ui.rect(
        Rect::new(x, 2.0, (bw * (1.0 - cd).max(0.0)).round(), 3.0),
        if ready {
            Color::new(0.78, 0.60, 0.40, 1.0)
        } else {
            world::DIRT
        },
    );
    // SP 게이지(그 아래). 가득이면 깜빡인다 — 필살기를 쓸 수 있다. 색은 캐릭터의 게이지 종류(두더지 = 분노).
    let sp = w.player.sp / t::SP_MAX;
    let full = w.player.sp_full();
    ui.rect(Rect::new(x - 1.0, 7.0, bw + 2.0, 7.0), frame);
    ui.rect(Rect::new(x, 8.0, bw, 5.0), dark);
    let gc = w.character.gauge().color();
    let gold = if full && ((w.time * 6.0) as i32) % 2 == 0 {
        Color::new(1.0, 1.0, 0.85, 1.0)
    } else {
        gc
    };
    ui.rect(Rect::new(x, 8.0, (bw * sp).round(), 5.0), gold);
    if w.juice.deny[0] > 0.0 {
        ui.rect_lines(
            Rect::new(x - 2.0, 6.0, bw + 4.0, 9.0),
            deny_color(w.juice.deny[0]),
        );
        deny_cross(ui, vec2(x + bw / 2.0, 10.0), 3.0);
    }
    if full {
        let a = 0.5 + 0.5 * (w.time * 5.0).sin();
        let c = crate::ui::ACCENT;
        ui.rect_lines(
            Rect::new(x - 2.0, 6.0, bw + 4.0, 9.0),
            Color::new(c.r, c.g, c.b, a),
        );
    }
}

/// 할 수 없는 행동(0149)의 붉은 테두리.
pub(super) const DENY_RED: Color = Color::new(1.0, 0.30, 0.32, 1.0);

/// 테두리는 흰색·붉은색을 번갈아 깜빡인다(0150): 1·2형 색약 시뮬레이션에서 붉은 테두리가 고른 칸의 금빛과 같아졌다.
/// 밝기가 바뀌는 깜빡임과 ✕ 모양(`deny_cross`)은 색을 몰라도 읽힌다.
pub(super) fn deny_color(left: f32) -> Color {
    if ((left * 16.0) as i32) % 2 == 0 {
        WHITE
    } else {
        DENY_RED
    }
}

/// 픽셀 ✕(반지름 `h`): 어두운 테두리 위에 흰 대각선 둘.
pub(super) fn deny_cross(ui: &Ui, c: Vec2, h: f32) {
    let c = c.round();
    let n = h as i32;
    for (col, grow) in [(OUTLINE, 1), (WHITE, 0)] {
        for i in -n..=n {
            let f = i as f32;
            for (dx, dy) in [(f, f), (f, -f)] {
                let g = grow as f32;
                ui.rect(
                    Rect::new(c.x + dx - g, c.y + dy - g, 1.0 + 2.0 * g, 1.0 + 2.0 * g),
                    col,
                );
            }
        }
    }
}

/// 떨림(px): 남은 시간 동안 1px씩 좌우로.
pub(super) fn deny_shake(left: f32) -> f32 {
    if left <= 0.0 {
        0.0
    } else if ((left * 40.0) as i32) % 2 == 0 {
        1.0
    } else {
        -1.0
    }
}

pub(super) fn draw_hud_screen(w: &World, scale: f32, ox: f32, oy: f32, gamepad_count: usize) {
    let fs = 16.0 * (scale / 4.0).max(1.0);
    let line = fs * 1.2;
    // 월드 HUD(체력 핍 8px 높이) 아래부터 텍스트.
    let mut y = oy + 9.0 * scale + fs;
    // 웨이브·처치 수는 아레나(웨이브 시험장)에서만. 실의 꿈(0055)도 아레나지만 보이지 않는다.
    if w.adv.place == crate::adventure::Place::Arena && !w.dream {
        draw_text(
            format!("wave {}  kills {}", w.wave, w.stats.kills),
            ox + 4.0,
            y,
            fs,
            GRAY,
        );
    }
    y += line;
    if !w.debug.overlay {
        return;
    }
    let dbg = Color::new(0.6, 0.6, 0.6, 0.9);
    let x = ox + 4.0;
    let lines = [
        format!(
            "fps {}  x{}  pads {}  F1 debug  F2 hitboxes",
            get_fps(),
            scale,
            gamepad_count
        ),
        format!(
            "player {:?}  hp {}  inv {:.2}  cd {:.2}  hitstop {:.2}  trauma {:.2}",
            short_state(&w.player.state),
            w.player.hp,
            w.player.invuln,
            w.player.burrow_cd,
            w.juice.feel.hitstop,
            w.juice.feel.trauma
        ),
        "move WASD/stick  attack J/Z/X-btn  burrow K/X/Space/A-btn  restart R/Start".to_string(),
    ];
    for l in lines {
        draw_text(&l, x, y, fs, dbg);
        y += line;
    }
    for e in &w.enemies {
        draw_text(
            format!("{:?} {} hp {}", e.kind, e.state_name(), e.hp),
            x,
            y,
            fs,
            Color::new(0.5, 0.5, 0.5, 0.8),
        );
        y += line;
    }
}

pub(super) fn short_state(s: &player::State) -> &'static str {
    match s {
        player::State::Free => "free",
        player::State::Attack { .. } => "attack",
        player::State::Charging { .. } => "charging",
        player::State::Heavy { .. } => "heavy",
        player::State::Burrow { .. } => "burrow",
        player::State::Emerge { .. } => "emerge",
        player::State::Drill { .. } => "drill",
        player::State::Recover { .. } => "recover",
        player::State::Eat { .. } => "eat",
        player::State::Hop { .. } => "hop",
        player::State::Leap { .. } => "leap",
        player::State::Chain { .. } => "chain",
        player::State::Run { .. } => "run",
        player::State::Jump { .. } => "jump",
        player::State::Revive { .. } => "revive",
        player::State::Dead => "dead",
    }
}

#[cfg(test)]
mod touch_icon_tests {
    use super::*;

    #[test]
    fn touch_icons_are_12_by_12() {
        for icon in [
            Icon::Claw,
            Icon::Paw,
            Icon::Mound,
            Icon::Hop,
            Icon::Drill,
            Icon::Chain,
            Icon::Acorn,
            Icon::Leap,
            Icon::Bury,
            Icon::Berry,
            Icon::Switch,
            Icon::Menu,
        ] {
            let m = icon_mask(icon);
            assert!(
                m.iter()
                    .all(|r| r.len() == 12 && r.bytes().all(|b| b == b'#' || b == b'.')),
                "{icon:?}"
            );
            assert!(m.iter().any(|r| r.contains('#')), "{icon:?} 빈 그림");
        }
    }
}
