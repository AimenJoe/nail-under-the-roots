//! 웹 셸 다리(0269, 요구 2.4): 전체 화면·설치 상태·세로 화면. JS 쪽은 `web/nail-host.js`의 `nail_web` 플러그인.
//! 웹이 아닌 빌드는 "없음"으로 답한다(데스크톱 전체 화면은 macroquad가, Android·iOS 앱은 늘 전체 화면).

#[cfg(target_arch = "wasm32")]
mod js {
    extern "C" {
        pub fn nail_web_flags() -> u32;
        pub fn nail_fs_active() -> u32;
        pub fn nail_fs_toggle();
        pub fn nail_portrait() -> u32;
    }
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_web_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
fn flags() -> u32 {
    // SAFETY: 정수만 주고받는 JS 함수(web/nail-host.js). 플러그인이 없으면(옛 캐시 페이지) miniquad가 빈 함수를 끼워 0.
    unsafe { js::nail_web_flags() }
}

/// 이 브라우저에서 전체 화면을 쓸 수 있나(iPhone Safari는 못 쓴다 — 요소 전체 화면 API가 없다).
pub fn fullscreen_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    return flags() & 1 != 0;
    #[cfg(not(target_arch = "wasm32"))]
    crate::DESKTOP
}

/// 홈 화면에 추가하라고 알릴 곳인가: iPhone·iPad에서 설치하지 않은 브라우저 탭으로 열었을 때.
pub fn suggest_home_screen() -> bool {
    #[cfg(target_arch = "wasm32")]
    return flags() & 4 != 0 && flags() & 2 == 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// 웹 전체 화면이 켜져 있나(Esc·뒤로로 나가면 브라우저가 끈다 — 설정 표시는 이것을 따른다).
pub fn fullscreen_active() -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    return unsafe { js::nail_fs_active() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// 웹 전체 화면을 켜거나 끈다. 사용자가 누른 입력의 프레임에서만 부른다(브라우저가 그때만 허락한다).
pub fn toggle_fullscreen() {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    unsafe {
        js::nail_fs_toggle()
    }
}

/// 세로로 든 터치 기기라 회전 안내가 떠 있나(그동안 게임은 일시정지).
pub fn portrait_blocked() -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 위와 같다.
    return unsafe { js::nail_portrait() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}
