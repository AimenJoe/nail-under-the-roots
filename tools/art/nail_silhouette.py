"""Naïl 실루엣 시안 1(물방울 몸, 0206): 나노바나나에 붙일 참고 그림 둘을 만든다.

실행: python3 tools/art/nail_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_nail_silhouette.png: 32칸 실루엣(한 색)을 16배로. "이 모양 그대로"의 기준.
  - ref_nail_blocking.png: 같은 모양을 큰 그림으로 칠한 색 안내(털·주둥이·분홍 코·삽 손·크림 발톱·주황 목도리·눈).

왜(0206): 둥근 털뭉치는 한 색으로 칠하면 덩어리라 두더지로 안 읽혔다. 두더지 실루엣의 셋 —
앞으로 뾰족한 주둥이, 옆으로 편 삽 손(발톱 부채), 물방울 몸 — 을 32칸에서 먼저 정하고 그 모양으로 그리게 한다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "sources" / "ai" / "sprites" / "ref"

# 모양(32칸 좌표). 실루엣과 색 안내가 같은 수를 쓴다.
BODY = [(13, 19, 10, 9.5), (17, 14, 7, 6.5)]                       # 몸·머리 타원(cx, cy, rx, ry)
FRONT = [(22, 21), (29, 22), (30, 27), (27, 28), (25, 26), (22, 26)]  # 앞 삽 손(옆으로 편)
FRONT_CLAWS = [26, 28, 30]
BACK = [(6, 22), (1, 23), (0.5, 27), (3, 28), (6, 26)]             # 뒷손
BACK_CLAWS = [1.5, 3.5]
TUFTS = [(10, 9.5), (13, 8.5), (16, 8.7)]
FEET = [(10, 29, 2.5, 1.5), (17, 29, 2.5, 1.5)]


def draw(d, z, col):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    for cx, cy, rx, ry in FEET:
        E(cx, cy, rx, ry, col("foot"))
    for x, y in TUFTS:
        P([(x - 1.5, y + 2), (x, y - 1.5), (x + 1.5, y + 2)], col("fur_dark"))
    E(*BODY[0], col("fur"))
    E(*BODY[1], col("fur"))
    E(15, 12, 5.5, 4.5, col("fur_light"))                           # 왼쪽 위 빛
    # 목도리: 몸을 가로지르는 띠 + 매듭
    if col("scarf") != col("fur"):
        d.rectangle([4.2 * z, 20.5 * z, 22 * z, 23 * z], fill=col("scarf"))
        E(14, 24.5, 1.4, 1.4, col("scarf_dark"))
        P([(13, 24.5), (11.5, 28), (13.5, 27.5)], col("scarf_dark"))
        P([(15, 24.5), (16.5, 28), (14.5, 27.5)], col("scarf_dark"))
    # 주둥이는 끝이 둥근 털 주둥이(원뿔·삼각형은 부리·당근으로 읽힌다 — KB 도트 가이드).
    E(24.5, 15.3, 5.2, 2.9, col("muzzle"))
    E(22.0, 14.4, 3.0, 3.0, col("muzzle"))
    E(29.3, 16.2, 2.2, 2.2, col("nose"))                             # 분홍 코(주둥이 끝)
    P(FRONT, col("paw"))
    for x in FRONT_CLAWS:
        P([(x - 1, 26.5), (x, 30), (x + 0.6, 26.5)], col("claw"))
    P(BACK, col("paw"))
    for x in BACK_CLAWS:
        P([(x - 0.8, 27), (x - 0.4, 30), (x + 0.4, 27)], col("claw"))
    E(19.5, 13, 1.3, 1.6, col("eye"))                                # 눈(가까운 쪽 하나, 3/4)
    E(19.0, 12.4, 0.45, 0.45, col("eye_light"))
    E(17.5, 16.2, 1.0, 0.6, col("blush"))


COLORS = {
    "fur": (128, 92, 74), "fur_dark": (92, 62, 52), "fur_light": (166, 128, 102), "muzzle": (206, 172, 142),
    "nose": (242, 152, 164), "paw": (242, 152, 164), "claw": (248, 238, 216), "scarf": (208, 102, 46),
    "scarf_dark": (140, 54, 36), "eye": (26, 16, 32), "eye_light": (255, 255, 255), "blush": (236, 132, 132),
    "foot": (200, 104, 124),
}


def silhouette(z=8):
    im = Image.new("L", (32 * z, 32 * z), 0)
    draw(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((32, 32), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    m = silhouette()
    sil = np.full((32, 32, 3), 255, np.uint8)
    sil[m] = (30, 24, 34)
    Image.fromarray(sil).resize((512, 512), Image.NEAREST).save(OUT / "ref_nail_silhouette.png")
    z = 24
    big = Image.new("RGB", (32 * z, 32 * z), (255, 255, 255))
    draw(ImageDraw.Draw(big), z, lambda k: COLORS[k])
    big.save(OUT / "ref_nail_blocking.png")


if __name__ == "__main__":
    main()
