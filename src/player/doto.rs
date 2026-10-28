//! 도토의 몸(0195): 던지기(탄 내기), 뒤로 뛰기(회피), 도토리 묻기(특기). 탄과 묻은 도토리는 월드가 날리고 터뜨린다
//! (`world::toss`) — 플레이어는 이번 틱에 낼 것을 `spawns`에 쌓기만 한다(소리·이펙트의 `events`처럼).

use super::*;
use crate::moveset::Throw;

/// 이번 틱에 플레이어가 월드에 내놓는 것(0195). 월드가 `World::tick_player`에서 비운다.
#[derive(Debug, Clone, Copy)]
pub enum Spawn {
    /// 던진 도토리 하나. `strike`는 맞을 때 쓸 타격(번호·모양은 월드가 정한다).
    Shot {
        from: Vec2,
        vel: Vec2,
        reach: f32,
        radius: f32,
        splash: f32,
        strike: Strike,
    },
    /// 묻은 도토리 하나: `fuse`초 뒤 `radius` 둘레를 친다(적만).
    Bury {
        at: Vec2,
        fuse: f32,
        radius: f32,
        damage: i32,
    },
}

impl Player {
    /// 던지는 타의 활성 구간 첫 틱에 탄을 낸다(평타·강타 공통). `strike`는 그 타의 근접 수치(피해·멈춤·박자 덤).
    pub(super) fn throw_shots(&mut self, throw: Throw, dir: Vec2, strike: Strike) {
        let Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
        } = throw
        else {
            return;
        };
        let n = count.max(1) as i32;
        for i in 0..n {
            let off = (i as f32 - (n - 1) as f32 * 0.5) * spread;
            let d = Vec2::from_angle(off).rotate(dir);
            self.spawns.push(Spawn::Shot {
                from: self.pos + d * 5.0,
                vel: d * speed,
                reach: reach * self.stats.throw_reach,
                radius,
                splash: splash * self.stats.blast,
                strike,
            });
        }
    }

    /// 뒤로 뛰기(0195): 움직이고 있으면 그쪽으로, 서 있으면 겨눈 쪽 반대로. 겨눈 쪽(`facing`)은 그대로 — 물러나며 겨눈다.
    pub(super) fn start_leap(&mut self, move_dir: Vec2) {
        let dir = if move_dir != Vec2::ZERO {
            move_dir.normalize()
        } else {
            -self.facing
        };
        self.dodge_buf = 0.0;
        self.attack_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        if self.talents.leap_drop() {
            // 떨구고 뛰기: 뛰어오른 자리에 도토리 하나.
            self.spawns.push(Spawn::Bury {
                at: self.pos,
                fuse: t::TAL_DROP_FUSE,
                radius: t::BURY_RADIUS * 0.8 * self.stats.blast,
                damage: t::TAL_DROP_DAMAGE + self.stats.damage,
            });
        }
        self.state = State::Leap { time: 0.0, dir };
        self.events.push(Event::Hop);
    }

    pub(super) fn tick_leap(&mut self, c: &Ctx<'_>, time: f32, dir: Vec2) -> Option<Strike> {
        let time = time + c.dt;
        self.pos +=
            dir * (self.talents.leap_dist() * self.stats.leap_mul / t::BACKLEAP_TIME) * c.dt;
        if time >= t::BACKLEAP_TIME {
            self.burrow_cd = self.burrow_cd_full();
            self.events.push(Event::Land(false));
            self.state = State::Free;
        } else {
            self.state = State::Leap { time, dir };
        }
        None
    }

    /// 도토리 묻기(0195): 겨눈 쪽 앞에 부채꼴로(재주에 따라 셋·다섯), 잠시 뒤 터진다. 게이지를 다 쓴다.
    pub(super) fn start_bury(&mut self, move_dir: Vec2) {
        let dir = move_dir.normalize_or(self.facing);
        self.sp = 0.0;
        self.attack_buf = 0.0;
        self.dodge_buf = 0.0;
        self.combo_timer = 0.0;
        self.vel = Vec2::ZERO;
        self.facing = dir;
        let n = self.talents.bury_count();
        for i in 0..n {
            let k = i as f32 - (n - 1) as f32 * 0.5;
            let d = Vec2::from_angle(k * t::BURY_SPREAD).rotate(dir);
            // 다섯이면 가운데 줄이 조금 더 멀리 — 겹치지 않게.
            let ahead = t::BURY_AHEAD + if n > 3 { (2.0 - k.abs()) * 8.0 } else { 0.0 };
            self.spawns.push(Spawn::Bury {
                at: self.pos + d * ahead,
                fuse: self.talents.bury_fuse(),
                radius: self.talents.bury_radius() * self.stats.blast,
                damage: t::BURY_DAMAGE + self.stats.damage,
            });
        }
        self.state = State::Recover {
            time: t::BURY_RECOVER,
        };
        self.events.push(Event::Bury);
    }
}
