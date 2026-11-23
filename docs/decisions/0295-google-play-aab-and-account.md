# 0295. Google Play 준비 — 조직 계정(D-U-N-S), AAB는 APK에서 바꿔 만든다, 올리기 키

- 날짜: 2026-11-19
- 상태: **채택**(사장 2026-11-19: "Playstore 등록 준비하자" → 조직 계정 쪽으로, "할 수 있는 것 먼저")
- 바탕: 0284(Android 상점판은 Google Play, A1 패키지 `io.beardworks.nail`·A2 정식 키·A3 응원값 가격), 0286(패키지 마지막 조각 = 라이브러리 이름), 0005(로컬에 Android SDK 없음 — 빌드는 CI)

## 계정: 조직(개인사업자)으로 — D-U-N-S부터
Play Console 도움말 원문(answer 13634885·13628312·14151465, 2026-11-19 직접 읽음):

| | 개인 계정 | 조직 계정 |
|---|---|---|
| 정식 출시 전 | 2023-11-13 뒤 만든 개인 계정은 **12명 이상 × 14일 연속** 비공개 테스트 | 없음(그 도움말은 개인 계정 대상) |
| Play에 보이는 것 | 법적 이름·국가·개발자 이메일, **수익을 내면 전체 주소** | 법적 이름·법적 주소·D-U-N-S·개발자 이메일·**전화번호** |
| 필요한 것 | 신분 확인 | **D-U-N-S**(D&B 무료, 최대 30영업일 — 빠른 발급은 유료·선택), 조직 이름·주소·전화·웹사이트 |

Play판은 응원값 유료(0284 A3)라 **어느 계정이든 주소는 보인다** — 주소는 고르는 기준이 아니다. 조직으로 가는 까닭은 둘: 12명 × 14일 시험이 없다, 그리고 앱인토스가 등급 대조에서 "자체등급분류 게임물의 등록자명 = 사업자명"을 본다(앱인토스 콘솔 문서 5-3, 2026-11-19 — GRAC 기록에 Play의 어느 이름이 들어가는지는 확인 못 함).

**지켜야 할 것**: Google 결제 프로필의 법적 이름·주소 = D&B 기록 = 사업자등록증 — 글자까지(다르면 "더 이상 인증되지 않음" → 계정 제한·앱 내려감, 13628312). 상호가 한글(털보네공업사)이라 법적 이름 표기는 사장이 정한다(아래 "사람이 할 일"). Play의 **개발자 이름**(상점에 보이는 이름)은 법적 이름과 따로 `Beardworks`로 둔다.

## AAB: cargo-quad-apk APK → aapt2 proto → bundletool
Play는 새 앱을 AAB로만 받는데 `cargo-quad-apk`는 APK만 만든다(aapt2 link → javac·d8 → zip, 그 안 `build.rs` 확인). 고르지 않은 길: cargo-quad-apk를 고쳐 `--proto-format`으로 링크(포크를 떠안는다), Gradle 프로젝트로 옮기기(빌드를 통째로 바꾼다). 고른 길(`tools/android/make_aab.sh`):

1. 다 만든 APK를 `aapt2 convert --output-format proto` — 매니페스트·리소스 표만 번들 꼴(proto)로, dex·lib·assets·res는 그대로.
2. 모듈 꼴로 옮겨 담기: `manifest/AndroidManifest.xml`, `resources.pb`, `dex/`, `lib/`, `assets/`, `res/`(META-INF는 버린다, 남는 파일이 있으면 실패).
3. `bundletool build-bundle`(소리 ogg·그림 png는 다시 누르지 않는다) → `validate`.
4. 서명은 `jarsigner`: 상점판은 올리기 키, 그 밖은 디버그 키(변환 길이 늘 도는지 보는 시험 — Play는 디버그 번들을 받지 않는다).
5. 검사: 번들에서 `build-apks --mode=universal`로 APK를 다시 뽑아 APK 검사와 같은 것(패키지·아이콘·exported·`lib<마지막 조각>.so`)을 본다.

APK 하나에서 바꾸니 기기에서 확인한 APK와 번들 속 내용이 같다. bundletool은 버전 + sha256으로 고정(`build-android.yml` env).

**16KB 페이지**: Play는 targetSdk 35+ 앱의 64비트 네이티브 라이브러리가 16KB 정렬이기를 요구한다(2차: Android Developers 공지 — 원문 날짜 확인 전). `llvm-readelf`로 LOAD 정렬을 보고, 상점판은 못 맞으면 실패·그 밖은 경고. **첫 CI(2026-11-19, run 44495342893)에서 `libnail.so`의 LOAD 넷이 모두 0x10000(64KB) 정렬** — 지금 NDK·러스트 링크 그대로 맞는다.

첫 CI 결과: AAB 34MB(APK 35MB와 거의 같다), 번들에서 뽑은 universal APK의 패키지·targetSdk 36·아이콘·`arm64-v8a`·exported 모두 그대로.

## 흐름
- 평소 빌드(`build-android.yml`): APK + 디버그 서명 AAB 아티팩트 `nail-aab-<dev|release>-<sha>`.
- 상점판: Actions → `store-deploy` → store = **play** → `build-android.yml`(store: play)이 운영판(`--no-default-features`) + 패키지 `io.beardworks.nail`로 빌드, Environment `store`의 **올리기 키**로 서명한 `nail-aab-play-<sha>`. Play Console에는 사람이 올린다 — 새 앱의 첫 번들은 콘솔에서만(Play Developer API는 앱이 있어야 한다). 자동 올리기(내부 테스트 트랙)는 앱이 생긴 뒤에.
- **Play App Signing**: 앱 서명 키는 Google이 만들어 쥐고, 우리는 올리기 키만 둔다 — 올리기 키는 잃어도 콘솔에서 바꿀 수 있다(0284 A2의 "평생 키" 걱정이 줄어든다). 올리기 키는 사장이 자기 컴퓨터에서 만들고(대화창에 키를 오가게 하지 않는다) Environment `store`에 넣는다.

## 사람이 할 일
- [ ] 법적 이름 표기 정하기: 사업자등록증 그대로(한글) / 홈택스에 영문 상호를 올려 그것으로 통일 — 정한 것을 D&B·결제 프로필에 똑같이
- [ ] 사업장 주소를 공개해도 되는지(유료 앱이면 Play에 보인다) — 안 되면 D-U-N-S 신청 **전에** 사업장 주소를 옮긴다
- [ ] D-U-N-S 조회(my.dnb.com/lookup) → 없으면 D&B에 무료 신청(유료 대행 거절)
- [ ] 번호가 나오면 Play Console 조직 계정 가입($25), 개발자 이름 `Beardworks`
- [ ] 올리기 키 만들기(`docs/release.md` "Google Play") → Environment `store`에 둘, 키 파일은 레포 밖에 백업

## 모르는 것
- GRAC 자체등급분류 DB의 등록자명이 Play의 법적 이름인지 개발자 이름인지.
- 한국 D-U-N-S 신청이 나이스디앤비로 넘어가는지, 그 경로의 수수료(nicednb.com은 세션에서 닿지 않았다).
- 번들에서 뽑은 APK가 기기에서 켜지는지(연기 시험은 아직 APK만 — 상점판 첫 빌드 뒤 내부 테스트 트랙으로 실기).
