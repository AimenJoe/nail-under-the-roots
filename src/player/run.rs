//! 달리기·멀리뛰기(0209): 모모·도토는 회피를 꾹 쥐면 깡충·뒤로 뛰기가 내려앉은 뒤 달리고, 떼면 멀리 뛴다.
//! Naïl의 땅파기(꾹 = 더 머물며 모으기, 떼면 솟구침)와 같은 문법 — 회피 자체는 누르는 순간 나가므로 늦어지지 않는다.

use super::*;

impl Player {
    /// 이 캐릭터가 회피를 쥐고 달리는가(깡충·뒤로 뛰기). Naïl은 땅파기가 쥐기를 쓴다.
    pub(super) fn runs(&self) -> bool {
        matches!(
            self.kit.def().dodge,
            crate::characters::Dodge::Hop | crate::characters::Dodge::Leap
        )
    }

    /// 깡충·뒤로 뛰기가 내려앉는 틱: 아직 쥐고 있으면 달리기로(대기는 멀리뛰기 뒤에 돈다), 아니면 대기를 돌리고 선다.
    pub(super) fn land_dodge(&mut self, c: &Ctx<'_>) {
        if self.runs() && c.a.dodge_held {
            self.state = State::Run { time: 0.0 };
        } else {
            self.burrow_cd = self.burrow_cd_full();
            self.state = State::Free;
        }
    }

    /// 달리기: 입력 쪽으로 빠르게. 움직이는 동안만 도움닫기가 찬다. 떼면 멀리뛰기(짧으면 그냥 선다).
    pub(super) fn tick_run(&mut self, c: &Ctx<'_>, time: f32) -> Option<Strike> {
        let a = c.a;
        let moving = c.move_dir != Vec2::ZERO;
        let time = if moving { time + c.dt } else { time };
        self.pos += c.move_dir * self.speed() * t::RUN_SPEED * c.dt;
        if a.special && self.armed && self.sp_full() {
            self.burrow_cd = self.burrow_cd_full();
            self.start_special(c.move_dir);
        } else if self.armed && self.no_attack <= 0.0 && self.attack_buf > 0.0 {
            // 달리다 치면 그 자리에서 평타(첫 타부터).
            self.burrow_cd = self.burrow_cd_full();
            self.start_swing(0, a.aim, false);
        } else if !a.dodge_held {
            if time >= t::RUN_MIN {
                self.start_jump(time);
            } else {
                self.burrow_cd = self.burrow_cd_full();
                self.state = State::Free;
            }
        } else {
            self.state = State::Run { time };
        }
        None
    }

    /// 멀리뛰기 거리: 도움닫기 `RUN_MIN`에서 가장 짧고 `RUN_BUILD`에서 가장 멀다.
    pub fn jump_dist(run: f32) -> f32 {
        let k = ((run - t::RUN_MIN) / (t::RUN_BUILD - t::RUN_MIN)).clamp(0.0, 1.0);
        t::JUMP_DIST_MIN + (t::JUMP_DIST_MAX - t::JUMP_DIST_MIN) * k
    }

    fn start_jump(&mut self, run: f32) {
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.state = State::Jump {
            time: 0.0,
            dir: self.facing,
            dist: Self::jump_dist(run),
            thrown: false,
        };
        self.events.push(Event::Hop);
    }

    /// 멀리뛰기: 공중 무적. 도토는 공중에서 한 번 던진다(1타 그대로, 겨눈 쪽으로). 모모는 내려앉으며 둘레를 밟는다.
    pub(super) fn tick_jump(
        &mut self,
        c: &Ctx<'_>,
        time: f32,
        dir: Vec2,
        dist: f32,
        thrown: bool,
    ) -> Option<Strike> {
        let time = time + c.dt;
        self.pos += dir * (dist / t::JUMP_TIME) * c.dt;
        let mut thrown = thrown;
        let doto = self.kit.def().dodge == crate::characters::Dodge::Leap;
        if doto && !thrown && self.armed && self.no_attack <= 0.0 && self.attack_buf > 0.0 {
            let aim = c.a.aim.normalize_or(self.facing);
            self.facing = aim;
            self.swing_id += 1;
            self.attack_buf = 0.0;
            self.throw_shots(self.moveset.hits[0].throw, aim, self.hit_strike(0, aim));
            self.events.push(Event::Swing(0));
            thrown = true;
        }
        if time < t::JUMP_TIME {
            self.state = State::Jump {
                time,
                dir,
                dist,
                thrown,
            };
            return None;
        }
        self.burrow_cd = self.burrow_cd_full();
        self.state = State::Free;
        let stomp = !doto;
        self.events.push(Event::JumpLand { stomp });
        stomp.then(|| self.jump_stomp())
    }

    /// 모모 멀리뛰기 착지 밟기 반지름(0209). 재주 발 구르기(0142)를 골랐으면 더 넓다. 월드의 고리도 이 값.
    pub fn jump_stomp_radius(&self) -> f32 {
        if self.talents.hop_stamp() {
            t::JUMP_STOMP_RADIUS * t::JUMP_STOMP_TALENT
        } else {
            t::JUMP_STOMP_RADIUS
        }
    }

    fn jump_stomp(&mut self) -> Strike {
        let bonus = i32::from(self.talents.hop_stamp());
        Strike {
            damage: t::JUMP_STOMP_DAMAGE + bonus + self.stats.damage,
            shape: Shape::Circle {
                center: self.pos,
                r: self.jump_stomp_radius(),
            },
            knockback: t::CHAIN_KNOCKBACK,
            stun: t::CHAIN_STUN,
            shake: t::CHAIN_SHAKE * 0.7,
            ..self.stomp_strike()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Character;

    fn player(kit: Character) -> (Player, Room) {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.kit = kit;
        p.moveset = match kit {
            Character::Momo => &moveset::KICKS,
            _ => &moveset::ACORNS,
        };
        p.facing = vec2(1.0, 0.0);
        // 방 왼쪽, 두 기둥 사이 높이에서 오른쪽으로(뛰고 달리고 멀리 뛰어도 벽·기둥에 닿지 않게).
        p.pos = vec2(24.0, 90.0);
        (p, room)
    }

    fn act(dodge: bool, held: bool, attack: bool) -> Actions {
        Actions {
            move_dir: vec2(1.0, 0.0),
            dodge,
            dodge_held: held,
            attack,
            ..Default::default()
        }
    }

    /// 쥔 채 `run`초 달리고 뗀다. (멀리뛰기 동안의 타격·이벤트, 뗀 순간부터 내려앉을 때까지 움직인 거리)
    fn hold_run_release(p: &mut Player, room: &Room, run: f32) -> (Vec<Strike>, Vec<Event>, f32) {
        p.update(&act(true, true, false), room, t::TICK_DT);
        while !matches!(p.state, State::Run { .. }) {
            p.update(&act(false, true, false), room, t::TICK_DT);
        }
        let mut ran = 0.0;
        while ran < run {
            p.update(&act(false, true, false), room, t::TICK_DT);
            ran += t::TICK_DT;
        }
        let x0 = p.pos.x;
        let (mut strikes, mut events) = (Vec::new(), Vec::new());
        for _ in 0..60 {
            let s = p.update(&Actions::default(), room, t::TICK_DT);
            strikes.extend(s);
            events.extend(p.events.iter().copied());
            if p.state == State::Free {
                break;
            }
        }
        (strikes, events, p.pos.x - x0)
    }

    /// 톡 누르면 지금과 같다: 깡충만, 달리기 없이 대기가 돈다(회피는 누르는 순간 나가고 늦어지지 않는다).
    #[test]
    fn tap_is_still_a_plain_dodge() {
        for kit in [Character::Momo, Character::Doto] {
            let (mut p, room) = player(kit);
            p.update(&act(true, true, false), &room, t::TICK_DT);
            assert!(p.is_invulnerable(), "{kit:?}: 누른 틱에 바로 공중");
            for _ in 0..40 {
                p.update(&Actions::default(), &room, t::TICK_DT);
            }
            assert_eq!(p.state, State::Free, "{kit:?}");
            assert!(p.burrow_cd > 0.0);
        }
    }

    /// 쥐고 있으면 내려앉은 뒤 달리고(맞는다, 걷기보다 빠르다), 떼면 멀리 뛴다 — 오래 달릴수록 멀리, 공중 무적.
    #[test]
    fn hold_runs_and_release_jumps_farther_with_run_up() {
        let (mut p, room) = player(Character::Doto);
        p.update(&act(true, true, false), &room, t::TICK_DT);
        while !matches!(p.state, State::Run { .. }) {
            p.update(&act(false, true, false), &room, t::TICK_DT);
        }
        assert!(
            !p.is_invulnerable() && p.on_foot(),
            "달리는 동안은 맞고 발밑 함정도 밟는다"
        );
        assert_eq!(p.burrow_cd, 0.0, "대기는 멀리뛰기 뒤에");
        let x = p.pos.x;
        p.update(&act(false, true, false), &room, t::TICK_DT);
        let step = p.pos.x - x;
        assert!((step - t::PLAYER_SPEED * t::RUN_SPEED * t::TICK_DT).abs() < 0.01);

        let (_, ev, short) = hold_run_release(&mut player(Character::Doto).0, &room, t::RUN_MIN);
        assert!(ev.contains(&Event::JumpLand { stomp: false }));
        let (_, _, longer) = hold_run_release(&mut player(Character::Doto).0, &room, 0.35);
        // 달린 시간은 틱 단위라 조금 넘친다(한 틱 = 거리 1px 남짓).
        let near = |got: f32, run: f32| (got - Player::jump_dist(run)).abs() < 4.0;
        assert!(near(short, t::RUN_MIN), "{short}");
        assert!(near(longer, 0.35), "{longer}");
        assert!(longer > short + 5.0);
        assert_eq!(Player::jump_dist(t::RUN_MIN), t::JUMP_DIST_MIN);
        assert_eq!(Player::jump_dist(t::RUN_BUILD + 1.0), t::JUMP_DIST_MAX);
        // 짧게 달리고 떼면 뛰지 않고 선다(회피를 두 번 얻지 않는다).
        let (mut p, room) = player(Character::Momo);
        let (s, ev, _) = hold_run_release(&mut p, &room, 0.05);
        assert!(s.is_empty() && !ev.iter().any(|e| matches!(e, Event::JumpLand { .. })));
        assert!(p.burrow_cd > 0.0);
    }

    /// 모모는 멀리뛰기 착지에 둘레를 밟는다(발 구르기 재주면 더 넓고 +1). 도토는 공중에서 한 번만 던진다.
    #[test]
    fn momo_stomps_and_doto_throws_once_in_the_air() {
        let (mut p, room) = player(Character::Momo);
        let (s, _, _) = hold_run_release(&mut p, &room, 0.4);
        assert_eq!(s.len(), 1);
        assert!(matches!(s[0].shape, Shape::Circle { r, .. } if r == t::JUMP_STOMP_RADIUS));
        let (mut p, room) = player(Character::Momo);
        p.talents = crate::talents::Talents {
            pick: [2, 0, 0, 0],
            who: Character::Momo,
        };
        let (s, _, _) = hold_run_release(&mut p, &room, 0.4);
        let big = s.last().unwrap();
        assert!(
            matches!(big.shape, Shape::Circle { r, .. } if r == t::JUMP_STOMP_RADIUS * t::JUMP_STOMP_TALENT)
        );
        assert_eq!(big.damage, t::JUMP_STOMP_DAMAGE + 1 + p.stats.damage);

        let (mut p, room) = player(Character::Doto);
        p.update(&act(true, true, false), &room, t::TICK_DT);
        while !matches!(p.state, State::Run { .. }) {
            p.update(&act(false, true, false), &room, t::TICK_DT);
        }
        for _ in 0..30 {
            p.update(&act(false, true, false), &room, t::TICK_DT);
        }
        p.update(&Actions::default(), &room, t::TICK_DT);
        assert!(matches!(p.state, State::Jump { .. }));
        let mut shots = 0;
        while p.state != State::Free {
            let a = Actions {
                attack: true,
                ..Default::default()
            };
            p.update(&a, &room, t::TICK_DT);
            shots += p
                .spawns
                .iter()
                .filter(|s| matches!(s, Spawn::Shot { .. }))
                .count();
            assert!(p.is_invulnerable() || p.state == State::Free);
        }
        assert_eq!(shots, 1, "공중에서 한 번");
    }

    /// 달리다 공격하면 그 자리에서 1타. Naïl은 달리지 않는다(쥐기는 땅파기).
    #[test]
    fn attack_cuts_the_run_and_nail_never_runs() {
        let (mut p, room) = player(Character::Momo);
        p.update(&act(true, true, false), &room, t::TICK_DT);
        while !matches!(p.state, State::Run { .. }) {
            p.update(&act(false, true, false), &room, t::TICK_DT);
        }
        p.update(&act(false, true, true), &room, t::TICK_DT);
        assert!(matches!(p.state, State::Attack { idx: 0, .. }));
        let (mut p, room) = player(Character::Mole);
        p.moveset = &moveset::CLAWS;
        for _ in 0..90 {
            p.update(&act(false, true, false), &room, t::TICK_DT);
            assert!(!matches!(p.state, State::Run { .. } | State::Jump { .. }));
        }
    }
}
