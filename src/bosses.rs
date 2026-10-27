//! 길의 주인(이름 있는 보스) 표(0170 R12): 어느 길 끝에 서나, 이기면 남기는 기록, 복선.
//! 새 보스 = 여기 한 줄 + `species.rs`(수치·도감) + `enemy/boss.rs`(수) + `cutscene.rs`의 `boss_meet`·`boss_beaten` 가지 + 그림·글.
//! 수가 느는 것(모모의 세 번)·이번 런의 싸움 여부는 `world::Story`, 마을의 자리는 `village::State`, 실의 꿈은 `dream.rs`.

use crate::enemy::Kind;
use crate::profile::{key, Profile};
use crate::route::NodeKind;
use crate::tuning as t;
use crate::world::BossBeaten;

pub struct Boss {
    pub kind: Kind,
    /// 몇째 길의 끝(1·2·마지막)과 어느 칸(끝방·굴토끼 옛 굴).
    pub stage: u32,
    pub node: NodeKind,
    /// 지키는 굴의 방마다 시든 꽃잎이 떠돈다(모모의 복선, 0159).
    pub petals: bool,
    /// 이기면(돌아오면) 세우는 기록 깃발. `wins_key`가 있으면 이긴 수를 세고 돌아온 때(`BossBeaten::back`)만 세운다(모모, 0046).
    pub back_flag: &'static str,
    pub wins_key: Option<&'static str>,
    /// 돌아올 때 덤으로 세우는 깃발(콩이: 바위 발톱, 0054).
    pub extra_flags: &'static [&'static str],
}

pub static ALL: [Boss; 4] = [
    Boss {
        kind: Kind::Momo,
        stage: 1,
        node: NodeKind::Boss,
        petals: true,
        back_flag: key::MOMO_BACK,
        wins_key: Some(key::MOMO_WINS),
        extra_flags: &[],
    },
    Boss {
        kind: Kind::Doto,
        stage: 2,
        node: NodeKind::Boss,
        petals: false,
        back_flag: key::DOTO_MEMORY,
        wins_key: None,
        extra_flags: &[],
    },
    Boss {
        kind: Kind::Kongi,
        stage: t::FINAL_STAGE,
        node: NodeKind::Boss,
        petals: false,
        back_flag: key::KONGI_BACK,
        wins_key: None,
        extra_flags: &[key::STONE],
    },
    Boss {
        kind: Kind::Clover,
        stage: t::FINAL_STAGE,
        node: NodeKind::Warren,
        petals: false,
        back_flag: key::CLOVER_BACK,
        wins_key: None,
        extra_flags: &[],
    },
];

/// 길의 주인이면 그 줄.
pub fn of(kind: Kind) -> Option<&'static Boss> {
    ALL.iter().find(|b| b.kind == kind)
}

impl Boss {
    /// 이긴 결과를 기록에 남긴다(첫 시작에서만 부른다).
    pub fn record(&self, p: &mut Profile, r: BossBeaten) {
        if let Some(k) = self.wins_key {
            p.set_count(k, r.wins as u32);
        }
        if r.back {
            p.set(self.back_flag);
            for f in self.extra_flags {
                p.set(f);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 기록 키는 바꾸지 않는다(0041): 표가 옛 키 그대로인지.
    #[test]
    fn record_keys_match_the_decisions() {
        let mut p = Profile::default();
        of(Kind::Momo).unwrap().record(
            &mut p,
            BossBeaten {
                wins: 2,
                back: false,
            },
        );
        assert_eq!(p.count("momo_wins"), 2);
        assert!(!p.flag("momo"));
        of(Kind::Momo).unwrap().record(
            &mut p,
            BossBeaten {
                wins: 3,
                back: true,
            },
        );
        assert!(p.flag("momo"));
        of(Kind::Kongi).unwrap().record(
            &mut p,
            BossBeaten {
                wins: 1,
                back: true,
            },
        );
        assert!(p.flag("kongi") && p.flag("weapon.stone"));
        of(Kind::Doto).unwrap().record(
            &mut p,
            BossBeaten {
                wins: 1,
                back: true,
            },
        );
        assert!(p.flag("doto_memory"));
        of(Kind::Clover).unwrap().record(
            &mut p,
            BossBeaten {
                wins: 1,
                back: true,
            },
        );
        assert!(p.flag("clover"));
        assert!(of(Kind::Chief).is_none(), "대장은 길의 주인이 아니다(공터)");
    }
}
