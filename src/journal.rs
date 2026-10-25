//! 저널(docs/decisions/0048): 런을 넘어 남는 목표 목록과 보상(Hades 「작은 예언 목록」).
//!
//! 평소 하는 일(처치·패링·상자·이웃 만나기·이야기)이 목표가 되고, 다 하면 저널에서 "받기"로 보상을 받는다.
//! 앞 목표를 이루면 다음 목표가 보인다(`after`). 진행은 모두 기록(profile)에서 계산한다 — 저널 자체는 받았는지만 적는다.
//! 보상은 쓰는 재화(반딧불·마을 씨앗)와 힘이 아닌 것(저널 전용 꾸미기·털빛), 옆으로 넓히는 장신구다(0040).

use crate::i18n::Tr;
use crate::profile::{key, Profile};

/// 런 통계를 기록에 쌓는 키(앱이 월드의 수를 차이만큼 더한다). 이름은 바꾸지 않는다(0041).
pub mod stat {
    pub const KILLS: &str = "stat.kills";
    pub const EMERGE_KILLS: &str = "stat.emerge_kills";
    pub const PARRIES: &str = "stat.parries";
    pub const DRILLS: &str = "stat.drills";
    pub const SLAMS: &str = "stat.drill_slams";
    pub const FINISHERS: &str = "stat.finishers";
    pub const EATEN: &str = "stat.eaten";
    pub const CHESTS: &str = "stat.chests";
    pub const SPRINGS: &str = "stat.springs";
    /// 마을로 보낸 반딧불 누적(쓴 것과 무관).
    pub const LIGHT: &str = "stat.light";
    pub const DAILY_CLEARS: &str = "stat.daily_clears";
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    Story,
    Fight,
    Explore,
    Neighbors,
    Village,
}

impl Cat {
    pub fn name(self) -> Tr {
        match self {
            Cat::Story => |s| s.cat_story,
            Cat::Fight => |s| s.cat_fight,
            Cat::Explore => |s| s.cat_explore,
            Cat::Neighbors => |s| s.cat_neighbors,
            Cat::Village => |s| s.cat_village,
        }
    }
}

/// 무엇을 세나.
#[derive(Clone, Copy, Debug)]
pub enum Metric {
    Count(&'static str),
    Flag(&'static str),
    /// 도감에서 만난 이웃 수.
    Mobs,
    /// 가게에서 산 꾸미기 수.
    Bought,
    /// 깨운 재주 수.
    Talents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    Light(u32),
    Seeds(u32),
    /// village::DECOR_ALL 번호(저널 전용은 6~8).
    Decor(usize),
    /// 털빛 번호(1~3, 0은 기본).
    Fur(u8),
    /// equipment::CHARMS 번호(저널로 여는 것은 BASE_CHARMS부터).
    Charm(usize),
}

pub struct Goal {
    /// 기록 키 `journal.<id>`(받았다). 바꾸지 않는다.
    pub id: &'static str,
    pub cat: Cat,
    pub title: Tr,
    pub desc: Tr,
    pub metric: Metric,
    pub goal: u32,
    /// 이 목표를 이뤄야 보인다.
    pub after: Option<&'static str>,
    pub reward: Reward,
}

use Metric::*;
use Reward::*;

macro_rules! goal {
    ($id:literal, $cat:ident, $t:ident, $d:ident, $m:expr, $n:expr, $after:expr, $r:expr) => {
        Goal {
            id: $id,
            cat: Cat::$cat,
            title: |s| s.$t,
            desc: |s| s.$d,
            metric: $m,
            goal: $n,
            after: $after,
            reward: $r,
        }
    };
}

pub const GOALS: &[Goal] = &[
    // 이야기
    goal!(
        "tutorial",
        Story,
        jg_tutorial,
        jg_tutorial_d,
        Flag(key::TUTORIAL),
        1,
        None,
        Light(20)
    ),
    goal!(
        "bori",
        Story,
        jg_bori,
        jg_bori_d,
        Flag(key::BORI_BACK),
        1,
        None,
        Charm(5)
    ),
    goal!(
        "camp",
        Story,
        jg_camp,
        jg_camp_d,
        Count(key::DEEPEST),
        1,
        None,
        Light(20)
    ),
    goal!(
        "deeper",
        Story,
        jg_deeper,
        jg_deeper_d,
        Count(key::DEEPEST),
        2,
        Some("camp"),
        Light(30)
    ),
    goal!(
        "deepest",
        Story,
        jg_deepest,
        jg_deepest_d,
        Flag(key::HALF_ENDING),
        1,
        Some("deeper"),
        Charm(6)
    ),
    goal!(
        "momo1",
        Story,
        jg_momo1,
        jg_momo1_d,
        Count(key::MOMO_WINS),
        1,
        Some("camp"),
        Light(30)
    ),
    goal!(
        "momo3",
        Story,
        jg_momo3,
        jg_momo3_d,
        Flag(key::MOMO_BACK),
        1,
        Some("momo1"),
        Charm(7)
    ),
    // 싸움
    goal!(
        "kills",
        Fight,
        jg_kills,
        jg_kills_d,
        Count(stat::KILLS),
        30,
        None,
        Light(20)
    ),
    goal!(
        "kills2",
        Fight,
        jg_kills2,
        jg_kills2_d,
        Count(stat::KILLS),
        200,
        Some("kills"),
        Light(60)
    ),
    goal!(
        "ambush",
        Fight,
        jg_ambush,
        jg_ambush_d,
        Count(stat::EMERGE_KILLS),
        10,
        None,
        Light(30)
    ),
    goal!(
        "ambush2",
        Fight,
        jg_ambush2,
        jg_ambush2_d,
        Count(stat::EMERGE_KILLS),
        50,
        Some("ambush"),
        Light(60)
    ),
    goal!(
        "parry",
        Fight,
        jg_parry,
        jg_parry_d,
        Count(stat::PARRIES),
        10,
        None,
        Light(30)
    ),
    goal!(
        "parry2",
        Fight,
        jg_parry2,
        jg_parry2_d,
        Count(stat::PARRIES),
        60,
        Some("parry"),
        Fur(1)
    ),
    goal!(
        "drill",
        Fight,
        jg_drill,
        jg_drill_d,
        Count(stat::DRILLS),
        10,
        None,
        Light(30)
    ),
    goal!(
        "slam",
        Fight,
        jg_slam,
        jg_slam_d,
        Count(stat::SLAMS),
        10,
        Some("drill"),
        Seeds(5)
    ),
    goal!(
        "finisher",
        Fight,
        jg_finisher,
        jg_finisher_d,
        Count(stat::FINISHERS),
        100,
        None,
        Seeds(5)
    ),
    // 탐험
    goal!(
        "chest",
        Explore,
        jg_chest,
        jg_chest_d,
        Count(stat::CHESTS),
        5,
        None,
        Seeds(5)
    ),
    goal!(
        "chest2",
        Explore,
        jg_chest2,
        jg_chest2_d,
        Count(stat::CHESTS),
        25,
        Some("chest"),
        Fur(2)
    ),
    goal!(
        "spring",
        Explore,
        jg_spring,
        jg_spring_d,
        Count(stat::SPRINGS),
        5,
        None,
        Light(20)
    ),
    goal!(
        "berry",
        Explore,
        jg_berry,
        jg_berry_d,
        Count(stat::EATEN),
        20,
        None,
        Seeds(5)
    ),
    goal!(
        "runs",
        Explore,
        jg_runs,
        jg_runs_d,
        Count(key::RUNS),
        5,
        None,
        Light(30)
    ),
    goal!(
        "falls",
        Explore,
        jg_falls,
        jg_falls_d,
        Count(key::DEATHS),
        10,
        None,
        Seeds(5)
    ),
    goal!(
        "daily",
        Explore,
        jg_daily,
        jg_daily_d,
        Count(stat::DAILY_CLEARS),
        1,
        None,
        Seeds(5)
    ),
    // 이웃
    goal!(
        "mobs",
        Neighbors,
        jg_mobs,
        jg_mobs_d,
        Mobs,
        4,
        None,
        Light(20)
    ),
    goal!(
        "mobs2",
        Neighbors,
        jg_mobs2,
        jg_mobs2_d,
        Mobs,
        7,
        Some("mobs"),
        Decor(7)
    ),
    goal!(
        "light",
        Neighbors,
        jg_light,
        jg_light_d,
        Count(stat::LIGHT),
        300,
        None,
        Decor(6)
    ),
    goal!(
        "talent",
        Neighbors,
        jg_talent,
        jg_talent_d,
        Talents,
        1,
        None,
        Light(20)
    ),
    goal!(
        "talent2",
        Neighbors,
        jg_talent2,
        jg_talent2_d,
        Talents,
        4,
        Some("talent"),
        Fur(3)
    ),
    // 마을
    goal!(
        "decor",
        Village,
        jg_decor,
        jg_decor_d,
        Bought,
        3,
        None,
        Seeds(5)
    ),
    goal!(
        "decor2",
        Village,
        jg_decor2,
        jg_decor2_d,
        Bought,
        6,
        Some("decor"),
        Decor(8)
    ),
    goal!(
        "knot",
        Village,
        jg_knot,
        jg_knot_d,
        Count(KNOTS_OPEN),
        2,
        Some("deepest"),
        Light(50)
    ),
];

/// 실뜨기가 열린 단계(2 = 1단계로 깼다). profile::key::knots_open(Mole)와 같은 문자열.
const KNOTS_OPEN: &str = "knots.Mole";

pub fn key_claimed(id: &str) -> String {
    format!("journal.{id}")
}
/// 털빛: 열림(깃발 `fur.<n>`), 고른 것(수 `fur`).
pub fn key_fur(n: u8) -> String {
    format!("fur.{n}")
}
pub const KEY_FUR: &str = "fur";
/// 저널로 연 장신구(상자에서 나오기 시작한다).
pub fn key_charm(i: usize) -> String {
    format!("charm.{i}")
}
pub const FURS: u8 = 4;

/// 털빛 이름(알림·보상 글).
pub fn fur_name(n: u8) -> Tr {
    match n {
        1 => |s| s.fur_1,
        2 => |s| s.fur_2,
        3 => |s| s.fur_3,
        _ => |s| s.fur_0,
    }
}

/// 열린 털빛 가운데 `now` 다음 것(마을 물웅덩이, 0048). 기본(0)은 늘 열려 있다.
pub fn next_fur(p: &Profile, now: u8) -> u8 {
    (1..=FURS)
        .map(|k| (now + k) % FURS)
        .find(|n| *n == 0 || p.flag(&key_fur(*n)))
        .unwrap_or(0)
}

impl Goal {
    pub fn progress(&self, p: &Profile) -> u32 {
        let n = match self.metric {
            Count(k) => p.count(k),
            Flag(k) => p.flag(k) as u32,
            Mobs => crate::species::BESTIARY
                .iter()
                .filter(|k| p.flag(&key::mob(**k)))
                .count() as u32,
            Bought => (0..crate::village::DECOR.len())
                .filter(|i| p.flag(&crate::village::key_decor(*i)))
                .count() as u32,
            Talents => crate::talents::SLOTS
                .iter()
                .flat_map(|s| [0, 1].map(|side| crate::talents::key_awake(*s, side)))
                .filter(|k| p.flag(k))
                .count() as u32,
        };
        n.min(self.goal)
    }

    pub fn done(&self, p: &Profile) -> bool {
        self.progress(p) >= self.goal
    }

    pub fn claimed(&self, p: &Profile) -> bool {
        p.flag(&key_claimed(self.id))
    }

    /// 다 했는데 아직 안 받았다.
    pub fn ready(&self, p: &Profile) -> bool {
        self.done(p) && !self.claimed(p)
    }
}

fn by_id(id: &str) -> Option<&'static Goal> {
    GOALS.iter().find(|g| g.id == id)
}

/// 지금 보이는 목표(앞 목표를 이룬 것만), 표 순서.
pub fn visible(p: &Profile) -> Vec<&'static Goal> {
    GOALS
        .iter()
        .filter(|g| g.after.and_then(by_id).is_none_or(|a| a.done(p)))
        .collect()
}

/// 받을 수 있는 목표 수(알림·표시).
pub fn ready_count(p: &Profile) -> usize {
    visible(p).iter().filter(|g| g.ready(p)).count()
}

/// 받는다: 기록에 보상을 넣고 받았다고 적는다. 받을 수 없으면 None. 저장은 부른 쪽이.
pub fn claim(p: &mut Profile, g: &Goal) -> Option<Reward> {
    if !g.ready(p) {
        return None;
    }
    p.set(&key_claimed(g.id));
    match g.reward {
        Light(n) => p.add(crate::talents::KEY_LIGHT, n),
        Seeds(n) => p.add(crate::village::KEY_SEEDS, n),
        Decor(i) => {
            p.set(&crate::village::key_decor(i));
        }
        Fur(n) => {
            p.set(&key_fur(n));
            p.set_count(KEY_FUR, n as u32);
        }
        Charm(i) => {
            p.set(&key_charm(i));
        }
    }
    Some(g.reward)
}

/// 저널로 받은 장신구(docs/decisions/0050): 받으면 바로 가방에, 그 뒤 런마다 들고 시작한다.
pub fn owned_charms(p: &Profile) -> Vec<usize> {
    (crate::equipment::BASE_CHARMS..crate::equipment::CHARMS.len())
        .filter(|i| p.flag(&key_charm(*i)))
        .collect()
}

/// 받은 보상이 어디 있는지(저널 상태 줄).
pub fn reward_place(r: Reward) -> Tr {
    match r {
        Light(_) => |s| s.j_got_light,
        Seeds(_) => |s| s.j_got_seeds,
        Decor(_) => |s| s.j_got_decor,
        Fur(_) => |s| s.j_got_fur,
        Charm(_) => |s| s.j_got_charm,
    }
}

/// 상자에서 나올 수 있는 장신구(기본 + 저널로 연 것).
pub fn charm_pool(p: &Profile) -> Vec<usize> {
    (0..crate::equipment::CHARMS.len())
        .filter(|i| *i < crate::equipment::BASE_CHARMS || p.flag(&key_charm(*i)))
        .collect()
}

/// 고른 털빛(열린 것만, 아니면 기본).
pub fn fur(p: &Profile) -> u8 {
    let n = p.count(KEY_FUR).min(FURS as u32 - 1) as u8;
    if n == 0 || p.flag(&key_fur(n)) {
        n
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_prerequisites_exist() {
        for (i, g) in GOALS.iter().enumerate() {
            assert!(GOALS[..i].iter().all(|h| h.id != g.id), "{}", g.id);
            if let Some(a) = g.after {
                assert!(by_id(a).is_some(), "{} → {a}", g.id);
            }
            assert!(g.goal > 0);
        }
        assert_eq!(
            key::knots_open(crate::equipment::Character::Mole),
            KNOTS_OPEN
        );
    }

    #[test]
    fn later_goals_appear_after_earlier_ones_and_claim_once() {
        let mut p = Profile::default();
        let kills2 = by_id("kills2").unwrap();
        assert!(!visible(&p).iter().any(|g| g.id == "kills2"));
        p.add(stat::KILLS, 30);
        assert!(visible(&p).iter().any(|g| g.id == "kills2"));
        let kills = by_id("kills").unwrap();
        assert_eq!(ready_count(&p), 1);
        let light = p.count(crate::talents::KEY_LIGHT);
        assert_eq!(claim(&mut p, kills), Some(Light(20)));
        assert_eq!(p.count(crate::talents::KEY_LIGHT), light + 20);
        assert_eq!(claim(&mut p, kills), None, "두 번 받지 않는다");
        assert_eq!(kills2.progress(&p), 30);
        assert_eq!(ready_count(&p), 0);
    }

    #[test]
    fn puddle_cycles_only_unlocked_furs() {
        let mut p = Profile::default();
        assert_eq!(next_fur(&p, 0), 0);
        p.set(&key_fur(2));
        assert_eq!(next_fur(&p, 0), 2);
        assert_eq!(next_fur(&p, 2), 0);
        p.set(&key_fur(3));
        assert_eq!(next_fur(&p, 2), 3);
    }

    #[test]
    fn rewards_unlock_fur_charm_and_decor() {
        let mut p = Profile::default();
        assert_eq!(fur(&p), 0);
        p.set_count(KEY_FUR, 2);
        assert_eq!(fur(&p), 0, "열지 않은 털빛은 못 고른다");
        p.add(stat::CHESTS, 25);
        claim(&mut p, by_id("chest").unwrap());
        assert_eq!(claim(&mut p, by_id("chest2").unwrap()), Some(Fur(2)));
        assert_eq!(fur(&p), 2);
        assert!(!charm_pool(&p).contains(&5));
        p.set(key::BORI_BACK);
        assert!(owned_charms(&p).is_empty());
        claim(&mut p, by_id("bori").unwrap());
        assert!(charm_pool(&p).contains(&5));
        assert_eq!(owned_charms(&p), vec![5], "받은 장신구는 들고 시작한다");
        p.add(stat::LIGHT, 300);
        claim(&mut p, by_id("light").unwrap());
        assert!(p.flag(&crate::village::key_decor(6)));
    }
}
