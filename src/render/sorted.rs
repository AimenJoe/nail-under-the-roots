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
    /// 들어준 부탁의 흔적(0273 B, `quests::ALL` 번호, 자리).
    Trace(usize, Vec2),
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
                    if v.traces[crate::quests::DOTO] {
                        let t = *at + TRACE_OFFSET[crate::quests::DOTO];
                        list.push((t.y + 3.0, Drawable::Trace(crate::quests::DOTO, t)));
                    }
                    // 깨울 재주가 있으면 밑동이 빛난다(그림자처럼 먼저, 0061).
                    if v.tree_ready && ground {
                        draw_tree_glow(*at + vec2(0.0, 14.0), w.time);
                    }
                }
                // 물웅덩이는 바닥: 맨 먼저.
                'W' if ground && w.character == crate::equipment::Character::Mole => {
                    draw_puddle(*at, w.time)
                }
                'X' if v.halmae => list.push((at.y + 6.0, Drawable::DreamLoop(*at))),
                // 들어준 부탁의 흔적(0273 B): 그 이웃 자리 곁(이웃이 없어도 — 플레이어가 그 이웃이어도 흔적은 남는다).
                'K' | 'b' | 'M' | 'n' | 'g' | 'j' if v.traces[trace_of(*c)] => {
                    let i = trace_of(*c);
                    let at = *at + TRACE_OFFSET[i];
                    list.push((at.y + 3.0, Drawable::Trace(i, at)));
                }
                c if crate::village::slot_decor(*c).is_some() => {
                    let i = crate::village::slot_decor(*c).unwrap_or(0);
                    if v.decor.get(i).copied().unwrap_or(false) {
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
    for p in w.toss.patches.iter().filter(|_| ground) {
        draw_thorn_patch(p, w.time);
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
    // 물살(0275): 물 바닥은 지형에 구워 있고, 여기서는 움직이는 물결·반짝임과 물줄기 끝만.
    let baked = w.room.grid.is_some();
    for (c, d) in w.room.streams.iter().filter(|_| ground) {
        let has = |p: Vec2| {
            w.room
                .streams
                .iter()
                .any(|(q, _)| q.distance_squared(p) < 1.0)
        };
        // 굽은 곳(다음 칸이 옆으로 한 칸 비껴 이어짐)도 이어진 것으로 — 끝이 아니다.
        let perp = vec2(d.y.abs(), d.x.abs()) * 16.0;
        let end = |p: Vec2| {
            if has(p) || has(p + perp) || has(p - perp) {
                StreamEnd::Open
            } else if w.room.is_solid(p, 1.0) {
                StreamEnd::Wall
            } else {
                StreamEnd::Floor
            }
        };
        draw_stream(
            *c,
            *d,
            w.time,
            end(*c - *d * 16.0),
            end(*c + *d * 16.0),
            baked,
        );
    }
    // 고인 웅덩이(0275): 물결 없이 반짝임만.
    for c in w.room.pools.iter().filter(|_| ground) {
        draw_pool_glint(*c, w.time, baked);
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
        // 바라보는 쪽(0253): 싸우는 곳에서만 — 마을·쉼터는 고리만.
        let hub = matches!(
            w.adv.place,
            crate::adventure::Place::Village | crate::adventure::Place::Camp
        );
        if !hub {
            draw_facing_mark(pp, w.player.facing, w.time);
        }
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
        // 해 그림자 패스의 몸(플레이어·적·이웃)은 눕힌 그림을 위아래 1px씩 더 찍어 메운다(0208): 세로가 1/3로
        // 납작해지면서 목·꼬리처럼 가는 곳이 1px 줄이나 틈으로 남아 머리·몸·꼬리 그림자가 따로 놀았다(도토).
        // 위로 1px은 거의 몸에 가려지고, 보이는 두께는 아래로 1px만 는다.
        let body = matches!(
            d,
            Drawable::Player | Drawable::Enemy(_) | Drawable::Npc | Drawable::Neighbor(_)
        );
        let Some(l) = sun else {
            draw_one(art, w, alpha, d);
            continue;
        };
        let Some(k) = shadow_dir(vec2(drawable_x(w, alpha, &d), foot), l.fire, l.bright) else {
            continue;
        };
        let gl = unsafe { get_internal_gl() };
        for dy in if body {
            &[-1.0, 0.0, 1.0][..]
        } else {
            &[0.0][..]
        } {
            gl.quad_gl.push_model_matrix(
                Mat4::from_translation(vec3(0.0, *dy, 0.0)) * sun_shear(foot, k),
            );
            draw_one(art, w, alpha, d);
            gl.quad_gl.pop_model_matrix();
        }
    }
    SHADOW_PASS.with(|c| c.set(false));
}

/// 정렬 그리기 한 개(땅 위 그림, 해 그림자 패스에서는 눕힌 행렬 안에서).
fn draw_one(art: &Art, w: &World, alpha: f32, d: Drawable) {
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
                // 새 그림(0224 A)이 있으면 그루터기·바위는 그것(마을 들판의 나무는 위에서 팩 나무).
                PropKind::Stump | PropKind::Rock if art.hub.is_some() => {
                    if let Some(hub) = &art.hub {
                        let (frame, rx) = if p.kind == PropKind::Stump {
                            (art::HUB_STUMP, 9.0)
                        } else {
                            (art::HUB_BOULDER, 10.0)
                        };
                        let foot = vec2(r.center().x, r.bottom() + 2.0);
                        draw_ellipse(foot.x, foot.y - 1.0, rx, 2.5, 0.0, SHADOW_SOFT);
                        art::draw_hub(hub, frame, foot, w.biome.wall_tint());
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
                        crate::sunny::draw_at_foot(&sunny.sheet, spr, foot, w.biome.wall_tint());
                    }
                }
                PropKind::Stump | PropKind::Rock => {
                    let col = if p.kind == PropKind::Stump { 0.0 } else { 16.0 };
                    let src = Rect::new(col, 32.0, 16.0, 32.0);
                    draw_tile(&art.tiles, src, r.x, r.bottom() - 32.0, w.biome.wall_tint());
                }
                PropKind::Chest { opened } => draw_chest(art.props.as_ref(), r, opened, w.time),
                PropKind::Breakable(b) => draw_breakable(art.props.as_ref(), b, r, w.time),
                PropKind::Campfire => match &art.sunny {
                    Some(sunny) => draw_campfire_sunny(sunny, art.hub.as_ref(), r, w.time),
                    None => draw_campfire(r, w.time),
                },
                PropKind::Stall => match (&art.hub, &art.sunny) {
                    (Some(hub), _) => {
                        let foot = vec2(r.center().x.round(), r.bottom().round() + 1.0);
                        draw_ellipse(foot.x, foot.y - 1.0, 18.0, 3.0, 0.0, SHADOW_SOFT);
                        art::draw_hub(hub, art::HUB_STALL, foot, WHITE);
                    }
                    (None, Some(sunny)) => draw_stall_sunny(sunny, r),
                    (None, None) => draw_stall(r),
                },
                PropKind::Seat if art.hub.is_some() => {
                    if let Some(hub) = &art.hub {
                        draw_seat_hub(hub, r, w.story.momo_wins());
                    }
                }
                PropKind::Seat => match &art.sunny {
                    Some(sunny) => draw_seat_sunny(sunny, r, w.story.momo_wins()),
                    None => draw_seat(r, w.story.momo_wins()),
                },
            }
        }
        Drawable::Pickup(i) => draw_pickup(&w.pickups[i], w.time),
        Drawable::Loot(i) => draw_loot(&w.loot[i], alpha, w.time, false),
        Drawable::Player => draw_nail(art, &w.player, alpha, w.time),
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
                world::Who::Halmae => draw_halmae_at(
                    art,
                    n.pos,
                    w.time,
                    w.adv.village.is_some_and(|v| v.halmae_awake),
                ),
                world::Who::Momo => {
                    let on_seat = n.pos - vec2(0.0, 7.0);
                    draw_critter_at(art, art::MOMO_BLOOM_ROW, on_seat, left, false, w.time);
                }
                world::Who::Doto => {
                    draw_critter_at(art, art::DOTO_BACK_ROW, n.pos, left, false, w.time)
                }
                world::Who::Susu => draw_critter_at(art, art::SUSU_ROW, n.pos, left, false, w.time),
                world::Who::Kongi => {
                    draw_critter_at(art, art::KONGI_BACK_ROW, n.pos, left, false, w.time)
                }
                world::Who::Nail => draw_nail_at(art, n.pos, left, w.time),
                world::Who::Clover => {
                    // 토끼풀 + 아기 둘(0165): 어미 곁에서 아기들이 번갈아 깡충.
                    draw_critter_at(art, art::CLOVER_BACK_ROW, n.pos, left, false, w.time);
                    for (k, off) in [vec2(-13.0, 5.0), vec2(12.0, 7.0)].into_iter().enumerate() {
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
            // 새 그림(0238)이 있으면 그것, 없으면 예전 그리기(팩 잎 · 코드 줄기).
            match (art.tree.as_ref(), art.sunny.as_ref()) {
                (Some(tex), _) => {
                    let b = at + vec2(0.0, 8.0);
                    // 밑동이 땅에 붙어 보이게(사장 2026-10-29 "밑부분이 어울리지 않는다"): 뿌리 아래 넓은 그늘,
                    // 그리고 뿌리 끝을 덮는 풀 몇 포기(풀밭과 같은 초록).
                    if !shadow_pass() {
                        draw_ellipse(b.x, b.y + 11.0, 56.0, 9.0, 0.0, SHADOW_SOFT);
                        draw_ellipse(b.x, b.y + 9.0, 34.0, 6.0, 0.0, SHADOW_SOFT);
                    }
                    art::draw_tree(tex, growth, b);
                    if !shadow_pass() {
                        draw_root_grass(b);
                    }
                }
                (None, Some(sunny)) if growth > 0 => {
                    draw_great_tree_sunny(sunny, at, growth, w.time)
                }
                _ => draw_great_tree(at, growth, w.time),
            }
            if w.adv.village.is_some_and(|v| v.tree_ready) && !shadow_pass() {
                draw_tree_motes(at + vec2(0.0, 14.0), w.time);
            }
        }
        Drawable::Decor(i, at) => {
            match &art.sunny {
                Some(sunny) if decor_uses_pack(i) => draw_decor_sunny(sunny, i, at, w.time),
                _ => draw_decor(i, at, w.time),
            }
            // 돌 우물 위의 반딧불: 맡긴 빛의 단계만큼(0269 2.9 A). 그림자 패스에는 넣지 않는다.
            if i == crate::village::EXTRA_FIRST && !shadow_pass() {
                draw_well_fireflies(at, w.time, w.adv.village.map_or(0, |v| v.glow));
            }
        }
        Drawable::DreamLoop(at) => draw_dream_loop(at, w.time),
        Drawable::Trace(i, at) => draw_quest_trace(i, at, w.time),
    }
}

/// 부탁 흔적이 붙는 지도 기호 → 부탁 번호(큰 나무 'Y'의 깍정이는 따로).
fn trace_of(c: char) -> usize {
    match c {
        'K' => crate::quests::POPO,
        'b' => crate::quests::BORI,
        'n' => crate::quests::SUSU,
        'g' => crate::quests::KONGI,
        'j' => crate::quests::CLOVER,
        _ => crate::quests::MOMO,
    }
}

/// 흔적 자리: 기호에서 얼마나 떨어져 있나(포포 왼쪽 아래 — 좌판과 사이 · 보리 오른쪽 · 모모 자리 오른쪽 · 큰 나무 밑동 오른쪽).
/// 수수는 마을에 살지 않아 모닥불 곁(Naïl 자리 'n'에서 모닥불 왼쪽 아래), 콩이는 바위 곁, 토끼풀은 가족 자리 오른쪽.
const TRACE_OFFSET: [Vec2; 7] = [
    vec2(-11.0, 12.0),
    vec2(11.0, 5.0),
    vec2(18.0, 7.0),
    vec2(17.0, 24.0),
    vec2(-2.0, 20.0),
    vec2(14.0, 6.0),
    vec2(16.0, 8.0),
];

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
        Drawable::Tree(at, _)
        | Drawable::Decor(_, at)
        | Drawable::DreamLoop(at)
        | Drawable::Trace(_, at) => at.x,
    }
}

/// 큰 나무 뿌리 끝을 덮는 풀 포기: 뿌리가 풀밭 위에 얹힌 것이 아니라 땅에서 솟은 것처럼.
/// 같은 간격으로 늘어서면 심은 것 같다(사장, 2026-10-29): 뿌리 끝 몇 곳에 덩어리로 모으고, 크기·높이·간격을 다 다르게.
fn draw_root_grass(b: Vec2) {
    let dark = Color::from_rgba(82, 138, 72, 255);
    let mid = Color::from_rgba(96, 156, 80, 255);
    let light = Color::from_rgba(123, 207, 92, 255);
    // (dx, dy, 잎 높이들(왼쪽부터), 왼쪽으로 기울기). 덩어리 셋 + 외톨이 잎 몇.
    let tufts: [(f32, f32, &[f32], bool); 11] = [
        (-55.0, 9.0, &[2.0, 3.0], true),
        (-49.0, 12.0, &[3.0, 5.0, 4.0, 2.0], true),
        (-44.0, 14.0, &[2.0, 3.0], false),
        (-27.0, 16.0, &[2.0], false),
        (-15.0, 15.0, &[3.0, 4.0, 2.0], true),
        (-11.0, 17.0, &[2.0, 3.0], false),
        (6.0, 16.0, &[2.0], true),
        (21.0, 15.0, &[4.0, 6.0, 3.0, 4.0, 2.0], false),
        (27.0, 16.0, &[2.0, 3.0], false),
        (45.0, 13.0, &[3.0, 2.0], false),
        (54.0, 10.0, &[2.0, 4.0, 3.0], false),
    ];
    for (dx, dy, blades, lean_left) in tufts {
        let (x, y) = ((b.x + dx).round(), (b.y + dy).round());
        let n = blades.len() as f32;
        for (k, &h) in blades.iter().enumerate() {
            let bx = x + k as f32 - (n / 2.0).floor();
            // 바깥 잎은 짙게(그늘), 가운데 큰 잎은 밝게.
            let c = if h >= 4.0 {
                light
            } else if k == 0 || k + 1 == blades.len() {
                dark
            } else {
                mid
            };
            draw_rectangle(bx, y - h, 1.0, h, c);
            // 긴 잎 끝은 한 점 옆으로 휜다.
            if h >= 4.0 {
                let tip = if lean_left { -1.0 } else { 1.0 };
                draw_rectangle(bx + tip, y - h - 1.0, 1.0, 1.0, c);
            }
        }
    }
}

/// 들어준 부탁의 흔적(0273 B). 0 샘 그릇(물이 반짝), 1 갈대 피리(가끔 음표가 오른다), 2 꽃밭(분홍·흰 꽃 넷), 3 큰 깍정이,
/// 4 풀 방석, 5 버섯 등(빛), 6 솜털 잠자리. 마을 수첩 그림도 이것(`pub(super)`).
pub(super) fn draw_quest_trace(i: usize, at: Vec2, time: f32) {
    let (x, y) = (at.x.round(), at.y.round());
    let px = |dx: f32, dy: f32, w: f32, h: f32, c: Color| draw_rectangle(x + dx, y + dy, w, h, c);
    let dark = Color::new(0.22, 0.15, 0.13, 1.0);
    if !shadow_pass() {
        draw_ellipse(x, y + 3.0, 6.0, 2.0, 0.0, SHADOW_SOFT);
    }
    match i {
        0 => {
            // 돌 그릇에 담긴 샘물.
            px(-5.0, -2.0, 11.0, 5.0, dark);
            px(-4.0, -1.0, 9.0, 3.0, Color::new(0.60, 0.60, 0.64, 1.0));
            px(-3.0, -1.0, 7.0, 1.0, Color::new(0.45, 0.70, 0.90, 1.0));
            if (time * 1.3).sin() > 0.4 {
                px(-1.0, -1.0, 2.0, 1.0, Color::new(0.85, 0.95, 1.0, 1.0));
            }
        }
        1 => {
            // 통나무에 기대 둔 갈대 피리 + 몇 초마다 오르는 음표 하나.
            px(-1.0, -7.0, 3.0, 10.0, dark);
            px(0.0, -6.0, 1.0, 8.0, Color::new(0.88, 0.76, 0.48, 1.0));
            if shadow_pass() {
                return;
            }
            let t = (time * crate::tuning::PIPE_RATE).fract();
            if t < 0.45 {
                let k = t / 0.45;
                let (nx, ny) = (x + 3.0 + (k * 6.0).sin() * 2.0, y - 10.0 - k * 12.0);
                let a = 1.0 - k;
                let c = Color::new(0.96, 0.92, 0.80, a);
                draw_rectangle(nx.round(), ny.round(), 2.0, 2.0, c);
                draw_rectangle(nx.round() + 1.0, ny.round() - 4.0, 1.0, 4.0, c);
                draw_rectangle(nx.round() + 2.0, ny.round() - 4.0, 1.0, 1.0, c);
            }
        }
        2 => {
            // 갈아 놓은 흙 한 줄 위에 꽃 넷: 분홍·흰 번갈아, 줄기 초록(바닥 꽃 장식과 갈리게 심은 자리).
            px(-7.0, 1.0, 15.0, 3.0, Color::new(0.40, 0.27, 0.18, 1.0));
            px(-6.0, 1.0, 13.0, 1.0, Color::new(0.52, 0.36, 0.24, 1.0));
            let stem = Color::new(0.36, 0.62, 0.34, 1.0);
            let pink = Color::new(0.96, 0.58, 0.74, 1.0);
            let white = Color::new(0.97, 0.95, 0.90, 1.0);
            for (k, (dx, dy)) in [(-5.0, 0.0), (-1.0, -1.0), (3.0, 0.0), (6.0, -1.0)]
                .iter()
                .enumerate()
            {
                let sway = ((time * 1.2 + k as f32).sin() * 0.6).round();
                px(*dx, *dy, 1.0, 3.0, stem);
                let c = if k % 2 == 0 { pink } else { white };
                px(*dx - 1.0 + sway, *dy - 2.0, 3.0, 1.0, c);
                px(*dx + sway, *dy - 3.0, 1.0, 3.0, c);
                px(
                    *dx + sway,
                    *dy - 2.0,
                    1.0,
                    1.0,
                    Color::new(1.0, 0.88, 0.40, 1.0),
                );
            }
        }
        4 => {
            // 풀 방석: 둥글게 엮은 노란 풀 + 엮은 줄.
            px(-6.0, -2.0, 13.0, 5.0, dark);
            px(-5.0, -3.0, 11.0, 1.0, dark);
            let straw = Color::new(0.86, 0.74, 0.42, 1.0);
            px(-5.0, -2.0, 11.0, 4.0, straw);
            let weave = Color::new(0.66, 0.52, 0.28, 1.0);
            for dx in [-4.0, -1.0, 2.0, 5.0] {
                px(dx, -1.0, 1.0, 2.0, weave);
            }
            px(-4.0, -2.0, 9.0, 1.0, Color::new(0.95, 0.86, 0.58, 1.0));
        }
        5 => {
            // 버섯 등: 빛나는 갓을 얹은 작은 돌 받침 + 둥근 빛.
            if !shadow_pass() {
                let a = 0.12 + 0.08 * (time * 1.7).sin().abs();
                draw_circle(x + 0.5, y - 5.0, 10.0, Color::new(0.55, 0.85, 1.0, a));
            }
            px(-3.0, -1.0, 7.0, 4.0, dark);
            px(-2.0, 0.0, 5.0, 2.0, Color::new(0.55, 0.55, 0.60, 1.0));
            px(-4.0, -6.0, 9.0, 5.0, dark);
            px(-3.0, -5.0, 7.0, 3.0, Color::new(0.55, 0.82, 0.95, 1.0));
            px(-2.0, -5.0, 2.0, 1.0, Color::new(0.85, 0.97, 1.0, 1.0));
        }
        6 => {
            // 솜털 잠자리: 흰 솜털을 둥글게 깐 자리.
            px(-7.0, -2.0, 15.0, 5.0, dark);
            let fluff = Color::new(0.97, 0.96, 0.93, 1.0);
            let shade = Color::new(0.84, 0.82, 0.78, 1.0);
            px(-6.0, -2.0, 13.0, 4.0, shade);
            for dx in [-5.0, -2.0, 1.0, 4.0] {
                px(dx, -3.0, 3.0, 3.0, fluff);
            }
        }
        _ => {
            // 큰 깍정이: 뒤집힌 반달 + 비늘 무늬.
            px(-5.0, -2.0, 11.0, 2.0, dark);
            px(-4.0, 0.0, 9.0, 2.0, dark);
            px(-3.0, 2.0, 7.0, 1.0, dark);
            let cup = Color::new(0.60, 0.42, 0.25, 1.0);
            px(-4.0, -1.0, 9.0, 1.0, Color::new(0.72, 0.54, 0.34, 1.0));
            px(-3.0, 0.0, 7.0, 2.0, cup);
            let scale = Color::new(0.42, 0.28, 0.16, 1.0);
            for (dx, dy) in [
                (-3.0, 0.0),
                (-1.0, 0.0),
                (1.0, 0.0),
                (3.0, 0.0),
                (-2.0, 1.0),
                (0.0, 1.0),
                (2.0, 1.0),
            ] {
                px(dx, dy, 1.0, 1.0, scale);
            }
        }
    }
}
