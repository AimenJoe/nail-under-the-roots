# 걷기 네 칸 프롬프트(0229)

> 여섯 다 들어감(2026-10-29, v0.5.2): 포포·보리·수수·콩이·토끼풀·모모. 옮기는 법은 `tools/art/walk_ai.py`·결정 0229.

이웃·캐릭터마다 **새 대화에서 그 이웃의 참고 그림 한 장만** 붙인다(`python3 tools/art/walk_ref.py`, 지금 시트의
서기 칸(왼쪽) + 지금 걷기 칸(오른쪽)을 6배로 — 우리 그림). 받은 원본은 `tools/art/sources/ai/sprites/<이름>_walk_v1.png`
(✦는 `strip_logo.py`로 지운다). 도토 달리기(`doto-run-prompt.md`)와 같은 방식.

| 이름 | 참고 그림 | 걸음 |
|---|---|---|
| 포포(라쿤) | `ref/ref_walk_popo.png` | 두 발 걷기 |
| 보리(들쥐 대장) | `ref/ref_walk_bori.png` | 두 발 걷기(늙어서 느릿) |
| 수수(땅강아지) | `ref/ref_walk_susu.png` | 두 발 걷기(통통, 짧은 다리) |
| 콩이(세띠아르마딜로) | `ref/ref_walk_kongi.png` | 네 발 걷기 |
| 토끼풀(어미 굴토끼) | `ref/ref_walk_clover.png` | 깡충(토끼는 뛰어 간다) |
| 모모(굴토끼) | `ref/ref_walk_momo.png` | 깡충 |

먼저 포포 하나로 시험해 보고, 괜찮으면 나머지를 같은 틀로.

## 두 발 걷기(포포·보리·수수) — `<…>`만 바꾼다
- 포포: `Popo, a friendly raccoon guide and shopkeeper with a small yellow pencil tucked behind one ear`
- 보리: `Bori, an old field mouse village chief, a little hunched and slow`
- 수수: `Susu, a chubby mole cricket craftsman with little goggles on the head and short sturdy legs`

```
This is <NAME AND DESCRIPTION>, from a cute top-down pixel art action game
(left: standing; right: the current walking frames). Draw a 4-frame WALK CYCLE of the same character,
facing RIGHT in the same 3/4 view, same size and same pixel block size as the sprites in the image,
same colors, same dark outline.

1 row x 4 frames, evenly spaced, solid magenta background (#FF00FF), no text, no shadows:
1. contact: front foot stepping forward and touching the ground, back foot behind, arms swinging opposite
2. passing: weight on the front foot, body a little lower, back foot lifting past it
3. contact: the other foot steps forward and touches the ground, arms swinging the other way
4. passing: weight on that foot, body a little higher, the other foot lifting past it
Keep the head, face, ears and accessories identical in every frame; only the legs, arms, tail
and a small up-down body bob change.
```

## 네 발 걷기(콩이)
```
This is Kongi, a small three-banded armadillo with a pink face and a banded olive shell, from a cute
top-down pixel art action game (left: standing; right: the current walking frames). Draw a 4-frame WALK
CYCLE of the same armadillo, side view facing RIGHT, same size and same pixel block size as the sprites
in the image, same colors, same dark outline.

1 row x 4 frames, evenly spaced, solid magenta background (#FF00FF), no text, no shadows:
1. front-right and back-left legs reaching forward, the other two pushing back
2. legs passing under the body, shell slightly lower
3. front-left and back-right legs reaching forward, the other two pushing back
4. legs passing under the body, shell slightly higher
Keep the head, snout, ears and shell bands identical in every frame; only the short legs and a small
body bob change.
```

## 깡충(토끼풀·모모) — `<…>`만 바꾼다
- 토끼풀: `Clover, a mother rabbit with long ears and a small pink flower on one ear`
- 모모: `Momo, a young wild rabbit`

```
This is <NAME AND DESCRIPTION>, from a cute top-down pixel art action game (left: standing;
right: the current hopping frames). Draw a 4-frame HOP CYCLE of the same rabbit, side view facing RIGHT,
same size and same pixel block size as the sprites in the image, same colors, same dark outline.

Rabbits move in small hops. 1 row x 4 frames, evenly spaced, solid magenta background (#FF00FF), no text,
no shadows:
1. crouch: body bunched low, hind feet under the body, ears back
2. push-off: hind legs stretching back, body rising and leaning forward
3. airborne: all paws off the ground, body long, front paws reaching forward
4. landing: front paws touching down first, hind legs swinging forward under the body
Keep the head, face, ears and accessories identical in every frame; only the body and legs change.
```
