//! y 정렬 그리기: 월드 안의 배우·물건을 발 줄 순서로 한 번에.

use super::*;

#[derive(Clone, Copy)]
pub(super) enum Drawable {
    Pickup(usize),
    Loot(usize),
    Pillar(usize),
    Player,
    Enemy(usize),
    Projectile(usize),
    /// 도토가 던진 도토리(0195).
    Shot(usize),
    Npc,
    Neighbor(usize),
    /// 마을(0045): 큰 나무(자리, 잎 단계), 꾸미기(번호, 자리).
    Tree(Vec2, u8),
    Decor(usize, Vec2),
    /// 실 고리(0055).
    DreamLoop(Vec2),
}

/// 3/4 시점: 발 위치(y)가 아래인 것이 앞. 그림자는 모두 먼저.
/// `sun`이 있으면 해 그림자 패스(0102): 바닥 것(발밑 그림자·웅덩이·지뢰)은 빼고, 서 있는 것마다 발 줄을 축으로
/// 바닥에 눕혀(`sun_shear`) 그대로 다시 그린다. 색은 버리고 모양만 쓴다(`Renderer::draw_sun_shadows`).
pub(super) fn draw_sorted(art: &Art, w: &World, alpha: f32, sun: Option<ShadowLight>) {
    let ground = sun.is_none();
    SHADOW_PASS.with(|c| c.set(!ground));
    let mut list: Vec<(f32, Drawable)> = Vec::new();
    for (i, p) in w.room.props.iter().enumerate() {
        list.push((p.rect.bottom(), Drawable::Pillar(i)));
    }
    for (i, p) in w.pickups.iter().enumerate() {
        list.push((p.pos.y + 4.0, Drawable::Pickup(i)));
        if ground {
            draw_ellipse(p.pos.x, p.pos.y + 4.0, 5.0, 1.8, 0.0, SHADOW_SOFT);
        }
    }
    for (i, l) in w.loot.iter().enumerate() {
        if !ground {
            continue;
        }
        let q = lerp(l.prev_pos, l.pos, alpha);
        list.push((q.y + 2.0, Drawable::Loot(i)));
        let r = if l.kind == loot::LootKind::Berry {
            3.0
        } else {
            2.0
        };
        draw_ellipse(q.x, q.y + 2.0, r, 1.0, 0.0, SHADOW_SOFT);
    }
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    list.push((pp.y + 6.0, Drawable::Player));
    if let Some(n) = &w.npc {
        let q = lerp(n.prev_pos, n.pos, alpha);
        list.push((q.y + 6.0, Drawable::Npc));
        if ground {
            draw_ellipse(q.x, q.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
        }
    }
    for (i, n) in w.neighbors.iter().enumerate() {
        let big = n.who == world::Who::Bori;
        if n.who == world::Who::Momo {
            // 모모는 빈 자리 그루터기 위에 앉는다: 그루터기(칸 아래 끝) 뒤에 그린다, 그림자 없이.
            list.push((n.pos.y + 9.0, Drawable::Neighbor(i)));
            continue;
        }
        // 보리(32px 그림)는 발이 그림 맨 아래 줄(pos.y + 7)이고 몸이 발보다 넓다: 발밑 그림자를 발 줄에 크게 두어야
        // 몸 밖으로 보인다(y + 6, 가로 12였을 때는 몸에 거의 다 가려 그림자가 없는 것처럼 보였다).
        let (foot, shadow_y, rx, ry): (f32, f32, f32, f32) = if big {
            (7.0, 7.5, 14.0, 3.0)
        } else {
            (6.0, 6.0, 6.0, 2.5)
        };
        list.push((n.pos.y + foot, Drawable::Neighbor(i)));
        if ground {
            draw_ellipse(n.pos.x, n.pos.y + shadow_y, rx, ry, 0.0, SHADOW_SOFT);
        }
    }
    if let Some(v) = &w.adv.village {
        for (c, at) in &w.adv.marks {
            match c {
                'Y' => {
                    list.push((at.y + 8.0, Drawable::Tree(*at, v.growth)));
                    // 깨울 재주가 있으면 밑동이 빛난다(그림자처럼 먼저, 0061).
                    if v.tree_ready && ground {
                        draw_tree_glow(*at + vec2(0.0, 14.0), w.time);
                    }
                }
                // 물웅덩이는 바닥: 맨 먼저.
                'W' if ground => draw_puddle(*at, w.time),
                'X' if v.halmae => list.push((at.y + 6.0, Drawable::DreamLoop(*at))),
                '1'..='9' => {
                    let i = (*c as u8 - b'1') as usize;
                    if v.decor[i] {
                        list.push((at.y + 6.0, Drawable::Decor(i, *at)));
                    }
                }
                _ => {}
            }
        }
    }
    for (i, e) in w.enemies.iter().enumerate() {
        let ep = lerp(e.prev_pos, e.pos, alpha);
        if !e.hittable() {
            // 땅속: 그림자 없이 둔덕만.
            if ground {
                list.push((ep.y + e.radius(), Drawable::Enemy(i)));
            }
            continue;
        }
        list.push((ep.y + e.radius(), Drawable::Enemy(i)));
        if !ground {
            continue;
        }
        draw_ellipse(
            ep.x,
            ep.y + e.radius(),
            e.radius() + 1.0,
            2.5,
            0.0,
            SHADOW_SOFT,
        );
    }
    // 묻힌 도토리는 바닥에(세워 그리는 것들보다 먼저).
    for m in w.hazards.mines.iter().filter(|_| ground) {
        draw_mine(m, w.time);
    }
    for b in w.toss.buried.iter().filter(|_| ground) {
        draw_buried(b, w.time);
    }
    for c in w.hazards.clouds.iter().filter(|_| ground) {
        draw_cloud(c, w.time);
    }
    let pp = lerp(w.player.prev_pos, w.player.pos, alpha);
    for c in w.room.thorns.iter().filter(|_| ground) {
        let near = 1.0 - ((*c - pp).length() - 14.0).clamp(0.0, 30.0) / 30.0;
        draw_thorn(*c, w.time, near);
    }
    for c in w.room.webs.iter().filter(|_| ground) {
        let near = 1.0 - ((*c - pp).length() - 14.0).clamp(0.0, 30.0) / 30.0;
        draw_web(*c, w.time, near);
    }
    for (i, p) in w.projectiles.iter().enumerate().filter(|_| ground) {
        let q = lerp(p.prev_pos, p.pos, alpha);
        list.push((q.y + 3.0, Drawable::Projectile(i)));
        draw_ellipse(q.x, q.y + 6.0, 2.5, 1.2, 0.0, SHADOW_SOFT);
    }
    for (i, s) in w.toss.shots.iter().enumerate().filter(|_| ground) {
        let q = lerp(s.prev, s.pos, alpha);
        list.push((q.y + 3.0, Drawable::Shot(i)));
        draw_ellipse(q.x, q.y + 6.0, s.radius * 0.8, 1.2, 0.0, SHADOW_SOFT);
    }
    let standing = !matches!(
        w.player.state,
        player::State::Burrow { .. } | player::State::Dead
    );
    if standing && ground {
        draw_ellipse(pp.x, pp.y + 6.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
        draw_player_halo(pp, w.time);
    }
    if !ground {
        // 해 그림자: 땅속 플레이어(둔덕)·줍는 것은 그림자를 드리우지 않는다. 모닥불은 돌만(아래).
        list.retain(|(_, d)| match d {
            Drawable::Player => standing,
            Drawable::Pickup(_) => false,
            _ => true,
        });
    }
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (foot, d) in list {
        // 해·모닥불 그림자: 발 줄(foot)을 축으로 눕힌다. 끝에서 되돌린다. 빛이 닿지 않으면 그리지 않는다.
        if let (Some(_), Drawable::Pillar(i)) = (sun, &d) {
            let p = w.room.props[*i];
            if p.kind == PropKind::Campfire {
                // 모닥불 돌(0105): 불꽃은 빛이라 빼고, 돌만 키워 그려 돌마다 바깥쪽 그림자가 되게 한다.
                // 가로는 불 한가운데를 축으로 양옆으로, 세로는 돌무더기 뒤 끝을 축으로: 3/4 시점에서 뒤(위)로 눕는
                // 그림자는 짧게 줄어 보이므로 뒤쪽 돌은 거의 늘지 않고 앞쪽 돌일수록 앞으로 길어진다.
                let cx = p.rect.center().x;
                let back = p.rect.bottom() + 2.0 - crate::sunny::FIRE_PIT.3 as f32 + 2.0;
                let k = 1.0 + t::FIRE_PIT_SPREAD;
                let m = Mat4::from_cols(
                    vec4(k, 0.0, 0.0, 0.0),
                    vec4(0.0, k, 0.0, 0.0),
                    vec4(0.0, 0.0, 1.0, 0.0),
                    vec4(cx * (1.0 - k), back * (1.0 - k), 0.0, 1.0),
                );
                let gl = unsafe { get_internal_gl() };
                gl.quad_gl.push_model_matrix(m);
                draw_campfire_stones(art.sunny.as_ref(), p.rect);
                gl.quad_gl.pop_model_matrix();
                continue;
            }
        }
        if let Some(l) = sun {
            let at = vec2(drawable_x(w, alpha, &d), foot);
            let Some(k) = shadow_dir(at, l.fire, l.bright) else {
                continue;
            };
            unsafe { get_internal_gl() }
                .quad_gl
                .push_model_matrix(sun_shear(foot, k));
        }
        match d {
            Drawable::Pillar(i) => {
                let p = w.room.props[i];
                let r = p.rect;
                match p.kind {
                    // 들판의 나무(T): 팩이 있으면 흔들리는 둥근 나무. 충돌은 밑동 한 칸 그대로.
                    PropKind::Stump
                        if w.biome == crate::biome::Biome::Meadow && art.sunny.is_some() =>
                    {
                        if let Some(sunny) = &art.sunny {
                            let (tw, th) = art::SUNNY_TREE;
                            let f = ((w.time * 3.0 + r.x * 0.07) as u32 % 4) as f32;
                            draw_texture_ex(
                                &sunny.tree,
                                (r.center().x - tw / 2.0).round(),
                                (r.bottom() - th + 3.0).round(),
                                WHITE,
                                DrawTextureParams {
                                    source: Some(Rect::new(f * tw, 0.0, tw, th)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                    // 팩이 있으면 그루터기·바위도 Sunnyside(0066). 바위는 자리 해시로 셋 중 하나.
                    PropKind::Stump | PropKind::Rock if art.sunny.is_some() => {
                        if let Some(sunny) = &art.sunny {
                            let pick = (r.x as usize / 16 * 7 + r.y as usize / 16 * 3) % 3;
                            let spr = if p.kind == PropKind::Stump {
                                crate::sunny::STUMPS[pick % 2]
                            } else {
                                crate::sunny::BOULDERS[pick]
                            };
                            let foot = vec2(r.center().x, r.bottom() + 2.0);
                            draw_ellipse(
                                foot.x,
                                foot.y - 1.0,
                                spr.2 as f32 * 0.42,
                                2.5,
                                0.0,
                                SHADOW_SOFT,
                            );
                            crate::sunny::draw_at_foot(
                                &sunny.sheet,
                                spr,
                                foot,
                                w.biome.wall_tint(),
                            );
                        }
                    }
                    PropKind::Stump | PropKind::Rock => {
                        let col = if p.kind == PropKind::Stump { 0.0 } else { 16.0 };
                        let src = Rect::new(col, 32.0, 16.0, 32.0);
                        draw_tile(&art.tiles, src, r.x, r.bottom() - 32.0, w.biome.wall_tint());
                    }
                    PropKind::Chest { opened } => draw_chest(r, opened, w.time),
                    PropKind::Breakable(b) => draw_breakable(b, r, w.time),
                    PropKind::Campfire => match &art.sunny {
                        Some(sunny) => draw_campfire_sunny(sunny, r, w.time),
                        None => draw_campfire(r, w.time),
                    },
                    PropKind::Stall => match &art.sunny {
                        Some(sunny) => draw_stall_sunny(sunny, r),
                        None => draw_stall(r),
                    },
                    PropKind::Seat => match &art.sunny {
                        Some(sunny) => draw_seat_sunny(sunny, r, w.story.momo_wins()),
                        None => draw_seat(r, w.story.momo_wins()),
                    },
                }
            }
            Drawable::Pickup(i) => draw_pickup(&w.pickups[i], w.time),
            Drawable::Loot(i) => draw_loot(&w.loot[i], alpha, w.time, false),
            Drawable::Player => draw_nail(art, &w.player, w.inv.fur, alpha, w.time),
            Drawable::Enemy(i) => draw_enemy(
                Some(art),
                &w.enemies[i],
                alpha,
                w.time,
                w.juice.feel.hitstop,
            ),
            Drawable::Projectile(i) => draw_projectile(&w.projectiles[i], alpha),
            Drawable::Shot(i) => draw_shot(&w.toss.shots[i], alpha),
            Drawable::Npc => {
                if let Some(n) = &w.npc {
                    let q = lerp(n.prev_pos, n.pos, alpha);
                    let moving = (n.pos - n.prev_pos).length() > 0.05;
                    draw_critter_at(art, art::RACCOON_ROW, q, n.face_left, moving, w.time);
                }
            }
            Drawable::Neighbor(i) => {
                let n = &w.neighbors[i];
                let left = (w.player.pos.x < n.pos.x) != n.face_left;
                match n.who {
                    world::Who::Popo => {
                        draw_critter_at(art, art::RACCOON_ROW, n.pos, left, false, w.time)
                    }
                    world::Who::Bori => draw_bori_at(art, n.pos, left, false, w.time),
                    world::Who::Halmae => draw_halmae_at(n.pos, w.time),
                    world::Who::Momo => {
                        let on_seat = n.pos - vec2(0.0, 7.0);
                        draw_critter_at(art, art::MOMO_BLOOM_ROW, on_seat, left, false, w.time);
                    }
                    world::Who::Doto => {
                        draw_critter_at(art, art::DOTO_BACK_ROW, n.pos, left, false, w.time)
                    }
                    world::Who::Susu => {
                        draw_critter_at(art, art::SUSU_ROW, n.pos, left, false, w.time)
                    }
                    world::Who::Kongi => {
                        draw_critter_at(art, art::KONGI_BACK_ROW, n.pos, left, false, w.time)
                    }
                    world::Who::Clover => {
                        // 토끼풀 + 아기 둘(0165): 어미 곁에서 아기들이 번갈아 깡충.
                        draw_critter_at(art, art::CLOVER_BACK_ROW, n.pos, left, false, w.time);
                        for (k, off) in [vec2(-13.0, 5.0), vec2(12.0, 7.0)].into_iter().enumerate()
                        {
                            let hop = ((w.time * 2.0 + k as f32 * 1.7).sin() * 2.0)
                                .max(0.0)
                                .round();
                            draw_critter_at(
                                art,
                                art::BABY_ROW,
                                n.pos + off - vec2(0.0, hop),
                                k == 0,
                                false,
                                w.time,
                            );
                        }
                    }
                }
            }
            Drawable::Tree(at, growth) => {
                match art.sunny.as_ref() {
                    Some(sunny) if growth > 0 => draw_great_tree_sunny(sunny, at, growth, w.time),
                    _ => draw_great_tree(at, growth, w.time),
                }
                if w.adv.village.is_some_and(|v| v.tree_ready) && !shadow_pass() {
                    draw_tree_motes(at + vec2(0.0, 14.0), w.time);
                }
            }
            Drawable::Decor(i, at) => match &art.sunny {
                Some(sunny) if matches!(i, 0 | 2 | 3 | 7) => draw_decor_sunny(sunny, i, at, w.time),
                _ => draw_decor(i, at, w.time),
            },
            Drawable::DreamLoop(at) => draw_dream_loop(at, w.time),
        }
        if sun.is_some() {
            unsafe { get_internal_gl() }.quad_gl.pop_model_matrix();
        }
    }
    SHADOW_PASS.with(|c| c.set(false));
}

/// 정렬 그리기 한 개의 발 x(그림자 패스가 모닥불까지 거리를 잴 때).
pub(super) fn drawable_x(w: &World, alpha: f32, d: &Drawable) -> f32 {
    match *d {
        Drawable::Pillar(i) => w.room.props[i].rect.center().x,
        Drawable::Pickup(i) => w.pickups[i].pos.x,
        Drawable::Loot(i) => lerp(w.loot[i].prev_pos, w.loot[i].pos, alpha).x,
        Drawable::Player => lerp(w.player.prev_pos, w.player.pos, alpha).x,
        Drawable::Enemy(i) => lerp(w.enemies[i].prev_pos, w.enemies[i].pos, alpha).x,
        Drawable::Projectile(i) => lerp(w.projectiles[i].prev_pos, w.projectiles[i].pos, alpha).x,
        Drawable::Shot(i) => lerp(w.toss.shots[i].prev, w.toss.shots[i].pos, alpha).x,
        Drawable::Npc => w
            .npc
            .as_ref()
            .map_or(0.0, |n| lerp(n.prev_pos, n.pos, alpha).x),
        Drawable::Neighbor(i) => w.neighbors[i].pos.x,
        Drawable::Tree(at, _) | Drawable::Decor(_, at) | Drawable::DreamLoop(at) => at.x,
    }
}
