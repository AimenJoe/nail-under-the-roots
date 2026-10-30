# 기여 규칙 (CONTRIBUTING)

이 레포는 1인 개발(사장) + Claude가 함께 만든다. 공개 레포지만 오픈소스 라이선스는 없다(모든 권리 보유). 외부 PR을 받을 준비는 되어 있지 않지만, 규칙은 누가 와도 같다. 코드 규칙의 전문은 `CLAUDE.md`, 설계의 "왜"는 `docs/decisions/`.

## 브랜치와 배포
- **main = 운영판 배포.** main에 push하면 웹이 바로 배포되고(Netlify), 버전이 올라가 있으면 Release(APK)가 만들어진다. 그래서 **main에는 배포하기로 정한 것만** push한다. 운영판은 개발 기능 없이 빌드한다(`--no-default-features`, 0193).
- **`release/v<마일스톤>` = 개발판**(지금 `release/v0.6` — v0.6.0 "팔 수 있는 판"은 2026-10-29 배포했고, 0232의 나머지는 v0.6.x에서 진행한다). 작업 브랜치는 여기로 합친다. push하면 개발판 웹(https://dev--nail-under-the-roots.netlify.app)과 APK(미리 보기 Release `dev`의 `nail-dev.apk`, 운영 앱과 나란히 깔린다)가 갈린다. main에 넣은 패치는 이 갈래에도 합친다. main에는 사장이 배포를 정한 패치만 넣는다.
- 평소 작업은 작업 브랜치(`claude/…` 등)에 커밋·push한다. 운영판 배포는 사장이 정한다: 고친 것을 모아 "이것들을 고쳤는데 v<버전>으로 배포할까요?"라고 묻고, 답을 받은 뒤 `docs/release.md`대로.
- PR은 필수가 아니다(1인). PR을 열면 `deploy-web.yml`이 `pr-<번호>--nail-under-the-roots.netlify.app`에 미리보기를 올린다.
- 커밋 메시지·문서는 **한국어**, 코드 식별자는 영어. 커밋 첫 줄에 결정 번호를 붙인다(예: `박자 콤보(0167): …`). 모델 이름·ID는 커밋에 쓰지 않는다.

## 커밋 전 검사(전부 통과해야 push)
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo clippy --target wasm32-unknown-unknown --no-default-features -- -D warnings   # 운영판(0193)
cargo build --release --target wasm32-unknown-unknown
cargo check --target aarch64-linux-android          # Android에서만 깨지는 의존성
cargo clippy --release --target x86_64-pc-windows-msvc --no-default-features -- -D warnings   # PC(0240), 링크 없이 리눅스에서도
cargo check --release --target aarch64-apple-ios    # 리눅스에서도 된다
cargo test                                           # 리눅스: libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config
python3 tools/licenses.py                            # 의존성을 바꿨을 때만
```
CI(`deploy-web.yml`)는 fmt·clippy·test·wasm 빌드를 다시 돈다. 로컬에서 통과한 것만 올린다 — 빨간 CI 한 번이 배포 한 번을 막는다.

## 무엇을 어디에 쓰나
- **수치**는 `src/tuning.rs` 한 곳. 플레이어 코드에 타별 숫자를 직접 쓰지 않는다.
- **표로 끝나는 것**은 표에: 콤보 `moveset.rs`, 적 `species.rs`, 장비·장신구 `equipment.rs`, 재주 `talents.rs`, 저널 `journal.rs`, 런 사이 한마디 `story.rs`, 개발 시나리오 `scenario.rs`, 그림 줄 `art.rs`, 소리 `sfx.rs`·`tools/audio/make_audio.py`(단계 `kenney`·`music`·`oga`, 0234), 어디서 어느 곡·환경음인지 `music::choose` 한 곳(0234·0235 R21), **길의 주인(보스) `bosses.rs`**, **플레이어블 캐릭터 `characters.rs`**(0170).
- **종 규칙은 적 안에**: "맞으면 무슨 일"(튕김·열림·독·벽 쿵)은 `Enemy::receive`, 닿는 판정(혀·등딱지·날기)은 `Enemy`의 메서드. `world.rs`에 종 이름으로 if를 더하지 않는다(0170 R10). 이야기 깃발은 `World::story`의 메서드로만 바꾼다(앱은 `StorySeed`·`Story::beat`, 0170 R11).
- **플레이어가 보는 글**은 `src/i18n/<언어>.rs`에만(키는 `src/i18n/mod.rs`의 `Strings`), **열한 언어 전부**(한국어·영어·일본어·중국어 간체·브라질 포르투갈어·프랑스어·스페인어·독일어·러시아어·중국어 번체·폴란드어, 이 순서). 직접 번역한다(기계 번역은 사장이 말할 때만). 일본어·중국어 글을 바꿨으면 `python3 tools/fonts/build_cjk.py <ark-pixel-font> <Cubic_11.woff2>`로 폰트를 다시 만든다 — `cargo test`가 빠진 글자를 잡는다.
- **흐름**(어디서 시작하나·쓰러지면 어디로·전환 뒤에 무엇을)은 `app/flow.rs`·`app/after.rs`에서 정하고 시험을 더한다. `App`에는 결정을 실행하는 코드만(0230 R15).
- **입력**은 `Actions`로만. `KeyCode`는 `input.rs` 밖에서 보지 않는다.
- **소리**는 로직에서 `self.juice.audio.play(id)`(큐)로만. `Sfx`(오디오 장치)는 `app`만 안다 — 그래서 월드를 테스트에서 틱 단위로 돌릴 수 있다. 새 메커닉은 `World::step`의 단계 메서드 중 맞는 곳에 넣고(플레이어 쪽은 `tick_player`의 `player_rumble`·`player_events`·`drill_fx`), 쓰러짐은 `enemy_down`을 부른다. 보스 2단계는 `Enemy::rouse`·`roused` 한 곳(종마다의 "반 아래 한 수 더"는 `*_next`, 0233). 새 위험물은 `Hazards`에 칸을 더하고 `clear_fight`(싸움이 남긴 것)에 넣는다 — 치우는 곳은 거기 한 곳(0230).
- **파일 자리**: 그리기는 `src/render/`(hud·actors·fx·props·sorted·shadow·lights·deep·descent), 장면은 `src/app/`(stage·slots·dream·pick·run·pause·settings·album·title·help + 소리·그림 없는 흐름 결정 `flow`·`after`). 자식 모듈은 `use super::*;`, 항목은 `pub(super)`. 한 파일이 2~3천 줄을 넘으면 쪼갠다(0169).
- **그림**은 PNG를 손으로 고치지 않는다. 직접 그린 것은 `tools/art/*.py`를 고치고 다시 생성, 나노바나나 원본은 `tools/art/sources/ai/`에 넣고 `strip_logo.py` 한 번 → `ai_portraits.py`/`ai_illust.py`가 옮긴다. 옛 코드 그림은 `tools/art/legacy/`(고치지 않는다). 변환기(`sprite_ai.py` 등)·원본을 고친 뒤에는 `python3 tools/art/check_rebuild.py` — 시트 24장이 커밋된 PNG와 점 하나까지 같아야 한다(바꾸려던 시트만 다르면 그것만 커밋). 렌더에서 칸은 숫자 대신 `art::frame` 이름(0230). 유료 팩 원본은 커밋하지 않는다(`assets/vendor/`, gitignore). 출처는 `assets/LICENSES.md`.
- **저장 형식**: `profile.rs`의 키 이름은 바꾸지 않는다. `prefs.rs` 칸 형식을 바꾸면 `MAGIC`을 올린다. 런 상태를 새로 만들면 `suspend.rs`의 `RunSave`에 칸을 더한다.

## 결정 기록
- 설계를 바꾸거나 새 메커닉을 넣으면 `docs/decisions/NNNN-<제목>.md`를 **새 번호**로 쓴다(기존 문서를 고쳐 쓰지 않는다. 되돌리면 "덧붙임: 철회(새 번호)"만 남긴다). 날짜·상태(제안/채택)·사장의 말·왜·결정·확인(테스트 이름, 사람이 볼 것)·다시 볼 조건.
- 세션이 끝나면 `docs/phase-1-combat-prototype.md`의 **구현 상태 표**와 **사람이 할 일**을 갱신한다. `CLAUDE.md`의 "코드 규칙"에 새 모듈·표의 자리를 한 줄 더한다.
- 프로젝트가 사라져도 유용한 지식은 이 레포가 아니라 KB(`github.com/AimenJoe/KB`)에 쓴다. KB 규칙(frontmatter·`verified`·지도 링크·`python3 tools/kb.py check && index`)을 따른다.

## 테스트
- 새 메커닉에는 단위 테스트 하나 이상: 규칙(순서·수치·버려지지 않음)을 테스트 이름으로 읽을 수 있게(`combo_on_the_beat_is_fastest_and_mashing_is_slower`).
- 눈으로 봐야 하는 것(연출·화면)은 헤드리스 Chromium으로 찍어 본다(Playwright, 개발 시나리오로 진입). 스크린숏은 결정 기록의 "확인"에 적는다.
- 웹 빌드 연기 시험 `node tools/test/smoke.js dist`(0169 R7): 타이틀·개발 시나리오 아홉 개에 입력을 넣고 콘솔 에러·패닉·멈춘 화면을 잡는다. CI(`smoke-web.yml`)가 main push마다 돌리므로 로컬에서는 wasm·JS 번들·로딩 화면을 건드렸을 때만.
- 손맛은 테스트로 못 잰다. "사람이 볼 것"에 남기고 사장이 판정한다.

## 하지 않는 것
- 마일스톤(`docs/decisions/0145`) 밖의 큰 기능을 사장 지시 없이 먼저 만들지 않는다. 마이너 버전은 마일스톤을 마쳤을 때만.
- `web/*.js` 복사본·`cargo-quad-apk` 커밋(`build-android.yml`의 `CARGO_QUAD_APK_REV`)·`third_party/miniquad`는 의도적으로만 바꾼다. 바꾸면 CI 로그를 본다.
- 외부 URL 핫링크, 라이선스 미확인 에셋, 코드 안의 글 리터럴.
