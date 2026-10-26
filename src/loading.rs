//! 웹 로딩 화면과 주고받기(web/index.html의 nail_loading 플러그인, docs/decisions/0028).
//!
//! 첫 프레임 전까지는 macroquad가 화면을 그리지 않는다. 페이지가 글 없는 로딩 화면(걷는 Naïl + 막대)을 덮고,
//! wasm을 받는 동안은 받은 바이트로, 그 뒤엔 여기서 알려 주는 준비 단계(폰트·그림·소리)로 막대를 채운다.
//! 첫 프레임을 그린 뒤 `ready()`로 걷어 낸다. 웹 밖에서는 아무것도 하지 않는다.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// 받는 중인 파일 하나.
type Fetch<'a> = Pin<Box<dyn Future<Output = Result<Vec<u8>, macroquad::Error>> + 'a>>;

thread_local! {
    /// `prefetch`가 받아 둔 파일(assets 기준 경로 → 바이트). 꺼내 쓰면 지운다.
    static CACHE: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
}

/// 시작할 때 읽을 파일을 **한꺼번에** 받는다(0092). 하나씩 `await`하면 웹에서 파일마다 왕복 지연이 쌓여
/// (그림·소리 43개 × 약 0.1~0.15초) 막대 뒤쪽이 몇 초씩 멈춰 있었다. 받는 대로 막대를 한 칸씩 채운다:
/// 받은 파일 수 / (파일 수 + `after`). `after`는 받은 뒤 남은 단계 수(풀기 등).
pub async fn prefetch(paths: &[String], after: u32) {
    let total = paths.len() as u32 + after;
    let mut futs: Vec<Fetch> = paths
        .iter()
        .map(|p| Box::pin(macroquad::file::load_file(p)) as Fetch)
        .collect();
    let mut left: Vec<usize> = (0..paths.len()).collect();
    let mut done = 0;
    std::future::poll_fn(|cx| {
        left.retain(|&i| match futs[i].as_mut().poll(cx) {
            Poll::Ready(r) => {
                if let Ok(bytes) = r {
                    CACHE.with(|c| c.borrow_mut().insert(paths[i].clone(), bytes));
                }
                done += 1;
                step(done, total);
                false
            }
            Poll::Pending => true,
        });
        if left.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
}

/// 받아 둔 바이트를 꺼낸다. 없으면(목록에 없던 파일·받기 실패) 지금 받는다.
pub async fn bytes(path: &str) -> Result<Vec<u8>, macroquad::Error> {
    if let Some(b) = CACHE.with(|c| c.borrow_mut().remove(path)) {
        return Ok(b);
    }
    macroquad::file::load_file(path).await
}

pub async fn texture(path: &str) -> Result<macroquad::texture::Texture2D, macroquad::Error> {
    let b = bytes(path).await?;
    Ok(macroquad::texture::Texture2D::from_file_with_format(
        &b, None,
    ))
}

pub async fn image(path: &str) -> Result<macroquad::texture::Image, macroquad::Error> {
    let b = bytes(path).await?;
    macroquad::texture::Image::from_file_with_format(&b, None)
}

#[cfg(target_arch = "wasm32")]
mod js {
    extern "C" {
        pub fn nail_load_step(done: u32, total: u32);
        pub fn nail_load_ready();
    }
}

/// 준비 단계 `total` 가운데 `done`번째가 끝났다.
pub fn step(done: u32, total: u32) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: 정수를 넘기는 JS 함수(web/index.html). 없으면 miniquad가 빈 함수를 끼운다.
    unsafe {
        js::nail_load_step(done, total);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (done, total);
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
