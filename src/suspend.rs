//! 쉼터부터 이어하기(docs/decisions/0080): 진행 중인 런을 쉼터에서 적어 둔다.
//!
//! - 슬롯마다 하나(저장 칸 4~6, `profile::load_run`·`save_run`). 형식은 기록(`profile.rs`)과 같은 깃발 + 수 글이라
//!   사람이 읽을 수 있고, 모르는 키는 무시한다.
//! - 쉼터에 닿을 때·쉼터에서 가게를 닫을 때·쉼터에서 타이틀로 나갈 때 쓴다. 굴 안에서 끄면 마지막 쉼터부터다
//!   (쓰러져도 쉼터로 돌아오니, 굴 안에서 얻은 것만 잃는다).
//! - 런이 끝나면(새 런·마을로 돌아감·가장 깊은 곳) 지운다.
//!
//! 이 파일은 월드·앱을 모른다. 앱(`app/run.rs`)이 값을 채우고 되돌린다.

use crate::profile::Profile;

/// 형식 번호. 칸의 뜻을 바꾸면 올린다(다른 번호는 읽지 않는다 = 이어하기 없음).
const VERSION: u32 = 1;

/// 가방 한 칸: 무기 또는 장신구(번호).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Weapon(usize),
    Charm(usize),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunSave {
    /// 몇 번째 쉼터인가(1부터).
    pub stage: u32,
    /// 쉼터에 들어선 뒤의 모험 시드(길 지도가 여기서 나온다).
    pub seed: u64,
    pub depth: u32,
    /// 0 = Naïl, 1 = 모모.
    pub character: u8,
    pub knots: u8,
    pub seeds: u32,
    pub glowbugs: u32,
    pub berries: u8,
    pub worms: u8,
    /// 퀵 슬롯(0 = 산딸기, 1 = 지렁이).
    pub selected: u8,
    pub root_hearts: u32,
    pub heart_sold: bool,
    /// 필살기 게이지 × 1000.
    pub sp_milli: u32,
    pub momo_fight: bool,
    pub doto_fight: bool,
    pub kongi_fight: bool,
    pub no_damage: bool,
    /// 이번 런 통계(상태 탭·저널): 이름 → 수.
    pub stats: Vec<(String, u32)>,
    /// 이번 런 시간(초).
    pub run_secs: u32,
    pub slots: Vec<Option<Slot>>,
    pub equipped: Option<usize>,
    pub charms: Vec<usize>,
    pub charms_found: Vec<usize>,
}

/// 리스트 칸 키(`slot.3`).
fn k(prefix: &str, i: usize) -> String {
    format!("{prefix}.{i}")
}

impl RunSave {
    pub fn to_profile(&self) -> Profile {
        let mut p = Profile::default();
        p.set_count("run.version", VERSION);
        p.set_count("stage", self.stage);
        p.set_count("seed.hi", (self.seed >> 32) as u32);
        p.set_count("seed.lo", self.seed as u32);
        p.set_count("depth", self.depth);
        p.set_count("character", self.character as u32);
        p.set_count("knots", self.knots as u32);
        p.set_count("seeds", self.seeds);
        p.set_count("glowbugs", self.glowbugs);
        p.set_count("berries", self.berries as u32);
        p.set_count("worms", self.worms as u32);
        p.set_count("selected", self.selected as u32);
        p.set_count("root_hearts", self.root_hearts);
        p.set_count("sp", self.sp_milli);
        p.set_count("run_secs", self.run_secs);
        for (on, key) in [
            (self.heart_sold, "heart_sold"),
            (self.momo_fight, "momo_fight"),
            (self.doto_fight, "doto_fight"),
            (self.kongi_fight, "kongi_fight"),
            (self.no_damage, "no_damage"),
        ] {
            if on {
                p.set(key);
            }
        }
        for (name, n) in &self.stats {
            p.set_count(&format!("stat.{name}"), *n);
        }
        p.set_count("slots", self.slots.len() as u32);
        for (i, s) in self.slots.iter().enumerate() {
            match s {
                Some(Slot::Weapon(w)) => p.set_count(&k("weapon", i), *w as u32),
                Some(Slot::Charm(c)) => p.set_count(&k("charm", i), *c as u32),
                None => {}
            }
        }
        if let Some(w) = self.equipped {
            p.set_count("equipped", w as u32 + 1);
        }
        for (i, c) in self.charms.iter().enumerate() {
            p.set_count(&k("worn", i), *c as u32 + 1);
        }
        for (i, c) in self.charms_found.iter().enumerate() {
            p.set_count(&k("found", i), *c as u32 + 1);
        }
        p
    }

    /// 형식 번호가 다르거나 쉼터 번호가 없으면 None(이어하기 없음).
    pub fn from_profile(p: &Profile) -> Option<RunSave> {
        if p.count("run.version") != VERSION || p.count("stage") == 0 {
            return None;
        }
        let list = |prefix: &str| -> Vec<usize> {
            (0..)
                .map(|i| p.count(&k(prefix, i)))
                .take_while(|&n| n > 0)
                .map(|n| n as usize - 1)
                .collect()
        };
        let n_slots = p.count("slots").min(64) as usize;
        let slots = (0..n_slots)
            .map(|i| {
                let (w, c) = (k("weapon", i), k("charm", i));
                if p.has_count(&w) {
                    Some(Slot::Weapon(p.count(&w) as usize))
                } else if p.has_count(&c) {
                    Some(Slot::Charm(p.count(&c) as usize))
                } else {
                    None
                }
            })
            .collect();
        let stats = p
            .counts()
            .filter_map(|(key, n)| key.strip_prefix("stat.").map(|s| (s.to_string(), n)))
            .collect();
        Some(RunSave {
            stage: p.count("stage"),
            seed: ((p.count("seed.hi") as u64) << 32) | p.count("seed.lo") as u64,
            depth: p.count("depth"),
            character: p.count("character").min(1) as u8,
            knots: p.count("knots").min(u8::MAX as u32) as u8,
            seeds: p.count("seeds"),
            glowbugs: p.count("glowbugs"),
            berries: p.count("berries").min(u8::MAX as u32) as u8,
            worms: p.count("worms").min(u8::MAX as u32) as u8,
            selected: p.count("selected").min(1) as u8,
            root_hearts: p.count("root_hearts"),
            heart_sold: p.flag("heart_sold"),
            sp_milli: p.count("sp"),
            momo_fight: p.flag("momo_fight"),
            doto_fight: p.flag("doto_fight"),
            kongi_fight: p.flag("kongi_fight"),
            no_damage: p.flag("no_damage"),
            stats,
            run_secs: p.count("run_secs"),
            slots,
            equipped: match p.count("equipped") {
                0 => None,
                n => Some(n as usize - 1),
            },
            charms: list("worn"),
            charms_found: list("found"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RunSave {
        RunSave {
            stage: 2,
            seed: 0xDEAD_BEEF_0051_A7E5,
            depth: 2,
            character: 1,
            knots: 3,
            seeds: 41,
            glowbugs: 0,
            berries: 2,
            worms: 1,
            selected: 1,
            root_hearts: 1,
            heart_sold: true,
            sp_milli: 6500,
            momo_fight: false,
            doto_fight: true,
            kongi_fight: true,
            no_damage: false,
            stats: vec![("kills".into(), 17), ("parries".into(), 3)],
            run_secs: 412,
            slots: vec![
                Some(Slot::Weapon(4)),
                None,
                Some(Slot::Charm(0)),
                Some(Slot::Weapon(0)),
            ],
            equipped: Some(4),
            charms: vec![0],
            charms_found: vec![0, 5],
        }
    }

    /// 쓰고 읽으면(글로 바꿨다 되돌려도) 같은 런이다. 무기·장신구 번호 0도 잃지 않는다.
    #[test]
    fn run_save_round_trips_through_profile_text() {
        let r = sample();
        let text = r.to_profile().encode();
        let back = RunSave::from_profile(&Profile::decode(&text).unwrap()).unwrap();
        let mut a = r.stats.clone();
        let mut b = back.stats.clone();
        a.sort();
        b.sort();
        assert_eq!(a, b);
        assert_eq!(
            RunSave {
                stats: vec![],
                ..back
            },
            RunSave { stats: vec![], ..r }
        );
    }

    /// v0.3.8이 쉼터에서 쓴 이어하기 글의 고정 샘플(0169 R8). 키 이름·번호 +1 약속이 바뀌면 여기서 깨진다
    /// (바꾸면 VERSION을 올리고 옛 글을 읽는 길을 둔다).
    #[test]
    fn reads_and_writes_the_fixed_v0_3_run_text() {
        let text = "nail-profile 1\nflag doto_fight\nflag heart_sold\nflag kongi_fight\ncount berries 2\ncount character 1\ncount charm.2 0\ncount depth 2\ncount equipped 5\ncount found.0 1\ncount found.1 6\ncount glowbugs 0\ncount knots 3\ncount root_hearts 1\ncount run.version 1\ncount run_secs 412\ncount seed.hi 3735928559\ncount seed.lo 5351397\ncount seeds 41\ncount selected 1\ncount slots 4\ncount sp 6500\ncount stage 2\ncount stat.kills 17\ncount stat.parries 3\ncount weapon.0 4\ncount weapon.3 0\ncount worms 1\ncount worn.0 1\n";
        let r = sample();
        assert_eq!(r.to_profile().encode(), text);
        let back = RunSave::from_profile(&Profile::decode(text).unwrap()).unwrap();
        let mut stats = back.stats.clone();
        stats.sort();
        assert_eq!(stats, r.stats);
        assert_eq!(
            RunSave {
                stats: vec![],
                ..back
            },
            RunSave { stats: vec![], ..r }
        );
    }

    /// 빈 기록·다른 형식 번호는 이어하기가 아니다.
    #[test]
    fn empty_or_other_version_is_not_a_run() {
        assert!(RunSave::from_profile(&Profile::default()).is_none());
        let mut p = sample().to_profile();
        p.set_count("run.version", VERSION + 1);
        assert!(RunSave::from_profile(&p).is_none());
    }
}
