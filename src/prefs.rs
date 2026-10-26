//! 설정 저장(소리·흔들림·언어·조작). 웹은 브라우저 localStorage(web/index.html의 nail_prefs 플러그인)에
//! 정수 칸 몇 개로 둔다. 데스크톱·Android는 아직 저장하지 않는다(켤 때마다 기본값).
//!
//! 칸: 0 표식(MAGIC), 1 소리+1, 2 흔들림+1, 3 언어(0 = 기기 언어), 4..11 키 번호+1, 11..18 패드 번호+1,
//! 18 음악+1(0081, 뒤에 붙인 칸이라 옛 저장은 0 = 기본값으로 읽는다 → MAGIC은 그대로).
//! 0은 "없음"(플러그인이 없거나 저장 전). 표식이 다르면 통째로 무시한다(형식이 바뀌면 MAGIC을 올린다).

use crate::i18n::Lang;
use crate::input::Bindings;

const MAGIC: u32 = 0x4E41_0001;
/// 저장 칸 수.
const CELLS: usize = 19;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prefs {
    pub volume: u8,
    pub music: u8,
    pub shake: u8,
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
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    pub fn get(_i: u32) -> u32 {
        0
    }
    pub fn set(_i: u32, _v: u32) {}
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_prefs_crate_version() -> u32 {
    1
}

pub fn load() -> Option<Prefs> {
    let mut v = [0u32; CELLS];
    for (i, x) in v.iter_mut().enumerate() {
        *x = store::get(i as u32);
    }
    decode(&v)
}

pub fn save(p: &Prefs) {
    for (i, x) in encode(p).iter().enumerate() {
        store::set(i as u32, *x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            lang: Some(Lang::Ja),
            bindings: b,
        };
        assert_eq!(decode(&encode(&p)), Some(p));
        assert_eq!(decode(&[0; CELLS]), None, "저장 전(전부 0)");
        // 음악 칸이 생기기 전 저장: 마지막 칸이 0 → 음악은 기본 7, 나머지는 그대로(0081).
        let mut old = encode(&p);
        old[18] = 0;
        assert_eq!(decode(&old).map(|q| q.music), Some(7));
        let mut v = encode(&p);
        v[0] = 1;
        assert_eq!(decode(&v), None, "표식이 다르면 무시");
    }
}
