# Microsoft Store 상점 글·그림 (0301, 2026-11-23)

본문 글은 Epic 것을 다시 쓴다(`docs/epic/store-text.md` "About" — 영어·한국어, AI 공개 한 줄 포함). 다른 상점·무료판으로 이끄는 글은 넣지 않는다(0276). 그림은 `python3 tools/store/msstore_media.py` → `docs/msstore/media/`(규격은 그 스크립트 머리말 — Microsoft 문서 2026-11-23).

## Partner Center에서 사람이(순서)
1. **계정**: storedeveloper.microsoft.com → Get started for free → 개인 또는 회사(Beardworks). 신분증·셀카 확인. Partner Center로 바로 가면 옛 유료 경로가 뜰 수 있다.
2. **앱 이름 예약**: `Naïl: Under the Roots` — 예약 이름과 패키지 표시 이름이 같아야 한다(다르면 `pc/msix/identity.env`의 `MSIX_DISPLAY_NAME`을 예약 이름으로).
3. **제품 ID**(앱 → 제품 관리 → 제품 ID): `Package/Identity/Name`·`Publisher`·`PublisherDisplayName` 세 값을 Claude에게 → `pc/msix/identity.env`에 커밋 → Actions → store-deploy → `msstore` → 아티팩트의 `.msix`를 제출의 "패키지"에.
4. **가격·시장**: 사장이 정한다(무료 / $0.99 — Steam·Epic과 맞출지). 시장은 전 세계.
5. **속성**: 범주 게임 → 액션 및 모험(하위 롤플레잉), 개인정보처리방침 `https://beardworks.io/privacy.html`, 지원 연락처는 이메일 — 지원 링크에 GitHub는 넣지 않는다(Epic 반려 이유와 같게). 하드웨어: 키보드·게임패드 지원, 터치 없음. 최소 Windows 10 1809 x64.
6. **연령 등급**: "이미 IARC 등급이 있다" → Global Rating ID `e2dc4e64-c235-82d2-8a40-3fbbabbe1eed`(Epic에서 받은 것). 설문을 물으면 Epic 때와 같은 답(폭력: 만화풍·피 없음, 결제·채팅·확률형 없음).
7. **상점 목록**(영어 필수 + 한국어): 아래 칸 → 제출.

## 상점 목록 칸
| 칸 | 넣을 것 |
|---|---|
| 설명 | Epic "About" 본문 그대로(영어·한국어), 마지막 줄 AI 공개 |
| 짧은 설명 | Epic "짧은 설명"(영어 176자·한국어) |
| 이 버전의 새로운 기능 | 아래 "1.0.1" |
| 제품 기능(줄마다 하나) | 아래 "기능" |
| 검색어 | mole, roguelite, action, pixel art, parry, top-down, cute (한국어: 두더지, 로그라이트, 액션, 도트, 패링, 탑다운, 귀여운) |
| 스크린숏 | 영어 목록 `media/screenshot-en-*.png`(1920×1080, 6장), 한국어 목록 `media/screenshot-ko-*.png`(5장) |
| 상자 그림 1:1 | `media/box-art-2160x2160.png` |
| 포스터 | 화면이 묻는 비율로 `media/poster-1440x2160.png`(2:3) 또는 `media/poster-1440x2560.png`(9:16) |
| 히어로 | 글자 없는 것 `media/hero-3840x2160.png`, 제목 넣은 것 `media/hero-titled-1920x1080.png` |
| 저작권 | © 2026 Beardworks |

### 이 버전의 새로운 기능 (1.0.1)
- EN: First Microsoft Store release. The game pauses and saves when you leave it, the map key closes the menu it opened, holding opposite directions follows the last key, and the Great Tree's pick-one-of-two cells are easier to read.
- KO: Microsoft Store 첫 출시. 게임을 내려 두면 일시정지하고 저장하며, 지도 키로 연 메뉴는 같은 키로 닫히고, 맞선 방향 키는 나중에 누른 쪽을 따르며, 큰 나무의 둘 중 하나 칸이 더 잘 보인다.

### 기능
- EN: Beat-timed claw combos, heavy strikes and parries / Burrow under attacks and burst up / Branching path maps with burrows dug fresh every run / Four zones and a Path Keeper at the end of each path / Free the neighbors and grow the village under the great tree / Play as Naïl, Momo or Doto / Great Tree talents, a separate tree for each hero / Boss rush, journal goals, bestiary and album / Keyboard and gamepad with remappable controls / Assist options and warnings readable without color / 11 languages
- KO: 박자에 맞춘 발톱 콤보·강타·받아치기 / 땅속으로 피하고 솟구쳐 들이받기 / 매번 새로 파이는 굴과 갈래 지도 / 지대 넷과 길 끝의 길의 주인 / 이웃을 풀어 큰 나무 아래 마을을 키우기 / 나일·모모·도토로 플레이 / 캐릭터마다 따로인 큰 나무 재주 / 보스 다시 싸우기·발자국 목표·도감·앨범 / 키보드·게임패드, 버튼 바꾸기 / 도움 설정과 색 없이도 읽히는 예고 / 11개 언어

## 앱인토스 등급(0299)
상점 페이지가 열리면 그 링크 + 페이지의 게임위 등급을 토스 콘솔 등급 칸에(채널톡으로 받는지 먼저 확인).
