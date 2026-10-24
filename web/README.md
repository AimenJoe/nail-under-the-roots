# web/

브라우저 배포용 정적 파일. CI가 `dist/`를 조립할 때 이 폴더 내용 + `nail.wasm` + `assets/`를 복사한다.

- `mq_js_bundle.js`: macroquad 레포 `js/mq_js_bundle.js` 복사본. 가져온 시점: 2026-10-24, macroquad master(Cargo.toml version 0.4.16).
  **macroquad 버전을 올릴 때 같은 태그의 파일로 교체할 것.** 핫링크 금지. 이유: KB `reference/macroquad/wasm-build.md` "JS 번들의 버전 계약".
- 게임패드를 넣을 때 `gamepads` 크레이트의 JS 플러그인도 여기에 복사해 `index.html`에서 로드한다.
