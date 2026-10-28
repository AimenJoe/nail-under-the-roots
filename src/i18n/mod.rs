//! 다국어(docs/decisions/0013, 0014). 플레이어가 보는 글은 전부 여기 언어별 표 한 곳에 있다.
//!
//! - 키는 `Strings`의 필드 이름. 한 언어에 필드가 빠지면 컴파일이 안 된다.
//! - 숫자가 들어가는 글은 `{0}`, `{1}` 자리표시자 + `fill()`. 언어마다 어순이 달라도 된다. 문장을 조각으로 잇지 않는다.
//! - 다른 모듈은 글 대신 `Tr`(표에서 글을 고르는 함수)을 들고 있다가, 그릴 때 `tr()`로 꺼낸다.
//!   월드 로직은 지금 언어를 모른다.
//! - 언어마다 폰트가 다르다(`Lang::font_file`). 한국어·영어·브라질 포르투갈어·프랑스어·스페인어·독일어·러시아어·폴란드어는 Galmuri, 일본어·중국어(간체·번체)는 Ark Pixel에서 만든 서브셋
//!   (tools/fonts/build_cjk.py — 표를 고치면 다시 돌린다).
//! - 개발 전용 화면(시나리오 목록, 디버그 오버레이)은 번역하지 않고 기본 폰트로 찍는다.
//! - 개발 빌드에는 가짜 언어 "Pseudo"(영어를 악센트 + 40% 늘림)가 있다. 넘치는 칸·번역 안 된 글을 미리 찾는다.
//! - 테스트: 모든 언어의 모든 글이 그 언어 폰트에 있는 글자로만 되어 있는지, 자리표시자가 언어끼리 같은지.

// 0169 R6: 언어 표는 한 파일에 하나씩. `Lang::strings()`가 고른다.
mod de;
mod en;
mod es;
mod fr;
mod ja;
mod ko;
mod pl;
mod pt_br;
mod ru;
mod zh_hans;
mod zh_hant;

pub use de::DE;
pub use en::EN;
pub use es::ES;
pub use fr::FR;
pub use ja::JA;
pub use ko::KO;
pub use pl::PL;
pub use pt_br::PT_BR;
pub use ru::RU;
pub use zh_hans::ZH_HANS;
pub use zh_hant::ZH_HANT;

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
    /// 브라질 포르투갈어(docs/decisions/0033). 폰트는 한국어와 같은 Galmuri(라틴 악센트가 다 있다).
    PtBr,
    /// 프랑스어(docs/decisions/0084). Galmuri(œ Œ Ÿ를 서브셋에 더했다).
    Fr,
    /// 스페인어(docs/decisions/0084, 중립 스페인어). Galmuri.
    Es,
    /// 독일어(docs/decisions/0085). Galmuri(Latin-1 + „ “).
    De,
    /// 러시아어(docs/decisions/0085). Galmuri(키릴을 서브셋에 더했다).
    Ru,
    /// 중국어 번체(대만 표준, docs/decisions/0144). 대만·홍콩·마카오. 폰트는 Ark Pixel 번체 모양 서브셋.
    ZhHant,
    /// 폴란드어(docs/decisions/0144). Galmuri(라틴 확장 A를 서브셋에 더했다).
    Pl,
    #[cfg(feature = "dev")]
    Pseudo,
}

impl Lang {
    /// 설정에서 넘기는 순서.
    #[cfg(not(feature = "dev"))]
    pub const ALL: &'static [Lang] = &[
        Lang::Ko,
        Lang::En,
        Lang::Ja,
        Lang::ZhHans,
        Lang::ZhHant,
        Lang::PtBr,
        Lang::Fr,
        Lang::Es,
        Lang::De,
        Lang::Ru,
        Lang::Pl,
    ];
    #[cfg(feature = "dev")]
    pub const ALL: &'static [Lang] = &[
        Lang::Ko,
        Lang::En,
        Lang::Ja,
        Lang::ZhHans,
        Lang::ZhHant,
        Lang::PtBr,
        Lang::Fr,
        Lang::Es,
        Lang::De,
        Lang::Ru,
        Lang::Pl,
        Lang::Pseudo,
    ];

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::Ko => &KO,
            Lang::En => &EN,
            Lang::Ja => &JA,
            Lang::ZhHans => &ZH_HANS,
            Lang::PtBr => &PT_BR,
            Lang::Fr => &FR,
            Lang::Es => &ES,
            Lang::De => &DE,
            Lang::Ru => &RU,
            Lang::ZhHant => &ZH_HANT,
            Lang::Pl => &PL,
            #[cfg(feature = "dev")]
            Lang::Pseudo => pseudo(),
        }
    }

    /// 이 언어를 찍는 폰트(assets 기준 경로).
    pub fn font_file(self) -> &'static str {
        match self {
            Lang::Ja => "fonts/NailCJK-ja.ttf",
            Lang::ZhHans => "fonts/NailCJK-zh-Hans.ttf",
            Lang::ZhHant => "fonts/NailCJK-zh-Hant.ttf",
            _ => BASE_FONT,
        }
    }

    /// BCP 47 태그("ko-KR", "en-US", "zh-Hans-CN", "en_US.UTF-8")가 우리가 가진 언어면 그 언어.
    /// 중국어는 번체(zh-Hant, zh-TW·HK·MO)와 간체(그 밖)를 가른다. 글자 체계가 달라 서로 바꾸지 않는다(0144).
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let lower = tag.to_ascii_lowercase();
        let mut parts = lower.split(['-', '_', '.', '@']);
        let primary = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();
        match primary {
            "ko" => Some(Lang::Ko),
            "en" => Some(Lang::En),
            "ja" => Some(Lang::Ja),
            // zh-Hans-HK처럼 문자 표시가 있으면 그것이 먼저.
            "zh" if rest.contains(&"hans") => Some(Lang::ZhHans),
            "zh" if rest
                .iter()
                .any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo")) =>
            {
                Some(Lang::ZhHant)
            }
            "zh" => Some(Lang::ZhHans),
            // 포르투갈(pt-PT)도 브라질 포르투갈어로: 글자는 같고 낱말·존댓말이 조금 다르다(영어보다는 가깝다).
            "pt" => Some(Lang::PtBr),
            // 캐나다·벨기에·스위스 프랑스어도, 스페인·중남미 스페인어도 하나씩(0084).
            "fr" => Some(Lang::Fr),
            "es" => Some(Lang::Es),
            // 오스트리아·스위스 독일어도 하나로(0085).
            "de" => Some(Lang::De),
            "ru" => Some(Lang::Ru),
            "pl" => Some(Lang::Pl),
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
                v.extend("·…▶▼▲→←♥♡ï".chars());
                // 표 밖에서 찍는 라틴 글자(버튼 이름 "Enter"·"J", 타이틀 "Naïl", 개발 화면). 빠지면 첫 프레임에 아틀라스가 커진다.
                v.extend((' '..='~').filter(|c| !c.is_whitespace()));
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
    title_start, title_settings, title_dev, title_controls, title_pick, version, copyright,
    // 저장 슬롯(docs/decisions/0078)
    title_continue, title_new, slot_overwrite, slot_name, slot_empty, slot_summary, slot_summary_start, slot_resting, slot_erase, slot_erase_again, slots_hint,
    // 설정
    settings, set_sound, set_music, set_shake, set_effects, set_debug, set_language, set_back, on, off, settings_hint,
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
    // 환경
    biome_meadow, biome_roots, biome_deep, biome_silk, depth_m, depth_short,
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
    toast_save, toast_respawn, toast_fall, toast_fall_root, toast_fall_last, toast_run_lost, toast_boss, gift_charm, map_field, map_dungeon, map_legend, toast_parry,
    // 적 종류·도감·게이지(docs/decisions/0021)
    gauge_rage, gauge_ki, mob_field_mouse, mob_field_mouse_lore, mob_field_mouse_atk, mob_field_mouse_tip, mob_house_mouse, mob_house_mouse_lore, mob_house_mouse_atk, mob_house_mouse_tip, mob_sewer_rat, mob_sewer_rat_lore, mob_sewer_rat_atk, mob_sewer_rat_tip, mob_chipmunk, mob_chipmunk_lore, mob_chipmunk_atk, mob_chipmunk_tip, mob_red_squirrel, mob_red_squirrel_lore, mob_red_squirrel_atk, mob_red_squirrel_tip, mob_shrew, mob_shrew_lore, mob_shrew_atk, mob_shrew_tip, mob_hedgehog, mob_hedgehog_lore, mob_hedgehog_atk, mob_hedgehog_tip, mob_toad, mob_toad_lore, mob_toad_atk, mob_toad_tip, mob_bat, mob_bat_lore, mob_bat_atk, mob_bat_tip, mob_spore, mob_spore_lore, mob_spore_atk, mob_spore_tip, mob_dummy, mob_dummy_lore, mob_dummy_atk, mob_dummy_tip, w_hp_1, w_hp_2, w_hp_3, w_hp_4, w_pow_1, w_pow_2, w_spd_1, w_spd_2, w_spd_3, w_spd_4,
    // 상태 탭·도감 탭
    tab_bestiary, stat_power, stat_aspd, stat_dig_cd, stat_gauge, stat_weapon, seeds, fmt_secs, fmt_pct, fmt_hp, wallet_note, best_unknown, best_unseen, best_hp, best_pow, best_spd, best_atk, best_tip, best_count,
    // 조작 설정(docs/decisions/0022)
    set_controls, act_attack, act_dodge, act_special, act_use_item, act_item_prev, act_item_next, act_map, ctl_key, ctl_pad, ctl_reset, ctl_wait, ctl_hint, ctl_fixed,
    // 튜토리얼: 라쿤 포포(docs/decisions/0023)
    popo_name, popo_hi, popo_hi2, popo_hi0, op_stop, toast_petal, toast_petal_trail, best_petal, popo_pickup, popo_combo, popo_heavy, popo_burrow, popo_emerge, popo_parry, popo_parry_skip, popo_special, popo_item, popo_item2, popo_menu, popo_save, popo_bye, tut_step, tut_heavy, tut_parry, tut_special, tut_item,
    // 스테이지 고리: 튜토리얼 보스·쉼터·길 지도·가게 (docs/decisions/0030~0032)
    mob_chief, mob_chief_lore, mob_chief_atk, mob_chief_tip, toast_chief, toast_camp, toast_spring, toast_eat_full, toast_goal, tut_boss, popo_glade, bori_name, chief_intro_1, chief_intro_2, chief_beaten_1, chief_beaten_2, chief_beaten_3, chief_beaten_4, chief_beaten_5, camp_1, camp_2, camp_3, camp_again, bori_1, bori_2, bori_3, seat_1, node_burrow, node_burrow_desc, node_nest, node_nest_desc, node_cache, node_cache_desc, node_spring, node_spring_desc, node_elite, node_elite_desc, node_boss, node_boss_desc, node_warren, node_warren_desc, node_unknown, node_unknown_desc, route_title, route_hint, route_hint_back, shop_title, shop_seeds, shop_hi, shop_poor, shop_full, shop_thanks, shop_price, shop_heart, shop_heart_desc, shop_hint, ia_shop, ia_talk, ia_look, ia_route, map_camp,
    // 가게: 사기 전에 묻기
    shop_confirm, shop_yes, shop_no,
    // 장치별 안내(키보드·패드·터치)
    v_touch, v_touch_out, v_touch_here, v_drag, v_stick, title_controls_touch, tab_hint_touch, route_hint_touch, route_hint_back_touch,
    // 이야기 컷씬: 오프닝·보리·모모·엔딩(docs/decisions/0037, 0039)
    name_mom, name_halmae, name_momo, op_n1, op_n2, op_n3, op_mom_1, op_mom_2, bori_free, momo_r1, momo_r2, momo_r3, momo_r4, momo_r5, half_1, half_2, half_3, half_4, half_5, true_1, true_2, true_3, true_4, epi_1, epi_2, epi_clover,
    // 다회차: 앨범·도움 모드·기록 지우기·깨어날 때 한마디(docs/decisions/0041)
    title_album, album_count, album_hint, album_close, set_assist, set_erase, set_erase_confirm, ret_first, ret_roots, ret_bori, ret_assist, ret_ten, run_again, run_again_bori,
    // 런의 끝(docs/decisions/0042)
    node_deepest, node_deepest_desc, run_after_half, deep_sub, deep_1, deep_k1, deep_k2, deep_k3, deep_wrap, deep_sleep,
    // 실뜨기(docs/decisions/0043)
    knot_title, knot_intro, knot_level, knot_none, knot_locked, knot_hint, knot_1, knot_2, knot_3, knot_4, knot_5, knot_6, knot_7, knot_8, knot_9, knot_10, toast_knot_open,
    // 오늘의 굴(docs/decisions/0044)
    title_daily, daily_best, daily_clear, toast_daily_clear,
    // 마을(docs/decisions/0045)
    village_name, decor_title, village_seeds, decor_owned, decor_hi, decor_poor, decor_thanks, decor_hint, decor_1, decor_2, decor_3, decor_4, decor_5, decor_6, village_intro, halmae_arrive, ia_decor, ia_descend, ia_knots,
    // 모모(docs/decisions/0046)
    mob_momo, mob_momo_lore, mob_momo_atk, mob_momo_tip, momo_meet_1, momo_meet_1b, momo_meet_2, momo_meet_2b, momo_meet_3, momo_meet_3b, momo_flee_1, momo_flee_2, momo_flee_wait, momo_petal, momo_r6, momo_r7, seat_p1, seat_p2, seat_p3, momo_v1, momo_v2, momo_v3, bori_momo, popo_momo, momo_v4, bori_clover, halmae_clover,
    // 큰 나무의 재주(docs/decisions/0047)
    tal_burrow, tal_burrow_a, tal_burrow_a_desc, tal_burrow_b, tal_burrow_b_desc, tal_parry, tal_parry_a, tal_parry_a_desc, tal_parry_b, tal_parry_b_desc, tal_drill, tal_drill_a, tal_drill_a_desc, tal_drill_b, tal_drill_b_desc, tal_eat, tal_eat_a, tal_eat_a_desc, tal_eat_b, tal_eat_b_desc, tal_hop, tal_hop_a, tal_hop_a_desc, tal_hop_b, tal_hop_b_desc, tal_chain, tal_chain_a, tal_chain_a_desc, tal_chain_b, tal_chain_b_desc, c_clover, c_clover_desc, c_cotton, c_cotton_desc, c_crown, c_crown_desc, c_knot, c_knot_desc, tree_title, tree_light, tree_cost, tree_poor, tree_awake, tree_equip, tree_unequip, tree_on, tree_off, tree_hint, ia_tree, toast_camp_bank, tree_intro,
    // 저널(docs/decisions/0048)
    cat_story, cat_fight, cat_explore, cat_neighbors, cat_village, j_reward, j_claim, j_progress, j_toast, r_light, r_decor, r_charm, fur_0, fur_1, fur_2, fur_3, puddle_none, ia_puddle, decor_7, decor_8, decor_9, c_bell, c_bell_desc, c_thread, c_thread_desc, c_petal, c_petal_desc, jg_tutorial, jg_tutorial_d, jg_bori, jg_bori_d, jg_camp, jg_camp_d, jg_deeper, jg_deeper_d, jg_deepest, jg_deepest_d, jg_momo1, jg_momo1_d, jg_momo3, jg_momo3_d, jg_kills, jg_kills_d, jg_kills2, jg_kills2_d, jg_ambush, jg_ambush_d, jg_ambush2, jg_ambush2_d, jg_parry, jg_parry_d, jg_parry2, jg_parry2_d, jg_drill, jg_drill_d, jg_slam, jg_slam_d, jg_finisher, jg_finisher_d, jg_chest, jg_chest_d, jg_chest2, jg_chest2_d, jg_spring, jg_spring_d, jg_berry, jg_berry_d, jg_runs, jg_runs_d, jg_falls, jg_falls_d, jg_daily, jg_daily_d, jg_mobs, jg_mobs_d, jg_mobs2, jg_mobs2_d, jg_light, jg_light_d, jg_talent, jg_talent_d, jg_talent2, jg_talent2_d, jg_decor, jg_decor_d, jg_decor2, jg_decor2_d, jg_knot, jg_knot_d, jg_mobs3, jg_mobs3_d, jg_heavy, jg_heavy_d, jg_heavy2, jg_heavy2_d, j_done, j_waiting,
    // 저널 보상이 어디 있나(docs/decisions/0050)
    j_got_light, j_got_seeds, j_got_decor, j_got_fur, j_got_charm,
    // 상태 탭의 재주(docs/decisions/0047)
    stat_talents, tal_none,
    // 도감 설명 쪽(docs/decisions/0051)
    best_lore,
    // 이웃 말 걸기(docs/decisions/0051)
    bori_t_momo, bori_t_halmae, bori_t_falls, bori_t_tree, momo_t_deep, momo_t_bori,
    // 도토(docs/decisions/0052)
    mob_doto, mob_doto_lore, mob_doto_atk, mob_doto_tip, name_doto, doto_meet_1, doto_meet_1b, doto_b1, doto_b2, doto_b3, doto_b4, doto_b5, doto_b6, doto_memory, doto_v1, doto_v2, doto_v3, bori_t_doto, momo_t_doto, popo_doto, tree_claw, tree_venom_desc, tree_venom_on, jg_doto, jg_doto_d, jg_true, jg_true_d,
    // 수수(docs/decisions/0053)
    name_susu, susu_1, susu_2, susu_3, susu_4, susu_5, susu_v1, susu_v2, susu_v3, jg_susu, jg_susu_d,
    // 진행을 따르는 이웃 말 더(docs/decisions/0053)
    bori_t_susu, bori_t_true, momo_t_knot, momo_t_true, doto_t_venom, doto_t_true, popo_true,
    // 콩이(docs/decisions/0054)
    mob_kongi, mob_kongi_lore, mob_kongi_atk, mob_kongi_tip, name_kongi, mob_clover, mob_clover_lore, mob_clover_atk, mob_clover_tip, name_clover, clover_meet_1, clover_meet_1b, clover_meet_momo_1, clover_meet_momo_1b, clover_b1, clover_cocoon, toast_cocoon, kongi_meet_1, kongi_meet_1b, kongi_b1, kongi_b2, kongi_b3, kongi_b4, kongi_b5, kongi_b6, kongi_door, kongi_v1, kongi_v2, kongi_v3, clover_b2, clover_b3, clover_b2_momo, clover_b3_momo, clover_v1, clover_v2, clover_v3, toast_shell, toast_spines, toast_thorn, toast_thorn_momo, toast_web, toast_web_momo, toast_guard_break, jg_kongi, jg_kongi_d, jg_clover, jg_clover_d,
    // 실의 꿈(docs/decisions/0055)
    dream_title, dream_intro, dream_locked, dream_all, dream_none, dream_legend, dream_hint, toast_dream_in, toast_dream_next, toast_dream_win, toast_dream_best, toast_dream_wake, ia_dream, halmae_dream, jg_dream, jg_dream_d, jg_dream2, jg_dream2_d,
    // 주인공 이름(한국어는 나일, docs/decisions/0057)
    name_nail,
    // 모모와 내려가기(docs/decisions/0056)
    w_kick, w_kick_desc, toast_shell_momo, stat_hop_cd, pick_title, pick_nail, pick_momo, pick_clears, pick_hint, mend_1, mend_2, mend_3, mend_4, run_momo, seat_momo, halmae_petal, jg_momo_deep, jg_momo_deep_d,
    // 안내 화면(docs/decisions/0178)
    pm_help, h_move, h_move_d, h_combo, h_combo_d, h_heavy, h_heavy_d, h_burrow, h_burrow_d, h_parry, h_parry_d, h_special, h_special_d, h_eat, h_eat_d, h_enemy, h_enemy_d, h_loot, h_loot_d, h_camp, h_camp_d, h_gear, h_gear_d, h_break, h_break_d, h_village, h_village_d,
    // 플레이어블 도토(0195)
    pick_doto, stat_leap_cd, toast_shell_doto, toast_thorn_doto, toast_web_doto, gauge_acorn, w_acorn, w_acorn_desc, tal_leap, tal_leap_a, tal_leap_a_desc, tal_leap_b, tal_leap_b_desc, tal_bury, tal_bury_a, tal_bury_a_desc, tal_bury_b, tal_bury_b_desc,
    c_pinecone, c_pinecone_desc, c_acorn_cap, c_acorn_cap_desc, c_maple, c_maple_desc, jg_doto_deep, jg_doto_deep_d,
    set_tells,
    h_doto, h_doto_d, h_run, h_run_d,
    run_doto, seat_doto, dend_1, dend_2, dend_3, dend_4, doto_t_deep,
}

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
        assert_eq!(Lang::from_tag("pt-BR"), Some(Lang::PtBr));
        assert_eq!(Lang::from_tag("pt-PT"), Some(Lang::PtBr));
        assert_eq!(Lang::pick(["pt-BR", "en-US"]), Lang::PtBr);
        assert_eq!(Lang::from_tag("zh-TW"), Some(Lang::ZhHant));
        assert_eq!(Lang::from_tag("zh-Hant-HK"), Some(Lang::ZhHant));
        assert_eq!(Lang::from_tag("zh_HK"), Some(Lang::ZhHant));
        assert_eq!(Lang::from_tag("zh-Hans-HK"), Some(Lang::ZhHans));
        assert_eq!(Lang::from_tag("pl-PL"), Some(Lang::Pl));
        assert_eq!(Lang::from_tag("fr-CA"), Some(Lang::Fr));
        assert_eq!(Lang::from_tag("es-419"), Some(Lang::Es));
        assert_eq!(Lang::from_tag("de-AT"), Some(Lang::De));
        assert_eq!(Lang::from_tag("ru-RU"), Some(Lang::Ru));
        assert_eq!(Lang::from_tag("it"), None);
        // 선호 목록에서 가진 첫 언어. 없으면 영어.
        assert_eq!(Lang::pick(["it-IT", "ko-KR", "en"]), Lang::Ko);
        assert_eq!(Lang::pick(["it-IT", "en-GB", "ko"]), Lang::En);
        assert_eq!(Lang::pick(["fr-FR", "en"]), Lang::Fr);
        assert_eq!(Lang::pick(["zh-TW", "ja"]), Lang::ZhHant);
        assert_eq!(Lang::pick(["it", "nl"]), Lang::En);
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
