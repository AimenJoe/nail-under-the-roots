//! 큰 나무 아래 마을(docs/decisions/0045): 런과 런 사이의 허브. 손으로 그린 ASCII(기호는 map.rs).
//!
//! 튜토리얼을 마친 뒤 "시작"이 여기로 온다. 아래 뿌리 길(G)로 내려가면 런이 시작되고, 가장 깊은 곳에 닿으면 여기로 돌아온다.
//! - 위의 큰 나무(Y): 깬 횟수만큼 잎이 돌아온다(concept "나무가 조금씩 살아난다").
//! - 포포(K, 좌판 k): 마을 꾸미기 가게. 돈은 마을 씨앗(런이 끝나면 남은 씨앗 + 보너스).
//! - 보리(b, 할매 아래·통나무 의자 왼쪽 — 왼쪽 가게·포포 곁은 복작이고 오른쪽 끝은 구석이라, 사장 2026-10-28): 돌아온 뒤. 누룩 할매(H): 반쪽 엔딩 뒤, 실뜨기를 고른다. 빈 자리(M): 모모를 기다린다.
//! - 도토(d, 오른쪽 아래 그루터기 밑): 씨앗 자리를 기억해 낸 뒤(0052). 콩이(g, 오른쪽 위 바위 곁): 이긴 뒤(0054).
//! - 토끼풀 가족(j, 모모 자리 곁): 굴토끼 옛 굴에서 깨운 뒤(0165).
//! - Naïl(n, 큰 나무 아래 모닥불 곁 — 모모의 "큰 나무를 돌보고 있겠지"): 모모·도토로 마을을 걸을 때 — 주인공이 마을에서 사라지지 않게(0249).
//! - 꾸미기 자리 1~6: 산 것만 놓인다. 7~9: 저널 보상으로만(0048).
//! - 물웅덩이(W): 풍경(옛 털빛 비춰 보기 0048은 0272에서 뺐다).
//! - 실 고리(X, 할매 곁): 할매가 온 뒤, 실의 꿈에서 이긴 길의 주인과 다시 싸운다(0055).
//! - 텃밭(f ×3, 뿌리 길 오른쪽 아래 — 우물 곁은 물길 자리): 마을 씨앗을 심어 런마다 기른다(0304 G1, `garden.rs`). 옮겨 심은 꽃은 `garden::FLOWER_SPOTS`.

use crate::i18n::Tr;
use crate::map::{self, Parsed};

/// 위 벽 네 줄(0238): 새 큰 나무(키 약 86px, `art::TREE_*`)의 잎이 지도 위 끝에 잘리지 않게 두 줄을 더했다.
pub const ROWS: [&str; 16] = [
    "##########################",
    "##########################",
    "##########################",
    "##########################",
    "#..T.......#######..8..R.#",
    "#.....1....###Y###...g...#",
    "#.......W................#",
    "#..k.K....n.........H..p.#",
    "#..........F....b........#",
    "#....3............4...X..#",
    "#...M..j.................#",
    "#.......5..........6.T...#",
    "#..R........PG.......d.h.#",
    "#.....u........fff..9....#",
    "##########################",
    "##########################",
];

pub fn parse() -> Parsed {
    map::parse(&ROWS)
}

/// 지운 꾸미기(0269): 1 반딧불 등(큰 나무에 가려 안 보였다), 6 반딧불 병(그림이 맞지 않았다). 번호는 기록 키라
/// 표의 줄은 자리만 남기고 마을 창·지도·그림에서 뺀다. 갖고 있던 기록은 읽을 때 씨앗으로 돌려준다(`migrate_removed_decor`).
pub const REMOVED_DECOR: [usize; 2] = [1, 6];
/// 지운 꾸미기를 갖고 있던 기록에 한 번 돌려주는 마을 씨앗: 등은 산 값 8, 병은 지금 그 저널 목표가 주는 15.
pub const REMOVED_REFUND: [u32; 2] = [8, 15];
const KEY_REFUNDED: &str = "mig.0269.decor";

/// 지운 꾸미기의 값을 한 번 돌려준다(0269). 깃발은 그대로 둔다(키를 지우지 않는다, 0041).
pub fn migrate_removed_decor(p: &mut crate::profile::Profile) {
    if p.flag(KEY_REFUNDED) {
        return;
    }
    let back: u32 = REMOVED_DECOR
        .iter()
        .zip(REMOVED_REFUND)
        .filter(|(i, _)| p.flag(&key_decor(**i)))
        .map(|(_, n)| n)
        .sum();
    if back > 0 {
        p.add(KEY_SEEDS, back);
    }
    // 빈 기록에는 표시를 남기지 않는다(빈 슬롯은 빈 칸으로 보여야 한다).
    if !p.is_empty() {
        p.set(KEY_REFUNDED);
    }
}

/// 꾸미기 하나: 이름, 한 줄 설명(마을 창 자세히 칸, 0269), 마을 씨앗 값.
pub struct Decor {
    pub name: Tr,
    pub desc: Tr,
    pub price: u32,
}

pub const DECOR: [Decor; 6] = [
    Decor {
        name: |s| s.decor_1,
        desc: |s| s.decor_1_d,
        price: 5,
    },
    Decor {
        name: |s| s.decor_2,
        desc: |s| s.decor_2_d,
        price: 8,
    },
    Decor {
        name: |s| s.decor_3,
        desc: |s| s.decor_3_d,
        price: 8,
    },
    Decor {
        name: |s| s.decor_4,
        desc: |s| s.decor_4_d,
        price: 6,
    },
    Decor {
        name: |s| s.decor_5,
        desc: |s| s.decor_5_d,
        price: 10,
    },
    Decor {
        name: |s| s.decor_6,
        desc: |s| s.decor_6_d,
        price: 12,
    },
];

/// 짐 꾸리기(0259): 마을 씨앗으로 다음 런 가방에 넣어 둘 것. 꾸미기 말고도 마을 씨앗을 쓸 곳(사장 "꾸미기 외에 사용할 수 있어야 함").
/// 다음 런을 시작할 때 꺼내고 지운다(`App::unpack`). 한 런에 `max`까지.
pub struct Pack {
    pub kind: PackKind,
    pub name: Tr,
    pub desc: Tr,
    pub price: u32,
    pub max: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackKind {
    Berry,
    Worm,
    /// 씨앗 주머니: 런 씨앗 `PACK_POUCH_SEEDS`(쉼터 가게 돈). 값보다 조금 적게 — 런이 끝나면 남은 씨앗이 다시 마을 씨앗이 되니
    /// 오가며 불어나지 않게.
    Pouch,
}

pub const PACK: [Pack; 3] = [
    Pack {
        kind: PackKind::Berry,
        name: |s| s.it_berry,
        desc: |s| s.pack_berry_d,
        price: 3,
        max: 3,
    },
    Pack {
        kind: PackKind::Worm,
        name: |s| s.it_worm,
        desc: |s| s.pack_worm_d,
        price: 10,
        max: 1,
    },
    Pack {
        kind: PackKind::Pouch,
        name: |s| s.pack_pouch,
        desc: |s| s.pack_pouch_d,
        price: 6,
        max: 2,
    },
];

/// 씨앗 주머니 하나 = 런 씨앗 다섯.
pub const PACK_POUCH_SEEDS: u32 = 5;

/// 꾸려 둔 수의 기록 키.
pub fn key_pack(k: PackKind) -> &'static str {
    match k {
        PackKind::Berry => "pack.berry",
        PackKind::Worm => "pack.worm",
        PackKind::Pouch => "pack.pouch",
    }
}

/// 저널 보상으로만 얻는 꾸미기(번호 6~8, 자리 7~9, docs/decisions/0048): (옛 반딧불 병), 들꽃 화분, 실 그네.
/// 6번 반딧불 병은 0269에서 지웠다(코드 그림이 주변 그림과 맞지 않았다) — 지도 자리 '7'도 없고 어디서도 주지 않는다.
/// 번호는 기록 키(`decor.<i>`)라 줄은 자리만 남긴다(빼면 7·8번이 6·7번으로 밀린다).
pub const JOURNAL_DECOR: [Tr; 3] = [|s| s.decor_7, |s| s.decor_8, |s| s.decor_9];
/// 저널 꾸미기의 설명(6번 자리는 지운 병이라 비움).
pub const JOURNAL_DECOR_DESC: [Tr; 3] = [|_| "", |s| s.decor_8_d, |s| s.decor_9_d];
/// 마을 창에 보이는 저널 꾸미기 번호(지운 6번 병은 빼고, 0269).
pub const JOURNAL_SHOWN: [usize; 2] = [7, 8];
/// 저널 꾸미기 뒤에 더한 가게 꾸미기(0269, 번호 9부터 — 팩 그림: 돌 우물·해바라기·작은 소나무). 지도 자리는 `EXTRA_SLOTS`.
pub const EXTRA_DECOR: [Decor; 3] = [
    Decor {
        name: |s| s.decor_10,
        desc: |s| s.decor_10_d,
        price: 15,
    },
    Decor {
        name: |s| s.decor_11,
        desc: |s| s.decor_11_d,
        price: 7,
    },
    Decor {
        name: |s| s.decor_12,
        desc: |s| s.decor_12_d,
        price: 6,
    },
];
/// 더한 꾸미기의 지도 글자(`ROWS`): 우물 `u`, 해바라기 `h`, 작은 소나무 `p`. 가게·저널 꾸미기는 `1`~`9`(번호 + 1).
pub const EXTRA_SLOTS: [char; 3] = ['u', 'h', 'p'];
/// 더한 꾸미기의 첫 번호.
pub const EXTRA_FIRST: usize = DECOR.len() + JOURNAL_DECOR.len();
/// 꾸미기 전체 수(가게 + 저널 + 더한 것).
pub const DECOR_ALL: usize = EXTRA_FIRST + EXTRA_DECOR.len();

/// 지도 글자 → 꾸미기 번호.
pub fn slot_decor(c: char) -> Option<usize> {
    match c {
        '1'..='9' => Some((c as u8 - b'1') as usize),
        _ => EXTRA_SLOTS
            .iter()
            .position(|x| *x == c)
            .map(|k| EXTRA_FIRST + k),
    }
}

/// 꾸미기 이름(가게 것이든 저널 것이든).
pub fn decor_name(i: usize) -> Tr {
    if i >= EXTRA_FIRST {
        return EXTRA_DECOR[(i - EXTRA_FIRST).min(EXTRA_DECOR.len() - 1)].name;
    }
    match i.checked_sub(DECOR.len()) {
        Some(j) => JOURNAL_DECOR[j.min(JOURNAL_DECOR.len() - 1)],
        None => DECOR[i].name,
    }
}

/// 꾸미기 설명.
pub fn decor_desc(i: usize) -> Tr {
    if i >= EXTRA_FIRST {
        return EXTRA_DECOR[(i - EXTRA_FIRST).min(EXTRA_DECOR.len() - 1)].desc;
    }
    match i.checked_sub(DECOR.len()) {
        Some(j) => JOURNAL_DECOR_DESC[j.min(JOURNAL_DECOR_DESC.len() - 1)],
        None => DECOR[i].desc,
    }
}

/// 꾸미기 값(마을 씨앗). 저널 꾸미기는 None(사지 않고 받는다).
pub fn decor_price(i: usize) -> Option<u32> {
    if i >= EXTRA_FIRST {
        return EXTRA_DECOR.get(i - EXTRA_FIRST).map(|d| d.price);
    }
    DECOR.get(i).map(|d| d.price)
}

/// 마을 모습(앱이 기록에서 만들어 월드에 준다. 월드는 그리기·이웃 배치에만 쓴다).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub bori: bool,
    pub halmae: bool,
    /// 참 엔딩 뒤: 할매가 눈을 뜬다(0270, 그림 칸 4 놓아주기·5 기쁨을 숨 쉬듯 번갈아).
    pub halmae_awake: bool,
    /// 모모가 돌아와 빈 자리에 앉았다(0046).
    pub momo: bool,
    /// 도토가 씨앗 자리를 기억해 내고 마을에 왔다(0052).
    pub doto: bool,
    /// 콩이가 문을 열고 마을에 왔다(0054).
    pub kongi: bool,
    /// 토끼풀이 아기들과 마을에 왔다(0165): 모모 자리 곁.
    pub clover: bool,
    /// 무무의 실을 풀었다(0275 P5): 물웅덩이(`W`)가 다시 솟는 샘이 된다 — 무무는 오지 않고 물이 온다.
    pub spring: bool,
    /// 다른 캐릭터로 걷는다: Naïl이 큰 나무 곁에 이웃으로 선다(0249).
    pub nail: bool,
    /// 들어준 부탁의 흔적(0273 B, `quests::ALL` 번호): 포포 샘 그릇·보리 피리·모모 꽃밭·큰 나무 밑동의 깍정이·모닥불 곁 풀 방석·콩이 버섯 등·토끼풀 솜털 잠자리.
    pub traces: [bool; 7],
    /// 이웃 등불(0304): 셋째 이야기까지 들은 이웃 곁에 등불이 걸린다(`lanterns::ALL` 번호).
    pub lamps: [bool; crate::lanterns::ALL.len()],
    /// 텃밭 칸(0304 G1)과 마을에 옮겨 심은 꽃 수.
    pub plots: [crate::garden::Plot; crate::garden::PLOTS],
    pub blooms: [Option<crate::garden::Bloom>; crate::garden::FLOWER_SPOTS.len()],
    pub decor: [bool; DECOR_ALL],
    /// 큰 나무의 잎(0 = 거의 없음 … 4 = 꽃).
    pub growth: u8,
    /// 맡긴 반딧불로 깨울 수 있는 재주가 있다: 밑동이 빛난다(0061).
    pub tree_ready: bool,
    /// 맡긴(아직 쓰지 않은) 반딧불의 단계 0..4(`glow_level`): 돌 우물 위에 그만큼 반딧불이 떠다닌다(0269, 요구 2.9 A).
    pub glow: u8,
}

impl State {
    /// 기록 → 마을 모습(0307 R41). 마을의 모든 칸은 기록에서 나온다 — 기록이 바뀌면 이것으로 다시 만든다(`App::sync_village`),
    /// 칸을 하나씩 손으로 맞추지 않는다. `walking`: 지금 걷는 캐릭터(그 이웃 자리는 빈다 — 플레이어가 그 자신이다, 0249).
    pub fn of(p: &crate::profile::Profile, walking: crate::equipment::Character) -> State {
        use crate::equipment::Character;
        use crate::profile::key;
        let mut decor = [false; DECOR_ALL];
        for (i, d) in decor.iter_mut().enumerate() {
            *d = p.flag(&key_decor(i));
        }
        State {
            bori: p.flag(key::BORI_BACK),
            halmae: p.flag(key::HALF_ENDING) || p.flag(key::TRUE_ENDING),
            // 참 엔딩 뒤 할매는 눈을 뜬다(0270): 그림 칸 4·5.
            halmae_awake: p.flag(key::TRUE_ENDING),
            momo: p.flag(key::MOMO_BACK) && walking != Character::Momo,
            doto: p.flag(key::DOTO_MEMORY) && walking != Character::Doto,
            kongi: p.flag(key::KONGI_BACK),
            clover: p.flag(key::CLOVER_BACK),
            spring: p.flag(key::MUMU_FREED),
            traces: std::array::from_fn(|i| crate::quests::done(p, i)),
            lamps: std::array::from_fn(|i| crate::lanterns::level(p, i) >= 3),
            plots: crate::garden::plots(p),
            blooms: crate::garden::spots(p),
            // 거꾸로 Naïl은 다른 캐릭터로 걸을 때 큰 나무 곁에 남는다(0249).
            nail: walking != Character::Mole,
            decor,
            // 토끼풀 가족이 돌아오면(0165) 큰 나무의 잎이 한 단계 더.
            growth: growth(
                crate::characters::ALL
                    .iter()
                    .map(|d| p.count(&key::clears(d.id)))
                    .sum::<u32>()
                    + u32::from(p.flag(key::CLOVER_BACK)),
            ),
            tree_ready: crate::talents::any_affordable(p),
            glow: glow_level(p.count(crate::talents::KEY_LIGHT)),
        }
    }
}

/// 맡긴 반딧불 수 → 우물 위 반딧불 단계(0 = 없음, 4 = 많이). 큰 나무를 다 깨운 뒤에도 쌓이는 빛이 마을에 보이게(0269 2.9 A).
/// 기준은 재주 한 칸 값(120~495, 0246)과 런 하나에 모이는 양(약 240, 0236 — 지금은 끝까지 간 런 ≈ 285, 0305)에서: 조금·한 런·나무 한 칸 넘게·많이.
pub fn glow_level(banked: u32) -> u8 {
    match banked {
        0 => 0,
        1..=119 => 1,
        120..=399 => 2,
        400..=999 => 3,
        _ => 4,
    }
}

/// 무무가 풀린 뒤(0275, 사장 2026-11-04 "마을에 물길 하나" → "우물 옆쪽에"): 돌 우물(`u`) 오른쪽 곁에서 넘친 물이
/// 동쪽으로 `BROOK_LEN`칸 흐르고, 마지막 칸은 아래(남쪽 땅 밑)로 든다. (칸 가운데, 흐르는 쪽).
/// 마을에서는 밀지 않는다(`World::stream_push_player` — 쉬는 곳). 그림은 굴과 같은 굽기(`Baker::with_water`)와 `draw_stream`.
pub fn brook(p: &Parsed) -> Vec<(macroquad::math::Vec2, macroquad::math::Vec2)> {
    use macroquad::math::vec2;
    let Some((ux, uy)) = p
        .marks
        .iter()
        .find(|(m, _)| *m == 'u')
        .map(|(_, at)| ((at.x / map::TILE) as usize, (at.y / map::TILE) as usize))
    else {
        return Vec::new();
    };
    let last = ux + BROOK_LEN;
    (ux + 1..=last)
        .map(|x| {
            let dir = if x == last {
                vec2(0.0, 1.0)
            } else {
                vec2(1.0, 0.0)
            };
            (map::cell_center(x, uy), dir)
        })
        .collect()
}
/// 우물 곁 물길의 칸 수 — 우물에서 넘친 작은 도랑(여섯은 과했다, 사장 2026-11-04 "조그만 짧게").
pub const BROOK_LEN: usize = 3;

/// 기록 키.
pub fn key_decor(i: usize) -> String {
    format!("decor.{i}")
}
pub const KEY_SEEDS: &str = "village_seeds";

/// 깬 횟수 → 나무 잎 단계.
pub fn growth(clears: u32) -> u8 {
    clears.min(4) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Cell;

    /// 마을 모습은 기록의 함수(0307 R41): 걷는 캐릭터의 자리는 비고, Naïl은 다른 캐릭터로 걸을 때만 이웃으로 선다.
    #[test]
    fn state_leaves_the_walking_characters_seat_empty() {
        use crate::equipment::Character;
        use crate::profile::{key, Profile};
        let mut p = Profile::default();
        for k in [
            key::BORI_BACK,
            key::MOMO_BACK,
            key::DOTO_MEMORY,
            key::KONGI_BACK,
        ] {
            p.set(k);
        }
        let v = State::of(&p, Character::Mole);
        assert!(v.bori && v.momo && v.doto && v.kongi && !v.nail);
        assert!(!v.halmae && !v.clover && !v.spring);
        let v = State::of(&p, Character::Momo);
        assert!(
            !v.momo && v.doto && v.nail,
            "모모로 걸으면 모모 자리가 비고 Naïl이 선다"
        );
        let v = State::of(&p, Character::Doto);
        assert!(v.momo && !v.doto && v.nail);
        // 할매: 반쪽 엔딩이면 마을에, 참 엔딩이면 눈을 뜬다.
        p.set(key::HALF_ENDING);
        let v = State::of(&p, Character::Mole);
        assert!(v.halmae && !v.halmae_awake);
        p.set(key::TRUE_ENDING);
        assert!(State::of(&p, Character::Mole).halmae_awake);
    }

    /// 맡긴 반딧불이 바뀌면 우물 위 반딧불 단계와 큰 나무 밑동 빛이 따라온다 — 재주를 깨우거나 등불을 건넨 뒤 다시 만들면 준다.
    #[test]
    fn state_follows_banked_light() {
        use crate::equipment::Character;
        use crate::profile::Profile;
        use crate::talents::KEY_LIGHT;
        let mut p = Profile::default();
        let v = State::of(&p, Character::Mole);
        assert_eq!((v.glow, v.tree_ready), (0, false));
        p.set_count(KEY_LIGHT, 450);
        let v = State::of(&p, Character::Mole);
        assert_eq!(v.glow, glow_level(450));
        assert!(v.tree_ready, "첫 칸 값(120)은 넘는다");
        p.set_count(KEY_LIGHT, 30);
        let v = State::of(&p, Character::Mole);
        assert_eq!(
            (v.glow, v.tree_ready),
            (1, false),
            "쓰고 나면 단계가 내려가고 밑동 빛이 꺼진다"
        );
    }

    /// 꾸미기·부탁의 흔적·등불(셋째 이야기 뒤)·텃밭·화단이 기록 그대로다.
    #[test]
    fn state_mirrors_decor_traces_lamps_and_garden() {
        use crate::equipment::Character;
        use crate::garden::{self, Crop, Plot};
        use crate::profile::Profile;
        let mut p = Profile::default();
        let v = State::of(&p, Character::Mole);
        assert!(v.decor.iter().all(|d| !d) && v.traces.iter().all(|t| !t));
        assert!(v.lamps.iter().all(|l| !l) && v.blooms.iter().all(|b| b.is_none()));
        p.set(&key_decor(2));
        p.set(crate::quests::DONE_KEYS[crate::quests::DOTO]);
        p.set_count(crate::lanterns::ALL[0].key, 2);
        p.set_count(crate::lanterns::ALL[1].key, 3);
        p.set_count(KEY_SEEDS, 20);
        garden::plant(&mut p, 1, Crop::Berry).unwrap();
        let v = State::of(&p, Character::Mole);
        assert!(v.decor[2] && !v.decor[0]);
        assert!(v.traces[crate::quests::DOTO] && !v.traces[crate::quests::POPO]);
        assert!(
            !v.lamps[0] && v.lamps[1],
            "등불은 셋째 이야기까지 들은 이웃 곁에만"
        );
        assert!(matches!(v.plots[1], Plot::Growing { .. }) && v.plots[0] == Plot::Empty);
        assert_eq!(v.plots, garden::plots(&p));
        assert_eq!(v.blooms, garden::spots(&p));
    }

    #[test]
    fn village_has_gate_neighbors_tree_and_six_decor_slots() {
        let p = parse();
        assert_eq!(p.grid.find(Cell::Gate).len(), 1);
        assert!(p.enemies.is_empty());
        for m in [
            'K', 'b', 'M', 'H', 'Y', 'W', 'd', 'g', 'j', 'n', 'X', '1', '3', '4', '5', '6', '8',
            '9',
        ] {
            // 'j'(토끼풀 가족, 0165)가 마커 목록에서 빠져 마을에 나오지 않았다 — 이웃 자리는 모두 여기서 본다.
            assert!(p.marks.iter().any(|(c, _)| *c == m), "{m}");
        }
        // 이웃 자리 바로 아래는 걸을 수 있다(말 걸 수 있게).
        for (c, at) in &p.marks {
            if matches!(c, 'K' | 'b' | 'H' | 'M' | 'n') {
                let (cx, cy) = ((at.x / 16.0) as usize, (at.y / 16.0) as usize);
                assert_eq!(p.grid.get(cx, cy + 1), Cell::Floor, "{c}");
            }
        }
        // 나무 자리는 막혀 있다.
        let y = p.marks.iter().find(|(c, _)| *c == 'Y').unwrap().1;
        assert_eq!(
            p.grid.get((y.x / 16.0) as usize, (y.y / 16.0) as usize),
            Cell::Wall
        );
    }

    /// 우물 곁 물길(0275): 우물 바로 오른쪽에서 시작해 걸을 수 있는 빈 칸(다른 자리 표시 없음)만 지나 남쪽 벽 위에서 끝난다.
    #[test]
    fn brook_runs_beside_the_well() {
        let p = parse();
        let b = brook(&p);
        assert_eq!(b.len(), BROOK_LEN);
        let u = p.marks.iter().find(|(c, _)| *c == 'u').unwrap().1;
        assert!((b[0].0 - u).length() <= 16.0, "우물 바로 곁에서 시작");
        for (c, _) in &b {
            let (x, y) = ((c.x / 16.0) as usize, (c.y / 16.0) as usize);
            assert_eq!(p.grid.get(x, y), Cell::Floor);
            assert!(
                !p.marks.iter().any(|(_, at)| (*at - *c).length() < 1.0),
                "자리 표시 위로 흐르지 않는다"
            );
        }
        let (last, dir) = *b.last().unwrap();
        let (x, y) = ((last.x / 16.0) as usize, (last.y / 16.0) as usize);
        assert_eq!(dir.y, 1.0);
        assert_eq!(p.grid.get(x, y + 1), Cell::Wall, "남쪽 땅 밑으로 든다");
    }

    #[test]
    fn growth_caps_at_four() {
        assert_eq!(growth(0), 0);
        assert_eq!(growth(2), 2);
        assert_eq!(growth(99), 4);
    }
}

#[cfg(test)]
mod removed_tests {
    /// 0269: 지운 꾸미기(반딧불 등 1·반딧불 병 6)의 지도 자리는 없다.
    #[test]
    fn removed_decor_have_no_map_slot() {
        let p = super::parse();
        assert!(!p.marks.iter().any(|(c, _)| *c == '2' || *c == '7'));
    }
}

#[cfg(test)]
mod refund_tests {
    use super::*;

    #[test]
    fn removed_decor_are_refunded_once() {
        let mut p = crate::profile::Profile::default();
        p.set(&key_decor(1));
        p.set(&key_decor(6));
        p.add(KEY_SEEDS, 2);
        migrate_removed_decor(&mut p);
        assert_eq!(p.count(KEY_SEEDS), 2 + 8 + 15);
        migrate_removed_decor(&mut p);
        assert_eq!(p.count(KEY_SEEDS), 2 + 8 + 15, "두 번 돌려주지 않는다");
        let mut q = crate::profile::Profile::default();
        migrate_removed_decor(&mut q);
        assert!(q.is_empty());
    }
}

#[cfg(test)]
mod glow_tests {
    /// 0269 2.9 A: 맡긴 빛이 많을수록 우물 위 반딧불이 는다 — 거꾸로 가지 않는다.
    #[test]
    fn well_fireflies_grow_with_banked_light() {
        assert_eq!(super::glow_level(0), 0);
        assert_eq!(super::glow_level(240), 2);
        assert_eq!(super::glow_level(5000), 4);
        let mut last = 0;
        for n in (0..3000).step_by(7) {
            let g = super::glow_level(n);
            assert!(g >= last);
            last = g;
        }
    }
}
