# 0291. iOS ipa를 Release에 붙이고 공식 페이지에서 "직접 설치"로 알린다

- 날짜: 2026-11-10
- 상태: **채택 — 구현**(사장 2026-11-10: "1,2,3 다해줘. 나중에 문의사항으로 받으면 되지뭐"). 0086 "정한 것"의 "Release에는 아직 붙이지 않는다"를 바꾼다. iOS 상점판(TestFlight·App Store)은 그대로 v1.0 밖(0284).
- 바탕: 0086(서명 안 한 ipa), 0288(iOS 패드 — v1.0.0에 들어 있다), 0276(웹·APK 무료 전판 그대로), KB `reference/macroquad/ios-build.md`.

## 정한 것
1. **릴리스마다 운영판 ipa를 Release에 `nail-ios.ipa`로 붙인다**: `release.yml`의 `ios` 잡이 `build-ios.yml`을 `release_tag`로 부른다(APK 잡이 Release를 만든 뒤). 손으로 돌릴 때도 `release_tag`를 넣으면 이미 있는 Release에 붙인다 — build-desktop과 같은 꼴. 운영판은 비공개 에셋이 없으면 실패(0277), 에셋 체크아웃은 `persist-credentials: false`.
2. **공식 페이지(두 언어)**:
   - 첫 화면 버튼은 파일 형식으로: "Android .apk"·"iOS .ipa"(사장 2026-11-10 — APK와 같은 꼴로). iOS는 "준비 중(TestFlight 예정)"이었다. 설치 방법은 해 보기 칸.
   - 해 보기에 "iPhone · iPad (직접 설치)" 칸: 쉬운 길은 브라우저(홈 화면에 추가)라고 먼저 말하고, Sideloadly·AltStore로 내 Apple ID 서명, iOS 16+ 개발자 모드·설정에서 개발자 신뢰, 무료 Apple ID는 7일마다 다시·앱 셋까지.
   - `site.js`: 최신 Release에 `.ipa`가 있으면 그 주소, 없으면(붙기 전) 없는 파일 대신 Release 페이지로.
   - 파일 줄·플랫폼 칸·기록 저장 FAQ·게임패드 FAQ("직접 설치한 iOS 앱에서도 된다")를 고침. 구조화 데이터(`gamePlatform`)에는 iOS를 넣지 않는다 — 상점판이 아니고 실기 확인 전이라.
3. **실기 확인 없이 연다**(사장): 문제가 생기면 문의로 받아 고친다.

## 왜
- 무료 전판(웹·APK)과 같은 결: 앱으로 하고 싶은 사람에게 길을 열어 둔다. 대부분은 브라우저가 더 쉬워서 큰 버튼보다 설치 칸으로 보낸다.
- 서명 안 한 ipa는 받는 사람이 자기 Apple ID로 서명한다(2차 — Sideloadly FAQ 등). 우리 쪽 Apple 계정·비용이 들지 않는다.

## 확인한 것·못 한 것
- 1차: 배포본과 같은 `make_images.py --stamp`로 찍은 페이지를 헤드리스 Chromium으로 한국어·영어 × 1280·390·320px — 가로 넘침 0, 설치 칸 안 넘침 0. 워크플로 YAML 읽힘.
- **못 한 것**: 실기 설치·실행(사장 결정으로 건너뜀), `release_tag` 경로의 실제 실행(main에 들어간 뒤 첫 실행에서 본다).

## 사람이 할 일
- 이 PR을 Merge한 뒤 Actions → build-ios → Run workflow(main) → `release_tag`에 `v1.0.0` — v1.0.0 Release에 `nail-ios.ipa`가 붙는다(그 전까지 페이지의 ipa 버튼은 Release 페이지로 간다). 다음 릴리스부터는 저절로.
