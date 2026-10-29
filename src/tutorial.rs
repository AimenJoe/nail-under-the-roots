//! 체계적인 튜토리얼: 라쿤 포포가 단계마다 짧게 말하고(대화창), 그 자리에서 해 본다(docs/decisions/0023).
//!
//! 단계: 이동 → 발톱 줍기 → 3연타 → 강공격(0161) → 굴파기 → 솟구치기 → 패링 → 필살기 → 아이템 → 메뉴·저장 안내.
//! 그 뒤(모험이면): 공터 문이 열리고 "남쪽 공터의 대장" 힌트가 대장을 이길 때까지 남는다(docs/decisions/0032).
//! 각 단계는 (1) 포포의 말(`take_talk`로 앱이 컷씬을 띄운다) (2) 상황 만들기(허수아비·연습 다람쥐·게이지·산딸기)
//! (3) 월드 통계로 판정(이동 거리, 마지막 타 명중, 굴파기·솟구침·패링·필살기·먹기 횟수).
//! 화면 아래 힌트는 "[3/10] X 세 번…"처럼 몇 번째인지와 지금 버튼 이름을 보인다. 연습 동안은 맞아도 체력이 줄지 않는다.
//! 패링은 어려우니 오래 못 하면 포포가 "괜찮아" 하고 넘어간다. 첫 시작(Scenario::Normal)과 개발 시나리오 "튜토리얼"에서만.

use macroquad::prelude::*;

use crate::cutscene::Face;
use crate::enemy::Kind;
use crate::equipment::Character;
use crate::i18n::{fill, Tr};
use crate::input::{self, Act};
use crate::items::Consumable;
use crate::world::{Npc, Pickup, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Move,
    /// 반짝이는 두더지 발톱을 줍는다. 줍기 전에는 공격이 안 나간다.
    Pickup,
    Attack,
    /// 강공격(0161): 누르고 있다 빛나면 떼기. 허수아비에 한 번 맞히면 된다.
    Heavy,
    Burrow,
    Emerge,
    Parry,
    Special,
    Item,
    Wrap,
    /// 튜토리얼 보스: 공터 문을 열고, 대장을 이길 때까지 힌트를 남긴다(번호 없음).
    Boss,
    Done,
}

/// 힌트에 보이는 단계 수(Done 제외).
const STEPS: [Stage; 10] = [
    Stage::Move,
    Stage::Pickup,
    Stage::Attack,
    Stage::Heavy,
    Stage::Burrow,
    Stage::Emerge,
    Stage::Parry,
    Stage::Special,
    Stage::Item,
    Stage::Wrap,
];

/// 패링을 이만큼 못 하면 넘어간다(초).
const PARRY_GIVE_UP: f32 = 30.0;

type Line = (Tr, Face, Option<Act>);

pub struct Tutorial {
    pub stage: Stage,
    start_pos: Vec2,
    t: f32,
    /// 단계에 들어설 때의 통계(그 뒤로 늘었나 본다).
    base: u32,
    /// 앱이 컷씬으로 띄울 포포의 말.
    talk: Vec<Line>,
}

impl Tutorial {
    pub fn new(w: &mut World) -> Self {
        w.auto_waves = false;
        w.no_damage = true;
        // 아레나는 비운다. 필드의 적은 멀리서 자고 있으니 둔다(깬 적만 튜토리얼 상대로 센다).
        if w.adv.place == crate::adventure::Place::Arena {
            w.enemies.clear();
        }
        Self {
            stage: Stage::Move,
            start_pos: w.player.pos,
            t: 0.0,
            base: 0,
            talk: Vec::new(),
        }
    }

    pub fn is_done(&self) -> bool {
        self.stage == Stage::Done
    }

    /// 포포가 할 말(있으면 앱이 대화 컷씬을 띄우고 비운다).
    pub fn take_talk(&mut self) -> Vec<Line> {
        std::mem::take(&mut self.talk)
    }

    /// 지금 보여 줄 힌트: "[n/9] …" + 버튼 이름.
    pub fn hint(&self) -> Option<String> {
        let (text, arg): (Tr, Option<Act>) = match self.stage {
            Stage::Move => (|s| s.tut_move, None),
            Stage::Pickup => (|s| s.tut_pickup, None),
            Stage::Attack => (|s| s.tut_attack, Some(Act::Attack)),
            Stage::Heavy => (|s| s.tut_heavy, Some(Act::Attack)),
            Stage::Burrow => (|s| s.tut_burrow, Some(Act::Dodge)),
            Stage::Emerge => (|s| s.tut_emerge, Some(Act::Dodge)),
            Stage::Parry => (|s| s.tut_parry, Some(Act::Attack)),
            Stage::Special => (|s| s.tut_special, Some(Act::Special)),
            Stage::Item => (|s| s.tut_item, Some(Act::UseItem)),
            Stage::Wrap => (|s| s.tut_outro, None),
            Stage::Boss => return Some(crate::i18n::s().tut_boss.to_string()),
            Stage::Done => return None,
        };
        let body = match arg {
            Some(a) => fill(crate::i18n::tr(text), &[&crate::ui::key(input::prompt(a))]),
            // 걷기: 키보드 WASD, 패드 스틱, 터치 왼쪽 끌기.
            None if self.stage == Stage::Move => fill(
                crate::i18n::tr(text),
                &[&crate::ui::key(input::verb(input::Verb::Walk))],
            ),
            None => crate::i18n::tr(text).to_string(),
        };
        let n = STEPS.iter().position(|s| *s == self.stage).unwrap_or(0) + 1;
        Some(fill(crate::i18n::s().tut_step, &[&n, &STEPS.len(), &body]))
    }

    pub fn update(&mut self, w: &mut World, dt: f32) {
        self.t += dt;
        // 포포: 오프닝 컷씬이 끝난 자리(Naïl 오른쪽 아래)에 나타나고, 그 뒤로는 플레이어 뒤쪽으로 따라온다.
        if w.npc.is_none() {
            let p = w.player.pos + vec2(30.0, 10.0);
            w.npc = Some(Npc {
                who: crate::world::Who::Popo,
                pos: p,
                prev_pos: p,
                face_left: true,
                target: None,
            });
        }
        if let Some(n) = w.npc.as_mut() {
            let side = if w.player.face_left { 1.0 } else { -1.0 };
            n.target = Some(w.player.pos + vec2(30.0 * side, -10.0));
        }
        match self.stage {
            Stage::Move => {
                if (w.player.pos - self.start_pos).length() > 40.0 {
                    if w.player.armed {
                        // 이미 무기가 있으면(개발 시나리오) 줍기를 건너뛴다.
                        self.enter_attack(w);
                    } else {
                        let p = ahead(w, 40.0);
                        w.pickups.push(Pickup::new(
                            p,
                            crate::world::Gift::Weapon(Character::Mole.starting_weapon()),
                        ));
                        self.say(&[(|s| s.popo_pickup, Face::Normal, None)]);
                        self.enter(Stage::Pickup, 0);
                    }
                }
            }
            Stage::Pickup => {
                if w.player.armed {
                    self.enter_attack(w);
                }
            }
            Stage::Attack => {
                // 허수아비(체력 6)를 연타로 부숴 버리면 칠 것이 없어 영영 막혔다(사장 2026-10-28, 0207).
                self.ensure(w, Kind::Dummy);
                if w.stats.finisher_hits > self.base {
                    // 허수아비는 강공격 연습까지 쓴다(0161).
                    self.say(&[(|s| s.popo_heavy, Face::Happy, Some(Act::Attack))]);
                    self.enter(Stage::Heavy, w.stats.heavy_hits);
                }
            }
            Stage::Heavy => {
                self.ensure(w, Kind::Dummy);
                if w.stats.heavy_hits > self.base {
                    // 허수아비는 역할을 다했다.
                    clear_kind(w, Kind::Dummy);
                    let p = spot(w, 2);
                    w.spawn_enemy(Kind::Chipmunk, p);
                    self.say(&[(|s| s.popo_burrow, Face::Normal, Some(Act::Dodge))]);
                    self.enter(Stage::Burrow, w.stats.burrows);
                }
            }
            Stage::Burrow => {
                if w.stats.burrows > self.base {
                    self.ensure(w, Kind::Chipmunk);
                    self.say(&[(|s| s.popo_emerge, Face::Normal, Some(Act::Dodge))]);
                    self.enter(Stage::Emerge, w.stats.emerge_hits);
                }
            }
            Stage::Emerge => {
                self.ensure(w, Kind::Chipmunk);
                if w.stats.emerge_hits > self.base {
                    self.say(&[(|s| s.popo_parry, Face::Happy, Some(Act::Attack))]);
                    self.enter(Stage::Parry, w.stats.parries);
                }
            }
            Stage::Parry => {
                self.ensure(w, Kind::Chipmunk);
                let done = w.stats.parries > self.base;
                if done || self.t > PARRY_GIVE_UP {
                    if !done {
                        self.say(&[(|s| s.popo_parry_skip, Face::Normal, None)]);
                    }
                    clear_kind(w, Kind::Chipmunk);
                    // 게이지를 채워 주고, 앞에 허수아비 둘(벽까지 밀어붙일 거리).
                    w.player.sp = crate::tuning::SP_MAX;
                    let a = ahead(w, 40.0);
                    let b = ahead(w, 60.0) + vec2(0.0, 8.0);
                    w.spawn_enemy(Kind::Dummy, a);
                    w.spawn_enemy(Kind::Dummy, b);
                    self.say(&[(|s| s.popo_special, Face::Happy, Some(Act::Special))]);
                    self.enter(Stage::Special, w.stats.drills);
                }
            }
            Stage::Special => {
                let finished = w.stats.drills > self.base
                    && !matches!(w.player.state, crate::player::State::Drill { .. });
                if finished {
                    clear_kind(w, Kind::Dummy);
                    if w.player.hp >= w.player.stats.max_hp {
                        w.player.hp = w.player.stats.max_hp - 1;
                    }
                    if w.inv.bag.count(Consumable::Berry) == 0 {
                        w.inv.bag.add(Consumable::Berry);
                    }
                    w.inv.bag.selected = Consumable::Berry;
                    self.say(&[(|s| s.popo_item, Face::Normal, Some(Act::UseItem))]);
                    self.enter(Stage::Item, w.stats.items_used);
                }
                // 필살기를 아직 안 썼는데 게이지가 비었으면(다른 데 써 버림) 다시 채운다.
                if w.stats.drills == self.base && w.player.sp < crate::tuning::SP_MAX {
                    w.player.sp = crate::tuning::SP_MAX;
                }
            }
            Stage::Item => {
                // 연습 동안은 맞지 않으니, 저장 지점(시작 자리 곁)·뿌리 심장으로 체력이 가득 차면 먹을 수 없어
                // 이 단계에서 영영 막혔다(사장 2026-10-26, 0093). 먹기 전까지 한 칸 비워 두고 산딸기를 둔다.
                if w.stats.items_used == self.base {
                    if w.player.hp >= w.player.stats.max_hp {
                        w.player.hp = w.player.stats.max_hp - 1;
                    }
                    if w.inv.bag.count(Consumable::Berry) == 0 {
                        w.inv.bag.add(Consumable::Berry);
                    }
                    if w.inv.bag.count(w.inv.bag.selected) == 0 {
                        w.inv.bag.selected = Consumable::Berry;
                    }
                }
                if w.stats.items_used > self.base {
                    self.say(&[
                        (|s| s.popo_item2, Face::Normal, None),
                        (|s| s.popo_menu, Face::Normal, Some(Act::Map)),
                        (|s| s.popo_save, Face::Normal, None),
                        (|s| s.popo_glade, Face::Normal, None),
                        (|s| s.popo_bye, Face::Happy, None),
                    ]);
                    self.enter(Stage::Wrap, 0);
                }
            }
            Stage::Wrap => {
                if self.t > 3.0 {
                    w.no_damage = false;
                    // 포포는 그 자리에 남는다(따라오지 않음).
                    if let Some(n) = w.npc.as_mut() {
                        n.target = None;
                    }
                    // 아레나(웨이브 시험장)면 웨이브 1로. 모험(필드)이면 공터 문을 열고 대장에게 간다.
                    if w.adv.place == crate::adventure::Place::Arena {
                        w.auto_waves = true;
                        w.wave = 0;
                        w.next_wave();
                        self.enter(Stage::Done, 0);
                    } else {
                        w.open_glade();
                        self.enter(Stage::Boss, 0);
                    }
                }
            }
            Stage::Boss => {
                if w.adv.boss == crate::adventure::BossFight::Done {
                    self.enter(Stage::Done, 0);
                }
            }
            Stage::Done => {}
        }
    }

    fn enter_attack(&mut self, w: &mut World) {
        spawn_dummy(w);
        self.say(&[(|s| s.popo_combo, Face::Happy, Some(Act::Attack))]);
        self.enter(Stage::Attack, w.stats.finisher_hits);
    }

    fn say(&mut self, lines: &[Line]) {
        self.talk.extend_from_slice(lines);
    }

    fn enter(&mut self, s: Stage, base: u32) {
        self.stage = s;
        self.t = 0.0;
        self.base = base;
    }

    /// 맞혀야 넘어가는 단계의 과녁(허수아비·연습 다람쥐)이 없으면(부쉈거나 잡아 버렸으면) 하나 더.
    /// 허수아비는 바로 앞에, 다람쥐는 떨어진 자리에.
    fn ensure(&self, w: &mut World, kind: Kind) {
        if w.enemies.iter().any(|e| e.kind == kind && e.alive) {
            return;
        }
        if kind == Kind::Dummy {
            spawn_dummy(w);
        } else {
            let p = spot(w, 1);
            w.spawn_enemy(kind, p);
        }
    }
}

fn clear_kind(w: &mut World, k: Kind) {
    for e in w.enemies.iter_mut().filter(|e| e.kind == k) {
        e.alive = false;
    }
}

/// 플레이어 앞 `dist` 지점(방 안으로 잘라서).
fn ahead(w: &World, dist: f32) -> Vec2 {
    let mut p = w.player.pos + w.player.facing * dist;
    // 필드는 지도 전체가 inner라 화면(카메라) 안으로 자른다.
    let r = if w.room.grid.is_some() {
        Rect::new(w.adv.cam.x + 16.0, w.adv.cam.y + 32.0, 288.0, 132.0)
    } else {
        w.room.inner
    };
    p.x = p.x.clamp(r.x + 20.0, r.right() - 20.0);
    p.y = p.y.clamp(r.y + 20.0, r.bottom() - 20.0);
    p
}

/// 허수아비는 바로 앞에. 걷다 보면 부딪히는 거리.
fn spawn_dummy(w: &mut World) {
    let p = ahead(w, 56.0);
    w.spawn_enemy(Kind::Dummy, p);
}

/// 플레이어에서 떨어진 스폰 지점(필드는 화면 안의 지점만).
fn spot(w: &World, i: usize) -> Vec2 {
    let view = Rect::new(w.adv.cam.x + 16.0, w.adv.cam.y + 24.0, 288.0, 140.0);
    let in_view = |p: &Vec2| w.room.grid.is_none() || view.contains(*p);
    let far: Vec<Vec2> = w
        .room
        .spawn_points
        .iter()
        .copied()
        .filter(|p| (*p - w.player.pos).length() >= crate::tuning::SPAWN_MIN_DIST && in_view(p))
        .collect();
    if far.is_empty() {
        return ahead(w, 80.0);
    }
    far[i % far.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 단계마다 포포가 말하고, 통계가 늘면 다음 단계로.
    #[test]
    fn lessons_advance_on_stats_and_popo_talks() {
        let mut w = World::empty();
        let mut tu = Tutorial::new(&mut w);
        assert!(w.no_damage);
        w.player.pos += vec2(50.0, 0.0);
        tu.update(&mut w, 0.1);
        assert!(w.npc.is_some(), "포포가 나타난다");
        // 개발 월드는 처음부터 무기가 있어 줍기를 건너뛴다.
        assert_eq!(tu.stage, Stage::Attack);
        assert!(!tu.take_talk().is_empty());
        w.stats.finisher_hits += 1;
        tu.update(&mut w, 0.1);
        assert_eq!(tu.stage, Stage::Heavy, "3연타 뒤 강공격(0161)");
        assert!(tu.hint().unwrap().starts_with("[4/10]"));
        w.stats.heavy_hits += 1;
        tu.update(&mut w, 0.1);
        assert_eq!(tu.stage, Stage::Burrow);
        w.stats.burrows += 1;
        tu.update(&mut w, 0.1);
        w.stats.emerge_hits += 1;
        tu.update(&mut w, 0.1);
        assert_eq!(tu.stage, Stage::Parry);
        // 패링을 못 해도 시간이 지나면 넘어간다.
        tu.update(&mut w, PARRY_GIVE_UP + 1.0);
        assert_eq!(tu.stage, Stage::Special);
        assert!(w.player.sp_full(), "필살기 단계는 게이지를 채워 준다");
        w.stats.drills += 1;
        tu.update(&mut w, 0.1);
        assert_eq!(tu.stage, Stage::Item);
        assert!(w.inv.bag.count(Consumable::Berry) > 0);
        // 저장 지점에서 체력이 가득 차고 산딸기를 잃어도 다시 먹을 수 있게 된다(0093).
        w.player.hp = w.player.stats.max_hp;
        w.inv.bag.take(Consumable::Berry);
        tu.update(&mut w, 0.1);
        assert!(
            w.player.hp < w.player.stats.max_hp,
            "먹을 수 있게 한 칸 비운다"
        );
        assert!(w.inv.bag.count(Consumable::Berry) > 0, "산딸기를 다시 둔다");
        w.stats.items_used += 1;
        tu.update(&mut w, 0.1);
        tu.update(&mut w, 3.5);
        assert!(tu.is_done());
        assert!(!w.no_damage);
        assert!(tu.hint().is_none());
    }

    /// 맞혀야 넘어가는 단계에서 과녁을 다 없애도 다시 나와 막히지 않는다(0207).
    #[test]
    fn hit_lessons_bring_back_a_target_when_all_are_gone() {
        let alive =
            |w: &World, k: Kind| w.enemies.iter().filter(|e| e.kind == k && e.alive).count();
        let mut w = World::empty();
        let mut tu = Tutorial::new(&mut w);
        w.player.pos += vec2(50.0, 0.0);
        tu.update(&mut w, 0.1);
        for (stage, kind) in [
            (Stage::Attack, Kind::Dummy),
            (Stage::Heavy, Kind::Dummy),
            (Stage::Emerge, Kind::Chipmunk),
            (Stage::Parry, Kind::Chipmunk),
        ] {
            let base = match stage {
                Stage::Attack => w.stats.finisher_hits,
                Stage::Heavy => w.stats.heavy_hits,
                Stage::Emerge => w.stats.emerge_hits,
                _ => w.stats.parries,
            };
            tu.enter(stage, base);
            for e in w.enemies.iter_mut() {
                e.alive = false;
            }
            tu.update(&mut w, 0.1);
            assert_eq!(tu.stage, stage, "{stage:?}: 아직 넘어가지 않는다");
            assert_eq!(alive(&w, kind), 1, "{stage:?}: 과녁 {kind:?}가 다시 나온다");
        }
    }
}
