"""모모(돌아온 굴토끼) 실루엣·색 안내(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림 둘을 만든다.

실행: python3 tools/art/momo_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_momo_silhouette.png: 24칸(Naïl과 같은 칸, 몸 16px + 귀) 실루엣 한 색을 16배로.
  - ref_momo_blocking.png: 같은 모양을 크게 칠한 색 안내(초상화 `momo_normal` 색: 회갈색 털·크림 가슴 털·분홍 귓속·
    귀 밑 분홍 꽃·큰 반짝 눈).

왜: 모모는 16칸 옛 그림이라 새 Naïl(24칸, 큰 반짝 눈)과 나란히 서면 따로 논다. 토끼는 긴 귀 + 솜꼬리 + 큰 뒷발이
실루엣을 정한다 — 셋이 다 윤곽 밖으로 나오게(Naïl 때 손이 몸 위에 겹치면 실루엣이 공이 됐다).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "sources" / "ai" / "sprites" / "ref"
CELL = 24

COLORS = {
    "fur": (150, 128, 108), "fur_dark": (108, 88, 76), "fur_light": (186, 166, 144), "cream": (236, 222, 198),
    "ear_in": (232, 156, 168), "nose": (232, 140, 156), "eye": (40, 34, 64), "eye_light": (255, 255, 255),
    "blush": (236, 150, 150), "flower": (246, 170, 196), "flower_mid": (255, 222, 120), "leaf": (120, 170, 96),
    "foot": (214, 196, 172),
}


def draw(d, z, col):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    # 귀 둘: 먼 귀는 조금 뒤로 눕고, 가까운 귀는 곧게. 둘 다 머리 위로 길게(토끼의 첫째 표지).
    P([(12.6, 8.5), (11.0, 1.6), (12.4, 0.6), (14.6, 7.8)], col("fur_dark"))
    P([(15.0, 8.0), (15.6, 0.4), (17.4, 0.6), (17.4, 8.2)], col("fur"))
    P([(15.9, 7.0), (16.2, 1.6), (16.9, 1.8), (16.9, 7.0)], col("ear_in"))
    E(4.2, 15.0, 2.4, 2.3, col("cream"))                           # 솜꼬리(뒤로 튀어나온다)
    E(10.5, 16.0, 6.6, 5.2, col("fur"))                            # 둥근 몸
    E(16.6, 11.0, 4.4, 4.0, col("fur"))                            # 머리
    E(20.4, 12.2, 1.9, 1.6, col("fur_light"))                      # 주둥이
    E(21.6, 11.7, 0.8, 0.6, col("nose"))
    E(9.0, 20.6, 4.2, 1.5, col("foot"))                            # 큰 뒷발(앞으로 뻗는다)
    E(16.0, 18.4, 1.5, 2.6, col("fur"))                            # 앞다리(가슴 털 아래로 몸에 붙는다)
    E(16.8, 20.6, 1.6, 1.1, col("foot"))                           # 작은 앞발
    E(15.2, 15.2, 2.6, 3.0, col("cream"))                          # 가슴 털(앞다리 위)
    E(17.6, 10.4, 1.2, 1.4, col("eye"))                            # 큰 눈 하나(3/4)
    E(17.3, 9.9, 0.4, 0.4, col("eye_light"))
    E(17.0, 12.8, 1.0, 0.6, col("blush"))
    for dx, dy in ((0, -0.9), (0.9, 0), (0, 0.9), (-0.9, 0)):      # 귀 밑 꽃(돌아온 모모, 0046)
        E(14.2 + dx, 7.6 + dy, 0.8, 0.8, col("flower"))
    E(14.2, 7.6, 0.55, 0.55, col("flower_mid"))


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw(ImageDraw.Draw(im), z, lambda k: 255)
    m = np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110
    sil = np.full((CELL, CELL, 3), 255, np.uint8)
    sil[m] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 16, CELL * 16), Image.NEAREST).save(OUT / "ref_momo_silhouette.png")
    z = 32
    big = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
    draw(ImageDraw.Draw(big), z, lambda k: COLORS[k])
    big.save(OUT / "ref_momo_blocking.png")


if __name__ == "__main__":
    main()
