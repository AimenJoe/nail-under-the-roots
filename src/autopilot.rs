//! 공식 페이지 클립 찍기 전용 자동 조종(`--features clip`, 0228 덧붙임). 월드를 보고 `Actions`를 만든다.
//! 전에는 정해 둔 키를 시간대로만 눌러 허공을 치고 벽으로 달렸다(사장 "너무 못하는데"). 배포 빌드에는 들어가지 않는다.
//! - 가장 가까운 적에게 다가가 박자(`BEAT_PERIOD`)마다 친다. 넷째 이음은 강공격(누르고 있다 떼기).
//! - 가까운 적의 예고가 다 차 가면 피한다(나일 = 굴파기로 숨었다 솟구치기, 모모·도토 = 회피).
//! - 모모는 멀면 달리다 떼어 멀리뛰기로 적 위에 내려앉고, 도토는 거리를 두고 던진다. 게이지가 차고 적이 둘 넘으면 필살기.
use macroquad::prelude::*;

use crate::equipment::Character;
use crate::input::Actions;
use crate::tuning as t;
use crate::world::World;

#[derive(Default)]
pub struct Pilot {
    /// 다음 타까지 남은 시간.
    swing: f32,
    combo: u32,
    /// 강공격 모으기로 공격을 누르고 있을 남은 시간.
    charge: f32,
    /// 회피(굴파기·달리기)를 누르고 있을 남은 시간.
    dodge: f32,
    running: bool,
}

impl Pilot {
    pub fn act(&mut self, w: &World, mut a: Actions, dt: f32) -> Actions {
        // 사람이 누르면 사람이 먼저.
        if a.move_dir != Vec2::ZERO || a.attack || a.dodge || a.special {
            return a;
        }
        self.swing -= dt;
        self.charge -= dt;
        self.dodge -= dt;
        let p = &w.player;
        let target = w
            .enemies
            .iter()
            .filter(|e| e.alive && e.hittable())
            .min_by(|x, y| x.pos.distance(p.pos).total_cmp(&y.pos.distance(p.pos)));
        let Some(e) = target else {
            self.running = false;
            return a;
        };
        let to = e.pos - p.pos;
        let d = to.length();
        let dir = to.normalize_or_zero();
        a.aim = dir;
        let threat = w.enemies.iter().any(|e| {
            e.alive
                && e.pos.distance(p.pos) < 64.0
                && (e.telegraph().is_some_and(|k| k > 0.55) || e.parryable())
        });
        let crowd = w.enemies.iter().filter(|e| e.alive).count();
        if p.sp_full() && crowd >= 2 && d < 90.0 {
            a.special = true;
            return a;
        }
        match w.character {
            Character::Mole => self.melee(&mut a, d, dir, threat, p.burrow_cd <= 0.0),
            Character::Momo => {
                // 멀면 달리다가 멀리뛰기 거리에서 뗀다.
                if self.running {
                    a.move_dir = dir;
                    if d <= t::JUMP_DIST {
                        self.running = false;
                    } else {
                        a.dodge_held = true;
                    }
                    return a;
                }
                if d > t::JUMP_DIST + 30.0 && !threat {
                    self.running = true;
                    a.dodge = true;
                    a.dodge_held = true;
                    a.move_dir = dir;
                    return a;
                }
                self.melee(&mut a, d, dir, threat, true);
            }
            Character::Doto => {
                // 던지는 거리 안(너무 가까우면 물러난다)에서 박자마다 던진다.
                if threat && self.dodge <= -0.8 {
                    a.dodge = true;
                    self.dodge = 0.0;
                } else if d < 44.0 {
                    // 물러나되 벽으로 몰리지 않게 방 가운데 쪽으로 비켜 선다.
                    let mid = (w.room.center() - p.pos).normalize_or_zero();
                    a.move_dir = (-dir + mid * 1.2).normalize_or_zero();
                } else if d > 96.0 {
                    a.move_dir = dir;
                }
                self.strike(&mut a, d < 110.0);
            }
        }
        a
    }

    /// 붙어서 치기(나일·모모). `burrow` = 굴파기·회피를 쓸 수 있다.
    fn melee(&mut self, a: &mut Actions, d: f32, dir: Vec2, threat: bool, burrow: bool) {
        if self.dodge > 0.0 {
            // 땅속(나일): 적 쪽으로 파고들다 떼면 솟구친다.
            a.dodge_held = true;
            a.move_dir = dir;
            return;
        }
        if threat && burrow && self.dodge <= -0.6 {
            a.dodge = true;
            a.dodge_held = true;
            self.dodge = 0.35;
            return;
        }
        if d > 20.0 {
            a.move_dir = dir;
        }
        self.strike(a, d < 26.0);
    }

    /// 박자마다 한 타, 넷째 이음은 강공격.
    fn strike(&mut self, a: &mut Actions, in_range: bool) {
        if self.charge > 0.0 {
            a.attack_held = true;
            return;
        }
        if !in_range || self.swing > 0.0 {
            return;
        }
        a.attack = true;
        a.attack_held = true;
        self.combo += 1;
        if self.combo.is_multiple_of(4) {
            // 평타 뒤 계속 누르고 있다 뗀다(0161) — 다 모이는 시간 + 조금.
            self.charge = t::HEAVY_CHARGE + 0.2;
            self.swing = t::HEAVY_CHARGE + 0.2 + t::BEAT_PERIOD;
        } else {
            self.swing = t::BEAT_PERIOD;
        }
    }
}
