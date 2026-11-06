# Epic Games Store 상점 글·그림 (0276 E5, 2026-11-06)

규격은 Epic "Storefront Media Guide"·"Offer Details" 원문(2026-11-06 확인). 짧은 설명의 글자 수 제한은 원문에서 못 찾았다 — 상점 곳곳과 검색·공유 미리보기에 쓰인다고만 되어 있어 200자 안쪽으로 썼다. 필수 언어는 영어뿐.

## 짧은 설명 (Offers → General Details → Short Description)
- **English** (176자):
  A cute top-down action roguelite. Naïl, a baby mole, follows the roots of a withering great tree underground — parry to the beat and free the neighbors bound by purple threads.
- **한국어**:
  아기 두더지 나일이 시들어 가는 큰 나무의 뿌리를 따라 땅속으로 내려가는 귀여운 탑다운 액션 로그라이트. 박자에 맞춰 받아치며, 보라색 실에 묶인 이웃들을 풀어 주세요.

## About 글에 넣을 AI 공개 한 줄 (그림을 숨기지 않는다, 0232)
- EN: Character sprites, portraits and illustrations were made from originals generated with a Google Gemini image model, then converted to pixel art and touched up by hand.
- KO: 캐릭터 스프라이트·초상화·일러스트는 Google Gemini 이미지 모델로 그린 원본을 도트로 옮겨 손으로 다듬은 것입니다.

## 그림 (`python3 tools/store/epic_media.py` → `docs/epic/media/`)
| 파일 | 규격 | 올리는 곳 |
|---|---|---|
| `product-logo-960x540.png` | 960×540 PNG ≤1MB, 투명 | Store Settings → Product configuration → Product details |
| `offer-landscape-2560x1440.png` | 2560×1440 | Offers → Images: 일반(general) + 라이브러리(library) — "copy"로 같은 그림을 써도 된다 |
| `offer-portrait-1200x1600.png` | 1200×1600 | 같은 곳(가로·세로 하나씩 필수) |
| `carousel-1…6-*.png` | 1920×1080 ≤4MB | Pages → Product home → Media carousel(최소 1개, 영상 1개를 권장) |
| `social-1200x1200.jpg` | 1200×1200 JPG | Pages → Social preview(선택) |

About 칸 그림은 폭 700px 이상이면 되므로 캐러셀 그림을 다시 써도 된다. 추가 이미지(선택)는 1920×1080 **JPG ≤1MB**라 PNG를 그대로 못 쓴다 — 필요하면 변환.

## 스토어 설정 → 제품 구성 → 제품 세부 정보 (2026-11-06)
| 칸 | 값 |
|---|---|
| 제품 로고 * | `media/product-logo-960x540.png` |
| 제품 아이콘 | `media/product-icon-512.png`(앱 아이콘 `web/icons/icon-512.png` 그대로) |
| 제품 표시명 * (64) | `Naïl: Under the Roots` — 특수 문자가 안 들어가면 `Nail: Under the Roots` |
| 짧은 설명 * (280) | 위 "짧은 설명" 영어(한국어는 현지화 메뉴에서) |
| 장르 * | Action · Rogue-Lite · Indie (목록에 있는 이름으로) |
| 기능 * | Single Player · Controller Support · Achievements. 클라우드 저장(Epic)은 아직 안 만들었으니 고르지 않는다 |
| 로그인 요건 | 없음(게임 안 계정 없음 — Epic 런처 로그인만) |
| 지원 언어 · 오디오 | 비움(음성 없음) |
| 지원 언어 · 텍스트 * | English · Korean · Japanese · Chinese (Simplified) · Chinese (Traditional) · Portuguese (Brazil) · French · Spanish (Spain) · German · Russian · Polish |
| 법적 정보 푸터 * | 아래 |
| 개인정보 보호 링크 | `https://beardworks.io/en/privacy.html` |
| 이용 약관 · EULA | 비움(따로 만든 것 없음) |
| 지원 페이지 링크 | `https://github.com/AimenJoe/nail-under-the-roots/issues` |
| 지원 이메일 | `support@beardworks.io` |
| 개발자 · 퍼블리셔 이름 * | `Beardworks` · `Beardworks` |

법적 정보 푸터:

    © 2026 Beardworks (털보네공업사). All rights reserved.
    Environment art: Sunnyside World by danieldiggle. Music and sound effects: CC0 works by Cleyton Kauffman, fluffclipse, Kistol, Aureolus_Omicron, Yoiyami, KarateStudios, Juhani Junkala, mutantleg, Umplix, RandomMind, Écrivain, cynicmusic, beardalaxy, Trex0n, Kenney, artisticdude, Bobjt, themightyglider, qubodup, syncopika, cinameng and JaggedStone. Fonts: Galmuri, Ark Pixel and Cubic 11 (SIL Open Font License 1.1).
    Character sprites, portraits and illustrations were made from originals generated with a Google Gemini image model, then converted to pixel art and touched up by hand.
    Full third-party licenses are listed in the game under Settings → Licenses.

## 페이지 → Product home → About (필수: 글 + 그림 1장 이상, 10,000자 안)
그림은 `media/carousel-1-meadow.png`(폭 700px 이상이면 된다). 상점 글에는 무료판(웹·APK)을 알리지 않는다(0276).

### English
    The great tree is withering. Purple threads have crept along its roots and wrapped the underground folk one by one, turning them against anyone who passes. Naïl, a baby mole, leaves the burrow alone for the first time to follow the roots down.

    FIGHT LIKE A MOLE
    Claw combos that hit harder on the beat, heavy strikes that slam foes into walls, burrowing under attacks and bursting up into a ram, and parries that knock charges and thrown attacks back. Every enemy shows its strike in red first — parry the lanes, dodge the circles.

    A NEW PATH EVERY RUN
    Pick your way on a branching path map: burrows, nests, caches, springs and tangles, each dug fresh every time. Four zones grow darker the deeper you go, each with its own creatures and dangerous ground, and a Path Keeper waits at the end of every path. Gathered glowbugs and seeds are never lost when you fall.

    WAKE THE NEIGHBORS, BRING THEM HOME
    Beat a tangled neighbor and the threads come loose. Momo the rabbit, Doto the squirrel, Kongi the armadillo and more settle in the village under the great tree, and the tree blooms again. Wake Momo and Doto and play as them too — hind-leg kicks and leaps, or acorns from range.

    KEEP DIGGING
    • Great Tree talents: three branches of eight steps, a separate tree for each hero
    • Cat's Cradle difficulty 1–10, the Thread Dream boss rush, neighbors' requests, journal goals, bestiary and album
    • A fourth path that opens after you see an ending

    PLAY YOUR WAY
    Keyboard or gamepad, with remappable controls. Popo's Cheer assist mode, warnings readable without color, and options to tone down shake, flashes and rumble. Eleven languages.

    Character sprites, portraits and illustrations were made from originals generated with a Google Gemini image model, then converted to pixel art and touched up by hand.

### 한국어 (현지화)
    큰 나무가 시들어 간다. 뿌리를 타고 번진 보라색 실이 땅속 이웃들을 하나둘 감아, 지나는 이를 막아서게 만들었다. 아기 두더지 나일이 처음으로 혼자 굴을 나서 뿌리를 따라 내려간다.

    두더지답게 싸운다
    박자에 맞추면 더 세지는 발톱 콤보, 적을 벽까지 밀어붙이는 강타, 땅속으로 파고들어 공격을 흘리고 솟구쳐 들이받기, 돌진과 던지기를 되받아치는 받아치기. 적은 공격하기 전에 붉게 예고한다 — 띠는 받아치고, 원은 피하자.

    런마다 새 길
    갈래 지도에서 다음 굴을 고른다. 굴·둥지·저장고·샘·실타래, 매번 새로 파인다. 깊이 내려갈수록 어두워지는 지대 넷, 지대마다 다른 생물과 위험한 바닥, 길 끝마다 기다리는 길의 주인. 모은 반딧불과 씨앗은 쓰러져도 잃지 않는다.

    이웃을 깨워 마을로
    실에 물든 이웃을 이기면 실이 풀린다. 토끼 모모, 다람쥐 도토, 아르마딜로 콩이…… 깨어난 이웃은 큰 나무 아래 마을에 자리를 잡고, 나무는 다시 꽃을 피운다. 모모와 도토를 깨우면 직접 골라 내려갈 수도 있다 — 뒷발차기와 멀리뛰기, 또는 멀리서 던지는 도토리.

    더 파고들 것
    • 큰 나무 재주: 세 갈래 × 여덟 깊이, 캐릭터마다 따로
    • 실뜨기 1~10 단계, 실의 꿈(보스 다시 싸우기), 이웃의 부탁, 발자국 목표, 도감과 앨범
    • 엔딩을 본 뒤 열리는 넷째 길

    내 방식대로
    키보드·게임패드, 버튼 바꾸기. 포포의 응원(도움 모드), 색 없이도 읽히는 예고, 흔들림·번쩍임·진동 줄이기. 열한 언어.

    캐릭터 스프라이트·초상화·일러스트는 Google Gemini 이미지 모델로 그린 원본을 도트로 옮겨 손으로 다듬은 것입니다.

## 페이지 → Media carousel (필수: 1개 이상, 1920×1080)
순서대로 `carousel-1-meadow` → `2-roots` → `3-boss` → `4-village` → `5-silk` → `6-hall`. Epic은 영상 하나를 권한다(1920×1080 MP4, 30·60fps, ≤300MB, AAC 스테레오 48kHz) — 트레일러는 따로.
