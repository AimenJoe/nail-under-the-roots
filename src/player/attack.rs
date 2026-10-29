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
        // 던지는 타(0195): 활성 구간 첫 틱에 탄. 손앞의 판정(`claw_strike`)은 받아치기·탄 베기만.
        // 고유 재주(0247)는 마무리 타에: 가시 발(모모)·갈라지는 열매(도토).
        let finisher = idx == ms.last();
        if hit.throw != crate::moveset::Throw::None && time >= a0 && time - sdt < a0.max(1e-6) {
            let throw = self.sig_throw(hit.throw, finisher);
            self.throw_shots(throw, dir, self.hit_strike(idx, dir));
        }
        // 가시 덤불을 남기는 차기(가시덩굴 각반, 0246): 활성 구간 첫 틱에 발 앞에.
        if (hit.thorns
            || (self.talents.sig_thorns(finisher) && hit.throw == crate::moveset::Throw::None))
            && time >= a0
            && time - sdt < a0.max(1e-6)
        {
            self.leave_thorns(dir, hit.range);
        }
        self.pos += move_dir * self.speed() * ms.move_factor * dt;
        self.state = State::Attack { idx, time, dir };

        if a.special && self.sp_full() && time >= a1 {
            // 필살기도 휘두른 뒤(활성 구간 뒤)라면 캔슬로 낼 수 있다.
            self.start_special(move_dir);
        } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 && time >= hit.dodge_cancel_after {
            // 굴파기(깡충) 캔슬: 후딜레이를 끊고 다음 콤보로 섞는다.
            self.start_dodge(move_dir);
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
        } else if time >= hit.swing + hit.recovery {
            // 스윙이 끝났다. 박자 창 안에서 눌러 뒀으면 그 자리에서 다음 타(0174; 창이 후딜레이 안에 열리는 마무리 타).
            // 아니면 서서 콤보 창을 연다 — 창 전에 눌러 둔 타는 `tick_free`가 박자에 내보낸다. 마지막 타 뒤는 1타부터.
            let next = if idx < last { idx + 1 } else { 0 };
            if self.queue_due() {
                self.start_swing(next, a.aim, self.queued == Some(super::Queued::Beat));
            } else {
                self.state = State::Free;
                self.combo_next = next;
                self.combo_timer = self.combo_window_after_swing();
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
        if was < self.charge_time() && time >= self.charge_time() {
            self.events.push(Event::HeavyReady);
        }
        self.state = State::Charging { time, dir, after };
        if a.special && self.armed && self.sp_full() {
            self.start_special(move_dir);
        } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
            self.start_dodge(move_dir);
        } else if !a.attack_held {
            if time >= self.charge_time() {
                self.start_heavy(dir, after, time - self.charge_time() <= t::HEAVY_PERFECT);
            } else {
                // 덜 모으고 뗐다: 아무것도 안 나가고(평타는 이미 나갔다), 콤보는 거기서 이어진다.
                // 콤보 창은 스윙 끝과 같이 박자 창이 다 닫힐 때까지(0205): 보통 누름(0.1초 남짓)도 활성 구간 뒤까지
                // 쥐고 있어 여기로 오는데, 무기의 짧은 창(0.3초)만 주면 박자(0.42초) 전에 닫혀 박자에 누른 2타가 1타로 돌아갔다.
                self.state = State::Free;
                if after <= self.moveset.last() {
                    self.combo_next = after;
                    self.combo_timer = self.combo_window_after_swing();
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
        // 던지는 무기의 강타 = 큰 도토리(0195).
        if hit.throw != crate::moveset::Throw::None && time >= a0 && time - sdt < a0.max(1e-6) {
            let s = Strike {
                kind: StrikeKind::Heavy,
                ..self.heavy_strike(dir, perfect)
            };
            let throw = self.sig_throw(hit.throw, true);
            self.throw_shots(throw, dir, s);
        }
        if (hit.thorns || self.talents.sig_thorns(true))
            && hit.throw == crate::moveset::Throw::None
            && time >= a0
            && time - sdt < a0.max(1e-6)
        {
            self.leave_thorns(dir, hit.range);
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
            // 강공격의 후딜레이는 끊지 못한다(0167: 0161 덧붙임의 0.3초 잠금은 뺐다). 박자 창 안에서 눌러 뒀으면
            // 후딜레이가 끝나는 순간 다음 타, 아니면 서서 콤보 창(창 전에 눌러 둔 타는 박자에, 0174).
            let next = if after <= ms.last() { after } else { 0 };
            if self.queue_due() {
                self.start_swing(next, a.aim, self.queued == Some(super::Queued::Beat));
            } else {
                self.state = State::Free;
                self.combo_next = next;
                self.combo_timer = self.combo_window_after_swing();
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
        self.beat_streak = if beat { self.beat_streak + 1 } else { 0 };
        if beat {
            self.beat_show = t::BEAT_PIP_SHOW;
        }
        self.set_beat(self.moveset.hits[idx]);
        self.combo_timer = 0.0;
        self.state = State::Attack {
            idx,
            time: 0.0,
            dir,
        };
        self.events.push(Event::Swing(idx));
        if beat {
            self.events.push(Event::Beat(self.beat_streak));
        }
    }

    /// 스윙이 끝나 설 때 콤보 창(0183): 무기의 `combo_window`와, 박자 창이 다 닫힐 때까지(박자 + 창 두 배) 중 긴 쪽.
    /// 박자가 느려도(0.36초) 늦게 누른 타가 1타로 돌아가지 않게.
    fn combo_window_after_swing(&self) -> f32 {
        let until_beat = (self.beat_at - self.beat_clock) / self.stats.attack_speed.max(0.1);
        self.moveset
            .combo_window
            .max(until_beat + 2.0 * self.beat_window())
    }

    /// 박자 시계를 되감고 다음 타의 박자를 정한다(0174): 스윙 시작부터 `BEAT_PERIOD`, 스윙·후딜레이가 더 길면 그 끝.
    fn set_beat(&mut self, hit: crate::moveset::Hit) {
        self.beat_clock = 0.0;
        self.beat_at = t::BEAT_PERIOD.max(hit.swing + hit.recovery);
    }

    /// 콤보의 이음을 모두 박자에 맞춘 마무리 타인가(0168·0189): 넉백 `FULL_BEAT_KNOCK`배 + 피해 `FULL_BEAT_DAMAGE`.
    pub(super) fn full_beat(&self, idx: usize) -> bool {
        idx == self.moveset.last() && self.combo_beats >= self.moveset.last()
    }

    /// 이번 틱의 손 타격. 던지는 타면 손앞의 짧은 판정(`Toss`: 적은 안 맞고 받아치기·탄 베기만, 0195).
    pub(super) fn claw_strike(&self, idx: usize, dir: Vec2) -> Strike {
        let s = self.hit_strike(idx, dir);
        if self.moveset.hits[idx].throw == crate::moveset::Throw::None {
            s
        } else {
            Strike {
                kind: StrikeKind::Toss,
                ..s
            }
        }
    }

    /// 그 타의 수치로 만든 타격(근접 타 그대로, 던지는 타면 탄이 맞힐 때 쓴다).
    pub(super) fn hit_strike(&self, idx: usize, dir: Vec2) -> Strike {
        let h = self.moveset.hits[idx];
        Strike {
            id: self.swing_id,
            kind: StrikeKind::Claw(idx),
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: h.range,
            },
            damage: h.damage
                + self.stats.damage
                + if self.full_beat(idx) {
                    t::FULL_BEAT_DAMAGE
                } else {
                    0
                },
            knockback: if idx == self.moveset.last() {
                // 콤보의 이음을 모두 박자에 맞췄으면 마무리 타가 더 멀리 민다(0168).
                let full = if self.full_beat(idx) {
                    t::FULL_BEAT_KNOCK
                } else {
                    1.0
                };
                h.knockback * self.stats.finisher_knockback * full
            } else {
                h.knockback
            },
            stun: h.stun + self.stats.stun,
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
            poison: h
                .poison
                .max(self.talents.sig_poison(idx == self.moveset.last())),
            crush: h.crush,
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
        self.queued = None;
        self.combo_timer = 0.0;
        self.on_beat = false;
        self.set_beat(self.moveset.heavy());
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
            // 던지는 무기의 강타는 손에서 받아치기만, 큰 도토리가 맞힌다(0195).
            kind: if h.throw == crate::moveset::Throw::None {
                StrikeKind::Heavy
            } else {
                StrikeKind::Toss
            },
            shape: Shape::Sector {
                origin: self.pos + dir * 2.0,
                dir,
                r: h.range,
            },
            damage: h.damage + self.stats.damage + bonus * t::HEAVY_PERFECT_DAMAGE,
            knockback: h.knockback * self.stats.finisher_knockback,
            stun: h.stun + self.stats.stun,
            hitstop: h.hitstop + bonus as f32 * t::HEAVY_PERFECT_HITSTOP,
            shake: h.shake,
            finisher: true,
            poison: h.poison.max(self.talents.sig_poison(true)),
            crush: h.crush,
        }
    }
}
