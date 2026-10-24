# 0002: macroquad 유지, Android 컨트롤러 리스크는 2단계 스파이크로 실측

- 상태: 채택, 조건부 (2026-10-24)

## 맥락
최종 목표에 Android 게임 핸드헬드 + 컨트롤러 필수가 추가됐다. 확인 결과(KB `reference/macroquad/gamepad-and-android-input.md`):
- macroquad에는 게임패드 API가 없다. 데스크톱·웹은 `gamepads` 크레이트로 해결.
- Android에서는 그 크레이트도 동작하지 않고, miniquad Android 백엔드는 스틱 축을 받지 않으며 게임패드 버튼 키코드도 매핑하지 않는다(D-pad만 화살표로 들어옴).
- `cargo-quad-apk`는 4년 이상 릴리스가 없다.
- Bevy도 gilrs 기반이라 같은 공백. Godot은 플랫폼은 전부 되지만 Rust(gdext)가 웹에서 실험적.

## 결정
macroquad를 유지한다. 단,
1. 입력은 `Actions` 레이어로 추상화해 어느 쪽으로 가도 전투 코드가 살아남게 한다.
2. **2단계(버티컬 슬라이스) 안에 Android 스파이크를 넣는다**: 회색 박스 빌드를 `cargo-quad-apk`로 APK로 만들어 실기 핸드헬드에서 컨트롤러로 테스트하고, miniquad 포크 패치(버튼 키코드 매핑 + `onGenericMotionEvent` 축 전달, 약 200줄)를 실제로 작성한다.

## 재검토 조건
- 스파이크가 **1주 안에** 끝나면 macroquad 확정.
- 1주로 안 끝나거나 `cargo-quad-apk`가 최신 NDK에서 복구 불가면, 콘텐츠가 없는 그 시점에 Godot(GDScript 중심, 필요 시 gdext) 전환을 결정한다.

## 비용
miniquad 포크 유지. 웹 경량성(wasm 수백 KB)과 지금까지의 파이프라인은 그대로.
