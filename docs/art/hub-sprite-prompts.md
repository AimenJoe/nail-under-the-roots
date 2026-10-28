# 마을·쉼터 새 그림 프롬프트(0224)

시트마다 **새 대화**에서 그림체 판 `tools/art/sources/ai/sprites/ref/ref_hub_style.png` 한 장만 붙이고 아래 프롬프트를 넣는다
(`python3 tools/art/hub_ref.py`가 만든다. 도형 판 `ref_hub_<a|b|c>.png`는 우리가 크기를 볼 때만 — 붙이면 베낀다).
시트 B·C도 A처럼 고쳐 쓴다(A 결과를 보고).
받은 원본은 `tools/art/sources/ai/sprites/hub_<a|b|c>_v1.png`로(✦는 `strip_logo.py`로 지운다).

배운 것(소품 0222·잡몹 0221): 크기가 가장 중요하다고 먼저 말하고, 참고 그림의 색·자리를 따르게 하고, 작은 표지는 굵게.

## 시트 A — 쉼터·마을 공용 물건 여섯 (두 장, 셋씩)

**첫 시도(2026-10-28)는 도형 판 `ref_hub_a.png`를 붙였더니 도형을 그대로 베껴 납작한 색면 + 선만 왔다.** 이제 붙이는 것은
그림체 판 `ref_hub_style.png`(게임에 이미 들어간 우리 새 그림) 한 장뿐, 크기는 글로. 한 장에 셋만 — 물건마다 결을 더 그리게.
받은 원본은 `hub_a1_v1.png`·`hub_a2_v1.png`.

### A1 — 가게 좌판 · 모닥불 화덕 · 뿌리 길 입구

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: visible wood grain, nails, cloth folds, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
1. A raccoon's tiny market stall, about twice as tall as the mole and as wide: a red-and-cream striped cloth awning on
   two wooden posts, a wooden counter with a little basket of red berries and a small tied seed sack.
2. A campfire ring, about one and a half moles wide and low: an oval of rounded grey stones with two crossed logs
   in the middle, NO fire (the game adds the flames).
3. A burrow entrance in the ground, about two and a half moles wide: a dark round hole with a lip of packed earth,
   gnarled roots curling around the rim, and a small wooden arrow signpost on the left.
```

### A2 — 그루터기 · 바위 · 통나무 자리

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: bark texture, cracks, moss, small highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows.
1. An old tree stump about as big as the mole: rings on the cut top, rough bark, three small roots gripping the ground.
2. A mossy boulder a bit bigger than the mole: rounded grey stone with cracks and a patch of green moss on top.
3. A short log lying on its side as a seat, a little wider than the mole: bark on the side, rings on the cut end.
```

### A3 — 뿌리 길 입구 다시(첫 A1에서 흙 바닥 위에 그려 옴)

```
Draw one new prop in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left).
The attached sprites are only a STYLE reference: do not redraw them.

A burrow entrance, about two and a half moles wide: a dark round hole with a raised lip of packed earth around it,
gnarled roots curling over the rim, and a small wooden arrow signpost on the left.
IMPORTANT: draw ONLY the object on a solid magenta background (#FF00FF). Do NOT draw any ground, grass or soil tile
around it; the magenta must touch the outer dark outline of the earth lip all around. No text, no shadows.
```

## 시트 B — 마을 꾸미기 여섯

```
Redraw the six decorations in the attached image (to the right of the small mole) as a pixel art sprite sheet for
a cute top-down action game, a cozy village of forest animals. SIZE IS THE MOST IMPORTANT RULE: draw each object
exactly as big as it is in the attached image, next to the small mole, with the same pixel block size.

3 columns x 2 rows, 6 objects, each well separated, solid magenta background (#FF00FF), no text, no grid lines,
no shadows, no glow effects. Top-down 3/4 view, light from the top-left, dark plum 1px outline. Keep details bold.
1. a small flowerbed: a soil mound with two round green bushes and three big flowers (red, yellow, white)
2. a firefly lantern: a glass lantern with tiny glowing bugs inside, hanging from the top of a wooden post
3. three glowing blue mushrooms of different heights with pale stems
4. a rustic bench made from a split log on two short legs
5. a wind chime: a wooden post with a cross bar and three hanging metal tubes
6. a string of small triangle flags (red, yellow, blue) between two wooden posts
```

## 시트 C — 마을 꾸미기 다섯

```
Redraw the five objects in the attached image (to the right of the small mole) as a pixel art sprite sheet for
a cute top-down action game, a cozy village of forest animals. SIZE IS THE MOST IMPORTANT RULE: draw each object
exactly as big as it is in the attached image, next to the small mole, with the same pixel block size.

3 columns x 2 rows (last cell empty), 5 objects, each well separated, solid magenta background (#FF00FF), no text,
no grid lines, no shadows, no glow effects. Top-down 3/4 view, light from the top-left, dark plum 1px outline.
1. a glass jar with a cork lid and four little yellow fireflies inside
2. a clay flower pot with three wildflowers (red, yellow, purple)
3. a swing: a wooden frame with two white silk threads holding a small wooden seat
4. a small shallow puddle of blue water with a light reflection line
5. a white silk thread loop tied to the top of a short wooden stick stuck in the ground
```
