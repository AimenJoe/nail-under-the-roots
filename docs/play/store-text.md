# Google Play 등록정보·앱 콘텐츠 답 (0295, 2026-11-19 초안)

그림은 `python3 tools/store/epic_media.py && python3 tools/store/play_media.py` → `docs/play/media/`(규격은 그 스크립트 머리말). 긴 글은 Epic About 글(`docs/epic/store-text.md`)을 그대로 쓰되, Play판은 **터치로도 끝까지 한다**(0284 A4)는 줄만 바꾼다. 상점 글에 무료판(웹·APK)을 알리지 않는다(0276).

## 기본 스토어 등록정보
| 칸 | 값 |
|---|---|
| 앱 이름(30자) | `Naïl: Under the Roots` |
| 기본 언어 | English(미국) — 한국어는 번역 추가 |
| 짧은 설명(80자) EN | `A cute mole action roguelite: parry to the beat, free neighbors bound by thread.`(80자) |
| 짧은 설명 KO | `아기 두더지의 귀여운 액션 로그라이트. 박자에 맞춰 받아치고, 실에 묶인 이웃을 구하자.` |
| 자세한 설명(4,000자) | Epic About 글 영어·한국어 + 아래 "조작" 줄 바꾸기 + AI 공개 한 줄 |
| 앱 아이콘 | `media/icon-512.png` |
| 그래픽 이미지 | `media/feature-1024x500.png` |
| 휴대전화 스크린샷 | `media/screenshot-1…6-*.png`(영어 화면 1920×1080) |
| 앱/게임 · 카테고리 | 게임 · 액션 |
| 이메일 | `support@beardworks.io` |
| 웹사이트 | `https://nail.beardworks.io/en/` |
| 개인정보처리방침 | `https://beardworks.io/en/privacy.html` |

자세한 설명에서 바꾸는 줄(Epic 글의 PLAY YOUR WAY / 내 방식대로):
- EN: `Touch controls or a gamepad, with remappable buttons. Popo's Cheer assist mode, warnings readable without color, and options to tone down shake, flashes and rumble. Eleven languages.`
- KO: `터치·게임패드, 버튼 바꾸기. 포포의 응원(도움 모드), 색 없이도 읽히는 예고, 흔들림·번쩍임·진동 줄이기. 열한 언어.`

## 앱 콘텐츠(정책) 답
| 항목 | 답 | 근거 |
|---|---|---|
| 데이터 보안 | **수집·공유하는 데이터 없음** | Android 판은 네트워크 코드가 없고(`quad_net`·HTTP 크레이트 없음) 권한도 요청하지 않는다. 기록은 기기 안 앱 폴더에만 |
| 광고 | 없음 | |
| 앱 액세스 | 제한 없음(로그인 없음) | |
| 콘텐츠 등급 | Play 안 IARC 설문 — Epic에서 받은 Global Rating ID(`docs/epic/store-text.md` "IARC 등급")를 넣는 칸이 있으면 그것으로 | 같은 답이어야 한다(폭력: 만화풍 동물 싸움, 결제 없음, 이용자 간 소통 없음) |
| 타겟층 | **13세 이상**으로 고르는 것을 제안 — 13세 미만을 넣으면 가족 정책(Families)의 추가 요건이 붙는다 | 사장이 정한다 |
| 뉴스 앱·정부 앱·금융 기능·건강 | 아니요 | |
| 가격 | 응원값 유료(0284 A3) — 금액은 사장이 정한다 | 유료면 판매자 계정(결제 프로필) 필요, 개발자 주소가 Play에 보인다(0295) |

## 확인 못 한 것
- Play 콘텐츠 등급 화면에 다른 상점의 IARC 아이디를 넣는 칸이 있는지(IARC는 재사용을 말하지만 Play 화면은 아직 못 봄).
- 그래픽 이미지 위 재생 단추는 홍보 동영상(YouTube 링크)을 넣을 때만 얹힌다고 알려져 있다 — 영상을 넣으면 가운데가 가려지는지 콘솔 미리보기로 본다.
