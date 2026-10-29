"""Naïl 머리 큰 시트(0242 비교)를 나노바나나로 새로 받을 때 붙이는 참고 그림.

실행: python3 tools/art/chibi_ref.py   → tools/art/sources/ai/sprites/ref/ref_nail_chibi.png (+ 비교용 ref_nail_chibi_game.png)

`nail_ai.chibi`가 원본(nail_v3_a·b)에서 머리를 1.35배 키우고 눈을 크게 그린 큰 그림 네 칸(서기·걷기 / 치기·거두기)을
마젠타 바탕 한 장에 늘어놓는다. 사장이 고른 비율(0242)을 나노바나나에 그대로 보이려는 것 — 생김새·목도리·앞발은 지금 Naïl 그대로.
비교용 한 장은 지금 게임 칸(28px)을 8배로 — 생성 대화에는 붙이지 않는다(점 그림을 따라 그리면 덩어리가 커진다).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import nail_ai  # noqa: E402

REF = HERE / "sources" / "ai" / "sprites" / "ref"
PICK = [(0, "idle_a"), (0, "walk_a"), (1, "attack0"), (1, "settle")]   # 맞음·들기는 앞발이 머리와 겹쳐 흠이 남아 뺀다
H = 300          # 칸 높이(원본 px) — 칸마다 발을 같은 줄에
GAP = 40


def frames():
    out = {}
    for si, src in enumerate(nail_ai.SPEC_V3.sources):
        img = np.asarray(Image.open(src.path).convert("RGB")).astype(int)
        done, boxes = nail_ai.chibi(img, dict(src.frames), ref=True)
        for k, (x0, y0, x1, y1) in boxes.items():
            out[(si, k)] = Image.fromarray(done[y0:y1, x0:x1].astype(np.uint8), "RGB")
    return out


def main():
    f = frames()
    cells = [f[p] for p in PICK]
    W = max(c.width for c in cells)
    Hh = max(max(c.height for c in cells), H)
    sheet = Image.new("RGB", (2 * (W + GAP) + GAP, 2 * (Hh + GAP) + GAP), (255, 0, 255))
    for i, c in enumerate(cells):
        x = GAP + (i % 2) * (W + GAP) + (W - c.width) // 2
        y = GAP + (i // 2) * (Hh + GAP) + (Hh - c.height)
        sheet.paste(c, (x, y))
    REF.mkdir(parents=True, exist_ok=True)
    sheet.save(REF / "ref_nail_chibi.png")
    game = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA")
    C, Z = nail_ai.CELL, 8
    picks = [(0, 0), (2, 0), (3, 1), (0, 1), (7, 1), (8, 1)]
    g = Image.new("RGBA", (len(picks) * C * Z, C * Z), (255, 0, 255, 255))
    for i, (c, r) in enumerate(picks):
        g.alpha_composite(game.crop((c * C, r * C, (c + 1) * C, (r + 1) * C)).resize((C * Z, C * Z), Image.NEAREST),
                          (i * C * Z, 0))
    g.convert("RGB").save(REF / "ref_nail_chibi_game.png")
    print(REF / "ref_nail_chibi.png")


if __name__ == "__main__":
    main()
