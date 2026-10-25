//! JRPG식 컷씬: 배우가 걷고, 아래 대화창에 글자가 타자처럼 찍히고, 확인으로 넘긴다.
//!
//! 컷씬은 월드 위에서 재생된다(월드 로직은 멈춤). 배우 0번은 플레이어(월드의 위치를 직접 움직인다),
//! 나머지는 컷씬이 들고 있는 임시 배우다. 뒤로(Esc/B)로 건너뛰면 남은 이동·등장·페이드를 즉시 적용한다.
//! 스크립트는 `Step` 목록이다. 데이터 파일로 옮기는 것은 내용이 쌓인 뒤에.

use macroquad::prelude::*;

use crate::i18n::{self, Tr};
use crate::input::Actions;
use crate::world::World;

/// 글자가 찍히는 속도(초당 글자 수).
const CHARS_PER_SEC: f32 = 40.0;

#[derive(Clone)]
pub struct Actor {
    pub name: &'static str,
    pub color: Color,
    pub pos: Vec2,
    pub facing: Vec2,
    pub visible: bool,
}

#[derive(Clone)]
pub enum Step {
    /// 대화. 확인으로 넘긴다. 다 찍히기 전에 확인하면 전부 보여 준다.
    Say {
        actor: usize,
        text: Tr,
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
pub struct Dialogue<'a> {
    pub name: &'static str,
    pub color: Color,
    pub text: &'a str,
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
            Step::Say { text, .. } => {
                let total = i18n::tr(text).chars().count() as f32;
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
                w.gifts.push(i);
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
                Step::Give(i) => w.gifts.push(i),
                Step::Fade { to_clear, .. } => self.fade = if to_clear { 0.0 } else { 1.0 },
                _ => {}
            }
        }
        self.done = true;
        w.freeze();
    }

    /// 지금 보여 줄 대화창.
    pub fn dialogue(&self) -> Option<Dialogue<'_>> {
        match self.steps.get(self.idx)? {
            Step::Say { actor, text } => {
                let text = i18n::tr(*text);
                let total = text.chars().count();
                let shown = (self.shown as usize).min(total);
                let a = &self.actors[*actor];
                Some(Dialogue {
                    name: a.name,
                    color: a.color,
                    text,
                    shown_chars: shown,
                    complete: shown >= total,
                })
            }
            _ => None,
        }
    }
}

// ---- 스크립트 ---------------------------------------------------------------

/// 오프닝: Naïl이 첫 방에 들어서고, 변해 버린 이웃이 말을 건다.
pub fn opening(w: &World) -> Cutscene {
    let mole = crate::world::MOLE;
    let neighbor = crate::world::SHOOTER;
    let actors = vec![
        Actor {
            name: "Naïl",
            color: mole,
            pos: w.player.pos,
            facing: vec2(1.0, 0.0),
            visible: true,
        },
        Actor {
            name: "???",
            color: neighbor,
            pos: vec2(280.0, 90.0),
            facing: vec2(-1.0, 0.0),
            visible: false,
        },
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
        Step::Say {
            actor: 0,
            text: |s| s.op_1,
        },
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
        Step::Say {
            actor: 1,
            text: |s| s.op_2,
        },
        Step::Say {
            actor: 1,
            text: |s| s.op_3,
        },
        Step::Say {
            actor: 0,
            text: |s| s.op_4,
        },
        Step::Shake(0.4),
        Step::Hide { actor: 1 },
        Step::Wait(0.4),
        Step::Say {
            actor: 0,
            text: |s| s.op_5,
        },
    ];
    Cutscene::new(actors, steps)
}
