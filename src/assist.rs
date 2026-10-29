//! 도움 모드 "포포의 응원"(0041 → 0256): 항목 넷을 따로 고른다 — 받는 피해·쓰러져도 계속·공격 예고 길게·받아치기 창 넓게.
//! 기록(슬롯마다)에 남고, 앱이 `App::apply_assist`로 월드·플레이어에 넣는다. 길의 주인을 이기지 않고 지나가는 길은 없다(0256).

use crate::profile::{key, Profile};

/// 받는 피해 단계: 0 그대로, 1~3 고정(-25%·-50%·-75%), 4 = 쓰러질수록(`RESIST_SCALING`).
pub const RESIST_FIXED: [f32; 4] = [0.0, 0.25, 0.5, 0.75];
/// "쓰러질수록": 20%에서 시작해 쓰러질 때마다 2%씩, 80%까지(Hades God Mode, 0041의 예전 한 줄 켜기).
pub const RESIST_SCALING: usize = 4;
/// 적의 공격 예고 배율(`Enemy::windup_mul`에 곱한다).
pub const TELLS: [f32; 3] = [1.0, 1.3, 1.6];
/// 받아치기 창 배율(`Player::parry_assist`).
pub const PARRY: [f32; 3] = [1.0, 1.5, 2.0];
/// 창의 줄: 받는 피해 | 쓰러져도 계속 | 공격 예고 | 받아치기 창.
pub const ROWS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Assist {
    /// 받는 피해 단계(0..=`RESIST_SCALING`).
    pub resist: usize,
    /// 쓰러져도 계속: 실뿌리가 감기지 않고(런이 끝나지 않는다), 맡기지 않은 반딧불은 70%가 남는다.
    pub go_on: bool,
    /// `TELLS` 번호.
    pub tells: usize,
    /// `PARRY` 번호.
    pub parry: usize,
}

impl Assist {
    /// 기록에서 읽는다. 받는 피해 칸이 없는 옛 기록(0041의 한 줄 켜기 = `ASSIST` 깃발)은 "쓰러질수록" + "쓰러져도 계속"으로.
    pub fn read(p: &Profile) -> Self {
        let go_on = p.flag(key::ASSIST);
        let resist = if p.has_count(key::ASSIST_RESIST) {
            (p.count(key::ASSIST_RESIST) as usize)
                .saturating_sub(1)
                .min(RESIST_SCALING)
        } else if go_on {
            RESIST_SCALING
        } else {
            0
        };
        Self {
            resist,
            go_on,
            tells: (p.count(key::ASSIST_TELLS) as usize).min(TELLS.len() - 1),
            parry: (p.count(key::ASSIST_PARRY) as usize).min(PARRY.len() - 1),
        }
    }

    /// 기록에 쓴다. 받는 피해는 번호 + 1로(0 = 칸 없음과 가르려고).
    pub fn write(self, p: &mut Profile) {
        if self.go_on {
            p.set(key::ASSIST);
        } else {
            p.unset(key::ASSIST);
        }
        p.set_count(key::ASSIST_RESIST, self.resist as u32 + 1);
        p.set_count(key::ASSIST_TELLS, self.tells as u32);
        p.set_count(key::ASSIST_PARRY, self.parry as u32);
    }

    /// 지금 줄여 주는 받는 피해 비율(0..1). `deaths` = "쓰러질수록"을 켠 채 쓰러진 수.
    pub fn resist_now(self, deaths: u32) -> f32 {
        if self.resist >= RESIST_SCALING {
            (0.2 + 0.02 * deaths as f32).min(0.8)
        } else {
            RESIST_FIXED[self.resist]
        }
    }

    pub fn tell_mul(self) -> f32 {
        TELLS[self.tells]
    }

    pub fn parry_mul(self) -> f32 {
        PARRY[self.parry]
    }

    /// 창의 `row`줄을 `d`만큼(좌우). 끝에서 멈춘다. 켬/끔 줄은 어느 쪽이든 바꾼다.
    pub fn step(&mut self, row: usize, d: i8) {
        let nudge = |v: usize, n: usize| (v as i32 + d as i32).clamp(0, n as i32 - 1) as usize;
        match row {
            0 => self.resist = nudge(self.resist, RESIST_SCALING + 1),
            1 => self.go_on = !self.go_on,
            2 => self.tells = nudge(self.tells, TELLS.len()),
            _ => self.parry = nudge(self.parry, PARRY.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 옛 한 줄 켜기(0041)는 예전과 같은 도움으로 읽히고, 새로 쓴 것은 그대로 돌아온다.
    #[test]
    fn old_switch_reads_as_scaling_and_new_choices_round_trip() {
        let mut p = Profile::default();
        assert_eq!(Assist::read(&p), Assist::default());
        p.set(key::ASSIST);
        let old = Assist::read(&p);
        assert_eq!(old.resist, RESIST_SCALING);
        assert!(old.go_on && old.tells == 0 && old.parry == 0);
        assert!((old.resist_now(0) - 0.2).abs() < 1e-6 && (old.resist_now(99) - 0.8).abs() < 1e-6);
        let a = Assist {
            resist: 0,
            go_on: false,
            tells: 2,
            parry: 1,
        };
        a.write(&mut p);
        assert_eq!(Assist::read(&p), a);
        assert!(a.resist_now(5) == 0.0);
        assert!((a.tell_mul() - 1.6).abs() < 1e-6 && (a.parry_mul() - 1.5).abs() < 1e-6);
    }

    #[test]
    fn steps_stop_at_the_ends() {
        let mut a = Assist::default();
        a.step(0, -1);
        assert_eq!(a.resist, 0);
        for _ in 0..9 {
            a.step(0, 1);
            a.step(3, 1);
        }
        assert_eq!((a.resist, a.parry), (RESIST_SCALING, PARRY.len() - 1));
        a.step(1, -1);
        assert!(a.go_on);
    }
}
