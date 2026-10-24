# 0004: 웹 링크는 `--import-undefined`로, 외부 JS 플러그인 복사본은 1곳 패치해서 쓴다

- 상태: 채택 (2026-10-24)

## 맥락
1단계에서 사운드(macroquad `audio` feature → quad-snd)와 게임패드(`gamepads` 크레이트)를 붙이자 `cargo build --target wasm32-unknown-unknown`이 링크 단계에서 실패했다.

```
rust-lld: error: undefined symbol: audio_init / audio_add_buffer / ... / getGamepads
```

원인: 두 크레이트는 JS 호스트 함수를 `#[link(wasm_import_module = "env")]` 없이 맨 `extern "C"`로 선언한다. miniquad 자신은 그 속성을 쓰므로 뼈대만 있을 때는 문제가 없었다. 최신 stable rustc(이 세션 1.99.1)는 wasm 타깃에서 `--allow-undefined`를 더 이상 넘기지 않아 이런 심볼이 에러가 된다(Rust 블로그 2026-04-04 "Changes to WebAssembly targets and handling undefined symbols", KB `reference/macroquad/wasm-build.md`).

또, `gamepads`의 macroquad JS 플러그인은 `version:"0.1.7"`(문자열)을 선언하는데 크레이트는 `(major<<24)+(minor<<16)+patch` 정수(65543)를 export해서 `mq_js_bundle.js`가 매 로드마다 `version mismatch` 에러를 찍는다(헤드리스 Chromium으로 확인).

## 결정
1. `.cargo/config.toml`에 `[target.wasm32-unknown-unknown] rustflags = ["-C", "link-arg=--import-undefined"]`를 둔다. 레포에 커밋되므로 로컬과 CI가 같은 플래그를 쓴다.
2. `web/macroquad-gamepads-0.1.js`는 상류 복사본에 **버전 숫자 1곳만** 고치고, 파일 머리에 수정 내용과 원본 URL을 적는다. `mq_js_bundle.js`는 수정 없이 그대로 둔다.

## 왜
- 의존 크레이트 소스를 고칠 수 없고(패치 포크는 유지비), 플래그 한 줄이 예전 링커 동작을 그대로 되돌린다. 상류가 `#[link(wasm_import_module)]`를 붙이면 플래그는 무해한 잉여가 된다.
- 콘솔에 상시 찍히는 에러는 진짜 문제를 가린다. 플러그인 JS는 1KB짜리 복사본이고 어차피 핫링크하지 않기로 했으므로(CLAUDE.md), 고친 지점을 문서화하면 유지비가 거의 없다.

## 비용
- 크레이트/번들을 올릴 때마다 체크리스트: `web/README.md`의 버전 숫자 갱신, 콘솔 mismatch 확인.
- `--import-undefined`는 **오타로 생긴 미정의 심볼도 import로 넘겨** 런타임에야 "Missed function" 경고로 드러난다. 배포 후 콘솔 확인을 생략하지 않는다.

## 재검토 조건
- `quad-snd`·`gamepads`가 `wasm_import_module`을 선언하면 플래그 제거.
- `gamepads` 상류가 JS 버전 표기를 고치면 패치 제거.
