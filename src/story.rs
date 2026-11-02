//! 런 사이의 한마디(docs/decisions/0041). 쓰러졌다 깨어날 때·새 런을 시작할 때 이웃이 한 줄 한다.
//!
//! 고르는 법(KB concepts/roguelite-narrative-delivery.md, Valve 규칙 DB 방식):
//! 조건이 다 맞는 줄 가운데 **조건이 가장 많은(구체적인) 줄**. 같으면 표에서 앞의 것.
//! `once` 줄은 한 번 보면 기록(`line.<id>`)에 남아 다시 안 나온다. 맞는 줄이 없으면 아무 말도 안 한다.
//! 지는 사람도 이야기를 보게 쓰러진 횟수도 조건으로 쓴다.

use crate::cutscene::{Face, Speaker};
use crate::equipment::Character;
use crate::i18n::Tr;
use crate::profile::{key, Profile};
use crate::world::Who;

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
    /// 지금 걷는 캐릭터가 이것이다 / 아니다(0264). 이웃이 눈앞의 캐릭터를 제삼자로 말하지 않게, 다른 캐릭터의 일을
    /// 제 일처럼 말하지 않게 가른다. 전에는 기록의 고른 번호(`char_pick`)를 봤다(0201) — 지금은 그 런의 캐릭터를 바로.
    As(Character),
    NotAs(Character),
    Flag(&'static str),
    NotFlag(&'static str),
    /// 다른 줄을 이미 봤다(순서 잇기).
    Seen(&'static str),
}

impl Cond {
    fn holds(self, p: &Profile, ch: Character) -> bool {
        match self {
            Cond::DeathsAtLeast(n) => p.count(key::DEATHS) >= n,
            Cond::CountAtLeast(k, n) => p.count(k) >= n,
            Cond::As(c) => ch == c,
            Cond::NotAs(c) => ch != c,
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

impl Line {
    /// 조건이 얼마나 구체적인가. 캐릭터 조건(`As`·`NotAs`)은 누가 듣는지만 거르고 세지 않는다(0264) — 같은 이야기를
    /// 캐릭터마다 갈라 쓴 줄이 다른 줄보다 먼저 나오지 않게(나일이 듣는 순서는 그대로).
    fn specificity(&self) -> usize {
        self.needs
            .iter()
            .filter(|c| !matches!(c, Cond::As(_) | Cond::NotAs(_)))
            .count()
    }
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
    // 토끼풀 가족(0165): 돌아온 뒤 할매가 마을에서, 보리·모모가 말을 걸면 한 번.
    Line {
        id: "halmae_clover",
        when: When::Village,
        who: Speaker::Halmae,
        text: |s| s.halmae_clover,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            Flag(key::CLOVER_BACK),
            Seen("halmae_arrive"),
        ],
        once: true,
    },
    Line {
        id: "bori_clover",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_clover,
        needs: &[Flag(key::CLOVER_BACK)],
        once: true,
    },
    Line {
        id: "momo_clover",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_v4,
        needs: &[Flag(key::CLOVER_BACK)],
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
    // 모모가 가장 깊은 곳에서 꽃잎을 가져왔다(0056): 할매가 실 속 굴토끼들 이야기를 한다.
    Line {
        id: "halmae_petal",
        when: When::Village,
        who: Speaker::Halmae,
        text: |s| s.halmae_petal,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            Flag(key::MOMO_DEEP),
            Seen("halmae_arrive"),
        ],
        once: true,
    },
    // 실의 꿈(0055): 할매가 온 뒤, 이긴 길의 주인이 있으면 실 고리를 알려 준다.
    Line {
        id: "halmae_dream",
        when: When::Village,
        who: Speaker::Halmae,
        text: |s| s.halmae_dream,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            Seen("halmae_arrive"),
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
        needs: &[Flag(key::MOMO_BACK), NotAs(Character::Momo)],
        once: true,
    },
    Line {
        id: "bori_t_momo_self",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_momo_self,
        needs: &[Flag(key::MOMO_BACK), As(Character::Momo)],
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
        // "발톱을 휘두르는 첫머리에 맞받으면" — 나일의 손에 맞춘 요령이라 나일에게만(0264).
        needs: &[DeathsAtLeast(5), As(Character::Mole)],
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
        needs: &[Flag(key::TRUE_ENDING), NotAs(Character::Doto)],
        once: true,
    },
    // 도토에게는 도토를 "도토가"라 하지 않는다(0264).
    Line {
        id: "momo_t_true_doto",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_true_doto,
        needs: &[Flag(key::TRUE_ENDING), As(Character::Doto)],
        once: true,
    },
    Line {
        id: "doto_t_venom",
        when: When::TalkDoto,
        who: Speaker::Doto,
        text: |s| s.doto_t_venom,
        // "네 발톱에서 쌉쌀한 냄새" — 독은 나일의 고유 재주(0247)라 나일에게만(0264).
        needs: &[Flag(key::VENOM), As(Character::Mole)],
        once: true,
    },
    // 도토로 가장 깊은 곳에 닿았다(0201): 씨앗은 실이 안고 있다 — 묻어 줄 것은 나일(참 엔딩 true_1).
    // "나일, …네 손"이라 부르는 줄이라 나일에게만, 모모에게는 나일에게 전해 달라고(0264).
    Line {
        id: "doto_t_deep",
        when: When::TalkDoto,
        who: Speaker::Doto,
        text: |s| s.doto_t_deep,
        needs: &[
            Flag(key::DOTO_DEEP),
            NotFlag(key::TRUE_ENDING),
            As(Character::Mole),
        ],
        once: true,
    },
    Line {
        id: "doto_t_deep_momo",
        when: When::TalkDoto,
        who: Speaker::Doto,
        text: |s| s.doto_t_deep_momo,
        needs: &[
            Flag(key::DOTO_DEEP),
            NotFlag(key::TRUE_ENDING),
            As(Character::Momo),
        ],
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
        needs: &[Flag(key::DOTO_MEMORY), NotAs(Character::Doto)],
        once: true,
    },
    // 돌아온 이웃 본인에게는 그 이웃을 제삼자로 말하지 않는다(0264): 도토로 와서 보리·모모에게 말을 걸면
    // "도토가 돌아왔다고?"·"도토가 도토리를 줬어"가 눈앞의 도토에게 나왔다.
    Line {
        id: "bori_t_doto_self",
        when: When::TalkBori,
        who: Speaker::Bori,
        text: |s| s.bori_t_doto_self,
        needs: &[Flag(key::DOTO_MEMORY), As(Character::Doto)],
        once: true,
    },
    Line {
        id: "momo_t_doto",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_doto,
        needs: &[Flag(key::DOTO_MEMORY), NotAs(Character::Doto)],
        once: true,
    },
    Line {
        id: "momo_t_doto_self",
        when: When::TalkMomo,
        who: Speaker::Momo,
        text: |s| s.momo_t_doto_self,
        needs: &[Flag(key::DOTO_MEMORY), As(Character::Doto)],
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
        // "가장 깊은 곳까지 갔었다며?" — 반쪽 엔딩은 나일의 것(0056)이라 나일 런에(0264). 전에는 다음 런이 모모면
        // 모모에게(배웅 줄과 조건 수가 같아 표에서 앞선 이 줄이 이겼다).
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::HALF_ENDING),
            NotFlag(key::TRUE_ENDING),
            As(Character::Mole),
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
    // 모모와 내려갈 때(0056): 포포가 모모를 배웅한다. again_bori와 조건 수가 같아 표에서 앞선 이 줄이 이긴다.
    Line {
        id: "again_momo",
        when: When::RunStart,
        who: Speaker::Popo,
        text: |s| s.run_momo,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::MOMO_BACK),
            // "고른 번호 1 이상"이면 도토 런에도 모모를 배웅했다(0201) → 이번 런의 캐릭터로(0264).
            As(Character::Momo),
        ],
        once: false,
    },
    // 도토와 내려갈 때(0201): 포포가 도토를 배웅한다.
    Line {
        id: "again_doto",
        when: When::RunStart,
        who: Speaker::Popo,
        text: |s| s.run_doto,
        needs: &[
            Flag(key::TUTORIAL),
            Flag(key::DOTO_MEMORY),
            As(Character::Doto),
        ],
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

/// 이웃의 늘 하는 말(0265): 말 걸 때마다 차례로 셋. 듣는 캐릭터마다 줄이 다르면 캐릭터마다 한 줄씩 둔다 —
/// 요령(무기·회피)이나 겪은 일을 말하는 줄은 그 캐릭터의 것으로(수수의 무기 바꾸기·콩이의 등딱지·Naïl의 배웅).
/// 예전엔 `app/stage.rs`에 이웃마다 배열이 흩어져 있어, Naïl의 요령이 모모·도토에게도 나왔다.
pub struct Chatter {
    pub who: Who,
    /// 누구에게(None = 누구에게나).
    pub to: Option<Character>,
    pub lines: [Tr; 3],
    pub face: Face,
}

pub const CHATTER: &[Chatter] = &[
    Chatter {
        who: Who::Bori,
        to: None,
        lines: [|s| s.bori_1, |s| s.bori_2, |s| s.bori_3],
        face: Face::Normal,
    },
    Chatter {
        who: Who::Susu,
        to: Some(Character::Mole),
        lines: [|s| s.susu_v1, |s| s.susu_v2, |s| s.susu_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Susu,
        to: Some(Character::Momo),
        lines: [|s| s.susu_v1_momo, |s| s.susu_v2, |s| s.susu_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Susu,
        to: Some(Character::Doto),
        lines: [|s| s.susu_v1_doto, |s| s.susu_v2, |s| s.susu_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Kongi,
        to: Some(Character::Mole),
        lines: [|s| s.kongi_v1, |s| s.kongi_v2, |s| s.kongi_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Kongi,
        to: Some(Character::Momo),
        lines: [|s| s.kongi_v1, |s| s.kongi_v2_momo, |s| s.kongi_v3_momo],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Kongi,
        to: Some(Character::Doto),
        lines: [|s| s.kongi_v1, |s| s.kongi_v2_doto, |s| s.kongi_v3_doto],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Clover,
        to: None,
        lines: [|s| s.clover_v1, |s| s.clover_v2, |s| s.clover_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Doto,
        to: None,
        lines: [|s| s.doto_v1, |s| s.doto_v2, |s| s.doto_v3],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Momo,
        to: None,
        lines: [|s| s.momo_v1, |s| s.momo_v2, |s| s.momo_v3],
        face: Face::Happy,
    },
    // 마을에 남은 Naïl(0249): 다른 캐릭터로 걸을 때만 서 있다. 가운데 줄은 지금 걷는 이에게.
    Chatter {
        who: Who::Nail,
        to: Some(Character::Momo),
        lines: [|s| s.nail_v1, |s| s.nail_v_momo, |s| s.nail_v2],
        face: Face::Happy,
    },
    Chatter {
        who: Who::Nail,
        to: Some(Character::Doto),
        lines: [|s| s.nail_v1, |s| s.nail_v_doto, |s| s.nail_v2],
        face: Face::Happy,
    },
];

/// `who`가 `ch`에게 하는 늘 하는 말. 그 캐릭터의 줄이 먼저, 없으면 누구에게나 하는 줄.
pub fn chatter(who: Who, ch: Character) -> Option<&'static Chatter> {
    CHATTER
        .iter()
        .find(|c| c.who == who && c.to == Some(ch))
        .or_else(|| CHATTER.iter().find(|c| c.who == who && c.to.is_none()))
}

/// 이웃이 지금 걷는 캐릭터 본인에게 주는 무기(0246 → 0265): 아직 없으면 말을 걸 때 한 번. 무기는 그 캐릭터 것만.
/// 콩이는 길의 주인을 이긴 캐릭터에게 그 자리에서 주고(`cutscene::kongi_returns`), 다른 캐릭터는 마을에서 받는다.
pub struct Gift {
    pub who: Who,
    pub to: Character,
    pub weapon: usize,
    pub line: Tr,
}

pub const GIFTS: &[Gift] = &[
    Gift {
        who: Who::Susu,
        to: Character::Doto,
        weapon: crate::equipment::WALNUT,
        line: |s| s.susu_walnut,
    },
    Gift {
        who: Who::Kongi,
        to: Character::Momo,
        weapon: crate::equipment::PEBBLE,
        line: |s| s.kongi_pebble,
    },
    Gift {
        who: Who::Kongi,
        to: Character::Mole,
        weapon: crate::equipment::STONE_CLAW,
        line: |s| s.kongi_stone,
    },
];

pub fn gift(who: Who, ch: Character) -> Option<&'static Gift> {
    GIFTS.iter().find(|g| g.who == who && g.to == ch)
}

/// 이웃의 화자(대화창 이름·초상화).
pub fn speaker(who: Who) -> Speaker {
    match who {
        Who::Popo => Speaker::Popo,
        Who::Bori => Speaker::Bori,
        Who::Halmae => Speaker::Halmae,
        Who::Momo => Speaker::Momo,
        Who::Doto => Speaker::Doto,
        Who::Susu => Speaker::Susu,
        Who::Kongi => Speaker::Kongi,
        Who::Clover => Speaker::Clover,
        Who::Nail => Speaker::NailHome,
    }
}

/// 지금 할 한마디. `ch`는 지금 걷는 캐릭터(말을 듣는 이). 고른 `once` 줄은 기록에 남긴다(저장은 부른 쪽이).
pub fn take(p: &mut Profile, when: When, ch: Character) -> Option<&'static Line> {
    let mut best: Option<&'static Line> = None;
    for l in LINES {
        let fresh = !l.once || !p.flag(&key::line(l.id));
        if l.when == when
            && fresh
            && l.needs.iter().all(|c| c.holds(p, ch))
            && best.is_none_or(|b| l.specificity() > b.specificity())
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
    /// 포포의 배웅(0201): 이번 런의 캐릭터마다 제 줄. 예전엔 "char_pick ≥ 1"이라 도토 런에도 모모를 배웅했다.
    #[test]
    fn run_start_line_follows_picked_character() {
        let mut p = Profile::default();
        p.set(key::TUTORIAL);
        p.set(key::MOMO_BACK);
        p.set(key::DOTO_MEMORY);
        assert_eq!(
            take(&mut p, When::RunStart, Character::Momo).unwrap().id,
            "again_momo"
        );
        assert_eq!(
            take(&mut p, When::RunStart, Character::Doto).unwrap().id,
            "again_doto"
        );
        let id = take(&mut p, When::RunStart, Character::Mole).unwrap().id;
        assert!(id != "again_momo" && id != "again_doto", "{id}");
        // 도토로 끝까지 닿은 뒤 마을의 도토는 한 번 씨앗 이야기를 한다(참 엔딩 전에만).
        p.set(key::DOTO_DEEP);
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Mole).unwrap().id,
            "doto_t_deep"
        );
    }

    /// 말을 듣는 이(0264): 이웃은 눈앞의 캐릭터를 제삼자로 말하지 않고, 다른 캐릭터의 일을 그에게 하지 않는다.
    /// 도토로 걸으면 보리·모모는 "도토가 …"가 아니라 도토에게 말하고, 나일에게 하던 줄은 나일에게 남는다.
    #[test]
    fn neighbors_speak_to_the_character_in_front_of_them() {
        let doto_run = |p: &mut Profile, when| take(p, when, Character::Doto).map(|l| l.id);
        let mut p = Profile::default();
        p.set(key::DOTO_MEMORY);
        p.set(key::DOTO_DEEP);
        assert_eq!(doto_run(&mut p, When::TalkMomo), Some("momo_t_doto_self"));
        assert_eq!(
            doto_run(&mut p, When::TalkMomo),
            None,
            "도토에게 '도토가 …' 줄은 없다"
        );
        assert_eq!(doto_run(&mut p, When::TalkBori), Some("bori_t_doto_self"));
        // 같은 기록을 나일로: 나일에게 하던 줄은 그대로 남아 있다(한 번씩).
        assert_eq!(
            take(&mut p, When::TalkMomo, Character::Mole).unwrap().id,
            "momo_t_doto"
        );
        assert_eq!(
            take(&mut p, When::TalkBori, Character::Mole).unwrap().id,
            "bori_t_doto"
        );
        // 참 엔딩 뒤 꽃잎 이야기도 도토에게는 "네가 그러랬잖아".
        p.set(key::TRUE_ENDING);
        assert_eq!(doto_run(&mut p, When::TalkMomo), Some("momo_t_true_doto"));

        // 모모로: 보리는 모모에게 직접, 도토의 씨앗 이야기는 "나일한테 전해 줄래?", 나일의 독·쓰러짐 요령·반쪽 엔딩 말은 없다.
        let mut p = Profile::default();
        p.set(key::TUTORIAL);
        p.set(key::MOMO_BACK);
        p.set(key::DOTO_DEEP);
        p.set(key::VENOM);
        p.set(key::HALF_ENDING);
        p.add(key::DEATHS, 9);
        assert_eq!(
            take(&mut p, When::TalkBori, Character::Momo).unwrap().id,
            "bori_t_momo_self"
        );
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Momo).unwrap().id,
            "doto_t_deep_momo"
        );
        assert!(
            take(&mut p, When::TalkDoto, Character::Momo).is_none(),
            "독 발톱은 나일의 것"
        );
        let bori: Vec<_> = std::iter::from_fn(|| take(&mut p, When::TalkBori, Character::Momo))
            .map(|l| l.id)
            .collect();
        assert!(
            !bori.contains(&"bori_t_falls") && !bori.contains(&"bori_t_momo"),
            "{bori:?}"
        );
        assert_eq!(
            take(&mut p, When::RunStart, Character::Momo).unwrap().id,
            "again_momo"
        );
        assert_eq!(
            take(&mut p, When::RunStart, Character::Mole).unwrap().id,
            "after_half"
        );
        // 나일이 들을 줄은 나일에게 남았다.
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Mole).unwrap().id,
            "doto_t_deep"
        );
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Mole).unwrap().id,
            "doto_t_venom"
        );
        assert_eq!(
            take(&mut p, When::TalkBori, Character::Mole).unwrap().id,
            "bori_t_momo"
        );
    }

    /// 이웃의 늘 하는 말·선물(0265): 표에 줄이 있는 이웃은 캐릭터 셋 모두에게 할 말이 있고(마을의 Naïl은 다른 캐릭터에게만),
    /// 모모·도토가 듣는 줄에는 Naïl의 동사·무기(발톱·굴·솟구치기·두더지)가 없다. 선물은 받는 캐릭터 본인의 무기 종류.
    #[test]
    fn neighbor_talk_is_written_for_each_character() {
        use crate::equipment::WEAPONS;
        let ko = &crate::i18n::KO;
        let naily = ["발톱", "굴로", "솟구", "두더지", "나일이"];
        let neighbors: Vec<Who> = {
            let mut v: Vec<Who> = CHATTER.iter().map(|c| c.who).collect();
            v.dedup();
            v
        };
        for who in neighbors {
            for d in &crate::characters::ALL {
                let ch = d.id;
                if who == Who::Nail && ch == Character::Mole {
                    continue;
                }
                let c =
                    chatter(who, ch).unwrap_or_else(|| panic!("{who:?}이 {ch:?}에게 할 말이 없다"));
                for l in c.lines {
                    let t = l(ko);
                    if ch != Character::Mole {
                        assert!(
                            !naily.iter().any(|w| t.contains(w)),
                            "{who:?} → {ch:?}: Naïl의 말 {t}"
                        );
                    }
                }
            }
        }
        for g in GIFTS {
            assert_eq!(
                WEAPONS[g.weapon].kind,
                g.to.def().weapon_kind,
                "{:?}가 {:?}에게 남의 무기를 준다",
                g.who,
                g.to
            );
        }
        // 콩이는 셋 모두에게(마을에서), 수수는 Naïl에게는 첫 만남 장면이 따로 있다.
        for ch in [Character::Mole, Character::Momo] {
            assert!(gift(Who::Kongi, ch).is_some());
        }
        assert!(gift(Who::Susu, Character::Doto).is_some());
    }

    /// 캐릭터로 가른 줄은 나일이 듣는 차례를 바꾸지 않는다: 캐릭터 조건은 구체성에 세지 않는다.
    #[test]
    fn character_conditions_do_not_reorder_lines() {
        let mut p = Profile::default();
        p.set(key::CLOVER_BACK);
        p.set(key::DOTO_MEMORY);
        p.set(key::BORI_BACK);
        let order: Vec<_> = std::iter::from_fn(|| take(&mut p, When::TalkMomo, Character::Mole))
            .map(|l| l.id)
            .collect();
        assert_eq!(order, ["momo_clover", "momo_t_doto", "momo_t_bori"]);
    }

    use super::*;

    fn died(p: &mut Profile, n: u32) {
        p.add(key::DEATHS, n);
    }

    #[test]
    fn most_specific_line_wins_and_once_lines_do_not_repeat() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::Wake, Character::Mole).is_none(),
            "쓰러진 적이 없으면 말이 없다"
        );
        died(&mut p, 1);
        assert_eq!(
            take(&mut p, When::Wake, Character::Mole).unwrap().id,
            "first_wake"
        );
        assert!(
            take(&mut p, When::Wake, Character::Mole).is_none(),
            "한 번 본 줄은 다시 안 나온다"
        );
        died(&mut p, 1);
        p.set(key::BORI_BACK);
        assert_eq!(
            take(&mut p, When::Wake, Character::Mole).unwrap().id,
            "bori_fall"
        );
        died(&mut p, 1);
        assert_eq!(
            take(&mut p, When::Wake, Character::Mole).unwrap().id,
            "roots_again"
        );
        died(&mut p, 2);
        assert_eq!(
            take(&mut p, When::Wake, Character::Mole).unwrap().id,
            "assist_hint"
        );
        died(&mut p, 10);
        assert_eq!(take(&mut p, When::Wake, Character::Mole).unwrap().id, "ten");
    }

    #[test]
    fn assist_hint_is_skipped_when_assist_is_on() {
        let mut p = Profile::default();
        died(&mut p, 5);
        p.set(&key::line("first_wake"));
        p.set(&key::line("roots_again"));
        p.set(key::ASSIST);
        assert!(take(&mut p, When::Wake, Character::Mole).is_none());
    }

    #[test]
    fn momo_village_lines_follow_wins() {
        let mut p = Profile::default();
        p.set(key::TUTORIAL);
        p.set(key::BORI_BACK);
        assert_eq!(
            take(&mut p, When::Village, Character::Mole).unwrap().id,
            "village_intro"
        );
        assert!(take(&mut p, When::Village, Character::Mole).is_none());
        p.add(key::MOMO_WINS, 1);
        assert_eq!(
            take(&mut p, When::Village, Character::Mole).unwrap().id,
            "bori_momo"
        );
        p.add(key::MOMO_WINS, 2);
        p.set(key::MOMO_BACK);
        assert_eq!(
            take(&mut p, When::Village, Character::Mole).unwrap().id,
            "popo_momo"
        );
        assert!(take(&mut p, When::Village, Character::Mole).is_none());
    }

    #[test]
    fn talk_lines_follow_progress_once_then_fall_back() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::TalkBori, Character::Mole).is_none(),
            "진행이 없으면 늘 하는 말"
        );
        p.set(key::HALF_ENDING);
        p.set(key::MOMO_BACK);
        assert_eq!(
            take(&mut p, When::TalkBori, Character::Mole).unwrap().id,
            "bori_t_momo"
        );
        assert_eq!(
            take(&mut p, When::TalkBori, Character::Mole).unwrap().id,
            "bori_t_halmae"
        );
        assert!(take(&mut p, When::TalkBori, Character::Mole).is_none());
        assert_eq!(
            take(&mut p, When::TalkMomo, Character::Mole).unwrap().id,
            "momo_t_deep"
        );
    }

    /// 0053: 도토의 말은 독 발톱·참 엔딩을 따르고, 실뜨기 셋째 단계가 열리면 모모가 한마디.
    #[test]
    fn later_progress_lines_play_once() {
        let mut p = Profile::default();
        assert!(take(&mut p, When::TalkDoto, Character::Mole).is_none());
        p.set(key::VENOM);
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Mole).unwrap().id,
            "doto_t_venom"
        );
        assert!(take(&mut p, When::TalkDoto, Character::Mole).is_none());
        p.add(crate::journal::KNOTS_OPEN, 3);
        assert_eq!(
            take(&mut p, When::TalkMomo, Character::Mole).unwrap().id,
            "momo_t_knot"
        );
        p.set(key::TRUE_ENDING);
        assert_eq!(
            take(&mut p, When::TalkDoto, Character::Mole).unwrap().id,
            "doto_t_true"
        );
    }

    #[test]
    fn run_start_lines_repeat_and_follow_bori() {
        let mut p = Profile::default();
        assert!(
            take(&mut p, When::RunStart, Character::Mole).is_none(),
            "튜토리얼 전엔 오프닝이 한다"
        );
        p.set(key::TUTORIAL);
        assert_eq!(
            take(&mut p, When::RunStart, Character::Mole).unwrap().id,
            "again"
        );
        assert_eq!(
            take(&mut p, When::RunStart, Character::Mole).unwrap().id,
            "again"
        );
        p.set(key::BORI_BACK);
        assert_eq!(
            take(&mut p, When::RunStart, Character::Mole).unwrap().id,
            "again_bori"
        );
    }
}
