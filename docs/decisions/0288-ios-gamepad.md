# 0288. iOS 게임패드 — GCController를 프레임마다 읽는다

- 날짜: 2026-11-10
- 상태: **채택 — 구현**(사장 2026-11-10: "Android에서 했던 것처럼, iOS에도 컨트롤러 지원 추가해 줘"). 0086 "다음 2."(패드 다리)를 한다. iOS 상점판·TestFlight 일정은 그대로 v1.0 밖(0284).
- 바탕: 0075·0143·0171(Android 패드 — 패치한 miniquad가 버튼을 F13~F25 키로, 축을 `joystick_axes`로), 0086(iOS 빌드), KB `reference/macroquad/gamepad-and-android-input.md`·`reference/macroquad/ios-build.md`.

## 무엇이 문제였나
- `gamepads` 크레이트는 iOS에서 gilrs를 쓰는데 gilrs-core 0.5.15에는 iOS 백엔드가 없다(빈 백엔드, 0086). 그래서 iOS 앱은 패드를 꽂아도 아무것도 안 들어왔다. 터치만 됐다.

## 정한 것
1. **Android와 같은 결, 다른 길**: Android는 OS가 이벤트를 밀어 주므로(`onKey`·`onGenericMotionEvent`) miniquad를 고쳐 키·축으로 넘겼다. iOS의 GameController 프레임워크는 지금 값을 들고 있어(Apple: 게임 루프에서 읽으려면 폴링) **miniquad를 고치지 않고** 게임 쪽 `src/input/ios_pad.rs`가 렌더 프레임마다 `+[GCController controllers]` → `extendedGamepad`를 읽는다. Objective-C 메시지는 miniquad가 이미 쓰는 `objc-rs` 0.2.8(같은 판 — 새 크레이트 없음, 라이선스 목록도 그대로)로 보낸다.
2. **연결**: 처음 읽을 때 `GCControllerDidConnect/DidDisconnectNotification`을 빈 처리기로 구독한다(Apple 권장. iOS 15부터 프레임워크는 `controllers`를 처음 읽거나 알림을 구독할 때 패드 찾기를 시작한다). 붙고 떨어지는 것은 프레임마다 다시 읽는 목록이 그대로 본다.
3. **짝**: 버튼은 자리로 — `buttonA` 아래·`buttonB` 오른쪽·`buttonX` 왼쪽·`buttonY` 위(소니·닌텐도 패드도 Apple이 자리로 맞춘다), 어깨·트리거, Start = `buttonMenu`, Select = `buttonOptions`, L3·R3 = 스틱 누르기. 홈 버튼은 시스템 몫(게임 센터·스크린숏)이라 읽지 않고, `preferredSystemGestureState`로 시스템 제스처를 끄지도 않는다(Apple: "드문 경우에만"). 없는 단추(nil)는 안 눌린 것.
4. **한 프레임 = 순수한 값 `PadFrame`**(버튼 `PADS` 순서·D패드 넷·스틱 둘, 위가 +y) → `pad_frame_actions(prev, cur)`가 `Actions`로. 짝은 데스크톱·웹·Android와 같다(게임 동작은 `Bindings`, 메뉴 A 확인·B 뒤로·Start 메뉴·LB/RB 탭, D패드 = 이동 + 메뉴 한 칸, 데드존 `STICK_DEADZONE`). 패드가 여럿이면 버튼은 어느 쪽이든·스틱은 더 기운 쪽. 이 순수 함수는 리눅스 `cargo test`가 본다(`ios_pad_frame_maps_like_other_pads`·`ios_pad_frames_merge`·`pad_order_matches_pads`). 조작 설정의 다시 붙이기(`Input::capture`)도 같은 두 프레임을 견준다.
5. `gamepads` 크레이트는 iOS에서도 뺀다(Android처럼) — 빈 백엔드를 두 길로 섞지 않는다.
6. **iOS 앱의 첫 장치 = 터치**(Android와 같다). 전엔 키보드여서 인트로 안내가 키 이름으로 나올 수 있었다. 패드를 만지면 그때부터 패드 이름(Xbox 글자 A·B·X·Y — Android와 같다).
7. `ios/Info.plist`에 Xcode의 Game Controllers 기능이 넣는 두 키: `GCSupportsControllerUserInteraction` = true, `GCSupportedGameControllers` = [`ExtendedGamepad`]. App Store에 "게임 컨트롤러 지원" 표시가 붙는 근거(2차 — libGDX 위키).
8. `build-ios.yml`이 실행 파일에 `GameController.framework`가 링크됐는지 `otool -L`로 본다.
9. `GCController` 클래스는 `Class::get`으로 찾고 없으면 패드 없음(`class!`는 못 찾으면 panic — objc-rs 소스). 같은 날 Knellroot 0137이 miniquad 패치 쪽으로 같은 다리를 놓으며 정리한 "두 길 모두 꼭" 다섯(링크 보장·`Class::get`·알림·autoreleasepool·합치기)과 견주어 빠진 이것만 더했다(KB `reference/macroquad/ios-build.md`).

## 하지 않은 것
- **진동**: GCController 진동은 Core Haptics 엔진(`controller.haptics`)을 거쳐야 해 코드가 크다. Android처럼 지금은 없다 — 설정의 "진동" 줄도 iOS에서는 그대로 숨긴다.
- 화면 위 가상 패드(`GCVirtualController`), 패드 불빛·배터리, 백그라운드 입력.

## 확인한 것·못 한 것
- 1차: 이 컨테이너에서 `cargo clippy --release --target aarch64-apple-ios`(개발판·운영판) 통과, `cargo test`의 짝 시험. Apple 문서(Info.plist 키 형식 `ProfileName`, `preferredSystemGestureState` 기본값·"드문 경우"), SDL `SDL_mfijoystick.m`(알림 구독 + 목록, 스틱 y 위가 +, iOS 13의 Menu·Options, 홈은 시스템 몫).
- 2차: iOS 15 지연 열거(Apple 개발자 포럼 답), App Store 컨트롤러 표시.
- **못 한 것**: 링크(macOS 러너 `build-ios`가 본다)와 실기. 실기는 사람이 — 아래.

## 사람이 할 일
- 실기: 개발판 `.ipa`(Actions `build-ios`의 `nail-ios-unsigned-dev`)를 Sideloadly로 깔고, 블루투스 패드(Xbox·DualSense·Switch Pro 중 가진 것)를 붙여 — 메뉴(A 확인·B 뒤로·D패드·스틱), 걷기(스틱 세기·D패드), 오른쪽 스틱 조준, 강타 모으기(누르고 있기), Start 일시정지, 조작 설정에서 다시 붙이기, 앱을 켠 뒤 패드를 붙이고 떼기, 화면을 만지면 터치 안내로·패드를 만지면 패드 안내로 바뀌는지.
