# Naïl: Under the Roots

귀엽지만 쓸쓸한 땅속 세계에서, 아기 두더지 Naïl이 발톱과 굴파기로 방을 돌파하며 시들어 가는 나무의 뿌리를 따라 내려가는 탑다운 액션 로그라이트. Rust + macroquad.

- 컨셉: [`docs/concept.md`](docs/concept.md)
- 현재 단계: [`docs/phase-1-combat-prototype.md`](docs/phase-1-combat-prototype.md) (1단계 전투 프로토타입, 회색 박스)
- 결정 기록: [`docs/decisions/`](docs/decisions/)
- 지식 베이스: [AimenJoe/KB](https://github.com/AimenJoe/KB) (`maps/gamedev.md`)

## 실행

```sh
# 데스크톱. Linux 의존성: libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config
#   (libudev는 gamepads→gilrs, libasound는 macroquad audio)
cargo run

# 웹
cargo build --release --target wasm32-unknown-unknown
mkdir -p dist && cp web/index.html web/*.js dist/ && cp target/wasm32-unknown-unknown/release/nail.wasm dist/
python3 -m http.server 8000 --directory dist   # http://127.0.0.1:8000
```

## 조작

| 동작 | 키보드 | 게임패드 |
|---|---|---|
| 이동 | WASD / 화살표 | 왼쪽 스틱, D-pad |
| 발톱(3연타) | J / Z | X(□) / R1 |
| 굴파기 → 솟구침 | K / X / Space (다시 누르면 조기 솟구침) | A(×) / R2 |
| 조준 | (마지막 이동 방향) | 오른쪽 스틱 |
| 재시작 | R | Start |
| 일시정지 | Esc | B(○) / Select |
| 디버그 | F1 오버레이, F2 히트박스 | — |

## 커밋 전

```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo build --release --target wasm32-unknown-unknown
```

## 배포

### 웹 (Netlify)
- 사이트: https://nail-under-the-roots.netlify.app (Netlify Site ID `a0bc4242-4bdd-4fc8-8266-adeaf143e59c`, Git 연동 없음)
- PR을 열면 `pr-<번호>--nail-under-the-roots.netlify.app` 에 미리보기가 올라간다.
- `main`에 push하면 `deploy-web.yml`이 wasm을 빌드하고, 레포 Secrets에 `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`가 있으면 배포한다. 시크릿이 없으면 빌드 검증만. Netlify 사이트는 **Git 연동 없이**(Deploy manually) 만든다. 이유와 절차: KB `guides/macroquad-wasm-netlify-github-actions.md`.

### Android (APK)
- `build-android.yml`이 `src/`·`Cargo.*`가 바뀐 push마다 `cargo quad-apk build --release`로 **디버그 키 서명 APK**를 만들어 Actions 아티팩트(`nail-apk-<sha>`)로 올린다. Actions 탭 → 해당 실행 → Artifacts에서 받는다.
- `v*` 태그를 push하면 같은 APK가 GitHub Release에 첨부된다: `git tag v0.1.0 && git push origin v0.1.0`.
- 실기 설치: `adb install -r nail.apk` 또는 파일을 옮겨 사이드로드(알 수 없는 출처 허용). arm64 전용.
- 로컬에서 만들려면 Android SDK(build-tools, platforms;android-36)·NDK·JDK 17과 `cargo install --git https://github.com/not-fl3/cargo-quad-apk --rev <build-android.yml의 CARGO_QUAD_APK_REV>`, 그리고 `ANDROID_HOME`·`NDK_HOME` 환경 변수가 필요하다. 보통은 CI에 맡긴다(docs/decisions/0005).
- 아직 Android에서 게임패드 **버튼**은 안 들어온다(D-pad만). docs/decisions/0002의 2단계 스파이크 범위.
