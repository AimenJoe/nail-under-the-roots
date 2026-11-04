//! Steam 업적(0276 S2): 저널(발자국) 목표에서 고른 서른. 조건·이름·설명은 그 목표의 것(열한 언어 그대로) —
//! 업적만 따로 세는 수는 없다. 기록이 바뀔 때 `fresh`로 새로 이룬 것만 골라 `steam::Steam`이 연다.
//! 이미 이룬 옛 기록도 처음 켤 때 한꺼번에 열린다(Steam은 이미 열린 업적을 다시 열어도 아무 일 없다).
//!
//! 아이디(`api`)는 Steamworks 파트너 사이트에 등록하는 "API 이름" — 한 번 올리면 바꾸지 않는다(기록 키처럼, 0041).
//! 이름 규칙(2026-11-04 확정, 0276): `ACH_` + 조건을 말하는 영어 대문자·밑줄(나중의 진행 막대 통계는 `STAT_`로 갈라 둔다),
//! 숫자 기준은 넣지 않는다(목표 수를 고쳐도 이름은 그대로). 확정 목록은 시험 `api_names_are_frozen` — 새 업적은 끝에만 더한다.
//! `hidden`은 Steam의 Spoiler: 이루기 전까지 이름·설명을 감춘다(엔딩·이야기 뒤의 것, 캐릭터 해금을 드러내는 것).

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
    /// Epic Games Store 업적 XP(0276 E3): 합이 정확히 1,000, 하나에 5~200, 5의 배수(동 5~45·은 50~95·금 100~200).
    /// 올린 뒤에는 못 바꾼다 — 시험 `epic_xp_follows_store_rules`. 게임은 읽지 않는다(등록 자료를 만드는 시험만).
    #[cfg_attr(not(test), allow(dead_code))]
    pub xp: u16,
}

const fn a(api: &'static str, goal: &'static str, hidden: bool, xp: u16) -> Achievement {
    Achievement {
        api,
        goal,
        hidden,
        xp,
    }
}

/// 서른: 이야기 열여섯(엔딩 뒤 것은 숨김) · 싸움 여섯 · 탐험 셋 · 이웃 셋 · 마을 둘.
/// 갈아 넣는 수(처치 1000 같은)는 넣지 않는다 — 저널의 둘째 단계 정도까지만(0276 조사: 반복보다 이야기·발견).
pub const ALL: [Achievement; 30] = [
    // 이야기
    a("ACH_TUTORIAL", "tutorial", false, 10),
    a("ACH_BORI_BACK", "bori", false, 25),
    a("ACH_MOMO_FIRST_WIN", "momo1", false, 20),
    a("ACH_MOMO_BACK", "momo3", false, 40),
    a("ACH_DOTO_BACK", "doto", false, 40),
    a("ACH_SUSU_SPROUT_CLAWS", "susu", false, 15),
    a("ACH_KONGI_BACK", "kongi", false, 40),
    a("ACH_DEEPEST_PLACE", "deepest", false, 50),
    a("ACH_CLOVER_BACK", "clover", true, 50),
    a("ACH_ALL_NEIGHBORS_BACK", "all_back", true, 60),
    a("ACH_TRUE_ENDING", "true_end", true, 100),
    a("ACH_ALL_MEMORIES", "memory", true, 50),
    a("ACH_SIDE_BURROW", "side", true, 30),
    a("ACH_MUMU_FREED", "mumu", true, 50),
    a("ACH_MOMO_DEEPEST", "momo_deep", true, 40),
    a("ACH_DOTO_DEEPEST", "doto_deep", true, 40),
    // 싸움
    a("ACH_KILLS", "kills2", false, 30),
    a("ACH_BURST_KILLS", "ambush2", false, 25),
    a("ACH_PARRIES", "parry2", false, 30),
    a("ACH_HEAVY_HITS", "heavy2", false, 25),
    a("ACH_FINISHERS", "finisher", false, 20),
    a("ACH_MOMO_BOWLING", "bowl", true, 20),
    // 탐험
    a("ACH_CHESTS", "chest2", false, 20),
    a("ACH_BREAKABLES", "break", false, 20),
    a("ACH_FALLS", "falls", false, 10),
    // 이웃
    a("ACH_BESTIARY", "mobs2", false, 30),
    a("ACH_TALENTS", "talent2", false, 25),
    a("ACH_DREAM_RUSH", "dream_all", true, 40),
    // 마을
    a("ACH_VILLAGE_DECOR", "decor2", false, 20),
    a("ACH_FIRST_KNOT", "knot", false, 25),
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
        assert_eq!(first, ["ACH_TUTORIAL", "ACH_BORI_BACK"]);
        assert_eq!(fresh(&p, &first).count(), 0, "연 것은 다시 열지 않는다");
    }

    /// 업적마다 아이콘 두 장(열림·잠김, 256×256)이 있다 — `tools/steam/achievement_icons.py`의 표와 이 표가 어긋나지 않게.
    /// 이름은 Steam 일괄 올리기 규칙 `<API>_ACHIEVED.png`·`<API>_UNACHIEVED.png`(이름으로 업적에 붙는다).
    #[test]
    fn every_achievement_has_both_icons() {
        for x in &ALL {
            for f in [
                format!("{}_ACHIEVED.png", x.api),
                format!("{}_UNACHIEVED.png", x.api),
            ] {
                let p = std::path::Path::new("docs/steam/icons").join(&f);
                assert!(p.exists(), "아이콘이 없다: {f}");
            }
        }
    }

    /// 파트너 사이트 등록 자료: 사람이 읽는 표 `docs/steam/achievements.md` + 일괄 올리기 CSV 둘
    /// (`achievements-definitions.csv`·`achievements-localization.csv`, Steamworks 업적 편집기 "Bulk Import/Export" 꼴).
    /// `cargo test write_steam_achievement_sheet -- --ignored`로 다시 만든다.
    #[test]
    #[ignore]
    fn write_steam_achievement_sheet() {
        use crate::i18n::Lang;
        // Steamworks API 언어 코드(partner.steamgames.com/doc/store/localization/languages — 한국어는 `koreana`).
        // 스페인어는 지역색 옅은 중립 스페인어라(0084) 스페인(`spanish`)·중남미(`latam`) 두 칸에 같은 글.
        let langs = [
            (Lang::Ko, "koreana"),
            (Lang::En, "english"),
            (Lang::Ja, "japanese"),
            (Lang::ZhHans, "schinese"),
            (Lang::ZhHant, "tchinese"),
            (Lang::PtBr, "brazilian"),
            (Lang::Fr, "french"),
            (Lang::Es, "spanish"),
            (Lang::Es, "latam"),
            (Lang::De, "german"),
            (Lang::Ru, "russian"),
            (Lang::Pl, "polish"),
        ];
        let mut out = String::from(
            "# Steam 업적 등록표(0276 S2)\n\n`src/achievements.rs`가 만든다 — 손으로 고치지 않는다(`cargo test write_steam_achievement_sheet -- --ignored`).\n\
             파트너 사이트 → Steamworks 설정 편집 → 통계 및 업적 → 업적 편집기 → Bulk Import/Export에 \
             `achievements-definitions.csv`·`achievements-localization.csv`·`icons/*.png`를 한꺼번에 끌어다 놓는다(머리줄은 편집기에서 받은 틀과 대조). \
             API 이름은 한 번 올리면 바꾸지 않는다. \"숨김\" = Steam의 Spoiler.\n\n\
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

        // CSV: 쉼표·따옴표·줄바꿈이 있으면 따옴표로 싸고 안의 따옴표는 둘로.
        fn cell(s: &str) -> String {
            if s.contains([',', '"', '\n']) {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        }
        let mut defs = String::from(
            "api_name,setid,permission,spoiler,archived,progress_stat_name,progress_stat_min,progress_stat_max\n",
        );
        for x in &ALL {
            defs += &format!("{},,Client,{},false,,,\n", x.api, x.hidden);
        }
        let mut loc = String::from("api_name,field");
        for (_, code) in langs {
            loc += &format!(",{code}");
        }
        loc += "\n";
        for x in &ALL {
            let g = journal::goal(x.goal).unwrap();
            for (field, text) in [("name", g.title), ("description", g.desc)] {
                loc += &format!("{},{field}", x.api);
                for (l, _) in langs {
                    loc += &format!(",{}", cell(text(l.strings())));
                }
                loc += "\n";
            }
        }
        std::fs::write("docs/steam/achievements-definitions.csv", defs).unwrap();
        std::fs::write("docs/steam/achievements-localization.csv", loc).unwrap();

        // Epic(0276 E3): EOS 업적 일괄 올리기 꼴(Epic 개발자 포털 BulkTemplate.zip, 2026-11-04 받아 본 그대로) —
        // `achievementDefinitions.csv`(name,hidden,statThresholds) + `achievementLocalizations.csv`(언어마다 한 줄,
        // 빈 locale = 기본 = 영어). 아이콘은 Steam 것을 같은 이름으로(`tools/epic/make_eos_zip.py`가 zip으로 묶는다).
        // Epic 상점 업적 XP는 따로 올리는 표라 `epic-store-xp.csv`(이름·XP·등급)로.
        let eos_langs = [
            (Lang::En, ""),
            (Lang::Ko, "ko"),
            (Lang::Ja, "ja"),
            (Lang::ZhHans, "zh-Hans"),
            (Lang::ZhHant, "zh-Hant"),
            (Lang::PtBr, "pt-BR"),
            (Lang::Fr, "fr"),
            (Lang::Es, "es"),
            (Lang::De, "de"),
            (Lang::Ru, "ru"),
            (Lang::Pl, "pl"),
        ];
        let mut edefs = String::from("name,hidden,statThresholds\n");
        let mut eloc = String::from("name,locale,lockedTitle,lockedDescription,unlockedTitle,unlockedDescription,flavorText,lockedIcon,unlockedIcon\n");
        let mut exp = String::from("name,xp,tier\n");
        for x in &ALL {
            edefs += &format!("{},{},\n", x.api, if x.hidden { "TRUE" } else { "FALSE" });
            let g = journal::goal(x.goal).unwrap();
            for (l, code) in eos_langs {
                let s = l.strings();
                let (t, d) = ((g.title)(s), (g.desc)(s));
                eloc += &format!(
                    "{},{code},{},{},{},{},,{}_UNACHIEVED.png,{}_ACHIEVED.png\n",
                    x.api,
                    cell(t),
                    cell(d),
                    cell(t),
                    cell(d),
                    x.api,
                    x.api
                );
            }
            let tier = match x.xp {
                0..=45 => "bronze",
                50..=95 => "silver",
                _ => "gold",
            };
            exp += &format!("{},{},{tier}\n", x.api, x.xp);
        }
        std::fs::create_dir_all("docs/epic").unwrap();
        std::fs::write("docs/epic/achievementDefinitions.csv", edefs).unwrap();
        std::fs::write("docs/epic/achievementLocalizations.csv", eloc).unwrap();
        std::fs::write("docs/epic/epic-store-xp.csv", exp).unwrap();
    }

    /// Epic Games Store XP 규칙(Epic 업적 문서 2026-11-04): 합 1,000, 하나에 5~200, 5의 배수. 이야기 끝(참 엔딩)이 금.
    #[test]
    fn epic_xp_follows_store_rules() {
        let total: u32 = ALL.iter().map(|x| x.xp as u32).sum();
        assert_eq!(total, 1000, "Epic 업적 XP 합은 정확히 1,000");
        for x in &ALL {
            assert!(
                (5..=200).contains(&x.xp) && x.xp % 5 == 0,
                "{} xp {}",
                x.api,
                x.xp
            );
        }
        let gold = ALL.iter().find(|x| x.api == "ACH_TRUE_ENDING").unwrap();
        assert!(gold.xp >= 100, "참 엔딩은 금(100+)");
    }

    /// Steam에 올린 API 이름(2026-11-04 확정). **이 목록을 고치지 않는다** — 바꾸면 이미 나간 판의 호출이 조용히 빗나가고,
    /// 지운 업적은 딴 사람에게서도 사라진다(Steamworks 업적 문서). 새 업적은 `ALL`과 이 목록 끝에 함께 더한다.
    #[test]
    fn api_names_are_frozen() {
        const FROZEN: [&str; 30] = [
            "ACH_TUTORIAL",
            "ACH_BORI_BACK",
            "ACH_MOMO_FIRST_WIN",
            "ACH_MOMO_BACK",
            "ACH_DOTO_BACK",
            "ACH_SUSU_SPROUT_CLAWS",
            "ACH_KONGI_BACK",
            "ACH_DEEPEST_PLACE",
            "ACH_CLOVER_BACK",
            "ACH_ALL_NEIGHBORS_BACK",
            "ACH_TRUE_ENDING",
            "ACH_ALL_MEMORIES",
            "ACH_SIDE_BURROW",
            "ACH_MUMU_FREED",
            "ACH_MOMO_DEEPEST",
            "ACH_DOTO_DEEPEST",
            "ACH_KILLS",
            "ACH_BURST_KILLS",
            "ACH_PARRIES",
            "ACH_HEAVY_HITS",
            "ACH_FINISHERS",
            "ACH_MOMO_BOWLING",
            "ACH_CHESTS",
            "ACH_BREAKABLES",
            "ACH_FALLS",
            "ACH_BESTIARY",
            "ACH_TALENTS",
            "ACH_DREAM_RUSH",
            "ACH_VILLAGE_DECOR",
            "ACH_FIRST_KNOT",
        ];
        let now: Vec<&str> = ALL.iter().map(|x| x.api).collect();
        assert!(now.len() >= FROZEN.len(), "업적을 지우지 않는다(Archive)");
        assert_eq!(
            &now[..FROZEN.len()],
            &FROZEN[..],
            "올린 API 이름은 바꾸지 않는다"
        );
        for x in &ALL {
            assert!(x.api.starts_with("ACH_") && x.api.len() <= 64, "{}", x.api);
            assert!(
                !x.api.chars().any(|c| c.is_ascii_digit()),
                "숫자 기준은 이름에 넣지 않는다: {}",
                x.api
            );
        }
    }

    /// 엔딩 뒤 이야기(옆 굴·무무·참 엔딩 …)와 캐릭터 해금을 드러내는 것은 Spoiler — 상점 업적 목록이 스포일러가 되지 않게.
    #[test]
    fn after_ending_achievements_are_hidden() {
        for api in [
            "ACH_TRUE_ENDING",
            "ACH_SIDE_BURROW",
            "ACH_MUMU_FREED",
            "ACH_ALL_MEMORIES",
            "ACH_ALL_NEIGHBORS_BACK",
            "ACH_MOMO_DEEPEST",
            "ACH_DOTO_DEEPEST",
            "ACH_MOMO_BOWLING",
        ] {
            assert!(ALL.iter().find(|x| x.api == api).unwrap().hidden, "{api}");
        }
    }
}
