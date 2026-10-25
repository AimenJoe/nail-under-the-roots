# web/

브라우저 배포용 정적 파일. CI가 `dist/`를 조립할 때 이 폴더의 `index.html` + `*.js` + `nail.wasm` + `assets/`를 복사한다.

- `mq_js_bundle.js`: macroquad 레포 `js/mq_js_bundle.js` 복사본(수정 없음). 가져온 시점: 2026-10-24, macroquad master(Cargo.toml version 0.4.16).
  **macroquad 버전을 올릴 때 같은 태그의 파일로 교체할 것.** 핫링크 금지. 이유: KB `reference/macroquad/wasm-build.md` "JS 번들의 버전 계약".
  - 알려진 상류 결함: 번들 안 `quad_net` 플러그인이 strict 모드에서 선언 없는 전역 `register_plugin=`을 써서 페이지 로드 시 `ReferenceError: register_plugin is not defined`가 콘솔에 한 번 찍힌다. quad_net을 쓰지 않으므로 무해하다.
- `macroquad-gamepads-0.1.js`: `gamepads` 크레이트 0.1.7의 macroquad 플러그인 복사본 (원본 `js/macroquad-gamepads-0.1.js`).
  **2곳 수정**: (1) `version:"0.1.7"` → `version:65543`. (2) `mapping!=="standard"` 패드를 무시하는 조건 제거 — Android Chrome의 블루투스 패드·핸드헬드 내장 패드는 비표준 매핑으로 잡혀 컨트롤러가 전혀 안 먹는다. 표준 인덱스 가정으로 받는다. 크레이트가 export하는 `gamepads_crate_version()`은 `(major<<24)+(minor<<16)+patch` 정수인데 원본 JS는 문자열이라 매 로드마다 "version mismatch" 에러가 찍힌다. 크레이트를 올리면 파일을 다시 받고 이 숫자도 갱신한다. (docs/decisions/0004)

## 확인
배포 후 브라우저 콘솔에 `Version mismatch`·`version mismatch`가 없어야 한다. 헤드리스 확인 방법: KB `reference/macroquad/wasm-build.md`.
