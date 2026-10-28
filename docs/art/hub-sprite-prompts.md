# 마을·쉼터 새 그림 프롬프트(0224)

시트마다 **새 대화**에서 그 시트의 참고 그림 한 장만 붙이고 아래 프롬프트를 넣는다.
참고 그림은 `python3 tools/art/hub_ref.py` → `tools/art/sources/ai/sprites/ref/ref_hub_<a|b|c>.png`.
받은 원본은 `tools/art/sources/ai/sprites/hub_<a|b|c>_v1.png`로(✦는 `strip_logo.py`로 지운다).

배운 것(소품 0222·잡몹 0221): 크기가 가장 중요하다고 먼저 말하고, 참고 그림의 색·자리를 따르게 하고, 작은 표지는 굵게.

## 시트 A — 쉼터·마을 공용 물건 여섯

```
Redraw the six objects in the attached image (to the right of the small mole) as a pixel art sprite sheet for
a cute top-down action game. SIZE IS THE MOST IMPORTANT RULE: draw each object exactly as big as it is in the
attached image, next to the small mole, with the same pixel block size. Keep their colors and layout.

3 columns x 2 rows, 6 objects, each well separated, solid magenta background (#FF00FF), no text, no grid lines,
no shadows, no effects, no fire. Top-down 3/4 view, light from the top-left, dark plum 1px outline. Keep details bold.
1. a raccoon's little market stall: red-and-cream striped awning on two wooden posts, a wooden counter with a basket of red berries and a small seed sack
2. a campfire ring: nine grey stones in an oval around two crossed logs, NO flames (the game draws the fire)
3. a tree stump with rings on top and three small roots
4. a round grey boulder with a little green moss on top
5. a short log lying on its side, used as a seat
6. a dark burrow hole in the ground with four roots around its rim and a small wooden signpost on the left
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
