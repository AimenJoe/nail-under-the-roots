//! 다국어(docs/decisions/0013). 플레이어가 보는 글은 전부 여기 언어별 표 한 곳에 있다.
//!
//! - 키는 `Strings`의 필드 이름. 한 언어에 필드가 빠지면 컴파일이 안 된다.
//! - 숫자가 들어가는 글은 `{0}`, `{1}` 자리표시자 + `fill()`. 언어마다 어순이 달라도 된다. 문장을 조각으로 잇지 않는다.
//! - 다른 모듈은 글 대신 `Tr`(표에서 글을 고르는 함수)을 들고 있다가, 그릴 때 `tr()`로 꺼낸다.
//!   월드 로직은 지금 언어를 모른다.
//! - 개발 전용 화면(시나리오 목록, 디버그 오버레이)은 번역하지 않는다.
//! - 테스트: 모든 언어의 모든 글이 폰트에 있는 글자로만 되어 있는지, 자리표시자가 언어끼리 같은지.

use std::sync::atomic::{AtomicU8, Ordering};

/// 표에서 글 하나를 고르는 함수. 데이터 표(무기·퀘스트·컷씬)가 글 대신 이걸 든다.
pub type Tr = fn(&Strings) -> &'static str;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ko,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Ko, Lang::En];

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::Ko => &KO,
            Lang::En => &EN,
        }
    }

    /// BCP 47 태그("ko-KR", "en-US", "ko", "en_US.UTF-8")가 우리가 가진 언어면 그 언어.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "ko" => Some(Lang::Ko),
            "en" => Some(Lang::En),
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

/// 기기 언어로 시작 언어를 고른다: 선호 목록에서 우리가 가진 첫 언어, 없으면 영어.
/// 웹은 브라우저의 navigator.languages(index.html의 nail_locale 플러그인), 그 밖은 OS 선호 언어(sys-locale).
pub fn detect() -> Lang {
    Lang::pick(preferred_tags())
}

/// 웹: JS가 i번째 선호 언어의 주 언어 코드(두 글자)를 u32 하나로 접어 준다(문자열을 넘기지 않으려고).
/// 0이면 목록 끝. 없으면(옛 index.html) miniquad가 빈 함수를 끼워 넣어 0이 된다.
#[cfg(target_arch = "wasm32")]
fn preferred_tags() -> Vec<String> {
    extern "C" {
        fn nail_locale(i: u32) -> u32;
    }
    (0..8)
        // SAFETY: 정수 하나를 받고 돌려주는 JS 함수(web/index.html).
        .map(|i| unsafe { nail_locale(i) })
        .take_while(|code| *code != 0)
        .map(|code| {
            let b = [(code >> 8) as u8, code as u8];
            String::from_utf8_lossy(&b).into_owned()
        })
        .collect()
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_locale_crate_version() -> u32 {
    1
}

#[cfg(not(target_arch = "wasm32"))]
fn preferred_tags() -> Vec<String> {
    sys_locale::get_locales().collect()
}

macro_rules! strings {
    ($($key:ident),* $(,)?) => {
        /// 언어 하나의 글 전부.
        pub struct Strings {
            $(pub $key: &'static str,)*
        }

        impl Strings {
            /// (키, 글) 전부. 테스트용.
            #[cfg(test)]
            pub fn all(&self) -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($key), self.$key)),*]
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
    mole_desc, hp, this_run, run_stats, combo_stats, bare_hands, bare_hands_desc,
    burrow, burrow_stats, special, special_tbd,
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
    mole_desc: "아기 두더지. 발톱으로 싸우고 발톱으로 판다.",
    hp: "체력",
    this_run: "이번 런",
    run_stats: "웨이브 {0}  처치 {1}  시간 {2}",
    combo_stats: "{0}연타. 마지막 타 {1}s, 넉백 {2}",
    bare_hands: "맨손",
    bare_hands_desc: "아직 발톱이 없다. 공격할 수 없다.",
    burrow: "굴파기",
    burrow_stats: "땅속 {0}s 무적, 솟구침 반경 {1}px, 쿨다운 {2}s",
    special: "특기",
    special_tbd: "(미정 — 2단계)",
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
    mole_desc: "A baby mole. Fights with claws, digs with claws.",
    hp: "HP",
    this_run: "This run",
    run_stats: "Wave {0}  Kills {1}  Time {2}",
    combo_stats: "{0}-hit combo. Finisher {1}s, knockback {2}",
    bare_hands: "Bare paws",
    bare_hands_desc: "No claws yet. You can't attack.",
    burrow: "Burrow",
    burrow_stats: "Invulnerable {0}s, burst {1}px, cooldown {2}s",
    special: "Special",
    special_tbd: "(TBD — phase 2)",
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
        for l in Lang::ALL {
            for ((key, a), (_, b)) in ko.iter().zip(l.strings().all()) {
                assert_eq!(placeholders(a), placeholders(b), "{l:?} {key}");
            }
        }
    }

    #[test]
    fn no_empty_strings() {
        for l in Lang::ALL {
            for (key, v) in l.strings().all() {
                assert!(!v.trim().is_empty(), "{l:?} {key}");
            }
        }
    }

    /// 폰트는 서브셋이라(CLAUDE.md) 표의 글자가 폰트에 없으면 빈칸으로 나온다. TTF cmap을 직접 읽어 확인한다.
    #[test]
    fn every_glyph_is_in_the_font() {
        let font = std::fs::read("assets/fonts/Galmuri11-Condensed.ttf").expect("font");
        let cmap = font_cmap(&font);
        for l in Lang::ALL {
            for (key, v) in l.strings().all() {
                for c in v.chars() {
                    assert!(
                        cmap.contains(&(c as u32)),
                        "{l:?} {key}: 폰트에 없는 글자 {c:?} (U+{:04X})",
                        c as u32
                    );
                }
            }
        }
    }

    #[test]
    fn fill_reorders() {
        assert_eq!(fill("{1} then {0}", &[&1, &"a"]), "a then 1");
        assert_eq!(Lang::from_tag("ko-KR"), Some(Lang::Ko));
        assert_eq!(Lang::from_tag("en_US.UTF-8"), Some(Lang::En));
        assert_eq!(Lang::from_tag("ja"), None);
        // 선호 목록에서 가진 첫 언어. 없으면 영어.
        assert_eq!(Lang::pick(["ja-JP", "ko-KR", "en"]), Lang::Ko);
        assert_eq!(Lang::pick(["fr-FR", "en-GB", "ko"]), Lang::En);
        assert_eq!(Lang::pick(["ja", "zh-Hans"]), Lang::En);
        assert_eq!(Lang::pick(Vec::<String>::new()), Lang::En);
        assert_eq!(Lang::Ko.cycle(1), Lang::En);
        assert_eq!(Lang::Ko.cycle(-1), Lang::En);
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
