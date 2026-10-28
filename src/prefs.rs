//! 설정 저장(소리·흔들림·언어·조작). 웹은 브라우저 localStorage(web/index.html의 nail_prefs 플러그인)에
//! 정수 칸 몇 개로 둔다. 데스크톱·Android·iOS는 기록 폴더(`profile::data_dir`)의 `prefs.txt`에 같은 칸을
//! 줄 하나로 적는다(0175. 전엔 저장하지 않아 켤 때마다 기본값이었다 — 사장: "업데이트하면 언어가 사라진다").
//!
//! 칸: 0 표식(MAGIC), 1 소리+1, 2 흔들림+1, 3 언어(0 = 기기 언어), 4..11 키 번호+1, 11..18 패드 번호+1,
//! 18 음악+1(0081), 19 입자 효과+1(0141), 20 공격 예고 진하기+1(0198). 뒤에 붙인 칸이라 옛 저장은 0 = 기본값으로 읽는다 → MAGIC은 그대로.
//! 0은 "없음"(플러그인이 없거나 저장 전). 표식이 다르면 통째로 무시한다(형식이 바뀌면 MAGIC을 올린다).

use crate::i18n::Lang;
use crate::input::Bindings;

const MAGIC: u32 = 0x4E41_0001;
/// 저장 칸 수.
const CELLS: usize = 21;
/// 공격 예고 진하기 기본값(1..=10, 0198).
pub const TELLS_DEFAULT: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prefs {
    pub volume: u8,
    pub music: u8,
    pub shake: u8,
    /// 입자 효과(0..=10, 0141). 접근성: 먼지·불똥·환경 입자를 줄인다.
    pub effects: u8,
    /// 공격 예고 진하기(1..=10, 0198).
    pub tells: u8,
    /// None = 기기 언어를 따른다.
    pub lang: Option<Lang>,
    pub bindings: Bindings,
}

/// 저장 칸에 쓰는 언어 번호(가짜 언어는 저장하지 않는다).
fn lang_code(l: Lang) -> u32 {
    match l {
        Lang::Ko => 1,
        Lang::En => 2,
        Lang::Ja => 3,
        Lang::ZhHans => 4,
        Lang::PtBr => 5,
        Lang::Fr => 6,
        Lang::Es => 7,
        Lang::De => 8,
        Lang::Ru => 9,
        Lang::ZhHant => 10,
        Lang::Pl => 11,
        #[cfg(feature = "dev")]
        Lang::Pseudo => 0,
    }
}

fn lang_from(c: u32) -> Option<Lang> {
    match c {
        1 => Some(Lang::Ko),
        2 => Some(Lang::En),
        3 => Some(Lang::Ja),
        4 => Some(Lang::ZhHans),
        5 => Some(Lang::PtBr),
        6 => Some(Lang::Fr),
        7 => Some(Lang::Es),
        8 => Some(Lang::De),
        9 => Some(Lang::Ru),
        10 => Some(Lang::ZhHant),
        11 => Some(Lang::Pl),
        _ => None,
    }
}

/// 칸 배열 ↔ 설정. 저장소와 떼어 두어 테스트한다.
fn encode(p: &Prefs) -> [u32; CELLS] {
    let mut v = [0u32; CELLS];
    v[0] = MAGIC;
    v[1] = p.volume as u32 + 1;
    v[2] = p.shake as u32 + 1;
    v[3] = p.lang.map(lang_code).unwrap_or(0);
    v[18] = p.music as u32 + 1;
    v[19] = p.effects as u32 + 1;
    v[20] = p.tells as u32 + 1;
    for i in 0..7 {
        v[4 + i] = p.bindings.keys[i] as u32 + 1;
        v[11 + i] = p.bindings.pads[i] as u32 + 1;
    }
    v
}

fn decode(v: &[u32; CELLS]) -> Option<Prefs> {
    if v[0] != MAGIC || v[1..].iter().any(|x| *x > 255) {
        return None;
    }
    let mut b = Bindings::DEFAULT;
    for i in 0..7 {
        b.keys[i] = v[4 + i].saturating_sub(1) as u8;
        b.pads[i] = v[11 + i].saturating_sub(1) as u8;
    }
    Some(Prefs {
        volume: (v[1].saturating_sub(1) as u8).min(10),
        // 음악 칸이 없던 옛 저장(0)은 기본 7.
        music: if v[18] == 0 {
            7
        } else {
            (v[18] - 1).min(10) as u8
        },
        shake: (v[2].saturating_sub(1) as u8).min(10),
        // 입자 칸이 없던 옛 저장(0)은 기본 10(전부).
        effects: if v[19] == 0 {
            10
        } else {
            (v[19] - 1).min(10) as u8
        },
        // 예고 칸이 없던 옛 저장(0)은 기본값.
        tells: if v[20] == 0 {
            TELLS_DEFAULT
        } else {
            (v[20] - 1).clamp(1, 10) as u8
        },
        lang: lang_from(v[3]),
        bindings: b.sanitized(),
    })
}

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn nail_pref_get(i: u32) -> u32;
        fn nail_pref_set(i: u32, v: u32);
    }
    pub fn get(i: u32) -> u32 {
        // SAFETY: 정수를 주고받는 JS 함수(web/index.html). 없으면 miniquad가 빈 함수를 끼워 0.
        unsafe { nail_pref_get(i) }
    }
    pub fn set(i: u32, v: u32) {
        // SAFETY: 위와 같다.
        unsafe { nail_pref_set(i, v) }
    }

    pub fn load_all() -> [u32; super::CELLS] {
        let mut v = [0u32; super::CELLS];
        for (i, x) in v.iter_mut().enumerate() {
            *x = get(i as u32);
        }
        v
    }

    pub fn save_all(v: &[u32; super::CELLS]) {
        for (i, x) in v.iter().enumerate() {
            set(i as u32, *x);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use super::CELLS;
    use std::path::{Path, PathBuf};

    fn path() -> Option<PathBuf> {
        crate::profile::data_dir().map(|d| d.join("prefs.txt"))
    }

    /// 파일 한 줄: 칸 값을 빈칸으로 띄어 쓴다. 없거나 깨졌으면 0(= 저장 전).
    pub fn read_from(p: &Path) -> [u32; CELLS] {
        let mut v = [0u32; CELLS];
        if let Ok(s) = std::fs::read_to_string(p) {
            for (x, w) in v.iter_mut().zip(s.split_whitespace()) {
                *x = w.parse().unwrap_or(0);
            }
        }
        v
    }

    /// 임시 파일에 쓰고 이름을 바꾼다(기록 파일과 같은 방식).
    pub fn write_to(p: &Path, v: &[u32; CELLS]) {
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let line = v
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
            + "\n";
        let tmp = p.with_extension("tmp");
        if std::fs::write(&tmp, line).is_ok() {
            let _ = std::fs::rename(&tmp, p);
        }
    }

    pub fn load_all() -> [u32; CELLS] {
        path().map(|p| read_from(&p)).unwrap_or([0; CELLS])
    }

    pub fn save_all(v: &[u32; CELLS]) {
        if let Some(p) = path() {
            write_to(&p, v);
        }
    }
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_prefs_crate_version() -> u32 {
    1
}

pub fn load() -> Option<Prefs> {
    decode(&store::load_all())
}

pub fn save(p: &Prefs) {
    store::save_all(&encode(p));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 데스크톱·Android 설정 파일(0175): 쓴 칸을 그대로 읽고, 없는 파일은 0(저장 전), 깨진 글자는 0.
    #[test]
    fn native_prefs_file_round_trips() {
        let dir = std::env::temp_dir().join(format!("nail-prefs-test-{}", std::process::id()));
        let p = dir.join("prefs.txt");
        assert_eq!(store::read_from(&p), [0u32; CELLS], "없으면 0");
        let prefs = Prefs {
            volume: 3,
            music: 2,
            shake: 0,
            effects: 5,
            tells: 3,
            lang: Some(Lang::Pl),
            bindings: Bindings::DEFAULT,
        };
        store::write_to(&p, &encode(&prefs));
        assert_eq!(decode(&store::read_from(&p)), Some(prefs));
        std::fs::write(&p, "garbage 1 2\n").unwrap();
        assert_eq!(
            decode(&store::read_from(&p)),
            None,
            "표식이 깨지면 통째로 무시"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// v0.3.8이 쓴 칸 배열의 고정 샘플(0169 R8): 칸 번호 ↔ 값 약속이 바뀌면 여기서 깨진다(바꾸면 MAGIC을 올린다).
    #[test]
    fn reads_a_fixed_v0_3_cell_sample() {
        let mut v = [0u32; CELLS];
        v[0] = MAGIC;
        v[1] = 8; // 소리 7
        v[2] = 1; // 흔들림 0
        v[3] = 2; // 영어
        v[18] = 4; // 음악 3
        v[19] = 11; // 입자 10
        for i in 0..7 {
            v[4 + i] = Bindings::DEFAULT.keys[i] as u32 + 1;
            v[11 + i] = Bindings::DEFAULT.pads[i] as u32 + 1;
        }
        v[4] = 8; // 공격 키를 표의 7번(Space)으로 바꿔 둔 저장(기본 키와 겹치면 sanitized가 되돌린다)
        let p = decode(&v).unwrap();
        assert_eq!(p.volume, 7);
        assert_eq!(p.shake, 0);
        assert_eq!(p.lang, Some(Lang::En));
        assert_eq!(p.music, 3);
        assert_eq!(p.effects, 10);
        assert_eq!(p.bindings.keys[0], 7);
        assert_eq!(p.bindings.keys[1..], Bindings::DEFAULT.keys[1..]);
        assert_eq!(p.bindings.pads, Bindings::DEFAULT.pads);
        // 예고 칸(20, 0198)은 v0.3에 없었다: 기본값으로 읽고, 다시 쓰면 그 칸만 채워진다.
        assert_eq!(p.tells, TELLS_DEFAULT);
        assert_eq!(encode(&p)[..20], v[..20]);
        // 음악(0081)·입자(0141) 칸이 없던 옛 저장: 0은 기본값 7·10.
        v[18] = 0;
        v[19] = 0;
        let old = decode(&v).unwrap();
        assert_eq!((old.music, old.effects), (7, 10));
    }

    #[test]
    fn round_trip_and_rejects_foreign_data() {
        let mut b = Bindings::DEFAULT;
        b.bind_key(
            crate::input::Act::Attack,
            macroquad::prelude::KeyCode::Space,
        );
        let p = Prefs {
            volume: 3,
            music: 9,
            shake: 0,
            effects: 4,
            tells: 9,
            lang: Some(Lang::Ja),
            bindings: b,
        };
        assert_eq!(decode(&encode(&p)), Some(p));
        assert_eq!(decode(&[0; CELLS]), None, "저장 전(전부 0)");
        // 음악 칸이 생기기 전 저장: 마지막 칸이 0 → 음악은 기본 7, 나머지는 그대로(0081).
        let mut old = encode(&p);
        old[18] = 0;
        assert_eq!(decode(&old).map(|q| q.music), Some(7));
        // 입자 칸이 생기기 전 저장: 그 칸이 0 → 기본 10(0141).
        let mut old = encode(&p);
        old[19] = 0;
        assert_eq!(decode(&old).map(|q| q.effects), Some(10));
        // 예고 칸이 생기기 전 저장(0198): 기본값.
        let mut old = encode(&p);
        old[20] = 0;
        assert_eq!(decode(&old).map(|q| q.tells), Some(TELLS_DEFAULT));
        let mut v = encode(&p);
        v[0] = 1;
        assert_eq!(decode(&v), None, "표식이 다르면 무시");
    }
}
