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
- 입력은 `Actions` 구조체로만. 게임 로직 어디에서도 `KeyCode`를 직접 보지 않는다. 게임 동작의 키·버튼은 `input::Bindings`(설정에서 바꿈, docs/decisions/0022), 화면에 버튼 이름을 쓸 때는 `input::prompt(Act)`, 메뉴 동사(확인·뒤로 …)는 `input::verb(Verb)` — 글에 키 이름을 박지 않는다. 둘 다 지금 쓰는 장치(키보드·패드·터치)를 따른다(docs/decisions/0036).
- 내부 해상도 320×180, 정수 배율. 월드 좌표는 그릴 때 반올림. 카메라만은 반올림하지 않는다: 정수 부분으로 그리고 소수점은 화면에서 민다(렌더 타깃 1px 여분, docs/decisions/0034). HUD는 화면 패스에.
- 로직은 고정 타임스텝(60Hz). 히트스톱은 로직 시계만 멈춘다.
- 손맛 수치는 `src/tuning.rs` 한 곳에. 적 종류별 수치·도감 글은 `src/species.rs` 표(docs/decisions/0021). 콤보(무기별)는 `src/moveset.rs`의 `Moveset` 표, 어떤 캐릭터가 어떤 무기 종류를 쓰는지는 `src/equipment.rs`(docs/decisions/0011). 플레이어 코드에 타별 숫자를 직접 쓰지 않는다(docs/decisions/0009).
- UI는 커서 없이 동작해야 한다(좌우 이동 + 확인). 마우스는 보조.
- 모듈 경계: `input`, `player`, `enemy`, `combat`, `room`, `render`, `world`(틱 순서). 각 모듈은 `Actions`와 월드 상태만 본다. 장면·메뉴는 `app`(+`ui`)에만 있고 `world`는 메뉴를 모른다.
- 모험 구조(docs/decisions/0019): 장소·카메라·던전 오가기는 `adventure`, 격자·ASCII 기호는 `map`, 필드 지도는 `field`(ASCII를 고치면 `cargo test`의 field 테스트로 컷씬 줄·입구·저장 지점 확인), 던전 생성은 `dungeon`(같은 시드 = 같은 던전). 월드 좌표는 지도 전체, 화면은 `world.adv.cam`. 스테이지 고리(0030~0032): 쉼터 지도는 `camp`, 길 지도(갈래)는 `route`, 굴 모양은 `dungeon::Params`, 길 지도·가게·말 걸기 화면은 `app/stage.rs`. 런 사이 허브(마을)는 `village.rs`(지도·꾸미기 표), 장소는 `Place::Village`(0045). 길 끝방의 이름 있는 보스는 `dungeon::Params::boss`(첫 길 = 모모, 세 번, 0046; 둘째 길 = 도토, 한 번, 묻은 도토리는 적이 아니라 `World::mines`, 0052; 셋째 길 = 콩이, 한 번, 말린 동안 발톱은 `Enemy::guarding`이면 튕긴다, 0054). 반딧불로 깨우는 영구 재주는 `talents.rs`(자리 넷 × 갈래 둘, 플레이어는 `Player::talents`만 본다, 0047). 이긴 보스와 다시 싸우는 실의 꿈은 `dream.rs`(목록·기록 키) + `app/dream.rs`(창·흐름), 꿈 월드는 `World::dream`이라 아무것도 떨어뜨리지 않는다(0055). 저널 목표·보상은 `journal.rs` 표(진행은 기록에서 계산, 런 통계는 `stat.*` 키, 아이디는 바꾸지 않는다, 0048). 월드 패스 뒤 그리는 컷씬 배우는 `Renderer::to_view`로 화면 좌표.
- 그림 문법은 3/4 탑다운(docs/decisions/0006). 아이소메트릭 아님.
- 테스트용 방 구성은 `src/scenario.rs`에 한 줄 추가(타이틀 "개발"). 컷씬은 `src/cutscene.rs`의 `Step` 스크립트, JRPG식(docs/decisions/0008). 화면 전체 그림은 `Step::Illust` + `Step::Narrate`, 그림 목록·패닝·입자는 `src/illust.rs`(0039). 튜토리얼은 `src/tutorial.rs`의 단계 + 포포의 말(docs/decisions/0023).
- 설정 저장은 `src/prefs.rs`(웹 localStorage 정수 칸). 칸 형식을 바꾸면 `MAGIC`을 올린다. 런을 넘어 남는 기록은 `src/profile.rs`(깃발 + 수, 문자열 키 — 키 이름은 바꾸지 않는다, 0041). 기록은 첫 시작(Normal)만 남긴다. 런 사이 한마디는 `src/story.rs` 표(조건이 가장 구체적인 줄, `once`는 한 번).
- UI 텍스트는 `assets/fonts/Galmuri11-Condensed.ttf`(KS X 1001 2,350자 서브셋)로 실제 해상도에 찍는다. 그 밖의 한자·확장 한글은 안 나온다.
- 플레이어가 보는 글은 `src/i18n.rs`의 언어별 표에만 쓴다(한국어·영어·일본어·중국어 간체·브라질 포르투갈어). 코드에 글 리터럴을 두지 않는다. 숫자는 `{0}` + `fill()`, 데이터 표는 `Tr`. 개발 전용 화면만 예외(docs/decisions/0013).
- 번역은 직접 쓴다. DeepL 같은 기계 번역은 사장이 따로 말할 때만 쓴다(초안 품질이 낮다: 굴 → oyster, 보리 → barley, 털빛 → feather 같은 게임 용어 오독이 많았다). 기존 표의 용어를 먼저 찾아 맞춘다.
- 글을 넣거나 바꾸면 다섯 언어를 다 쓰고, 일본어·간체 표를 고쳤으면 `python3 tools/fonts/build_cjk.py <ark-pixel-font 체크아웃>`으로 폰트를 다시 만든다. 새 화면은 가짜 언어 Pseudo(설정 → 언어 끝)와 일본어로 넘침을 본다. 고정 칸은 `ui.fit`, 긴 글은 `ui::wrap`(docs/decisions/0014).

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo build --release --target wasm32-unknown-unknown
cargo check --target aarch64-linux-android   # NDK 없이 됨. Android에서만 깨지는 의존성(gamepads 등)을 잡는다
cargo test                                   # 규칙 테스트(무브셋·독). 리눅스면 libx11-dev 등 필요(README)
python3 tools/licenses.py                    # 의존성을 바꿨을 때만. assets/THIRD_PARTY_LICENSES.txt 갱신
```
CI: `deploy-web.yml`이 위 세 줄을 돌리고 Netlify 시크릿이 있으면 배포한다. `build-android.yml`이 APK를 만들어 아티팩트(태그면 Release)로 올린다. 로컬에 Android SDK/NDK는 두지 않는다(docs/decisions/0005).

## 하지 않는 것
- `mq_js_bundle.js`나 게임패드 JS 플러그인을 외부 URL로 핫링크하지 않는다. `web/`에 복사본.
- 에셋 팩을 라이선스 확인 없이 커밋하지 않는다. `assets/LICENSES.md`에 기록. **이 레포는 공개다. 유료·제3자 팩 원본은 `assets/vendor/`(gitignore)에만 두고 커밋하지 않는다**(docs/decisions/0010). 팩 이미지를 AI 도구에 넣지 않는다.
- 직접 그린 그림은 PNG를 손으로 고치지 않는다. `tools/art/*.py`를 고치고 다시 생성한다(Naïl `nail.py`, 적·이웃 `critters.py`, 보스 `chief.py`, 초상화 `portraits.py`, 타일 `tiles.py`, 컷씬 일러스트 `illust.py` — 임시 그림. 사람이 그린 그림으로 바꿀 때는 `docs/art/cutscene-illustrations.md`).
- 1단계 범위 밖(에셋, 허브, 저장, 두 번째 캐릭터)을 판정 전에 만들지 않는다.
- `cargo-quad-apk` 커밋(`build-android.yml`의 `CARGO_QUAD_APK_REV`)과 `web/*.js` 복사본은 의도적으로만 올린다. 올리면 CI 콘솔/로그를 확인한다.
- 언어: 문서·커밋 메시지 한국어, 코드 식별자 영어.
