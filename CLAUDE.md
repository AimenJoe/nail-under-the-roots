# CLAUDE.md — nail-under-the-roots

Naïl: Under the Roots. 귀여운 탑다운 액션 로그라이트. Rust + macroquad. 웹(Netlify) 먼저, 나중에 Android 핸드헬드(컨트롤러 필수).

## 먼저 읽을 것
1. `docs/concept.md` — 무엇을 만드는가
2. `docs/phase-1-combat-prototype.md` — 지금 단계의 범위와 판정 기준. 끝의 **"구현 상태"와 "사람이 할 일"**이 현재 위치다
3. `docs/decisions/` — 왜 그렇게 정했나. 바꾸려면 새 번호로 기록

세션 시작 프롬프트는 따로 없다. 이 파일과 위 세 문서가 그 역할을 한다. 세션이 끝나면 "구현 상태"·"사람이 할 일"을 갱신하고 커밋한다.

## 지식 베이스 (KB)
일반 지식은 이 레포가 아니라 **github.com/AimenJoe/KB** 에 있다. 세션에 KB 레포를 함께 붙이고, 질문 전에 `maps/gamedev.md` → 해당 노트를 먼저 읽는다.
- 렌더링: `reference/macroquad/pixel-perfect-rendering.md`
- 입력·게임패드·Android: `reference/macroquad/gamepad-and-android-input.md`
- WASM 빌드·JS 번들: `reference/macroquad/wasm-build.md`
- 배포: `guides/macroquad-wasm-netlify-github-actions.md`
- 설계 원칙: `concepts/action-roguelite-for-solo-dev.md`
- 에셋: `guides/pixel-art-without-an-artist.md`

**프로젝트 기록(TODO, 튜닝 수치, 결정)은 이 레포에, 프로젝트가 사라져도 유용한 지식은 KB에.** KB에 새 지식을 쓸 때는 KB의 `CLAUDE.md` 규칙(frontmatter, verified, 지도 링크, `tools/kb.py check`)을 따른다.

## 코드 규칙
- 입력은 `Actions` 구조체로만. 게임 로직 어디에서도 `KeyCode`를 직접 보지 않는다.
- 내부 해상도 320×180, 정수 배율. 월드 좌표는 그릴 때 반올림.
- 로직은 고정 타임스텝(60Hz). 히트스톱은 로직 시계만 멈춘다.
- 손맛 수치는 `src/tuning.rs` 한 곳에. 콤보(무기·캐릭터별)는 `src/moveset.rs`의 `Moveset` 표. 플레이어 코드에 타별 숫자를 직접 쓰지 않는다(docs/decisions/0009).
- UI는 커서 없이 동작해야 한다(좌우 이동 + 확인). 마우스는 보조.
- 모듈 경계: `input`, `player`, `enemy`, `combat`, `room`, `render`, `world`(틱 순서). 각 모듈은 `Actions`와 월드 상태만 본다. 장면·메뉴는 `app`(+`ui`)에만 있고 `world`는 메뉴를 모른다.
- 그림 문법은 3/4 탑다운(docs/decisions/0006). 아이소메트릭 아님.
- 테스트용 방 구성은 `src/scenario.rs`에 한 줄 추가(타이틀 "개발"). 컷씬은 `src/cutscene.rs`의 `Step` 스크립트, JRPG식(docs/decisions/0008).
- UI 텍스트는 `assets/fonts/Galmuri11-Condensed.ttf`(KS X 1001 2,350자 서브셋)로 실제 해상도에 찍는다. 그 밖의 한자·확장 한글은 안 나온다.

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo build --release --target wasm32-unknown-unknown
cargo check --target aarch64-linux-android   # NDK 없이 됨. Android에서만 깨지는 의존성(gamepads 등)을 잡는다
```
CI: `deploy-web.yml`이 위 세 줄을 돌리고 Netlify 시크릿이 있으면 배포한다. `build-android.yml`이 APK를 만들어 아티팩트(태그면 Release)로 올린다. 로컬에 Android SDK/NDK는 두지 않는다(docs/decisions/0005).

## 하지 않는 것
- `mq_js_bundle.js`나 게임패드 JS 플러그인을 외부 URL로 핫링크하지 않는다. `web/`에 복사본.
- 에셋 팩을 라이선스 확인 없이 커밋하지 않는다. `assets/LICENSES.md`에 기록.
- 1단계 범위 밖(에셋, 허브, 저장, 두 번째 캐릭터)을 판정 전에 만들지 않는다.
- `cargo-quad-apk` 커밋(`build-android.yml`의 `CARGO_QUAD_APK_REV`)과 `web/*.js` 복사본은 의도적으로만 올린다. 올리면 CI 콘솔/로그를 확인한다.
- 언어: 문서·커밋 메시지 한국어, 코드 식별자 영어.
