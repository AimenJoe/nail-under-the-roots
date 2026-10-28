"""이웃 둘(포포 라쿤 · 수수 땅강아지) 실루엣·색 안내·크기 기준(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림.

실행: python3 tools/art/neighbors_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_popo_silhouette.png · ref_susu_silhouette.png: 32칸 실루엣, 16배.
  - ref_popo_blocking.png · ref_susu_blocking.png: 크게 칠한 색 안내(캐릭터 시트 `sheets/popo.png`·`sheets/susu.png` 색).
  - ref_neighbors_scale.png: 게임 크기 Naïl(몸 16px) 옆에 포포·수수 실루엣, 마젠타 바탕 8배(점 크기·몸 크기 기준).

왜: 둘 다 옛 16칸 코드 그림이라 새 Naïl(24칸)·모모(32칸) 곁에서 따로 논다.
  - 포포: 튜토리얼 길잡이이자 마을 가게 — 어른이라 Naïl보다 조금 크다(키 24px). 표지: 눈가의 검은 가면, 둥근 귀 둘,
    귀 뒤에 꽂은 연필, 몸 뒤로 나온 굵은 줄무늬 꼬리.
  - 수수: 땅강아지 — Naïl만 한 키(19px)에 옆으로 둥글고 묵직하다. 표지: 머리 위에 올린 고글 둘, 앞으로 내민 삽 같은
    앞발(검보라), 등의 둥근 갈색 몸통, 가는 더듬이 둘.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL = 32

POPO = {
    "fur": (138, 138, 150), "fur_dark": (96, 96, 110), "mask": (40, 36, 46), "white": (240, 238, 236),
    "tail_dark": (60, 58, 70), "foot": (232, 214, 170), "eye_light": (255, 255, 255), "nose": (34, 28, 36),
    "pencil": (240, 196, 70), "eraser": (232, 130, 150), "ear_in": (220, 150, 160),
}
SUSU = {
    "body": (126, 98, 62), "body_dark": (92, 70, 44), "band": (74, 56, 36), "head": (140, 110, 72),
    "claw": (60, 50, 66), "goggle": (176, 128, 60), "glass": (150, 210, 230), "eye": (36, 28, 30),
    "eye_light": (255, 255, 255), "blush": (230, 150, 150), "antenna": (60, 44, 30),
}


def tools(d, z):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    L = lambda pts, w, c: d.line([(x * z, y * z) for x, y in pts], fill=c, width=max(1, int(w * z)))
    return E, P, L


def draw_popo(d, z, col):
    E, P, L = tools(d, z)
    E(5.6, 20.0, 3.4, 4.6, col("fur"))                                  # 굵은 줄무늬 꼬리(몸 뒤로)
    E(4.0, 16.4, 2.6, 2.8, col("fur"))
    for y in (17.6, 21.0):
        L([(2.6, y), (8.4, y + 0.6)], 0.9, col("tail_dark"))
    E(15.0, 23.6, 6.2, 6.4, col("fur"))                                 # 몸
    E(16.6, 24.4, 3.4, 4.4, col("white"))                               # 배
    E(12.8, 29.6, 2.4, 1.1, col("foot"))                                # 발
    E(18.6, 29.6, 2.4, 1.1, col("foot"))
    E(19.6, 22.4, 1.6, 2.8, col("fur_dark"))                            # 앞발(팔)
    E(12.4, 8.2, 2.4, 2.4, col("fur_dark"))                             # 먼 귀
    E(20.8, 7.8, 2.4, 2.4, col("fur"))                                  # 가까운 귀
    E(20.8, 8.2, 1.2, 1.2, col("ear_in"))
    E(17.0, 13.2, 6.6, 5.6, col("fur"))                                 # 머리
    E(18.8, 12.8, 5.2, 2.0, col("mask"))                                # 검은 가면
    E(21.6, 15.4, 2.8, 2.0, col("white"))                               # 주둥이
    E(23.8, 14.6, 0.9, 0.7, col("nose"))
    E(20.2, 12.6, 0.6, 0.6, col("eye_light"))                           # 가면 속 눈빛
    L([(18.6, 5.4), (25.2, 8.8)], 1.3, col("pencil"))                   # 귀 뒤 연필
    E(18.6, 5.4, 0.8, 0.8, col("eraser"))


def draw_susu(d, z, col):
    E, P, L = tools(d, z)
    E(6.6, 25.6, 3.4, 2.8, col("body_dark"))                            # 뒷다리
    E(13.0, 21.0, 7.8, 6.8, col("body"))                                # 둥근 등(몸통)
    for x in (8.4, 11.0, 13.6):
        L([(x, 16.0), (x - 1.2, 26.6)], 0.6, col("band"))
    E(11.6, 17.6, 3.6, 2.0, col("head"))                                # 등의 빛
    L([(23.8, 19.2), (30.0, 13.6)], 0.5, col("antenna"))                # 더듬이 둘
    L([(23.0, 19.0), (27.4, 12.0)], 0.5, col("antenna"))
    E(22.4, 22.2, 4.2, 3.8, col("head"))                                # 머리
    P([(21.6, 25.6), (28.6, 26.8), (27.4, 30.0), (21.0, 29.6)], col("claw"))   # 삽 같은 앞발
    P([(18.0, 26.0), (22.0, 27.0), (21.2, 30.0), (17.4, 29.6)], col("claw"))
    for cx in (12.6, 16.8):                                             # 머리 위로 올린 고글
        E(cx, 13.6, 2.0, 2.0, col("goggle"))
        E(cx, 13.6, 1.2, 1.2, col("glass"))
    E(23.8, 21.6, 1.0, 1.1, col("eye"))
    E(23.5, 21.2, 0.35, 0.35, col("eye_light"))
    E(24.6, 23.6, 0.9, 0.45, col("blush"))


def shape(draw_fn):
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw_fn(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    shapes = {}
    for name, fn, colors in (("popo", draw_popo, POPO), ("susu", draw_susu, SUSU)):
        m = shapes[name] = shape(fn)
        sil = np.full((CELL, CELL, 3), 255, np.uint8)
        sil[m] = (30, 24, 34)
        Image.fromarray(sil).resize((CELL * 16, CELL * 16), Image.NEAREST).save(OUT / f"ref_{name}_silhouette.png")
        z = 24
        big = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
        fn(ImageDraw.Draw(big), z, lambda k: colors[k])
        big.save(OUT / f"ref_{name}_blocking.png")
        ys, xs = np.nonzero(m)
        print(f"{name}: 키 {ys.max() - ys.min() + 1}px, 폭 {xs.max() - xs.min() + 1}px (Naïl 몸 16px)")
    # 크기 기준: 게임 크기 Naïl 서기(24칸, 발 22) 옆에 포포·수수(32칸, 발 30), 발 줄을 맞춰 8배.
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    board = Image.new("RGBA", (4 + 24 + 6 + CELL + 6 + CELL + 4, CELL + 8), (255, 0, 255, 255))
    board.alpha_composite(nail, (4, 4 + 30 - 22))
    for i, name in enumerate(("popo", "susu")):
        a = np.zeros((CELL, CELL, 4), np.uint8)
        a[shapes[name]] = (30, 24, 34, 255)
        board.alpha_composite(Image.fromarray(a, "RGBA"), (4 + 24 + 6 + i * (CELL + 6), 4))
    board.resize((board.width * 8, board.height * 8), Image.NEAREST).save(OUT / "ref_neighbors_scale.png")


if __name__ == "__main__":
    main()
