"""Naïl·콩이·큰 나무 다시 그리기(0238): 나노바나나에 붙일 그림체 판.

실행: python3 tools/art/redo_ref.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_redo_style.png: 그림체 기준 — 사장이 좋다고 한 새 그림(모모·도토·포포·수수·보리)만, 발 줄을 맞춰 6배, 마젠타 바탕.
    지금 Naïl·콩이는 넣지 않는다(다시 그릴 대상이라 붙이면 그 생김새를 따라 그린다 — KB image-model-for-game-illustrations).

모두 나노바나나 원본에서 옮긴 우리 그림이다(팩 그림 아님, 0064).
"""
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
ART = HERE.parents[1] / "assets" / "art"

# (파일, 칸, 발 줄(칸 안)). 줄 0 첫 칸 = 서기.
CELLS = [
    ("momo.png", 32, 30),
    ("doto.png", 40, 38),
    ("popo.png", 32, 30),
    ("susu.png", 32, 30),
    ("chief.png", 40, 38),
]


def style_board():
    cells = [(Image.open(ART / f).convert("RGBA").crop((0, 0, c, c)), foot) for f, c, foot in CELLS]
    k, line = 6, 42
    w = sum(c.width for c, _ in cells) + 4 * (len(cells) + 1)
    out = Image.new("RGBA", (w, line + 4), (255, 0, 255, 255))
    x = 4
    for c, foot in cells:
        out.alpha_composite(c, (x, line - foot))
        x += c.width + 4
    return out.resize((out.width * k, out.height * k), Image.NEAREST)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    style_board().save(OUT / "ref_redo_style.png")
    print(OUT / "ref_redo_style.png")


if __name__ == "__main__":
    main()
