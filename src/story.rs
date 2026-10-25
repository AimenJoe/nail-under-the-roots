//! 런 사이의 한마디(docs/decisions/0041). 쓰러졌다 깨어날 때·새 런을 시작할 때 이웃이 한 줄 한다.
//!
//! 고르는 법(KB concepts/roguelite-narrative-delivery.md, Valve 규칙 DB 방식):
//! 조건이 다 맞는 줄 가운데 **조건이 가장 많은(구체적인) 줄**. 같으면 표에서 앞의 것.
//! `once` 줄은 한 번 보면 기록(`line.<id>`)에 남아 다시 안 나온다. 맞는 줄이 없으면 아무 말도 안 한다.
//! 지는 사람도 이야기를 보게 쓰러진 횟수도 조건으로 쓴다.

use crate::cutscene::Speaker;
use crate::i18n::Tr;
use crate::profile::{key, Profile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// 쓰러졌다가 저장 지점·쉼터에서 깨어났다.
    Wake,
    /// 튜토리얼을 마친 뒤 새 런을 시작했다(오프닝을 건너뛸 때).
    RunStart,
    /// 큰 나무 아래 마을에 왔다(0045).
    Village,
    /// 보리·모모에게 말을 걸었다(0051): 진행에 맞는 한 번짜리 줄이 먼저, 없으면 늘 하는 말(도토 줄은 0052).
    TalkBori,
    TalkMomo,
    /// 도토에게 말을 걸었다(0053).
    TalkDoto,
}

#[derive(Clone, Copy, Debug)]
pub enum Cond {
    DeathsAtLeast(u32),
    /// 기록의 수가 이만큼 이상(모모를 이긴 수 등).
    CountAtLeast(&'static str, u32),
    Flag(&'static str),
    NotFlag(&'static str),
    /// 다른 줄을 이미 봤다(순서 잇기).
    Seen(&'static str),
}

impl Cond {
    fn holds(self, p: &Profile) -> bool {
        match self {
            Cond::DeathsAtLeast(n) => p.count(key::DEATHS) >= n,
            Cond::CountAtLeast(k, n) => p.count(k) >= n,
            Cond::Flag(k) => p.flag(k),
            Cond::NotFlag(k) => !p.flag(k),
            Cond::Seen(id) => p.flag(&key::line(id)),
        }
    }
}

pub struct Line {
    pub id: &'static str,
    pub when: When,
    pub who: Speaker,
    pub text: Tr,
    pub needs: &'static [Cond],
    pub once: bool,
}

use Cond::*;

pub const LINES: &[Line] = &[
    // 깨어날 때: 갈색 뿌리가 데려왔다(0037 2-A) → 나무가 지켜 준다는 의심 → …
    Line {
        id: "first_wake",
        when: When::Wake,
        who: Speaker::Popo,
        text: |s| s.ret_first,
        needs: &[DeathsAtLeast(1)],
        once: true,
    },
    Line {
        id: "bori_fall",
        when: When::Wake,
        who: Speaker::Bori,
        text: |s| s.ret_bori,
        needs: &[DeathsAtLeast(2), Flag(key::BORI_BACK)],
        once: true,
    },
    Line {
        id: "roots_again",
        when: When::Wake,
        who: Speaker::Popo,
        text: |s| s.ret_roots,
        needs: &[DeathsAtLeast(3), Seen("first_wake")],
        once: true,
    },
    // 자주 쓰러지면 도움 모드를 알려 준다(켜져 있으면 안 한다).
    Line {
        id: "assist_hint",
        when: When::Wake,
        who: Speaker::Popo,
        text: |s| s.ret_assist,
        needs: &[DeathsAtLeast(5), NotFlag(key::ASSIST), Seen("first_wake")],
        once: true,
    },
    Line {
        id: "ten",
        when: When::Wake,
        who: Speaker::Popo,
        text: |s| s.ret_ten,
        needs: &[DeathsAtLeast(10), Seen("roots_again")],
        once: true,
    },
    // 마을(0045): 처음 왔을 때, 할매가 처음 왔을 때.
    Line {
        id: "village_intro",
        when: When::Village,
        who: Speaker::Popo,
        text: |s| s.village_intro,
        needs: &[Flag(key::TUTORIAL)],
        once: true,
    },
    Line {
        id: "halmae_arrive",
        when: When::Village,
        who: Speaker::Halmae,
        text: |s| s.halmae_arrive,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            Seen("village_intro"),
        ],
        once: true,
    },
    // 모모(0046): 처음 이긴 뒤 보리가 "그 아이"를 알아본다, 돌아온 뒤 포포가 빈 자리를 본다.
    Line {
        id: "bori_momo",
        when: When::Village,
        who: Speaker::Bori,
        text: |s| s.bori_momo,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::BORI_BACK),
            CountAtLeast(key::MOMO_WINS, 1),
            Seen("village_intro"),
        ],
        once: true,
    },
    Line {
        id: "popo_momo",
        when: When::Village,
        who: Speaker::Popo,
        text: |s| s.popo_momo,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::MOMO_BACK),
            Seen("village_intro"),
        ],
        once: true,
    },
    // 도토(0052): 씨앗 자리를 기억해 낸 뒤 포포가 마을에 온 도토를 알린다.
    Line {
        id: "popo_doto",
        when: When::Village,
        who: Speaker::Popo,
        text: |s| s.popo_doto,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::DOTO_MEMORY),
            Seen("village_intro"),
        ],
        once: true,
    },
    // 참 엔딩 뒤 처음 마을에 오면 포포가 한마디(0053).
    Line {
        id: "popo_true",
        when: When::Village,
        who: Speaker::Popo,
        text: |s| s.popo_true,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::TRUE_ENDING),
            Seen("village_intro"),
        ],
        once: true,
    },
    // 반딧불을 처음 마을로 보낸 뒤: 큰 나무 재주를 알려 준다(0047).
    Line {
        id: "tree_intro",
        when: When::Village,
        who: Speaker::Popo,
        text: |s| s.tree_intro,
        needs: &[
            Flag(key::TUTORIAL),
            CountAtLeast(crate::talents::KEY_LIGHT, 1),
            Seen("village_intro"),
        ],
        once: true,
    },
    // 말 걸기(0051): 이야기가 진행되면 이웃의 말이 바뀐다(한 번씩).
    Line {
        id: "bori_t_momo",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_momo,
        needs: &[Flag(key::MOMO_BACK)],
        once: true,
    },
    Line {
        id: "bori_t_halmae",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_halmae,
        needs: &[Flag(key::HALF_ENDING)],
        once: true,
    },
    Line {
        id: "bori_t_falls",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_falls,
        needs: &[DeathsAtLeast(5)],
        once: true,
    },
    Line {
        id: "bori_t_tree",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_tree,
        needs: &[Seen("tree_intro")],
        once: true,
    },
    // 0053: 수수·실뜨기·독 발톱·참 엔딩 뒤의 한 번짜리 말.
    Line {
        id: "bori_t_susu",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_susu,
        needs: &[Flag(key::SPROUT)],
        once: true,
    },
    Line {
        id: "bori_t_true",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_true,
        needs: &[Flag(key::TRUE_ENDING)],
        once: true,
    },
    Line {
        id: "momo_t_knot",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_knot,
        needs: &[CountAtLeast(crate::journal::KNOTS_OPEN, 3)],
        once: true,
    },
    Line {
        id: "momo_t_true",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_true,
        needs: &[Flag(key::TRUE_ENDING)],
        once: true,
    },
    Line {
        id: "doto_t_venom",
        when: When::TalkDoto,
        who: Speaker::Doto,
        text: |s| s.doto_t_venom,
        needs: &[Flag(key::VENOM)],
        once: true,
    },
    Line {
        id: "doto_t_true",
        when: When::TalkDoto,
        who: Speaker::Doto,
        text: |s| s.doto_t_true,
        needs: &[Flag(key::TRUE_ENDING)],
        once: true,
    },
    Line {
        id: "bori_t_doto",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_doto,
        needs: &[Flag(key::DOTO_MEMORY)],
        once: true,
    },
    Line {
        id: "momo_t_doto",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_doto,
        needs: &[Flag(key::DOTO_MEMORY)],
        once: true,
    },
    Line {
        id: "momo_t_deep",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_deep,
        needs: &[Flag(key::HALF_ENDING)],
        once: true,
    },
    Line {
        id: "momo_t_bori",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_bori,
        needs: &[Flag(key::BORI_BACK)],
        once: true,
    },
    // 반쪽 엔딩 뒤 첫 런: 할매 이야기(한 번).
    Line {
        id: "after_half",
        when: When::RunStart,
        who: Speaker::Popo,
        text: |s| s.run_after_half,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            NotFlag(key::TRUE_ENDING),
        ],
        once: true,
    },
    // 새 런(튜토리얼을 마친 뒤): 매번 짧게.
    Line {
        id: "again",
        when: When::RunStart,
        who: Speaker::Popo,
        text: |s| s.run_again,
        needs: &[Flag(key::TUTORIAL)],
        once: false,
    },
    Line {
        id: "again_bori",
        when: When::RunStart,
        who: Speaker::Popo,
        text: |s| s.run_again_bori,
        needs: &[Flag(key::TUTORIAL), Flag(key::BORI_BACK)],
        once: false,
    },
];

/// 지금 할 한마디. 고른 `once` 줄은 기록에 남긴다(저장은 부른 쪽이).
pub fn take(p: &mut Profile, when: When) -> Option<&'static Line> {
    let mut best: Option<&'static Line> = None;
    for l in LINES {
        let fresh = !l.once || !p.flag(&key::line(l.id));
        if l.when == when
            && fresh
            && l.needs.iter().all(|c| c.holds(p))
            && best.is_none_or(|b| l.needs.len() > b.needs.len())
        {
            best = Some(l);
        }
    }
    if let Some(l) = best.filter(|l| l.once) {
        p.set(&key::line(l.id));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn died(p: &mut Profile, n: u32) {
        p.add(key::DEATHS, n);
    }

    #[test]
    fn most_specific_line_wins_and_once_lines_do_not_repeat() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::Wake).is_none(),
            "쓰러진 적이 없으면 말이 없다"
        );
        died(&mut p, 1);
        assert_eq!(take(&mut p, When::Wake).unwrap().id, "first_wake");
        assert!(
            take(&mut p, When::Wake).is_none(),
            "한 번 본 줄은 다시 안 나온다"
        );
        died(&mut p, 1);
        p.set(key::BORI_BACK);
        assert_eq!(take(&mut p, When::Wake).unwrap().id, "bori_fall");
        died(&mut p, 1);
        assert_eq!(take(&mut p, When::Wake).unwrap().id, "roots_again");
        died(&mut p, 2);
        assert_eq!(take(&mut p, When::Wake).unwrap().id, "assist_hint");
        died(&mut p, 10);
        assert_eq!(take(&mut p, When::Wake).unwrap().id, "ten");
    }

    #[test]
    fn assist_hint_is_skipped_when_assist_is_on() {
        let mut p = Profile::default();
        died(&mut p, 5);
        p.set(&key::line("first_wake"));
        p.set(&key::line("roots_again"));
        p.set(key::ASSIST);
        assert!(take(&mut p, When::Wake).is_none());
    }

    #[test]
    fn momo_village_lines_follow_wins() {
        let mut p = Profile::default();
        p.set(key::TUTORIAL);
        p.set(key::BORI_BACK);
        assert_eq!(take(&mut p, When::Village).unwrap().id, "village_intro");
        assert!(take(&mut p, When::Village).is_none());
        p.add(key::MOMO_WINS, 1);
        assert_eq!(take(&mut p, When::Village).unwrap().id, "bori_momo");
        p.add(key::MOMO_WINS, 2);
        p.set(key::MOMO_BACK);
        assert_eq!(take(&mut p, When::Village).unwrap().id, "popo_momo");
        assert!(take(&mut p, When::Village).is_none());
    }

    #[test]
    fn talk_lines_follow_progress_once_then_fall_back() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::TalkBori).is_none(),
            "진행이 없으면 늘 하는 말"
        );
        p.set(key::HALF_ENDING);
        p.set(key::MOMO_BACK);
        assert_eq!(take(&mut p, When::TalkBori).unwrap().id, "bori_t_momo");
        assert_eq!(take(&mut p, When::TalkBori).unwrap().id, "bori_t_halmae");
        assert!(take(&mut p, When::TalkBori).is_none());
        assert_eq!(take(&mut p, When::TalkMomo).unwrap().id, "momo_t_deep");
    }

    /// 0053: 도토의 말은 독 발톱·참 엔딩을 따르고, 실뜨기 셋째 단계가 열리면 모모가 한마디.
    #[test]
    fn later_progress_lines_play_once() {
        let mut p = Profile::default();
        assert!(take(&mut p, When::TalkDoto).is_none());
        p.set(key::VENOM);
        assert_eq!(take(&mut p, When::TalkDoto).unwrap().id, "doto_t_venom");
        assert!(take(&mut p, When::TalkDoto).is_none());
        p.add(crate::journal::KNOTS_OPEN, 3);
        assert_eq!(take(&mut p, When::TalkMomo).unwrap().id, "momo_t_knot");
        p.set(key::TRUE_ENDING);
        assert_eq!(take(&mut p, When::TalkDoto).unwrap().id, "doto_t_true");
    }

    #[test]
    fn run_start_lines_repeat_and_follow_bori() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::RunStart).is_none(),
            "튜토리얼 전엔 오프닝이 한다"
        );
        p.set(key::TUTORIAL);
        assert_eq!(take(&mut p, When::RunStart).unwrap().id, "again");
        assert_eq!(take(&mut p, When::RunStart).unwrap().id, "again");
        p.set(key::BORI_BACK);
        assert_eq!(take(&mut p, When::RunStart).unwrap().id, "again_bori");
    }
}
