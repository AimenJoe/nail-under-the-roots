# Naïl·콩이·큰 나무 다시 그리기 — 프롬프트(0238)

모두 **새 대화**에서. 붙이는 것은 그림체 판 `tools/art/sources/ai/sprites/ref/ref_redo_style.png` 한 장뿐
(`python3 tools/art/redo_ref.py`가 만든다 — 사장이 좋다고 한 모모·도토·포포·수수·보리만 들어 있다).
**지금 Naïl·콩이 그림은 붙이지 않는다**: 붙이면 그 생김새를 따라 그린다(0206·0224에서 두 번 겪음).
받은 원본은 `tools/art/sources/ai/sprites/<이름>_v1.png`로 두고 ✦는 `strip_logo.py`로 지운다.

순서: **1단계 생김새 고르기(큰 그림 여섯 안)** → 사장이 하나 고름 → **2단계 도트 시트**(고른 그림을 붙여서).
작게 줄였을 때 덩어리가 되지 않는지가 기준이라, 1단계에서 고른 안은 우리가 실제 크기(Naïl 28px, 콩이 30px)로 줄여 한 색 실루엣 시험을 먼저 한다.

## Naïl 1단계 — 생김새 여섯 안
```
Character design exploration for the HERO of a cute top-down pixel art action game: a BABY MOLE named Naïl who fights
and digs with big claws. The attached sprites are the STYLE reference (chunky pixels, dark plum 1px outline, soft
3-4 tone shading, light from the top-left, warm cozy colors). Do not redraw the attached characters.

Draw SIX different designs of the baby mole in a 3x2 grid, each standing in a 3/4 top-down view facing right,
each about as tall as the raccoon in the reference. Solid magenta background (#FF00FF), no text, no shadows.
Every design must read as a MOLE from its silhouette alone, even when shrunk to 28 pixels tall:
- two BIG spade-shaped front paws held out to the sides with pale cream claws (the biggest shape of the body),
- a short snout ending in a big round pink nose (not a long trunk, not a beak),
- tiny eyes (can be shiny dots or small happy arcs), velvety dark brown fur, short stubby legs, tiny tail.
Vary the proportions between the six: chubby pear body / upright bean / wide shoulders and big paws / round head
with small body / slightly hunched digger / heroic stance with one paw raised. Give some a small accessory that
says "hero who left home": an orange scarf, a leaf cape, or a tiny miner's headband. Keep it cute and readable.
```

## 콩이 1단계 — 생김새 여섯 안
```
Character design exploration for a BOSS-then-friend character in a cute top-down pixel art action game: KONGI, a
three-banded armadillo (Tolypeutes) gatekeeper who can curl into a perfect ball. The attached sprites are the STYLE
reference (chunky pixels, dark plum 1px outline, soft 3-4 tone shading, light from the top-left, warm colors).
Do not redraw the attached characters.

Draw SIX different designs in a 3x2 grid, each in a 3/4 top-down view facing right, each about as tall as the
squirrel in the reference. Solid magenta background (#FF00FF), no text, no shadows.
Must read from the silhouette alone at 30 pixels tall:
- a domed shell with 3 clear bands across the middle and a separate head shield and tail shield,
- a small pointy face with big ears and a pink snout, short sturdy legs with claws,
- proud, stubborn gatekeeper personality (arms crossed, chin up, or planted feet).
Vary: round dome / tall shell with tiny legs / chunky with big ears / sleek with long tail / old-soldier with
a scratch on the shell / youthful and bouncy. Shell colors in warm sand or olive with darker band lines.
In one corner of each cell also draw the SAME design curled into a ball (bands visible), half size.
```

## 2단계 — 도트 시트(사장 고름 2026-10-29: Naïl 1번, 콩이 3번)
붙이는 것: **고른 안**(`ref/ref_nail_pick.png`·`ref/ref_kongi_pick.png`, `redo_concepts.py`가 1단계 원본에서 잘라 냄) +
**그림체 판**(`ref/ref_redo_style.png`). 대화마다 새로. 받은 시트는 `sprites/nail_v3_a.png`·`nail_v3_b.png`·`kongi_v2.png`.
게임 크기: Naïl 키 약 22px(지금 18 → 모모 23·도토 25와 나란히), 콩이 키 약 20px·공 지름 약 16px. 줄이는 건 우리가 한다 —
시트는 크게, 칸마다 같은 크기·같은 점 크기로만 받으면 된다.

### Naïl 시트 A — 서기·맞음·쓰러짐·걷기(붙임: ref_nail_pick + ref_redo_style)
```
Turn the first attached character design (a baby mole hero with an orange scarf and big spade paws held out to the
sides) into a pixel art sprite sheet, in EXACTLY the pixel style of the second attached image (chunky pixels, dark plum
1px outline, soft 3-4 tone shading, light from the top-left). The second image is only a style reference: do not draw
those animals. Keep the mole's design exactly: dark brown velvety fur, lighter brown belly, short snout with a big round
pink nose, small shiny eye, orange scarf tied at the neck with the knot at the back, big pink spade paws with pale cream
claws held out wide to the sides, small pink feet, tiny tail.

2 rows x 4 frames, evenly spaced on a solid magenta background (#FF00FF), no text, no grid lines, no shadows, no ground,
no effects. Every frame the same character size and the same pixel block size, facing right in a 3/4 top-down view,
feet on the same baseline in each row.
Row 1: (1) idle, paws out to the sides; (2) idle breathing, body 1 pixel lower and paws slightly lower;
       (3) hurt: flinching backward, eyes squeezed shut, paws up; (4) knocked down: lying on his back, paws limp.
Row 2: walk cycle, 4 frames: short stubby legs stepping, the spade paws swinging forward and back in opposition,
       body bobbing 1 pixel.
```

### Naïl 시트 B — 치기 셋·모으기·거두기·파고들기(A를 받은 뒤. 붙임: 받은 시트 A + ref_redo_style)
```
Using the attached sprite sheet of the baby mole (same character, same size, same pixel block size, same colors and
outline), draw his attack frames in the same pixel style. 3 rows x 3 frames, evenly spaced on a solid magenta background
(#FF00FF), no text, no grid lines, no shadows, no ground, no motion lines, no slash effects, facing right, feet on the
same baseline.
Row 1: (1) wind-up 1: right paw pulled back; (2) strike 1: right paw swiped forward, claws leading;
       (3) wind-up 2: left paw pulled back across the body.
Row 2: (1) strike 2: left paw swiped forward; (2) heavy wind-up: crouched low, both paws raised high above the head;
       (3) heavy strike: both paws slammed down in front, body leaning forward.
Row 3: (1) settle: paws coming back down to the sides; (2) diving into the ground: head and front paws pointing down,
       body tilted forward as if starting to dig; (3) bursting up: jumping straight up, paws raised, body stretched.
```

### 콩이 시트 — 서기·맞음·쓰러짐·걷기·공(붙임: ref_kongi_pick + ref_redo_style)
```
Turn the first attached character design (a three-banded armadillo with a sand-colored domed shell, big pink-lined
ears, a pointy face with a pink snout, short sturdy legs and a tail shield; the ball next to it is the same armadillo
curled up) into a pixel art sprite sheet, in EXACTLY the pixel style of the second attached image (chunky pixels, dark
plum 1px outline, soft 3-4 tone shading, light from the top-left). The second image is only a style reference: do not
draw those animals. Keep the armadillo's design exactly: three clear shell bands across the middle, a separate head
shield and tail shield, big ears, pale belly, proud stubborn expression.

3 rows x 4 frames, evenly spaced on a solid magenta background (#FF00FF), no text, no grid lines, no shadows, no ground,
no effects. Every frame the same character size and the same pixel block size, facing right in a 3/4 top-down view,
feet on the same baseline in each row.
Row 1: (1) idle, standing proud with chin up; (2) idle breathing, shell 1 pixel lower;
       (3) hurt: flinching back, eyes shut; (4) knocked over: lying on his back, legs in the air, shell on the ground.
Row 2: walk cycle, 4 frames: short legs stepping, body and shell bobbing 1 pixel.
Row 3: (1) curled into a tight ball, bands visible; (2) the same ball rolled a quarter turn (bands rotated);
       (3) half curled: head tucked, tail shield coming around; (4) uncurling: peeking out of the ball.
The ball is about two thirds of the standing height.
```
받은 뒤 우리 쪽: `nail_ai.py`(Spec 원본 둘·`body_h` 20 · 칸 24 → 28)와 `kongi_ai.py`(원본·칸 상자, 걷기 넷은 시트 안에서),
`check_rebuild.py`, 게임 크기 그림 확인.

## 큰 나무 — 두 장(시든 나무 + 다 핀 나무)
다섯 단계를 한 장에 청하면 줄기·뿌리가 단계마다 달라진다(같은 그림을 다섯 번 그리게 하는 것은 잘 안 된다).
그래서 **두 장만** 받고, 단계는 우리가 만든다: 시든 나무(자람 0) 위에 다 핀 나무의 잎을 덩어리별로 잘라
자람 1→4에 3·7·10·12덩어리씩 얹는다(지금 코드와 같은 수, 0045). 줄기·뿌리는 시든 그림 하나라 단계마다 같다.
```
Draw the GREAT TREE at the heart of a cozy underground-animal village, for a cute top-down pixel art game, in
EXACTLY the pixel style of the attached sprites (chunky pixels, dark plum 1px outline, soft 3-4 tone shading,
light from the top-left). The attached sprites are only a style reference: do not redraw them.
This tree is the soul of the game, ancient and kind, the most important object in the village. Make it majestic.
SIZE: about five times as tall as the raccoon in the reference and about six times as wide; the trunk is as wide
as three raccoons. Top-down 3/4 view.
Draw TWO versions side by side, IDENTICAL trunk, roots and branches (only the leaves differ):
LEFT — the tree is sick: thick gnarled trunk with deep bark grooves, huge roots gripping a small cliff edge and
diving into the ground, a soft glowing hollow at the base like a tiny shrine, bare twisted branches with only a
few dry leaves, thin purple threads wrapped around two roots.
RIGHT — the same tree fully healed: a big round lush crown of many leaf clusters (clearly separated clumps, each
with its own highlight), small pink blossoms scattered in the clumps, fireflies, no purple threads.
Solid magenta background (#FF00FF) around and between the two trees, no ground tile, no text, no shadows.
```
우리 쪽 일: `hub_ai.py`에 나무 칸(폭 약 110·키 약 90 게임 px — 마을 지도에서 나무 위 여백이 60px라 지도 `village.rs`의
Y 자리를 한두 줄 내릴 수 있다), 잎 덩어리 자르기(마젠타·외곽선으로 덩어리 나누기), 자람 단계 합성, 반딧불·흔들림은 코드.

## 파 내려가기 — 땅 단면 전환의 큰 Naïl(0156 → 0238 덧붙임) — 보류
받은 시트(2026-10-29)는 머리를 위로, 서서 등을 보인 자세였다. 180° 돌려 써 보려 했지만 사장: "머리로 파는 게 아니잖아" —
두더지는 삽 손으로 앞을 파며 들어간다. 지금 `dig.png`(손 그림)를 그대로 둔다. 다시 청할 때는 "앞발로 흙을 헤치며 비스듬히
파 들어가는 옆모습"처럼 손이 주인공인 자세로.

붙임: **새 Naïl 시트 A**(`sprites/nail_v3_a.png`, 몸·색) + **자세 참고**(`ref/ref_dig_pose.png`, `tools/art/dig_ref.py`가 지금
`dig.png`의 Naïl 줄을 아래 배치대로 8배로 — 옛 몸이라 자세·카메라만). 받은 것은 `sprites/nail_dig_v1.png`.
게임은 32×32 칸 16개를 쓴다: 긁기 6 · 보기 · 윙크 · 돌기 8. 돌기 넷은 우리가 둘씩 늘려 8칸으로, 털빛 셋은 색 바꾸기.
```
Draw the same baby mole as the first attached sprite sheet (dark brown fur, lighter belly, orange scarf with the knot
at the back, big pink spade paws with pale cream claws, big round pink nose, small pink feet, tiny tail), in exactly
the same pixel style, colors and pixel block size, digging straight DOWN into the earth.
Camera: we look down at his back from above and behind. His head points DOWN toward the bottom of each frame, his back
and rump face us, the tiny tail and the pink soles of his feet are at the top of the frame. The second attached image
shows the POSES and the camera only (it is an older drawing of him): copy those poses, not that drawing.

3 rows x 4 frames, evenly spaced on a solid magenta background (#FF00FF), no text, no grid lines, no soil, no dirt
particles, no shadows, no effects. Same character size and same pixel block size in every frame, body centered.
Row 1: dig cycle, 4 frames: the two spade paws scoop alternately beside his head and throw earth back past the
       shoulders (left paw reaching forward / both paws at the shoulders / right paw reaching forward / both at the shoulders).
Row 2: (1) he pauses and turns his head to look back up at us over his shoulder, face upside down, happy;
       (2) the same, winking; (3) both paws reaching far down past his head; (4) both paws pulled back to his shoulders.
Row 3: spinning around his own head-to-tail axis like a corkscrew, 4 frames: back view / right side view /
       belly view (pale belly and the front of the scarf) / left side view.
```

## Naïl 3단계 — 머리 큰 시트(0242 비교용, 2026-10-29)
지금 게임의 Naïl은 v3 시트에서 **줄이기 전에 머리를 1.35배 키우고 눈을 크게 그린 것**(`nail_ai.chibi`)이다. 이 비율을 처음부터 그린
시트를 새로 받아, 같은 변환기로 줄인 것과 지금 것을 나란히 놓고 사장이 고른다.

붙이는 것(새 대화): **비율 참고** `ref/ref_nail_chibi.png`(`python3 tools/art/chibi_ref.py` — 머리 키운 큰 그림 넷: 서기·걷기 / 치기·거두기) +
**그림체 판** `ref/ref_redo_style.png`. 지금 게임 칸을 8배로 키운 `ref_nail_chibi_game.png`는 **붙이지 않는다**(비교용 — 점 그림을
붙이면 덩어리 그대로 따라 그린다). 받은 원본은 `sprites/nail_v4_a.png`·`nail_v4_b.png`(✦는 `strip_logo.py`).
게임 크기: 키 약 24px, 칸 28. 줄이는 건 우리가 한다 — 칸마다 같은 크기·같은 점 크기로만.

### Naïl v4 시트 A — 서기·맞음·쓰러짐·걷기(붙임: ref_nail_chibi + ref_redo_style)
```
Redraw the baby mole hero from the first attached image as a pixel art sprite sheet with CUTE CHIBI PROPORTIONS,
in EXACTLY the pixel style of the second attached image (chunky pixels, dark plum 1px outline, soft 3-4 tone shading,
light from the top-left). The second image is only a style reference: do not draw those animals.
Keep his design: dark brown velvety fur, lighter brown oval belly, short snout with a big round pink nose, orange scarf
tied at the neck with the knot at the back of the head, big pink spade paws with pale cream claws, small pink feet,
tiny tail. Proportions like the first image, pushed a little further: about 2.5 heads tall, the round head is about
45% of his total height and as wide as his body, a short soft pear-shaped body, short stubby legs. ONE big shiny black
eye with a white highlight placed just behind the snout (it must stay readable when the sprite is shrunk to 24 pixels
tall), a small pink blush under the eye. Keep the outline thin and even: no thick black areas under the head or around
the neck — shade the chin and neck with darker brown, not black.

2 rows x 4 frames, evenly spaced on a solid magenta background (#FF00FF), no text, no grid lines, no shadows, no ground,
no effects. Every frame the same character size and the same pixel block size, facing right in a 3/4 top-down view,
feet on the same baseline in each row.
Row 1: (1) idle, spade paws held out wide to the sides; (2) idle breathing, body 1 pixel lower, paws slightly lower;
       (3) hurt: flinching backward, eye squeezed shut, paws up near the face; (4) knocked down: lying on his back, paws limp.
Row 2: walk cycle, 4 frames: short stubby legs clearly stepping (one foot forward, then the other), the spade paws
       swinging forward and back in opposition, head and body bobbing 1 pixel. The head size stays the same in every frame.
```

### Naïl v4 시트 B — 치기·모으기·거두기(A를 받은 뒤. 붙임: 받은 시트 A + ref_redo_style)
```
Using the attached sprite sheet of the chibi baby mole (same character, same big head, same size, same pixel block size,
same colors and outline), draw his attack frames in the same pixel style. 3 rows x 3 frames, evenly spaced on a solid
magenta background (#FF00FF), no text, no grid lines, no shadows, no ground, no motion lines, no slash effects, facing
right, feet on the same baseline. Keep the big shiny eye visible in every frame (do not hide the face behind the paws).
Row 1: (1) wind-up 1: right paw pulled back; (2) strike 1: right paw swiped forward, claws leading;
       (3) wind-up 2: left paw pulled back across the body.
Row 2: (1) strike 2: left paw swiped forward; (2) heavy wind-up: crouched low, both paws raised beside the head (not
       covering it); (3) heavy strike: both paws slammed down in front, body leaning forward.
Row 3: (1) settle: paws coming back down to the sides; (2) diving into the ground: head and front paws pointing down,
       body tilted forward; (3) bursting up: jumping straight up, paws raised, body stretched.
```

받으면: `nail_ai.py`에 v4 원본·칸 상자를 넣고 `chibi`는 끈 채(원본이 이미 머리 큰 비율) 줄여, 지금 시트와 8배 나란히 비교 그림을 만든다 →
사장이 고른 쪽을 쓴다. 고르기 전에는 `assets/art/nail.png`를 바꾸지 않는다.
