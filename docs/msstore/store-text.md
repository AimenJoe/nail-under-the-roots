# Microsoft Store 상점 글·그림 (0301, 2026-11-23 초안)

글·그림은 Epic 것을 다시 쓴다(`docs/epic/store-text.md`·`docs/epic/media/`). 다른 상점·무료판으로 이끄는 글은 넣지 않는다(0276).

## Partner Center에서 사람이
1. **계정**: storedeveloper.microsoft.com → Get started for free → 개인 또는 회사(Beardworks). 신분증·셀카 확인. Partner Center로 바로 가면 옛 유료 경로가 뜰 수 있다.
2. **앱 이름 예약**: `Naïl: Under the Roots`(패키지 표시 이름과 같아야 한다 — 다르면 `pc/msix/identity.env`의 `MSIX_DISPLAY_NAME`을 예약 이름으로).
3. **제품 ID**(앱 → 제품 관리 → 제품 ID): `Package/Identity/Name`·`Publisher`·`PublisherDisplayName`을 `pc/msix/identity.env`에 → 커밋 → Actions → store-deploy → `msstore` → 아티팩트의 `.msix`를 패키지 칸에.
4. **연령 등급**: "다른 곳에서 받은 등급 가져오기"에 Epic의 Global Rating ID `e2dc4e64-c235-82d2-8a40-3fbbabbe1eed`. 설문을 물으면 Epic 때와 같은 답.
5. **가격·시장**: 사장이 정한다(무료 / $0.99).
6. **속성**: 범주 게임 → 액션 및 모험(+ 롤플레잉). 개인정보처리방침 `https://beardworks.io/privacy.html`. 시스템 요구: Windows 10 1809 이상 x64, 게임패드·키보드 지원, 터치 없음.
7. **상점 목록(한국어·영어)**: 설명은 Epic About 글, 짧은 설명은 Epic 것, AI 그림 공개 한 줄 그대로. 스크린숏은 `docs/epic/media/carousel-*.png`(1920×1080). 게임 목록의 포스터(9:16)·상자(1:1) 그림은 규격을 원문으로 확인 못 함 — 칸이 요구하면 `tools/store/epic_media.py`에서 같은 방식으로 만든다.

## 앱인토스 등급(0299)
상점 페이지가 열리면 그 링크 + 페이지의 게임위 등급을 토스 콘솔 등급 칸에(채널톡으로 받는지 먼저 확인).
