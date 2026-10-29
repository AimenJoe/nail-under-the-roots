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

## 2단계 — 도트 시트(고른 안을 붙여서, Naïl 예)
```
Turn the attached design (the one I chose) into a pixel art sprite sheet in EXACTLY the style of the other attached
sprites. Keep the design's proportions and colors. Solid magenta background (#FF00FF), no text, no grid lines.
Row 1: idle (2 frames, slight breathing). Row 2: walk cycle (4 frames, paws swinging).
Same size and same pixel block size in every frame, feet on the same line, facing right.
```
그다음 시트 B(치기 3·들기·맞음·쓰러짐·거두기)는 0206과 같다 — 시트 A를 붙여서. 콩이는 서기·걷기·말기·구르기·맞음·
뒤집힘(0054의 칸 목록, `kongi_ai.py`의 `Spec`).

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
