# 0086. iOS 빌드: 우선 서명하지 않은 .ipa(내 기기 시험), 배포는 나중에

- 날짜: 2026-10-26
- 상태: 채택(1단계). 사장: "일단은 내 기기에서 테스트하고, 향후 배포도 생각해 봐야지."

## 확인한 것
- **엔진**(1차, 소스):
  - miniquad 0.4.11에 iOS 백엔드(`native/ios.rs`, Metal·GLES)가 있고, README에 iOS 빌드 방법이 있다.
  - 파일은 `NSBundle mainBundle`의 `pathForResource`로 읽는다 → `assets/` 폴더를 .app 바로 아래에 두면 지금 경로("assets/…")가 그대로 맞는다.
  - quad-snd는 macOS·iOS에 CoreAudio를 쓴다.
  - 이 컨테이너(리눅스)에서 `cargo check --release --target aarch64-apple-ios`가 통과했다. 링크·실행은 macOS 러너에서만 확인된다.
- **게임패드는 iOS에서 지금 안 된다**(1차, 소스): `gamepads` 크레이트는 iOS에서 gilrs를 쓰는데, gilrs-core 0.5.15에는 iOS 플랫폼이 없어 기본(빈) 백엔드가 된다. 터치는 된다.
  - 패드를 쓰려면 Apple GameController(`GCController`)를 잇는 작은 다리가 필요하다(Android 패치 0075처럼 키 코드로 넘기기).
- **저장 자리**: iOS는 앱 샌드박스라 HOME 바로 아래(`~/.local/share`)에 쓸 수 없다. macOS와 같은 `~/Library/Application Support/nail`로 바꿨다(`profile.rs`).
- **서명**(2차, 웹 글):
  - 서명하지 않은 앱은 시뮬레이터에서만 돈다.
  - 실제 기기에는 서명이 필요하다:
    - 무료 Apple ID: AltStore·SideStore·Sideloadly로 직접 서명한다. 7일마다 다시 서명하고, 앱은 세 개까지다.
    - Apple Developer Program(연 $99): 1년 서명, TestFlight·App Store.
  - EU에는 공식 대체 마켓이 있다(2026 규칙은 확인 못 함).
- **CI**(2차): 공개 레포의 GitHub 호스트 macOS 러너는 무료다. 자동 서명(Xcode)은 CI에서 쓰기 어렵다 → 인증서·프로필을 Secrets로 두고 직접 서명하거나 fastlane match를 쓴다.

## 정한 것(1단계)
- `.github/workflows/build-ios.yml`(macos-latest):
  - `cargo build --release --target aarch64-apple-ios`로 빌드한다.
  - `Payload/Nail.app`에 실행 파일·assets·아이콘·Info.plist·PkgInfo를 넣는다.
  - `nail-ios-unsigned.ipa`를 Actions 아티팩트로 올린다. Release에는 아직 붙이지 않는다(서명 없는 파일은 그대로 설치되지 않는다).
- `ios/Info.plist`: 번들 아이디는 Android와 같은 `io.github.aimenjoe.nail`이다.
  - 버전·빌드 번호는 Cargo.toml의 `version`·`version_code`를 쓴다.
  - 가로 화면만 쓰고, 상태 표시줄을 숨긴다. `UILaunchScreen`(스토리보드 없이 전체 화면, iOS 14+).
  - 아이콘은 에셋 카탈로그 없이 `CFBundleIconFiles`로 지정한다.
- 아이콘: `tools/icons/make_icons.py`(Android와 같은 스크립트, 0087)가 Naïl 초상화를 정수배로 키워 어두운 흙빛 + 아래쪽 따뜻한 빛 위에 올린다(120·152·167·180·1024).
- 시험 방법(사장): Actions의 build-ios 실행 → 아티팩트 `nail-ios-unsigned`를 받는다 → 압축을 풀어 .ipa를 Sideloadly(또는 AltStore)로 내 Apple ID 서명·설치한다 → iPhone 설정 → 일반 → VPN 및 기기 관리에서 개발자 앱을 신뢰한다. 무료 계정은 7일마다 다시 해야 한다.

## 다음(배포를 생각할 때)
1. 실기에서 화면·터치·소리·저장·글꼴이 되는지 본다(1단계 결과).
2. 게임패드 다리(`GCController` → 키 코드). 핸드헬드·컨트롤러가 목표라 배포 전에 필요하다.
3. 서명:
   - Apple Developer Program에 가입한다.
   - 배포 인증서(.p12)·프로비저닝 프로필·App Store Connect API 키를 Secrets에 둔다.
   - CI에서 codesign → TestFlight에 올린다.
4. App Store 준비: 개인정보 처리 안내(수집 없음), 스크린숏, 나이 등급, 아이콘 1024(만들어 둠).
