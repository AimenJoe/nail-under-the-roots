# 0282 키는 자리로, 이름은 지금 배열의 글자로

- 상태: 채택(사장 2026-11-05 "이미 다국어를 지원하는 이상 키보드도 위치 기반으로 해야지")
- 관련: 0280(WASD + 오른손), 0278·0281(웹 키 입력), 0075·0171(패치한 miniquad), KB `reference/macroquad/gamepad-and-android-input.md`

## 왜
프랑스(AZERTY)·독일(QWERTZ) 같은 배열에서도 WASD·J·K·L은 **같은 자리**여야 한다 — 손가락 배치가 기본값의 뜻이다. 읽는 방식을 보니(miniquad 소스):
- 웹 `e.code`, Windows 스캔코드, macOS `NSEvent.keyCode` — 원래 자리.
- **Linux X11·Wayland — 배열을 거친 keysym(글자)**. AZERTY면 W 자리가 Z로 들어왔다.
그리고 화면의 키 이름은 QWERTY 글자로 박혀 있었다("WASD", "Q/E", "J") — 자리로 읽어도 AZERTY 키캡(Z·Q·S·D)과 글이 어긋난다.

## 정한 것
- **입력은 모든 PC에서 자리**: 패치한 miniquad에 `src/native/pc_scancode.rs`(PC 스캔코드 = evdev 코드 → `KeyCode`, 글자·숫자·기호 블록만). X11(`xkey.keycode − 8`)·Wayland(`key`)가 먼저 이것을 보고, 표 밖(화살표·F키·Enter …)은 원래 keysym 번역. 시험 `linux_keys_follow_position`. Xvfb에서 X 키 매핑을 AZERTY처럼 바꿔 W·A·Q·Z 자리를 눌러 보니 원본은 Z·Q·A·W, 패치본은 W·A·Q·Z(2026-11-05, 작은 시험 프로그램).
- **화면 이름은 지금 배열에서 그 자리에 찍힌 글자**(`input::key_name`·`refresh_key_names` — 켤 때·2초마다):
  - 웹: `navigator.keyboard.getLayoutMap()`(Chrome 계열만, Safari·Firefox는 없어 QWERTY 이름) — `web/nail-host.js` 플러그인 `nail_keys`(`nail_key_label`). 게임은 `KeyCode` 값(X11 keysym — 글자·숫자·기호는 US 글자 코드)을 넘기고 셸이 `e.code` 이름으로 바꿔 찾는다.
  - Windows: 스캔코드 → `MapVirtualKeyW(VSC_TO_VK)` → `MapVirtualKeyW(VK_TO_CHAR)`.
  - macOS: 가상 키 → `TISCopyCurrentKeyboardLayoutInputSource` + `UCKeyTranslate`(GLFW `glfwGetKeyName`과 같은 길). 가상 키 표는 miniquad 표의 거꾸로 — 시험 `mac_virtual_keys_match_patched_miniquad`.
  - Linux·Android·iOS: 묻지 않는다(QWERTY 이름).
  - 받은 글자가 라틴(ASCII·Latin-1·라틴 확장 A — 글꼴 범위)이 아니면(러시아 배열의 키릴 등) QWERTY 이름 — 그런 키보드에도 라틴 글자가 같이 찍혀 있다. 대문자로, 공백은 이름이 안 된다. 시험 `key_names_follow_the_layout`(가짜 AZERTY → 걷기 "ZQSD"·탭 "A/E"·지도 ",").
- 걷기 동사(`Verb::Walk`)·탭 동사(`Verb::Tabs`)·조작 설정의 키 칸·고정 안내 글(`ctl_fixed`의 `{0}`, 열한 언어)이 이 이름을 쓴다.

## 확인 못 한 것
- Windows·macOS 실제 AZERTY에서 이름이 바뀌는지(컴파일만 — Windows `x86_64-pc-windows-msvc`, macOS `aarch64-apple-darwin` clippy 통과). macOS는 일부 입력기(자판 자료가 없는 입력 소스)에서 None → QWERTY 이름.
- 0278의 웹 대비책(빈 `code`를 `key`로 채우기)은 글자 기준이라 그 경우에만 자리가 아닐 수 있다 — `code`가 비어 오면 자리 정보가 없다.
- Linux 키 이름(배열 글자): miniquad 안의 X·xkb 핸들이 필요해 이번엔 안 했다. Steam Deck은 US 배열이 기본.
