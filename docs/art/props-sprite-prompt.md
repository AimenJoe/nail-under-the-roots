# 소품 새 그림 프롬프트(0222)

새 대화에서 `tools/art/sources/ai/sprites/ref/ref_props.png` 한 장만 붙이고 아래를 넣는다. 받은 원본은 `tools/art/sources/ai/sprites/props_v1.png`로.

```
Redraw the six small objects in the attached image (to the right of the small mole) as a pixel art sprite sheet for
a cute top-down action game. Keep their shapes and colors, but make them look like nice pixel art props.

Use the attached image for size: draw each object as big as it is there, next to the small mole, same pixel block size.

3 columns x 2 rows, 6 objects, each well separated, solid magenta background (#FF00FF), no text, no grid lines,
no shadows, no effects. Seen from a top-down 3/4 view, light from top-left, dark plum 1px outline around each object.
Every breakable object (1-4) has one bright zigzag crack down its middle, so the player sees it can be broken.
1. a small brown dirt mound with two pebbles
2. a reddish root gall (a knobby lump) wrapped by a dark root
3. a pile of three grey stones
4. a pale lilac ball of silk thread with a few purple threads around it
5. a small wooden treasure chest, closed, with a gold latch
6. the same chest, opened (lid up, dark empty inside)
```

## 2. 물길 굴의 부술 것(0275 P3) — 하나만 (쓰지 않음: 사장이 Sunnyside 시안 D를 골라 `Breakable::RiverStone`으로, 2026-11-03)
새 대화에서 `tools/art/sources/ai/sprites/props_v1.png`(지금 소품 원본, 그림체·크기 참고) 한 장만 붙이고 아래를 넣는다. 받은 원본은 `tools/art/sources/ai/sprites/props_water_v1.png`로(Claude가 ✦를 지우고 `props_ai.py`에 일곱째 칸으로 넣는다). 들어오기 전에는 심층 돌무더기를 쓴다.

```
Using the attached sprite sheet only as a style and size reference, draw ONE new breakable object for the same
cute top-down pixel art game: a small pile of three wet, rounded river stones in cool blue-grey, with a patch of
teal-green moss on top and one tiny pale snail shell tucked between the stones. Draw it about as big as the
grey stone pile (the third object) in the attached sheet, with the same pixel block size, the same dark plum 1px
outline and the same top-down 3/4 view with light from the top-left. Like the other breakable objects, it has
one bright pale-yellow zigzag crack down its middle so the player sees it can be broken. Make it look wet: a few
small bright highlights on the stones, no water around it. One object only, centered, on a solid magenta
background (#FF00FF), no text, no grid lines, no shadows, no effects. Aspect ratio 1:1.
```
- 바뀌어 나오기 쉬운 것: 금이 노랑이 아니라 하양(→ "pale-yellow zigzag crack like the other objects"), 물웅덩이를 그려 넣음(→ "no water around it"), 외곽선이 검정(→ "dark plum outline").
