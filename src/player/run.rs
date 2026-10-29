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
                self.start_jump();
            } else {
                self.burrow_cd = self.burrow_cd_full();
                self.state = State::Free;
            }
        } else {
            self.state = State::Run { time };
        }
        None
    }

    /// 달리는 동안 멀리뛰기 조준(0211·0213): (내려앉을 자리까지의 벡터, 지금 떼면 뛰는가, 적에게 붙었나).
    /// 렌더가 바닥에 그린다. 붙지 않으면 달리는 쪽으로 `JUMP_DIST` — 벽에 막히는 만큼은 그리는 쪽이 줄인다.
    pub fn jump_aim(&self) -> Option<(Vec2, bool, bool)> {
        let State::Run { time } = self.state else {
            return None;
        };
        let ready = time >= t::RUN_MIN;
        Some(match self.jump_lock() {
            Some(d) => (d, ready, true),
            None => (self.facing * t::JUMP_DIST, ready, false),
        })
    }

    /// 모모 멀리뛰기가 붙을 적(0213, 사장 "아직도 조준이 너무 어려움"): 84px 앞 표시는 스틱을 조금만 틀어도 크게 휘고,
    /// 빠르게 달리며 떼는 순간까지 맞춰야 했다. 달리는 쪽 부채꼴 안의 적에게 표시가 붙는다(드릴 조준 0139와 같은 생각).
    /// 도토는 붙지 않는다 — 적 위로 내려앉으면 위험하고, 공중 던지기가 이미 가장 가까운 적을 겨눈다(0212).
    fn jump_lock(&self) -> Option<Vec2> {
        if self.kit.def().dodge != crate::characters::Dodge::Hop {
            return None;
        }
        let plain = self.facing * t::JUMP_DIST;
        self.targets
            .iter()
            .map(|q| *q - self.pos)
            .filter(|d| {
                let l = d.length();
                (t::JUMP_LOCK_MIN..=t::JUMP_LOCK_MAX).contains(&l)
                    && d.normalize().dot(self.facing) >= t::JUMP_LOCK_COS
            })
            .min_by(|a, b| (*a - plain).length().total_cmp(&(*b - plain).length()))
    }

    fn start_jump(&mut self) {
        let to = self.jump_lock().unwrap_or(self.facing * t::JUMP_DIST);
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.facing = to.normalize_or(self.facing);
        self.state = State::Jump {
            time: 0.0,
            dir: self.facing,
            dist: to.length(),
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
        // 마지막 틱은 남은 만큼만: 조준 자리(0211)에 딱 내려앉게.
        let was = time;
        let time = time + c.dt;
        self.pos += dir * (dist / t::JUMP_TIME) * (time.min(t::JUMP_TIME) - was);
        let mut thrown = thrown;
        let doto = self.kit.def().dodge == crate::characters::Dodge::Leap;
        // 도토(0212, 사장 "가장 가까운 적에게 쏴야 될 듯, 아니면 메리트가 없어"): 공중 꼭대기에서 저절로, 닿는 거리 안의
        // 가장 가까운 적에게 셋째 타(부채꼴 셋)를 한 번. 꼭대기 전에 공격을 누르면 그때(적이 없으면 겨눈 쪽으로).
        if doto && !thrown && self.armed && self.no_attack <= 0.0 {
            let pressed = self.attack_buf > 0.0;
            let apex = was < t::JUMP_TIME * 0.5 && time >= t::JUMP_TIME * 0.5;
            let fan = self.moveset.hits.len() - 1;
            let aim = self
                .air_target(fan)
                .or_else(|| pressed.then(|| c.a.aim.normalize_or(self.facing)));
            if let (true, Some(aim)) = (pressed || apex, aim) {
                self.facing = aim;
                self.swing_id += 1;
                self.attack_buf = 0.0;
                self.throw_shots(self.hit_of(fan).throw, aim, self.hit_strike(fan, aim));
                self.events.push(Event::Swing(fan));
                thrown = true;
            }
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

    /// 공중 던지기가 겨눌 쪽(0212): 그 타의 도토리가 닿는 거리 안에서 가장 가까운 적. 없으면 None.
    fn air_target(&self, idx: usize) -> Option<Vec2> {
        let crate::moveset::Throw::Shot { reach, .. } = self.moveset.hits[idx].throw else {
            return None;
        };
        let reach = reach * self.stats.throw_reach;
        self.targets
            .iter()
            .map(|q| *q - self.pos)
            .filter(|d| d.length() <= reach && d.length() > 0.5)
            .min_by(|a, b| a.length().total_cmp(&b.length()))
            .map(|d| d.normalize())
    }

    /// 회피의 밀치기(0214): 지금 자리 둘레, 피해 없이 밀기만.
    pub(super) fn shove_strike(&mut self) -> Strike {
        self.swing_id += 1;
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Shove,
            shape: Shape::Circle {
                center: self.pos,
                r: t::SHOVE_RADIUS,
            },
            damage: 0,
            knockback: t::SHOVE_KNOCK,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        }
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
        if let Some((to, ready, _)) = p.jump_aim() {
            assert_eq!(ready, ran >= t::RUN_MIN, "조준 자리는 뛸 수 있을 때 밝다");
            assert_eq!(to.normalize(), vec2(1.0, 0.0));
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

    /// 쥐고 있으면 내려앉은 뒤 달리고(맞는다, 걷기보다 빠르다), 떼면 멀리 뛴다 — 공중 무적.
    /// 거리는 달린 시간과 상관없이 `JUMP_DIST`, 달리는 동안 보이는 조준 자리에 그대로 내려앉는다(0211).
    #[test]
    fn hold_runs_and_release_jumps_to_the_aimed_spot() {
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
        assert_eq!(
            p.jump_aim(),
            Some((vec2(t::JUMP_DIST, 0.0), false, false)),
            "아직 덜 달렸다"
        );
        let x = p.pos.x;
        p.update(&act(false, true, false), &room, t::TICK_DT);
        let step = p.pos.x - x;
        assert!((step - t::PLAYER_SPEED * t::RUN_SPEED * t::TICK_DT).abs() < 0.01);

        for run in [t::RUN_MIN, 0.35, 0.6] {
            let (_, ev, d) = hold_run_release(&mut player(Character::Doto).0, &room, run);
            assert!(ev.contains(&Event::JumpLand { stomp: false }));
            assert!((d - t::JUMP_DIST).abs() < 1.0, "달린 시간 {run}: {d}");
        }
        // 짧게 달리고 떼면 뛰지 않고 선다(회피를 두 번 얻지 않는다).
        let (mut p, room) = player(Character::Momo);
        let (s, ev, _) = hold_run_release(&mut p, &room, 0.05);
        assert!(s.is_empty() && !ev.iter().any(|e| matches!(e, Event::JumpLand { .. })));
        assert!(p.burrow_cd > 0.0);
    }

    /// 모모는 멀리뛰기 착지에 둘레를 밟는다(발 구르기 재주면 더 넓고 +1). 도토는 공중에서 가장 가까운 적에게 한 번(0212).
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
            ..Default::default()
        };
        let (s, _, _) = hold_run_release(&mut p, &room, 0.4);
        let big = s.last().unwrap();
        assert!(
            matches!(big.shape, Shape::Circle { r, .. } if r == t::JUMP_STOMP_RADIUS * t::JUMP_STOMP_TALENT)
        );
        assert_eq!(big.damage, t::JUMP_STOMP_DAMAGE + 1 + p.stats.damage);

        // 도토(0212): 누르고 있으면(적 없음) 겨눈 쪽으로 부채꼴 셋 한 번. 안 눌러도 적이 닿는 거리면 꼭대기에서 저절로 그 적에게.
        let air = |targets: Vec<Vec2>, press: bool| {
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
            let mut shots = Vec::new();
            while p.state != State::Free {
                p.targets = targets.clone();
                let a = Actions {
                    attack: press,
                    ..Default::default()
                };
                p.update(&a, &room, t::TICK_DT);
                for s in &p.spawns {
                    if let Spawn::Shot { vel, .. } = s {
                        shots.push((p.pos, *vel));
                    }
                }
                assert!(p.is_invulnerable() || p.state == State::Free);
            }
            shots
        };
        assert_eq!(air(vec![], true).len(), 3, "누르면 한 번(부채꼴 셋)");
        assert!(
            air(vec![], false).is_empty(),
            "적도 누름도 없으면 안 던진다"
        );
        let foe = vec2(24.0 + 200.0, 60.0);
        let shots = air(vec![foe], false);
        assert_eq!(shots.len(), 3, "꼭대기에서 저절로 한 번");
        let (from, mid) = (shots[1].0, shots[1].1);
        assert!(
            mid.normalize().dot((foe - from).normalize()) > 0.99,
            "가장 가까운 적 쪽으로"
        );
    }

    /// 모모 멀리뛰기 조준 붙기(0213): 달리는 쪽 부채꼴 안의 적에게 표시가 붙고, 떼면 그 적 위로 내려앉아 밟는다.
    /// 뒤의 적·너무 먼 적은 붙지 않는다. 도토는 붙지 않는다.
    #[test]
    fn momo_jump_locks_onto_the_foe_ahead() {
        let run_and = |kit: Character, foe: Vec2| {
            let (mut p, room) = player(kit);
            p.update(&act(true, true, false), &room, t::TICK_DT);
            while !matches!(p.state, State::Run { .. }) {
                p.update(&act(false, true, false), &room, t::TICK_DT);
            }
            for _ in 0..15 {
                p.update(&act(false, true, false), &room, t::TICK_DT);
            }
            // 적은 지금 자리 기준으로 둔다(달려온 만큼 앞으로).
            let at = p.pos + foe;
            p.targets = vec![at];
            let aim = p.jump_aim().unwrap();
            for _ in 0..60 {
                p.targets = vec![at];
                p.update(&Actions::default(), &room, t::TICK_DT);
                if p.state == State::Free {
                    break;
                }
            }
            (aim, p.pos, at)
        };
        // 앞쪽 비스듬히 50px: 붙고, 그 자리에 내려앉는다.
        let ((to, _, locked), landed, at) = run_and(Character::Momo, vec2(40.0, -30.0));
        assert!(locked && (to - vec2(40.0, -30.0)).length() < 0.01);
        assert!((landed - at).length() < 1.5, "{landed} vs {at}");
        // 뒤쪽·너무 먼 적은 붙지 않는다.
        assert!(!run_and(Character::Momo, vec2(-40.0, 0.0)).0 .2);
        assert!(
            !run_and(Character::Momo, vec2(t::JUMP_LOCK_MAX + 20.0, 0.0))
                .0
                 .2
        );
        // 도토는 붙지 않는다.
        assert!(!run_and(Character::Doto, vec2(40.0, 0.0)).0 .2);
    }

    /// 첫 점프의 밀치기(0214): 모모는 깡충 착지 자리, 도토는 뒤로 뛰기 도약 자리 — 피해 0, 둘레 `SHOVE_RADIUS`.
    #[test]
    fn first_jump_shoves_without_damage() {
        let (mut p, room) = player(Character::Momo);
        p.update(&act(true, false, false), &room, t::TICK_DT);
        let mut got = None;
        for _ in 0..40 {
            if let Some(s) = p.update(&Actions::default(), &room, t::TICK_DT) {
                got = Some((s, p.pos));
            }
        }
        let (s, at) = got.expect("착지 밀치기");
        assert_eq!(s.kind, StrikeKind::Shove);
        assert_eq!(s.damage, 0);
        assert!(
            matches!(s.shape, Shape::Circle { center, r } if r == t::SHOVE_RADIUS && (center - at).length() < 0.01)
        );
        let (mut p, room) = player(Character::Doto);
        let from = p.pos;
        let s = p.update(&act(true, false, false), &room, t::TICK_DT);
        assert!(s.is_none() && matches!(p.state, State::Leap { .. }));
        let s = p
            .update(&Actions::default(), &room, t::TICK_DT)
            .expect("도약 밀치기");
        assert_eq!(s.kind, StrikeKind::Shove);
        assert!(matches!(s.shape, Shape::Circle { center, .. } if (center - from).length() < 0.01));
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
