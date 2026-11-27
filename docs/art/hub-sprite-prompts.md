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
> 들어감(2026-10-28, v0.5.1): 시안 다섯 가운데 팻말 + 첫 구멍 → `hub.png` 칸 5(`HUB_GATE`). A 여섯 끝, 남은 것은 B·C·D.

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

## 시트 E — 이웃의 흔적·이웃 등불·코드로 그린 꾸미기 (사장 판정 2026-11-27)

사장: "상의 없는 그림들", "갈대 피리·이웃 곁의 등불·샘 그릇 너무 성의 없음", 등불은 "그림 자체도 성의 없고 어떤 의미인지도 모르겠다".
이웃의 부탁 흔적 일곱(0273)과 이웃 등불(0304)은 0224 뒤에 코드로 그려 넣은 것이고, 꾸미기 가운데 바람종·깃발 줄·실 그네와 실 고리는
0224의 시트 B·C를 아직 받지 않아 코드 그림으로 남아 있다(나머지 꾸미기는 팩 그림). 위 시트 B·C 대신 **이 넷을 받는다** — A에서 배운 대로
그림체 판 `ref_hub_style.png` 한 장만 붙이고, 한 장에 셋, 크기는 글로.
받은 원본은 `tools/art/sources/ai/sprites/hub_e<1~4>_v1.png`로(✦는 `strip_logo.py`), `hub_ai.py`의 `ITEMS`에 줄을 더해 `hub.png` 칸으로.
등불 유리 속 빛·버섯 등의 빛·피리의 음표는 지금처럼 코드가 얹는다(그림에는 넣지 않는다).

### E1 — 이웃 등불 · 샘 그릇 · 갈대 피리

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: wood grain, glass shine, wet stone, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. A small firefly lantern on a wooden post, about as tall as the mole and one third as wide: a round glass lantern with a tiny wooden roof, hanging from a curved hook at the top of a thin post. The glass is EMPTY and dark teal (the game adds the light).
2. A small stone wash bowl, about half a mole wide and low: a rough round grey stone bowl filled with clear blue spring water, one bright highlight on the water, two wet pebbles beside it.
3. A reed flute leaning on a short log, about as wide as the mole and half as tall: a short mossy log lying on its side, and a pale yellow reed flute with four finger holes and a red cord tied at one end leaning against it.
```

### E2 — 큰 깍정이 · 풀 방석 · 버섯 등

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: scaly acorn texture, braided straw, soft mushroom gills, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. A big old acorn cap lying upside-down like a bowl, about half a mole wide: brown and scaly outside, smooth and pale inside, with a short stem.
2. A round woven grass cushion, about two thirds of a mole wide and flat: dry yellow straw braided in a spiral, a few loose blades sticking out.
3. A mushroom lamp, about half a mole wide and as tall as half a mole: a pale blue mushroom cap on a short white stem, set on a small flat grey stone. No glow (the game adds the light).
```

### E3 — 솜털 잠자리 · 모모의 꽃밭 · 바람종

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: fluffy seeds, tilled soil, petals, metal shine, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. A little bed of dandelion fluff, about as wide as the mole and low: a ring of small twigs filled with soft white fluffy seeds.
2. A short row of four flowers planted in a strip of dark tilled soil, about as wide as the mole: pink and white flowers alternating, green stems and leaves.
3. A wind chime, about as tall as the mole and half as wide: a thin wooden post with a short cross bar and three hanging metal tubes of different lengths.
```

### E4 — 깃발 줄 · 실 그네 · 실 고리

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: wood grain, cloth folds, fine white thread, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. A string of small triangle flags between two short wooden posts, about two moles wide and as tall as the mole: flags in red, yellow and blue.
2. A small swing, about as tall and as wide as the mole: a wooden frame with two white silk threads holding a small wooden seat.
3. A white silk thread loop tied to the top of a short wooden stick stuck upright, about half a mole wide and as tall as the mole.
```

### E5 — 텃밭: 빈 흙 칸 · 갓 심은 칸 · 다 익은 산딸기 덤불 (사장 2026-11-27 "텃밭도")

텃밭에서 코드로 그리는 것은 흙 칸(네모 넷)·갓 심은 씨앗(점 셋)·다 익은 산딸기의 빨간 알 셋이다. 자라는 새싹과 꽃 일곱은
팩·CC0 그림(`plants.png`)이라 그대로 둔다(`docs/art/code-drawn-inventory.md`).

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

Make the props rich and handcrafted: crumbly dark soil, wooden edging, leaves and berries, little highlights and dark corners.
Top-down 3/4 view. Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. An empty garden plot, about as wide as the mole and low: a small rectangle of dark tilled soil with three furrows, edged with short wooden planks.
2. The same garden plot just planted: the same soil and plank edging, with three tiny seed mounds and a small wooden marker stick at one corner.
3. A ripe wild berry bush, about as wide as the mole and two thirds as tall: a round leafy green bush full of small bright red berries.
```

### 받은 것(2026-11-27)
- E1·E2·E3·E5: `hub_e1_v1.png`·`hub_e2_v1.png`·`hub_e3_v1.png`·`hub_e5_v1.png` → `hub_ai.py` `ITEMS` → `hub.png` 칸 6~17.
  바람종은 가는 기둥이 외곽선 벗기기(6px)에 사라져 따로 벗김 4px·키 30(`THIN`), 텃밭 흙은 줄이면 민무늬가 돼 이랑과 알갱이를
  찍는다(`soil_grain` — 사장 "중간에 노이즈가 좀 있어야").
- E4(깃발 줄·실 그네·실 고리): `hub_e4_v1.png`(JPEG로 와서 바탕 잡티를 순수 마젠타로 정리해 PNG로 저장) → 칸 21~23. 셋 다 가는 줄·실이
  있어 `THIN`. 실 고리는 받은 것이 "실을 묶은 막대 하나"라, 막대 그림을 둘 세우고 사이에 뜨는 실뜨기·빛은 코드로 둔다.

### E6 — 물웅덩이 · 다시 솟는 샘 · 길의 샘
첫 판은 각도를 적지 않아 돌 테두리의 옆면이 보이는 우물 같은 그림이 왔다(사장 "각도가 좀 맘에 안 드네"). 그림체 판에 화덕 돌 고리를
넣고(`hub_ref.py`) "그 고리와 같은 각도, 옆면 없이, 납작한 타원"이라고 적는다. 물결·솟는 물방울은 코드가 얹는다.

```
Draw three new props in EXACTLY the same pixel art style as the attached sprites (a cute top-down action game:
chunky pixels, each pixel a 6x6 block, dark plum 1px outline, soft 3-4 tone shading, light from the top-left,
warm cozy colors). The attached sprites are only a STYLE reference: do not redraw them.

VIEWING ANGLE IS THE MOST IMPORTANT RULE: all three are FLAT things lying on the ground, seen from high above,
at exactly the same angle as the grey stone campfire ring in the attached image (the last sprite on the right).
The water surface is level with the ground. Rim stones are low flat stones lying on the ground: show only their
tops, NO side walls, NO raised basin, NOT a well. Each shape is a wide flat oval, about twice as wide as it is tall.

Make them rich and handcrafted: clear water with soft reflections, wet dark earth, moss, small pebbles.
Solid magenta background (#FF00FF), one row, three props well separated, no text, no shadows, no glow.
IMPORTANT: draw ONLY the objects. Do NOT draw any ground, grass or soil around them.
1. A small shallow puddle, about as wide as the mole: clear blue water with a thin uneven rim of wet dark earth and one soft light reflection. No stones.
2. A small natural spring, a little wider than the mole: the same kind of flat pool with a loose ring of small flat mossy stones around it and a small circle of lighter water in the middle where water wells up.
3. A larger spring pool, about two moles wide: a flat oval pool of clear blue-green water with a loose ring of flat mossy stones and a few tiny white flowers between them.
```

**E6 받음(2026-11-27, `hub_e6_v1.png`)**: 위 프롬프트로 다시 받은 것도 "아직 너무 위에서 보는 것 같다"(사장) — 모델이 각도를 글로는 잘
안 따른다. 다시 받지 않고 **변환기가 세로로 누른다**(`hub_ai.py` `SQUASH`: 물웅덩이 0.72·샘 0.64·길의 샘 0.76 → 폭이 높이의 두 배쯤).
바닥에 누운 것을 받을 때는 각도를 글로 맞추려 하지 말고 받은 뒤 누르는 쪽이 빠르다.

