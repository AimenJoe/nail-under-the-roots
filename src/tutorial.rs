//! 플레이로 배우는 튜토리얼(마리오식). 글은 한 줄 힌트뿐이고, 상황이 가르친다.
//!
//! 단계: 이동 → 허수아비를 3연타로 → 원거리형 앞에서 굴파기 → 솟구쳐서 처치 → 웨이브 시작.
//! 각 단계는 월드의 통계(이동 거리, 마지막 타 명중, 굴파기 횟수, 솟구침 처치)로 넘어간다.
//! 첫 시작(Scenario::Normal)과 개발 시나리오 "튜토리얼"에서만 돈다.

use macroquad::prelude::*;

use crate::enemy::Kind;
use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Move,
    Attack,
    Burrow,
    Emerge,
    Outro,
    Done,
}

pub struct Tutorial {
    pub stage: Stage,
    start_pos: Vec2,
    t: f32,
    burrows_at_start: u32,
}

impl Tutorial {
    pub fn new(w: &mut World) -> Self {
        w.auto_waves = false;
        w.enemies.clear();
        Self {
            stage: Stage::Move,
            start_pos: w.player.pos,
            t: 0.0,
            burrows_at_start: w.burrows,
        }
    }

    pub fn is_done(&self) -> bool {
        self.stage == Stage::Done
    }

    /// 지금 보여 줄 한 줄 힌트.
    pub fn hint(&self) -> Option<&'static str> {
        match self.stage {
            Stage::Move => Some("스틱 / WASD: 움직여 보자"),
            Stage::Attack => Some("공격(J · X · ATK)을 세 번: 세 번째가 묵직하다"),
            Stage::Burrow => Some("굴파기(K · A · DIG): 땅속에서는 맞지 않는다"),
            Stage::Emerge => Some("땅속에서 적 아래로 가서 다시 누르면 솟구친다"),
            Stage::Outro => Some("뿌리를 따라 내려가자"),
            Stage::Done => None,
        }
    }

    pub fn update(&mut self, w: &mut World, dt: f32) {
        self.t += dt;
        match self.stage {
            Stage::Move => {
                if (w.player.pos - self.start_pos).length() > 40.0 {
                    // 허수아비는 바로 앞에. 걷다 보면 부딪히는 거리.
                    let mut p = w.player.pos + w.player.facing * 56.0;
                    let r = w.room.inner;
                    p.x = p.x.clamp(r.x + 20.0, r.right() - 20.0);
                    p.y = p.y.clamp(r.y + 20.0, r.bottom() - 20.0);
                    w.spawn_enemy(Kind::Dummy, p);
                    self.enter(Stage::Attack);
                }
            }
            Stage::Attack => {
                if w.finisher_hits >= 1 {
                    // 허수아비는 역할을 다했다.
                    for e in w.enemies.iter_mut().filter(|e| e.kind == Kind::Dummy) {
                        e.alive = false;
                    }
                    let p = spot(w, 2);
                    w.spawn_enemy(Kind::Shooter, p);
                    self.burrows_at_start = w.burrows;
                    self.enter(Stage::Burrow);
                }
            }
            Stage::Burrow => {
                if w.burrows > self.burrows_at_start {
                    self.enter(Stage::Emerge);
                }
                if w.enemies.is_empty() {
                    // 발톱만으로 잡아 버렸다. 그래도 넘어간다.
                    self.enter(Stage::Outro);
                }
            }
            Stage::Emerge => {
                if w.emerge_kills >= 1 || w.enemies.is_empty() {
                    self.enter(Stage::Outro);
                }
            }
            Stage::Outro => {
                if self.t > 2.0 {
                    w.auto_waves = true;
                    w.wave = 0;
                    w.next_wave();
                    self.enter(Stage::Done);
                }
            }
            Stage::Done => {}
        }
    }

    fn enter(&mut self, s: Stage) {
        self.stage = s;
        self.t = 0.0;
    }
}

/// 플레이어에서 떨어진 스폰 지점.
fn spot(w: &World, i: usize) -> Vec2 {
    let far: Vec<Vec2> = w
        .room
        .spawn_points
        .iter()
        .copied()
        .filter(|p| (*p - w.player.pos).length() >= crate::tuning::SPAWN_MIN_DIST)
        .collect();
    far[i % far.len()]
}
