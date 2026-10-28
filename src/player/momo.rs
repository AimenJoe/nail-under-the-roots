//! 모모의 뛰기(0170 R9): 깡충(회피) → 이어 뛰기(특기)와 찍기 타격. 본문은 그대로 옮겼다.

use super::*;

impl Player {
    /// 모모의 깡충(회피).
    pub(super) fn tick_hop(&mut self, c: &Ctx<'_>, time: f32, dir: Vec2) -> Option<Strike> {
        let dt = c.dt;
        let mut strike = None;
        let time = time + dt;
        self.pos += dir * (self.talents.hop_dist() / t::HOP_TIME) * dt;
        if time >= t::HOP_TIME {
            self.events.push(Event::Land(false));
            // 착지: 재주 발 구르기면 밟기(피해 1), 아니면 밀치기만(0214).
            if self.talents.hop_stamp() {
                strike = Some(self.stamp_strike());
                self.events.push(Event::Stamp);
            } else {
                strike = Some(self.shove_strike());
                self.events.push(Event::Shove);
            }
            self.land_dodge(c);
        } else {
            self.state = State::Hop { time, dir };
        }
        strike
    }

    /// 모모의 이어 뛰기(특기): 세 번, 내려앉을 때마다 찍기.
    pub(super) fn tick_chain(
        &mut self,
        c: &Ctx<'_>,
        left: u32,
        time: f32,
        dir: Vec2,
    ) -> Option<Strike> {
        let dt = c.dt;
        let move_dir = c.move_dir;
        let mut strike = None;
        let time = time + dt;
        self.pos += dir * (t::CHAIN_HOP_DIST / t::CHAIN_HOP_TIME) * dt;
        if time >= t::CHAIN_HOP_TIME {
            strike = Some(self.stomp_strike());
            self.events.push(Event::Land(true));
            if left > 1 {
                // 다음 뛰기: 입력 쪽으로 다시 겨눈다(없으면 그대로).
                let dir = move_dir.normalize_or(dir);
                self.facing = dir;
                self.events.push(Event::Hop);
                self.state = State::Chain {
                    left: left - 1,
                    time: 0.0,
                    dir,
                };
            } else {
                self.state = State::Recover {
                    time: t::CHAIN_RECOVER,
                };
            }
        } else {
            self.state = State::Chain { left, time, dir };
        }
        strike
    }

    pub(super) fn start_hop(&mut self, move_dir: Vec2) {
        let dir = move_dir.normalize_or(self.facing);
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.facing = dir;
        self.state = State::Hop { time: 0.0, dir };
        self.events.push(Event::Hop);
    }

    pub(super) fn start_chain(&mut self, move_dir: Vec2) {
        let dir = move_dir.normalize_or(self.facing);
        self.sp = 0.0;
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.facing = dir;
        self.state = State::Chain {
            left: self.talents.chain_hops(),
            time: 0.0,
            dir,
        };
        self.events.push(Event::Hop);
    }

    /// 이어 뛰기 밟기 반지름(재주 높이 뛰기 + 들꽃 화관, 0142). 월드의 고리도 이 값.
    pub fn chain_radius(&self) -> f32 {
        self.talents.chain_radius() + self.stats.chain_radius
    }

    pub(super) fn stomp_strike(&mut self) -> Strike {
        self.swing_id += 1;
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Stomp,
            shape: Shape::Circle {
                center: self.pos,
                r: self.chain_radius(),
            },
            damage: t::CHAIN_DAMAGE + self.stats.damage,
            knockback: t::CHAIN_KNOCKBACK,
            stun: self.talents.chain_stun(),
            hitstop: t::CHAIN_HITSTOP,
            shake: t::CHAIN_SHAKE,
            finisher: false,
            poison: 0.0,
        }
    }

    /// 발 구르기(0142): 깡충 착지의 작은 밟기. 이어 뛰기 밟기보다 작고 약하다.
    pub(super) fn stamp_strike(&mut self) -> Strike {
        Strike {
            damage: t::TAL_STAMP_DAMAGE + self.stats.damage,
            shape: Shape::Circle {
                center: self.pos,
                r: t::TAL_STAMP_RADIUS,
            },
            knockback: t::CHAIN_KNOCKBACK * 0.5,
            stun: t::CHAIN_STUN * 0.6,
            shake: t::CHAIN_SHAKE * 0.4,
            ..self.stomp_strike()
        }
    }
}
