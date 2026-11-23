# 0301. Microsoft Store — 서명하지 않은 MSIX를 CI가 묶고, 올리기는 사람이

- 날짜: 2026-11-23
- 상태: **채택**(사장 2026-11-23: "위 배포 파이프라인 만들고")
- 바탕: 0240(PC 빌드), 0276(상점 배포 파이프라인), 0299(앱인토스 등급 — 받는 증빙 목록에 Microsoft Store 링크 + 게임위 정보)

## 왜
- 앱인토스 게임 등급(0299) 증빙에 **오픈마켓 링크 + 게임위 자체등급분류 정보**가 필요하다. Microsoft는 IARC 참여 상점이라 Epic에서 받은 Global Rating ID(`docs/epic/store-text.md`)를 가져오면 게임위 등급이 함께 붙는다 — Google Play(D-U-N-S 대기, 0295)보다 먼저 열 수 있다.
- 비용이 없다: 개인 개발자 등록 무료(2025-09, storedeveloper.microsoft.com 새 경로), 회사 계정도 $99 없앰(2026-05 Windows 개발자 블로그) — 옛 문서엔 아직 $99.

## 확인한 것(1차, learn.microsoft.com)
- 두 갈래: **MSIX**는 상점이 서명한다. **MSI/EXE**는 Microsoft Trusted Root에 닿는 인증서로 직접 서명(자체 서명 불가) + 내 서버에 판 고정 URL로 호스팅해야 한다 → MSIX를 고른다.
- 등급: Partner Center 등급 화면에서 기존 IARC/Global Rating ID를 가져올 수 있다(설문은 제품마다 거친다고 적혀 있다).
- MSIX 판 넷째 자리는 상점 몫(0) — `X.Y.Z.0`. 같은 판은 다시 올릴 수 없다.

## 한 것
- `pc/msix/AppxManifest.xml.in`: 데스크톱 앱(`Windows.FullTrustApplication` + `runFullTrust`), x64, Windows 10 1809(17763) 이상, 열한 언어 `Resource`. 로고 넷 `pc/msix/*.png`는 `tools/icons/make_icons.py msix`가 앱 아이콘에서 만든다(크기 꼬리 없이 — resources.pri 없이 묶으니까).
- `pc/msix/identity.env`: Partner Center "제품 ID"의 `Name`·`Publisher`·`PublisherDisplayName`(비밀 아님 — 패키지마다 들어 있다). 비면 시험용 아이디로 묶고 경고한다(상점은 받지 않는다).
- `tools/pc/package.sh windows msstore msstore`: Windows 묶음 폴더 + 로고 + 매니페스트 → Windows SDK `makeappx pack /h SHA256`(매니페스트 형식은 묶을 때 검사된다) → `nail-windows-msstore.msix`.
- `build-desktop.yml` store = `msstore`: Windows만, 운영판(`--no-default-features`, 상점 SDK 없음). `store-deploy.yml` msstore: 그것을 불러 **아티팩트만** — 상점 자격이 없으니 Environment `store`에 들어가지 않는다.
- **C 런타임을 실행 파일에**(`.cargo/config.toml`의 `x86_64-pc-windows-msvc` `+crt-static`): Rust MSVC 빌드는 기본으로 `VCRUNTIME140.dll`을 찾아, VC++ 재배포 패키지가 없는 PC에서는 켜지지 않는다. MSIX는 프레임워크 의존성(VCLibs)을 따로 걸어야 하는데 그 대신 정적으로 넣는다 — Steam·Epic·zip 판도 같이 좋아진다. `package.sh`가 실행 파일에 그 이름이 남았으면 실패시킨다.

## 기록은 그대로
- 기록·설정은 `%APPDATA%\nail`(`profile::data_dir`)에 쓴다. MSIX 앱의 AppData 쓰기는 패키지 전용 폴더로 돌려지므로(그 앱에게는 같은 자리로 보인다) 코드는 그대로 둔다. 같은 PC의 zip 판 기록과는 나뉜다. **실기 확인 전.**

## 아직
- 사람: 계정(개인/회사), 앱 이름 예약, `identity.env` 채우기, 첫 제출(패키지·상점 글·등급·가격). 글·그림은 `docs/msstore/store-text.md`.
- 자동 제출(`msstore` CLI + Entra 앱 자격)은 첫 제출이 통과한 뒤에.
- 실기: 설치·켜기·기록 남기·지우기, 시작 메뉴 타일.
