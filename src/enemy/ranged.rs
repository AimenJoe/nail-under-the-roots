//! 거리를 두는 적의 상태(0169 R5): 거리 두기 → 겨누기(던지기·혀·포자), 웅크려 뛰어오르기. 본문은 그대로 옮겼다.

use super::*;

impl Enemy {
    /// 거리 두기(던지기·뛰기·혀·포자): 멀면 다가가고 가까우면 물러나며, 때가 되면 겨눈다.
    pub(super) fn tick_keep(&mut self, c: &mut Ctx<'_>) {
        let dt = c.dt;
        let player_pos = c.player_pos;
        let player_targetable = c.targetable;
        let dist = c.dist;
        let dir_to_player = c.dir_to_player;
        let sp = self.sp();
        if player_targetable {
            self.facing = dir_to_player;
            let (near, far) = sp.keep;
            if dist < near {
                self.pos -= dir_to_player * self.speed() * dt;
            } else if dist > far {
                self.pos += dir_to_player * self.speed() * dt;
            } else if matches!(sp.behavior, Behavior::Leaper | Behavior::Swooper) {
                // 청설모·박쥐: 거리 안에서는 옆으로 돈다(가만히 있지 않는다).
                let side = vec2(-dir_to_player.y, dir_to_player.x);
                self.pos += side * self.speed() * 0.6 * dt;
            }
            // 실버섯은 가까이 와야 부푼다. 다른 것은 늘 센다.
            if sp.behavior != Behavior::Spore || dist < sp.trigger {
                self.fire_cd -= dt;
            }
            if self.fire_cd <= 0.0 {
                self.state = match sp.behavior {
                    Behavior::Leaper => State::Crouch {
                        time: 0.0,
                        target: player_pos,
                    },
                    Behavior::Swooper => State::Windup { time: 0.0 },
                    _ => State::Aim { time: 0.0 },
                };
            }
        }
    }

    /// 웅크리기: 예고 뒤 뛰어오른다.
    pub(super) fn tick_crouch(&mut self, c: &mut Ctx<'_>, time: f32, target: Vec2) {
        let dt = c.dt;
        let events = &mut *c.events;
        let sp = self.sp();
        let time = time + dt;
        self.facing = (target - self.pos).normalize_or(self.facing);
        if time >= self.windup() {
            // 너무 멀면 갈 수 있는 데까지만.
            let reach = (target - self.pos).clamp_length_max(sp.trigger * 1.2);
            self.state = State::Leap {
                time: 0.0,
                from: self.pos,
                to: self.pos + reach,
            };
            events.push(Event::Leap);
        } else {
            self.state = State::Crouch { time, target };
        }
    }

    /// 뛰어오르기: 내려앉으면 쉼.
    pub(super) fn tick_leap(&mut self, c: &mut Ctx<'_>, time: f32, from: Vec2, to: Vec2) {
        let dt = c.dt;
        let player_pos = c.player_pos;
        let player_targetable = c.targetable;
        let events = &mut *c.events;
        let sp = self.sp();
        let time = time + dt;
        let k = (time / sp.dash_time).min(1.0);
        self.pos = from + (to - from) * k;
        if time >= sp.dash_time {
            events.push(Event::Land);
            self.fire_cd = sp.fire_every;
            self.state = if self.chain > 0 && player_targetable {
                // 모모 세 번째: 내려앉자마자 다시 웅크린다(예고는 짧게).
                self.chain -= 1;
                State::Crouch {
                    time: self.windup() * t::MOMO_CHAIN_SKIP,
                    target: player_pos,
                }
            } else {
                self.chain = 0;
                State::Rest { time: sp.rest }
            };
        } else {
            self.state = State::Leap { time, from, to };
        }
    }

    /// 두꺼비의 혀 내밀기(0146).
    pub(super) fn tick_lash(&mut self, c: &mut Ctx<'_>, time: f32, dir: Vec2) {
        let dt = c.dt;
        let sp = self.sp();
        let time = time + dt;
        self.state = if time >= t::TOAD_LASH_TIME {
            State::Rest { time: sp.rest }
        } else {
            State::Lash { time, dir }
        };
    }

    /// 포자버섯의 뿜기(0146).
    pub(super) fn tick_puff(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let sp = self.sp();
        let time = time + dt;
        self.state = if time >= t::SPORE_PUFF_TIME {
            State::Rest { time: sp.rest }
        } else {
            State::Puff { time }
        };
    }

    /// 겨누기: 혀·포자·던지기(도토는 부채꼴).
    pub(super) fn tick_aim(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dir_to_player = c.dir_to_player;
        let events = &mut *c.events;
        let projectiles = &mut *c.projectiles;
        let sp = self.sp();
        let time = time + dt;
        if player_targetable {
            self.facing = dir_to_player;
        }
        if time >= self.windup() && sp.behavior == Behavior::Tongue {
            // 두꺼비: 겨눈 쪽으로 혀를 내민다(맞았는지는 월드가 이 틱에 본다).
            events.push(Event::Tongue);
            self.fire_cd = sp.fire_every;
            self.state = State::Lash {
                time: 0.0,
                dir: self.facing,
            };
        } else if time >= self.windup() && sp.behavior == Behavior::Spore {
            // 포자버섯: 제자리에 포자 구름(월드가 놓는다).
            events.push(Event::Spore);
            self.fire_cd = sp.fire_every;
            self.state = State::Puff { time: 0.0 };
        } else if time >= self.windup() {
            // 도토: 부채꼴로 여럿(가운데가 플레이어 쪽), 던진 뒤 잠깐 쉰다.
            let n = match sp.behavior {
                Behavior::Doto if self.hurt_half() => t::DOTO_FAN_HURT,
                Behavior::Doto => t::DOTO_FAN,
                _ => 1,
            };
            for i in 0..n {
                let a = (i as f32 - (n - 1) as f32 * 0.5) * t::DOTO_FAN_SPREAD;
                let dir = Vec2::from_angle(a).rotate(self.facing);
                let start = self.pos + dir * (self.radius() + t::PROJECTILE_RADIUS + 1.0);
                projectiles.push(Projectile {
                    pos: start,
                    prev_pos: start,
                    vel: dir * sp.shot_speed,
                    alive: true,
                    friendly: false,
                });
            }
            events.push(Event::Shoot);
            self.fire_cd = sp.fire_every;
            self.state = if sp.behavior == Behavior::Doto {
                State::Rest { time: sp.rest }
            } else {
                State::Keep
            };
        } else {
            self.state = State::Aim { time };
        }
    }
}
