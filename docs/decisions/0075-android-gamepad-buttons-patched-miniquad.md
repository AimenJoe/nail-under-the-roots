# 0075. Android 패드 버튼: miniquad를 고쳐 쓴다

- 날짜: 2026-10-26
- 상태: 채택
- 후속(2026-10-26): 사장이 실기에서 패드 버튼이 "다 됨"을 확인했다(기기 모델·Android 버전은 기록 없음).

## 배경
사장이 v0.1.5 APK를 기기에 설치해 해 보니, **D패드와 왼쪽 스틱 말고는 아무 버튼도 듣지 않았다.**

원인은 miniquad 0.4.11의 Android 쪽에 있다(소스 확인).
- `java/MainActivity.java`의 `onKey`는 모든 키 이벤트를 Rust로 넘기고 `true`를 돌려준다(시스템이 먹지 않는다).
- `src/native/android/keycodes.rs`의 표에 D패드(0x13~0x17 → 화살표·Enter)는 있다. 그러나 패드 버튼 `KEYCODE_BUTTON_A`(96)~`BUTTON_MODE`(110)는 없어서 모두 `KeyCode::Unknown`으로 버려진다.
- 왼쪽 스틱이 된 것은 우연이다. 앱이 조이스틱 `MotionEvent`를 처리하지 않으면(`onGenericMotionEvent` 없음) Android가 스틱을 D패드 키로 바꿔 준다.
- Android 빌드에서는 `gamepads` 크레이트를 뺐다(Android에서는 빌드가 깨진다, CLAUDE.md "커밋 전"). 그래서 게임 쪽 `read_gamepads`는 빈 값을 돌려주고 있었다.

키코드 값은 developer.android.com `KeyEvent` 문서로 확인했다(BUTTON_A 96, B 97, X 99, Y 100, L1 102, R1 103, L2 104, R2 105, THUMBL 106, THUMBR 107, START 108, SELECT 109, MODE 110).

## 결정
- miniquad 0.4.11 원본을 `third_party/miniquad/`에 두고 루트 `Cargo.toml`의 `[patch.crates-io]`로 쓴다. 고친 곳은 `keycodes.rs`의 표 한 군데다. 패드 버튼을 데스크톱 키보드에 없는 `KeyCode::F13`~`F25`로 넘긴다. 고친 내용과 올릴 때 다시 넣는 법은 `third_party/miniquad/PATCHED.md`에 적는다.
- cargo-quad-apk는 miniquad 패키지 폴더에서 `java/MainActivity.java`를 가져간다. 그래서 patch를 쓰면 이 폴더의 사본을 쓴다(Java는 고치지 않았다).
- 게임 쪽(`input.rs`)은 Android에서 `Pad::key()`(F13~F24)로 버튼을 읽는다. 동작은 `Bindings`의 패드 칸을 따르고, 메뉴는 데스크톱·웹 패드와 같다(A 확인, B 또는 기기 뒤로 = 뒤로, Start 메뉴, LB·RB 탭). 다시 붙이기(`capture`)도 F 키로 패드 버튼을 잡는다.
- Android에서는 화살표 키(D패드·스틱)도 패드 입력으로 센다. 그래야 안내 글이 키보드 글자로 바뀌지 않는다.
- `cargo test`의 `android_pad_keys_match_patched_miniquad`가 패치 표와 `Pad::key`가 같은 짝인지 본다. 한쪽만 고치면 실패한다.
- 고른 이유: 다른 길은 (1) 업스트림 수정을 기다리기(언제 될지 모른다), (2) `cargo-quad-apk`에 Java 파일을 따로 넣기(이 도구에 그런 칸이 없다)였다. 패치는 표 열세 줄이라 올릴 때 다시 넣기 쉽다.

## 한계
- 스틱은 아직 디지털이다(Android가 바꿔 준 D패드 키). 오른쪽 스틱 조준과 아날로그 세기는 `MainActivity.java`에 `onGenericMotionEvent`를 넣어야 한다. 실기에서 필요하다고 느끼면 그때 한다.
- 실기 확인은 사장 몫이다(사람이 할 일). 컨테이너에는 Android 기기가 없다.
