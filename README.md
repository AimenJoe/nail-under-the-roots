# Naïl: Under the Roots

귀엽지만 쓸쓸한 땅속 세계에서, 아기 두더지 Naïl이 발톱과 굴파기로 방을 돌파하며 시들어 가는 나무의 뿌리를 따라 내려가는 탑다운 액션 로그라이트. Rust + macroquad.

- 컨셉: [`docs/concept.md`](docs/concept.md)
- 현재 단계: [`docs/phase-1-combat-prototype.md`](docs/phase-1-combat-prototype.md) (1단계 전투 프로토타입, 회색 박스)
- 에셋 라이선스: [`assets/LICENSES.md`](assets/LICENSES.md). 캐릭터·대체 타일은 직접 그림(원본 `tools/art/*.py`, `python3 tools/art/nail.py && python3 tools/art/tiles.py`로 PNG 재생성). 유료 팩은 `assets/vendor/`(레포 밖, docs/decisions/0010)
- 결정 기록: [`docs/decisions/`](docs/decisions/)
- 지식 베이스: [AimenJoe/KB](https://github.com/AimenJoe/KB) (`maps/gamedev.md`)

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
| 발톱(3연타) | J / Z | X(□) / R1 |
| 굴파기 → 솟구침 | K / X / Space (다시 누르면 조기 솟구침). 공격 중에 누르면 후딜레이 취소 | A(×) / R2 |
| 조준 | (마지막 이동 방향) | 오른쪽 스틱 |
| 재시작 | R | — (메뉴 → 다시 시작) |
| 메뉴(일시정지) | Esc | Start / Select |
| 메뉴 탐색 | 방향키·WASD, Enter 확인, Esc 뒤로, Q/E 탭 | D-pad·스틱, A 확인, B 뒤로, L1/R1 탭 |
| 디버그 | F1 오버레이, F2 히트박스 | — |

흐름: 인트로 → 타이틀(시작·설정·개발) → (오프닝 컷씬) → 게임. 게임 중 메뉴를 열면 탭 다섯 개(메뉴·상태·가방·저널·지도). 가방·지도·저널은 아직 틀만 있다(docs/decisions/0007).

- **장비**: 캐릭터마다 무기 종류가 다르다. Naïl은 발톱 — 새싹(가벼움, 4연타)·두더지(기본)·바위(무거움, 2연타)·독. 메뉴 → 가방에서 확인으로 장착(docs/decisions/0011).
- **무기 획득**: 처음엔 맨손. 튜토리얼에서 두더지 발톱을 줍고, 이후는 스토리(컷씬 `Give`·바닥 획득물)를 따라 하나씩(docs/decisions/0012). 개발 시나리오 "발톱 4종 비교"는 넷 다 들고 시작.
- **초상화**: 대화창에 2배 밀도 도트 초상화(Naïl 표정 4, 정체 모를 이웃). 원본 `tools/art/portraits.py`(docs/decisions/0015).
- **장신구·퀵 아이템**: 능력치 = 무기 + 장신구 2칸(체력·공격 속도·굴파기 대기·공격력·SP) + 생명의 씨앗(최대 체력 영구 +1). 산딸기(체력 1)·쥬시 지렁이(체력 가득, 들고 있다가 쓰러지면 저절로 일어남 = 목숨, 최대 3)는 가방에 모아 F/Y로 먹고 Q/L1로 넘긴다(docs/decisions/0017).
- **필살기 드릴 돌진**: SP가 차면 L/B. 연타하며 직진, 방향은 조금만, 벽에 부딪히면 쿵(docs/decisions/0018). 개발 시나리오 "장신구·필살기 확인".
- **반딧불·산딸기**: 적이 반딧불(화폐, 가까이 가면 끌려옴)과 확률로 산딸기(체력 1 회복, 깎인 만큼 잘 나오고 오래 안 나오면 보장)를 떨어뜨린다. 최대 체력 5(docs/decisions/0016). 개발 시나리오 "떨어지는 것 확인".
- **다국어**: 한국어·영어·일본어·중국어 간체. 기기(브라우저) 선호 언어 중 가진 첫 언어로 시작(없으면 영어), 설정 → 언어에서 바꾼다. 글은 `src/i18n.rs` 한 곳, 빠진 번역·폰트에 없는 글자는 `cargo test`가 잡는다. 일본어·간체 폰트는 Ark Pixel(OFL)에서 `tools/fonts/build_cjk.py`로 만든다. 개발 빌드엔 넘침 검사용 가짜 언어 Pseudo. 다음 언어 계획은 docs/decisions/0014.
- **튜토리얼**: 첫 시작은 오프닝 컷씬 뒤 플레이 튜토리얼(이동 → 허수아비 3연타 → 굴파기 → 솟구침)을 거쳐 웨이브 1로. 글은 아래 한 줄 힌트뿐.
- **개발 시나리오**: 타이틀 "개발"에서 튜토리얼·무거운 무기(실험)·빈 방·돌격형 1/3·원거리형 1·1+1·웨이브 3부터·오프닝 컷씬을 바로 시작한다. 목록은 `src/scenario.rs` 한 곳. `dev` feature(기본 켬)이며 배포 빌드는 `--no-default-features`로 뺀다.
- **환경**: 꽃 핀 입구 → 뿌리 지대 → 어두운 심층. 지금은 웨이브 1~2/3~4/5~가 깊이를 대신한다. 개발 시나리오 "환경: …"으로 바로 볼 수 있다.
- **컷씬**: JRPG식. 들어가면 위아래 레터박스가 내려오고 조작이 돌아오면 걷힌다. 배우가 걷고 아래 대화창에 글자가 찍힌다. 확인으로 넘기고 Esc/B로 건너뛴다. 스크립트는 `src/cutscene.rs`(docs/decisions/0008).

모바일 브라우저(터치): 화면 왼쪽 반을 누른 채 끌면 가상 스틱, 오른쪽 아래 `ATK`·`DIG` 버튼, 오른쪽 위 `R` 재시작, 왼쪽 위 `MENU`. 메뉴 안에서는 항목을 탭한다. 터치가 한 번이라도 들어오면 오버레이가 보인다. 가로로 들고 플레이.

## 커밋 전

```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo test                                        # 규칙 테스트(무브셋·독)
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
- `build-android.yml`이 `src/`·`Cargo.*`가 바뀐 push마다 `cargo quad-apk build --release`로 **디버그 키 서명 APK**를 만들어 Actions 아티팩트(`nail-apk-<sha>`)로 올린다. Actions 탭 → 해당 실행 → Artifacts에서 받는다.
- `v*` 태그를 push하면 같은 APK가 GitHub Release에 첨부된다: `git tag v0.1.0 && git push origin v0.1.0`.
- 실기 설치: 아티팩트 **zip을 풀어** 안의 `nail.apk`만 설치한다(`nail.apk.idsig`는 무시). `adb install -r nail.apk` 또는 사이드로드(알 수 없는 출처 허용). arm64 전용, Android 8.0+.
- **서명 키 고정**: 레포 Secrets에 `ANDROID_DEBUG_KEYSTORE_B64`(디버그 키스토어의 base64, 비밀번호 `android`)가 있어야 모든 빌드가 같은 키로 서명되어 기기에서 덮어 설치가 된다. 없으면 Actions 캐시의 키를 쓰고, 그것도 없으면 새 키를 만들어 경고를 남긴다. 키가 바뀐 APK는 **기기에서 앱을 삭제한 뒤** 설치해야 한다("패키지가 유효하지 않음"의 흔한 원인). 키 만들기: `keytool -genkey -v -keystore debug.keystore -storepass android -alias androiddebugkey -keypass android -dname "CN=Android Debug,O=Android,C=US" -keyalg RSA -keysize 2048 -validity 10000 && base64 -w0 debug.keystore`.
- 로컬에서 만들려면 Android SDK(build-tools, platforms;android-36)·NDK·JDK 17과 `cargo install --git https://github.com/not-fl3/cargo-quad-apk --rev <build-android.yml의 CARGO_QUAD_APK_REV>`, 그리고 `ANDROID_HOME`·`NDK_HOME` 환경 변수가 필요하다. 보통은 CI에 맡긴다(docs/decisions/0005).
- 아직 Android에서 게임패드 **버튼**은 안 들어온다(D-pad만). docs/decisions/0002의 2단계 스파이크 범위.
