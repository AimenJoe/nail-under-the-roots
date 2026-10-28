//! JRPG식 컷씬: 배우가 걷고, 아래 대화창에 글자가 타자처럼 찍히고, 확인으로 넘긴다.
//!
//! 컷씬은 월드 위에서 재생된다(월드 로직은 멈춤). 배우 0번은 플레이어(월드의 위치를 직접 움직인다),
//! 나머지는 컷씬이 들고 있는 임시 배우다. 뒤로(Esc/B)로 건너뛰면 남은 이동·등장·페이드를 즉시 적용한다.
//! 스크립트는 `Step` 목록이다. 데이터 파일로 옮기는 것은 내용이 쌓인 뒤에.

use macroquad::prelude::*;

use crate::i18n::{self, Tr};
use crate::illust::Illust;
use crate::input::{Act, Actions};
use crate::world::{Cue, World};

/// 글자가 찍히는 속도(초당 글자 수).
const CHARS_PER_SEC: f32 = 40.0;
/// 컷씬이 시작되고 이 시간 동안은 확인을 버린다. 싸우다 바로 이어지는 장면(보스 처치)에서
/// 공격 연타가 대사를 다 넘겨 버리지 않게(헤드리스 확인에서 실제로 그랬다).
const START_GUARD: f32 = 1.0;
/// 대사 한 줄이 뜨고 이 시간 안의 확인도 버린다(연타로 줄을 건너뛰지 않게).
const LINE_GUARD: f32 = 0.3;

// 초상화 칸 번호는 그림 표(`art.rs`)에(0170 R14: 캐릭터 표가 쓴다).
pub use crate::art::{Face, Portrait};

#[cfg(test)]
mod device_tests {
    use super::*;

    /// 모모 등장 장치(0159): 음악을 내리고 어둠을 덮고 카메라를 모모에게 옮긴 뒤, 끝나면(건너뛰어도) 모두 제자리.
    #[test]
    fn momo_intro_restores_music_light_and_camera_when_skipped() {
        let mut w = World::new();
        let at = w.player.pos + vec2(120.0, 0.0);
        let mut c = momo_meet(&w, 1, at);
        let quiet = Actions::default();
        // 어둠 → 팬 → 꽃잎·일어섬 → 카드 → 말(확인을 기다린다): 5초면 말 차례.
        for _ in 0..300 {
            c.update(&quiet, 1.0 / 60.0, &mut w);
        }
        assert!(w.juice.hush, "말하기 전엔 음악이 내려가 있다");
        assert!(
            w.juice.gloom.amount > 0.5,
            "어둠이 덮인다 {}",
            w.juice.gloom.amount
        );
        assert!(
            w.juice.audio.played(crate::sfx::Id::Rumble)
                || w.juice
                    .fx
                    .iter()
                    .any(|f| matches!(f.kind, crate::world::FxKind::Petal { .. })),
            "꽃잎·울림"
        );
        let skip = Actions {
            cancel: true,
            ..Default::default()
        };
        c.update(&skip, 1.0 / 60.0, &mut w);
        assert!(c.is_done());
        assert!(!w.juice.hush, "끝나면 음악(보스 곡)");
        assert_eq!(w.juice.gloom.amount, 0.0, "어둠은 걷힌다");
        assert!(w.adv.cam_hold.is_none(), "카메라는 플레이어에게");
    }

    /// 물든 것의 등장 셋(0163): 대장·도토·콩이도 모모처럼 음악이 내려가고 어둠·카드가 뜨며, 끝나면 제자리.
    #[test]
    fn tainted_intros_are_staged_like_momo() {
        let w = World::new();
        for (name, c) in [
            ("boss", boss_intro(&w)),
            ("doto", doto_meet(&w)),
            ("kongi", kongi_meet(&w)),
        ] {
            let steps = &c.steps;
            assert!(
                matches!(steps.first(), Some(Step::Hush(true))),
                "{name}: 음악부터"
            );
            assert!(
                steps
                    .iter()
                    .any(|s| matches!(s, Step::Card { name: "???", .. })),
                "{name}: ??? 카드"
            );
            assert!(
                steps
                    .iter()
                    .any(|s| matches!(s, Step::Cue { cue: Cue::Rise, .. })),
                "{name}: 일어선다"
            );
            assert!(
                matches!(steps.last(), Some(Step::Hush(false))),
                "{name}: 끝나면 음악"
            );
            assert!(
                steps
                    .iter()
                    .filter(|s| matches!(s, Step::Say { .. }))
                    .count()
                    == 2,
                "{name}: 말 두 줄은 그대로"
            );
        }
        // 잠드는 장면: 어둠이 1.0까지 닫히고 검게 끝난다. 되살아날 때 걷는 것은 adventure가 한다.
        let mut w = World::new();
        let mut c = deep_sleep(&w);
        let quiet = Actions::default();
        for _ in 0..200 {
            c.update(&quiet, 1.0 / 60.0, &mut w);
        }
        assert!(w.juice.hush);
        assert!(w.juice.gloom.amount > 0.95, "{}", w.juice.gloom.amount);
        assert!(
            w.juice.audio.played(crate::sfx::Id::Rumble),
            "실이 휘감는 울림"
        );
        // 도망(0163): 사라지는 자리에 흙·실.
        let at = w.player.pos + vec2(60.0, 0.0);
        let c = momo_flee(&w, at, 1);
        assert!(c.steps.iter().any(|s| matches!(
            s,
            Step::Cue {
                cue: Cue::Vanish,
                ..
            }
        )));
    }

    /// 오프닝의 모모(0164): 눈빛 → 등장 → 카드 → 덮침(Naïl이 밀려남) → 멈춤·착각 → 사라지며 시든 꽃잎을 남긴다
    /// → 포포가 굳는다. 끝까지 돌리면 꽃잎이 바닥에 있고, 밟으면 줍는다.
    #[test]
    fn opening_momo_lunges_and_leaves_a_petal() {
        let mut w = World::new();
        let c = opening(&w);
        let steps = &c.steps;
        let pos = |f: &dyn Fn(&Step) -> bool| steps.iter().position(f);
        let eyes = pos(&|s| matches!(s, Step::Cue { cue: Cue::Eyes, .. })).expect("눈빛");
        let card = pos(&|s| matches!(s, Step::Card { name: "???", .. })).expect("카드");
        let lunge = pos(&|s| matches!(s, Step::Move { actor: 1, speed, .. } if *speed > 200.0))
            .expect("덮침");
        let recoil = pos(&|s| matches!(s, Step::Move { actor: 0, speed, .. } if *speed > 150.0))
            .expect("밀려남");
        let leave = pos(&|s| {
            matches!(
                s,
                Step::Cue {
                    cue: Cue::Leave,
                    ..
                }
            )
        })
        .expect("꽃잎");
        let popo = pos(&|s| matches!(s, Step::Say { actor: 2, .. })).expect("포포");
        assert!(eyes < card && card < lunge && lunge < recoil && recoil < leave && leave < popo);
        // 포포의 첫마디는 굳은 말(Worried)이고, 그다음이 인사.
        assert!(matches!(
            steps[popo],
            Step::Say {
                face: Face::Worried,
                ..
            }
        ));
        // 건너뛰기는 모모 앞에서 멈춘다(덧붙임): 한 번 누르면 일러스트·내레이션만 넘기고 모모는 아직 안 나왔다.
        let mut c = c;
        let skip = Actions {
            cancel: true,
            ..Default::default()
        };
        c.update(&skip, 1.0 / 60.0, &mut w);
        assert!(!c.is_done(), "한 번 건너뛰면 모모 등장은 남는다");
        assert!(c.illust.is_none() && !c.actors[1].visible && !w.juice.hush);
        // 그대로 두면 모모가 나온다(어둠·음악 끊김).
        let go = Actions::default();
        for _ in 0..240 {
            c.update(&go, 1.0 / 60.0, &mut w);
            if c.actors[1].visible {
                break;
            }
        }
        assert!(
            c.actors[1].visible && w.juice.hush,
            "멈춘 자리부터 재생돼 모모가 걸어 나온다"
        );
        // 한 번 더 건너뛰면 끝까지, 자리에 꽃잎이 남는다.
        for _ in 0..400 {
            c.update(&skip, 1.0 / 60.0, &mut w);
            if c.is_done() {
                break;
            }
        }
        assert!(c.is_done());
        // 줍는 꽃잎은 보리 공터 앞 길의 하나뿐(0194): 모모가 선 자리(첫 발톱 옆)에는 흩날리기만 한다.
        assert_eq!(w.story.petals(), &[crate::field::petal_road()][..]);
        assert!(!w.petal_tick(), "멀리서는 안 줍는다");
        w.player.pos = crate::field::petal_road();
        assert!(w.petal_tick(), "길에서 밟으면 줍는다");
        assert!(w.story.petal_found() && w.story.petals().is_empty());
        assert!(w
            .adv
            .events
            .contains(&crate::adventure::AdvEvent::PetalFound));
    }

    /// 토끼풀 가족(0165): 실이 풀린 뒤 고치가 풀려(월드의 고치가 없어지고 꽃잎) 아기 둘이 나타나고, 셋이 떠난다.
    #[test]
    fn clover_family_hatches_after_unbind_and_leaves() {
        let mut w = World::new();
        let at = w.player.pos + vec2(0.0, -40.0);
        w.hazards.cocoons = vec![at + vec2(-13.0, -10.0), at + vec2(13.0, -8.0)];
        let mut c = clover_returns(&w, at);
        assert!(!c.actors[2].visible && !c.actors[3].visible);
        let go = Actions {
            confirm: true,
            ..Default::default()
        };
        let mut seen_babies = false;
        let mut saw_petals = false;
        for _ in 0..1200 {
            c.update(&go, 1.0 / 60.0, &mut w);
            saw_petals |= w
                .juice
                .fx
                .iter()
                .any(|f| matches!(f.kind, crate::world::FxKind::Petal { .. }));
            if c.actors[2].visible && c.actors[3].visible {
                seen_babies = true;
                assert!(
                    w.hazards.cocoons.is_empty(),
                    "아기들이 보이면 고치는 풀렸다"
                );
            }
            if c.is_done() {
                break;
            }
        }
        assert!(c.is_done() && seen_babies);
        assert!(
            !c.actors[1].visible && !c.actors[2].visible,
            "가족이 먼저 떠난다"
        );
        assert!(saw_petals, "고치가 풀릴 때 분홍 꽃잎");
    }

    /// 돌아오는 장면(0159): 실이 풀리는 이펙트가 먼저 나고, 그 뒤에야 배우가 보인다.
    #[test]
    fn returning_neighbor_appears_after_threads_unbind() {
        let mut w = World::new();
        let at = w.player.pos + vec2(60.0, 0.0);
        let mut c = doto_returns(&w, at);
        assert!(!c.actors[1].visible, "처음엔 안 보인다");
        let quiet = Actions::default();
        let mut unbound_at = None;
        for i in 0..200 {
            c.update(&quiet, 1.0 / 60.0, &mut w);
            if unbound_at.is_none()
                && w.juice
                    .fx
                    .iter()
                    .any(|f| matches!(f.kind, crate::world::FxKind::Thread { .. }))
            {
                unbound_at = Some(i);
            }
            if c.actors[1].visible {
                assert!(unbound_at.is_some_and(|u| u < i), "실이 풀린 뒤에 나타난다");
                return;
            }
        }
        panic!("배우가 나타나지 않는다");
    }
}

#[cfg(test)]
mod portrait_tests {
    use super::*;

    #[test]
    fn momo_faces_map_to_her_two_frames() {
        assert_eq!(Portrait::MOMO_TAINTED.frame(Face::Normal), 7);
        assert_eq!(Portrait::MOMO_TAINTED.frame(Face::Worried), 8);
        assert_eq!(Portrait::POPO.frame(Face::Happy), 6);
        assert_eq!(Portrait::NAIL.frame(Face::Determined), 3);
        assert_eq!(Portrait::nail(0).frame(Face::Happy), 1);
        assert_eq!(Portrait::nail(1).frame(Face::Normal), 18);
        assert_eq!(Portrait::nail(3).frame(Face::Determined), 29);
        assert_eq!(Portrait::CHIEF.frame(Face::Worried), 9);
        assert_eq!(Portrait::BORI.frame(Face::Happy), 11);
        assert_eq!(Portrait::MOMO.frame(Face::Worried), 14);
        assert_eq!(Portrait::HALMAE.frame(Face::Worried), 15);
        assert_eq!(Portrait::CLOVER_TAINTED.frame(Face::Worried), 38);
        assert_eq!(Portrait::CLOVER.frame(Face::Happy), 40);
        assert_eq!(Portrait::CLOVER.frame(Face::Worried), 39);
        assert_eq!(Portrait::MOM.frame(Face::Happy), 17);
    }
}

/// 배우의 월드 그림.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sprite {
    /// critters.png의 줄.
    Critter(u32),
    /// chief.png의 돌아온 보리.
    Bori,
}

#[derive(Clone)]
pub struct Actor {
    pub name: &'static str,
    /// 월드 그림(없으면 색 동그라미).
    pub sprite: Option<Sprite>,
    pub color: Color,
    pub portrait: Option<Portrait>,
    pub pos: Vec2,
    pub facing: Vec2,
    pub visible: bool,
}

#[derive(Clone)]
pub enum Step {
    /// 대화. 확인으로 넘긴다. 다 찍히기 전에 확인하면 전부 보여 준다.
    /// `arg`가 있으면 글의 `{0}`을 그 동작의 지금 버튼 이름으로 채운다.
    Say {
        actor: usize,
        face: Face,
        text: Tr,
        arg: Option<Act>,
    },
    /// 배우를 목표까지 걷게 한다(px/s).
    Move {
        actor: usize,
        to: Vec2,
        speed: f32,
    },
    Face {
        actor: usize,
        dir: Vec2,
    },
    Wait(f32),
    /// 건너뛰기가 멈추는 자리: 앞을 건너뛰면 여기서 다시 재생한다(한 번 더 건너뛰면 그다음까지). 오프닝의 모모 등장 앞(0164 덧붙임).
    Checkpoint,
    /// 무기를 준다(equipment::WEAPONS 번호). 스토리 보상. 앱이 가방에 넣고 획득 창을 띄운다.
    /// 첫 사용처는 2단계 보스 처치 컷씬(docs/decisions/0012). 그때까지 쓰는 스크립트가 없다.
    #[allow(dead_code)]
    Give(usize),
    Show {
        actor: usize,
        at: Vec2,
    },
    Hide {
        actor: usize,
    },
    /// 검은 화면에서 밝아짐(true) / 어두워짐(false), 초.
    Fade {
        to_clear: bool,
        secs: f32,
    },
    Shake(f32),
    /// 카메라를 `to`가 가운데 오도록 `secs` 동안 옮긴다(0159). `None`이면 플레이어에게 돌아가고 잡아 둔 것을 푼다.
    Pan {
        to: Option<Vec2>,
        secs: f32,
    },
    /// 어둠(0159): `secs` 동안 `to`(0~1)까지. 빛의 자리는 `actor`(0 = 플레이어), 반지름 `radius`.
    Gloom {
        to: f32,
        actor: usize,
        radius: f32,
        secs: f32,
    },
    /// 음악을 내린다/올린다(0159). 내린 동안은 보스 곡도 아직.
    Hush(bool),
    /// 월드 이펙트 묶음(0159)을 `at`에.
    Cue {
        cue: Cue,
        at: Vec2,
    },
    /// 화면 가운데 큰 이름 카드(0159): 이름, 한 줄 설명, 초.
    Card {
        name: &'static str,
        sub: Tr,
        secs: f32,
    },
    /// 화면 전체 일러스트를 띄운다(`None`이면 내린다). 대화·내레이션은 그 위에 뜬다(docs/decisions/0039).
    /// 바꾸는 순간은 보통 `Fade`로 가린다.
    Illust(Option<Illust>),
    /// 내레이션: 이름·초상화 없이 아래에 글만. 넘기기는 대화와 같다.
    Narrate(Tr),
}

pub struct Cutscene {
    pub actors: Vec<Actor>,
    steps: Vec<Step>,
    idx: usize,
    /// 현재 스텝 안에서 흐른 시간.
    t: f32,
    /// 대화: 지금까지 보인 글자 수.
    shown: f32,
    /// 검은 화면 알파(0 = 안 보임).
    pub fade: f32,
    done: bool,
    /// 컷씬이 시작되고 흐른 시간.
    age: f32,
    /// 지금 떠 있는 일러스트와 뜬 뒤 흐른 시간(패닝·입자).
    pub illust: Option<Illust>,
    pub illust_t: f32,
    /// 지금 스텝이 이번 프레임에 시작했나(팬·어둠의 출발값을 잡는다).
    fresh: bool,
    /// 팬·어둠의 출발값.
    pan_from: Vec2,
    gloom_from: f32,
    /// 빛이 떠나는 자리·반지름(0190): 새 자리로 미끄러지듯 옮긴다(한 번에 뛰면 어색했다, 사장).
    gloom_at0: Vec2,
    gloom_r0: f32,
}

/// 렌더용: 지금 열린 대화창.
pub struct Dialogue {
    pub name: &'static str,
    pub color: Color,
    /// 초상화 시트 칸. 없으면 색 네모.
    pub portrait: Option<u32>,
    pub text: String,
    pub shown_chars: usize,
    pub complete: bool,
    /// 내레이션(이름표·초상화 없음).
    pub narration: bool,
}

impl Cutscene {
    pub fn new(actors: Vec<Actor>, steps: Vec<Step>) -> Self {
        // 첫 그림을 정하는 스텝 뒤에 밝아지기가 오면 검은 화면에서 시작한다.
        let fade = match steps.iter().find(|s| !matches!(s, Step::Illust(_))) {
            Some(Step::Fade { to_clear: true, .. }) => 1.0,
            _ => 0.0,
        };
        Self {
            actors,
            steps,
            idx: 0,
            t: 0.0,
            shown: 0.0,
            fade,
            done: false,
            age: 0.0,
            illust: None,
            illust_t: 0.0,
            fresh: true,
            pan_from: Vec2::ZERO,
            gloom_from: 0.0,
            gloom_at0: Vec2::ZERO,
            gloom_r0: 0.0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    /// 이 배우가 지금 걷는 중인가(그림 칸 고르기).
    pub fn moving(&self, actor: usize) -> bool {
        matches!(self.steps.get(self.idx), Some(Step::Move { actor: a, .. }) if *a == actor)
    }

    fn actor_pos<'a>(&'a mut self, w: &'a mut World, i: usize) -> &'a mut Vec2 {
        if i == 0 {
            &mut w.player.pos
        } else {
            &mut self.actors[i].pos
        }
    }

    /// 한 프레임. 건너뛰기면 남은 효과를 즉시 적용하고 끝낸다.
    pub fn update(&mut self, a: &Actions, dt: f32, w: &mut World) {
        if self.done {
            return;
        }
        if a.cancel || a.menu {
            self.finish_fast(w);
            return;
        }
        // 입자 효과(실·꽃잎·반짝임)와 이름 카드는 컷씬 동안에도 흐른다(0159). 로직은 멈춘다.
        w.tick_fx(dt);
        w.time += dt;
        if let Some((_, _, left)) = &mut w.juice.boss_card {
            *left -= dt;
            if *left <= 0.0 {
                w.juice.boss_card = None;
            }
        }
        let Some(step) = self.steps.get(self.idx).cloned() else {
            self.done = true;
            return;
        };
        let fresh = std::mem::replace(&mut self.fresh, false);
        self.t += dt;
        self.age += dt;
        self.illust_t += dt;
        let advance = match step {
            Step::Say { text, arg, .. } => self.type_text(&say_text(text, arg), a, dt),
            Step::Narrate(text) => self.type_text(i18n::tr(text), a, dt),
            Step::Illust(il) => {
                self.illust = il;
                self.illust_t = 0.0;
                true
            }
            Step::Move { actor, to, speed } => {
                let p = self.actor_pos(w, actor);
                let d = to - *p;
                let step_len = speed * dt;
                if d.length() <= step_len {
                    *p = to;
                    true
                } else {
                    let dir = d.normalize();
                    *p += dir * step_len;
                    self.set_facing(w, actor, dir);
                    false
                }
            }
            Step::Face { actor, dir } => {
                self.set_facing(w, actor, dir);
                true
            }
            Step::Wait(secs) => self.t >= secs,
            Step::Checkpoint => true,
            Step::Give(i) => {
                w.inv.gifts.push(crate::world::Gift::Weapon(i));
                true
            }
            Step::Show { actor, at } => {
                self.actors[actor].pos = at;
                self.actors[actor].visible = true;
                true
            }
            Step::Hide { actor } => {
                self.actors[actor].visible = false;
                true
            }
            Step::Fade { to_clear, secs } => {
                let p = (self.t / secs.max(0.01)).min(1.0);
                self.fade = if to_clear { 1.0 - p } else { p };
                p >= 1.0
            }
            Step::Shake(amount) => {
                w.juice.feel.add_trauma(amount);
                true
            }
            Step::Pan { to, secs } => {
                if fresh {
                    self.pan_from = w.adv.cam;
                }
                let focus = to.unwrap_or(w.player.pos);
                let target = w.cam_at(focus);
                let p = (self.t / secs.max(0.01)).min(1.0);
                let k = p * p * (3.0 - 2.0 * p);
                w.adv.cam = self.pan_from + (target - self.pan_from) * k;
                w.adv.prev_cam = w.adv.cam;
                if p >= 1.0 {
                    w.adv.cam_hold = to;
                    true
                } else {
                    false
                }
            }
            Step::Gloom {
                to,
                actor,
                radius,
                secs,
            } => {
                let p = (self.t / secs.max(0.01)).min(1.0);
                let target = if actor == 0 {
                    w.player.pos
                } else {
                    self.actors[actor].pos
                };
                if fresh {
                    self.gloom_from = w.juice.gloom.amount;
                    // 어둠이 아직 없으면 빛은 처음부터 그 자리에(어디서 미끄러져 올 데가 없다).
                    let (at0, r0) = if w.juice.gloom.amount < 0.05 {
                        (target, radius)
                    } else {
                        (w.juice.gloom.at, w.juice.gloom.radius)
                    };
                    self.gloom_at0 = at0;
                    self.gloom_r0 = r0;
                }
                // 빛의 자리·크기는 부드럽게(처음·끝이 느린 곡선) 옮긴다 — Naïl에서 보리로 빛이 건너간다(0190).
                let e = p * p * (3.0 - 2.0 * p);
                w.juice.gloom = crate::world::Gloom {
                    amount: self.gloom_from + (to - self.gloom_from) * p,
                    at: self.gloom_at0.lerp(target, e),
                    radius: self.gloom_r0 + (radius - self.gloom_r0) * e,
                };
                p >= 1.0
            }
            Step::Hush(on) => {
                w.juice.hush = on;
                true
            }
            Step::Cue { cue, at } => {
                w.cue(cue, at);
                true
            }
            Step::Card { name, sub, secs } => {
                w.juice.boss_card = Some((name, sub, secs));
                true
            }
        };
        if advance {
            self.idx += 1;
            self.t = 0.0;
            self.shown = 0.0;
            self.fresh = true;
            if self.idx >= self.steps.len() {
                self.done = true;
            }
        }
        w.freeze();
    }

    /// 글이 한 자씩 찍힌다. 다 찍히기 전에 누르면 전부, 다 찍힌 뒤 누르면 다음(true).
    fn type_text(&mut self, text: &str, a: &Actions, dt: f32) -> bool {
        let total = text.chars().count() as f32;
        let complete = self.shown >= total;
        let pressed = a.confirm || a.attack || a.tap.is_some();
        if pressed && self.age >= START_GUARD && self.t >= LINE_GUARD {
            if complete {
                true
            } else {
                self.shown = total;
                false
            }
        } else {
            self.shown = (self.shown + CHARS_PER_SEC * dt).min(total);
            false
        }
    }

    fn set_facing(&mut self, w: &mut World, actor: usize, dir: Vec2) {
        if dir == Vec2::ZERO {
            return;
        }
        if actor == 0 {
            w.player.facing = dir.normalize();
        } else {
            self.actors[actor].facing = dir.normalize();
        }
    }

    fn finish_fast(&mut self, w: &mut World) {
        // 다음 멈춤 자리(Checkpoint)까지만, 없으면 끝까지. 멈춤 자리가 있으면 거기서 다시 재생한다.
        let stop = self.steps[self.idx..]
            .iter()
            .position(|s| matches!(s, Step::Checkpoint));
        let end = stop.map_or(self.steps.len(), |k| self.idx + k + 1);
        let rest: Vec<Step> = self.steps.drain(self.idx..end).collect();
        if stop.is_some() {
            self.t = 0.0;
            self.shown = 0.0;
            self.fresh = true;
        }
        for step in rest {
            match step {
                Step::Move { actor, to, .. } => *self.actor_pos(w, actor) = to,
                Step::Face { actor, dir } => self.set_facing(w, actor, dir),
                Step::Show { actor, at } => {
                    self.actors[actor].pos = at;
                    self.actors[actor].visible = true;
                }
                Step::Hide { actor } => self.actors[actor].visible = false,
                Step::Give(i) => w.inv.gifts.push(crate::world::Gift::Weapon(i)),
                Step::Fade { to_clear, .. } => self.fade = if to_clear { 0.0 } else { 1.0 },
                Step::Illust(il) => self.illust = il,
                // 장치(0159)는 끝값만: 카메라는 잡아 둔 자리(또는 플레이어)로, 어둠은 그 값으로, 음악은 그대로.
                Step::Pan { to, .. } => {
                    w.adv.cam_hold = to;
                    w.snap_cam();
                }
                Step::Gloom {
                    to, actor, radius, ..
                } => {
                    w.juice.gloom = crate::world::Gloom {
                        amount: to,
                        at: if actor == 0 {
                            w.player.pos
                        } else {
                            self.actors[actor].pos
                        },
                        radius,
                    };
                }
                Step::Hush(on) => w.juice.hush = on,
                // 남는 것(0164): 건너뛰어도 길의 시든 꽃잎은 둔다(이펙트는 없이). 선 자리(`Leave`)는 흩날리기뿐이라 없다(0194).
                Step::Cue {
                    cue: Cue::Trail,
                    at,
                } => w.story.add_petal(at),
                _ => {}
            }
        }
        if stop.is_none() {
            self.done = true;
            w.freeze();
        }
    }

    /// 지금 보여 줄 대화창.
    pub fn dialogue(&self) -> Option<Dialogue> {
        match self.steps.get(self.idx)? {
            Step::Say {
                actor,
                face,
                text,
                arg,
            } => {
                let text = say_text(*text, *arg);
                let total = text.chars().count();
                let shown = (self.shown as usize).min(total);
                let a = &self.actors[*actor];
                Some(Dialogue {
                    name: a.name,
                    color: a.color,
                    portrait: a.portrait.map(|p| p.frame(*face)),
                    text,
                    shown_chars: shown,
                    complete: shown >= total,
                    narration: false,
                })
            }
            Step::Narrate(text) => {
                let text = i18n::tr(*text).to_string();
                let total = text.chars().count();
                let shown = (self.shown as usize).min(total);
                Some(Dialogue {
                    name: "",
                    color: WHITE,
                    portrait: None,
                    text,
                    shown_chars: shown,
                    complete: shown >= total,
                    narration: true,
                })
            }
            _ => None,
        }
    }
}

/// 대사 글: 자리표시자가 있으면 지금 버튼 이름으로.
fn say_text(text: Tr, arg: Option<Act>) -> String {
    let t = i18n::tr(text);
    match arg {
        Some(a) => i18n::fill(t, &[&crate::ui::key(crate::input::prompt(a))]),
        None => t.to_string(),
    }
}

fn say(actor: usize, face: Face, text: Tr) -> Step {
    Step::Say {
        actor,
        face,
        text,
        arg: None,
    }
}

/// 포포 배우(배우 1번). 월드 그림은 critters.png 라쿤 줄.
fn popo_actor(pos: Vec2, visible: bool) -> Actor {
    Actor {
        name: i18n::s().popo_name,
        sprite: Some(Sprite::Critter(crate::art::RACCOON_ROW)),
        color: Color::new(0.55, 0.55, 0.60, 1.0),
        portrait: Some(Portrait::POPO),
        pos,
        facing: vec2(-1.0, 0.0),
        visible,
    }
}

/// 플레이어 배우: 이번 런에 내려간 캐릭터(Naïl 또는 모모, 0056). 몸은 월드가 그린다.
fn nail_actor(w: &World) -> Actor {
    let momo = w.character == crate::equipment::Character::Momo;
    Actor {
        name: w.character.name(),
        sprite: None,
        color: if momo { MOMO_PETAL } else { crate::world::MOLE },
        portrait: Some(if momo {
            Portrait::MOMO
        } else {
            Portrait::nail(w.inv.fur)
        }),
        pos: w.player.pos,
        facing: w.player.facing,
        visible: true,
    }
}

/// 튜토리얼 한마디: 포포(월드의 npc 자리)가 말한다. 줄마다 (글, 표정, 버튼 자리표시자).
pub fn popo_talk(w: &World, lines: &[(Tr, Face, Option<Act>)]) -> Cutscene {
    let at = w
        .npc
        .map(|n| n.pos)
        .unwrap_or(w.player.pos + vec2(24.0, 0.0));
    let actors = vec![nail_actor(w), popo_actor(at, false)];
    let steps = lines
        .iter()
        .map(|(text, face, arg)| Step::Say {
            actor: 1,
            face: *face,
            text: *text,
            arg: *arg,
        })
        .collect();
    Cutscene::new(actors, steps)
}

/// 대화에 나오는 이(쉼터·보스 장면). 배우는 보이지 않고 말만 한다(월드가 그 이웃을 그린다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    Nail,
    Popo,
    /// 물든 들쥐 대장(이름표 "들쥐 대장").
    Chief,
    Bori,
    /// 누룩 할매(마을, 0045). 초상화 칸 15~16(0049).
    Halmae,
    /// 물든 모모(첫 길의 보스, 이름표 "???", 0046).
    MomoTainted,
    /// 돌아온 모모(마을). 초상화 칸 12~14(0049).
    Momo,
    /// 물든 도토(둘째 길의 보스, 이름표 "???") / 돌아온 도토(마을, 0052).
    DotoTainted,
    Doto,
    /// 땅강아지 장인 수수(첫 쉼터, 0053).
    Susu,
    /// 물든 콩이(셋째 길의 보스, 이름표 "???") / 돌아온 콩이(마을, 0054).
    KongiTainted,
    Kongi,
    /// 물든 토끼풀(굴토끼 옛 굴의 보스, 이름표 "???") / 돌아온 토끼풀(0165). 초상화는 아직 없다(색 네모).
    CloverTainted,
    Clover,
}

fn speaker_actor(w: &World, who: Speaker) -> Actor {
    let hidden = |name, portrait, color| Actor {
        name,
        sprite: None,
        color,
        portrait: Some(portrait),
        pos: w.player.pos,
        facing: vec2(1.0, 0.0),
        visible: false,
    };
    let fur = Color::new(0.62, 0.45, 0.36, 1.0);
    match who {
        Speaker::Nail => nail_actor(w),
        Speaker::Popo => popo_actor(w.player.pos, false),
        Speaker::Chief => hidden(i18n::tr(crate::species::CHIEF.name), Portrait::CHIEF, fur),
        Speaker::Bori => hidden(i18n::s().bori_name, Portrait::BORI, fur),
        Speaker::Halmae => halmae_actor(),
        Speaker::MomoTainted => momo_tainted_actor(w.player.pos),
        Speaker::Momo => momo_actor(),
        Speaker::DotoTainted => Actor {
            name: "???",
            sprite: Some(Sprite::Critter(crate::art::DOTO_ROW)),
            portrait: Some(Portrait::DOTO_TAINTED),
            pos: w.player.pos,
            ..story_actor("???", DOTO_FUR)
        },
        Speaker::Doto => doto_actor(w.player.pos),
        Speaker::Susu => susu_actor(w.player.pos),
        Speaker::KongiTainted => Actor {
            sprite: Some(Sprite::Critter(crate::art::KONGI_ROW)),
            portrait: Some(Portrait::KONGI_TAINTED),
            pos: w.player.pos,
            ..story_actor("???", KONGI_SHELL)
        },
        Speaker::Kongi => kongi_actor(w.player.pos),
        Speaker::CloverTainted => Actor {
            sprite: Some(Sprite::Critter(crate::art::CLOVER_ROW)),
            portrait: Some(Portrait::CLOVER_TAINTED),
            pos: w.player.pos,
            ..story_actor("???", CLOVER_FUR)
        },
        Speaker::Clover => clover_actor(w.player.pos),
    }
}

/// 토끼풀의 색(이름표·말풍선 테두리, 0165).
const CLOVER_FUR: Color = Color::new(0.62, 0.52, 0.48, 1.0);

/// 돌아온 토끼풀 배우(월드 그림 = 맑은 눈, 초상화 칸 39~40).
fn clover_actor(pos: Vec2) -> Actor {
    Actor {
        sprite: Some(Sprite::Critter(crate::art::CLOVER_BACK_ROW)),
        portrait: Some(Portrait::CLOVER),
        pos,
        ..story_actor(i18n::s().name_clover, CLOVER_FUR)
    }
}

/// 굴토끼 옛 굴 끝 방에 토끼풀이 있다(0165). 모모 런이면 서로 알아보지 못하는 모녀의 말.
pub fn clover_meet(w: &World) -> Cutscene {
    let at = w.adv.boss_last;
    let actors = vec![nail_actor(w), tainted_actor(w, Speaker::CloverTainted, at)];
    let momo = w.character == crate::equipment::Character::Momo;
    let lines = if momo {
        vec![
            say(1, Face::Normal, |s| s.clover_meet_momo_1),
            say(0, Face::Worried, |s| s.clover_meet_momo_1b),
        ]
    } else {
        vec![
            say(1, Face::Normal, |s| s.clover_meet_1),
            say(0, Face::Determined, |s| s.clover_meet_1b),
        ]
    };
    Cutscene::new(actors, tainted_meet_steps(w, at, |s| s.mob_clover, lines))
}

/// 토끼풀을 이겼다(0165): 실이 풀리고 맑은 눈으로 돌아온다 → "…막내는?" → 고치가 풀려 아기 둘이 나타난다 →
/// 모모가 마을에서 기다린다고(모모 런이면 "엄마… 나 여기 있어") → 가족이 먼저 마을로.
/// 배우: 0 Naïl, 1 토끼풀, 2·3 아기(박하·민들레, 고치 자리에서 나타난다).
pub fn clover_returns(w: &World, at: Vec2) -> Cutscene {
    let mut clover = clover_actor(at);
    clover.visible = false;
    clover.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    let baby = |pos: Vec2| Actor {
        sprite: Some(Sprite::Critter(crate::art::BABY_ROW)),
        pos,
        ..story_actor("", CLOVER_FUR)
    };
    let c0 = w
        .hazards
        .cocoons
        .first()
        .copied()
        .unwrap_or(at + vec2(-14.0, -10.0));
    let c1 = w
        .hazards
        .cocoons
        .get(1)
        .copied()
        .unwrap_or(at + vec2(14.0, -8.0));
    let actors = vec![nail_actor(w), clover, baby(c0), baby(c1)];
    let momo = w.character == crate::equipment::Character::Momo;
    let side = if at.x >= w.player.pos.x { 22.0 } else { -22.0 };
    let near = w.player.pos + vec2(side, 0.0);
    let away = (at - w.player.pos).normalize_or(vec2(0.0, -1.0));
    let mut steps = unbind_steps(at, 1, false);
    // 실이 풀린 뒤, 그동안의 모습 한 장(0165): 실에 감긴 채 아기들의 고치를 품고 잠들어 있던 어미. 내레이션 한 줄.
    steps.extend(cut_to(Some(Illust::CloverCocoon)));
    steps.push(Step::Narrate(|s| s.clover_cocoon));
    steps.extend(cut_to(None));
    steps.extend([
        say(1, Face::Normal, |s| s.clover_b1),
        Step::Move {
            actor: 1,
            to: near,
            speed: 35.0,
        },
        Step::Face {
            actor: 1,
            dir: vec2(-side, 0.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(side, 0.0),
        },
        if momo {
            say(0, Face::Happy, |s| s.clover_b2_momo)
        } else {
            say(0, Face::Happy, |s| s.clover_b2)
        },
        // 고치가 풀린다: 실·반짝임·분홍 꽃잎, 아기 둘이 나타난다.
        Step::Cue {
            cue: Cue::Hatch,
            at: c0,
        },
        Step::Shake(0.15),
        Step::Wait(0.6),
        Step::Show { actor: 2, at: c0 },
        Step::Show { actor: 3, at: c1 },
        Step::Cue {
            cue: Cue::Sparkle,
            at: c0,
        },
        Step::Cue {
            cue: Cue::Sparkle,
            at: c1,
        },
        Step::Wait(0.6),
        if momo {
            say(1, Face::Happy, |s| s.clover_b3_momo)
        } else {
            say(1, Face::Happy, |s| s.clover_b3)
        },
        // 아기들이 어미 곁으로 깡충, 그리고 셋이 먼저 떠난다.
        Step::Move {
            actor: 2,
            to: near + vec2(-10.0, 6.0),
            speed: 70.0,
        },
        Step::Move {
            actor: 3,
            to: near + vec2(10.0, 7.0),
            speed: 70.0,
        },
        Step::Wait(0.4),
        Step::Move {
            actor: 1,
            to: near + away * 150.0,
            speed: 80.0,
        },
        Step::Move {
            actor: 2,
            to: near + away * 150.0 + vec2(-10.0, 6.0),
            speed: 80.0,
        },
        Step::Move {
            actor: 3,
            to: near + away * 150.0 + vec2(10.0, 7.0),
            speed: 80.0,
        },
        Step::Hide { actor: 1 },
        Step::Hide { actor: 2 },
        Step::Hide { actor: 3 },
    ]);
    Cutscene::new(actors, steps)
}

/// 콩이의 색(초상화를 못 읽었을 때 이름표 옆 네모).
const KONGI_SHELL: Color = Color::new(0.62, 0.56, 0.50, 1.0);

/// 돌아온 콩이 배우(월드 그림 = 맑은 눈, 초상화 칸 36~37).
fn kongi_actor(pos: Vec2) -> Actor {
    Actor {
        sprite: Some(Sprite::Critter(crate::art::KONGI_BACK_ROW)),
        portrait: Some(Portrait::KONGI),
        pos,
        ..story_actor(i18n::s().name_kongi, KONGI_SHELL)
    }
}

/// 셋째 길 끝 방에 콩이가 있다(0054): 깊은 곳의 문을 막는다. "아무도 떠나지 마"(0037)의 문지기판.
pub fn kongi_meet(w: &World) -> Cutscene {
    let at = w.adv.boss_last;
    let actors = vec![nail_actor(w), tainted_actor(w, Speaker::KongiTainted, at)];
    let lines = vec![
        say(1, Face::Normal, |s| s.kongi_meet_1),
        say(0, Face::Determined, |s| s.kongi_meet_1b),
    ];
    Cutscene::new(actors, tainted_meet_steps(w, at, |s| s.mob_kongi, lines))
}

/// 콩이를 이겼다(0054): 맑은 눈으로 돌아와 이름을 말하고, 붙잡는 것만이 지키는 게 아니었다며 바위 발톱을 준다. 먼저 마을로 간다.
pub fn kongi_returns(w: &World, at: Vec2) -> Cutscene {
    let mut kongi = kongi_actor(at);
    kongi.visible = false;
    kongi.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    let actors = vec![nail_actor(w), kongi];
    let side = if at.x >= w.player.pos.x { 22.0 } else { -22.0 };
    let near = w.player.pos + vec2(side, 0.0);
    let away = (at - w.player.pos).normalize_or(vec2(0.0, -1.0));
    let mut steps = unbind_steps(at, 1, false);
    steps.extend([
        say(1, Face::Normal, |s| s.kongi_b1),
        Step::Move {
            actor: 1,
            to: near,
            speed: 35.0,
        },
        Step::Face {
            actor: 1,
            dir: vec2(-side, 0.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(side, 0.0),
        },
        say(0, Face::Worried, |s| s.kongi_b2),
    ]);
    // 깊은 곳의 문(0085): 콩이가 지키던 문, 틈으로 빛이 샌다. 그림 위에서 내레이션 + 콩이의 말, 그리고 돌아온다.
    steps.extend(cut_to(Some(Illust::KongiDoor)));
    steps.extend([
        Step::Narrate(|s| s.kongi_door),
        say(1, Face::Normal, |s| s.kongi_b3),
        say(1, Face::Normal, |s| s.kongi_b4),
    ]);
    steps.extend(cut_to(None));
    steps.extend([
        Step::Give(crate::equipment::STONE_CLAW),
        say(0, Face::Happy, |s| s.kongi_b5),
        say(1, Face::Happy, |s| s.kongi_b6),
        Step::Move {
            actor: 1,
            to: near + away * 150.0,
            speed: 90.0,
        },
        Step::Hide { actor: 1 },
    ]);
    Cutscene::new(actors, steps)
}

/// 수수 배우(초상화 칸 33~34). 쉼터의 월드 그림이 서 있으니 배우는 말만 한다.
fn susu_actor(pos: Vec2) -> Actor {
    Actor {
        portrait: Some(Portrait::SUSU),
        pos,
        ..story_actor(i18n::s().name_susu, Color::new(0.52, 0.38, 0.28, 1.0))
    }
}

/// 수수를 처음 만났다(0053): 같은 "파는 손"을 알아보고 새싹 발톱을 만들어 준다(획득 창).
pub fn susu_meet(w: &World) -> Cutscene {
    let actors = vec![nail_actor(w), susu_actor(w.player.pos)];
    let steps = vec![
        say(1, Face::Happy, |s| s.susu_1),
        say(0, Face::Normal, |s| s.susu_2),
        say(1, Face::Normal, |s| s.susu_3),
        say(1, Face::Happy, |s| s.susu_4),
        Step::Give(crate::equipment::SPROUT_CLAW),
        say(0, Face::Happy, |s| s.susu_5),
    ];
    Cutscene::new(actors, steps)
}

/// 도토의 색(초상화를 못 읽었을 때 이름표 옆 네모).
const DOTO_FUR: Color = Color::new(0.70, 0.42, 0.28, 1.0);

/// 돌아온 도토 배우(월드 그림 = 맑은 눈, 초상화 칸 31~32).
fn doto_actor(pos: Vec2) -> Actor {
    Actor {
        sprite: Some(Sprite::Critter(crate::art::DOTO_BACK_ROW)),
        portrait: Some(Portrait::DOTO),
        pos,
        ..story_actor(i18n::s().name_doto, DOTO_FUR)
    }
}

/// 돌아온 모모의 색(초상화를 못 읽었을 때 이름표 옆 네모).
const MOMO_PETAL: Color = Color::new(0.80, 0.62, 0.66, 1.0);

/// 돌아온 모모 배우(월드 그림 = 꽃 핀 모습, 초상화 칸 12~14).
fn momo_actor() -> Actor {
    Actor {
        sprite: Some(Sprite::Critter(crate::art::MOMO_BLOOM_ROW)),
        portrait: Some(Portrait::MOMO),
        ..story_actor(i18n::s().name_momo, MOMO_PETAL)
    }
}

/// 누룩 할매 배우(초상화 칸 15~16).
fn halmae_actor() -> Actor {
    Actor {
        portrait: Some(Portrait::HALMAE),
        ..story_actor(i18n::s().name_halmae, Color::new(0.86, 0.84, 0.92, 1.0))
    }
}

/// 물든 모모 배우: 이름은 아직 "???"(이름은 돌아올 때 스스로 말한다, 0027).
fn momo_tainted_actor(pos: Vec2) -> Actor {
    Actor {
        name: "???",
        sprite: Some(Sprite::Critter(crate::art::MOMO_ROW)),
        color: Color::new(0.45, 0.35, 0.50, 1.0),
        portrait: Some(Portrait::MOMO_TAINTED),
        pos,
        facing: vec2(-1.0, 0.0),
        visible: false,
    }
}

/// 여러 이웃이 번갈아 말하는 짧은 대화(쉼터·보스 장면). 배우 순서 = `Speaker` 순서.
pub fn talk(w: &World, lines: &[(Speaker, Tr, Face)]) -> Cutscene {
    let order = [
        Speaker::Nail,
        Speaker::Popo,
        Speaker::Chief,
        Speaker::Bori,
        Speaker::Halmae,
        Speaker::MomoTainted,
        Speaker::Momo,
        Speaker::DotoTainted,
        Speaker::Doto,
        Speaker::Susu,
        Speaker::KongiTainted,
        Speaker::Kongi,
    ];
    let actors = order.iter().map(|s| speaker_actor(w, *s)).collect();
    let steps = lines
        .iter()
        .map(|(who, text, face)| {
            say(
                order.iter().position(|s| s == who).unwrap_or(0),
                *face,
                *text,
            )
        })
        .collect();
    Cutscene::new(actors, steps)
}

/// 물든 것의 등장(0163): 음악이 끊기고 어둠이 Naïl 둘레만 남긴 채 내려앉는다 → 빛·카메라가 `at`으로 →
/// 보라 울림으로 일어선다 → '???' 카드(도감 이름) → 말 → 빛·카메라가 돌아오고 보스 곡.
/// 모모의 두 번째 만남과 같은 길이(짧은 판). 배우 0 = Naïl, 1 = 물든 것(월드의 적을 그대로 그린다).
fn tainted_meet_steps(w: &World, at: Vec2, sub: Tr, lines: Vec<Step>) -> Vec<Step> {
    let mut steps = vec![
        Step::Hush(true),
        Step::Gloom {
            to: 0.88,
            actor: 0,
            radius: 64.0,
            secs: 0.5,
        },
        Step::Wait(0.2),
        Step::Face {
            actor: 0,
            dir: at - w.player.pos,
        },
        Step::Pan {
            to: Some(at),
            secs: 0.9,
        },
        // 빛이 Naïl에서 물든 것으로 건너간다(0190, 한 번에 뛰지 않고).
        Step::Gloom {
            to: 0.88,
            actor: 1,
            radius: 58.0,
            secs: 0.9,
        },
        Step::Cue { cue: Cue::Rise, at },
        Step::Shake(0.2),
        Step::Wait(0.6),
        Step::Card {
            name: "???",
            sub,
            secs: 3.0,
        },
    ];
    steps.extend(lines);
    steps.extend([
        Step::Pan {
            to: None,
            secs: 0.7,
        },
        Step::Gloom {
            to: 0.0,
            actor: 1,
            radius: 58.0,
            secs: 0.6,
        },
        Step::Hush(false),
    ]);
    steps
}

/// 물든 것의 배우: 말풍선·초상화만, 자리는 `at`(어둠의 빛이 거기 선다). 그림은 월드의 적.
fn tainted_actor(w: &World, who: Speaker, at: Vec2) -> Actor {
    let mut a = speaker_actor(w, who);
    a.pos = at;
    a.visible = false;
    a.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    a
}

/// 공터에 들어섰다: 대장이 깨어나 길을 막는다(등장 연출 0163).
pub fn boss_intro(w: &World) -> Cutscene {
    let at = w
        .enemies
        .iter()
        .find(|e| e.kind == crate::enemy::Kind::Chief)
        .map(|e| e.pos)
        .unwrap_or(w.player.pos + vec2(0.0, -40.0));
    let actors = vec![nail_actor(w), tainted_actor(w, Speaker::Chief, at)];
    let lines = vec![
        say(1, Face::Normal, |s| s.chief_intro_1),
        say(0, Face::Determined, |s| s.chief_intro_2),
    ];
    Cutscene::new(actors, tainted_meet_steps(w, at, |s| s.mob_chief, lines))
}

/// 대장을 이겼다: 보리로 돌아와 고맙다 하고, 굴 입구로 먼저 내려간다.
pub fn boss_beaten(w: &World, at: Vec2, entrance: Vec2) -> Cutscene {
    let mut actors: Vec<Actor> = [Speaker::Nail, Speaker::Bori]
        .iter()
        .map(|s| speaker_actor(w, *s))
        .collect();
    actors[1].sprite = Some(Sprite::Bori);
    actors[1].pos = at;
    actors[1].visible = false;
    actors[1].facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    // 보리가 Naïl 곁으로 걸어온다(카메라는 Naïl을 따르니, 멀리 쓰러지면 대화창 뒤에 가려진다).
    let side = if at.x >= w.player.pos.x { 26.0 } else { -26.0 };
    let near = w.player.pos + vec2(side, 0.0);
    // 쓰러진 자리에서 실이 풀린다(월드 이펙트, 0159) → 일러스트(0039) → 월드로 돌아와 보리가 걸어온다.
    let mut steps = unbind_steps(at, 1, false);
    steps.extend(cut_to(Some(Illust::BoriFreed)));
    steps.extend([Step::Narrate(|s| s.bori_free)]);
    steps.extend(cut_to(None));
    steps.extend([
        Step::Move {
            actor: 1,
            to: near,
            speed: 45.0,
        },
        Step::Face {
            actor: 1,
            dir: vec2(-side, 0.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(side, 0.0),
        },
        say(1, Face::Normal, |s| s.chief_beaten_1),
        say(0, Face::Worried, |s| s.chief_beaten_2),
        say(1, Face::Happy, |s| s.chief_beaten_3),
        say(1, Face::Normal, |s| s.chief_beaten_4),
        Step::Move {
            actor: 1,
            to: entrance,
            speed: 50.0,
        },
        Step::Hide { actor: 1 },
        say(0, Face::Determined, |s| s.chief_beaten_5),
    ]);
    Cutscene::new(actors, steps)
}

/// 길의 주인을 만나는 장면(0170 R12, 표는 `bosses.rs`): 주인마다 하나. 새 보스는 여기 가지 하나.
pub fn keeper_meet(kind: crate::enemy::Kind, w: &World) -> Cutscene {
    use crate::enemy::Kind;
    match kind {
        Kind::Momo => momo_meet(w, w.story.momo_wins().saturating_add(1), w.adv.boss_last),
        Kind::Doto => doto_meet(w),
        Kind::Kongi => kongi_meet(w),
        Kind::Clover => clover_meet(w),
        other => unreachable!("길의 주인이 아니다: {other:?}"),
    }
}

/// 길의 주인을 이긴 뒤의 장면: 모모는 돌아오거나(세 번째) 달아나고, 나머지는 돌아온다.
pub fn keeper_beaten(kind: crate::enemy::Kind, w: &World, r: crate::world::BossBeaten) -> Cutscene {
    use crate::enemy::Kind;
    let at = w.adv.boss_last;
    match kind {
        Kind::Momo if r.back => momo_returns(w, at),
        Kind::Momo => momo_flee(w, at, r.wins as u32),
        Kind::Doto => doto_returns(w, at),
        Kind::Kongi => kongi_returns(w, at),
        Kind::Clover => clover_returns(w, at),
        other => unreachable!("길의 주인이 아니다: {other:?}"),
    }
}

#[cfg(test)]
mod keeper_tests {
    use super::*;

    /// 보리의 등장(0190): 빛은 한 번에 뛰지 않고 Naïl에서 보리로 건너간다 — 가운데쯤엔 둘 사이에 있다.
    #[test]
    fn the_light_slides_from_nail_to_bori() {
        let mut w = World::new();
        let mut c = boss_intro(&w);
        let from = w.player.pos;
        let bori = c.actors[1].pos;
        let gap = (bori - from).length();
        assert!(gap > 20.0, "시험이 되려면 둘이 떨어져 있어야 한다: {gap}");
        let go = Actions::default();
        let mut seen_between = false;
        for _ in 0..600 {
            c.update(&go, 1.0 / 60.0, &mut w);
            let g = w.juice.gloom;
            let (d0, d1) = ((g.at - from).length(), (g.at - bori).length());
            if g.amount > 0.5 && d0 > gap * 0.25 && d1 > gap * 0.25 {
                seen_between = true;
            }
            if d1 < 1.0 {
                break;
            }
        }
        assert!(seen_between, "빛이 둘 사이를 지나간다");
        assert!((w.juice.gloom.at - bori).length() < 1.0, "끝에는 보리에게");
    }

    /// 보스 표(bosses.rs)의 주인마다 만남·이긴 뒤 장면이 있다(표에 줄을 더하고 가지를 빼먹으면 여기서 터진다).
    #[test]
    fn every_keeper_has_its_scenes() {
        let mut w = World::camp_start(1);
        w.adv.boss_last = w.player.pos;
        for b in &crate::bosses::ALL {
            let _ = keeper_meet(b.kind, &w);
            for back in [false, true] {
                let _ = keeper_beaten(b.kind, &w, crate::world::BossBeaten { wins: 1, back });
            }
        }
    }
}

/// 첫 길 끝 방에 모모가 있다(0046). 만날 때마다 말이 조금씩 무너진다: 막기 → 사라진 가족 → 걱정.
/// 등장(0159): 음악이 끊기고 어둠이 Naïl 둘레만 남긴 채 내려앉는다 → 빛이 천천히 모모 쪽으로 옮겨 가며
/// 시든 꽃잎이 흩날리고 모모가 일어선다(땅 울림) → '???' 카드 → 말 → 빛과 카메라가 돌아오고 보스 곡.
/// `at`은 모모가 선 자리(월드의 적을 그대로 그린다, 배우는 안 보인다).
pub fn momo_meet(w: &World, level: u8, at: Vec2) -> Cutscene {
    let lines: [(Tr, Tr); 3] = [
        (|s| s.momo_meet_1, |s| s.momo_meet_1b),
        (|s| s.momo_meet_2, |s| s.momo_meet_2b),
        (|s| s.momo_meet_3, |s| s.momo_meet_3b),
    ];
    let (momo, nail) = lines[(level.max(1) as usize - 1).min(2)];
    let face = if level >= 3 {
        Face::Worried
    } else {
        Face::Normal
    };
    let mut m = momo_tainted_actor(at);
    m.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    let actors = vec![nail_actor(w), m];
    let first = level <= 1;
    let mut steps = vec![
        Step::Hush(true),
        Step::Gloom {
            to: 0.88,
            actor: 0,
            radius: 64.0,
            secs: if first { 0.9 } else { 0.5 },
        },
        Step::Wait(if first { 0.5 } else { 0.2 }),
        Step::Face {
            actor: 0,
            dir: at - w.player.pos,
        },
        Step::Pan {
            to: Some(at),
            secs: if first { 1.6 } else { 0.9 },
        },
        Step::Gloom {
            to: 0.88,
            actor: 1,
            radius: 58.0,
            secs: if first { 1.2 } else { 0.6 },
        },
        Step::Cue {
            cue: Cue::Petals(false),
            at,
        },
        Step::Wait(0.5),
        Step::Cue { cue: Cue::Rise, at },
        Step::Shake(0.2),
        Step::Wait(0.6),
        Step::Card {
            name: "???",
            sub: |s| s.mob_momo,
            secs: 3.0,
        },
        say(1, face, momo),
        say(0, Face::Determined, nail),
        Step::Pan {
            to: None,
            secs: 0.7,
        },
        Step::Gloom {
            to: 0.0,
            actor: 1,
            radius: 58.0,
            secs: 0.6,
        },
        Step::Hush(false),
    ];
    if !first {
        // 다시 만날 때는 짧게: 꽃잎 없이 일어서고, 카드도 짧게.
        steps.retain(|s| {
            !matches!(
                s,
                Step::Cue {
                    cue: Cue::Petals(_),
                    ..
                }
            )
        });
    }
    Cutscene::new(actors, steps)
}

/// 돌아오는 이웃의 첫 스텝(0159): 쓰러진 자리에서 실이 풀리고(흰 고리·반짝임), 잠시 뒤 제 모습으로 나타난다.
/// `actor`가 0이면 배우를 내보이지 않는다(일러스트로 이어지는 모모). `bloom`이면 분홍 꽃잎도.
fn unbind_steps(at: Vec2, actor: usize, bloom: bool) -> Vec<Step> {
    let mut v = vec![
        Step::Wait(0.5),
        Step::Cue {
            cue: Cue::Unbind,
            at,
        },
        Step::Shake(0.15),
        Step::Wait(0.7),
    ];
    if bloom {
        v.push(Step::Cue {
            cue: Cue::Petals(true),
            at,
        });
    }
    if actor > 0 {
        v.push(Step::Show { actor, at });
        v.push(Step::Cue {
            cue: Cue::Sparkle,
            at,
        });
    }
    v.push(Step::Wait(if bloom { 0.9 } else { 0.5 }));
    v
}

/// 모모를 이겼다(첫 번째·두 번째): 한마디 하고 굴 속으로 달아난다. 마을 빈 자리의 꽃잎이 하나 물든다.
pub fn momo_flee(w: &World, at: Vec2, wins: u32) -> Cutscene {
    let mut momo = momo_tainted_actor(at);
    momo.visible = true;
    momo.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    let actors = vec![nail_actor(w), momo];
    let line: Tr = if wins <= 1 {
        |s| s.momo_flee_1
    } else {
        |s| s.momo_flee_2
    };
    // 플레이어 반대쪽으로 뛰어 사라진다.
    let away = (at - w.player.pos).normalize_or(vec2(0.0, -1.0));
    // 달아나기(0163): 시든 꽃잎이 흩날리며 뛰어가고, 굴 속으로 파고들어 사라진다.
    let hole = at + away * 140.0;
    let steps = vec![
        Step::Wait(0.5),
        say(1, Face::Worried, line),
        Step::Cue {
            cue: Cue::Petals(false),
            at,
        },
        Step::Shake(0.15),
        Step::Move {
            actor: 1,
            to: hole,
            speed: 170.0,
        },
        Step::Cue {
            cue: Cue::Vanish,
            at: hole,
        },
        Step::Hide { actor: 1 },
        Step::Wait(0.4),
        say(0, Face::Worried, |s| s.momo_flee_wait),
        Step::Narrate(|s| s.momo_petal),
    ];
    Cutscene::new(actors, steps)
}

/// 둘째 길 끝 방에 도토가 있다(0052): 묻은 것을 지키려 길을 막는다.
pub fn doto_meet(w: &World) -> Cutscene {
    let at = w.adv.boss_last;
    let actors = vec![nail_actor(w), tainted_actor(w, Speaker::DotoTainted, at)];
    let lines = vec![
        say(1, Face::Normal, |s| s.doto_meet_1),
        say(0, Face::Determined, |s| s.doto_meet_1b),
    ];
    Cutscene::new(actors, tainted_meet_steps(w, at, |s| s.mob_doto, lines))
}

/// 도토를 이겼다(0052): 맑은 눈으로 돌아와 이름을 말하고, 오래전 가장 깊은 곳에 묻은 씨앗을 기억해 낸다(참 엔딩 조건).
/// 그리고 먼저 마을로 간다.
pub fn doto_returns(w: &World, at: Vec2) -> Cutscene {
    let mut doto = doto_actor(at);
    doto.visible = false;
    doto.facing = (w.player.pos - at).normalize_or(vec2(-1.0, 0.0));
    let actors = vec![nail_actor(w), doto];
    let side = if at.x >= w.player.pos.x { 22.0 } else { -22.0 };
    let near = w.player.pos + vec2(side, 0.0);
    let away = (at - w.player.pos).normalize_or(vec2(0.0, -1.0));
    let mut steps = unbind_steps(at, 1, false);
    steps.extend([
        say(1, Face::Normal, |s| s.doto_b1),
        Step::Move {
            actor: 1,
            to: near,
            speed: 45.0,
        },
        Step::Face {
            actor: 1,
            dir: vec2(-side, 0.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(side, 0.0),
        },
        say(0, Face::Worried, |s| s.doto_b2),
        say(1, Face::Happy, |s| s.doto_b3),
    ]);
    // 기억 속 장면(0053): 오래전 흰 실 곁에 씨앗을 묻던 날. 그림 위에서 말하고, 내레이션으로 맺는다.
    steps.extend(cut_to(Some(Illust::DotoMemory)));
    steps.extend([
        say(1, Face::Normal, |s| s.doto_b4),
        Step::Narrate(|s| s.doto_memory),
    ]);
    steps.extend(cut_to(None));
    steps.extend([
        say(0, Face::Determined, |s| s.doto_b5),
        say(1, Face::Happy, |s| s.doto_b6),
        Step::Move {
            actor: 1,
            to: near + away * 150.0,
            speed: 120.0,
        },
        Step::Hide { actor: 1 },
    ]);
    Cutscene::new(actors, steps)
}

/// 쉼터에 닿았다. 첫 쉼터는 포포·보리가 쉼터를 알려 주고, 그 뒤로는 보리 한마디.
pub fn camp_arrive(w: &World, stage: u32) -> Cutscene {
    if stage <= 1 {
        talk(
            w,
            &[
                (Speaker::Popo, |s| s.camp_1, Face::Happy),
                (Speaker::Popo, |s| s.camp_2, Face::Normal),
                (Speaker::Bori, |s| s.camp_3, Face::Normal),
            ],
        )
    } else {
        talk(w, &[(Speaker::Bori, |s| s.camp_again, Face::Happy)])
    }
}

// ---- 스크립트 ---------------------------------------------------------------

/// 오프닝: Naïl이 첫 방에 들어서고, 물든 이웃(모모, 아직 이름 모름)이 길을 막는다.
pub fn opening(w: &World) -> Cutscene {
    let neighbor = Color::new(0.45, 0.35, 0.50, 1.0);
    let mut nail = nail_actor(w);
    nail.facing = vec2(1.0, 0.0);
    let actors = vec![
        nail,
        Actor {
            name: "???",
            sprite: Some(Sprite::Critter(crate::art::MOMO_ROW)),
            color: neighbor,
            portrait: Some(Portrait::MOMO_TAINTED),
            pos: vec2(280.0, 90.0),
            facing: vec2(-1.0, 0.0),
            visible: false,
        },
        popo_actor(vec2(150.0, 150.0), false),
        Actor {
            portrait: Some(Portrait::MOM),
            ..story_actor(i18n::s().name_mom, Color::new(0.48, 0.34, 0.30, 1.0))
        },
    ];
    let mut steps = vec![
        // 일러스트(0037·0039): 시드는 큰 나무 → 땅속의 보라 실 → 어미가 Naïl을 굴 밖으로.
        Step::Illust(Some(Illust::GreatTree)),
        Step::Fade {
            to_clear: true,
            secs: 1.0,
        },
        Step::Narrate(|s| s.op_n1),
        Step::Narrate(|s| s.op_n2),
    ];
    steps.extend(cut_to(Some(Illust::MotherBurrow)));
    steps.extend([
        Step::Narrate(|s| s.op_n3),
        say(3, Face::Normal, |s| s.op_mom_1),
        say(0, Face::Determined, |s| s.op_mom_2),
    ]);
    steps.extend(cut_to(None));
    steps.extend([
        Step::Move {
            actor: 0,
            to: vec2(110.0, 90.0),
            speed: 60.0,
        },
        Step::Wait(0.3),
        say(0, Face::Worried, |s| s.op_1),
        // 건너뛰기는 여기서 멈춘다: 일러스트·내레이션을 넘겨도 모모의 등장은 본다(한 번 더 건너뛰면 그것도 넘긴다).
        Step::Checkpoint,
        // 모모의 첫 등장(0159 → 0164, 사장 "아직도 임팩트 없지? 궁금해하는 게 없나"): 음악이 끊기고 시든 꽃잎이
        // 날아들며 대낮에 어둠이 Naïl 둘레만 남긴다. 어둠 속에 보라 눈빛 두 점 → 모모가 걸어 나와 빛 가장자리에 선다
        // → '???' 카드 → 순간에 덮쳐 한 방(피해 없음, Naïl이 밀려난다) → 스스로 멈춘다("…아니야.") → Naïl을
        // 다른 누군가로 착각하고 이상한 한마디만 남긴다 → 보라 울림·꽃잎과 함께 사라지고 자리에 시든 꽃잎 하나 →
        // 1초의 침묵 → 어둠이 걷힌다 → Naïl "지금… 뭐라고 한 거지?" → 포포가 뛰어오다 굳는다.
        Step::Hush(true),
        Step::Cue {
            cue: Cue::Petals(false),
            at: vec2(230.0, 80.0),
        },
        Step::Wait(0.5),
        Step::Shake(0.15),
        Step::Gloom {
            to: 0.8,
            actor: 0,
            radius: 84.0,
            secs: 1.0,
        },
        Step::Wait(0.3),
        Step::Cue {
            cue: Cue::Eyes,
            at: vec2(262.0, 90.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(1.0, 0.0),
        },
        Step::Wait(1.2),
        Step::Show {
            actor: 1,
            at: vec2(290.0, 90.0),
        },
        Step::Move {
            actor: 1,
            to: vec2(188.0, 90.0),
            speed: 40.0,
        },
        Step::Cue {
            cue: Cue::Rise,
            at: vec2(188.0, 90.0),
        },
        Step::Wait(0.5),
        Step::Card {
            name: "???",
            sub: |s| s.mob_momo,
            secs: 3.0,
        },
        Step::Wait(1.0),
        // 덮친다: 한 번에 Naïl 앞까지, Naïl은 밀려난다.
        Step::Move {
            actor: 1,
            to: vec2(128.0, 90.0),
            speed: 320.0,
        },
        Step::Cue {
            cue: Cue::Rise,
            at: vec2(118.0, 90.0),
        },
        Step::Shake(0.35),
        Step::Move {
            actor: 0,
            to: vec2(92.0, 90.0),
            speed: 220.0,
        },
        Step::Wait(0.6),
        say(1, Face::Worried, |s| s.op_stop),
        say(1, Face::Normal, |s| s.op_2),
        say(1, Face::Worried, |s| s.op_3),
        Step::Cue {
            cue: Cue::Rise,
            at: vec2(128.0, 90.0),
        },
        Step::Cue {
            cue: Cue::Petals(false),
            at: vec2(128.0, 90.0),
        },
        Step::Shake(0.4),
        Step::Hide { actor: 1 },
        Step::Cue {
            cue: Cue::Leave,
            at: vec2(136.0, 96.0),
        },
        // 하나 더, 보리 공터 앞 길에(0190): 걸어 내려가다 밟으면 "이 길로 갔다".
        Step::Cue {
            cue: Cue::Trail,
            at: crate::field::petal_road(),
        },
        // 침묵: 꽃잎만 떨어진다.
        Step::Wait(1.0),
        Step::Gloom {
            to: 0.0,
            actor: 0,
            radius: 84.0,
            secs: 0.9,
        },
        say(0, Face::Worried, |s| s.op_4),
        Step::Wait(0.4),
        Step::Hush(false),
        // 라쿤 포포가 뛰어오다 모모가 선 자리를 보고 굳는다 → 그다음에야 안내.
        Step::Show {
            actor: 2,
            at: vec2(150.0, 150.0),
        },
        Step::Move {
            actor: 2,
            to: vec2(120.0, 104.0),
            speed: 70.0,
        },
        Step::Face {
            actor: 2,
            dir: vec2(1.0, 0.0),
        },
        Step::Wait(0.5),
        say(2, Face::Worried, |s| s.popo_hi0),
        Step::Face {
            actor: 2,
            dir: vec2(-1.0, 0.0),
        },
        Step::Face {
            actor: 0,
            dir: vec2(1.0, 0.0),
        },
        say(2, Face::Happy, |s| s.popo_hi),
        say(2, Face::Normal, |s| s.popo_hi2),
    ]);
    Cutscene::new(actors, steps)
}

/// 장면 전환: 어두워졌다가 그림을 바꾸고(`None`이면 월드로) 밝아진다.
fn cut_to(il: Option<Illust>) -> [Step; 3] {
    [
        Step::Fade {
            to_clear: false,
            secs: 0.5,
        },
        Step::Illust(il),
        Step::Fade {
            to_clear: true,
            secs: 0.6,
        },
    ]
}

/// 월드에 그림이 없는 배우의 바탕(이름 + 색). 초상화는 부르는 쪽이 얹는다(엄마·모모·할매, 0049).
fn story_actor(name: &'static str, color: Color) -> Actor {
    Actor {
        name,
        sprite: None,
        color,
        portrait: None,
        pos: Vec2::ZERO,
        facing: vec2(1.0, 0.0),
        visible: false,
    }
}

/// 모모·엔딩 장면의 배우: 0 Naïl, 1 모모, 2 누룩 할매.
fn story_actors(w: &World) -> Vec<Actor> {
    vec![nail_actor(w), momo_actor(), halmae_actor()]
}

/// 모모가 돌아온다: 세 번째 승리 뒤(0037). 꽃이 피고 이름을 말한다.
fn momo_steps() -> Vec<Step> {
    let mut v = vec![
        Step::Illust(Some(Illust::MomoBloom)),
        Step::Fade {
            to_clear: true,
            secs: 0.8,
        },
        Step::Narrate(|s| s.momo_r1),
        say(1, Face::Normal, |s| s.momo_r2),
        say(0, Face::Worried, |s| s.momo_r3),
        say(1, Face::Normal, |s| s.momo_r4),
        say(1, Face::Happy, |s| s.momo_r5),
        say(0, Face::Happy, |s| s.momo_r6),
        say(1, Face::Happy, |s| s.momo_r7),
    ];
    v.extend(cut_to(None));
    v
}

/// 반쪽 엔딩: 가장 깊은 곳에 처음 닿았다. 할매는 실을 거두지만 놓지 않는다.
/// 가장 깊은 곳에 들어섰다(0162): 이름 카드 + 할매의 말. 그 뒤 실매듭 끊기(월드). 반쪽 엔딩의 첫 두 줄이 여기로 왔다.
pub fn halmae_meet(w: &World) -> Cutscene {
    let steps = vec![
        Step::Wait(0.4),
        Step::Card {
            name: i18n::s().name_halmae,
            sub: |s| s.deep_sub,
            secs: 3.2,
        },
        Step::Narrate(|s| s.half_1),
        say(2, Face::Normal, |s| s.half_2),
        say(0, Face::Determined, |s| s.deep_1),
    ];
    Cutscene::new(story_actors(w), steps)
}

/// 실에 잠긴다(0163): 세 번 감긴 뒤. 음악이 끊기고 보라 어둠이 조여 오며 실이 휘감는다 → 한 줄 → 검게.
/// 끝나면 앱이 맡겨 둔 쉼터로 되살린다(`AfterCut::Respawn`), 되살아날 때 어둠·침묵을 걷는다.
pub fn deep_sleep(w: &World) -> Cutscene {
    let at = w.player.pos;
    let steps = vec![
        Step::Hush(true),
        Step::Gloom {
            to: 0.9,
            actor: 0,
            radius: 46.0,
            secs: 0.9,
        },
        Step::Cue { cue: Cue::Wrap, at },
        Step::Shake(0.2),
        Step::Wait(0.5),
        Step::Gloom {
            to: 1.0,
            actor: 0,
            radius: 14.0,
            secs: 1.1,
        },
        Step::Wait(0.3),
        Step::Narrate(|s| s.deep_sleep),
        Step::Fade {
            to_clear: false,
            secs: 0.8,
        },
    ];
    Cutscene::new(vec![nail_actor(w)], steps)
}

fn half_ending_steps() -> Vec<Step> {
    vec![
        Step::Illust(Some(Illust::HalmaeDeep)),
        Step::Fade {
            to_clear: true,
            secs: 1.0,
        },
        // 첫 두 줄(장소·"왔구나")은 들어설 때 했다(`halmae_meet`, 0162).
        say(0, Face::Worried, |s| s.half_3),
        say(2, Face::Normal, |s| s.half_4),
        Step::Narrate(|s| s.half_5),
        Step::Fade {
            to_clear: false,
            secs: 1.0,
        },
    ]
}

/// 참 엔딩 + 에필로그: 씨앗을 심고, 큰 나무의 마지막 꽃, 흰 실과 새싹, 새 두더지 둔덕.
fn true_ending_steps(family: bool) -> Vec<Step> {
    let mut v = vec![
        Step::Illust(Some(Illust::HalmaeDeep)),
        Step::Fade {
            to_clear: true,
            secs: 1.0,
        },
        Step::Narrate(|s| s.true_1),
    ];
    v.extend(cut_to(Some(Illust::LastBloom)));
    v.extend([
        Step::Narrate(|s| s.true_2),
        say(2, Face::Normal, |s| s.true_3),
    ]);
    v.extend(cut_to(Some(Illust::Sprout)));
    v.push(Step::Narrate(|s| s.true_4));
    v.extend(cut_to(Some(Illust::Molehill)));
    v.extend([Step::Narrate(|s| s.epi_1), Step::Narrate(|s| s.epi_2)]);
    if family {
        // 토끼풀 가족이 돌아왔으면(0165) 한 줄 더.
        v.push(Step::Narrate(|s| s.epi_clover));
    }
    v.push(Step::Fade {
        to_clear: false,
        secs: 1.5,
    });
    v
}

/// 모모가 돌아오는 장면: 세 번째 승리 뒤(0046).
pub fn momo_returns(w: &World, at: Vec2) -> Cutscene {
    // 쓰러진 자리에서 실이 풀리고 분홍 꽃잎이 흩날린 뒤(0159) 꽃이 피는 일러스트로.
    let mut steps = unbind_steps(at, 0, true);
    steps.push(Step::Fade {
        to_clear: false,
        secs: 0.5,
    });
    steps.extend(momo_steps());
    Cutscene::new(story_actors(w), steps)
}

/// 반쪽 엔딩: 마지막 길의 주인을 이기고 처음(또는 참 엔딩 조건 없이) 가장 깊은 곳에 닿았을 때(0042).
pub fn ending_half(w: &World) -> Cutscene {
    Cutscene::new(story_actors(w), half_ending_steps())
}

/// 참 엔딩 + 에필로그: 참 엔딩 조건(모모·도토)이 다 섰을 때.
pub fn ending_true(w: &World) -> Cutscene {
    Cutscene::new(story_actors(w), true_ending_steps(w.story.clover_back()))
}

/// 모모로 가장 깊은 곳에 닿았다(0056): 흰 실 속의 굴토끼 털과 시든 꽃잎. 모모는 꽃잎을 마을로 가져간다.
pub fn ending_momo(w: &World) -> Cutscene {
    let steps = vec![
        Step::Illust(Some(Illust::MomoDeep)),
        Step::Fade {
            to_clear: true,
            secs: 1.0,
        },
        Step::Narrate(|s| s.mend_1),
        say(0, Face::Worried, |s| s.mend_2),
        say(0, Face::Normal, |s| s.mend_3),
        Step::Narrate(|s| s.mend_4),
        Step::Fade {
            to_clear: false,
            secs: 1.0,
        },
    ];
    Cutscene::new(vec![nail_actor(w)], steps)
}

/// 도토로 가장 깊은 곳(0201): 오래전에 묻은 씨앗을 흰 실이 안고 있다. 도토는 곁에 표시를 묻고 돌아간다.
/// 씨앗을 할매 곁에 묻는 것은 나일의 참 엔딩(true_1)이라 여기서는 하지 않는다(0056: 다른 캐릭터의 끝은 대신하지 않는다).
/// 그림은 도토의 기억(같은 자리, 오래전) — 새 일러스트가 오면 바꾼다(0191 C).
pub fn ending_doto(w: &World) -> Cutscene {
    let steps = vec![
        Step::Illust(Some(Illust::DotoMemory)),
        Step::Fade {
            to_clear: true,
            secs: 1.0,
        },
        Step::Narrate(|s| s.dend_1),
        say(0, Face::Worried, |s| s.dend_2),
        say(0, Face::Normal, |s| s.dend_3),
        Step::Narrate(|s| s.dend_4),
        Step::Fade {
            to_clear: false,
            secs: 1.0,
        },
    ];
    Cutscene::new(vec![nail_actor(w)], steps)
}

/// 개발: 아직 게임에 자리가 없는 이야기 장면을 이어서(모모 → 반쪽 엔딩 → 참 엔딩·에필로그).
pub fn story_preview(w: &World) -> Cutscene {
    let mut steps = momo_steps();
    steps.extend(half_ending_steps());
    steps.extend(true_ending_steps(true));
    Cutscene::new(story_actors(w), steps)
}
