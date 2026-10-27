# 0143. Android 아날로그 스틱·오른쪽 스틱 조준

- 날짜: 2026-10-27
- 상태: 채택. v0.3.0 조건 9(0081·0140). 0075(버튼 패치)의 "스틱은 아직 디지털"을 잇는다.

## 결정
- 패치한 miniquad(`third_party/miniquad`)의 `java/MainActivity.java` `QuadSurface`에 `onGenericMotionEvent`를 넣는다.
  `SOURCE_JOYSTICK`의 `ACTION_MOVE`이면 축 여섯(왼쪽 `AXIS_X/Y`, 오른쪽 `AXIS_Z/RZ`, D패드 `AXIS_HAT_X/Y`)을
  `QuadNative.surfaceOnJoystick`으로 넘기고 **true**(받았다)를 돌려준다. 축 고르기는 Android 공식 문서 "Handle controller actions"대로.
- Rust(`src/native/android.rs`): JNI 진입점이 축을 원자값(f32 비트)에 적고, `miniquad::native::joystick_axes()`가 마지막 값을 돌려준다.
  Java UI 스레드가 쓰고 게임 스레드가 읽으니 메시지 대신 원자값(상태라 마지막 값만 있으면 된다).
- 게임(`src/input.rs` `android_axes`): 왼쪽 스틱이 데드존(`STICK_DEADZONE` 0.2, 데스크톱·웹과 같음)을 넘으면 세기 그대로 이동,
  아니면 D패드(HAT + 키코드로 오는 화살표)를 8방향으로. 오른쪽 스틱은 `Actions::aim`(데스크톱과 같은 조준).
  메뉴 한 칸은 이동 벡터에서 만들던 그대로(`nav_edges`).

## 왜
- 처리하지 않은 조이스틱 이벤트는 Android(`ViewRootImpl`의 `SyntheticJoystickHandler`)가 왼쪽 스틱·HAT를 D패드 키로 바꿔 다시 넣는다.
  받기만 하고 false를 돌려주면 같은 기울기가 아날로그 축과 화살표 키(제 반복까지)로 두 번 들어와, 메뉴에서 두 칸 가거나 반복 박자가 엇갈린다.
  그래서 받았다고 하고, 그 대신 HAT도 직접 받는다(D패드가 HAT로 오는 패드가 화살표를 잃지 않게). 키코드로 오는 D패드는 그대로 `onKey`.
- 오른쪽 스틱은 공식 문서대로 Z/RZ만. RX/RY는 패드마다 뜻이 달라(옛 DualShock 4는 트리거) 받지 않는다.
- 트리거 축(`AXIS_LTRIGGER/RTRIGGER`)은 받지 않는다: L2·R2는 이미 버튼 키(`BUTTON_L2/R2`, 0075)로 듣는다.

## 남은 것
- 실기 확인은 사장(이 세션은 APK를 기기에 올릴 수 없다). CI(`build-android.yml`)가 Java를 컴파일한다.
- 기울인 채로 패드가 끊기면 마지막 축이 남는다(Android가 중앙 값을 보내지 않으면). 실기에서 보이면 `InputManager` 연결 끊김 콜백으로 0을 적는다.
- 과거 표본(`getHistoricalAxisValue`)은 쓰지 않는다: 상태만 보니 마지막 값이면 된다.
