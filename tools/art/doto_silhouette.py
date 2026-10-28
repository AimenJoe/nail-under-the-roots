"""도토(돌아온 청설모) 실루엣·색 안내(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림 둘을 만든다.

실행: python3 tools/art/doto_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_doto_silhouette.png: 32칸(모모와 같은 칸, 몸은 Naïl·모모와 같은 크기 + 꼬리) 실루엣 한 색을 16배로.
  - ref_doto_blocking.png: 같은 모양을 크게 칠한 색 안내(초상화 `doto_normal` 색: 붉은 갈색 털·크림 배·분홍 귓속·
    도토리 깍정이 모자·큰 반짝 눈).

왜: 청설모는 몸보다 큰 꼬리가 등 뒤로 S자로 말려 올라가는 것이 첫째 표지다 — 꼬리가 몸 윤곽 밖으로 크게 나와야
들쥐·다람쥐와 갈린다. 둘째는 뾰족한 귀, 셋째는 머리 위 도토리 모자(도토만의 표지). 칸이 32인 것은 꼬리 때문이다
(모모 차기·쓰러짐이 24칸에서 잘렸다).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "sources" / "ai" / "sprites" / "ref"
CELL = 32

COLORS = {
    "fur": (176, 82, 46), "fur_dark": (120, 52, 34), "fur_light": (214, 120, 70), "cream": (240, 226, 200),
    "ear_in": (226, 150, 150), "nose": (60, 34, 36), "eye": (40, 26, 30), "eye_light": (255, 255, 255),
    "blush": (236, 140, 130), "cap": (150, 104, 60), "cap_dark": (104, 70, 40), "nut": (196, 136, 74),
    "foot": (150, 70, 40),
}


def draw(d, z, col):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    # 꼬리: 엉덩이에서 뒤로 나가 등 위로 S자로 말려 올라간다(머리보다 높이, 몸보다 크게).
    E(8.0, 22.0, 4.2, 5.0, col("fur_dark"))
    E(6.5, 15.0, 4.8, 5.4, col("fur"))
    E(8.2, 8.6, 4.6, 4.6, col("fur"))
    E(11.6, 5.6, 3.0, 2.8, col("fur_light"))
    E(16.0, 22.0, 5.6, 6.0, col("fur"))                            # 몸(앉은 자세, 조금 선)
    E(17.4, 23.6, 3.0, 4.0, col("cream"))                          # 배
    E(19.4, 14.8, 4.4, 4.0, col("fur"))                            # 머리
    P([(16.2, 12.2), (16.6, 7.6), (18.4, 11.4)], col("fur"))       # 먼 귀(뾰족)
    P([(19.6, 11.4), (21.4, 7.0), (22.4, 11.8)], col("fur"))       # 가까운 귀(뾰족, 끝에 털)
    P([(20.4, 11.2), (21.4, 8.6), (21.8, 11.4)], col("ear_in"))
    E(19.2, 10.6, 2.9, 1.5, col("cap"))                            # 도토리 깍정이 모자
    E(19.2, 10.0, 2.4, 0.9, col("cap_dark"))
    P([(19.0, 8.9), (19.4, 7.9), (19.8, 8.9)], col("cap_dark"))    # 모자 꼭지
    E(23.0, 16.0, 1.6, 1.4, col("fur_light"))                      # 주둥이
    E(24.2, 15.6, 0.6, 0.5, col("nose"))
    E(21.0, 14.2, 1.2, 1.4, col("eye"))                            # 큰 눈 하나(3/4)
    E(20.7, 13.7, 0.4, 0.4, col("eye_light"))
    E(20.6, 16.6, 1.0, 0.6, col("blush"))
    E(21.4, 21.0, 1.4, 1.2, col("fur"))                            # 앞발(가슴 앞에 모은다)
    E(22.0, 21.6, 1.1, 1.0, col("nut"))                            # 쥔 도토리
    E(14.0, 28.6, 3.6, 1.4, col("foot"))                           # 큰 뒷발
    E(19.6, 28.8, 1.8, 1.1, col("foot"))


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw(ImageDraw.Draw(im), z, lambda k: 255)
    m = np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110
    sil = np.full((CELL, CELL, 3), 255, np.uint8)
    sil[m] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 16, CELL * 16), Image.NEAREST).save(OUT / "ref_doto_silhouette.png")
    z = 24
    big = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
    draw(ImageDraw.Draw(big), z, lambda k: COLORS[k])
    big.save(OUT / "ref_doto_blocking.png")


if __name__ == "__main__":
    main()
