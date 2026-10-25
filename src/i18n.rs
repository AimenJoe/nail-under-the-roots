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
    c_shell, c_shell_desc, c_whisker, c_whisker_desc, c_soil, c_soil_desc, c_flint, c_flint_desc, c_amulet, c_amulet_desc, it_berry, it_berry_desc, it_worm, it_worm_desc, toast_seed, toast_revive, quick_hint, special_name, special_desc, charms_label, charms_none, stat_attack, charm_equip, charm_unequip, bag_items, charm_slots,
    // 모험(docs/decisions/0019)
    toast_save, toast_respawn, toast_boss, gift_charm, map_field, map_dungeon, map_legend, toast_parry,
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
    settings_hint: "좌우로 값 바꾸기 · 설정은 아직 저장되지 않음",
    tab_menu: "메뉴",
    tab_status: "상태",
    tab_bag: "가방",
    tab_journal: "저널",
    tab_map: "지도",
    tab_hint: "Q/E · L1/R1: 탭",
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
    tut_attack: "공격(J · X · ATK)을 세 번: 세 번째가 묵직하다",
    tut_burrow: "굴파기(K · A · DIG): 땅속에서는 맞지 않는다",
    tut_emerge: "땅속에서 적 아래로 가서 다시 누르면 솟구친다",
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
    toast_seed: "생명의 씨앗! 최대 체력이 늘었다.",
    toast_revive: "쥬시 지렁이를 먹고 다시 일어났다!",
    quick_hint: "F·Y 먹기",
    special_name: "드릴 돌진",
    special_desc: "SP가 차면 L·B. 곧게 연타하며 돌진, 벽에 부딪히면 쿵!",
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
    settings_hint: "Left/Right to change · Settings are not saved yet",
    tab_menu: "Menu",
    tab_status: "Status",
    tab_bag: "Bag",
    tab_journal: "Journal",
    tab_map: "Map",
    tab_hint: "Q/E · L1/R1: tabs",
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
    tut_attack: "Attack (J · X · ATK) three times: the third one is heavy",
    tut_burrow: "Burrow (K · A · DIG): you can't be hit underground",
    tut_emerge: "Underground, get beneath an enemy and press again to burst out",
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
    toast_seed: "Seed of Life! Max HP increased.",
    toast_revive: "Ate a Juicy Worm and got back up!",
    quick_hint: "F·Y eat",
    special_name: "Drill Dash",
    special_desc: "When SP is full: L·B. Dash forward drilling; hit a wall for a big bonk!",
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
    settings_hint: "左右で変更・設定はまだ保存されません",
    tab_menu: "メニュー",
    tab_status: "つよさ",
    tab_bag: "どうぐ",
    tab_journal: "きろく",
    tab_map: "ちず",
    tab_hint: "Q/E・L1/R1",
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
    tut_attack: "こうげき（J・X・ATK）を3回：3回目が重い",
    tut_burrow: "あなほり（K・A・DIG）：地中ではやられない",
    tut_emerge: "地中で敵の下へ行き、もう一度おすととびだす",
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
    toast_seed: "命の種！最大HPがふえた。",
    toast_revive: "ジューシーミミズを食べて起き上がった！",
    quick_hint: "F・Y たべる",
    special_name: "ドリル突進",
    special_desc: "SPがたまったらL・B。まっすぐ連打で突進、かべにぶつかるとドン！",
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
    settings_hint: "左右切换·设置尚未保存",
    tab_menu: "菜单",
    tab_status: "状态",
    tab_bag: "背包",
    tab_journal: "日志",
    tab_map: "地图",
    tab_hint: "Q/E·L1/R1：切换",
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
    tut_attack: "攻击（J·X·ATK）三次：第三下很重",
    tut_burrow: "挖洞（K·A·DIG）：在地下不会被打到",
    tut_emerge: "在地下移到敌人下方，再按一次就会冲出",
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
    toast_seed: "生命之种！最大体力增加了。",
    toast_revive: "吃了多汁蚯蚓，重新站起来了！",
    quick_hint: "F·Y 吃",
    special_name: "钻头冲刺",
    special_desc: "SP满时按L·B。连续钻击直线冲刺，撞墙会轰！",
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
