# 0005: APK 빌드 파이프라인은 지금 만든다 (cargo-quad-apk, CI), 컨트롤러 스파이크는 2단계 유지

- 상태: 채택 (2026-10-24)

## 맥락
0002는 Android를 2단계 스파이크로 미뤘지만, 배포 경로(APK)가 없으면 스파이크 자체를 시작할 수 없고, 웹과 달리 "빌드가 깨졌는지"를 아무도 모른 채 코드가 쌓인다. 그래서 APK **빌드 파이프라인**은 1단계에 넣고, 컨트롤러 입력(miniquad 포크)은 0002대로 2단계에 실측한다.

조사 결과(KB `reference/macroquad/gamepad-and-android-input.md`, `guides/macroquad-android-apk-github-actions.md`):
- `cargo-quad-apk`는 crates.io 0.1.4가 오래됐지만 **git master는 2025~2026에 갱신**됐다(cargo 0.87, API 36 기본, aapt2·d8, 명령줄 도구, NDK 29 `lib64` 대응). miniquad의 `MainActivity.java`를 크레이트 소스에서 가져와 함께 컴파일한다.
- 필요한 도구: `ANDROID_HOME`(build-tools 최신 디렉터리를 자동 선택, `platforms/android-36/android.jar`), `NDK_HOME`, `javac`/`keytool`(JDK). GitHub `ubuntu-latest` 러너에 전부 미리 설치돼 있다(build-tools 34~37, NDK 27/29/30, JDK 17 기본).
- `gamepads` 0.1.7은 Android에서 `android-winit` feature 없이는 **컴파일되지 않는다**(`poll_android_winit` 미정의). 그 feature는 winit 전용이라 miniquad에 못 쓴다.

## 결정
1. `.github/workflows/build-android.yml`: 관련 파일이 바뀐 push와 수동 실행에서 `cargo quad-apk build --release`로 **디버그 키 서명 APK**를 만들어 Actions 아티팩트로 올린다. `v*` 태그면 GitHub Release에 첨부한다.
2. `cargo-quad-apk`는 **git 커밋을 고정**해 설치하고(`CARGO_QUAD_APK_REV`), 빌드한 바이너리를 커밋 해시로 캐시한다.
3. `gamepads`는 `[target.'cfg(not(target_os = "android"))'.dependencies]`로 옮기고 `input.rs`의 패드 코드는 `cfg`로 뺀다. Android에서는 miniquad가 D-pad를 화살표 키로 넘기는 경로만 남는다.
4. `[package.metadata.android]`: `io.github.aimenjoe.nail`, API 36, minSdk 26, arm64만, 가로 고정, 전체 화면.
5. 커밋 전 검사에 `cargo check --target aarch64-linux-android`를 더한다(NDK 없이도 돌아가며 위 3번 같은 컴파일 깨짐을 잡는다).

## 왜
- 로컬에는 SDK/NDK가 없어도 되게 **빌드는 CI에서만** 한다. 1인 개발이라 로컬 환경 두 벌을 유지할 이유가 없다.
- 디버그 키 서명이면 `adb install`·사이드로드로 실기 테스트가 바로 된다. 스토어 배포(릴리스 키)는 그때 가서 `apksigner` 단계와 Secrets를 더한다.
- git 커밋 고정은 상류가 조용히 바뀌어 빌드가 깨지는 것을 막는다(0004의 JS 번들과 같은 원칙).

## 비용
- `cargo-quad-apk` 설치는 `cargo` 크레이트를 컴파일해 첫 실행이 몇 분 걸린다. 캐시가 흡수한다.
- 상류 커밋을 올릴 때마다 CI가 한 번 깨질 수 있다. 올리는 건 의도적으로만.
- Android에서 게임패드 **버튼**은 아직 안 들어온다(0002의 스파이크 범위).

## 재검토 조건
- `gamepads`가 miniquad용 Android 백엔드를 얻거나, miniquad 포크가 완성되면 3번을 되돌린다.
- 스토어 배포가 필요해지면 릴리스 키 서명 단계를 추가하고 `version_code` 자동 증가를 넣는다.

## 후속 (2026-10-25): 서명 키 고정
첫 실기 설치에서 "패키지가 유효하지 않음"이 났다. CI 검증(`apksigner verify`)은 통과했으므로 APK 자체는 멀쩡하고, 원인은 **빌드마다 keytool로 새 디버그 키를 만들어** 서명하기 때문에 이전 빌드 위에 설치할 수 없는 것이다(cargo-quad-apk는 `~/.android/debug.keystore`가 없으면 생성한다. 러너는 매번 새 VM).
- 워크플로에 키 복원 단계를 넣었다: Secrets `ANDROID_DEBUG_KEYSTORE_B64` → Actions 캐시(`android-debug-keystore-v1`) → 새 생성(경고).
- 디버그 키는 사이드로드 전용이다. 스토어 배포 키는 따로 만들고 Secrets로만 다룬다.
- 검증 단계에 키스토어 지문 출력을 추가해 빌드 간 키가 같은지 로그로 확인한다.
