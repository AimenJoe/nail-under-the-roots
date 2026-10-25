//! 다국어(docs/decisions/0013, 0014). 플레이어가 보는 글은 전부 여기 언어별 표 한 곳에 있다.
//!
//! - 키는 `Strings`의 필드 이름. 한 언어에 필드가 빠지면 컴파일이 안 된다.
//! - 숫자가 들어가는 글은 `{0}`, `{1}` 자리표시자 + `fill()`. 언어마다 어순이 달라도 된다. 문장을 조각으로 잇지 않는다.
//! - 다른 모듈은 글 대신 `Tr`(표에서 글을 고르는 함수)을 들고 있다가, 그릴 때 `tr()`로 꺼낸다.
//!   월드 로직은 지금 언어를 모른다.
//! - 언어마다 폰트가 다르다(`Lang::font_file`). 한국어·영어는 Galmuri, 일본어·중국어는 Ark Pixel에서 만든 서브셋
//!   (tools/fonts/build_cjk.py — 표를 고치면 다시 돌린다).
//! - 개발 전용 화면(시나리오 목록, 디버그 오버레이)은 번역하지 않고 기본 폰트로 찍는다.
//! - 개발 빌드에는 가짜 언어 "Pseudo"(영어를 악센트 + 40% 늘림)가 있다. 넘치는 칸·번역 안 된 글을 미리 찾는다.
//! - 테스트: 모든 언어의 모든 글이 그 언어 폰트에 있는 글자로만 되어 있는지, 자리표시자가 언어끼리 같은지.

use std::sync::atomic::{AtomicU8, Ordering};

/// 표에서 글 하나를 고르는 함수. 데이터 표(무기·퀘스트·컷씬)가 글 대신 이걸 든다.
pub type Tr = fn(&Strings) -> &'static str;

/// 한국어·영어 폰트(그리고 개발 화면·가짜 언어).
pub const BASE_FONT: &str = "fonts/Galmuri11-Condensed.ttf";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ko,
    En,
    Ja,
    ZhHans,
    #[cfg(feature = "dev")]
    Pseudo,
}

impl Lang {
    /// 설정에서 넘기는 순서.
    #[cfg(not(feature = "dev"))]
    pub const ALL: &'static [Lang] = &[Lang::Ko, Lang::En, Lang::Ja, Lang::ZhHans];
    #[cfg(feature = "dev")]
    pub const ALL: &'static [Lang] = &[Lang::Ko, Lang::En, Lang::Ja, Lang::ZhHans, Lang::Pseudo];

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::Ko => &KO,
            Lang::En => &EN,
            Lang::Ja => &JA,
            Lang::ZhHans => &ZH_HANS,
            #[cfg(feature = "dev")]
            Lang::Pseudo => pseudo(),
        }
    }

    /// 이 언어를 찍는 폰트(assets 기준 경로).
    pub fn font_file(self) -> &'static str {
        match self {
            Lang::Ja => "fonts/NailCJK-ja.ttf",
            Lang::ZhHans => "fonts/NailCJK-zh-Hans.ttf",
            _ => BASE_FONT,
        }
    }

    /// BCP 47 태그("ko-KR", "en-US", "zh-Hans-CN", "en_US.UTF-8")가 우리가 가진 언어면 그 언어.
    /// 중국어는 번체(zh-Hant, zh-TW·HK·MO)를 아직 갖고 있지 않으므로 간체로 바꾸지 않고 다음 선호 언어로 넘긴다.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let lower = tag.to_ascii_lowercase();
        let mut parts = lower.split(['-', '_', '.', '@']);
        let primary = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();
        match primary {
            "ko" => Some(Lang::Ko),
            "en" => Some(Lang::En),
            "ja" => Some(Lang::Ja),
            "zh" if rest
                .iter()
                .any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo")) =>
            {
                None
            }
            "zh" => Some(Lang::ZhHans),
            _ => None,
        }
    }

    /// 기기의 선호 언어 목록(앞이 우선)에서 우리가 가진 첫 언어. 하나도 없으면 영어.
    pub fn pick<S: AsRef<str>>(preferred: impl IntoIterator<Item = S>) -> Lang {
        preferred
            .into_iter()
            .find_map(|t| Lang::from_tag(t.as_ref()))
            .unwrap_or(Lang::En)
    }

    /// 설정에서 좌우로 넘긴다.
    pub fn cycle(self, d: i8) -> Lang {
        let n = Self::ALL.len() as i32;
        let i = Self::ALL.iter().position(|l| *l == self).unwrap_or(0) as i32;
        Self::ALL[(i + d as i32).rem_euclid(n) as usize]
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn lang() -> Lang {
    Lang::ALL[CURRENT.load(Ordering::Relaxed) as usize % Lang::ALL.len()]
}

pub fn set_lang(l: Lang) {
    let i = Lang::ALL.iter().position(|x| *x == l).unwrap_or(0);
    CURRENT.store(i as u8, Ordering::Relaxed);
}

/// 지금 언어의 표.
pub fn s() -> &'static Strings {
    lang().strings()
}

/// `Tr`을 지금 언어로.
pub fn tr(f: Tr) -> &'static str {
    f(s())
}

/// `{0}`, `{1}` …을 차례로 채운다. 숫자 모양(소수점 자리 등)은 부르는 쪽이 정한다.
pub fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = template.to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &a.to_string());
    }
    out
}

/// 가짜 언어: 영어를 악센트 글자로 바꾸고 40% 늘린다. 자리표시자는 그대로.
/// 늘어난 글이 칸을 넘치면 실제 번역(독일어 등 +30~40%)에서도 넘친다. 악센트가 없는 글은 표를 거치지 않은 글이다.
#[cfg(any(test, feature = "dev"))]
pub fn pseudo_text(s: &str) -> String {
    let mut out = String::from("[");
    let mut in_ph = false;
    for c in s.chars() {
        in_ph = match c {
            '{' => true,
            '}' => false,
            _ => in_ph,
        };
        out.push(if in_ph {
            c
        } else {
            match c {
                'a' => 'à',
                'e' => 'é',
                'i' => 'î',
                'o' => 'ö',
                'u' => 'ü',
                'y' => 'ý',
                'c' => 'ç',
                'n' => 'ñ',
                'A' => 'Å',
                'E' => 'É',
                'I' => 'Î',
                'O' => 'Ö',
                'U' => 'Ü',
                'C' => 'Ç',
                'N' => 'Ñ',
                _ => c,
            }
        });
    }
    let pad = (s.chars().count() * 2).div_ceil(5);
    out.extend(std::iter::repeat_n('~', pad));
    out.push(']');
    out
}

#[cfg(feature = "dev")]
fn pseudo() -> &'static Strings {
    static P: std::sync::OnceLock<Strings> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let mut p = EN.map(|s| Box::leak(pseudo_text(s).into_boxed_str()));
        p.lang_name = "Pseudo";
        p
    })
}

/// 기기 언어로 시작 언어를 고른다: 선호 목록에서 우리가 가진 첫 언어, 없으면 영어.
/// 웹은 브라우저의 navigator.languages(index.html의 nail_locale 플러그인), 그 밖은 OS 선호 언어(sys-locale).
pub fn detect() -> Lang {
    Lang::pick(preferred_tags())
}

/// 웹: JS가 i번째 선호 언어 태그의 k번째 네 글자를 u32 하나로 접어 준다(문자열 대신 정수, ASCII만).
/// 0이면 끝. 없으면(옛 index.html) miniquad가 빈 함수를 끼워 넣어 0이 된다.
#[cfg(target_arch = "wasm32")]
fn preferred_tags() -> Vec<String> {
    extern "C" {
        fn nail_locale(i: u32, k: u32) -> u32;
    }
    let mut tags = Vec::new();
    for i in 0..8 {
        let mut tag = Vec::new();
        for k in 0..6 {
            // SAFETY: 정수 둘을 받고 정수를 돌려주는 JS 함수(web/index.html).
            let word = unsafe { nail_locale(i, k) };
            if word == 0 {
                break;
            }
            tag.extend(word.to_be_bytes().into_iter().filter(|b| *b != 0));
        }
        if tag.is_empty() {
            break;
        }
        tags.push(String::from_utf8_lossy(&tag).into_owned());
    }
    tags
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_locale_crate_version() -> u32 {
    2
}

#[cfg(not(target_arch = "wasm32"))]
fn preferred_tags() -> Vec<String> {
    sys_locale::get_locales().collect()
}

macro_rules! strings {
    ($($key:ident),* $(,)?) => {
        /// 언어 하나의 글 전부.
        // 배포 빌드(dev 끔)에서는 개발 메뉴 이름(title_dev)을 쓰지 않는다.
        #[cfg_attr(not(feature = "dev"), allow(dead_code))]
        pub struct Strings {
            $(pub $key: &'static str,)*
        }

        impl Strings {
            /// (키, 글) 전부. 테스트용.
            #[cfg(test)]
            pub fn all(&self) -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($key), self.$key)),*]
            }

            /// 이 언어가 쓰는 글자 전부(겹침 없이). 글자 아틀라스를 미리 굽는 데 쓴다(ui::Ui::warm).
            pub fn glyphs(&self) -> String {
                let mut v: Vec<char> = [$(self.$key),*].concat().chars().collect();
                v.extend("0123456789%/:.,·…!?()[]+-▶▼▲→←♥♡".chars());
                v.sort_unstable();
                v.dedup();
                v.into_iter().filter(|c| !c.is_whitespace()).collect()
            }

            /// 모든 글을 바꾼 새 표(가짜 언어용).
            #[cfg(feature = "dev")]
            fn map(&self, f: impl Fn(&'static str) -> &'static str) -> Strings {
                Strings { $($key: f(self.$key),)* }
            }
        }
    };
}

strings! {
    lang_name,
    // 인트로·타이틀
    intro_1, intro_2, intro_3, skip_hint,
    title_start, title_settings, title_dev, title_controls, version,
    // 설정
    settings, set_sound, set_shake, set_debug, set_language, set_back, on, off, settings_hint,
    // 일시정지
    tab_menu, tab_status, tab_bag, tab_journal, tab_map, tab_hint,
    pm_resume, pm_restart, pm_settings, pm_title,
    // 상태
    hp, this_run, run_stats, combo_stats, bare_hands, bare_hands_desc,
    burrow, burrow_stats, special, glowbugs,
    // 가방
    bag_combo, equipped, equip_hint, empty_slot, empty_slot_desc, slots,
    // 지도
    map_floor, map_proto, map_note,
    // 저널
    q_roots, q_roots_desc, q_claws, q_claws_desc, q_ambush, q_ambush_desc, q_survive, q_survive_desc,
    // 환경
    biome_meadow, biome_roots, biome_deep,
    // 무기
    w_sprout, w_sprout_desc, w_mole, w_mole_desc, w_stone, w_stone_desc, w_venom, w_venom_desc,
    gift_title,
    // 튜토리얼
    tut_move, tut_pickup, tut_attack, tut_burrow, tut_emerge, tut_outro,
    // 컷씬
    cut_skip, op_1, op_2, op_3, op_4, op_5,
    // 사망
    dead, dead_hint,
    // 장신구·가방·필살기(docs/decisions/0017, 0018)
    c_shell, c_shell_desc, c_whisker, c_whisker_desc, c_soil, c_soil_desc, c_flint, c_flint_desc, c_amulet, c_amulet_desc, it_berry, it_berry_desc, it_worm, it_worm_desc, toast_heart, toast_revive, quick_hint, special_name, special_desc, charms_label, charms_none, stat_attack, charm_equip, charm_unequip, bag_items, charm_slots,
    // 모험(docs/decisions/0019)
    toast_save, toast_respawn, toast_boss, gift_charm, map_field, map_dungeon, map_legend, toast_parry,
    // 적 종류·도감·게이지(docs/decisions/0021)
    gauge_rage, gauge_ki, mob_field_mouse, mob_field_mouse_lore, mob_field_mouse_atk, mob_field_mouse_tip, mob_house_mouse, mob_house_mouse_lore, mob_house_mouse_atk, mob_house_mouse_tip, mob_sewer_rat, mob_sewer_rat_lore, mob_sewer_rat_atk, mob_sewer_rat_tip, mob_chipmunk, mob_chipmunk_lore, mob_chipmunk_atk, mob_chipmunk_tip, mob_red_squirrel, mob_red_squirrel_lore, mob_red_squirrel_atk, mob_red_squirrel_tip, mob_dummy, mob_dummy_lore, mob_dummy_atk, mob_dummy_tip, w_hp_1, w_hp_2, w_hp_3, w_hp_4, w_pow_1, w_pow_2, w_spd_1, w_spd_2, w_spd_3, w_spd_4,
    // 상태 탭·도감 탭
    tab_bestiary, stat_power, stat_aspd, stat_dig_cd, stat_gauge, stat_weapon, seeds, fmt_secs, fmt_pct, fmt_hp, wallet_note, best_unknown, best_unseen, best_hp, best_pow, best_spd, best_atk, best_tip, best_count,
    // 조작 설정(docs/decisions/0022)
    set_controls, act_attack, act_dodge, act_special, act_use_item, act_item_prev, act_item_next, act_map, ctl_key, ctl_pad, ctl_reset, ctl_wait, ctl_hint, ctl_fixed,
    // 튜토리얼: 라쿤 포포(docs/decisions/0023)
    popo_name, popo_hi, popo_hi2, popo_pickup, popo_combo, popo_burrow, popo_emerge, popo_parry, popo_parry_skip, popo_special, popo_item, popo_item2, popo_menu, popo_save, popo_bye, tut_step, tut_parry, tut_special, tut_item,
    // 스테이지 고리: 튜토리얼 보스·쉼터·길 지도·가게 (docs/decisions/0030~0032)
    mob_chief, mob_chief_lore, mob_chief_atk, mob_chief_tip, toast_chief, toast_camp, toast_spring, toast_goal, tut_boss, popo_glade, bori_name, chief_intro_1, chief_intro_2, chief_beaten_1, chief_beaten_2, chief_beaten_3, chief_beaten_4, chief_beaten_5, camp_1, camp_2, camp_3, camp_again, bori_1, bori_2, bori_3, seat_1, node_burrow, node_burrow_desc, node_nest, node_nest_desc, node_cache, node_cache_desc, node_spring, node_spring_desc, node_boss, node_boss_desc, node_unknown, node_unknown_desc, route_title, route_hint, route_hint_back, shop_title, shop_seeds, shop_hi, shop_poor, shop_full, shop_thanks, shop_price, shop_heart, shop_heart_desc, shop_hint, ia_shop, ia_talk, ia_look, ia_route, map_camp,
}

pub static KO: Strings = Strings {
    lang_name: "한국어",
    intro_1: "큰 나무가 시들어 간다.",
    intro_2: "뿌리가 땅속 이웃들을 이상하게 바꿔 놓았다.",
    intro_3: "아기 두더지 Naïl은 원인을 찾아 뿌리를 따라 내려간다.",
    skip_hint: "확인: 건너뛰기",
    title_start: "시작",
    title_settings: "설정",
    title_dev: "개발",
    title_controls: "상하 이동 · 확인(Enter/A) · 뒤로(Esc/B)",
    version: "프로토타입 v0.1",
    settings: "설정",
    set_sound: "소리",
    set_shake: "화면 흔들림",
    set_debug: "디버그 정보",
    set_language: "언어",
    set_back: "돌아가기",
    on: "켬",
    off: "끔",
    settings_hint: "좌우로 값 바꾸기 · 웹에서는 이 브라우저에 저장된다",
    tab_menu: "메뉴",
    tab_status: "상태",
    tab_bag: "가방",
    tab_journal: "저널",
    tab_map: "지도",
    tab_hint: "Q/E · LB/RB: 탭",
    pm_resume: "계속하기",
    pm_restart: "다시 시작",
    pm_settings: "설정",
    pm_title: "타이틀로",
    hp: "체력",
    this_run: "이번 런",
    run_stats: "웨이브 {0}  처치 {1}  시간 {2}",
    combo_stats: "{0}연타. 마지막 타 {1}s, 넉백 {2}",
    bare_hands: "맨손",
    bare_hands_desc: "아직 발톱이 없다. 공격할 수 없다.",
    burrow: "굴파기",
    burrow_stats: "땅속 {0}s 무적, 솟구침 반경 {1}px, 쿨다운 {2}s",
    special: "특기",
    glowbugs: "반딧불",
    bag_combo: "{0}연타 · 마지막 타 피해 {1}",
    equipped: "장착 중",
    equip_hint: "확인: 장착",
    empty_slot: "비어 있음",
    empty_slot_desc: "아이템은 2단계(방 클리어 보상 선택)에서 들어온다. 지금은 틀만 있다.",
    slots: "{0} / {1} 칸",
    map_floor: "1층 · {0}",
    map_proto: "프로토타입: 방 하나",
    map_note: "층 구조·방 연결은 3단계(런 구조)에서. 지금은 현재 방만.",
    q_roots: "뿌리를 따라",
    q_roots_desc: "시들어 가는 나무의 뿌리를 따라 더 깊이 내려간다. (층 구조는 3단계에서)",
    q_claws: "발톱 길들이기",
    q_claws_desc: "발톱 3연타로 적을 세 마리 처치한다. 3타째가 묵직한지 느껴 볼 것.",
    q_ambush: "기습",
    q_ambush_desc: "굴파기로 땅속에 들어갔다가 솟구쳐서 적을 처치한다.",
    q_survive: "버티기",
    q_survive_desc: "웨이브 3까지 살아남는다.",
    biome_meadow: "꽃 핀 입구",
    biome_roots: "뿌리 지대",
    biome_deep: "어두운 심층",
    w_sprout: "새싹 발톱",
    w_sprout_desc: "가벼움. 4연타가 짧고 빠르게 이어진다. 한 방은 약하다.",
    w_mole: "두더지 발톱",
    w_mole_desc: "기본. 3연타, 세 번째가 묵직하다. 언제든 굴파기로 끊는다.",
    w_stone: "바위 발톱",
    w_stone_desc: "무거움. 2연타, 넓고 세게 밀친다. 휘두른 뒤에야 굴파기로 끊는다.",
    w_venom: "독 발톱",
    w_venom_desc: "맞은 적에게 독을 건다. 마지막 타의 독이 길다. 독은 경직 없이 체력만 깎는다.",
    gift_title: "새 발톱을 얻었다!",
    tut_move: "스틱 / WASD: 움직여 보자",
    tut_pickup: "반짝이는 것에 다가가 보자",
    tut_attack: "{0} 세 번: 세 번째가 묵직하다",
    tut_burrow: "{0}: 굴파기. 땅속에서는 맞지 않는다",
    tut_emerge: "땅속에서 적 아래로 가서 다시 {0}: 솟구치기",
    tut_outro: "뿌리를 따라 내려가자",
    cut_skip: "뒤로(Esc/B): 건너뛰기",
    op_1: "…뿌리가 여기까지 내려와 있어. 흙이 다르게 느껴져.",
    op_2: "누구야. 여기는 우리 집이었어.",
    op_3: "나무가… 나무가 우리를 이렇게 만들었어. 돌아가.",
    op_4: "나는 원인을 찾으러 가는 길이야. 길을 비켜 줘.",
    op_5: "(발톱을 세운다.)",
    dead: "쓰러졌다",
    dead_hint: "확인: 다시 시작   메뉴: 일시정지",
    c_shell: "굳은 껍질",
    c_shell_desc: "최대 체력 +1.",
    c_whisker: "날쌘 수염",
    c_whisker_desc: "공격 속도 +20%.",
    c_soil: "촉촉한 흙",
    c_soil_desc: "굴파기 대기 시간 -40%.",
    c_flint: "날 선 부싯돌",
    c_flint_desc: "공격력 +1, 대신 공격 속도 -10%.",
    c_amulet: "뿌리 부적",
    c_amulet_desc: "SP가 1.5배 빨리 찬다.",
    it_berry: "산딸기",
    it_berry_desc: "체력 1 회복.",
    it_worm: "쥬시 지렁이",
    it_worm_desc: "먹으면 체력 가득. 들고 있다가 쓰러지면 저절로 먹고 일어난다.",
    toast_heart: "뿌리 심장! 최대 체력이 늘었다.",
    toast_revive: "쥬시 지렁이를 먹고 다시 일어났다!",
    quick_hint: "{0} 먹기",
    special_name: "드릴 돌진",
    special_desc: "{0}가 가득 차면 {1}. 부아아앙! 화면 끝까지 연타하며 돌진, 적을 벽까지 밀어 박으면 쾅!",
    charms_label: "장신구",
    charms_none: "없음",
    stat_attack: "공격 +{0} · 공격 속도 {1}%",
    charm_equip: "확인: 끼기",
    charm_unequip: "확인: 빼기",
    bag_items: "산딸기 {0}/{1} · 지렁이 {2}/{3}",
    charm_slots: "장신구 {0}/{1}",
    toast_save: "저장 지점: 쓰러지면 여기서 다시 선다.",
    toast_respawn: "저장 지점에서 다시 일어났다. 반딧불을 조금 잃었다.",
    toast_boss: "굴의 주인을 물리쳤다! 출구가 열렸다.",
    gift_charm: "새 장신구를 얻었다!",
    map_field: "들판 · {0}",
    map_dungeon: "굴 {0}층 · {1}",
    map_legend: "흰 점: 나 · 초록: 저장 · 검정: 굴",
    toast_parry: "튕겨냈다!",
    gauge_rage: "분노",
    gauge_ki: "기",
    mob_field_mouse: "들쥐",
    mob_field_mouse_lore: "들판 굴에 살던 수다쟁이. 뿌리 독에 물들어 눈이 붉게 빛난다.",
    mob_field_mouse_atk: "부르르 떨다가 곧게 돌진한다.",
    mob_field_mouse_tip: "떨 때 비키거나, 닿기 직전에 휘둘러 튕겨낸다.",
    mob_house_mouse: "생쥐",
    mob_house_mouse_lore: "작고 겁 많은 이웃. 지금은 떼로 몰려다니며 아무거나 문다.",
    mob_house_mouse_atk: "떼로 다가와 짧게 덤빈다.",
    mob_house_mouse_tip: "한 대면 쓰러진다. 솟구치기로 한꺼번에.",
    mob_sewer_rat: "시궁쥐",
    mob_sewer_rat_lore: "깊은 물길을 지키던 덩치. 독이 몸을 더 단단하게 만들었다.",
    mob_sewer_rat_atk: "오래 떨다가 멀리, 세게 돌진한다. 잘 밀리지 않는다.",
    mob_sewer_rat_tip: "벽에 박고 멈칫할 때가 기회. 드릴로 벽까지 밀어붙여라.",
    mob_chipmunk: "다람쥐",
    mob_chipmunk_lore: "도토리를 모으던 부지런한 이웃. 이제는 그 도토리를 던진다.",
    mob_chipmunk_atk: "거리를 두고 도토리를 던진다.",
    mob_chipmunk_tip: "도토리를 휘둘러 받아치면 되돌아간다.",
    mob_red_squirrel: "청설모",
    mob_red_squirrel_lore: "나무 꼭대기를 뛰어다니던 날쌘 이웃. 뿌리 아래로 끌려 내려왔다.",
    mob_red_squirrel_atk: "웅크렸다가 뛰어올라 덮친다. 내려앉는 자리 둘레가 아프다.",
    mob_red_squirrel_tip: "바닥에 그려진 고리 밖으로. 내려앉은 뒤가 빈틈.",
    mob_dummy: "허수아비",
    mob_dummy_lore: "포포가 만든 연습 상대.",
    mob_dummy_atk: "아무것도 하지 않는다.",
    mob_dummy_tip: "마음껏 때려 보자.",
    w_hp_1: "한 대면 쓰러짐",
    w_hp_2: "약함",
    w_hp_3: "튼튼함",
    w_hp_4: "아주 튼튼함",
    w_pow_1: "조금 아픔",
    w_pow_2: "많이 아픔",
    w_spd_1: "느림",
    w_spd_2: "보통",
    w_spd_3: "빠름",
    w_spd_4: "아주 빠름",
    tab_bestiary: "도감",
    stat_power: "공격력",
    stat_aspd: "공격 속도",
    stat_dig_cd: "굴파기 대기",
    stat_gauge: "{0} 충전",
    stat_weapon: "무기",
    seeds: "씨앗",
    fmt_secs: "{0}초",
    fmt_pct: "{0}%",
    fmt_hp: "{0} / {1}",
    wallet_note: "반딧불은 기술을 깨우고, 씨앗은 상점에서 쓴다(곧).",
    best_unknown: "???",
    best_unseen: "아직 만나지 못했다.",
    best_hp: "체력",
    best_pow: "힘",
    best_spd: "빠르기",
    best_atk: "공격",
    best_tip: "요령",
    best_count: "만난 이웃 {0} / {1}",
    set_controls: "조작 설정",
    act_attack: "공격",
    act_dodge: "굴파기(회피)",
    act_special: "필살기",
    act_use_item: "아이템 쓰기",
    act_item_prev: "이전 아이템",
    act_item_next: "다음 아이템",
    act_map: "지도",
    ctl_key: "키보드",
    ctl_pad: "패드",
    ctl_reset: "기본값으로",
    ctl_wait: "누르세요… {0}",
    ctl_hint: "좌우: 키보드/패드 · 확인: 바꾸기 · 겹치면 맞바꿈",
    ctl_fixed: "고정: 이동 WASD·방향키·스틱 · 메뉴 Esc·Start · 확인 Enter·A · 뒤로 Esc·B",
    popo_name: "포포",
    popo_hi: "어이, 꼬마 두더지! 괜찮니? 나는 라쿤 포포야.",
    popo_hi2: "뿌리 아래로 간다고? 그럼 몸 쓰는 법부터 알려 줄게. 하나씩 해 보자!",
    popo_pickup: "저기 반짝이는 거 보이지? 네 발톱이야. 가서 주워 와.",
    popo_combo: "허수아비를 세 번 연달아 쳐 봐. 세 번째가 제일 묵직해!",
    popo_burrow: "저 다람쥐가 도토리를 던질 거야. 땅속으로 숨으면 안 맞아. 지금은 연습이라 다치지 않아.",
    popo_emerge: "땅속에서 다람쥐 밑으로 가서 한 번 더 누르면 솟구쳐 공격해!",
    popo_parry: "이번엔 받아치기야. 도토리가 닿기 직전에 휘두르면 튕겨 낼 수 있어. 되돌아간 도토리는 적을 맞혀!",
    popo_parry_skip: "어렵지? 괜찮아, 싸우다 보면 감이 와.",
    popo_special: "잘했어! 때리고 받아칠수록 분노가 차올라. 가득 차면 필살기! 내 기운을 나눠 줄게.",
    popo_item: "아야, 조금 다쳤네. 산딸기를 줄게. 먹으면 체력이 하나 돌아와.",
    popo_item2: "쥬시 지렁이는 체력을 전부 채워 줘. 들고 있으면 쓰러져도 한 번 일어날 수 있어.",
    popo_menu: "메뉴(Esc·Start)에서 상태와 도감을 볼 수 있고, {0}로 지도를 바로 열 수 있어.",
    popo_save: "초록 버섯 고리는 저장 지점이야. 쓰러지면 거기서 다시 일어나.",
    popo_bye: "굴 입구는 저 아래쪽이야. 조심해, Naïl!",
    tut_step: "[{0}/{1}] {2}",
    tut_parry: "도토리가 닿기 직전에 {0}: 튕겨내기",
    tut_special: "{0}: 필살기! 허수아비들을 벽까지 밀어붙여라",
    tut_item: "{0}: 산딸기 먹기",
    mob_chief: "들쥐 대장",
    mob_chief_lore: "들쥐 마을의 늙은 대장. 뿌리 독에 물들어 남쪽 공터를 지킨다.",
    mob_chief_atk: "길게 떨다가 곧게 돌진하고, 웅크렸다가 크게 뛰어 찍는다. 지치면 생쥐 떼를 부른다. 공격 중엔 맞아도 멈추지 않는다.",
    mob_chief_tip: "돌진을 바위나 벽으로 유도하면 한참 어질어질하다. 그때 때리자.",
    toast_chief: "들쥐 대장을 물리쳤다! 굴 입구가 열렸다.",
    toast_camp: "뿌리 밑 쉼터. 체력이 가득 찼다.",
    toast_spring: "맑은 샘물. 체력이 가득 찼다.",
    toast_goal: "굴 끝의 무리를 물리쳤다! 출구가 열렸다.",
    tut_boss: "남쪽 공터의 들쥐 대장을 쓰러뜨리자",
    popo_glade: "남쪽 공터 문을 열어 둘게. 거기 들쥐 대장이 뿌리에 물들어 버렸어… 정신 차리게 해 줘!",
    bori_name: "보리",
    chief_intro_1: "찍…! 여긴 내 공터다. 아무도 못 지나간다!",
    chief_intro_2: "비켜 주세요. 아래로 내려가야 해요!",
    chief_beaten_1: "으으… 머리가 맑아진다. 내가 무슨 짓을…",
    chief_beaten_2: "괜찮아요? 뿌리에 물들어 있었어요.",
    chief_beaten_3: "고맙다, 꼬마 두더지. 나는 들쥐 마을 대장 보리다.",
    chief_beaten_4: "저 굴 아래에 쉼터가 있다. 먼저 내려가서 불을 피워 두마.",
    chief_beaten_5: "그 아이도… 이렇게 돌아올 수 있을까.",
    camp_1: "어서 와! 여기는 뿌리 밑 쉼터야. 불 곁에서 푹 쉬어 가.",
    camp_2: "씨앗을 모아 오면 내 가게에서 바꿔 줄게. 초록 고리는 여기서도 저장 지점이야.",
    camp_3: "아래로 가는 뿌리 길은 저기다. 갈래마다 다른 굴이 있지. 잘 골라라.",
    camp_again: "또 한 길을 지나왔구나. 불 곁에서 쉬어 가라.",
    bori_1: "뿌리에 홀려 있던 동안의 일은 꿈처럼 흐릿하다. 미안하구나.",
    bori_2: "굴 끝에 모인 무리는 세다. 벽 가까이에서 싸우면 돌진을 유도하기 좋지.",
    bori_3: "샘이 보이면 들러 가라. 깊이 내려갈수록 쉴 곳이 귀하다.",
    seat_1: "누구 자리일까… 시든 꽃잎이 하나 떨어져 있다.",
    node_burrow: "굴",
    node_burrow_desc: "쥐들이 사는 굴. 끝의 무리를 잡으면 씨앗.",
    node_nest: "둥지",
    node_nest_desc: "더 센 무리가 모인 곳. 장신구 상자가 있다.",
    node_cache: "저장고",
    node_cache_desc: "작은 굴. 도토리 저장고 안에 상자가 있다.",
    node_spring: "샘",
    node_spring_desc: "싸움 없이 쉬어 간다. 체력이 가득 차고 산딸기.",
    node_boss: "길의 주인",
    node_boss_desc: "이 길을 지키는 무리의 우두머리. 이기면 다음 쉼터로.",
    node_unknown: "?",
    node_unknown_desc: "아직 모른다. 가까이 가면 보인다.",
    route_title: "뿌리 길 {0}",
    route_hint: "방향 고르기 · {0} 들어가기",
    route_hint_back: "방향 고르기 · {0} 들어가기 · 뒤로 닫기",
    shop_title: "포포의 가게",
    shop_seeds: "씨앗 {0}",
    shop_hi: "어서 와! 씨앗만 있으면 뭐든 바꿔 줄게.",
    shop_poor: "씨앗이 모자라… 굴에서 더 모아 와!",
    shop_full: "가방에 더 들어갈 자리가 없어.",
    shop_thanks: "고마워! 잘 써.",
    shop_price: "{0} 씨앗",
    shop_heart: "뿌리 심장",
    shop_heart_desc: "최대 체력 +1. 쉼터마다 하나만.",
    shop_hint: "{0} 사기 · 뒤로 닫기",
    ia_shop: "{0} 가게",
    ia_talk: "{0} 말 걸기",
    ia_look: "{0} 살펴보기",
    ia_route: "{0} 길 지도",
    map_camp: "쉼터 {0}",
};

pub static EN: Strings = Strings {
    lang_name: "English",
    intro_1: "The great tree is withering.",
    intro_2: "Its roots have turned the neighbors underground strange.",
    intro_3: "Naïl, a baby mole, follows the roots down to find out why.",
    skip_hint: "Confirm: skip",
    title_start: "Start",
    title_settings: "Settings",
    title_dev: "Dev",
    title_controls: "Up/Down: move · Confirm (Enter/A) · Back (Esc/B)",
    version: "Prototype v0.1",
    settings: "Settings",
    set_sound: "Sound",
    set_shake: "Screen shake",
    set_debug: "Debug info",
    set_language: "Language",
    set_back: "Back",
    on: "On",
    off: "Off",
    settings_hint: "Left/Right to change · Saved in this browser on the web",
    tab_menu: "Menu",
    tab_status: "Status",
    tab_bag: "Bag",
    tab_journal: "Journal",
    tab_map: "Map",
    tab_hint: "Q/E · LB/RB: tabs",
    pm_resume: "Resume",
    pm_restart: "Restart",
    pm_settings: "Settings",
    pm_title: "Title screen",
    hp: "HP",
    this_run: "This run",
    run_stats: "Wave {0}  Kills {1}  Time {2}",
    combo_stats: "{0}-hit combo. Finisher {1}s, knockback {2}",
    bare_hands: "Bare paws",
    bare_hands_desc: "No claws yet. You can't attack.",
    burrow: "Burrow",
    burrow_stats: "Invulnerable {0}s, burst {1}px, cooldown {2}s",
    special: "Special",
    glowbugs: "Glowbugs",
    bag_combo: "{0}-hit combo · finisher damage {1}",
    equipped: "Equipped",
    equip_hint: "Confirm: equip",
    empty_slot: "Empty",
    empty_slot_desc: "Items arrive in phase 2 (room-clear rewards). For now it's just the frame.",
    slots: "{0} / {1} slots",
    map_floor: "Floor 1 · {0}",
    map_proto: "Prototype: one room",
    map_note: "Floors and linked rooms come in phase 3. Just this room for now.",
    q_roots: "Follow the Roots",
    q_roots_desc: "Follow the withering tree's roots deeper down. (Floors come in phase 3)",
    q_claws: "Taming the Claws",
    q_claws_desc: "Defeat three enemies with a 3-hit claw combo. Feel how heavy the third hit is.",
    q_ambush: "Ambush",
    q_ambush_desc: "Burrow underground, then burst out to defeat an enemy.",
    q_survive: "Hold Out",
    q_survive_desc: "Survive until wave 3.",
    biome_meadow: "Flowering Entrance",
    biome_roots: "Root Tangle",
    biome_deep: "Dark Depths",
    w_sprout: "Sprout Claws",
    w_sprout_desc: "Light. A quick, snappy 4-hit combo. Each hit is weak.",
    w_mole: "Mole Claws",
    w_mole_desc: "Standard. 3 hits, the third lands heavy. Burrow to cancel any time.",
    w_stone: "Stone Claws",
    w_stone_desc: "Heavy. 2 wide hits that shove hard. Burrow cancels only after the swing.",
    w_venom: "Venom Claws",
    w_venom_desc: "Poisons what it hits; the finisher's poison lasts longer. Poison drains HP without stagger.",
    gift_title: "Got new claws!",
    tut_move: "Stick / WASD: try moving",
    tut_pickup: "Go up to the shiny thing",
    tut_attack: "{0} three times: the third one is heavy",
    tut_burrow: "{0}: burrow. You can't be hit underground",
    tut_emerge: "Underground, get beneath an enemy and {0} again: burst out",
    tut_outro: "Follow the roots down",
    cut_skip: "Back (Esc/B): skip",
    op_1: "…The roots reach all the way down here. The soil feels different.",
    op_2: "Who's there? This used to be our home.",
    op_3: "The tree… the tree made us like this. Go back.",
    op_4: "I'm going to find out why. Please let me through.",
    op_5: "(Raises claws.)",
    dead: "You fell",
    dead_hint: "Confirm: restart   Menu: pause",
    c_shell: "Hard Shell",
    c_shell_desc: "Max HP +1.",
    c_whisker: "Quick Whiskers",
    c_whisker_desc: "Attack speed +20%.",
    c_soil: "Damp Soil",
    c_soil_desc: "Burrow cooldown -40%.",
    c_flint: "Sharp Flint",
    c_flint_desc: "Attack +1, but attack speed -10%.",
    c_amulet: "Root Charm",
    c_amulet_desc: "SP fills 1.5× faster.",
    it_berry: "Wild Berry",
    it_berry_desc: "Restores 1 HP.",
    it_worm: "Juicy Worm",
    it_worm_desc: "Restores all HP. If you fall while carrying one, you eat it and get back up.",
    toast_heart: "Root Heart! Max HP increased.",
    toast_revive: "Ate a Juicy Worm and got back up!",
    quick_hint: "{0} eat",
    special_name: "Drill Dash",
    special_desc: "When {0} is full: {1}. Vrrrooom! Drill across the screen, pushing foes into a wall for a huge bonk!",
    charms_label: "Charms",
    charms_none: "None",
    stat_attack: "Attack +{0} · speed {1}%",
    charm_equip: "Confirm: wear",
    charm_unequip: "Confirm: remove",
    bag_items: "Berries {0}/{1} · Worms {2}/{3}",
    charm_slots: "Charms {0}/{1}",
    toast_save: "Save point: you will get back up here if you fall.",
    toast_respawn: "Back at the save point. Lost a few glowbugs.",
    toast_boss: "Beat the burrow keeper! The exit is open.",
    gift_charm: "Got a new charm!",
    map_field: "Field · {0}",
    map_dungeon: "Burrow F{0} · {1}",
    map_legend: "White: you · Green: save · Black: burrow",
    toast_parry: "Parried!",
    gauge_rage: "Rage",
    gauge_ki: "Ki",
    mob_field_mouse: "Field Mouse",
    mob_field_mouse_lore: "A chatty burrow-dweller of the meadow. The root blight made its eyes glow red.",
    mob_field_mouse_atk: "Shivers, then charges in a straight line.",
    mob_field_mouse_tip: "Sidestep while it shivers, or swing just before it hits to parry.",
    mob_house_mouse: "House Mouse",
    mob_house_mouse_lore: "A tiny, timid neighbor. Now it roams in packs and bites anything.",
    mob_house_mouse_atk: "Swarms close and lunges in short bursts.",
    mob_house_mouse_tip: "One hit is enough. Catch the whole pack with an emerge.",
    mob_sewer_rat: "Sewer Rat",
    mob_sewer_rat_lore: "A big keeper of the deep waterways. The blight hardened its body.",
    mob_sewer_rat_atk: "Shivers long, then charges far and hard. Hard to knock back.",
    mob_sewer_rat_tip: "Strike when it rams a wall. Drill it into the wall.",
    mob_chipmunk: "Chipmunk",
    mob_chipmunk_lore: "A hardworking acorn collector. Now it throws them.",
    mob_chipmunk_atk: "Keeps its distance and throws acorns.",
    mob_chipmunk_tip: "Swing into an acorn at the right moment to send it back.",
    mob_red_squirrel: "Red Squirrel",
    mob_red_squirrel_lore: "A nimble treetop leaper, dragged down below the roots.",
    mob_red_squirrel_atk: "Crouches, then leaps on you. Its landing hurts all around.",
    mob_red_squirrel_tip: "Get out of the ring on the ground. It is open after landing.",
    mob_dummy: "Scarecrow",
    mob_dummy_lore: "A practice partner Popo made.",
    mob_dummy_atk: "Does nothing.",
    mob_dummy_tip: "Hit it as much as you like.",
    w_hp_1: "Falls in one hit",
    w_hp_2: "Frail",
    w_hp_3: "Sturdy",
    w_hp_4: "Very sturdy",
    w_pow_1: "Stings a little",
    w_pow_2: "Hurts a lot",
    w_spd_1: "Slow",
    w_spd_2: "Average",
    w_spd_3: "Quick",
    w_spd_4: "Very quick",
    tab_bestiary: "Bestiary",
    stat_power: "Attack",
    stat_aspd: "Attack speed",
    stat_dig_cd: "Burrow cooldown",
    stat_gauge: "{0} gain",
    stat_weapon: "Weapon",
    seeds: "Seeds",
    fmt_secs: "{0}s",
    fmt_pct: "{0}%",
    fmt_hp: "{0} / {1}",
    wallet_note: "Glowbugs awaken skills; seeds are spent at shops (soon).",
    best_unknown: "???",
    best_unseen: "Not met yet.",
    best_hp: "Toughness",
    best_pow: "Power",
    best_spd: "Speed",
    best_atk: "Attack",
    best_tip: "Tip",
    best_count: "Neighbors met {0} / {1}",
    set_controls: "Controls",
    act_attack: "Attack",
    act_dodge: "Burrow (dodge)",
    act_special: "Special",
    act_use_item: "Use item",
    act_item_prev: "Previous item",
    act_item_next: "Next item",
    act_map: "Map",
    ctl_key: "Keyboard",
    ctl_pad: "Pad",
    ctl_reset: "Reset to default",
    ctl_wait: "Press… {0}",
    ctl_hint: "Left/Right: key/pad · Confirm: change · Swaps if taken",
    ctl_fixed: "Fixed: move WASD/arrows/stick · menu Esc/Start · confirm Enter/A · back Esc/B",
    popo_name: "Popo",
    popo_hi: "Hey there, little mole! You okay? I'm Popo the raccoon.",
    popo_hi2: "Going down below the roots? Then let me teach you how to fight first. One step at a time!",
    popo_pickup: "See that shiny thing? Those are your claws. Go pick them up.",
    popo_combo: "Hit the scarecrow three times in a row. The third hit is the heaviest!",
    popo_burrow: "That chipmunk will throw acorns. Hide underground and they can't hit you. It's practice, so you won't get hurt.",
    popo_emerge: "While underground, get under the chipmunk and press again to burst out and strike!",
    popo_parry: "Now, parrying. Swing right before an acorn hits to send it back. A returned acorn hits the enemy!",
    popo_parry_skip: "Tricky, right? It's fine. You'll get the feel of it in battle.",
    popo_special: "Nice! Hitting and parrying fills your Rage. When it's full, use your special! Here, have some of my energy.",
    popo_item: "Ouch, you're a little hurt. Here's a raspberry. Eat it to get one HP back.",
    popo_item2: "A juicy worm restores all your HP. Carry one and you can get back up once if you fall.",
    popo_menu: "In the menu (Esc/Start) you can see your status and the bestiary. {0} opens the map.",
    popo_save: "The ring of green mushrooms is a save point. If you fall, you get back up there.",
    popo_bye: "The burrow entrance is down that way. Take care, Naïl!",
    tut_step: "[{0}/{1}] {2}",
    tut_parry: "Right before an acorn hits, {0}: parry",
    tut_special: "{0}: special! Drive the scarecrows into the wall",
    tut_item: "{0}: eat the raspberry",
    mob_chief: "Field Mouse Chief",
    mob_chief_lore: "The old chief of the field mice. Tainted by the root poison, he guards the southern glade.",
    mob_chief_atk: "Shakes for a long time, then charges straight; crouches, then slams down in a big leap. When worn down, he calls a mouse pack. He won't flinch mid-attack.",
    mob_chief_tip: "Lure his charge into a rock or wall and he'll be dizzy for a while. Hit him then.",
    toast_chief: "You beat the Field Mouse Chief! The burrow entrance is open.",
    toast_camp: "The camp under the roots. HP fully restored.",
    toast_spring: "A clear spring. HP fully restored.",
    toast_goal: "Cleared the pack at the end of the burrow! The exit is open.",
    tut_boss: "Defeat the Field Mouse Chief in the southern glade",
    popo_glade: "I'll open the gate to the southern glade. The field mouse chief there got tainted by the roots... Snap him out of it!",
    bori_name: "Bori",
    chief_intro_1: "Squeak...! This is my glade. No one gets through!",
    chief_intro_2: "Please let me through. I have to go down there!",
    chief_beaten_1: "Ugh... my head is clearing. What have I done...",
    chief_beaten_2: "Are you okay? You were tainted by the roots.",
    chief_beaten_3: "Thank you, little mole. I'm Bori, chief of the field mouse village.",
    chief_beaten_4: "There's a camp down that burrow. I'll go ahead and light a fire.",
    chief_beaten_5: "Could that one... come back like this too?",
    camp_1: "Welcome! This is the camp under the roots. Rest up by the fire.",
    camp_2: "Bring me seeds and I'll trade at my shop. The green ring is a save point here too.",
    camp_3: "The root path down is over there. Each fork leads to a different burrow. Choose well.",
    camp_again: "Another path behind you. Rest by the fire.",
    bori_1: "My time under the roots' spell is as hazy as a dream. I'm sorry.",
    bori_2: "The packs at the end of a burrow are tough. Fight near a wall to bait their charges.",
    bori_3: "Stop at a spring when you see one. The deeper you go, the rarer a rest.",
    seat_1: "Whose seat is this...? A single wilted petal lies here.",
    node_burrow: "Burrow",
    node_burrow_desc: "A burrow full of rats. Beat the pack at the end for seeds.",
    node_nest: "Nest",
    node_nest_desc: "Where tougher packs gather. Holds a charm chest.",
    node_cache: "Cache",
    node_cache_desc: "A small burrow. A chest sits in the acorn store.",
    node_spring: "Spring",
    node_spring_desc: "Rest without a fight. Full HP and wild berries.",
    node_boss: "Path Keeper",
    node_boss_desc: "The leader guarding this path. Win to reach the next camp.",
    node_unknown: "?",
    node_unknown_desc: "Unknown. You'll see it when you get closer.",
    route_title: "Root Path {0}",
    route_hint: "Arrows choose · {0} enter",
    route_hint_back: "Arrows choose · {0} enter · Back closes",
    shop_title: "Popo's Shop",
    shop_seeds: "Seeds {0}",
    shop_hi: "Welcome! Got seeds? I'll trade you anything.",
    shop_poor: "Not enough seeds... Gather more in the burrows!",
    shop_full: "There's no more room in your bag.",
    shop_thanks: "Thanks! Use it well.",
    shop_price: "{0} seeds",
    shop_heart: "Root Heart",
    shop_heart_desc: "Max HP +1. Only one per camp.",
    shop_hint: "{0} buy · Back closes",
    ia_shop: "{0} Shop",
    ia_talk: "{0} Talk",
    ia_look: "{0} Look",
    ia_route: "{0} Path map",
    map_camp: "Camp {0}",
};

pub static JA: Strings = Strings {
    lang_name: "日本語",
    intro_1: "大きな木が枯れていく。",
    intro_2: "根っこが地下の住人たちを変えてしまった。",
    intro_3: "子モグラのNaïlは、わけを知るため根をたどって下りていく。",
    skip_hint: "決定：スキップ",
    title_start: "はじめる",
    title_settings: "設定",
    title_dev: "開発",
    title_controls: "上下：えらぶ・決定（Enter/A）・もどる（Esc/B）",
    version: "プロトタイプ v0.1",
    settings: "設定",
    set_sound: "音量",
    set_shake: "画面のゆれ",
    set_debug: "デバッグ表示",
    set_language: "言語",
    set_back: "もどる",
    on: "オン",
    off: "オフ",
    settings_hint: "左右で変更・Webではこのブラウザに保存",
    tab_menu: "メニュー",
    tab_status: "つよさ",
    tab_bag: "どうぐ",
    tab_journal: "きろく",
    tab_map: "ちず",
    tab_hint: "Q/E・LB/RB",
    pm_resume: "つづける",
    pm_restart: "やりなおす",
    pm_settings: "設定",
    pm_title: "タイトルへ",
    hp: "HP",
    this_run: "今回",
    run_stats: "ウェーブ{0}  {1}体  {2}",
    combo_stats: "{0}連打・とどめ{1}秒・ふっとび{2}",
    bare_hands: "すで",
    bare_hands_desc: "まだツメがない。こうげきできない。",
    burrow: "あなほり",
    burrow_stats: "むてき{0}秒・{1}px・待ち{2}秒",
    special: "とくぎ",
    glowbugs: "ホタル",
    bag_combo: "{0}連打・とどめ{1}ダメージ",
    equipped: "そうび中",
    equip_hint: "決定：そうび",
    empty_slot: "からっぽ",
    empty_slot_desc: "アイテムはフェーズ2（へやクリアのほうび）で入る。今はわくだけ。",
    slots: "{0} / {1} マス",
    map_floor: "地下1階・{0}",
    map_proto: "プロトタイプ：へや1つ",
    map_note: "階とへやのつながりはフェーズ3で。",
    q_roots: "根をたどって",
    q_roots_desc: "枯れていく木の根をたどって、もっと深くへ。（階はフェーズ3）",
    q_claws: "ツメのならし",
    q_claws_desc: "ツメの3連打で敵を3体たおす。3発目の重さを感じてみよう。",
    q_ambush: "ふいうち",
    q_ambush_desc: "あなほりで地中にもぐり、とびだして敵をたおす。",
    q_survive: "もちこたえる",
    q_survive_desc: "ウェーブ3まで生きのこる。",
    biome_meadow: "花さく入り口",
    biome_roots: "根っこエリア",
    biome_deep: "くらい深み",
    w_sprout: "新芽のツメ",
    w_sprout_desc: "かるい。すばやい4連打。1発は弱い。",
    w_mole: "モグラのツメ",
    w_mole_desc: "ふつう。3連打で3発目が重い。いつでもあなほりで中断できる。",
    w_stone: "岩のツメ",
    w_stone_desc: "おもい。広く強くふっとばす2連打。ふり終わってから中断できる。",
    w_venom: "毒のツメ",
    w_venom_desc: "当てた敵を毒にする。とどめは毒が長い。毒はひるませず体力だけけずる。",
    gift_title: "新しいツメを手に入れた！",
    tut_move: "スティック／WASD：うごいてみよう",
    tut_pickup: "ひかるものに近づいてみよう",
    tut_attack: "{0}を3回：3回目が重い",
    tut_burrow: "{0}：あなほり。地中ではやられない",
    tut_emerge: "地中で敵の下へ行き、もう一度{0}：とびだす",
    tut_outro: "根をたどって下りていこう",
    cut_skip: "もどる（Esc/B）：スキップ",
    op_1: "…根っこがこんなところまで。土の感じがちがう。",
    op_2: "だれ？ ここはぼくたちの家だったんだ。",
    op_3: "木が…木がぼくたちをこんなふうにした。帰って。",
    op_4: "ぼくはわけをさがしに行くんだ。通して。",
    op_5: "（ツメをかまえる。）",
    dead: "たおれた",
    dead_hint: "決定：やりなおす　メニュー：ポーズ",
    c_shell: "かたいカラ",
    c_shell_desc: "最大HP+1。",
    c_whisker: "すばやいヒゲ",
    c_whisker_desc: "こうげき速度+20%。",
    c_soil: "しめった土",
    c_soil_desc: "あなほりの待ち時間-40%。",
    c_flint: "するどい火打ち石",
    c_flint_desc: "こうげき+1。かわりに速度-10%。",
    c_amulet: "根のお守り",
    c_amulet_desc: "SPが1.5倍はやくたまる。",
    it_berry: "野いちご",
    it_berry_desc: "HPを1回復。",
    it_worm: "ジューシーミミズ",
    it_worm_desc: "食べるとHP全回復。持ったままたおれると、自動で食べて起き上がる。",
    toast_heart: "根のハート！最大HPがふえた。",
    toast_revive: "ジューシーミミズを食べて起き上がった！",
    quick_hint: "{0} たべる",
    special_name: "ドリル突進",
    special_desc: "{0}がたまったら{1}。ブオオーン！画面のはしまで連打で突進、敵をかべに押しこむとドカン！",
    charms_label: "おまもり",
    charms_none: "なし",
    stat_attack: "こうげき+{0}・速度{1}%",
    charm_equip: "決定：つける",
    charm_unequip: "決定：はずす",
    bag_items: "野いちご{0}/{1}・ミミズ{2}/{3}",
    charm_slots: "おまもり{0}/{1}",
    toast_save: "セーブ地点：たおれたらここから。",
    toast_respawn: "セーブ地点から起き上がった。ホタルを少しなくした。",
    toast_boss: "穴のぬしをたおした！出口があいた。",
    gift_charm: "新しいおまもりを手に入れた！",
    map_field: "野原・{0}",
    map_dungeon: "穴{0}階・{1}",
    map_legend: "白：自分・緑：セーブ・黒：穴",
    toast_parry: "はじいた！",
    gauge_rage: "いかり",
    gauge_ki: "気",
    mob_field_mouse: "ノネズミ",
    mob_field_mouse_lore: "野原の穴にすむおしゃべり屋。根の毒にそまって目が赤く光る。",
    mob_field_mouse_atk: "ブルブルふるえてから、まっすぐ突進する。",
    mob_field_mouse_tip: "ふるえている間によけるか、当たる直前に振ってはじき返す。",
    mob_house_mouse: "ハツカネズミ",
    mob_house_mouse_lore: "小さくておくびょうな近所の子。今は群れで動き、何にでもかみつく。",
    mob_house_mouse_atk: "群れで近づき、短く飛びかかる。",
    mob_house_mouse_tip: "一発でたおれる。とび出しでまとめて。",
    mob_sewer_rat: "ドブネズミ",
    mob_sewer_rat_lore: "深い水路を守っていた大物。毒が体をさらにかたくした。",
    mob_sewer_rat_atk: "長くふるえてから、遠くまで強く突進する。押し返しにくい。",
    mob_sewer_rat_tip: "壁にぶつかってひるんだ時がチャンス。ドリルで壁まで押しこめ。",
    mob_chipmunk: "シマリス",
    mob_chipmunk_lore: "どんぐりを集めていた働き者。今はそのどんぐりを投げてくる。",
    mob_chipmunk_atk: "きょりをとって、どんぐりを投げる。",
    mob_chipmunk_tip: "どんぐりにタイミングよく振ると打ち返せる。",
    mob_red_squirrel: "キタリス",
    mob_red_squirrel_lore: "木のてっぺんをかけ回っていたすばしっこい子。根の下へ引きずりこまれた。",
    mob_red_squirrel_atk: "しゃがんでから飛びかかる。着地のまわりが痛い。",
    mob_red_squirrel_tip: "地面の輪から出よう。着地したあとがスキ。",
    mob_dummy: "かかし",
    mob_dummy_lore: "ポポが作った練習相手。",
    mob_dummy_atk: "何もしない。",
    mob_dummy_tip: "思いきりたたいてみよう。",
    w_hp_1: "一発でたおれる",
    w_hp_2: "よわい",
    w_hp_3: "がんじょう",
    w_hp_4: "とてもがんじょう",
    w_pow_1: "すこし痛い",
    w_pow_2: "とても痛い",
    w_spd_1: "おそい",
    w_spd_2: "ふつう",
    w_spd_3: "はやい",
    w_spd_4: "とてもはやい",
    tab_bestiary: "ずかん",
    stat_power: "こうげき",
    stat_aspd: "こうげき速度",
    stat_dig_cd: "穴ほり待ち",
    stat_gauge: "{0}のたまり",
    stat_weapon: "武器",
    seeds: "たね",
    fmt_secs: "{0}秒",
    fmt_pct: "{0}%",
    fmt_hp: "{0} / {1}",
    wallet_note: "ホタルはわざをめざめさせ、たねはお店で使う（もうすぐ）。",
    best_unknown: "???",
    best_unseen: "まだ出会っていない。",
    best_hp: "体力",
    best_pow: "ちから",
    best_spd: "はやさ",
    best_atk: "こうげき",
    best_tip: "コツ",
    best_count: "出会った近所 {0} / {1}",
    set_controls: "そうさ設定",
    act_attack: "こうげき",
    act_dodge: "穴ほり（回避）",
    act_special: "ひっさつ技",
    act_use_item: "アイテムを使う",
    act_item_prev: "前のアイテム",
    act_item_next: "次のアイテム",
    act_map: "地図",
    ctl_key: "キーボード",
    ctl_pad: "パッド",
    ctl_reset: "初期設定にもどす",
    ctl_wait: "おしてください… {0}",
    ctl_hint: "左右：キー/パッド・決定：変更・かぶると入れかえ",
    ctl_fixed: "固定：移動 WASD・矢印・スティック・メニュー Esc・Start・決定 Enter・A・もどる Esc・B",
    popo_name: "ポポ",
    popo_hi: "おーい、ちびモグラ！だいじょうぶ？ぼくはアライグマのポポ。",
    popo_hi2: "根の下へ行くって？じゃあ、まず体の使い方を教えてあげる。ひとつずつやってみよう！",
    popo_pickup: "あそこに光ってるの、見える？きみのツメだよ。拾っておいで。",
    popo_combo: "かかしを3回続けてたたいてみて。3回目がいちばん重いよ！",
    popo_burrow: "あのシマリスがどんぐりを投げてくる。地中にかくれれば当たらないよ。練習だからケガはしない。",
    popo_emerge: "地中でシマリスの下まで行って、もう一度おすと飛び出してこうげき！",
    popo_parry: "次ははじき返し。どんぐりが当たる直前に振ると打ち返せる。返ったどんぐりは敵に当たるよ！",
    popo_parry_skip: "むずかしいよね？だいじょうぶ、たたかっているうちにコツがつかめるよ。",
    popo_special: "いいね！たたいたりはじき返すと、いかりがたまる。満タンでひっさつ技！ぼくの元気をわけてあげる。",
    popo_item: "いたた、ちょっとケガしたね。ラズベリーをあげる。食べると体力がひとつもどるよ。",
    popo_item2: "ジューシーミミズは体力を全部回復。持っていれば、たおれても一度起き上がれる。",
    popo_menu: "メニュー（Esc・Start）で状態とずかんが見られる。{0}で地図がすぐ開くよ。",
    popo_save: "緑のキノコの輪はセーブ地点。たおれたら、そこから起き上がるよ。",
    popo_bye: "あなの入り口はあっちの下のほう。気をつけてね、Naïl！",
    tut_step: "[{0}/{1}] {2}",
    tut_parry: "どんぐりが当たる直前に{0}：はじき返す",
    tut_special: "{0}：ひっさつ技！かかしをかべまで押しこめ",
    tut_item: "{0}：ラズベリーを食べる",
    mob_chief: "ノネズミの親分",
    mob_chief_lore: "ノネズミの村の年老いた親分。根の毒に染まり、南の空き地を守っている。",
    mob_chief_atk: "長くふるえてからまっすぐ突進し、かがんでから大きく跳んでたたきつける。弱るとハツカネズミの群れを呼ぶ。こうげき中は打っても止まらない。",
    mob_chief_tip: "突進を岩やかべに向けさせると、しばらくふらふら。そのすきにたたこう。",
    toast_chief: "ノネズミの親分をたおした！穴の入り口があいた。",
    toast_camp: "根の下の休み場。HPが満タンになった。",
    toast_spring: "すんだ泉の水。HPが満タンになった。",
    toast_goal: "穴のおくの群れをたおした！出口があいた。",
    tut_boss: "南の空き地のノネズミの親分をたおそう",
    popo_glade: "南の空き地の門をあけておくね。そこのノネズミの親分が根に染まっちゃったんだ…目をさまさせてあげて！",
    bori_name: "ボリ",
    chief_intro_1: "チュッ…！ここはおれの空き地だ。だれも通さん！",
    chief_intro_2: "どいてください。下へ行かなきゃいけないんです！",
    chief_beaten_1: "うう…頭がすっきりしてきた。わしは何を…",
    chief_beaten_2: "だいじょうぶ？根に染まっていたんです。",
    chief_beaten_3: "ありがとう、ちびモグラ。わしはノネズミの村の親分、ボリだ。",
    chief_beaten_4: "あの穴の下に休み場がある。先に行って火をおこしておこう。",
    chief_beaten_5: "あの子も…こんなふうにもどれるかな。",
    camp_1: "いらっしゃい！ここは根の下の休み場。火のそばでゆっくりしていって。",
    camp_2: "タネを集めてきたら、ぼくの店でとりかえてあげる。緑の輪はここでもセーブ地点だよ。",
    camp_3: "下へ行く根の道はあそこだ。分かれ道ごとにちがう穴がある。よく選べ。",
    camp_again: "またひとつ道をこえてきたな。火のそばで休んでいけ。",
    bori_1: "根にとらわれていたころのことは、夢みたいにぼんやりしている。すまなかったな。",
    bori_2: "穴のおくの群れは手ごわい。かべの近くで戦うと突進をさそいやすいぞ。",
    bori_3: "泉が見えたら寄っていけ。深く下りるほど、休める場所は少ない。",
    seat_1: "だれの席だろう…しおれた花びらがひとつ落ちている。",
    node_burrow: "穴",
    node_burrow_desc: "ネズミたちの穴。おくの群れをたおすとタネ。",
    node_nest: "巣",
    node_nest_desc: "手ごわい群れが集まる場所。おまもりの箱がある。",
    node_cache: "たくわえ穴",
    node_cache_desc: "小さな穴。ドングリのたくわえの中に箱がある。",
    node_spring: "泉",
    node_spring_desc: "戦わずに休める。HPが満タンになり、野いちごもある。",
    node_boss: "道のぬし",
    node_boss_desc: "この道を守る群れのかしら。かてば次の休み場へ。",
    node_unknown: "?",
    node_unknown_desc: "まだわからない。近づけば見える。",
    route_title: "根の道 {0}",
    route_hint: "方向で選ぶ・{0} 入る",
    route_hint_back: "方向で選ぶ・{0} 入る・もどるで閉じる",
    shop_title: "ポポの店",
    shop_seeds: "タネ {0}",
    shop_hi: "いらっしゃい！タネがあれば何でもとりかえるよ。",
    shop_poor: "タネが足りないよ…穴でもっと集めてきて！",
    shop_full: "かばんにもう入らないよ。",
    shop_thanks: "ありがとう！大事に使ってね。",
    shop_price: "タネ{0}",
    shop_heart: "根のハート",
    shop_heart_desc: "最大HP+1。休み場ごとにひとつだけ。",
    shop_hint: "{0} 買う・もどるで閉じる",
    ia_shop: "{0} 店",
    ia_talk: "{0} 話す",
    ia_look: "{0} 調べる",
    ia_route: "{0} 道の地図",
    map_camp: "休み場 {0}",
};

pub static ZH_HANS: Strings = Strings {
    lang_name: "简体中文",
    intro_1: "大树正在枯萎。",
    intro_2: "树根让地下的邻居们变得奇怪。",
    intro_3: "小鼹鼠Naïl顺着树根往下走，去寻找原因。",
    skip_hint: "确认：跳过",
    title_start: "开始",
    title_settings: "设置",
    title_dev: "开发",
    title_controls: "上下：选择·确认（Enter/A）·返回（Esc/B）",
    version: "原型 v0.1",
    settings: "设置",
    set_sound: "音量",
    set_shake: "画面震动",
    set_debug: "调试信息",
    set_language: "语言",
    set_back: "返回",
    on: "开",
    off: "关",
    settings_hint: "左右切换·网页版保存在此浏览器",
    tab_menu: "菜单",
    tab_status: "状态",
    tab_bag: "背包",
    tab_journal: "日志",
    tab_map: "地图",
    tab_hint: "Q/E·LB/RB：切换",
    pm_resume: "继续",
    pm_restart: "重新开始",
    pm_settings: "设置",
    pm_title: "返回标题",
    hp: "体力",
    this_run: "本局",
    run_stats: "第{0}波  击败{1}  时间{2}",
    combo_stats: "{0}连击·终结{1}秒·击退{2}",
    bare_hands: "空手",
    bare_hands_desc: "还没有爪子，无法攻击。",
    burrow: "挖洞",
    burrow_stats: "无敌{0}秒·范围{1}px·冷却{2}秒",
    special: "特技",
    glowbugs: "萤火虫",
    bag_combo: "{0}连击·终结伤害{1}",
    equipped: "已装备",
    equip_hint: "确认：装备",
    empty_slot: "空",
    empty_slot_desc: "道具将在第2阶段（通关房间后选择）加入。现在只有框架。",
    slots: "{0} / {1} 格",
    map_floor: "第1层·{0}",
    map_proto: "原型：只有一个房间",
    map_note: "楼层与房间的连接在第3阶段。现在只有这个房间。",
    q_roots: "顺着树根",
    q_roots_desc: "顺着枯萎大树的根，往更深处走。（楼层在第3阶段）",
    q_claws: "驯服爪子",
    q_claws_desc: "用爪子三连击打倒三个敌人。感受一下第三下的分量。",
    q_ambush: "突袭",
    q_ambush_desc: "挖洞钻进地下，再冲出来打倒敌人。",
    q_survive: "坚持住",
    q_survive_desc: "活到第3波。",
    biome_meadow: "花开的入口",
    biome_roots: "树根地带",
    biome_deep: "黑暗深处",
    w_sprout: "嫩芽之爪",
    w_sprout_desc: "轻。快速的四连击，单击较弱。",
    w_mole: "鼹鼠之爪",
    w_mole_desc: "标准。三连击，第三下很重。随时可以挖洞取消。",
    w_stone: "岩石之爪",
    w_stone_desc: "重。两连击，范围大、击退强。挥完后才能取消。",
    w_venom: "毒之爪",
    w_venom_desc: "命中的敌人会中毒，终结的毒更久。毒不会造成硬直，只扣体力。",
    gift_title: "获得了新的爪子！",
    tut_move: "左杆／WASD：试着移动",
    tut_pickup: "靠近那个发光的东西",
    tut_attack: "按{0}三次：第三下很重",
    tut_burrow: "{0}：挖洞。在地下不会被打到",
    tut_emerge: "在地下移到敌人下方，再按{0}：冲出",
    tut_outro: "顺着树根往下走吧",
    cut_skip: "返回（Esc/B）：跳过",
    op_1: "……树根一直伸到了这里。泥土的感觉不一样了。",
    op_2: "是谁？这里以前是我们的家。",
    op_3: "是树……是树把我们变成这样的。回去吧。",
    op_4: "我要去找出原因。请让我过去。",
    op_5: "（竖起爪子。）",
    dead: "倒下了",
    dead_hint: "确认：重新开始   菜单：暂停",
    c_shell: "硬壳",
    c_shell_desc: "最大体力+1。",
    c_whisker: "灵敏胡须",
    c_whisker_desc: "攻击速度+20%。",
    c_soil: "湿润泥土",
    c_soil_desc: "挖洞冷却-40%。",
    c_flint: "锋利石片",
    c_flint_desc: "攻击+1，但攻击速度-10%。",
    c_amulet: "树根护符",
    c_amulet_desc: "SP充能速度×1.5。",
    it_berry: "野莓",
    it_berry_desc: "回复1点体力。",
    it_worm: "多汁蚯蚓",
    it_worm_desc: "吃下回满体力。带着它倒下时会自动吃掉并站起来。",
    toast_heart: "根之心！最大体力增加了。",
    toast_revive: "吃了多汁蚯蚓，重新站起来了！",
    quick_hint: "{0} 吃",
    special_name: "钻头冲刺",
    special_desc: "{0}满时按{1}。轰隆隆！连续钻击冲过整个画面，把敌人顶到墙上就会轰！",
    charms_label: "护符",
    charms_none: "无",
    stat_attack: "攻击+{0}·攻速{1}%",
    charm_equip: "确认：装备",
    charm_unequip: "确认：卸下",
    bag_items: "野莓{0}/{1}·蚯蚓{2}/{3}",
    charm_slots: "护符{0}/{1}",
    toast_save: "存档点：倒下后会在这里重新开始。",
    toast_respawn: "在存档点重新站起来了。损失了一些萤火虫。",
    toast_boss: "打倒了洞穴之主！出口打开了。",
    gift_charm: "获得了新的护符！",
    map_field: "原野·{0}",
    map_dungeon: "洞穴{0}层·{1}",
    map_legend: "白：自己·绿：存档·黑：洞穴",
    toast_parry: "弹开了！",
    gauge_rage: "怒气",
    gauge_ki: "气",
    mob_field_mouse: "田鼠",
    mob_field_mouse_lore: "住在原野洞里的话痨。被根毒染上后，眼睛泛着红光。",
    mob_field_mouse_atk: "浑身发抖之后直线冲撞。",
    mob_field_mouse_tip: "发抖时闪开，或在撞到前一刻挥爪弹开。",
    mob_house_mouse: "小鼠",
    mob_house_mouse_lore: "胆小的小邻居。现在成群结队，见什么咬什么。",
    mob_house_mouse_atk: "成群靠近，短距离扑咬。",
    mob_house_mouse_tip: "一击就倒。用钻出攻击一网打尽。",
    mob_sewer_rat: "沟鼠",
    mob_sewer_rat_lore: "守护深处水道的大块头。毒让它的身体更硬了。",
    mob_sewer_rat_atk: "抖很久，接着又远又猛地冲撞。很难击退。",
    mob_sewer_rat_tip: "撞墙发愣时是机会。用钻头把它顶到墙上。",
    mob_chipmunk: "花栗鼠",
    mob_chipmunk_lore: "勤快地收集橡子的邻居。现在拿橡子扔人。",
    mob_chipmunk_atk: "保持距离扔橡子。",
    mob_chipmunk_tip: "看准时机挥爪，橡子会被打回去。",
    mob_red_squirrel: "松鼠",
    mob_red_squirrel_lore: "在树梢上跳来跳去的灵巧邻居，被拉到了根下。",
    mob_red_squirrel_atk: "蹲下后跳起扑来。落地的周围会受伤。",
    mob_red_squirrel_tip: "离开地上的圈。落地之后有破绽。",
    mob_dummy: "稻草人",
    mob_dummy_lore: "波波做的练习对象。",
    mob_dummy_atk: "什么也不做。",
    mob_dummy_tip: "尽情打吧。",
    w_hp_1: "一击就倒",
    w_hp_2: "脆弱",
    w_hp_3: "结实",
    w_hp_4: "非常结实",
    w_pow_1: "有点疼",
    w_pow_2: "很疼",
    w_spd_1: "慢",
    w_spd_2: "一般",
    w_spd_3: "快",
    w_spd_4: "非常快",
    tab_bestiary: "图鉴",
    stat_power: "攻击",
    stat_aspd: "攻速",
    stat_dig_cd: "钻地冷却",
    stat_gauge: "{0}积累",
    stat_weapon: "武器",
    seeds: "种子",
    fmt_secs: "{0}秒",
    fmt_pct: "{0}%",
    fmt_hp: "{0} / {1}",
    wallet_note: "萤火虫用来唤醒技能，种子在商店使用（以后开放）。",
    best_unknown: "???",
    best_unseen: "还没有遇到。",
    best_hp: "体力",
    best_pow: "力量",
    best_spd: "速度",
    best_atk: "攻击",
    best_tip: "要领",
    best_count: "遇到的邻居 {0} / {1}",
    set_controls: "操作设置",
    act_attack: "攻击",
    act_dodge: "挖洞（闪避）",
    act_special: "必杀技",
    act_use_item: "使用道具",
    act_item_prev: "上一个道具",
    act_item_next: "下一个道具",
    act_map: "地图",
    ctl_key: "键盘",
    ctl_pad: "手柄",
    ctl_reset: "恢复默认",
    ctl_wait: "请按键… {0}",
    ctl_hint: "左右：键盘/手柄·确认：更改·重复时互换",
    ctl_fixed: "固定：移动 WASD·方向键·左杆·菜单 Esc·Start·确认 Enter·A·返回 Esc·B",
    popo_name: "波波",
    popo_hi: "喂，小鼹鼠！你没事吧？我是浣熊波波。",
    popo_hi2: "要去树根下面？那我先教你怎么活动身体。一步一步来！",
    popo_pickup: "看到那个发光的东西了吗？那是你的爪子。去捡起来吧。",
    popo_combo: "连续打稻草人三下。第三下最重！",
    popo_burrow: "那只花栗鼠会扔橡子。钻到地下就打不到你。现在是练习，不会受伤。",
    popo_emerge: "在地下移到花栗鼠下面，再按一次就会冲出来攻击！",
    popo_parry: "接下来是弹反。橡子快打到时挥爪，就能把它打回去。打回去的橡子会打中敌人！",
    popo_parry_skip: "很难吧？没关系，打着打着就有感觉了。",
    popo_special: "干得好！打中和弹反都会积累怒气。满了就能放必杀技！我分你一点力气。",
    popo_item: "哎呀，受了点伤。给你一颗树莓。吃了能回复一点体力。",
    popo_item2: "多汁蚯蚓能回满体力。带在身上的话，倒下也能再站起来一次。",
    popo_menu: "在菜单（Esc·Start）里能看状态和图鉴，按{0}可以直接打开地图。",
    popo_save: "绿色的菇圈是存档点。倒下的话会在那里重新站起来。",
    popo_bye: "洞口在那边下面。小心哦，Naïl！",
    tut_step: "[{0}/{1}] {2}",
    tut_parry: "橡子快打到时按{0}：弹反",
    tut_special: "{0}：必杀技！把稻草人顶到墙上",
    tut_item: "{0}：吃树莓",
    mob_chief: "田鼠首领",
    mob_chief_lore: "田鼠村的老首领。被根毒感染，守着南边的空地。",
    mob_chief_atk:
        "长时间颤抖后直线冲撞，蹲下后高高跳起砸下。累了会叫来小鼠群。攻击时被打也不会停。",
    mob_chief_tip: "引他撞上岩石或墙壁，他会晕好一阵。趁那时打他。",
    toast_chief: "打败了田鼠首领！洞口打开了。",
    toast_camp: "根下的营地。体力已回满。",
    toast_spring: "清澈的泉水。体力已回满。",
    toast_goal: "打败了洞穴深处的鼠群！出口打开了。",
    tut_boss: "打倒南边空地的田鼠首领",
    popo_glade: "我把南边空地的门打开。那里的田鼠首领被根染上了……让他清醒过来吧！",
    bori_name: "波利",
    chief_intro_1: "吱……！这是我的空地。谁也别想过去！",
    chief_intro_2: "请让开。我必须下去！",
    chief_beaten_1: "呜……头脑清醒过来了。我做了什么……",
    chief_beaten_2: "你还好吗？你被根染上了。",
    chief_beaten_3: "谢谢你，小鼹鼠。我是田鼠村的首领波利。",
    chief_beaten_4: "那个洞下面有个营地。我先下去生火。",
    chief_beaten_5: "那个孩子也能……像这样回来吗？",
    camp_1: "欢迎！这里是根下的营地。在火边好好休息吧。",
    camp_2: "带种子来，我的店里可以换东西。绿色的圈在这里也是存档点。",
    camp_3: "往下的根路在那边。每条岔路都通往不同的洞穴。好好选。",
    camp_again: "又走过了一条路。在火边歇歇吧。",
    bori_1: "被根迷住时的事像梦一样模糊。对不起啊。",
    bori_2: "洞穴尽头的鼠群很强。靠近墙壁战斗，更容易引它们冲撞。",
    bori_3: "看到泉水就去歇歇。越往深处，能休息的地方越少。",
    seat_1: "这是谁的位子……地上落着一片枯萎的花瓣。",
    node_burrow: "洞穴",
    node_burrow_desc: "老鼠住的洞穴。打败尽头的鼠群可得种子。",
    node_nest: "巢穴",
    node_nest_desc: "更强的鼠群聚集之处。有护符宝箱。",
    node_cache: "储藏洞",
    node_cache_desc: "小洞穴。橡子储藏处里有个宝箱。",
    node_spring: "泉水",
    node_spring_desc: "不用战斗就能休息。体力回满，还有野莓。",
    node_boss: "路之主",
    node_boss_desc: "守护这条路的首领。获胜就能前往下一个营地。",
    node_unknown: "?",
    node_unknown_desc: "还不知道。靠近就能看到。",
    route_title: "根之路 {0}",
    route_hint: "方向选择 · {0} 进入",
    route_hint_back: "方向选择 · {0} 进入 · 返回关闭",
    shop_title: "波波的店",
    shop_seeds: "种子 {0}",
    shop_hi: "欢迎！有种子的话什么都能换。",
    shop_poor: "种子不够……去洞穴里多收集一些吧！",
    shop_full: "包里已经放不下了。",
    shop_thanks: "谢谢！好好用哦。",
    shop_price: "{0} 种子",
    shop_heart: "根之心",
    shop_heart_desc: "最大体力+1。每个营地只有一个。",
    shop_hint: "{0} 购买 · 返回关闭",
    ia_shop: "{0} 商店",
    ia_talk: "{0} 交谈",
    ia_look: "{0} 查看",
    ia_route: "{0} 路线图",
    map_camp: "营地 {0}",
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn placeholders(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut rest = s;
        while let Some(i) = rest.find('{') {
            let Some(j) = rest[i..].find('}') else { break };
            out.insert(rest[i..i + j + 1].to_string());
            rest = &rest[i + j + 1..];
        }
        out
    }

    #[test]
    fn placeholders_match_across_languages() {
        let ko = KO.all();
        for &l in Lang::ALL {
            for ((key, a), (_, b)) in ko.iter().zip(l.strings().all()) {
                assert_eq!(placeholders(a), placeholders(b), "{l:?} {key}");
            }
        }
    }

    #[test]
    fn no_empty_strings() {
        for &l in Lang::ALL {
            for (key, v) in l.strings().all() {
                assert!(!v.trim().is_empty(), "{l:?} {key}");
            }
        }
    }

    /// 폰트는 서브셋이라(CLAUDE.md) 표의 글자가 그 언어 폰트에 없으면 빈칸으로 나온다. TTF cmap을 직접 읽어 확인한다.
    /// 일본어·중국어 표를 고쳐서 여기서 깨지면 tools/fonts/build_cjk.py를 다시 돌린다.
    #[test]
    fn every_glyph_is_in_the_font() {
        for &l in Lang::ALL {
            let path = format!("assets/{}", l.font_file());
            let cmap = font_cmap(&std::fs::read(&path).expect("font"));
            for (key, v) in l.strings().all() {
                for c in v.chars() {
                    assert!(
                        cmap.contains(&(c as u32)),
                        "{l:?} {key}: {path}에 없는 글자 {c:?} (U+{:04X})",
                        c as u32
                    );
                }
            }
        }
    }

    #[test]
    fn pseudo_keeps_placeholders_and_grows() {
        let p = pseudo_text("{0}-hit combo");
        assert!(p.contains("{0}"));
        assert!(p.chars().count() as f32 >= "{0}-hit combo".len() as f32 * 1.4);
        assert!(p.contains('ö'));
    }

    #[test]
    fn fill_reorders() {
        assert_eq!(fill("{1} then {0}", &[&1, &"a"]), "a then 1");
        assert_eq!(Lang::from_tag("ko-KR"), Some(Lang::Ko));
        assert_eq!(Lang::from_tag("en_US.UTF-8"), Some(Lang::En));
        assert_eq!(Lang::from_tag("ja"), Some(Lang::Ja));
        assert_eq!(Lang::from_tag("zh-CN"), Some(Lang::ZhHans));
        assert_eq!(Lang::from_tag("zh-Hans-SG"), Some(Lang::ZhHans));
        assert_eq!(Lang::from_tag("zh-TW"), None);
        assert_eq!(Lang::from_tag("zh-Hant-HK"), None);
        assert_eq!(Lang::from_tag("fr"), None);
        // 선호 목록에서 가진 첫 언어. 없으면 영어.
        assert_eq!(Lang::pick(["fr-FR", "ko-KR", "en"]), Lang::Ko);
        assert_eq!(Lang::pick(["fr-FR", "en-GB", "ko"]), Lang::En);
        assert_eq!(Lang::pick(["zh-TW", "ja"]), Lang::Ja);
        assert_eq!(Lang::pick(["de", "it"]), Lang::En);
        assert_eq!(Lang::pick(Vec::<String>::new()), Lang::En);
        assert_eq!(Lang::Ko.cycle(1), Lang::En);
        assert_eq!(Lang::Ko.cycle(-1), *Lang::ALL.last().unwrap());
    }

    /// TTF cmap 형식 4(BMP)와 12에서 코드 포인트 집합을 읽는다. 테스트 전용 최소 파서.
    fn font_cmap(f: &[u8]) -> BTreeSet<u32> {
        let u16_at = |o: usize| u16::from_be_bytes([f[o], f[o + 1]]) as usize;
        let u32_at = |o: usize| u32::from_be_bytes([f[o], f[o + 1], f[o + 2], f[o + 3]]) as usize;
        let num_tables = u16_at(4);
        let cmap = (0..num_tables)
            .map(|i| 12 + i * 16)
            .find(|&r| &f[r..r + 4] == b"cmap")
            .map(|r| u32_at(r + 8))
            .expect("cmap table");
        let mut out = BTreeSet::new();
        for i in 0..u16_at(cmap + 2) {
            let sub = cmap + u32_at(cmap + 4 + i * 8 + 4);
            match u16_at(sub) {
                4 => {
                    let segs = u16_at(sub + 6) / 2;
                    let ends = sub + 14;
                    let starts = ends + segs * 2 + 2;
                    let deltas = starts + segs * 2;
                    let ranges = deltas + segs * 2;
                    for s in 0..segs {
                        let (start, end) = (u16_at(starts + s * 2), u16_at(ends + s * 2));
                        let delta = u16_at(deltas + s * 2);
                        let ro = u16_at(ranges + s * 2);
                        for c in start..=end.min(0xFFFE) {
                            let glyph = if ro == 0 {
                                (c + delta) & 0xFFFF
                            } else {
                                let at = ranges + s * 2 + ro + (c - start) * 2;
                                let g = u16_at(at);
                                if g == 0 {
                                    0
                                } else {
                                    (g + delta) & 0xFFFF
                                }
                            };
                            if glyph != 0 {
                                out.insert(c as u32);
                            }
                        }
                    }
                }
                12 => {
                    let groups = u32_at(sub + 12);
                    for g in 0..groups {
                        let o = sub + 16 + g * 12;
                        for c in u32_at(o)..=u32_at(o + 4) {
                            out.insert(c as u32);
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }
}
