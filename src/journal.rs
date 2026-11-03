//! 저널(docs/decisions/0048): 런을 넘어 남는 목표 목록과 보상(Hades 「작은 예언 목록」).
//!
//! 평소 하는 일(처치·패링·상자·이웃 만나기·이야기)이 목표가 되고, 다 하면 저널에서 "받기"로 보상을 받는다.
//! 앞 목표를 이루면 다음 목표가 보인다(`after`). 진행은 모두 기록(profile)에서 계산한다 — 저널 자체는 받았는지만 적는다.
//! 보상은 쓰는 재화(반딧불·마을 씨앗)와 힘이 아닌 것(저널 전용 꾸미기), 옆으로 넓히는 장신구·무기다(0040).
//! 털빛(0048)은 0272에서 뺐다 — 그 목표 셋은 같은 단계 목표와 맞춘 재화로, 이미 받은 기록에는 `migrate`가 한 번 돌려준다.

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
    // "stat.daily_clears"(오늘의 굴, 0044)는 0272에서 뺐다. 기록에 남은 수는 그대로 두고 키는 다시 쓰지 않는다.
    /// 부순 것(0175, 저널 0272).
    pub const BROKEN: &str = "stat.broken";
    /// 모모의 날려 보내기가 다른 적에 부딪힌 수(0267, 저널 0272).
    pub const BOWLS: &str = "stat.bowls";
    /// 강공격 명중(0168·0173).
    pub const HEAVY: &str = "stat.heavy_hits";
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
    /// 저널 목록의 갈래 띠 색(0173). 갈래 이름은 자세히 칸에 글로도 나온다(색만으로 가르지 않는다).
    pub fn color(self) -> macroquad::prelude::Color {
        use macroquad::prelude::Color;
        match self {
            Cat::Story => Color::new(0.90, 0.71, 0.47, 1.0),
            Cat::Fight => Color::new(0.85, 0.45, 0.40, 1.0),
            Cat::Explore => Color::new(0.55, 0.75, 0.45, 1.0),
            Cat::Neighbors => Color::new(0.50, 0.70, 0.85, 1.0),
            Cat::Village => Color::new(0.85, 0.55, 0.75, 1.0),
        }
    }

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
    /// equipment::CHARMS 번호(저널로 여는 것은 `Charm::journal`).
    Charm(usize),
    /// equipment::WEAPONS 번호(0246): 그 무기의 기록 깃발을 세운다 — 그 캐릭터의 런마다 가방에.
    Weapon(usize),
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
    // 수수(0053): 첫 쉼터에서 새싹 발톱을 받는다.
    goal!(
        "susu",
        Story,
        jg_susu,
        jg_susu_d,
        Flag(key::SPROUT),
        1,
        Some("camp"),
        Light(20)
    ),
    // 콩이(0054): 셋째 길 끝 문지기.
    goal!(
        "kongi",
        Story,
        jg_kongi,
        jg_kongi_d,
        Flag(key::KONGI_BACK),
        1,
        Some("deeper"),
        Light(40)
    ),
    // 토끼풀 가족(0165): 굴토끼 옛 굴 끝에서 깨운다. 보상은 흰 실 매듭.
    goal!(
        "clover",
        Story,
        jg_clover,
        jg_clover_d,
        Flag(key::CLOVER_BACK),
        1,
        Some("momo3"),
        Charm(crate::equipment::SILK_KNOT)
    ),
    // 모모와 가장 깊은 곳(0056): 모모를 골라 끝까지. 보상은 들꽃 화관(0142, 그 전에는 반딧불 40).
    goal!(
        "momo_deep",
        Story,
        jg_momo_deep,
        jg_momo_deep_d,
        Flag(key::MOMO_DEEP),
        1,
        Some("momo3"),
        Charm(crate::equipment::CROWN)
    ),
    // 실의 꿈(0055): 꿈에서 넷을 다 이기기 → 이어서 꾸기.
    goal!(
        "dream",
        Village,
        jg_dream,
        jg_dream_d,
        Count(crate::dream::KEY_WINS),
        4,
        Some("deepest"),
        Light(60)
    ),
    goal!(
        "dream_all",
        Village,
        jg_dream2,
        jg_dream2_d,
        Count("dream.all"),
        1,
        Some("dream"),
        Seeds(30)
    ),
    // 도토와 참 엔딩(0052): 도토가 생기면서 넣었다(닿을 수 없는 목표는 넣지 않는다, 0048).
    goal!(
        "doto",
        Story,
        jg_doto,
        jg_doto_d,
        Flag(key::DOTO_MEMORY),
        1,
        Some("deeper"),
        Light(40)
    ),
    // 도토와 가장 깊은 곳(0195): 도토를 골라 끝까지. 보상은 단풍잎 목도리.
    goal!(
        "doto_deep",
        Story,
        jg_doto_deep,
        jg_doto_deep_d,
        Flag(key::DOTO_DEEP),
        1,
        Some("doto"),
        Charm(crate::equipment::MAPLE_SCARF)
    ),
    // 모모·도토로 가장 깊은 곳에 두 번(0246): 그 캐릭터의 셋째 무기. 기록 키는 `clears.<캐릭터>`(profile::key::clears).
    goal!(
        "momo2",
        Story,
        jg_momo2,
        jg_momo2_d,
        Count("clears.Momo"),
        2,
        Some("momo_deep"),
        Weapon(crate::equipment::PETAL)
    ),
    goal!(
        "doto2",
        Story,
        jg_doto2,
        jg_doto2_d,
        Count("clears.Doto"),
        2,
        Some("doto_deep"),
        Weapon(crate::equipment::CHESTNUT)
    ),
    // 실의 기억(0273 A): 여섯 조각을 다 봄. 보상은 C의 이야기 조각으로 바꿀 자리 — 지금은 같은 단계 이야기 목표의 씨앗.
    goal!(
        "memory",
        Story,
        jg_memory,
        jg_memory_d,
        Flag(key::MEMORY_LAST),
        1,
        Some("deepest"),
        Seeds(20)
    ),
    goal!(
        "true_end",
        Story,
        jg_true,
        jg_true_d,
        Flag(key::TRUE_ENDING),
        1,
        Some("doto"),
        Seeds(20)
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
        // 털빛 1이었다(0272): 같은 둘째 단계 싸움 목표(kills2·ambush2)와 같은 값.
        Light(60)
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
    // 강공격(0161·0168·0173): 모아서 맞히는 손맛을 목표로.
    goal!(
        "heavy",
        Fight,
        jg_heavy,
        jg_heavy_d,
        Count(stat::HEAVY),
        30,
        None,
        Light(30)
    ),
    goal!(
        "heavy2",
        Fight,
        jg_heavy2,
        jg_heavy2_d,
        Count(stat::HEAVY),
        150,
        Some("heavy"),
        Seeds(10)
    ),
    // 모모의 날려 보내기(0267·0272): 모모가 돌아온 뒤에 보인다. 첫 단계 싸움 목표와 같은 값.
    goal!(
        "bowl",
        Fight,
        jg_bowl,
        jg_bowl_d,
        Count(stat::BOWLS),
        20,
        Some("momo3"),
        Light(30)
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
        // 털빛 2였다(0272): 둘째 단계 목표의 마을 씨앗(heavy2)과 같은 값.
        Seeds(10)
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
    // "daily"(오늘의 굴)는 0272에서 뺐다. 아이디는 다시 쓰지 않는다(받은 깃발 `journal.daily`가 남아 있다).
    // 부순 것(0175·0272): 굴의 항아리·버섯·돌무더기. 늘 하는 일이라 첫 단계 탐험 목표와 같은 값.
    goal!(
        "break",
        Explore,
        jg_break,
        jg_break_d,
        Count(stat::BROKEN),
        30,
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
        10,
        Some("mobs"),
        Decor(7)
    ),
    // 도감은 열다섯 종(0146·0165): 다 적으면 하나 더(0173).
    goal!(
        "mobs3",
        Neighbors,
        jg_mobs3,
        jg_mobs3_d,
        Mobs,
        crate::species::BESTIARY.len() as u32,
        Some("mobs2"),
        Light(60)
    ),
    goal!(
        "light",
        Neighbors,
        jg_light,
        jg_light_d,
        Count(stat::LIGHT),
        300,
        None,
        // 예전 보상 반딧불 병(꾸미기 6)은 지웠다(0269) → 마을 씨앗. 이미 받은 기록은 그대로(병만 그리지 않는다).
        Seeds(15)
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
        // 털빛 3이었다(0272): 재주 한 칸(덤 칸 120~495) 값의 일부 — 다음 칸을 앞당기지 않을 만큼.
        Light(40)
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
pub const KNOTS_OPEN: &str = "knots.Mole";

pub fn key_claimed(id: &str) -> String {
    format!("journal.{id}")
}
/// 저널로 연 장신구(상자에서 나오기 시작한다).
pub fn key_charm(i: usize) -> String {
    format!("charm.{i}")
}

/// 털빛을 뺀 뒤(0272) 옛 기록을 맞춘다 — 기록을 읽을 때마다 부른다(두 번째부터는 아무 일 없음).
/// - 털빛을 주던 목표를 이미 받은 기록: 지금 그 목표의 보상을 한 번 준다(털빛 깃발 `fur.<n>`이 받았다는 표시).
/// - 오늘의 굴(0044, 0272에서 뺌)의 날짜 기록 `daily.<날짜>`는 지운다(예전에는 지난 날짜만 지웠다).
pub fn migrate(p: &mut Profile) {
    p.drop_counts(|k| k.starts_with("daily."));
    if p.flag(KEY_FUR_REFUNDED) {
        return;
    }
    for (id, n) in FUR_GOALS {
        if p.flag(&format!("fur.{n}")) && p.flag(&key_claimed(id)) {
            if let Some(g) = by_id(id) {
                grant(p, g.reward);
            }
        }
    }
    if !p.is_empty() {
        p.set(KEY_FUR_REFUNDED);
    }
}

/// 털빛을 주던 목표와 그 털빛 번호(0048 → 0272).
const FUR_GOALS: [(&str, u8); 3] = [("parry2", 1), ("chest2", 2), ("talent2", 3)];
const KEY_FUR_REFUNDED: &str = "mig.0272.fur";

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
            Talents => crate::talents::all_awake_keys()
                .iter()
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
    let mut v: Vec<&'static Goal> = GOALS
        .iter()
        .filter(|g| g.after.and_then(by_id).is_none_or(|a| a.done(p)))
        .collect();
    // 받을 것이 맨 위, 하는 중이 가운데, 받은 것은 아래(같은 묶음 안에서는 표 순서, 0173).
    v.sort_by_key(|g| {
        if g.ready(p) {
            0
        } else if g.claimed(p) {
            2
        } else {
            1
        }
    });
    v
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
    grant(p, g.reward);
    Some(g.reward)
}

/// 보상을 기록에 넣는다(받기·옛 기록 맞추기).
fn grant(p: &mut Profile, r: Reward) {
    match r {
        Light(n) => p.add(crate::talents::KEY_LIGHT, n),
        Seeds(n) => p.add(crate::village::KEY_SEEDS, n),
        Decor(i) => {
            p.set(&crate::village::key_decor(i));
        }
        Charm(i) => {
            p.set(&key_charm(i));
        }
        Weapon(w) => {
            if let Some(k) = crate::equipment::WEAPONS[w].key {
                p.set(k);
            }
        }
    }
}

/// 저널로 받은 장신구인가. 깃발이 없어도 그 장신구를 주는 목표를 이미 받았으면 가진 것으로 친다:
/// 목표의 보상을 나중에 장신구로 바꿔도(momo_deep, 0142) 옛 기록이 그대로 받는다.
fn has_charm(p: &Profile, i: usize) -> bool {
    p.flag(&key_charm(i)) || GOALS.iter().any(|g| g.reward == Charm(i) && g.claimed(p))
}

/// 저널로 받은 장신구(docs/decisions/0050): 받으면 바로 가방에, 그 뒤 런마다 들고 시작한다. 그 캐릭터가 낄 것만(0142).
pub fn owned_charms(p: &Profile, ch: crate::equipment::Character) -> Vec<usize> {
    use crate::equipment::CHARMS;
    (0..CHARMS.len())
        .filter(|i| CHARMS[*i].journal && CHARMS[*i].fits(ch) && has_charm(p, *i))
        .collect()
}

/// 받은 보상이 어디 있는지(저널 상태 줄).
pub fn reward_place(r: Reward) -> Tr {
    match r {
        Light(_) => |s| s.j_got_light,
        Seeds(_) => |s| s.j_got_seeds,
        Decor(_) => |s| s.j_got_decor,
        Charm(_) | Weapon(_) => |s| s.j_got_charm,
    }
}

/// 상자·가게에서 나올 수 있는 장신구(기본 + 저널로 연 것, 그 캐릭터가 낄 것만, 0142).
pub fn charm_pool(p: &Profile, ch: crate::equipment::Character) -> Vec<usize> {
    use crate::equipment::CHARMS;
    (0..CHARMS.len())
        .filter(|i| CHARMS[*i].fits(ch) && (!CHARMS[*i].journal || has_charm(p, *i)))
        .collect()
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
    fn rewards_unlock_charm_and_decor() {
        let mut p = Profile::default();
        p.add(stat::CHESTS, 25);
        claim(&mut p, by_id("chest").unwrap());
        assert_eq!(claim(&mut p, by_id("chest2").unwrap()), Some(Seeds(10)));
        let mole = crate::equipment::Character::Mole;
        assert!(!charm_pool(&p, mole).contains(&5));
        p.set(key::BORI_BACK);
        assert!(owned_charms(&p, mole).is_empty());
        claim(&mut p, by_id("bori").unwrap());
        assert!(charm_pool(&p, mole).contains(&5));
        assert_eq!(
            owned_charms(&p, mole),
            vec![5],
            "받은 장신구는 들고 시작한다"
        );
        // "빛을 돌려주기"는 이제 마을 씨앗(0269: 반딧불 병을 지웠다). 어떤 목표도 지운 꾸미기 6을 주지 않는다.
        p.add(stat::LIGHT, 300);
        let seeds = p.count(crate::village::KEY_SEEDS);
        assert_eq!(claim(&mut p, by_id("light").unwrap()), Some(Seeds(15)));
        assert_eq!(p.count(crate::village::KEY_SEEDS), seeds + 15);
        assert!(!GOALS.iter().any(|g| matches!(g.reward, Decor(6))));
    }

    /// 털빛을 뺀 뒤(0272): 털빛 목표를 이미 받은 기록은 새 보상을 한 번만 받고, 오늘의 굴 날짜 기록은 지운다.
    #[test]
    fn migrate_refunds_claimed_fur_goals_once() {
        let mut p = Profile::default();
        p.set(&key_claimed("parry2"));
        p.set("fur.1");
        p.set(&key_claimed("chest2"));
        p.set("fur.2");
        p.set_count("fur", 2);
        p.add("daily.20261027", 3);
        let (light, seeds) = (
            p.count(crate::talents::KEY_LIGHT),
            p.count(crate::village::KEY_SEEDS),
        );
        migrate(&mut p);
        migrate(&mut p);
        assert_eq!(
            p.count(crate::talents::KEY_LIGHT),
            light + 60,
            "받아치기 60번의 새 보상, 한 번"
        );
        assert_eq!(
            p.count(crate::village::KEY_SEEDS),
            seeds + 10,
            "상자 25개의 새 보상, 한 번"
        );
        assert_eq!(p.count("daily.20261027"), 0);
        // 받지 않은 목표는 그대로 받을 수 있다(재주 4칸 = 반딧불 40).
        assert!(!by_id("talent2").unwrap().claimed(&p));
        assert!(
            !GOALS.iter().any(|g| g.id == "daily"),
            "오늘의 굴 목표는 뺐다"
        );
    }

    /// 모모 장신구(0142): 모모 런에서만 상자·가게에 나오고, 화관은 momo_deep을 받으면(옛 기록이 반딧불로 받았어도) 들고 시작한다.
    #[test]
    fn momo_charms_only_in_momo_runs() {
        use crate::equipment::{Character, CHARMS, CROWN};
        let mut p = Profile::default();
        let momo_only: Vec<usize> = (0..CHARMS.len())
            .filter(|i| CHARMS[*i].only == Some(Character::Momo))
            .collect();
        assert_eq!(momo_only.len(), 6, "셋 + 0246의 셋");
        for i in &momo_only {
            assert!(!charm_pool(&p, Character::Mole).contains(i));
        }
        let pool = charm_pool(&p, Character::Momo);
        assert!(pool.contains(&8) && pool.contains(&9) && !pool.contains(&CROWN));
        // 옛 기록: momo_deep을 받은 깃발만 있다(그때 보상은 반딧불).
        p.set(&key_claimed("momo_deep"));
        assert_eq!(owned_charms(&p, Character::Momo), vec![CROWN]);
        assert!(owned_charms(&p, Character::Mole).is_empty());
    }

    /// 도토 장신구(0195): 도토 런에서만, 목도리는 doto_deep을 받으면 들고 시작한다.
    #[test]
    fn doto_charms_only_in_doto_runs() {
        use crate::equipment::{Character, CHARMS, MAPLE_SCARF};
        let mut p = Profile::default();
        let doto_only: Vec<usize> = (0..CHARMS.len())
            .filter(|i| CHARMS[*i].only == Some(Character::Doto))
            .collect();
        assert_eq!(doto_only.len(), 6, "셋 + 0246의 셋");
        for ch in [Character::Mole, Character::Momo] {
            assert!(doto_only.iter().all(|i| !charm_pool(&p, ch).contains(i)));
        }
        let pool = charm_pool(&p, Character::Doto);
        assert!(pool.contains(&12) && pool.contains(&13) && !pool.contains(&MAPLE_SCARF));
        assert!(!pool.contains(&8), "모모 것은 도토 런에 없다");
        p.set(key::DOTO_MEMORY);
        p.set(key::DOTO_DEEP);
        claim(&mut p, by_id("doto").unwrap());
        claim(&mut p, by_id("doto_deep").unwrap());
        assert_eq!(owned_charms(&p, Character::Doto), vec![MAPLE_SCARF]);
        assert!(charm_pool(&p, Character::Doto).contains(&MAPLE_SCARF));
        assert!(owned_charms(&p, Character::Mole).is_empty());
    }
}
