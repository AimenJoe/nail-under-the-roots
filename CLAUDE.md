# CLAUDE.md — nail-under-the-roots

Naïl: Under the Roots. 귀여운 탑다운 액션 로그라이트. Rust + macroquad. 웹(Netlify)·Android 핸드헬드(APK, 컨트롤러 필수), iOS는 빌드만.

## 먼저 읽을 것
1. `docs/concept.md` — 무엇을 만드는가
2. `docs/phase-1-combat-prototype.md` — 1단계 범위와 판정 기준. 끝의 **"구현 상태"와 "사람이 할 일"**이 현재 위치다. 버전 규칙은 `docs/decisions/0079`(v0.2.0은 닫음), 지금 마일스톤(v0.3.0 조건표)은 `docs/decisions/0081`
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
- 입력은 `Actions` 구조체로만. 게임 로직 어디에서도 `KeyCode`를 직접 보지 않는다. 게임 동작의 키·버튼은 `input::Bindings`(설정에서 바꿈, docs/decisions/0022), 화면에 버튼 이름을 쓸 때는 `input::prompt(Act)`, 메뉴 동사(확인·뒤로 …)는 `input::verb(Verb)` — 글에 키 이름을 박지 않는다. 둘 다 지금 쓰는 장치(키보드·패드·터치)를 따른다(docs/decisions/0036). Android 패드 버튼은 패치한 miniquad(`third_party/miniquad`, `[patch.crates-io]`)가 F13~F25 키로 넘긴다. 패치를 고치거나 miniquad를 올리면 `PATCHED.md`대로 다시 넣고 `cargo test`의 `android_pad_keys_match_patched_miniquad`를 본다(0075).
- 내부 해상도 320×180, 정수 배율. 월드 좌표는 그릴 때 반올림. 카메라만은 반올림하지 않는다: 정수 부분으로 그리고 소수점은 화면에서 민다(렌더 타깃 1px 여분, docs/decisions/0034). HUD는 화면 패스에.
- 로직은 고정 타임스텝(60Hz). 히트스톱은 로직 시계만 멈춘다.
- 음악은 `src/music.rs`(곡 표 `TRACKS`, 웹 mp3·네이티브 ogg, 들을 때 읽고 다 내려가면 놓는다), 효과음은 `src/sfx.rs`(파일 소리 표 `FILES` + 합성음, 메뉴 소리는 `App::menu_sound`가 한꺼번에). 소리 파일은 손으로 넣지 않고 `tools/audio/make_audio.py`의 표를 고쳐 다시 만든다(CC0만, 출처는 `assets/LICENSES.md`). 특수효과는 `world.rs` FxKind + `render.rs`(draw_fx·draw_ambient·draw_world_lights, 더하기 빛은 `light_material`, 0076·0081).
- 손맛 수치는 `src/tuning.rs` 한 곳에. 적 종류별 수치·도감 글은 `src/species.rs` 표(docs/decisions/0021). 콤보(무기별)는 `src/moveset.rs`의 `Moveset` 표, 어떤 캐릭터가 어떤 무기 종류를 쓰는지는 `src/equipment.rs`(docs/decisions/0011). 플레이어 코드에 타별 숫자를 직접 쓰지 않는다(docs/decisions/0009).
- UI는 커서 없이 동작해야 한다(좌우 이동 + 확인). 마우스는 보조.
- 모듈 경계: `input`, `player`, `enemy`, `combat`, `room`, `render`, `world`(틱 순서). 각 모듈은 `Actions`와 월드 상태만 본다. 장면·메뉴는 `app`(+`ui`)에만 있고 `world`는 메뉴를 모른다.
- 모험 구조(docs/decisions/0019): 장소·카메라·던전 오가기는 `adventure`, 격자·ASCII 기호는 `map`, 필드 지도는 `field`(ASCII를 고치면 `cargo test`의 field 테스트로 컷씬 줄·입구·저장 지점 확인), 던전 생성은 `dungeon`(같은 시드 = 같은 던전). 월드 좌표는 지도 전체, 화면은 `world.adv.cam`. 스테이지 고리(0030~0032): 쉼터 지도는 `camp`, 길 지도(갈래)는 `route`, 굴 모양은 `dungeon::Params`, 길 지도·가게·말 걸기 화면은 `app/stage.rs`. 런 사이 허브(마을)는 `village.rs`(지도·꾸미기 표), 장소는 `Place::Village`(0045). 길 끝방의 이름 있는 보스는 `dungeon::Params::boss`(첫 길 = 모모, 세 번, 0046; 둘째 길 = 도토, 한 번, 묻은 도토리는 적이 아니라 `World::mines`, 0052; 셋째 길 = 콩이, 한 번, 말린 동안 발톱은 `Enemy::guarding`이면 튕긴다, 0054). 반딧불로 깨우는 영구 재주는 `talents.rs`(자리 넷 × 갈래 둘, 플레이어는 `Player::talents`만 본다, 0047. 모모는 자리 1·3이 `MOMO_TABLE`·키 `talent.hop.*`·`talent.chain.*`, 큰 나무 화면은 Naïl/모모 탭, 0142). 장신구의 `Charm::only`는 그 캐릭터 런에서만 상자·가게·저널로 나온다(모모 장신구 셋, 0142). 두 번째 캐릭터 모모(0056)는 `Player::kit`·`World::character` 하나로 갈린다: 회피·특기는 `start_dodge`·`start_special`, 무기는 종류(`WeaponKind`)로 거르고, 기록 키는 캐릭터마다(`clears.<캐릭터>`·`knots_open.<캐릭터>`), 고르기 창은 `app/pick.rs`(마을 뿌리 길). 이긴 보스와 다시 싸우는 실의 꿈은 `dream.rs`(목록·기록 키) + `app/dream.rs`(창·흐름), 꿈 월드는 `World::dream`이라 아무것도 떨어뜨리지 않는다(0055). 저널 목표·보상은 `journal.rs` 표(진행은 기록에서 계산, 런 통계는 `stat.*` 키, 아이디는 바꾸지 않는다, 0048). 월드 패스 뒤 그리는 컷씬 배우는 `Renderer::to_view`로 화면 좌표.
- 그림 문법은 3/4 탑다운(docs/decisions/0006). 아이소메트릭 아님. 격자 지도(필드·마을·쉼터·던전)는 `terrain.rs`가 장소마다 한 장으로 굽는다(벽 윤곽·벼랑·바닥·장식, 0062). 충돌은 칸 그대로고, 그림의 벽 칸 가운데 ±4px는 제 칸 쪽이다. 매 프레임 바뀌는 칸(문·출구·저장)은 위에 그린다. 굽기는 `terrain::Baker`로 나눠 한다(프레임마다 `BAKE_BUDGET`, 굽는 동안 화면 검정 + 앱이 월드 틱을 멈춘다, 최근 장소 세 장 캐시, 0068). 굽기 코드를 바꾸면 `baking_in_steps_matches_one_go`가 나눠 구운 그림 = 한 번에 구운 그림을 본다. 던전 방 모양은 `dungeon::shape_room`(문 앞 길·`KEEP` 자리 보호, 0063).
- 테스트용 방 구성은 `src/scenario.rs`에 한 줄 추가(타이틀 "개발"). 컷씬은 `src/cutscene.rs`의 `Step` 스크립트, JRPG식(docs/decisions/0008). 화면 전체 그림은 `Step::Illust` + `Step::Narrate`, 그림 목록·패닝·입자는 `src/illust.rs`(0039). 튜토리얼은 `src/tutorial.rs`의 단계 + 포포의 말(docs/decisions/0023).
- 설정 저장은 `src/prefs.rs`(웹 localStorage 정수 칸). 칸 형식을 바꾸면 `MAGIC`을 올린다. 런을 넘어 남는 기록은 `src/profile.rs`(깃발 + 수, 문자열 키 — 키 이름은 바꾸지 않는다, 0041). 저장 슬롯 셋(칸 0 = 마지막 슬롯, 슬롯 1 = 예전 한 칸 저장 자리), 슬롯 창은 `app/slots.rs`, 설정(prefs)은 슬롯 밖(0078). 쉼터부터 이어하기는 `suspend.rs`(런 상태 형식) + `app/run.rs`(적기·되돌리기): 쉼터에서만 적고 런이 끝나면 지운다. 런에 남는 상태를 새로 만들면 `RunSave`에 칸을 더한다(0080). 기록은 첫 시작(Normal)만 남긴다. 런 사이 한마디는 `src/story.rs` 표(조건이 가장 구체적인 줄, `once`는 한 번).
- UI 텍스트는 `assets/fonts/Galmuri11-Condensed.ttf`(서브셋: ASCII·Latin-1·라틴 확장 A·키릴·KS X 1001 한글 2,350자 등, 목록은 `assets/LICENSES.md`)로 실제 해상도에 찍는다. 일본어·간체·번체는 Ark Pixel 서브셋(`tools/fonts/build_cjk.py`, 번체는 + Cubic 11). 그 밖의 한자·확장 한글은 안 나온다. 새 글자가 필요하면 서브셋을 다시 만든다(`cargo test`가 표의 글자가 폰트에 있는지 본다).
- 플레이어가 보는 글은 `src/i18n.rs`의 언어별 표에만 쓴다(한국어·영어·일본어·중국어 간체·중국어 번체(대만)·브라질 포르투갈어·프랑스어·스페인어·독일어·러시아어·폴란드어, 0084·0085·0144). 코드에 글 리터럴을 두지 않는다. 숫자는 `{0}` + `fill()`, 데이터 표는 `Tr`. 개발 전용 화면만 예외(docs/decisions/0013).
- 번역은 직접 쓴다. DeepL 같은 기계 번역은 사장이 따로 말할 때만 쓴다(초안 품질이 낮다: 굴 → oyster, 보리 → barley, 털빛 → feather 같은 게임 용어 오독이 많았다). 기존 표의 용어를 먼저 찾아 맞춘다.
- 글을 넣거나 바꾸면 열한 언어를 다 쓰고, 일본어·간체·번체 표를 고쳤으면 `python3 tools/fonts/build_cjk.py <ark-pixel-font 체크아웃> <Cubic_11.woff2>`로 폰트를 다시 만든다(번체는 Ark에 없는 한자를 俐方體11號로 메운다, 0144). 새 화면은 가짜 언어 Pseudo(설정 → 언어 끝)와 일본어·러시아어·포르투갈어로 넘침을 본다. 개발 빌드에서 F3를 누르면 글자 넘침 검사(`ui::check`: 화면 밖·테두리 밖·겹침·`fit`으로 잘림을 콘솔에, 0089)가 켜진다. 고정 칸은 `ui.fit`, 긴 글은 `ui::wrap`(docs/decisions/0014).

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo build --release --target wasm32-unknown-unknown
cargo check --target aarch64-linux-android   # NDK 없이 됨. Android에서만 깨지는 의존성(gamepads 등)을 잡는다
cargo test                                   # 단위·규칙 테스트(무브셋·번역·폰트 글자·지도·굽기·패드 키 등). 리눅스면 libx11-dev 등 필요(README)
python3 tools/licenses.py                    # 의존성을 바꿨을 때만. assets/THIRD_PARTY_LICENSES.txt 갱신
```
**배포는 사장에게 먼저 묻는다**(2026-10-26 사장 지시): 고친 것은 작업 브랜치에만 커밋·push하고, main push·버전 올리기·릴리스는 하지 않는다(main에 push하면 웹이 바로 배포된다). 고친 것을 모아 "이것들을 고쳤는데 v<버전>으로 배포할까요?"라고 묻고, 사장이 다음 수정 사항을 말하면 그것까지 넣어 배포 시점을 함께 정한다.

CI: `deploy-web.yml`이 fmt·clippy·`cargo test`·wasm 빌드를 돌리고 Netlify 시크릿이 있으면 배포한다. `build-android.yml`이 APK를 만들어 아티팩트(태그면 Release)로 올린다. `build-ios.yml`(macOS 러너)이 서명하지 않은 `.ipa`를 아티팩트로 올린다(내 기기 시험은 Sideloadly·AltStore로 직접 서명, 0086). iOS에서는 패드가 아직 안 된다(gilrs에 iOS 백엔드 없음). `cargo check --release --target aarch64-apple-ios`는 리눅스에서도 된다. 앱 아이콘(iOS·Android)은 `tools/icons/make_icons.py`가 코드로 찍은 큰 나무·실에 얽힌 뿌리 실루엣에서 만든다(0087·0123). 릴리스는 `Cargo.toml` 버전을 올리고 `docs/releases/v<버전>.md`를 함께 main에 push하면 `release.yml`이 태그·APK·Release를 만든다(세션 프록시는 태그 push를 받지 않는다). 로컬에 Android SDK/NDK는 두지 않는다(docs/decisions/0005).

공식 페이지(0069): `site/`(한국어 `index.html`, 영어 `en/`)를 `pages.yml`이 GitHub Pages에 올린다. 그림·글꼴은 `python3 tools/site/make_images.py`(초상화 자르기·일러스트 복사·페이지 글자만 남긴 Galmuri), 스크린숏은 `tools/site/shots.js`. 페이지 글을 바꾸면 두 언어를 다 고치고 `make_images.py`를 다시 돌린다(글꼴 글자). 최신 버전·APK·소식은 `site.js`가 GitHub Releases에서 읽는다(못 읽으면 적어 둔 값).

## 하지 않는 것
- `mq_js_bundle.js`나 게임패드 JS 플러그인을 외부 URL로 핫링크하지 않는다. `web/`에 복사본.
- 에셋 팩을 라이선스 확인 없이 커밋하지 않는다. `assets/LICENSES.md`에 기록. **이 레포는 공개다. 유료·제3자 팩 원본은 `assets/vendor/`(gitignore)에만 두고 커밋하지 않는다**(docs/decisions/0010). 팩 그림은 Claude가 보고 고르고 확인해도 되지만 AI 학습·생성 참조에는 쓰지 않는다(0064).
- 직접 그린 그림은 PNG를 손으로 고치지 않는다. 점으로 다듬을 때는 `tools/pixel`(`.pxl` 텍스트 원본, `pxl.py check`의 계단·외톨이 검사, 브라우저 편집기 `editor.html`, 0070). 그 밖에는 `tools/art/*.py`를 고치고 다시 생성한다(Naïl `nail.py`, 적·이웃 `critters.py`, 보스 `chief.py`, 초상화 `portraits.py`(나노바나나 원본이 있는 캐릭터는 `tools/art/sources/ai/` + `ai_portraits.py`가 칸을 바꿔 끼운다, 0108. 일러스트 첨부용 시트 묶음은 `ref_pack.py`, 0124), 타일 `tiles.py`, 컷씬 일러스트·타이틀 바탕 `illust.py`(타이틀은 `title` 장면, 앨범 목록 밖, 0083. 나노바나나 원본이 있는 장면은 `tools/art/sources/ai/illust/` + `ai_illust.py`가 바꿔 끼운다, 0125. 나노바나나 원본을 새로 넣으면 `strip_logo.py`로 ✦ 로고를 원본에서 한 번 지운다, 0133) — 스프라이트·타일은 임시 그림. 지금 게임 초상화(칸 4 그림자 빼고)·일러스트·타이틀은 모두 나노바나나 원본에서 만들고, 코드 그림과 파스텔·수채 바탕(`tools/art/sources/`, `pastel_bg`, 0073)은 원본이 없을 때만 쓴다. 사람이 그린 그림으로 바꿀 때는 `docs/art/cutscene-illustrations.md`).
- 마일스톤(버전 규칙 0079, 지금은 0081) 밖의 큰 기능을 사장 지시 없이 먼저 만들지 않는다. 마이너 버전은 마일스톤을 마쳤을 때만 올린다(패치는 자유).
- `cargo-quad-apk` 커밋(`build-android.yml`의 `CARGO_QUAD_APK_REV`)과 `web/*.js` 복사본은 의도적으로만 올린다. 올리면 CI 콘솔/로그를 확인한다.
- 언어: 문서·커밋 메시지 한국어, 코드 식별자 영어.
