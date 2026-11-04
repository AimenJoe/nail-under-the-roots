//! Steam 업적(0276 S2): 저널(발자국) 목표에서 고른 서른. 조건·이름·설명은 그 목표의 것(열한 언어 그대로) —
//! 업적만 따로 세는 수는 없다. 기록이 바뀔 때 `fresh`로 새로 이룬 것만 골라 `steam::Steam`이 연다.
//! 이미 이룬 옛 기록도 처음 켤 때 한꺼번에 열린다(Steam은 이미 열린 업적을 다시 열어도 아무 일 없다).
//!
//! 아이디(`api`)는 Steamworks 파트너 사이트에 등록하는 "API 이름" — 한 번 올리면 바꾸지 않는다(기록 키처럼, 0041).
//! `hidden`은 상점·업적 목록에서 이루기 전까지 이름을 숨긴다(엔딩·이야기 뒤의 것 — 스포일러).

use crate::journal;
use crate::profile::Profile;

pub struct Achievement {
    /// Steamworks API 이름(대문자·밑줄). 바꾸지 않는다.
    pub api: &'static str,
    /// 조건·이름·설명을 빌려 오는 저널 목표 아이디(`journal::GOALS`).
    pub goal: &'static str,
    /// 이루기 전까지 숨김(이야기·엔딩 뒤). 게임은 읽지 않는다 — 파트너 사이트 등록표(`docs/steam/achievements.md`)를 만드는 시험만.
    #[cfg_attr(not(test), allow(dead_code))]
    pub hidden: bool,
}

const fn a(api: &'static str, goal: &'static str, hidden: bool) -> Achievement {
    Achievement { api, goal, hidden }
}

/// 서른: 이야기 열여섯(엔딩 뒤 것은 숨김) · 싸움 여섯 · 탐험 셋 · 이웃 셋 · 마을 둘.
/// 갈아 넣는 수(처치 1000 같은)는 넣지 않는다 — 저널의 둘째 단계 정도까지만(0276 조사: 반복보다 이야기·발견).
pub const ALL: [Achievement; 30] = [
    // 이야기
    a("TUTORIAL", "tutorial", false),
    a("BORI_BACK", "bori", false),
    a("MOMO_FIRST", "momo1", false),
    a("MOMO_BACK", "momo3", false),
    a("DOTO_MEMORY", "doto", false),
    a("SUSU_SPROUT", "susu", false),
    a("KONGI_BACK", "kongi", false),
    a("DEEPEST", "deepest", false),
    a("CLOVER_FAMILY", "clover", true),
    a("ALL_BACK", "all_back", true),
    a("TRUE_ENDING", "true_end", true),
    a("LAST_MEMORY", "memory", true),
    a("SIDE_PATH", "side", true),
    a("MUMU_FREED", "mumu", true),
    a("MOMO_DEEP", "momo_deep", true),
    a("DOTO_DEEP", "doto_deep", true),
    // 싸움
    a("KILLS", "kills2", false),
    a("AMBUSH", "ambush2", false),
    a("PARRY", "parry2", false),
    a("HEAVY", "heavy2", false),
    a("FINISHER", "finisher", false),
    a("BOWLING", "bowl", false),
    // 탐험
    a("CHESTS", "chest2", false),
    a("BREAKER", "break", false),
    a("FALLS", "falls", false),
    // 이웃
    a("BESTIARY", "mobs2", false),
    a("TALENTS", "talent2", false),
    a("DREAM_ALL", "dream_all", true),
    // 마을
    a("DECOR", "decor2", false),
    a("KNOTS", "knot", false),
];

/// 지금 기록에서 이룬 업적.
pub fn earned(p: &Profile) -> impl Iterator<Item = &'static Achievement> + '_ {
    ALL.iter()
        .filter(move |x| journal::goal(x.goal).is_some_and(|g| g.done(p)))
}

/// 이룬 것 가운데 아직 열지 않은(`opened`에 없는) 것.
pub fn fresh<'a>(
    p: &'a Profile,
    opened: &'a [&'static str],
) -> impl Iterator<Item = &'static Achievement> + 'a {
    earned(p).filter(move |x| !opened.contains(&x.api))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::key;

    #[test]
    fn every_achievement_borrows_a_real_goal_and_ids_are_steam_names() {
        let mut seen = Vec::new();
        for x in &ALL {
            assert!(
                journal::goal(x.goal).is_some(),
                "저널에 없는 목표 {}",
                x.goal
            );
            assert!(
                x.api
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
                "{}",
                x.api
            );
            assert!(!seen.contains(&x.api), "겹친 아이디 {}", x.api);
            assert!(!seen.contains(&x.goal), "겹친 목표 {}", x.goal);
            seen.push(x.api);
            seen.push(x.goal);
        }
    }

    #[test]
    fn fresh_opens_each_once_and_follows_the_record() {
        let mut p = Profile::default();
        assert_eq!(fresh(&p, &[]).count(), 0);
        p.set(key::TUTORIAL);
        p.set(key::BORI_BACK);
        let first: Vec<_> = fresh(&p, &[]).map(|x| x.api).collect();
        assert_eq!(first, ["TUTORIAL", "BORI_BACK"]);
        assert_eq!(fresh(&p, &first).count(), 0, "연 것은 다시 열지 않는다");
    }

    /// 업적마다 아이콘 두 장(열림·잠김, 64×64)이 있다 — `tools/steam/achievement_icons.py`의 표와 이 표가 어긋나지 않게.
    #[test]
    fn every_achievement_has_both_icons() {
        for x in &ALL {
            for f in [format!("{}.png", x.api), format!("{}_locked.png", x.api)] {
                let p = std::path::Path::new("docs/steam/icons").join(&f);
                assert!(p.exists(), "아이콘이 없다: {f}");
            }
        }
    }

    /// 파트너 사이트에 손으로 옮길 등록표: API 이름·숨김·열한 언어 이름과 설명(저널 글 그대로).
    /// `cargo test write_steam_achievement_sheet -- --ignored`로 `docs/steam/achievements.md`를 다시 만든다.
    #[test]
    #[ignore]
    fn write_steam_achievement_sheet() {
        use crate::i18n::Lang;
        // Steamworks 언어 이름(파트너 사이트의 언어 코드).
        let langs = [
            (Lang::Ko, "korean"),
            (Lang::En, "english"),
            (Lang::Ja, "japanese"),
            (Lang::ZhHans, "schinese"),
            (Lang::ZhHant, "tchinese"),
            (Lang::PtBr, "brazilian"),
            (Lang::Fr, "french"),
            (Lang::Es, "spanish"),
            (Lang::De, "german"),
            (Lang::Ru, "russian"),
            (Lang::Pl, "polish"),
        ];
        let mut out = String::from(
            "# Steam 업적 등록표(0276 S2)\n\n`src/achievements.rs`가 만든다 — 손으로 고치지 않는다(`cargo test write_steam_achievement_sheet -- --ignored`).\n\
             파트너 사이트 → 앱 관리 → 통계 및 업적 → 업적에 이 차례로 넣는다. API 이름은 한 번 올리면 바꾸지 않는다.\n\n\
             | # | API 이름 | 숨김 | 한국어 | English |\n|---|---|---|---|---|\n",
        );
        for (i, x) in ALL.iter().enumerate() {
            let g = journal::goal(x.goal).unwrap();
            let (ko, en) = (Lang::Ko.strings(), Lang::En.strings());
            out += &format!(
                "| {} | `{}` | {} | **{}** — {} | **{}** — {} |\n",
                i + 1,
                x.api,
                if x.hidden { "예" } else { "" },
                (g.title)(ko),
                (g.desc)(ko),
                (g.title)(en),
                (g.desc)(en)
            );
        }
        out += "\n## 모든 언어\n";
        for x in &ALL {
            let g = journal::goal(x.goal).unwrap();
            out += &format!("\n### `{}`\n", x.api);
            for (l, code) in langs {
                let s = l.strings();
                out += &format!("- {code}: **{}** — {}\n", (g.title)(s), (g.desc)(s));
            }
        }
        std::fs::create_dir_all("docs/steam").unwrap();
        std::fs::write("docs/steam/achievements.md", out).unwrap();
    }

    /// 엔딩 뒤 이야기(옆 굴·무무·참 엔딩 …)는 숨김 — 상점 업적 목록이 스포일러가 되지 않게.
    #[test]
    fn after_ending_achievements_are_hidden() {
        for api in [
            "TRUE_ENDING",
            "SIDE_PATH",
            "MUMU_FREED",
            "LAST_MEMORY",
            "ALL_BACK",
        ] {
            assert!(ALL.iter().find(|x| x.api == api).unwrap().hidden, "{api}");
        }
    }
}
