# 컷씬 일러스트 나노바나나 프롬프트

> 컷씬 일러스트 11장 + 타이틀 바탕을 Google 나노바나나(Gemini 이미지 모델)로 그릴 때 쓰는 프롬프트다. 장면 사양은 [`cutscene-illustrations.md`](cutscene-illustrations.md)를 따른다. AI 그림을 쓰기로 한 결정과 지킬 것은 `docs/decisions/0107`.

## 0. 쓰는 법
1. **장면마다 새 대화**를 연다. 한 대화에서 여러 장면을 이어 그리면 앞 그림이 섞인다. 고치다 캐릭터가 흐트러지면 새 대화에서 다시 시작한다(2차 출처 팁).
2. **첨부 순서**: 캐릭터 참고 그림(`ref_*.png`)을 먼저, 구도 참고(`layout_*.png`)를 **맨 마지막**에. 여러 장을 넣으면 마지막 그림의 가로세로 비를 따라간다는 2차 출처 팁이 있다. AI Studio처럼 비율을 고를 수 있으면 아래 표의 비율을 직접 고른다.
3. 프롬프트 = **[공통 스타일] + [세계 규칙] + [장면]**. 세 덩어리를 그대로 이어 붙여 넣는다. 영어로 넣는다.
4. 한 번에 다 맞추려 하지 말고, 나온 그림에 짧게 고쳐 달라고 한다. 예: "Make Naïl smaller and move him to the left third. Keep everything else the same."
5. 나온 그림은 **크게(원본 그대로) 저장해서** 넘겨 준다. 크기·픽셀 격자·팔레트·안전 영역은 Claude가 맞춘다(아래 7).

참고 그림(직접 그린 것만, 팩 그림 없음. 0064): 세션 스크래치에 만들어 둔 `ref_*.png`(초상화를 4배로 키운 것)와 `layout_*.png`(지금 임시 일러스트 2배)를 받아서 쓴다. 다시 만들 때는 `assets/art/portraits.png`(칸 번호는 `src/cutscene.rs` `Portrait`)와 `assets/art/illust/*.png`에서 자른다.

## 1. 공통 스타일 (모든 장면 앞에)
```
Style: a cozy, gently melancholic storybook cutscene illustration in high-quality 2D pixel art, like a cutscene from a 16-bit era console game. Chunky, clean pixel clusters on a regular pixel grid, flat color bands with 3 to 5 shades per material, crisp hard edges. Shading is done with solid bands of color rather than soft airbrush gradients, anti-aliasing or checkerboard dithering. Characters have dark plum outlines (#2B1D2A) about two pixels thick instead of black outlines. Soft light comes from the upper left. Warm, muted, earthy palette: browns, moss greens, dusky lavender and soft gold; saturated violet appears only in the threads. Cute, round animal characters with big gentle eyes; the mood is tender and a little lonely, never scary. Composition for a 16:9 game screen: faces and important elements sit in the upper two-thirds; the bottom third is calm ground or shadow because a dialogue box will cover it. The image shows only the scene, with no text, letters, logo, UI, frame or border.
```

## 2. 세계 규칙 (모든 장면 앞에, 공통 스타일 다음)
```
World rules: The violet threads belong to Halmae, an ancient and lonely fungus grandmother who is afraid of being left alone. They are thin, softly glowing violet strands that wrap around tree roots. Tree roots are always brown wood, never purple; the violet threads only coil around them. Animals bound by the threads have small glowing red eyes, except Momo the rabbit, whose bound eyes glow green. When an animal is freed, its eyes return to their own gentle dark color and the threads loosen, fall away, and fade to white or sparkle.
```

## 3. 캐릭터 (장면에 나오는 사람만 이어 붙인다)
| 이름 | 첨부 | 붙여 넣을 글 |
|---|---|---|
| Naïl | `ref_nail.png` | `Naïl: a tiny baby mole with a round, fluffy brown body, a big pink snout, large pink front paws with cream claws, small shiny dark eyes with a white highlight, and a knitted red-orange scarf tied at the front. Match the attached reference of Naïl.` |
| 엄마 | `ref_mom.png` | `Naïl's mother: the same kind of mole but larger, with slightly darker brown fur, no scarf, and gentle closed eyes. Match the attached reference of the mother mole.` |
| 보리 | `ref_bori.png` | `Bori: an old field-mouse elder with reddish-brown fur, big round ears with pink insides, a small scar on his forehead, a pink snout and a long pink tail; sturdy, tired and kind. Match the attached reference of Bori.` |
| 모모(돌아옴) | `ref_momo_freed.png` | `Momo: a young European wild rabbit (a burrow rabbit with medium-length ears; one ear tip folds over), warm grey-brown fur, a cream chest and muzzle, big glossy dark eyes with small lashes, and a pink flower at the base of one ear. Match the attached reference of Momo.` |
| 모모(물듦) | `ref_momo_tainted.png` | `Momo while bound: the same rabbit with dusky plum-grey fur and glowing green eyes, violet threads coiled around her body. Match the attached reference.` |
| 누룩 할매 | `ref_halmae.png` | `Halmae: an enormous, ancient fungus grandmother, a soft white cotton-like mycelium mound crowned with a cluster of tan mushroom caps, with a kind old face: closed eyes, rosy cheeks and a small smile. Her threads spread out in every direction, white with violet mixed in. Match the attached reference of Halmae.` |
| 도토 | `ref_doto.png` | `Doto: a red squirrel with a huge fluffy tail and an acorn-cap hat, cheerful closed eyes. Match the attached reference of Doto.` |
| 콩이 | `ref_kongi.png` | `Kongi: a small armadillo with a banded grey shell and long pink ears. Match the attached reference of Kongi.` |
| 포포 | `ref_popo.png` | `Popo: a raccoon with grey fur, a black eye mask, a white muzzle and round ears, with a small trinket tucked behind one ear. Match the attached reference of Popo.` |

## 4. 장면
비율은 나노바나나에서 고를 비율, 괄호는 게임 그림 크기(Claude가 잘라 맞춘다).

### `great_tree` — 1:1 (640×560, 위→아래로 내려가며 보인다)
첨부: `layout_great_tree.png`
```
Scene (a tall image that the game slowly pans from top to bottom): In the upper half, a huge old tree with almost no leaves stands alone on a hill against a dusky evening sky with a low horizon; the whole tree is visible. In the lower half, the view cuts below the ground into dark soil: thick brown roots spread out, and thin glowing violet threads coil around them and seep away into the darkness. Deep in the dark, a few pairs of tiny red eyes glow. The upper half feels calm and quiet; the lower half carries a quiet unease. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `mother_burrow` — 16:9 (640×360)
첨부: `ref_nail.png`, `ref_mom.png`, `layout_mother_burrow.png`
```
Scene: Inside a snug underground burrow chamber with a nest of dry grass. A large mother mole, eyes closed and tender, gently nudges her tiny baby Naïl with her nose toward a tunnel that leads out of the chamber; warm daylight glows at the far end of the tunnel. Naïl is small and stands facing the light. The mood is warm and a little lonely: a parent letting a child go. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `bori_freed` — 16:9 (640×360)
첨부: `ref_bori.png`, `layout_bori_freed.png`
```
Scene: An open clearing at dusk. The old field-mouse elder Bori sits with his eyes closed, catching his breath. Violet threads are loosening from his body and drifting upward, breaking into small sparkles as they go; none of them cover his face. His scar and big round ears are clearly visible. The feeling is relief. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `momo_bloom` — 16:9 (640×360)
첨부: `ref_momo_freed.png`, `ref_momo_tainted.png`, `layout_momo_bloom.png`
```
Scene: A close portrait of Momo the rabbit from the chest up, against a soft warm sky. The withered flower at the base of her ear is bursting into full pink bloom. The last violet threads are slipping off her and falling away below. Her eyes have just turned from glowing green back to her own glossy dark eyes, a little teary. She looks shy and relieved. Use the first attached image for her freed look and the second only to understand what she looked like while bound. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `halmae_deep` — 21:9 (704×360, 오른쪽으로 조금 흐른다)
첨부: `ref_halmae.png`, `ref_nail.png`, `layout_halmae_deep.png`
```
Scene (a wide image that the game slowly pans to the right): The deepest place under the roots. An enormous ancient fungus grandmother, Halmae, fills the middle and right: a vast soft mound of white mycelium crowned with clusters of mushroom caps, threads reaching out in every direction, white with violet mixed in, giving off a faint gentle glow. In the middle of the mound is a kind old face with closed eyes and a small smile. In the front left, tiny Naïl looks up at her. The feeling is awe and compassion, not fear. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `last_bloom` — 3:2 (640×440, 위→아래로 조금 내려간다)
첨부: `ref_nail.png`, `ref_popo.png`, `ref_bori.png`, `ref_momo_freed.png`, `layout_last_bloom.png`
```
Scene (an image the game pans slightly downward): Dawn. The great old tree blooms one last time, its crown a cloud of pale pink blossoms against a soft gold and lavender sky. At the foot of the tree, small in the frame, the neighbors gather and look up: the baby mole Naïl, the raccoon Popo, the old field mouse Bori and the rabbit Momo. Keep the gathered neighbors within the upper two-thirds of the image. A beautiful farewell. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `sprout` — 16:9 (640×360)
첨부: `layout_sprout.png`
```
Scene: Underground, in dark soft soil. A seed has cracked open and a tiny sprout with two seed leaves is pushing upward. Soft white threads curl around it gently, like cupped hands protecting it. Faint light falls from above. A quiet new beginning. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `molehill` — 16:9 (640×360)
첨부: `ref_mom.png`, `layout_molehill.png`
```
Scene: A morning meadow. The sprout has grown into a young tree. Beneath it, a mound village with little round doors and windows. In the foreground, a fresh new molehill of turned earth. Far away on the left, the mother mole peeks out of the ground with only her head showing, watching, small in the distance. Quiet joy. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `doto_memory` — 16:9 (640×360)
첨부: `ref_doto.png`, `layout_doto_memory.png`
```
Scene (a memory from long, long ago, so the whole image is faded toward warm sepia and the edges darken like an old photograph): The deepest place under the roots. A young Doto, a red squirrel wearing an acorn-cap hat with his big tail raised, eyes closed and content, buries a softly glowing seed where a white thread touches the ground. From the darkness on the right, white threads flow in. The feeling is nostalgia, the warmth of something forgotten. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `momo_deep` — 21:9 (704×360, 왼쪽→오른쪽으로 흐른다)
첨부: `ref_momo_freed.png`, `ref_halmae.png`, `layout_momo_deep.png`
```
Scene (a wide image that the game slowly pans from left to right): The deepest place, reached as Momo. In the darkness, Halmae's threads hang everywhere, white with some violet, and caught in them are tufts of rabbit fur and withered flower petals, traces of Momo's lost family. Small Momo stands below and looks up at them. Her eyes are her own glossy dark eyes. Longing and a quiet realization. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `kongi_door` — 16:9 (640×360)
첨부: `ref_kongi.png`, `layout_kongi_door.png`
```
Scene: The entrance to the deepest place: a huge stone door in the darkness. The gap in the middle has just begun to open, and pale golden light leaks out; the light beam lights a haze of dust and spills across the floor. The brown roots that bound the door now hang slack and loose, with violet threads sliding off them. In front of the door, Kongi the armadillo uncurls from a ball and lifts his head toward the gap; he is a silhouette against the light, with only the rim of his shell lit. Letting go, relief. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

### `title` (타이틀 바탕) — 16:9 (640×360)
첨부: `ref_nail.png`, `layout_title.png`
```
Scene (a game title screen background; the game draws the title text and menu on top): A cross-section of the earth. The top third is a meadow at sunset with a distant lonely hill and two small village mounds; on the right, the trunk of a great tree rises out of frame. Below the ground line, dark layered soil: the great tree's brown roots spread down and to the left, and violet threads coil around the tips of the thickest roots. In the lower left, a tunnel leads down to a small burrow chamber where baby Naïl sleeps curled beside a jar of fireflies. Keep the top band of sky calm and simple for the title text, and keep the central area of the soil dark and even for the menu; put the interesting details (the tree, the burrow chamber) at the sides. Use the last attached image only as a rough guide for where things are placed; redraw everything with much more detail in the style above.
```

## 5. 자주 나오는 고침 말
- 선이 검다: `Change all outlines around the characters to dark plum (#2B1D2A), about two pixels thick.`
- 너무 매끈하다(그라디언트): `Re-render as true pixel art: flat bands of color, 3 to 5 shades per material, crisp edges.`
- 뿌리가 보라다: `The roots must be brown wood. Only the thin threads wrapped around them are violet.`
- 눈 색이 틀리다: `Freed animals have their own dark eyes; only animals still bound have glowing red eyes (Momo's glow green).`
- 얼굴이 아래에 있다: `Move the characters up so their faces are in the upper two-thirds; keep the bottom third simple.`
- 글자가 들어갔다: `Remove all text and letters from the image.`

## 6. 하지 않는 것
- 팩(Sunnyside 등) 그림·스크린숏을 첨부하지 않는다. 게임 화면 캡처에도 팩 그림이 섞여 있으니 넣지 않는다(0064).
- 다른 작가의 그림·화풍 이름("○○ 스타일")을 넣지 않는다.

## 7. 넣고 나서(Claude)
1. 장면의 비율로 자르고, 게임 크기(640×360 등)로 줄여 픽셀 격자를 다시 맞춘다. 나노바나나는 진짜 격자에 맞추지 않는다(한 "픽셀"이 6px·7px로 들쭉날쭉, 2차 출처).
2. 팔레트를 줄이고(디더 없이), 1px 체크 무늬가 생기면 단으로 정리한다(사양 2).
3. 안전 영역(아래 112px)·색 약속(갈색 뿌리·보라 실·눈 색)을 다시 본다.
4. `assets/LICENSES.md`에 출처(모델·날짜·만든 사람)를 적고, 개발 → "이야기 일러스트"로 배율 2·3에서 본다.

## 출처(2차, 이 환경에서 Google 공식 문서에는 접속하지 못했다)
- 서술형 문장 > 낱말 나열, 작은 수정 반복, 새 대화로 다시 시작, 마지막 첨부 그림의 비율: https://deepakness.com/raw/nano-banana-prompt-best-practices, https://www.pixelbin.io/blog/nano-banana-prompt-guide, https://mindpal.space/blog/how-to-prompt-gemini-2-5-flash-image-generation-for-the-best-results-a1b2c3
- 지원 비율(1:1~21:9): https://glama.ai/mcp/servers/@brunoqgalvao/gemini-image-mcp-server/blob/67301e5f4841e41af909b23290f8b00139fa732e/README.md
- 도트 그림: 팔레트를 정하면 덩어리가 깔끔해지고, 격자는 나중에 맞춘다: https://www.spritecook.ai/blog/nanobanana-pixel-art-for-games
