# 컷씬 일러스트 나노바나나 프롬프트

> 컷씬 일러스트 11장 + 타이틀 바탕을 Google 나노바나나(Gemini 이미지 모델)로 그릴 때 쓰는 프롬프트다. 장면 사양은 [`cutscene-illustrations.md`](cutscene-illustrations.md)를 따른다. AI 그림을 쓰기로 한 결정과 지킬 것은 `docs/decisions/0107`.

## 0. 쓰는 법
1. **모델**: Nano Banana 2.1(`gemini-nano-banana-2.1`)이 기본, 인물이 많은 장면은 Nano Banana Pro(`gemini-3-pro-image`). **Nano Banana 2 Lite는 쓰지 않는다**(참고 그림 여러 장·여러 번 고치기에 맞추지 않은 모델). 옛 Nano Banana(2.5 Flash Image)는 참고 그림 3장까지가 좋다. [1차]
2. **참고 그림 한도**: 2.1은 캐릭터 4명까지 닮게, 물건 10개까지. Pro는 캐릭터 5명, 물건 6개, **그림체 참고 3장**. 한 장면에 넣는 캐릭터 그림은 이 안에서. [1차]
3. **첨부 순서**: 캐릭터 그림(시트·초상화)만 넣는다. 비율이 다른 그림을 여러 장 넣으면 **마지막 그림의 비율**을 따르므로, 비율은 프롬프트 끝의 `Aspect ratio …`로 박는다(시트·장면 모두 글로 적은 비율로 나왔다). 구도 그림(`layout_*.png`)은 넣지 않는다(위 0-2 끝의 참고 그림, 0128). AI Studio·API처럼 비율을 고를 수 있으면 직접 고른다. [1차 + 직접 본 것]
4. **해상도는 고를 수 있으면 가장 크게**(2K·4K, 지금 원본은 모두 1024px 폭). 도트처럼 그려도 진짜 픽셀 격자가 아니다(한 점이 3~5px로 들쭉날쭉, 주기가 없다). 격자를 찾지 않고 게임 칸마다 가장 흔한 색을 고르는데(0125), 원본이 클수록 칸마다 점이 많아 고르기가 안정된다. [직접 본 것. "격자를 맞추라"는 도구 회사 글(2차)은 우리 그림에 맞지 않았다]
5. 프롬프트 = **[공통 스타일] + [세계 규칙] + [캐릭터] + [장면]**. 이어 붙여 영어로 넣는다(한국어도 지원 언어지만 이 문서의 글은 영어로 맞췄다). 낱말 나열보다 **장면을 문장으로 그리는 것**이 기본 원칙이다. [1차]
6. **장면마다 새 대화.** 고치다 생김새가 흐트러지면 새 대화에서 자세한 설명과 함께 다시 시작한다. 한 번에 다 맞추려 하지 말고 짧게 고쳐 달라고 한다. 예: "Keep everything the same, but make Naïl smaller and move him to the left third." [1차]
7. 나온 그림은 **원본 그대로** 넘겨 준다. 크기·팔레트는 `ai_illust.py`가 맞추고, 안전 영역은 프롬프트로 잡고 Claude가 확인한다(아래 7).

### 0-1. 먼저 캐릭터 시트(권장)
> **용어**: 이 문서에서 "캐릭터 시트(character sheet)"는 늘 이 0-1 단계의 결과, 곧 그 캐릭터의 앞모습·3/4 모습 그림이다. "초상화(portrait)"는 0-2의 가슴 위 그림이다.
장면을 그리기 전에 캐릭터마다 새 그림체로 **한 장씩** 먼저 그려 확정한다. 공식 문서는 같은 캐릭터를 여러 장 그릴 때 **앞서 만든 그림을 다음 프롬프트에 넣으라**고 한다. 확정한 시트를 장면의 캐릭터 그림으로 `ref_*.png` 대신(또는 함께) 넣는다. [1차]
```
Create a character reference sheet for a cozy pixel-art game: the character from the attached image, standing, shown from the front and from a three-quarter view side by side on a plain light background, full body, same proportions and colors as the reference. [캐릭터 글]. [공통 스타일]. A clean, wordless image with no title, labels or color swatches. Aspect ratio 16:9.
```
마음에 드는 그림체(예: 앞서 받은 초상화 판)가 있으면 Pro에서 **그림체 참고**로 첨부하고 이렇게 적는다: `Use the last attached image only as a style reference for line, palette and pixel density; take each character's design only from its own reference image.`

### 0-2. 대화 초상화(표정)
- **한 장에 표정 하나.** 한 장에 넷을 줄 세우게 했더니 셋이 겹치고 한 몸에 얼굴이 둘 붙었다(2026-10-26 시험). 원인: 정사각형 넷에는 4:1이 필요한데 16:9를 적었다. 그리고 모델은 한 장 안의 개수를 잘 지키지 못한다(공식 Limitations: 요청한 개수를 늘 지키지는 않는다).
- **보통 표정을 먼저 1:1로**, 그다음 같은 대화에서 **표정만 바꾸기**(공식 inpainting 템플릿 "change only … keep everything else exactly the same"). 크기·자리·몸이 그대로라 대사가 바뀔 때 얼굴이 튀지 않는다.
- 표정을 바꿀 때는 **늘 보통 표정 그림을 기준으로** 고친다. 고친 그림을 다시 고치면 흐트러진다. 흐트러지면 새 대화에 보통 표정 그림을 첨부하고 다시.
- 게임 초상화는 모두 오른쪽을 보는 가슴 위 모습, 88×88(`assets/art/portraits.png`). 배경은 한 가지 색으로 받아 Claude가 지운다.

1) 보통 표정(시트 첨부):
```
Using the attached character sheet of [이름], create one dialogue portrait for the same game: a single head-and-shoulders portrait of [이름], centered, facing slightly to the right, with [그 캐릭터의 상징: Naïl은 his big pink front paws with cream claws raised in front of his scarf] visible at the bottom, on a plain flat light background (#E8E0D0). [표정]. Keep the character's design, clothing, colors and pixel style exactly the same as the sheet. A clean, wordless image with only this one character. Aspect ratio 1:1.
```
2) 표정 바꾸기(같은 대화, 보통 표정 그림 기준). "provided image"라고만 쓰면 같은 대화의 마지막 그림을 고친다. 헷갈리지 않게 보통 표정 그림을 **첨부하고** "the attached portrait"로 가리킨다:
```
Using the attached portrait of [이름], change only the facial expression to [표정]. Keep everything else exactly the same: the same size, position, framing, clothing, colors, background and pixel style.
```
3) 다른 그림에서 모양을 빌려 고칠 때는 첨부 순서로 가리킨다. 비율은 마지막 첨부 그림을 따르니 **고칠 그림을 마지막에** 둔다:
```
Edit the last attached image, the portrait of [이름]. Add only [고칠 것], copying its shape from the first attached image, the character sheet. Keep everything else in the portrait exactly the same. The output is the edited portrait only.
```
- **고친 것은 다음 고침에서 되살아날 수 있다**(시험: 귀를 지운 뒤 같은 대화에서 앞발을 더하자 귀가 다시 났다). 대화 안의 앞선 그림(귀 있는 그림)을 모델이 계속 보고, "나머지는 그대로"는 무엇을 고쳤는지 모른다. 그래서 (1) 고친 그림을 저장해 **새 대화**에 그 그림만(필요하면 시트와 함께) 첨부하고, (2) 프롬프트에 **지킬 것을 짚어** 적는다: `Ensure these features of the portrait remain completely unchanged: [고친 점들]`(공식 합성 템플릿의 "Ensure that the features … remain completely unchanged").
- **다른 캐릭터 초상화를 크기 맞춤용으로 넣으면 생김새까지 따라간다**(시험: 귀 없는 Naïl 초상화를 함께 넣자 포포의 귀가 사라졌다). 그 캐릭터의 시트에서 귀·모자 같은 머리 모양을 다시 가져오게 고친다(첨부: 시트 → 고칠 초상화 순서).
- **걱정 표정이 거절될 때**("저는 언어 모델일 뿐이라서…"): 부드러운 말로 바꾸고 맨 앞에 `Edit this image.`를 붙인다(`gently concerned look`, 또 거절되면 `thoughtful, slightly unsure look`). Naïl·모모 둘 다 이렇게 됐다.
- **물든 모습은 돌아온 모습 그림을 고쳐 그린다**(틀이 1px까지 같다). 실은 "선만큼 굵게, 볼·이마·귀 둘레로", 눈은 "반딧불 같은 선명한 초록"으로 적는다. 가는 실은 88px에서 사라지고, "glowing green"만 쓰면 흰빛 민트가 나왔다(0112).
- 빨간 눈은 "deep, saturated red like embers"로 써도 검붉게 나왔다(들쥐 대장). 넣을 때 눈 상자 안의 빨강을 밝힌다(0114).
- 초상화에서 얼굴을 새로 해석할 수 있다(할매가 사람 할머니 얼굴로). 그 얼굴의 표정들이 줄인 칸에서 갈리지 않으면 "Her face is the same as on the sheet: …"로 시트 얼굴을 짚어 다시 그린다(0115).
- 표정이 여럿이면 다음 표정도 **보통 표정 하나만** 첨부한다(기쁨 표정을 고치면 감은 눈이 남기 쉽다).
- **시트에서 몸집이 빠지기 쉽다**(보리가 날씬한 쥐가 됐다): 통통함·머리 크기를 캐릭터 글에 적고, 틀리면 참고 그림 → 시트 순서로 첨부해 고친다. 시트 프롬프트에 글자 없음을 넣지 않으면 제목·색 견본이 들어간다(0113).
- 첫 시험(Naïl 보통 표정): 시트에는 없던 둥근 귀가 생겼다(두더지는 귀가 털 속에 숨는다 → 캐릭터 글에 `no visible ears` 넣기). 가슴 위로 자르니 상징인 분홍 앞발이 빠졌다 → 초상화 프롬프트에 앞발을 넣는다.

| 캐릭터 | 칸 | 표정(왼쪽이 보통) |
|---|---|---|
| Naïl | 0~3 | `calm and neutral` · `happy, eyes closed in a smile` · `worried, brows raised and a small, uneasy mouth` · `gentle, brave determination: brows only slightly lowered, eyes bright and steady, a small firm but soft mouth, never fierce` |
| 모모(돌아옴) | 12~14 | `calm, a little shy` · `happy, eyes closed in a smile` · `worried, ears drooping` |
| 모모(물듦) | 7~8 | `cold and distant, glowing green eyes` · `pained, glowing green eyes squeezed half shut` |
| 포포 | 5~6 | `calm and friendly` · `happy, eyes closed in a big smile` |
| 보리 | 10~11 | `calm, tired but kind` · `happy, eyes closed in a smile` |
| 누룩 할매 | 15~16 | `peaceful, eyes closed` · `happy, eyes closed in a warm smile` |
| 도토 | 31~32 | `calm` · `happy, eyes closed in a smile` |
| 수수 | 33~34 | `calm` · `happy` |
| 콩이 | 36~37 | `calm` · `happy, eyes closed in a smile` |
| 도토·콩이·들쥐 대장(물듦) | 30·35·9 | `glaring, small glowing eyes` — 도토는 amber, 콩이는 pale sky blue, 대장은 red(0052·0054) |
| 엄마 | 17 | `gentle, eyes closed` |

참고 그림(팩 그림 없음, 0064):
- **캐릭터는 확정한 시트**(`sheet_<이름>.png`)와 시트가 없는 모습의 초상화(`ref_mom`·`ref_momo_tainted`·`ref_bori_tainted`·`ref_doto_tainted`·`ref_kongi_tainted`)를 쓴다(0124). `python3 tools/art/ref_pack.py <폴더>`가 `tools/art/sources/ai/sheets/`와 초상화 원본에서 만든다: 시트의 글자 띠를 잘라 낸다(첨부에 글자가 있으면 장면에도 글자가 들어가기 쉽다). ✦ 로고는 원본에서 이미 지웠다(0133). `overview.png`는 확인용 한 장.
- **돌아오는 장면에 물든 모습 그림을 넣지 않는다**: `bori_freed`에 물든 보리를 둘째로 넣었더니 돌아온 보리의 털이 푸른 잿빛이 되고, 흉터가 빨간 X, 몸이 날씬해졌다(물든 모습의 색·생김새가 번졌다). 묶였던 모습은 글로만 쓰고, 틀리면 시트 → 고칠 그림 순서로 첨부해 "Redraw only <이름> to match the first attached image"로 고친다(0127). 물든 모습 초상화는 물든 모습이 주인공인 장면에만 쓴다.
- **패닝하는 그림은 마지막 화면의 대화창 자리를 원본 높이 비율로 적는다**: `last_bloom`(640×440, 아래로 80 패닝)은 마지막 화면에서 대화창이 원본 높이 75% 아래를 덮는다. "위쪽 3분의 2"만 쓰면 이웃들이 61~93%에 서서 가려졌다(0130).
- **반짝이는 "small round dots of light"로 적는다**: "sparkles"만 쓰면 ✦ 로고와 똑같은 네 꼭짓점 별이 그려져 로고처럼 보였다(`momo_bloom`, 0128).
- 그림체를 더 맞추고 싶으면 Pro에서 초상화 하나를 그림체 참고로 더 넣는다(0-1 끝의 문장).
- **구도 그림(`layout_*.png`)은 넣지 않는다**(0128). 예전 코드 임시 일러스트라, 넣으면 모델이 그 어색한 모양까지 따라 그렸다(`great_tree` 셋째 판: 좌우 대칭·사다리꼴 줄기·번개 같은 실, `momo_bloom` 첫 판: 달걀 같은 몸·곧은 두 귀). 자리는 장면 글에("얼굴은 위쪽 3분의 2"), 비율은 `Aspect ratio …`로 적는다. 나노바나나가 고를 수 있는 비율로 받고 `ai_illust.py`가 게임 비율로 자른다(2:1 장면 → 21:9, 640×440 → 3:2).
- 예전 `ref_*.png`(직접 그린 초상화를 4배로 키운 것)는 시트를 그릴 때만 썼다.

## 1. 공통 스타일 (모든 장면 앞에)
공식 원칙대로 목적(무엇에 쓰는 그림인지)을 먼저 말하고, "~하지 마"보다 원하는 모습을 적는다(semantic negative prompt). [1차]
```
Purpose: a full-screen story cutscene illustration for "Naïl: Under the Roots", a cozy top-down pixel-art action game about a baby mole who travels down through the roots of a dying great tree to calm a lonely old neighbor. The mood is tender and a little lonely, never scary; there are no villains.
Style: high-quality 2D pixel art like a cutscene from a 16-bit era console game. Chunky, clean pixel clusters, crisp hard pixel edges, and solid flat color bands with 3 to 5 shades per material. Characters are outlined in dark plum (#2B1D2A), about two pixels thick. Soft light comes from the upper left. A warm, muted, earthy limited palette of browns, moss greens, dusky lavender and soft gold, with saturated violet kept only for the threads. Cute, round animal characters with big gentle eyes.
Layout: a landscape game screen. Faces and important things sit in the upper two-thirds; the bottom third is calm, simple ground or shadow, because a dialogue box covers it. The picture is a clean, wordless illustration that fills the frame edge to edge.
```

## 2. 세계 규칙 (모든 장면 앞에, 공통 스타일 다음)
```
World rules: The violet threads belong to Halmae, an ancient and lonely fungus grandmother who is afraid of being left alone. They are thin, softly glowing violet strands that wrap around tree roots. Tree roots are always brown wood, never purple; the violet threads only coil around them. Animals bound by the threads have small glowing eyes: red for the field mice, green for Momo the rabbit, amber for Doto the squirrel, and pale sky blue for Kongi the armadillo. When an animal is freed, its eyes return to their own gentle dark color and the threads loosen, fall away, and fade to white or sparkle.
```

**실이 없는 장면**에서는 세계 규칙 바로 뒤에 이 문장을 붙인다: `There are no violet threads anywhere in this scene; all roots are clean, bare brown wood.` 세계 규칙의 "실은 뿌리를 감는다" 때문에 `mother_burrow` 첫 그림에서 집 굴 방의 뿌리마다 실이 거미줄처럼 감겼다(0126). 해당: `mother_burrow`, `molehill`, `sprout`, `last_bloom`, `doto_memory`(흰 실만, `cutscene-illustrations.md`에서 실이 나오지 않는 장면).

## 3. 캐릭터 (장면에 나오는 사람만 이어 붙인다)
| 이름 | 첨부 | 붙여 넣을 글 |
|---|---|---|
| Naïl | `sheet_nail.png` | `Naïl: a tiny baby mole with no visible ears (a mole's ears are hidden in its fur), a round, fluffy brown body, a big pink snout, large pink front paws with cream claws, small shiny lavender eyes with a white highlight, and a knitted red-orange scarf tied at the front. Match the attached character sheet of Naïl.` |
| 엄마 | `ref_mom.png` | `Naïl's mother: the same kind of mole but larger, with slightly darker brown fur, no scarf, and gentle closed eyes. Match the attached reference of the mother mole.` |
| 보리 | `sheet_bori.png` | `Bori: a plump, round old field-mouse elder with a big wide head and reddish-brown fur, big round ears with pink insides, a small scar on his forehead, a pink snout and a long pink tail; sturdy, tired and kind. Match the attached character sheet of Bori.` |
| 모모(돌아옴) | `sheet_momo.png` | `Momo: a young European wild rabbit (a burrow rabbit with medium-length ears; one ear tip folds over), warm grey-brown fur, a cream chest and muzzle, big glossy lavender eyes with dark pupils with small lashes, and a pink flower at the base of one ear. Match the attached character sheet of Momo.` |
| 모모(물듦) | `ref_momo_tainted.png` | `Momo while bound: the same rabbit with dusky plum-grey fur and glowing green eyes, violet threads coiled around her body. Match the attached reference.` |
| 누룩 할매 | `sheet_halmae.png` | `Halmae: an enormous, ancient fungus grandmother, a soft white cotton-like mycelium mound crowned with a cluster of tan mushroom caps, with a kind old face: closed eyes, rosy cheeks and a small smile. Her threads spread out in every direction, white with violet mixed in. Match the attached character sheet of Halmae.` |
| 도토 | `sheet_doto.png` | `Doto: a red squirrel with a huge fluffy tail and an acorn-cap hat, cheerful closed eyes. Match the attached character sheet of Doto.` |
| 수수 | `sheet_susu.png` | `Susu: a cute mole-cricket craftsman, a small digging insect drawn as a round, friendly animal: a big rounded, velvety brown thorax like a soft helmet, a small round head with two long thin antennae and big gentle eyes, a pair of large dark shovel-like front legs for digging, a short segmented abdomen, and brass goggles with round sky-blue lenses pushed up on his forehead. Match the attached character sheet of Susu.` |
| 콩이 | `sheet_kongi.png` | `Kongi: a small, chubby, cute armadillo with a soft pink face, belly and limbs, a domed banded grey shell on his back and a grey head plate, a long rounded pink snout, two tall pink ears, small gentle dark eyes and rosy cheeks. Match the attached character sheet of Kongi.` (확정한 시트가 귀가 길고 분홍이라 시트에 맞췄다. 예전 글 "짧고 뾰족한 귀"는 시트 전 것) |
| 포포 | `sheet_popo.png` | `Popo: a raccoon with grey fur, a black eye mask, a white muzzle and round ears, with a small trinket tucked behind one ear. Match the attached character sheet of Popo.` |

## 4. 장면
비율은 나노바나나에서 고를 비율, 괄호는 게임 그림 크기(Claude가 잘라 맞춘다).

### `great_tree` — 8:7 세로로 긴 그림(640×560, 위→아래로 내려가며 보인다. 비율을 골라야 하면 1:1)
첨부: 없음(비율만 글로)
```
Scene (a tall image that the game slowly pans from top to bottom), built in steps. First, in the upper half, a huge old tree with almost no leaves stands alone on a hill against a dusky evening sky with a low horizon; the whole tree is visible. Then, in the lower half, the view cuts below the ground into dark soil: thick brown roots spread out, and thin glowing violet threads coil around them and seep away into the darkness. Finally, deep in the dark, a few pairs of tiny red eyes glow. The upper half feels calm and quiet; the lower half carries a quiet unease. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 8:7 (if 8:7 is not available, 1:1).
```

### `mother_burrow` — 16:9 (640×360)
첨부: `sheet_nail.png`, `ref_mom.png`
```
Scene: Inside a snug underground burrow chamber with a nest of dry grass. A large mother mole, eyes closed and tender, gently nudges her tiny baby Naïl with her nose toward a tunnel that leads out of the chamber; warm daylight glows at the far end of the tunnel. Naïl is small and stands facing the light. The mood is warm and a little lonely: a parent letting a child go. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `bori_freed` — 16:9 (640×360)
첨부: `sheet_bori.png`
```
Scene: An open clearing at dusk. The old field-mouse elder Bori sits with his eyes closed, catching his breath. Violet threads are loosening from his body and drifting upward, breaking into small sparkles as they go; none of them cover his face. His scar and big round ears are clearly visible. The feeling is relief. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `momo_bloom` — 16:9 (640×360)
첨부: `sheet_momo.png`, 모모 보통 표정 초상화 원본(`tools/art/sources/ai/momo_normal.png`)
```
Scene: Momo from the chest up, like the second attached image (her portrait), placed a little left of center so that her face is in the upper two-thirds of the frame. Behind her, a soft warm evening sky with gentle golden light and a few pale clouds, and the tops of distant meadow grasses. The flower at the base of her ear is bursting into full pink bloom, with a few petals floating. The last violet threads are slipping off her shoulders and falling away below, breaking into small white sparkles. Her eyes are a little teary. She looks shy and relieved. The bottom third of the frame is her soft chest and the falling sparkles, calm and simple, because a dialogue box covers it. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `halmae_deep` — 약 2:1 (704×360, 오른쪽으로 조금 흐른다. 비율을 골라야 하면 21:9)
첨부: `sheet_halmae.png`, `sheet_nail.png`
```
Scene (a wide image that the game slowly pans to the right), built in steps. First, the deepest place under the roots, dark and soft. Then, an enormous ancient fungus grandmother, Halmae, fills the middle and right: a vast soft mound of white mycelium crowned with clusters of mushroom caps, threads reaching out in every direction, white with violet mixed in, giving off a faint gentle glow. In the middle of the mound is a kind old face with closed eyes and a small smile. Finally, in the front left, tiny Naïl looks up at her. The feeling is awe and compassion, not fear. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 21:9.
```

### `last_bloom` — 약 3:2 (640×440, 위→아래로 조금 내려간다) · 인물 넷이라 Pro 권장
첨부: `sheet_nail.png`, `sheet_popo.png`, `sheet_bori.png`, `sheet_momo.png`
```
Scene (an image the game pans slightly downward), built in steps. First, dawn: the great old tree blooms one last time, its crown a cloud of pale pink blossoms against a soft gold and lavender sky. Then, at the foot of the tree, small in the frame, the neighbors gather and look up: the baby mole Naïl, the raccoon Popo, the old field mouse Bori and the rabbit Momo. Keep the gathered neighbors within the upper two-thirds of the image. A beautiful farewell. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 3:2.
```

### `sprout` — 16:9 (640×360)
첨부: 없음(비율만 글로)
```
Scene: Underground, in dark soft soil. A seed has cracked open and a tiny sprout with two seed leaves is pushing upward. Soft white threads curl around it gently, like cupped hands protecting it. Faint light falls from above. A quiet new beginning. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `molehill` — 16:9 (640×360)
첨부: `ref_mom.png`
```
Scene: A morning meadow. The sprout has grown into a young tree. Beneath it, a mound village with little round doors and windows. In the foreground, a fresh new molehill of turned earth. Far away on the left, the mother mole peeks out of the ground with only her head showing, watching, small in the distance. Quiet joy. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `doto_memory` — 16:9 (640×360)
첨부: `sheet_doto.png`
```
Scene (a memory from long, long ago, so the whole image is faded toward warm sepia and the edges darken like an old photograph): The deepest place under the roots. A young Doto, a red squirrel wearing an acorn-cap hat with his big tail raised, eyes closed and content, buries a softly glowing seed where a white thread touches the ground. From the darkness on the right, white threads flow in. The feeling is nostalgia, the warmth of something forgotten. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `momo_deep` — 약 2:1 (704×360, 왼쪽→오른쪽으로 흐른다. 비율을 골라야 하면 21:9)
첨부: `sheet_momo.png`, `sheet_halmae.png`
```
Scene (a wide image that the game slowly pans from left to right): The deepest place, reached as Momo. In the darkness, Halmae's threads hang everywhere, white with some violet, and caught in them are tufts of rabbit fur and withered flower petals, traces of Momo's lost family. Small Momo stands on the right side and looks up at them. Her eyes are her own gentle lavender eyes. Longing and a quiet realization. Keep Momo's whole body above the bottom quarter of the image; the bottom quarter is plain dark ground, because a text box covers it. Keep important things away from the far left and right edges. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 21:9.
```

### `kongi_door` — 16:9 (640×360)
첨부: `sheet_kongi.png`. 공통 스타일·세계 규칙·콩이 글을 앞에 붙인다(첫 판은 스타일 없이 그려 도트가 아닌 그림이 나왔다).
```
Scene: The entrance to the deepest place: a huge stone door in the darkness. The gap in the middle has just begun to open, and pale golden light leaks out; the light lights a haze of dust and spills across the floor. The brown roots that bound the door now hang slack and loose, with a few violet threads sliding off them. In front of the door, a little to the left of the light, Kongi has just uncurled from a ball and lifts his head toward the gap, eyes softly closed. The warm light from the gap rims his shell and ears, and his pink face and belly are still clearly visible in the soft glow; he is not a dark silhouette. Kongi is clearly visible, about a third of the image height, and his feet stand above the bottom quarter of the image; the bottom quarter is plain dark floor, because a text box covers it. Letting go, relief. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

### `title` (타이틀 바탕) — 16:9 (640×360)
첨부: `sheet_nail.png`. 공통 스타일의 Layout 줄은 타이틀용으로 바꾼다(위에 제목, 가운데 메뉴, 0137).
```
Scene (a game title screen background; the game draws the title text and menu on top): A cross-section of the earth, built in steps. First, the top third is a meadow at sunset with a distant lonely hill and two small village mounds; on the right, the trunk of a great tree rises out of frame. Then, below the ground line, dark layered soil: the great tree's brown roots spread down and to the left, and violet threads coil around the tips of the thickest roots. Finally, in the lower left, a tunnel leads down to a small burrow chamber where baby Naïl sleeps curled beside a jar of fireflies. Keep the top band of sky calm and simple for the title text, and keep the central area of the soil dark and even for the menu; put the interesting details (the tree, the burrow chamber) at the sides. A clean, wordless illustration that fills the frame edge to edge. Aspect ratio 16:9.
```

## 5. 자주 나오는 고침 말
- 오른쪽 아래 ✦ 표시(Gemini 앱 로고)는 원본을 `tools/art/sources/ai/`에 넣고 `python3 tools/art/strip_logo.py`로 원본에서 한 번 지운다(0133). 나노바나나로 고친 그림에는 앞 그림의 ✦를 모델이 다시 그린 흐린 보랏빛 별이 겹쳐 있어, 스크립트가 되돌린 뒤 그 자리를 메운다. 보이지 않는 SynthID 워터마크는 모든 생성 그림에 들어간다. [1차]
- 선이 검다: `Change all outlines around the characters to dark plum (#2B1D2A), about two pixels thick.`
- 너무 매끈하다(그라디언트): `Re-render as true pixel art: flat bands of color, 3 to 5 shades per material, crisp edges.`
- 뿌리가 보라다: `The roots must be brown wood. Only the thin threads wrapped around them are violet.`
- 눈 색이 틀리다: `Freed animals have their own dark eyes; only animals still bound have glowing eyes (field mice red, Momo green, Doto amber, Kongi pale sky blue).`
- 얼굴이 아래에 있다: `Move the characters up so their faces are in the upper two-thirds; keep the bottom third simple.`
- 글자가 들어갔다: `Keep everything the same, but make it a clean, wordless illustration: remove the lettering and fill those spots with the surrounding scenery.`

## 6. 하지 않는 것
- 팩(Sunnyside 등) 그림·스크린숏을 첨부하지 않는다. 게임 화면 캡처에도 팩 그림이 섞여 있으니 넣지 않는다(0064).
- 다른 작가의 그림·화풍 이름("○○ 스타일")을 넣지 않는다.

## 7. 넣고 나서(Claude)
1. 원본을 `tools/art/sources/ai/illust/<장면>.png`(초상화는 `sources/ai/`)에 넣고 `python3 tools/art/strip_logo.py <파일>`로 ✦를 원본에서 지운 뒤(0133) `python3 tools/art/illust.py assets/art/illust <장면>`으로 만든다. `ai_illust.py`가 장면의 비율로 자르고, 게임 칸(그림 크기의 절반)마다 가장 흔한 색을 골라 2배로 키운다(격자는 찾지 않는다, 0125). 작은 캐릭터는 `FOCUS`에 자리를 적는다(0135·0137). 프롬프트로 격자를 맞추려 하지 않는다.
2. 팔레트는 `ai_illust.py`가 줄인다(채도 높은 색·나머지·FOCUS, 디더 없음). 2×2 점이라 1px 체크 무늬는 생기지 않는다.
3. 안전 영역(대화창 y 248 아래, 내레이션 y 264 아래)·색 약속(갈색 뿌리·보라 실·눈 색)을 다시 본다.
4. `assets/LICENSES.md`에 출처(모델·날짜·만든 사람)를 적고, 개발 → "이야기 일러스트"로 배율 2·3에서 본다.
5. 초상화(`tools/art/ai_portraits.py`): 선이 검게 나오면 프롬프트로 다시 그리지 말고 자두빛으로 옮긴다(모모, 0110). 배경을 지운 미리보기는 **실제 바탕(어두운 대화창)**에서 본다. 밝은 바탕에서는 배경색과 비슷한 크림색 가슴이 지워진 것이 안 보였다(0110).

## 출처
- [1차] Google, Gemini API "Nano Banana image generation"(모델 고르기, 참고 그림 한도, 비율·해상도, 프롬프트 템플릿, Best practices, Limitations, SynthID): https://ai.google.dev/gemini-api/docs/image-generation (2026-10-26 읽음)
- [1차] Google Developers Blog, "How to prompt Gemini 2.5 Flash Image Generation for the best results"(2025-08-28: 장면을 문장으로, 마지막 첨부 그림의 비율, 생김새가 흐트러지면 새 대화, semantic negative prompt): https://developers.googleblog.com/en/how-to-prompt-gemini-2-5-flash-image-generation-for-the-best-results/
- [2차] SpriteCook, "Turning Nano Banana 2 AI Images into Actual Pixel Art"(2026-03-19: 격자 문제와 뒤에서 맞추기, 가장 큰 해상도, limited palette): https://www.spritecook.ai/blog/nanobanana-pixel-art-for-games — 도구 회사 글이라 자기 도구 홍보가 섞여 있다.
- 첫 시험(2026-10-26, 사장): 참고 그림 없이 홈페이지만 주고 초상화를 그리게 했더니 이름·종족 글로 흔한 그림을 채웠다(누룩 할매 → 빨간 독버섯 할아버지, 땅강아지 → 사슴벌레, 모모 → 흰 토끼, 게임에 없는 이웃 넷). 참고 그림 + 생김새 글이 필요한 근거.
