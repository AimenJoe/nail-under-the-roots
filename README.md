# Naïl: Under the Roots

귀엽지만 쓸쓸한 땅속 세계에서, 아기 두더지 Naïl이 발톱과 굴파기로 방을 돌파하며 시들어 가는 나무의 뿌리를 따라 내려가는 탑다운 액션 로그라이트. Rust + macroquad.

- 컨셉: [`docs/concept.md`](docs/concept.md)
- 현재 단계: [`docs/phase-1-combat-prototype.md`](docs/phase-1-combat-prototype.md)
- 결정 기록: [`docs/decisions/`](docs/decisions/)
- 지식 베이스: [AimenJoe/KB](https://github.com/AimenJoe/KB) (`maps/gamedev.md`)

## 실행

```sh
cargo run                                              # 데스크톱 (Linux는 libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev 필요)
cargo build --release --target wasm32-unknown-unknown  # 웹
mkdir -p dist && cp web/* dist/ && cp target/wasm32-unknown-unknown/release/nail.wasm dist/
cargo install basic-http-server && basic-http-server dist   # http://127.0.0.1:4000
```

## 배포
- 사이트: https://nail-under-the-roots.netlify.app (Netlify Site ID `a0bc4242-4bdd-4fc8-8266-adeaf143e59c`, Git 연동 없음)
- PR을 열면 `pr-<번호>--nail-under-the-roots.netlify.app` 에 미리보기가 올라간다.

`main`에 push하면 GitHub Actions가 wasm을 빌드하고, 레포 Secrets에 `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`가 있으면 Netlify에 배포한다. Netlify 사이트는 **Git 연동 없이**(Deploy manually) 만든다. 이유와 절차: KB `guides/macroquad-wasm-netlify-github-actions.md`.
