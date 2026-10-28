"""콩이(아르마딜로, 셋째 길의 문지기) 실루엣·색 안내·크기 기준(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림 셋.

실행: python3 tools/art/kongi_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_kongi_silhouette.png: 32칸(이름 있는 보스, 그림 규칙 0203 키 18~20px — 몸이 옆으로 길어 칸은 32) 서기 + 공 실루엣, 16배.
  - ref_kongi_blocking.png: 같은 모양을 크게 칠한 색 안내(초상화 `kongi_normal` 색: 분홍 살갗·올리브 띠 등딱지·뾰족한 귀·
    긴 주둥이·까만 눈·볼 홍조).
  - ref_kongi_scale.png: 게임 크기 Naïl(몸 16px) 옆에 콩이 서기·공 실루엣, 마젠타 바탕 8배(점 크기·몸 크기 기준).

왜: 콩이의 싸움은 "말리면 공"(0054: 웅크려 버티기·굴러 튕기기)이라 서 있는 몸과 공 두 모양이 다 읽혀야 한다. 서 있을 때의 표지는
등 위로 둥근 띠 등딱지, 머리 위로 솟은 뾰족한 귀, 앞으로 긴 주둥이. 공은 띠가 보이는 둥근 덩어리(귀·주둥이가 안 보인다).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL = 32

COLORS = {
    "skin": (208, 179, 166), "skin_dark": (169, 130, 125), "shell": (166, 158, 118), "shell_dark": (122, 114, 86),
    "band": (88, 78, 62), "ear_in": (222, 150, 150), "eye": (40, 26, 36), "eye_light": (255, 255, 255),
    "blush": (232, 140, 140), "nose": (132, 101, 102), "foot": (169, 130, 125),
}


def draw_stand(d, z, col):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    L = lambda pts, w, c: d.line([(x * z, y * z) for x, y in pts], fill=c, width=max(1, int(w * z)))
    P([(5.0, 25.0), (0.8, 27.6), (5.6, 27.2)], col("shell_dark"))       # 짧은 꼬리(띠)
    E(8.6, 28.6, 2.4, 1.3, col("foot"))                                 # 뒷발
    E(17.4, 28.8, 2.0, 1.2, col("foot"))                                # 앞발
    E(12.6, 22.2, 8.8, 6.6, col("shell"))                               # 둥근 등딱지(띠)
    for x in (7.6, 10.6, 13.6, 16.6):
        L([(x, 16.6), (x - 0.8, 27.6)], 0.45, col("band"))
    E(12.4, 26.8, 7.6, 1.6, col("skin_dark"))                           # 등딱지 아래 배
    E(21.0, 20.2, 4.2, 3.8, col("skin"))                                # 머리
    P([(22.4, 21.2), (29.6, 22.0), (29.2, 23.4), (22.0, 23.6)], col("skin"))   # 긴 주둥이
    E(29.4, 22.6, 0.8, 0.8, col("nose"))
    P([(18.6, 18.0), (18.2, 10.4), (21.0, 16.8)], col("skin_dark"))     # 먼 귀(뾰족)
    P([(20.6, 17.4), (22.4, 9.6), (23.6, 17.6)], col("skin"))           # 가까운 귀
    P([(21.4, 16.6), (22.4, 11.6), (22.9, 16.8)], col("ear_in"))
    E(22.6, 19.6, 0.9, 1.0, col("eye"))                                 # 까만 눈 + 빛점
    E(22.4, 19.2, 0.3, 0.3, col("eye_light"))
    E(22.8, 22.2, 0.9, 0.5, col("blush"))


def draw_ball(d, z, col, ox=0.0):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    L = lambda pts, w, c: d.line([(x * z, y * z) for x, y in pts], fill=c, width=max(1, int(w * z)))
    E(16.0 + ox, 22.0, 8.2, 7.8, col("shell"))
    for x in (11.0, 14.4, 17.8, 21.2):
        L([(x + ox, 15.0), (x + ox - 1.6, 29.0)], 0.45, col("band"))


def shape(draw_fn):
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw_fn(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    stand, ball = shape(draw_stand), shape(draw_ball)
    sil = np.full((CELL, CELL * 2, 3), 255, np.uint8)
    sil[:, :CELL][stand] = (30, 24, 34)
    sil[:, CELL:][ball] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 32, CELL * 16), Image.NEAREST).save(OUT / "ref_kongi_silhouette.png")
    z = 24
    big = Image.new("RGB", (CELL * z * 2, CELL * z), (255, 255, 255))
    d = ImageDraw.Draw(big)
    draw_stand(d, z, lambda k: COLORS[k])
    draw_ball(d, z, lambda k: COLORS[k], ox=CELL)
    big.save(OUT / "ref_kongi_blocking.png")
    # 크기 기준: 게임 크기 Naïl 서기(24칸, 발 22) 옆에 콩이 서기·공(32칸, 발 30), 발 줄을 맞춰 8배.
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    board = Image.new("RGBA", (4 + 24 + 6 + CELL + 6 + CELL + 4, CELL + 8), (255, 0, 255, 255))
    board.alpha_composite(nail, (4, 4 + 30 - 22))
    for i, m in enumerate((stand, ball)):
        a = np.zeros((CELL, CELL, 4), np.uint8)
        a[m] = (30, 24, 34, 255)
        board.alpha_composite(Image.fromarray(a, "RGBA"), (4 + 24 + 6 + i * (CELL + 6), 4))
    board.resize((board.width * 8, board.height * 8), Image.NEAREST).save(OUT / "ref_kongi_scale.png")
    for name, m in (("서기", stand), ("공", ball)):
        ys, xs = np.nonzero(m)
        print(f"콩이 {name}: 키 {ys.max() - ys.min() + 1}px, 폭 {xs.max() - xs.min() + 1}px (Naïl 몸 16px)")


if __name__ == "__main__":
    main()
