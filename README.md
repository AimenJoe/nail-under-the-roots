# Naïl: Under the Roots

귀엽지만 쓸쓸한 땅속 세계에서, 아기 두더지 Naïl이 발톱과 굴파기로 방을 돌파하며 시들어 가는 나무의 뿌리를 따라 내려가는 탑다운 액션 로그라이트. Rust + macroquad.

- 공식 페이지: https://aimenjoe.github.io/nail-under-the-roots/ (원본 `site/`, `pages.yml`이 GitHub Pages에 올린다, docs/decisions/0069). 게임은 https://nail-under-the-roots.netlify.app
- 컨셉: [`docs/concept.md`](docs/concept.md)
- 현재 단계: [`docs/phase-1-combat-prototype.md`](docs/phase-1-combat-prototype.md) (끝의 "구현 상태"·"사람이 할 일"이 현재 위치. 지금 마일스톤은 `docs/decisions/0081`(v0.3.0), 버전 규칙은 0079)
- 에셋 라이선스: [`assets/LICENSES.md`](assets/LICENSES.md). 스프라이트·대체 타일은 직접 그림(원본 `tools/art/*.py`, `python3 tools/art/<이름>.py`로 PNG 재생성 — nail·critters·chief·tiles). 초상화·일러스트·타이틀은 나노바나나 원본(`tools/art/sources/ai/`)을 `ai_portraits.py`·`ai_illust.py`가 옮긴다(`portraits.py`·`illust.py`가 부름, 새 원본은 먼저 `strip_logo.py`). 앱 아이콘은 `tools/icons/make_icons.py`. 유료 팩은 `assets/vendor/`(레포 밖, docs/decisions/0010)
- 결정 기록: [`docs/decisions/`](docs/decisions/)
- 지식 베이스: [AimenJoe/KB](https://github.com/AimenJoe/KB) (`maps/gamedev.md`)
- 저작권: © 2026 Beardworks. 코드·직접 그린 그림·글은 Beardworks 소유(레포는 공개지만 오픈소스 라이선스를 붙이지 않았다 = 모든 권리 보유). 폰트·의존성은 각자의 라이선스(`assets/LICENSES.md`, `assets/THIRD_PARTY_LICENSES.txt`)

## 실행

```sh
# 데스크톱. Linux 의존성: libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config
#   (libudev는 gamepads→gilrs, libasound는 macroquad audio)
cargo run

# 웹
cargo build --release --target wasm32-unknown-unknown
mkdir -p dist && cp web/index.html web/*.js dist/ && cp target/wasm32-unknown-unknown/release/nail.wasm dist/ && cp -r assets dist/assets
python3 -m http.server 8000 --directory dist   # http://127.0.0.1:8000
```

## 조작

| 동작 | 키보드 | 게임패드 |
|---|---|---|
| 이동 | WASD / 화살표 | 왼쪽 스틱, D-pad |
| 발톱(3연타) | J | X(□) |
| 굴파기 → 솟구침 | K (다시 누르면 조기 솟구침). 공격 중에 누르면 후딜레이 취소 | A(×) |
| 필살기(분노가 차면) | L | B(○) |
| 아이템 쓰기 / 넘기기 | I / Q·E | Y(△) / LB·RB |
| 지도 | M | Select |
| 조준 | (마지막 이동 방향) | 오른쪽 스틱 |
| 재시작 | 메뉴 → 다시 시작 | 메뉴 → 다시 시작 |
| 메뉴(일시정지) | Esc | Start |
| 메뉴 탐색 | 방향키·WASD, Enter/Space 확인, Esc 뒤로, Q/E 탭 | D-pad·스틱, A 확인, B 뒤로, LB/RB 탭 |
| 디버그 | F1 오버레이, F2 히트박스, F3 글자 넘침 검사(개발 빌드) | — |

게임 동작(공격~지도)은 설정 → 조작 설정에서 바꿀 수 있다. 웹에서는 브라우저에 저장된다(docs/decisions/0022).

흐름: 인트로 → 타이틀(시작하기·이어하기·오늘의 굴·앨범·설정·개발) → 슬롯 고르기(0078) → (오프닝 컷씬) → 게임. 게임 중 메뉴를 열면 탭 여섯 개(메뉴·상태·가방·저널·도감·지도).

- **장비**: 캐릭터마다 무기 종류가 다르다. Naïl은 발톱 — 새싹(가벼움, 4연타)·두더지(기본)·바위(무거움, 2연타)·독. 메뉴 → 가방에서 확인으로 장착(docs/decisions/0011).
- **무기 획득**: 처음엔 맨손. 튜토리얼에서 두더지 발톱을 줍고, 이후는 스토리(컷씬 `Give`·바닥 획득물)를 따라 하나씩(docs/decisions/0012). 개발 시나리오 "발톱 4종 비교"는 넷 다 들고 시작.
- **초상화**: 대화창에 88×88 2배 밀도 초상화. 아홉 캐릭터 열세 묶음(물든 모습 포함), 나노바나나 원본(`tools/art/sources/ai/`)을 `ai_portraits.py`가 칸에 끼운다(0108~0122). 틀·남은 칸은 `portraits.py`(0015).
- **장신구·퀵 아이템**: 능력치 = 무기 + 장신구 2칸(체력·공격 속도·굴파기 대기·공격력·분노 충전, 모모 런에서만 나오는 모모 것 셋 0142) + 뿌리 심장(최대 체력 영구 +1). 산딸기(체력 1)·쥬시 지렁이(체력 가득, 들고 있다가 쓰러지면 저절로 일어남 = 목숨, 최대 3)는 가방에 모아 I/Y로 먹고 Q·E/LB·RB로 넘긴다(docs/decisions/0017). 상태 탭에 능력치가 숫자로.
- **적 = 물든 이웃**: 들쥐·생쥐 떼·시궁쥐(돌격), 다람쥐(도토리), 청설모(뛰어 덮치기). 종류 표는 `src/species.rs`, 그림은 `tools/art/critters.py`. 메뉴 → 도감에 만난 이웃이 말로 된 능력(체력·힘·빠르기)과 이야기·요령으로 적힌다(docs/decisions/0021). 개발 시나리오 "적 다섯 종".
- **공격 예고**: 적이 공격을 준비하면 맞을 자리(돌진 길·도토리 줄·착지 고리)가 바닥에 붉게, 공격 순간이 가까울수록 진하게 깜빡인다(docs/decisions/0025).
- **화폐**: 반딧불(기술 해금, 많이) · 씨앗(상점, 드물게·상자·보스). 둘 다 HUD 왼쪽 위(docs/decisions/0021).
- **스테이지 고리**: 필드 남쪽 공터의 튜토리얼 보스(들쥐 대장 보리) → 굴 아래 쉼터(체력 가득·저장·포포 가게·이웃) → 뿌리 길 곁에서 공격 버튼 = 길 지도 → 갈래마다 다른 굴(굴·둥지·저장고·샘) → 그 길의 보스 → 다음 쉼터(docs/decisions/0030~0032). 개발 시나리오 "튜토리얼 보스", "쉼터".
- **모험**: 필드(연속 스크롤, 자는 적, 저장 지점)에서 굴 입구로 절차적 던전(화면 방, 들어서면 문이 닫히고 다 잡으면 열림, 보물·휴식·보스방)으로. 쓰러지면 저장 지점에서 다시(docs/decisions/0019). 개발 시나리오 "필드(모험)", "던전 바로", "아레나 웨이브".
- **패링**: 공격을 적 공격이 닿기 직전(스윙 첫 0.12초)에 앞쪽으로 맞추면 돌진은 크게 멈칫, 투사체는 되돌아간다. SP가 크게 찬다(docs/decisions/0020).
- **필살기 드릴 돌진**: 분노가 차면 L/B. 시동(적이 멈추고 화면이 어두워짐) → 부아아앙 화면 한 폭 돌진, 연타로 적을 벽까지 끌고 가 쾅(docs/decisions/0018, 0024). 개발 시나리오 "장신구·필살기 확인".
- **반딧불·산딸기**: 적이 반딧불(가까이 가면 끌려옴)과 확률로 산딸기(체력 1 회복, 깎인 만큼 잘 나오고 오래 안 나오면 보장)를 떨어뜨린다. 최대 체력 5(docs/decisions/0016). 개발 시나리오 "떨어지는 것 확인".
- **다국어**: 아홉 언어 — 한국어·영어·일본어·중국어 간체·브라질 포르투갈어·프랑스어·스페인어·독일어·러시아어(0033·0084·0085). 기기(브라우저) 선호 언어 중 가진 첫 언어로 시작(없으면 영어), 설정 → 언어에서 바꾼다. 글은 `src/i18n.rs` 한 곳, 빠진 번역·폰트에 없는 글자는 `cargo test`가 잡는다. 일본어·간체 폰트는 Ark Pixel(OFL)에서 `tools/fonts/build_cjk.py`로 만든다. 개발 빌드엔 넘침 검사용 가짜 언어 Pseudo. 언어를 늘린 이유와 순서는 docs/decisions/0014·0084·0085.
- **튜토리얼**: 오프닝 끝에 라쿤 포포가 나타나 9단계로 알려 준다(이동 → 발톱 → 3연타 → 굴파기 → 솟구치기 → 패링 → 필살기 → 아이템 → 메뉴·저장). 단계마다 짧은 대화 + 화면 아래 "[n/9]" 힌트, 연습 동안은 체력이 줄지 않는다(docs/decisions/0023).
- **개발 시나리오**: 타이틀 "개발"에서 필드·던전·아레나·튜토리얼·적 다섯 종·빈 방·들쥐 1/3·다람쥐 1·1+1·웨이브 3부터·오프닝 컷씬 등을 바로 시작한다(목록은 굴러간다). 목록은 `src/scenario.rs` 한 곳. `dev` feature(기본 켬)이며 배포 빌드는 `--no-default-features`로 뺀다.
- **환경**: 꽃 핀 입구(필드·마을) → 뿌리 지대 → 어두운 심층, 장소·길이 깊어질수록 바뀐다(`adventure.rs`). 아레나 개발 시나리오에서만 웨이브가 깊이를 대신한다. 개발 시나리오 "환경: …"으로 바로 볼 수 있다.
- **컷씬**: JRPG식. 들어가면 위아래 레터박스가 내려오고 조작이 돌아오면 걷힌다. 배우가 걷고 아래 대화창에 글자가 찍힌다. 확인으로 넘기고 Esc/B로 건너뛴다. 스크립트는 `src/cutscene.rs`(docs/decisions/0008).

모바일 브라우저(터치): 화면 왼쪽 반을 누른 채 끌면 가상 스틱, 오른쪽 아래 `ATK`·`DIG`·`SP`·`ITEM`·`<>` 버튼, 오른쪽 위 `R` 재시작, 왼쪽 위 `MENU`. 메뉴 안에서는 항목을 탭한다. 터치가 한 번이라도 들어오면 오버레이가 보인다. 가로로 들고 플레이.

## 커밋 전

```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo test                                        # 단위·규칙 테스트(무브셋·번역·폰트 글자·지도·굽기·패드 키 등)
cargo build --release --target wasm32-unknown-unknown
cargo check --target aarch64-linux-android
python3 tools/licenses.py                         # 의존성을 바꿨을 때만: 고지 파일 다시 만들기
```

## 배포

### 웹 (Netlify)
- 사이트: https://nail-under-the-roots.netlify.app (Netlify Site ID `a0bc4242-4bdd-4fc8-8266-adeaf143e59c`, Git 연동 없음)
- PR을 열면 `pr-<번호>--nail-under-the-roots.netlify.app` 에 미리보기가 올라간다.
- `main`에 push하면 `deploy-web.yml`이 wasm을 빌드하고, 레포 Secrets에 `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`가 있으면 배포한다. 시크릿이 없으면 빌드 검증만. Netlify 사이트는 **Git 연동 없이**(Deploy manually) 만든다. 이유와 절차: KB `guides/macroquad-wasm-netlify-github-actions.md`.

### Android (APK)
- `build-android.yml`이 `src/`·`assets/`·`android/`·`third_party/`·`Cargo.*`가 바뀐 push마다 `cargo quad-apk build --release`로 **디버그 키 서명 APK**를 만들어 Actions 아티팩트(`nail-apk-<sha>`)로 올린다. Actions 탭 → 해당 실행 → Artifacts에서 받는다.
- 보통은 아래 "릴리스 = 버전 올리기"(`release.yml`)로 낸다. 손으로 `v*` 태그를 push해도 같은 APK가 Release에 붙는다(세션 프록시는 태그 push를 받지 않는다).
- 실기 설치: 아티팩트 **zip을 풀어** 안의 `nail.apk`만 설치한다(`nail.apk.idsig`는 무시). `adb install -r nail.apk` 또는 사이드로드(알 수 없는 출처 허용). arm64 전용, Android 8.0+.
- **제3자 에셋(Sunnyside)**: 원본은 `assets/vendor/`(gitignore). CI는 비공개 레포 `AimenJoe/nail-assets`를 Secrets `ASSETS_REPO`(=`AimenJoe/nail-assets`)·`ASSETS_TOKEN`(그 레포만 Contents 읽기, fine-grained 토큰, 만료 최대 1년)으로 받는다. 팩에서 새 파일을 쓰면 `src/art.rs` 경로와 같은 자리로 `nail-assets`에도 올린다. 토큰이 만료되면 빌드는 직접 그린 그림으로 돌아간다(실패하지 않는다).
- **릴리스 = 버전 올리기**: `Cargo.toml`의 `version`(과 `[package.metadata.android]`의 `version_code`)을 올리고 `docs/releases/v<버전>.md`(릴리스 본문)를 함께 main에 push하면 `release.yml`이 태그 `v<버전>`을 만들고 `build-android`를 불러 APK를 GitHub Release에 붙인다. 태그가 이미 있거나 본문이 없으면 아무것도 안 한다. Actions 탭에서 `release`를 손으로 돌려도 된다. 손으로 `v*` 태그를 push해도 예전처럼 Release가 생긴다(본문 파일은 있어야 한다).
- **서명 키 고정**: 레포 Secrets에 `ANDROID_DEBUG_KEYSTORE_B64`(디버그 키스토어의 base64, 비밀번호 `android`)가 있어야 모든 빌드가 같은 키로 서명되어 기기에서 덮어 설치가 된다. 없으면 Actions 캐시의 키를 쓰고, 그것도 없으면 새 키를 만들어 경고를 남긴다. 키가 바뀐 APK는 **기기에서 앱을 삭제한 뒤** 설치해야 한다("패키지가 유효하지 않음"의 흔한 원인). 키 만들기: `keytool -genkey -v -keystore debug.keystore -storepass android -alias androiddebugkey -keypass android -dname "CN=Android Debug,O=Android,C=US" -keyalg RSA -keysize 2048 -validity 10000 && base64 -w0 debug.keystore`.
- 로컬에서 만들려면 Android SDK(build-tools, platforms;android-36)·NDK·JDK 17과 `cargo install --git https://github.com/not-fl3/cargo-quad-apk --rev <build-android.yml의 CARGO_QUAD_APK_REV>`, 그리고 `ANDROID_HOME`·`NDK_HOME` 환경 변수가 필요하다. 보통은 CI에 맡긴다(docs/decisions/0005).
- Android 게임패드 버튼은 패치한 miniquad(`third_party/miniquad`)가 F13~F25 키로 넘긴다(docs/decisions/0075). 스틱은 D패드(디지털)로 들어온다.
- 앱 아이콘: `android/res/mipmap-*`(옛 + 적응형, `tools/icons/make_icons.py`로 생성). CI가 APK에 아이콘이 없으면 실패한다(0087).

### iOS (빌드만)
- `build-ios.yml`(macOS 러너)이 push마다 `aarch64-apple-ios`로 빌드해 **서명하지 않은** `nail-ios-unsigned`(.ipa)를 Actions 아티팩트로 올린다. Info.plist는 `ios/Info.plist`, 아이콘은 `ios/icons/`(docs/decisions/0086).
- 그대로는 기기에 깔리지 않는다. 내 기기 시험은 Sideloadly·AltStore(Windows·Mac)로 내 Apple ID 서명(무료면 7일마다 다시). 배포(TestFlight)는 Apple Developer Program이 필요하고 아직 정하지 않았다.
- iOS에서는 게임패드가 아직 안 된다(gilrs에 iOS 백엔드가 없다). 저장은 앱 샌드박스 `Library/Application Support/nail`.
