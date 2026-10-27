//! 빛: 더하기 섞기 재질, 빛 목록, 월드 빛 겹.

use super::*;

/// 빛 더하기 재질(0076). 섞기만 다르고 셰이더는 macroquad 기본과 같다.
pub(super) fn light_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    varying lowp vec2 uv;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;
    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;
    const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;
    uniform sampler2D Texture;
    void main() {
        gl_FragColor = color * texture2D(Texture, uv);
    }"#;
    use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
    load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .ok()
}

/// 더하기 빛(0081): 반딧불(떨어진 것)·모닥불(바닥 빛 + 불티)·빛 버섯·심층 빛 알갱이. 어둠 위에 그린다.
pub(super) fn draw_world_lights(
    w: &World,
    cam: Vec2,
    alpha: f32,
    light: Option<&Material>,
    sheet: Option<&Texture2D>,
) {
    if w.room.grid.is_none() {
        return;
    }
    if let Some(m) = light {
        gl_use_material(m);
    }
    let k = if light.is_some() { 1.0 } else { 0.5 };
    let t = w.time;
    draw_light_shafts(w, cam, k);
    for l in &w.loot {
        if l.kind != loot::LootKind::Glowbug {
            continue;
        }
        let q = lerp(l.prev_pos, l.pos, alpha);
        let pulse = 0.55 + 0.45 * ((t * 6.0 + l.phase * 10.0) * 0.8).sin().abs();
        let g = world::GLOW;
        draw_circle(
            q.x.round(),
            (q.y - l.z - 3.0).round(),
            6.0,
            Color::new(g.r, g.g, g.b, 0.16 * pulse * k),
        );
    }
    // 지대 장애물(0149): 어둠이 덮은 뒤에도 보이게. 덤불은 붉은 기운, 떨어질 돌 자리는 붉은 고리가 조여 든다.
    for c in &w.room.thorns {
        let pulse = 0.5 + 0.5 * (t * 3.0 + c.x * 0.1).sin();
        draw_circle(
            c.x.round(),
            c.y.round(),
            8.0,
            Color::new(0.85, 0.15, 0.25, (0.10 + 0.08 * pulse) * k),
        );
    }
    // 흰 실 그물(0165)은 어둠 속에서 희게 비친다.
    for c in &w.room.webs {
        draw_circle(
            c.x.round(),
            c.y.round(),
            t::WEB_R,
            Color::new(0.92, 0.90, 1.0, 0.10 * k),
        );
    }
    for r in &w.hazards.rocks {
        let q = (r.time / t::ROCK_FALL).clamp(0.0, 1.0);
        let rad = t::ROCK_R * (1.5 - 0.5 * q);
        draw_circle_lines(
            r.pos.x.round(),
            r.pos.y.round(),
            rad,
            1.0,
            Color::new(1.0, 0.35, 0.30, (0.25 + 0.5 * q) * k),
        );
        draw_circle(
            r.pos.x.round(),
            r.pos.y.round(),
            t::ROCK_R * q,
            Color::new(0.9, 0.2, 0.15, 0.12 * q * k),
        );
    }
    // 포자 구름(0146)은 어둠 속에서도 옅게 빛난다(아플 때만 조금 더).
    for c in &w.hazards.clouds {
        let fade = 1.0 - ((c.time / t::SPORE_CLOUD_TIME - 0.75) / 0.25).clamp(0.0, 1.0);
        let hot = if c.time < t::SPORE_CLOUD_GRACE {
            0.5
        } else {
            1.0
        };
        draw_circle(
            c.pos.x.round(),
            c.pos.y.round(),
            t::SPORE_CLOUD_R,
            Color::new(0.55, 0.45, 0.80, 0.12 * fade * hot * k),
        );
    }
    for p in &w.room.props {
        if p.kind != PropKind::Campfire {
            continue;
        }
        let c = vec2(p.rect.center().x, p.rect.bottom() - 6.0);
        let flick = 0.5 + 0.25 * (t * 9.0).sin() + 0.25 * (t * 13.7).sin();
        // 바닥 빛: 겹을 여럿 두어 가장자리가 스티커처럼 끊기지 않게(바깥일수록 옅게).
        for (i, (rx, ry, a)) in [
            (40.0, 22.0, 0.03),
            (30.0, 16.0, 0.04),
            (20.0, 11.0, 0.05),
            (11.0, 6.0, 0.06),
        ]
        .into_iter()
        .enumerate()
        {
            let warm = 0.55 + i as f32 * 0.06;
            draw_ellipse(
                c.x,
                c.y + 2.0,
                rx + flick,
                ry + flick * 0.5,
                0.0,
                Color::new(1.0, warm, 0.25 + i as f32 * 0.04, (a + 0.02 * flick) * k),
            );
        }
        // 불티: 넷이 위로 떠오르다 사라진다.
        for i in 0..4u32 {
            let ph = (t * 0.9 + hash01(i, 7)).fract();
            let x = c.x + (t * 3.0 + i as f32 * 1.7).sin() * 3.0 + (hash01(i, 8) - 0.5) * 6.0;
            let y = c.y - 6.0 - ph * 22.0;
            draw_rectangle(
                x.round(),
                y.round(),
                1.0,
                1.0,
                Color::new(1.0, 0.75, 0.35, (1.0 - ph) * k),
            );
        }
    }
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.6 + 0.4 * ((t * 2.0 + x * 0.1).sin());
            // 동심원 세 겹으로 가장자리를 풀어 은은하게(0100).
            for r in [16.0, 11.0, 6.0] {
                draw_circle(
                    x + 8.0,
                    y + 10.0,
                    r,
                    Color::new(0.35, 0.65, 1.0, 0.035 * pulse * k),
                );
            }
        }
    }
    // 잠긴 문의 실과 빛(0094): 어둠 마스크 위에 더해 그려 어두운 심층에서도 막힌 문이 보인다.
    if w.room.door_anim > 0.0 {
        if let Some(g) = &w.room.grid {
            let (x0, y0) = (
                (cam.x / 16.0).floor() as i32 - 1,
                (cam.y / 16.0).floor() as i32 - 1,
            );
            for cy in y0..y0 + 14 {
                for cx in x0..x0 + 23 {
                    if g.get_i(cx, cy) == Cell::Door {
                        draw_root_gate(g, cx, cy, w.room.door_anim, t, true, sheet);
                    }
                }
            }
        }
    }
    draw_ambient(w, cam, true);
    if light.is_some() {
        gl_use_default_material();
    }
}

/// 빛 하나(0095): 자리·반지름·세기(0..1, 어둠을 얼마나 빼나).
pub struct LightSrc {
    pub pos: Vec2,
    pub radius: f32,
    pub strength: f32,
}

/// 지금 화면 둘레의 빛. 어둠 지도와 그림자(빛 반대쪽으로 늘어남)가 같이 쓴다.
pub fn light_sources(w: &World, alpha: f32, cam: Vec2, player_radius: f32) -> Vec<LightSrc> {
    let t = w.time;
    let mut v = Vec::new();
    // 가장 깊은 곳(0162): 할매의 흰 빛과 매듭의 보라 빛.
    if let Some(d) = &w.deep {
        let rel = d.done_t.map(|x| (x / 1.6).clamp(0.0, 1.0)).unwrap_or(0.0);
        v.push(LightSrc {
            pos: d.center,
            radius: 96.0 + 90.0 * rel,
            strength: 0.9 + 0.6 * rel,
        });
        for k in d.knots.iter().filter(|k| !k.cut) {
            v.push(LightSrc {
                pos: k.pos,
                radius: 36.0,
                strength: 0.55,
            });
        }
    }
    // 아기들의 고치(0165): 흰 빛이 조금.
    for c in &w.hazards.cocoons {
        v.push(LightSrc {
            pos: *c,
            radius: 30.0,
            strength: 0.5,
        });
    }
    // 길의 샘(0159): 어두운 굴에서도 샘이 먼저 보인다.
    if let Some(sp) = &w.spring {
        v.push(LightSrc {
            pos: sp.pos,
            radius: 46.0,
            strength: if sp.drunk { 0.4 } else { 0.65 },
        });
    }
    // 플레이어(예전 어둠 막의 구멍과 같은 크기, 가운데는 거의 다 걷힌다).
    v.push(LightSrc {
        pos: lerp(w.player.prev_pos, w.player.pos, alpha),
        radius: player_radius,
        strength: 1.0,
    });
    for (x, y, d) in floor_decor(w, cam) {
        if d == Decor::Mushroom {
            let pulse = 0.85 + 0.15 * ((t * 2.0 + x * 0.1).sin());
            v.push(LightSrc {
                pos: vec2(x + 8.0, y + 10.0),
                radius: t::MUSH_LIGHT_R,
                strength: t::MUSH_LIGHT * pulse,
            });
        }
    }
    for p in &w.room.props {
        if p.kind == PropKind::Campfire {
            let flick = 0.9 + 0.1 * (t * 9.0).sin();
            v.push(LightSrc {
                pos: vec2(p.rect.center().x, p.rect.bottom() - 6.0),
                radius: 80.0,
                strength: 0.95 * flick,
            });
        }
    }
    for l in &w.loot {
        if l.kind == loot::LootKind::Glowbug {
            v.push(LightSrc {
                pos: lerp(l.prev_pos, l.pos, alpha),
                radius: 18.0,
                strength: 0.55,
            });
        }
    }
    if let Some(g) = &w.room.grid {
        let (x0, y0) = (
            (cam.x / 16.0).floor() as i32 - 2,
            (cam.y / 16.0).floor() as i32 - 2,
        );
        for cy in y0..y0 + 16 {
            for cx in x0..x0 + 25 {
                let c = vec2(cx as f32 * 16.0 + 8.0, cy as f32 * 16.0 + 8.0);
                match g.get_i(cx, cy) {
                    Cell::Save => v.push(LightSrc {
                        pos: c,
                        radius: 44.0,
                        strength: 0.7,
                    }),
                    Cell::Exit => v.push(LightSrc {
                        pos: c,
                        radius: 34.0,
                        strength: 0.65,
                    }),
                    Cell::Door if w.room.door_anim > 0.0 => v.push(LightSrc {
                        pos: c,
                        radius: 22.0,
                        strength: 0.4 * w.room.door_anim,
                    }),
                    _ => {}
                }
            }
        }
    }
    v
}
