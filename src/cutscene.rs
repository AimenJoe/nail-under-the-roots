//! JRPG식 컷씬: 배우가 걷고, 아래 대화창에 글자가 타자처럼 찍히고, 확인으로 넘긴다.
//!
//! 컷씬은 월드 위에서 재생된다(월드 로직은 멈춤). 배우 0번은 플레이어(월드의 위치를 직접 움직인다),
//! 나머지는 컷씬이 들고 있는 임시 배우다. 뒤로(Esc/B)로 건너뛰면 남은 이동·등장·페이드를 즉시 적용한다.
//! 스크립트는 `Step` 목록이다. 데이터 파일로 옮기는 것은 내용이 쌓인 뒤에.

use macroquad::prelude::*;

use crate::i18n::{self, Tr};
use crate::input::{Act, Actions};
use crate::world::World;

/// 글자가 찍히는 속도(초당 글자 수).
const CHARS_PER_SEC: f32 = 40.0;

/// 대화 표정. 초상화 시트(assets/art/portraits.png, tools/art/portraits.py)의 칸을 고른다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Normal,
    /// 아직 쓰는 대사가 없다(첫 사용처: 보상·재회 컷씬). 그림은 있다.
    #[allow(dead_code)]
    Happy,
    Worried,
    Determined,
}

/// 배우의 초상화: 시트의 첫 칸과 표정 칸 수(보통·기쁨·걱정·결의 순서, 없는 표정은 보통).
#[derive(Clone, Copy, Debug)]
pub struct Portrait {
    pub first: u32,
    pub faces: u32,
}

impl Portrait {
    /// Naïl: 칸 0~3(보통·기쁨·걱정·결의).
    pub const NAIL: Portrait = Portrait { first: 0, faces: 4 };
    /// 변해 버린 이웃: 칸 4(정체를 모름: 그림자 + 빛나는 눈).
    pub const NEIGHBOR_UNKNOWN: Portrait = Portrait { first: 4, faces: 1 };
    /// 라쿤 포포: 칸 5~6(보통·기쁨).
    pub const POPO: Portrait = Portrait { first: 5, faces: 2 };

    pub fn frame(self, face: Face) -> u32 {
        let i = match face {
            Face::Normal => 0,
            Face::Happy => 1,
            Face::Worried => 2,
            Face::Determined => 3,
        };
        self.first + if i < self.faces { i } else { 0 }
    }
}

#[derive(Clone)]
pub struct Actor {
    pub name: &'static str,
    /// 월드 그림: critters.png의 줄(없으면 색 동그라미).
    pub sprite: Option<u32>,
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
}

impl Cutscene {
    pub fn new(actors: Vec<Actor>, steps: Vec<Step>) -> Self {
        let fade = match steps.first() {
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
        let Some(step) = self.steps.get(self.idx).cloned() else {
            self.done = true;
            return;
        };
        self.t += dt;
        let advance = match step {
            Step::Say { text, arg, .. } => {
                let total = say_text(text, arg).chars().count() as f32;
                let complete = self.shown >= total;
                if a.confirm || a.attack || a.tap.is_some() {
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
            Step::Give(i) => {
                w.gifts.push(crate::world::Gift::Weapon(i));
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
                w.feel.add_trauma(amount);
                true
            }
        };
        if advance {
            self.idx += 1;
            self.t = 0.0;
            self.shown = 0.0;
            if self.idx >= self.steps.len() {
                self.done = true;
            }
        }
        w.freeze();
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
        // 끝내는 길이므로 남은 스텝을 빼내 적용한다.
        let rest = self.steps.split_off(self.idx);
        for step in rest {
            match step {
                Step::Move { actor, to, .. } => *self.actor_pos(w, actor) = to,
                Step::Face { actor, dir } => self.set_facing(w, actor, dir),
                Step::Show { actor, at } => {
                    self.actors[actor].pos = at;
                    self.actors[actor].visible = true;
                }
                Step::Hide { actor } => self.actors[actor].visible = false,
                Step::Give(i) => w.gifts.push(crate::world::Gift::Weapon(i)),
                Step::Fade { to_clear, .. } => self.fade = if to_clear { 0.0 } else { 1.0 },
                _ => {}
            }
        }
        self.done = true;
        w.freeze();
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
        Some(a) => i18n::fill(t, &[&crate::input::prompt(a)]),
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
        sprite: Some(crate::art::RACCOON_ROW),
        color: Color::new(0.55, 0.55, 0.60, 1.0),
        portrait: Some(Portrait::POPO),
        pos,
        facing: vec2(-1.0, 0.0),
        visible,
    }
}

fn nail_actor(w: &World) -> Actor {
    Actor {
        name: crate::equipment::Character::Mole.name(),
        sprite: None,
        color: crate::world::MOLE,
        portrait: Some(Portrait::NAIL),
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

// ---- 스크립트 ---------------------------------------------------------------

/// 오프닝: Naïl이 첫 방에 들어서고, 변해 버린 이웃이 말을 건다.
pub fn opening(w: &World) -> Cutscene {
    let neighbor = Color::new(0.60, 0.40, 0.85, 1.0);
    let mut nail = nail_actor(w);
    nail.facing = vec2(1.0, 0.0);
    let actors = vec![
        nail,
        Actor {
            name: "???",
            sprite: None,
            color: neighbor,
            portrait: Some(Portrait::NEIGHBOR_UNKNOWN),
            pos: vec2(280.0, 90.0),
            facing: vec2(-1.0, 0.0),
            visible: false,
        },
        popo_actor(vec2(150.0, 150.0), false),
    ];
    let steps = vec![
        Step::Fade {
            to_clear: true,
            secs: 0.8,
        },
        Step::Move {
            actor: 0,
            to: vec2(110.0, 90.0),
            speed: 60.0,
        },
        Step::Wait(0.3),
        say(0, Face::Worried, |s| s.op_1),
        Step::Show {
            actor: 1,
            at: vec2(290.0, 90.0),
        },
        Step::Move {
            actor: 1,
            to: vec2(200.0, 90.0),
            speed: 45.0,
        },
        Step::Face {
            actor: 0,
            dir: vec2(1.0, 0.0),
        },
        say(1, Face::Normal, |s| s.op_2),
        say(1, Face::Normal, |s| s.op_3),
        say(0, Face::Normal, |s| s.op_4),
        Step::Shake(0.4),
        Step::Hide { actor: 1 },
        Step::Wait(0.4),
        say(0, Face::Determined, |s| s.op_5),
        // 라쿤 포포가 뛰어와 돕겠다고 한다(튜토리얼 안내).
        Step::Show {
            actor: 2,
            at: vec2(150.0, 150.0),
        },
        Step::Move {
            actor: 2,
            to: vec2(140.0, 100.0),
            speed: 70.0,
        },
        Step::Face {
            actor: 0,
            dir: vec2(1.0, 0.0),
        },
        say(2, Face::Happy, |s| s.popo_hi),
        say(2, Face::Normal, |s| s.popo_hi2),
    ];
    Cutscene::new(actors, steps)
}
