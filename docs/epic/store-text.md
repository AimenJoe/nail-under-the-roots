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
