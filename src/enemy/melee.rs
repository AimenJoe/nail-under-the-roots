//! 잡몹의 돌격 상태(0169 R5): 다가가기 → 예고 → 돌진 → 쉼, 경직. 본문은 `Enemy::update`에서 그대로 옮겼다.

use super::*;

impl Enemy {
    /// 경직: 끝나면 제자리 상태로.
    pub(super) fn tick_stunned(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let time = time - dt;
        if time <= 0.0 {
            self.state = self.idle_state();
        } else {
            self.state = State::Stunned { time };
        }
    }

    /// 잡몹의 다가가기: 거리 안이면 예고로.
    pub(super) fn tick_approach(&mut self, c: &mut Ctx<'_>) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dist = c.dist;
        let dir_to_player = c.dir_to_player;
        let sp = self.sp();
        if player_targetable {
            self.facing = dir_to_player;
            if dist < sp.trigger {
                if sp.behavior == Behavior::Darter {
                    self.chain = t::SHREW_BITES - 1;
                }
                self.state = State::Windup { time: 0.0 };
            } else {
                self.pos += dir_to_player * self.speed() * dt;
            }
        }
    }

    /// 예고: 끝나면 돌진.
    pub(super) fn tick_windup(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dir_to_player = c.dir_to_player;
        let events = &mut *c.events;
        let time = time + dt;
        if player_targetable {
            self.facing = dir_to_player;
        }
        if time >= self.windup() {
            self.state = State::Charge {
                time: 0.0,
                dir: self.facing,
            };
            events.push(Event::Charge);
        } else {
            self.state = State::Windup { time };
        }
    }

    /// 돌진: 벽·시간이 다하면 쉼(콩이는 튕김).
    pub(super) fn tick_charge(&mut self, c: &mut Ctx<'_>, time: f32, dir: Vec2) {
        let dt = c.dt;
        let player_pos = c.player_pos;
        let room = c.room;
        let events = &mut *c.events;
        let sp = self.sp();
        let time = time + dt;
        self.pos += dir * sp.dash_speed * dt;
        let r = self.radius();
        let dur = if sp.behavior == Behavior::Momo {
            t::MOMO_KICK_TIME
        } else {
            sp.dash_time
        };
        let before = self.pos;
        let hit_wall = room.resolve_circle(&mut self.pos, r);
        if hit_wall && sp.behavior == Behavior::Kongi && self.chain > 0 {
            // 콩이: 벽에 튕긴다(반사 + 플레이어 쪽으로 조금 꺾기). 다 튕기면 아래처럼 어질어질.
            self.chain -= 1;
            let n = (self.pos - before).normalize_or(-dir);
            let bounced = dir - 2.0 * dir.dot(n) * n;
            let aim = (player_pos - self.pos).normalize_or(bounced);
            let new_dir = (bounced * (1.0 - t::KONGI_BOUNCE_AIM) + aim * t::KONGI_BOUNCE_AIM)
                .normalize_or(bounced);
            self.facing = new_dir;
            events.push(Event::WallHit);
            self.state = State::Charge { time, dir: new_dir };
        } else if hit_wall {
            self.state = State::Rest { time: sp.wall_rest };
            self.flash = t::HIT_FLASH;
            events.push(Event::WallHit);
        } else if time >= dur
            && matches!(sp.behavior, Behavior::Darter | Behavior::Clover)
            && self.chain > 0
        {
            // 뒤쥐: 곧바로 다시 겨눠 한 번 더 문다(예고는 짧게). 토끼풀도 둘째 차기(0165).
            self.chain -= 1;
            let skip = if sp.behavior == Behavior::Clover {
                t::CLOVER_KICK_REAIM
            } else {
                t::SHREW_REAIM
            };
            self.state = State::Windup {
                time: self.windup() * (1.0 - skip),
            };
        } else if time >= dur {
            self.chain = 0;
            self.state = State::Rest { time: sp.rest };
        } else {
            self.state = State::Charge { time, dir };
        }
    }

    /// 쉼: 끝나면 제자리 상태로.
    pub(super) fn tick_rest(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let sp = self.sp();
        let time = time - dt;
        self.state = if time <= 0.0 {
            idle_of(sp.behavior)
        } else {
            State::Rest { time }
        };
    }
}
