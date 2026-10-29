# 0240. PC 빌드 — Windows · macOS · Linux

- 날짜: 2026-10-29
- 상태: **구현**. 사장: "Windows, macOS, Linux 빌드 추가하면서 0.6 배포하자." 0232 범위(PC 빌드)의 첫 판.

## 왜 쉬웠나 / 무엇이 막혔나
- macroquad(miniquad)가 세 OS를 원래 지원한다. 고치기 전에도 `cargo check --target x86_64-pc-windows-msvc`·`aarch64-apple-darwin`이 경고 없이 통과했다. 저장 자리(`profile::data_dir`: `%APPDATA%\nail`, `~/Library/Application Support/nail`, `~/.local/share/nail`)와 패드(gamepads/gilrs: Windows·macOS·Linux)는 이미 있었다.
- **막힌 것 하나**: miniquad 데스크톱은 파일을 "지금 폴더" 기준으로 연다(`fs.rs load_file_desktop`, `File::open(path)`). 바로가기·Finder·Steam으로 켜면 지금 폴더가 실행 파일 자리가 아니라 그림·소리를 못 찾는다. → `main.rs assets_dir()`: 실행 파일 옆 `assets`, `.app/Contents/Resources/assets`, 그다음 `assets`(cargo run). 리눅스 빌드를 `/`에서 켜서 확인.

## 바꾼 것
- **게임**: 타이틀 "끝내기"(데스크톱만, `TitleItem::Quit` → `order_quit`), 설정 "전체 화면"(데스크톱만, 값 줄 끝 — `SETTING_ROWS`가 빌드마다 길이가 다르다, `crate::DESKTOP`), F11. 설정 칸 21(`prefs`, 옛 저장은 창). Windows 운영판은 콘솔 창 없음(`windows_subsystem`). 글 둘 × 열한 언어, 간체 屏·번체 幕을 글꼴에 더했다(Ark 6e1abb4로 다시 만듦 — 기존 글자 모양은 모두 같음을 비교해 확인).
- **CI** `build-desktop.yml`: Windows(windows-latest, x64, rcedit로 아이콘·버전 → zip), macOS(macos-latest, arm64 + x86_64를 `lipo`로 겸용 `.app`, Info.plist, 임시 서명 → `ditto` zip), Linux(ubuntu-22.04 — glibc를 낮게, tar.gz). 묶음 = 실행 파일 + assets(mp3·vendor/.git 뺌) + 제3자 고지. release/**·손으로·`release.yml`(태그 때 운영판을 Release에, APK 잡 뒤).
- **아이콘**: `tools/icons/make_icons.py`가 `pc/nail.ico`(16~256)·`pc/nail.icns`(16~1024)도 만든다(같은 그림, iOS·Android는 바이트 그대로).
- **홈페이지**: 내려받기 칸에 PC 셋(`site.js`가 Release의 `nail-<os>-` 파일을 찾는다).

## 서명(사람 몫)
- **macOS**: 임시(ad-hoc) 서명만 — lipo로 합치면 arm64 링커 서명이 깨져 애플 실리콘에서 안 켜지므로 다시 서명한다. 개발자 ID 서명·공증이 없으면 macOS 15는 다운로드한 앱(격리 표시)의 첫 실행을 막는다 → "시스템 설정 → 개인정보 보호 및 보안 → 그래도 열기". Apple Developer($99/년) 가입 뒤 인증서·notarytool 키를 CI 시크릿으로(공증). Steam이 공증을 요구한다는 Valve 문서는 찾지 못했다 [확인 못 함].
- **Windows**: 서명 없음 → 직접 받으면 SmartScreen "추가 정보 → 실행". Steam 판매에는 보통 필요 없다 [2차].

## 비용
- macOS 러너는 분당 리눅스의 약 10배(0237). 그래서 main push마다는 돌리지 않는다(release/**·태그·손으로).

## 남은 것
- 실기 시험(사장): Windows PC·Mac에서 켜기, 패드, 전체 화면, 저장 자리, 소리.
- Mac 공증, Steam 데포(OS마다), Steam Deck 확인(리눅스 빌드 또는 Proton으로 Windows 빌드).
