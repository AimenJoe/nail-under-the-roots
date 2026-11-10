# 0286. 개발판 Android 패키지 이름 — 마지막 조각은 nail(켜자마자 죽던 것)

- 날짜: 2026-11-07
- 상태: **채택 — 구현**(사장 2026-11-07: "nail 개발판 Android APK가 켜자마자 꺼지는 버그를 고쳐 줘", Knellroot에서 같은 버그를 확인). 0193(갈래·개발판)을 잇는다.
- 바탕: 0193(개발판 = 패키지 `….nail.dev`, 운영 앱과 나란히), Knellroot `bab436d`(수정)·`045ff55`(에뮬레이터 연기 시험), KB `guides/macroquad-android-apk-github-actions.md` "패키지 이름의 마지막 조각이 네이티브 라이브러리 이름이 된다".

## 무엇이 문제였나
- cargo-quad-apk(`CARGO_QUAD_APK_REV` d411c8f)의 `src/ops/build.rs`는 `library_name = package_name.split(".").last()`를 쓰고, 그 이름을 MainActivity의 `System.loadLibrary(…)`에 넣는다. APK 안의 실제 `.so`는 크레이트 이름(`lib/arm64-v8a/libnail.so`)이다.
- 운영판 `io.github.aimenjoe.nail` → `libnail.so` → 맞다. 개발판 `io.github.aimenjoe.nail.dev` → `libdev.so`를 찾는다 → 없다 → 켜자마자 `UnsatisfiedLinkError`.
- 빌드·apksigner·aapt2 badging 검사는 모두 지나서 CI가 못 잡았다. 0193 뒤로 개발판 APK는 한 번도 켜진 적이 없다(사장도 직접 켜 보지 않았다).

## 정한 것
1. 개발판 패키지 = **`io.github.aimenjoe.dev.nail`**(마지막 조각이 `nail`로 남게). `build-android.yml` "Build flavor"의 sed·확인 줄, "Publish dev APK"의 릴리스 노트 글.
2. `profile::android_package`(Android 기록 폴더 `/data/data/<패키지>/files`를 고른다)는 운영 이름과 새 개발판 이름 둘만 우리 것으로 본다 — 그 밖이면 운영 이름. 옛 `….nail.dev`는 **받지 않는다**: 그 판은 라이브러리를 못 찾아 Rust 코드가 돌기 전에 죽었으니 이 함수에 닿은 적이 없고, 받아 둘 까닭이 없다. 시험 `android_save_dir_follows_the_package`에 새 이름·옛 이름.
3. "Verify APK"에 검사 하나: 패키지 이름의 마지막 조각 = APK 안의 `lib/arm64-v8a/lib<조각>.so`. 다르면 빌드 실패.
4. 에뮬레이터 연기 시험 `android-smoke.yml` + `tools/test/android-smoke.sh`(Knellroot에서 옮김): `release/**`에 이 둘을 고쳐 push하면, 개발판 APK(Release `dev`)를 x86_64 에뮬레이터(API 31, google_apis — ARM 번역)에 깔아 켜고 20초 뒤 살아 있나·화면 둘·logcat을 아티팩트로 남긴다. 켜자마자 죽으면 실패. 그 push가 APK를 다시 만드는 것이면 그 커밋의 APK가 Release `dev`에 올라오길 기다린다. 패키지 이름은 APK에서 읽는다(aapt2).
5. iOS 개발판 번들 아이디(`….nail.dev`)는 그대로 — iOS는 라이브러리 이름을 번들 아이디에서 짓지 않는다.

## 사람이 할 일
- 폰에 깔린 옛 "Naïl dev"(켜지지 않는 것)는 지우고 새 `nail-dev.apk`를 깔기. 패키지가 바뀌어 덮어 깔리지 않고 따로 깔린다.
- 잃는 것은 없다: 개발판 기록은 원래 운영판과 따로였고, 옛 개발판은 켜지지 않았다. (패키지가 바뀌면 옛 개발판 기록 폴더는 새 개발판에서 보이지 않는다.)

## 재현과 확인
- 고치기 전에 재현, 고친 뒤 확인 — 둘 다 같은 에뮬레이터 연기 시험(API 31, x86_64, google_apis), 아래 "실행 기록".

## 실행 기록
- **고치기 전**(2026-11-07, run 42326357635, APK `54af060` = 패키지 `io.github.aimenjoe.nail.dev`): 켜자마자 죽음 — 시험 실패.
  ```
  E AndroidRuntime: FATAL EXCEPTION: main
  E AndroidRuntime: Process: io.github.aimenjoe.nail.dev, PID: 2727
  E AndroidRuntime: java.lang.UnsatisfiedLinkError: dlopen failed: library "libdev.so" not found
  E AndroidRuntime:   at io.github.aimenjoe.nail.dev.MainActivity.<clinit>(MainActivity.java:277)
  I ActivityManager: Process io.github.aimenjoe.nail.dev (pid 2727) has died: fg TOP
  ```
  (첫 실행 42326241110은 에뮬레이터 액션을 옛 v2.9.0에 묶어 시작도 못 했다 — v2.38.0으로 고침.)
- **고친 뒤**(run 42326531704, APK `c52ae79` = 패키지 `io.github.aimenjoe.dev.nail`, `lib/arm64-v8a/libnail.so`): 깔림 `Success`, `Displayed io.github.aimenjoe.dev.nail/.MainActivity: +887ms`, **켜고 20초 뒤 alive, 시작(Enter 둘) 15초 뒤 alive** — 시험 통과. 같은 push의 build-android(run 42326531671)도 새 라이브러리 이름 검사를 지나 성공.
