//! 굴파기·필살기(0170 R9): 굴파기 → 솟구치기, 드릴과 그 시작 함수들. 본문은 그대로 옮겼다.

use super::*;

impl Player {
    /// 굴파기(0038): 누르고 있으면 더 모으고, 떼면 솟구친다.
    pub(super) fn tick_burrow(&mut self, c: &Ctx<'_>, was: f32) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        let mut strike = None;
        let time = was + dt;
        // 정한 시간까지는 빠르게 파고 가고(대시), 힘을 모으는 동안은 느리다(0160).
        let base = self.talents.burrow_time();
        let factor = if was < base {
            t::BURROW_DASH_FACTOR
        } else {
            t::BURROW_CHARGE_FACTOR
        };
        self.pos += move_dir * self.speed() * factor * dt;
        self.state = State::Burrow { time };
        let early = (self.dodge_buf > 0.0 && time > 0.05) || self.attack_buf > 0.0;
        // 톡 누르면 정한 시간, 누르고 있으면 BURROW_HOLD_EXTRA까지 더 머문다(떼면 솟구침, 0101).
        // 더 머무는 동안은 힘을 모은다(0158): 정한 시간을 넘기는 틱에 땅이 울기 시작.
        if a.dodge_held && was < base && time >= base {
            self.events.push(Event::Rumble);
        }
        let limit = if a.dodge_held {
            base + t::BURROW_HOLD_EXTRA
        } else {
            base
        };
        if time >= limit || early {
            strike = Some(self.start_emerge((time - base).max(0.0)));
        }
        strike
    }

    /// 솟구치기.
    pub(super) fn tick_emerge(&mut self, c: &Ctx<'_>, time: f32, charge: f32) -> Option<Strike> {
        let dt = c.dt;
        let time = time + dt;
        self.state = if time >= t::EMERGE_TIME {
            State::Free
        } else {
            State::Emerge { time, charge }
        };
        None
    }

    /// 필살기 드릴: 화면 길이만큼 돌진, 조금만 조향.
    pub(super) fn tick_drill(
        &mut self,
        c: &Ctx<'_>,
        time: f32,
        dir: Vec2,
        aim: Vec2,
        next_hit: f32,
    ) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        let mut strike = None;
        let time = time + dt;
        let mut dir = dir;
        let mut aim = aim;
        let mut next_hit = next_hit;
        if time < t::DRILL_REV {
            // 시동: 제자리. 방향은 입력대로 바로, 그 둘레(콘) 안의 적이 있으면 그 적 쪽으로(0139).
            let want = if a.aim != Vec2::ZERO { a.aim } else { move_dir };
            if want != Vec2::ZERO {
                aim = want.normalize();
            }
            self.drill_lock = drill_assist(self.pos, aim, &self.targets);
            dir = self
                .drill_lock
                .map_or(aim, |q| (q - self.pos).try_normalize().unwrap_or(aim));
        } else {
            self.drill_lock = None;
            if time - dt < t::DRILL_REV {
                self.events.push(Event::Drill);
            }
            // 직진성: 입력 쪽으로 초당 DRILL_TURN 라디안만 튼다.
            if move_dir != Vec2::ZERO {
                let want = move_dir.normalize();
                let ang = dir.angle_between(want);
                let step = ang.clamp(-t::DRILL_TURN * dt, t::DRILL_TURN * dt);
                dir = Vec2::from_angle(step).rotate(dir).normalize();
            }
            self.pos += dir * drill_speed(time - t::DRILL_REV) * dt;
            next_hit -= dt;
        }
        self.facing = dir;
        if time >= t::DRILL_REV && next_hit <= 0.0 {
            next_hit += t::DRILL_HIT_EVERY;
            self.swing_id += 1;
            strike = Some(Strike {
                id: self.swing_id,
                kind: StrikeKind::Drill,
                shape: Shape::Circle {
                    center: self.pos + dir * 6.0,
                    r: t::DRILL_RADIUS,
                },
                damage: t::DRILL_DAMAGE + self.stats.damage,
                knockback: t::DRILL_KNOCKBACK,
                stun: t::DRILL_STUN,
                hitstop: t::DRILL_HITSTOP,
                shake: t::DRILL_HIT_SHAKE,
                finisher: false,
                poison: 0.0,
                crush: false,
            });
        }
        self.state =
            if time >= t::DRILL_REV + t::DRILL_TIME * self.talents.drill_mul() * self.stats.drill {
                State::Recover {
                    time: t::DRILL_RECOVER,
                }
            } else {
                State::Drill {
                    time,
                    dir,
                    aim,
                    next_hit,
                }
            };
        strike
    }

    pub(super) fn start_drill(&mut self, move_dir: Vec2) {
        let dir = if move_dir != Vec2::ZERO {
            move_dir.normalize()
        } else {
            self.facing
        };
        self.sp = 0.0;
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.drill_lock = drill_assist(self.pos, dir, &self.targets);
        self.state = State::Drill {
            time: 0.0,
            dir: self
                .drill_lock
                .map_or(dir, |q| (q - self.pos).try_normalize().unwrap_or(dir)),
            aim: dir,
            next_hit: 0.0,
        };
        self.events.push(Event::DrillRev);
    }

    /// 필살기 시동 중(제자리에서 힘 모으기). 월드는 이 동안 적을 멈춘다.
    pub fn drill_revving(&self) -> bool {
        matches!(self.state, State::Drill { time, .. } if time < t::DRILL_REV)
    }

    pub(super) fn start_burrow(&mut self) {
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.vel = Vec2::ZERO;
        self.state = State::Burrow { time: 0.0 };
        self.events.push(Event::Burrow);
    }

    /// `held`: 정한 시간보다 더 머문 초. 그만큼 대기가 늘어 땅속에 숨어 지내지는 못한다(0101).
    /// 그 시간에 모은 힘(0~1)만큼 솟구침이 커진다(0158): 톡은 작게 톡, 끝까지 누르면 크게 쾅.
    pub(super) fn start_emerge(&mut self, held: f32) -> Strike {
        let charge = Self::charge_of(held);
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.burrow_cd = self.burrow_cd_full() + held * t::BURROW_HOLD_CD;
        self.no_attack = t::EMERGE_NO_ATTACK;
        self.swing_id += 1;
        self.state = State::Emerge { time: 0.0, charge };
        self.events.push(Event::Emerge);
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Emerge,
            shape: Shape::Circle {
                center: self.pos,
                r: self.emerge_radius_for(charge),
            },
            damage: t::EMERGE_DAMAGE + (t::EMERGE_CHARGE_DAMAGE as f32 * charge).round() as i32,
            knockback: t::EMERGE_KNOCKBACK * (1.0 + (t::EMERGE_CHARGE_KNOCKBACK - 1.0) * charge),
            stun: t::EMERGE_STUN + t::EMERGE_CHARGE_STUN * charge,
            hitstop: t::EMERGE_HITSTOP + t::EMERGE_CHARGE_HITSTOP * charge,
            shake: t::EMERGE_SHAKE + t::EMERGE_CHARGE_SHAKE * charge,
            finisher: false,
            poison: 0.0,
            crush: false,
        }
    }
}
