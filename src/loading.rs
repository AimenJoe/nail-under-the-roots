//! 웹 로딩 화면과 주고받기(web/index.html의 nail_loading 플러그인, docs/decisions/0028).
//!
//! 첫 프레임 전까지는 macroquad가 화면을 그리지 않는다. 페이지가 글 없는 로딩 화면(걷는 Naïl + 막대)을 덮고,
//! wasm을 받는 동안은 받은 바이트로, 그 뒤엔 여기서 알려 주는 준비 단계(폰트·그림·소리)로 막대를 채운다.
//! 첫 프레임을 그린 뒤 `ready()`로 걷어 낸다. 웹 밖에서는 아무것도 하지 않는다.

/// 준비 단계 수(main.rs가 부르는 step 횟수와 맞춘다: 기본 폰트 + 다른 언어 폰트 파일 수 + 그림 + 소리).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const STEPS: u32 = 5;

#[cfg(target_arch = "wasm32")]
mod js {
    extern "C" {
        pub fn nail_load_step(done: u32, total: u32);
        pub fn nail_load_ready();
    }
}

/// `done`번째 준비 단계가 끝났다.
pub fn step(done: u32) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 정수를 넘기는 JS 함수(web/index.html). 없으면 miniquad가 빈 함수를 끼운다.
    unsafe {
        js::nail_load_step(done, STEPS);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = done;
}

/// 첫 프레임을 그렸다: 로딩 화면을 걷는다.
pub fn ready() {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    unsafe {
        js::nail_load_ready();
    }
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_loading_crate_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use crate::i18n::{Lang, BASE_FONT};

    /// 언어 폰트를 늘리면 STEPS도 늘려야 막대가 끝까지 찬다.
    #[test]
    fn steps_match_what_main_loads() {
        let mut files: Vec<&str> = Lang::ALL
            .iter()
            .map(|l| l.font_file())
            .filter(|f| *f != BASE_FONT)
            .collect();
        files.sort_unstable();
        files.dedup();
        assert_eq!(super::STEPS as usize, 1 + files.len() + 2);
    }
}
