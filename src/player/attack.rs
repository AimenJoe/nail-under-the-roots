//! 발톱(0170 R9): 평타 스윙 → 모으기 → 강타 → 후딜레이와 그 시작 함수들. 본문은 `Player::update`에서 그대로 옮겼다.

use super::*;

impl Player {
    /// 평타 스윙(0167): 활성 구간·박자 이음·연타 페널티, 계속 누르고 있으면 모으기로.
    pub(super) fn tick_attack(
        &mut self,
        c: &Ctx<'_>,
        idx: usize,
        time: f32,
        dir: Vec2,
    ) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        let mut strike = None;
        let ms = self.moveset;
        let hit = ms.hits[idx];
        let last = ms.last();
        // 공격 속도: 스윙 시계를 빨리(느리게) 돌린다. 활성·캔슬·후딜레이가 함께 줄어든다.
        let sdt = dt * self.stats.attack_speed;
        let time = time + sdt;
        let (a0, a1) = hit.active;
        if time >= a0 && time - sdt < a1 {
            strike = Some(self.claw_strike(idx, dir));
        }
        self.pos += move_dir * self.speed() * ms.move_factor * dt;
        self.state = State::Attack { idx, time, dir };

        if a.special && self.sp_full() && time >= a1 {
            // 필살기도 휘두른 뒤(활성 구간 뒤)라면 캔슬로 낼 수 있다.
            self.start_special(move_dir);
        } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 && time >= hit.dodge_cancel_after {
            // 굴파기(깡충) 캔슬: 후딜레이를 끊고 다음 콤보로 섞는다.
            self.start_dodge(move_dir);
        } else if time >= a1 && self.queued == Some(true) && idx < last {
            // 박자에 맞춰 눌렀다(0167): 활성 구간이 끝나는 순간 다음 타 — 가장 빠르다.
            self.start_swing(idx + 1, a.aim, true);
        } else if time >= a1 && a.attack_held {
            // 활성 구간이 지났는데 아직 누르고 있다: 바로 강공격 모으기(0161). 어느 타 뒤든.
            // 강공격(또는 덜 모으고 뗀 것) 뒤 콤보는 다음 타로 이어진다.
            self.combo_timer = 0.0;
            self.events.push(Event::ChargeStart);
            self.state = State::Charging {
                time: 0.0,
                dir,
                after: idx + 1,
            };
        } else if self.queued.is_some() && time >= hit.swing + hit.recovery + t::MASH_PENALTY {
            // 박자 전에 눌렀다(연타, 0167): 스윙을 다 마치고 `MASH_PENALTY`만큼 늦게. 마지막 타 뒤면 1타부터.
            let next = if idx < last { idx + 1 } else { 0 };
            self.start_swing(next, a.aim, false);
        } else if self.queued.is_none() && time >= hit.swing + hit.recovery {
            self.state = State::Free;
            if idx < last {
                self.combo_next = idx + 1;
                self.combo_timer = ms.combo_window;
            } else {
                self.combo_timer = 0.0;
            }
        }
        strike
    }

    /// 모으기(0161·0168): 떼면 강타, 다 모이면 번쩍.
    pub(super) fn tick_charging(
        &mut self,
        c: &Ctx<'_>,
        time: f32,
        dir: Vec2,
        after: usize,
    ) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        let was = time;
        let time = time + dt;
        // 모으는 동안은 느리게 걷고, 조준·이동 쪽으로 겨눈다.
        let want = if a.aim != Vec2::ZERO { a.aim } else { move_dir };
        let dir = want.normalize_or(dir);
        self.facing = dir;
        self.pos += move_dir * self.speed() * t::HEAVY_MOVE_FACTOR * dt;
        if was < t::HEAVY_CHARGE && time >= t::HEAVY_CHARGE {
            self.events.push(Event::HeavyReady);
        }
        self.state = State::Charging { time, dir, after };
        if a.special && self.armed && self.sp_full() {
            self.start_special(move_dir);
        } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
            self.start_dodge(move_dir);
        } else if !a.attack_held {
            if time >= t::HEAVY_CHARGE {
                self.start_heavy(dir, after, time - t::HEAVY_CHARGE <= t::HEAVY_PERFECT);
            } else {
                // 덜 모으고 뗐다: 아무것도 안 나가고(평타는 이미 나갔다), 콤보는 거기서 이어진다.
                self.state = State::Free;
                if after <= self.moveset.last() {
                    self.combo_next = after;
                    self.combo_timer = self.moveset.combo_window;
                }
            }
        }
        None
    }

    /// 강타(0168): 활성 구간·후딜레이, 딱 맞춘 강타.
    pub(super) fn tick_heavy(
        &mut self,
        c: &Ctx<'_>,
        time: f32,
        dir: Vec2,
        after: usize,
        perfect: bool,
    ) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        let mut strike = None;
        let ms = self.moveset;
        let hit = ms.heavy();
        let sdt = dt * self.stats.attack_speed;
        let time = time + sdt;
        let (a0, a1) = hit.active;
        if time >= a0 && time - sdt < a1 {
            strike = Some(self.heavy_strike(dir, perfect));
        }
        self.pos += move_dir * self.speed() * ms.move_factor * dt;
        self.state = State::Heavy {
            time,
            dir,
            after,
            perfect,
        };
        if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 && time >= hit.dodge_cancel_after {
            self.start_dodge(move_dir);
        } else if time >= a1 && a.attack_held {
            // 강공격 뒤에도 누르고 있으면 다시 모은다.
            self.combo_timer = 0.0;
            self.events.push(Event::ChargeStart);
            self.state = State::Charging {
                time: 0.0,
                dir,
                after,
            };
        } else if time >= hit.swing + hit.recovery {
            // 강공격의 후딜레이는 끊지 못한다(0167: 0161 덧붙임의 0.3초 잠금은 뺐다). 박자 창 안에서 눌렀으면
            // 후딜레이가 끝나는 순간 다음 타, 그 전에 눌렀으면(연타) `MASH_PENALTY`만큼 늦게.
            let next = if after <= ms.last() { after } else { 0 };
            match self.queued {
                Some(true) => self.start_swing(next, a.aim, true),
                Some(false) if time >= hit.swing + hit.recovery + t::MASH_PENALTY => {
                    self.start_swing(next, a.aim, false)
                }
                Some(false) => {}
                None => {
                    self.state = State::Free;
                    if after <= ms.last() {
                        self.combo_next = after;
                        self.combo_timer = ms.combo_window;
                    } else {
                        self.combo_timer = 0.0;
                    }
                }
            }
        }
        strike
    }

    /// 후딜레이.
    pub(super) fn tick_recover(&mut self, c: &Ctx<'_>, time: f32) -> Option<Strike> {
        let dt = c.dt;
        let time = time - dt;
        self.state = if time <= 0.0 {
            State::Free
        } else {
            State::Recover { time }
        };
        None
    }

    pub(super) fn start_swing(&mut self, idx: usize, aim: Vec2, beat: bool) {
        let dir = if aim != Vec2::ZERO {
            aim.normalize()
        } else {
            self.facing
        };
        self.facing = dir;
        self.swing_id += 1;
        self.swing_age = 0.0;
        // 패링 창은 대기가 끝났을 때만 열린다. 연타하면 몇 번에 한 번만(docs/decisions/0038).
        self.parry_open = self.parry_cd <= 0.0;
        if self.parry_open {
            self.parry_cd = t::PARRY_COOLDOWN;
        }
        self.vel += dir * self.moveset.hits[idx].lunge;
        self.attack_buf = 0.0;
        self.queued = None;
        self.on_beat = beat;
        self.combo_beats = if idx == 0 {
            0
        } else {
            self.combo_beats + usize::from(beat)
        };
        self.combo_timer = 0.0;
        self.state = State::Attack {
            idx,
            time: 0.0,
            dir,
        };
        self.events.push(Event::Swing(idx));
        if beat {
            self.events.push(Event::Beat);
        }
    }

    pub(super) fn claw_strike(&self, idx: usize, dir: Vec2) -> Strike {
        let h = self.moveset.hits[idx];
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Claw(idx),
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: h.range,
            },
            damage: h.damage + self.stats.damage,
            knockback: if idx == self.moveset.last() {
                // 콤보의 이음을 모두 박자에 맞췄으면 마무리 타가 더 멀리 민다(0168).
                let full = if self.combo_beats >= self.moveset.last() {
                    t::FULL_BEAT_KNOCK
                } else {
                    1.0
                };
                h.knockback * self.stats.finisher_knockback * full
            } else {
                h.knockback
            },
            stun: h.stun,
            // 박자에 맞춘 타는 멈춤·흔들림이 조금 더(0168).
            hitstop: h.hitstop
                + if self.on_beat {
                    t::BEAT_HITSTOP_BONUS
                } else {
                    0.0
                },
            shake: h.shake
                + if self.on_beat {
                    t::BEAT_SHAKE_BONUS
                } else {
                    0.0
                },
            finisher: idx == self.moveset.last(),
            poison: h.poison,
        }
    }

    /// 강공격(0161): 다 모으고 뗐다. 받아치기 창은 늘 열린다(모으는 데 0.45초가 들어 연타가 안 된다).
    pub(super) fn start_heavy(&mut self, dir: Vec2, after: usize, perfect: bool) {
        self.facing = dir;
        self.swing_id += 1;
        self.swing_age = 0.0;
        self.parry_open = true;
        self.parry_cd = t::PARRY_COOLDOWN;
        self.vel += dir * self.moveset.heavy().lunge;
        self.attack_buf = 0.0;
        self.combo_timer = 0.0;
        self.on_beat = false;
        self.state = State::Heavy {
            time: 0.0,
            dir,
            after,
            perfect,
        };
        self.events.push(Event::Heavy);
        if perfect {
            self.events.push(Event::HeavyPerfect);
        }
    }

    pub(super) fn heavy_strike(&self, dir: Vec2, perfect: bool) -> Strike {
        let h = self.moveset.heavy();
        let bonus = if perfect { 1 } else { 0 };
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Heavy,
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: h.range,
            },
            damage: h.damage + self.stats.damage + bonus * t::HEAVY_PERFECT_DAMAGE,
            knockback: h.knockback * self.stats.finisher_knockback,
            stun: h.stun,
            hitstop: h.hitstop + bonus as f32 * t::HEAVY_PERFECT_HITSTOP,
            shake: h.shake,
            finisher: true,
            poison: h.poison,
        }
    }
}
