//! 보스의 상태와 다음 수(0169 R5): 대장(울부짖기)·모모(땅속·튀어나오기)·도토·콩이(버티기)·토끼풀(발 구르기·품기), 다음 수 고르기 `*_next`. 본문은 그대로 옮겼다.

use super::*;

impl Enemy {
    /// 보스의 다가가기: 다음 수를 고르고(`*_next`), 걸어갈 때는 종마다 다르게.
    pub(super) fn tick_boss_approach(&mut self, c: &mut Ctx<'_>) {
        let dt = c.dt;
        let player_pos = c.player_pos;
        let player_targetable = c.targetable;
        let dist = c.dist;
        let dir_to_player = c.dir_to_player;
        let events = &mut *c.events;
        let sp = self.sp();
        if player_targetable {
            self.facing = dir_to_player;
            self.state = match sp.behavior {
                Behavior::Chief => self.chief_next(dist, player_pos),
                Behavior::Momo => self.momo_next(dist, player_pos, events),
                Behavior::Kongi => self.kongi_next(dist),
                Behavior::Clover => self.clover_next(dist, player_pos),
                _ => self.doto_next(dist, player_pos),
            };
            if matches!(self.state, State::Approach) {
                if sp.behavior == Behavior::Clover {
                    // 토끼풀(0165): 플레이어가 아니라 "고치 앞 자리"로 간다.
                    let to = self.clover_post(player_pos) - self.pos;
                    if to.length() > 3.0 {
                        self.pos += to.normalize() * self.speed() * dt;
                    }
                } else {
                    // 도토는 너무 가까우면 물러난다(던지는 보스).
                    let away = sp.behavior == Behavior::Doto && dist < sp.keep.0;
                    let dir = if away { -dir_to_player } else { dir_to_player };
                    self.pos += dir * self.speed() * dt;
                }
            }
        }
    }

    /// 토끼풀의 발 구르기(0165).
    pub(super) fn tick_thump(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dir_to_player = c.dir_to_player;
        let events = &mut *c.events;
        let sp = self.sp();
        let time = time + dt;
        if player_targetable {
            self.facing = dir_to_player;
        }
        if time >= self.thump_windup() {
            events.push(Event::Thump(self.pos));
            self.state = State::Rest { time: sp.rest };
        } else {
            self.state = State::Thump { time };
        }
    }

    /// 토끼풀의 품기(0165).
    pub(super) fn tick_gather(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dir_to_player = c.dir_to_player;
        let events = &mut *c.events;
        let sp = self.sp();
        let was = time;
        let time = time + dt;
        // 고치 쪽으로 가서 실을 감는다. 그동안 플레이어를 본다(실을 뻗을 쪽).
        if let Some(g) = self.guard {
            let to = g + vec2(0.0, 14.0) - self.pos;
            if to.length() > 3.0 {
                self.pos += to.normalize() * self.speed() * 1.5 * dt;
            }
        }
        if player_targetable {
            self.facing = dir_to_player;
        }
        if was < t::CLOVER_THREAD_AT && time >= t::CLOVER_THREAD_AT {
            events.push(Event::Thread(self.pos, self.facing));
        }
        self.state = if time >= t::CLOVER_GATHER_TIME {
            State::Rest {
                time: sp.rest * 0.5,
            }
        } else {
            State::Gather { time }
        };
    }

    /// 대장의 울부짖기(소환) / 도토의 묻기.
    pub(super) fn tick_howl(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let events = &mut *c.events;
        let sp = self.sp();
        let time = time + dt;
        let doto = sp.behavior == Behavior::Doto;
        let dur = self.windup()
            * if doto {
                t::DOTO_BURY_MUL
            } else {
                t::CHIEF_HOWL_MUL
            };
        if time >= dur {
            if doto {
                events.push(Event::Bury(if self.hurt_half() {
                    t::DOTO_MINES_HURT
                } else {
                    t::DOTO_MINES
                }));
            } else {
                self.summons += 1;
                events.push(Event::Summon);
            }
            self.state = State::Rest { time: sp.rest };
        } else {
            self.state = State::Howl { time };
        }
    }

    /// 모모의 땅속(0046).
    pub(super) fn tick_under(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let to_player = c.to_player;
        let events = &mut *c.events;
        let time = time + dt;
        if time < t::MOMO_DIG_TRACK {
            // 둔덕이 플레이어를 따라온다(걸음보다 느리게).
            if player_targetable {
                let step = t::MOMO_DIG_SPEED * dt;
                self.pos += to_player.clamp_length_max(step);
            }
            self.state = State::Under { time };
        } else if time >= t::MOMO_DIG_TRACK + self.windup() {
            events.push(Event::Pop);
            self.state = State::Pop { time: 0.0 };
        } else {
            self.state = State::Under { time };
        }
    }

    /// 콩이의 버티기(0054).
    pub(super) fn tick_guard(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let player_targetable = c.targetable;
        let dir_to_player = c.dir_to_player;
        let sp = self.sp();
        let time = time + dt;
        if player_targetable {
            self.facing = dir_to_player;
        }
        self.state = if time >= t::KONGI_GUARD_TIME {
            State::Rest {
                time: sp.rest * 0.5,
            }
        } else {
            State::Guard { time }
        };
    }

    /// 모모의 튀어나오기.
    pub(super) fn tick_pop(&mut self, c: &mut Ctx<'_>, time: f32) {
        let dt = c.dt;
        let sp = self.sp();
        let time = time + dt;
        self.state = if time >= t::MOMO_POP_TIME {
            State::Rest { time: sp.rest }
        } else {
            State::Pop { time }
        };
    }

    /// 대장의 다음 수: 체력이 반 아래면 세 번에 한 번 부하를 부르고(두 번까지), 아니면 돌진과 뛰어 찍기를 번갈아.
    /// 멀면 걸어서 다가간다(Approach 그대로).
    pub(super) fn chief_next(&mut self, dist: f32, player_pos: Vec2) -> State {
        let sp = self.sp();
        let hurt = self.hp * 2 <= self.max_hp;
        if hurt && self.summons < t::CHIEF_SUMMONS && self.pattern % 3 == 2 {
            self.pattern += 1;
            return State::Howl { time: 0.0 };
        }
        if self.pattern.is_multiple_of(2) {
            if dist < sp.trigger {
                self.pattern += 1;
                return State::Windup { time: 0.0 };
            }
        } else if dist < sp.trigger * 1.3 {
            self.pattern += 1;
            return State::Crouch {
                time: 0.0,
                target: player_pos,
            };
        }
        State::Approach
    }

    /// 모모의 다음 수(0046). 만날수록 한 수씩 는다:
    /// 1번째 높이 뛰기 ↔ 뒷발 차기, 2번째 + 굴로 숨기, 3번째 + 뛰기를 세 번 이어서.
    /// 차기는 가까워야, 뛰기는 조금 멀어도 된다. 안 닿으면 걸어서 다가간다.
    pub(super) fn momo_next(
        &mut self,
        dist: f32,
        player_pos: Vec2,
        events: &mut Vec<Event>,
    ) -> State {
        let sp = self.sp();
        let moves = if self.level >= 2 { 3 } else { 2 };
        match self.pattern % moves {
            0 if dist < sp.trigger * 1.5 => {
                self.pattern += 1;
                self.chain = if self.level >= 3 {
                    t::MOMO_HOP_CHAIN - 1
                } else {
                    0
                };
                State::Crouch {
                    time: 0.0,
                    target: player_pos,
                }
            }
            1 if dist < sp.trigger => {
                self.pattern += 1;
                State::Windup { time: 0.0 }
            }
            2 => {
                self.pattern += 1;
                events.push(Event::Dig);
                State::Under { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    /// 도토의 다음 수(0052): 부채꼴 던지기 → 높이 뛰기 → 도토리 묻기를 돈다.
    /// 던지기는 멀어도(유지 거리 끝까지), 뛰기는 조금 멀어도 된다. 묻기는 거리와 상관없이. 안 되면 거리를 맞춘다.
    pub(super) fn doto_next(&mut self, dist: f32, player_pos: Vec2) -> State {
        let sp = self.sp();
        match self.pattern % 3 {
            0 if dist < sp.keep.1 * 1.3 => {
                self.pattern += 1;
                State::Aim { time: 0.0 }
            }
            1 if dist < sp.trigger * 1.5 => {
                self.pattern += 1;
                State::Crouch {
                    time: 0.0,
                    target: player_pos,
                }
            }
            2 => {
                self.pattern += 1;
                State::Howl { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    /// 콩이의 다음 수(0054): 구르기 ↔ 웅크려 버티기. 구르기는 거리 안에서, 버티기는 가까이 걸어온 뒤. 아니면 걸어온다.
    pub(super) fn kongi_next(&mut self, dist: f32) -> State {
        let sp = self.sp();
        match self.pattern % 2 {
            0 if dist < sp.trigger => {
                self.pattern += 1;
                self.chain = self.kongi_bounces();
                State::Windup { time: 0.0 }
            }
            1 if dist < t::KONGI_GUARD_NEAR => {
                self.pattern += 1;
                State::Guard { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    pub(super) fn kongi_bounces(&self) -> u32 {
        if self.hurt_half() {
            t::KONGI_BOUNCES_HURT
        } else {
            t::KONGI_BOUNCES
        }
    }

    /// 토끼풀의 다음 수(0165): 반 아래면 한 번 품기. 아니면 발 구르기 → 높이 뛰기(조금 멀어도) → 가까우면 두 번 차기를 돈다.
    /// 수가 안 되면 고치 앞 자리로 간다.
    pub(super) fn clover_next(&mut self, dist: f32, player_pos: Vec2) -> State {
        let sp = self.sp();
        if self.hurt_half() && self.summons == 0 {
            self.summons = 1;
            return State::Gather { time: 0.0 };
        }
        match self.pattern % 3 {
            0 => {
                self.pattern += 1;
                State::Thump { time: 0.0 }
            }
            1 if dist < sp.trigger * 1.5 => {
                self.pattern += 1;
                State::Crouch {
                    time: 0.0,
                    target: player_pos,
                }
            }
            2 => {
                self.pattern += 1;
                if dist < t::CLOVER_KICK_NEAR * 1.5 {
                    self.chain = t::CLOVER_KICKS - 1;
                    State::Windup { time: 0.0 }
                } else {
                    State::Approach
                }
            }
            _ => State::Approach,
        }
    }

    /// 토끼풀이 서려는 자리: 고치 가운데에서 플레이어 쪽으로 `CLOVER_GUARD_DIST`. 고치가 없으면 플레이어 쪽.
    pub(super) fn clover_post(&self, player_pos: Vec2) -> Vec2 {
        match self.guard {
            Some(g) => g + (player_pos - g).normalize_or(vec2(0.0, 1.0)) * t::CLOVER_GUARD_DIST,
            None => player_pos,
        }
    }
}
