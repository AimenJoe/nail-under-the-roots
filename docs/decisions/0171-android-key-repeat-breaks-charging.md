# 0171. Android에서 땅파기·강타를 모을 수 없었다: 패드 버튼을 누르고 있으면 반복 "누름"이 들어온다

- 날짜: 2026-10-27
- 상태: 채택(고침). 사장: "땅파기 모으는게 없어졌네..?"(v0.3.9, Android 핸드헬드 + 패드).

## 무슨 일이었나
웹(키보드)에서는 모아진다 — 헤드리스로 K를 1초 누르고 있으면 흙빛 고리가 자라다 0.8초에 크게 솟구친다(0158 그대로). Android에서는 안 됐다.

원인은 게임이 아니라 **패치한 miniquad의 Android 키 입력**(`third_party/miniquad`, 0075). Android는 버튼을 누르고 있으면 약 0.4초 뒤부터 50ms마다
`ACTION_DOWN`을 반복해서 보낸다(`KeyEvent.getRepeatCount()`가 는다). miniquad 원본의 `MainActivity.onKey`는 그 반복을 가리지 않고 전부
`surfaceOnKeyDown(keyCode)`로 넘기고, Rust 쪽은 `key_down_event(keycode, mods, false)`로 "반복 아님"이라고 전한다. macroquad는 `repeat == false`일 때만
`keys_pressed`에 넣으므로, 누르고 있는 내내 `is_key_pressed`가 50ms마다 다시 참이 됐다.

게임에서는 그것이 **새 누름**이다: 땅속에서 `dodge_buf`가 다시 차 `early`(톡 두 번 = 바로 솟구침, 0101)가 켜져 0.4초쯤에 힘 0으로 솟구쳤고(모으기 0.45초가 시작도 못 함),
평타를 누르고 있으면 스윙 중에 `queued`(연타, 0167)가 들어가 모으기(0161) 대신 다음 타가 나갔다. 웹은 `event.repeat`, 데스크톱은 OS가 플래그를 주므로 멀쩡했다.
0158(땅파기 모으기)·0161(강타 모으기) 때 헤드리스(웹)로만 확인하고 실기 확인은 "사람이 볼 것"으로 넘겼던 자리다.

## 결정
- `java/MainActivity.java`: `surfaceOnKeyDown(keyCode, event.getRepeatCount() > 0)`. `QuadNative.java`·`src/native/android.rs`(`Message::KeyDown { repeat }`)가
  플래그를 `key_down_event`까지 전한다. `PATCHED.md`에 기록.
- 게임 쪽은 손대지 않는다: `is_key_pressed`의 뜻("새로 눌렀다")이 플랫폼마다 같아야 입력 코드가 한 벌로 남는다. 입력 층에서 눌림 가장자리를 따로 계산하는
  우회는 두지 않는다(0036: 장치는 입력 층 안에서만 다르다).
- 테스트 `android_key_repeat_reaches_macroquad`: 세 파일에 패치가 남아 있는지(0075의 `android_pad_keys_match_patched_miniquad`와 같은 방식). miniquad를 올리면 같이 깨져서 알려 준다.
- 메뉴도 같아진다: 전엔 Android에서 D패드·버튼을 누르고 있으면 메뉴가 0.4초 뒤부터 쭉 내려갔다(키보드·웹은 한 칸). 이제 한 칸. 길게 눌러 넘기기가 필요하면 입력 층에 따로 둔다.

## 확인
- `cargo check --target aarch64-linux-android`로 Rust 쪽이 컴파일되는 것, `cargo test`. Java는 CI `build-android`(작업 브랜치 수동 실행)로 APK가 만들어지는 것까지.
- 사람이 볼 것(사장, Android 실기): 땅파기를 누르고 있으면 흙빛 고리가 자라다 크게 솟구치는지, 평타 뒤 계속 누르면 강타가 모이는지, 메뉴에서 버튼 한 번 = 한 칸인지.
