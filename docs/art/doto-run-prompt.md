# 도토 달리기 네 칸 프롬프트(0226)

새 대화에서 `tools/art/sources/ai/sprites/ref/ref_doto_run.png`(`python3 tools/art/doto_run_ref.py`, 지금 도토 시트의 서기·달리기 칸 6배 —
우리 그림) 한 장만 붙인다. 받은 원본은 `tools/art/sources/ai/sprites/doto_run_v1.png`(✦는 `strip_logo.py`로 지운다).
옮기면 `doto.png` 칸 8~11이 되고, 게임은 칸이 있으면 달릴 때 그 넷을 돈다(`render/actors.rs draw_kin_player`).

```
This is Doto, a small red squirrel with a little acorn-cap hat, from a cute top-down pixel art action game
(left: standing, right: running). Draw a 4-frame RUN CYCLE of the same squirrel, side view facing RIGHT,
same size and same pixel block size as the running sprite on the right, same colors, same dark outline.

Squirrels run in bounds. 1 row x 4 frames, evenly spaced, solid magenta background (#FF00FF), no text, no shadows:
1. push-off: hind legs pushing back, body stretched, tail streaming behind
2. airborne: all paws off the ground, body long and flat, tail waving up
3. landing: front paws touching down, back arching, hind legs swinging forward
4. gathered: hind paws landing just in front of the front paws, body bunched up, tail curled high
Keep the head, hat and tail the same in every frame; only the body and legs change.
```
