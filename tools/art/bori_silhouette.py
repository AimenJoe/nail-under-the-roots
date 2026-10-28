"""보리(들쥐 대장 → 돌아온 보리) 실루엣·색 안내(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림 셋을 만든다.

실행: python3 tools/art/bori_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_bori_silhouette.png: 32칸(큰 보스, 그림 규칙 0203 — 키 26~28px) 실루엣 한 색을 16배로.
  - ref_bori_blocking.png: 같은 모양을 크게 칠한 색 안내(초상화 `bori_normal` 색: 갈색 털·크림 주둥이·분홍 귓속·
    이마의 꿰맨 X 흉터·졸린 눈).
  - ref_bori_scale.png: 게임 크기 그대로 Naïl(서기, 몸 16px)과 보리 실루엣을 나란히, 마젠타 바탕에 8배 —
    "점 크기·몸 크기 기준"(모모·도토 때 게임 크기 그림을 크게 다시 그리게 한 것이 가장 잘 됐다, 0206).

왜: 보리는 큰 보스라 플레이어 셋(몸 16px)보다 한눈에 커야 한다(사장 "보리랑 같은 크기면 안 되는데"). 들쥐 대장의
첫째 표지는 머리 위로 크게 나온 둥근 귀 둘, 둘째는 목 없이 통통한 배(햄스터처럼), 셋째는 등 뒤의 굵은 꼬리.
이마의 X 흉터는 32칸에서 두세 점이지만 초상화와 이어 주는 표지라 남긴다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL = 32

# 초상화 bori_normal을 14색으로 묶은 값에서.
COLORS = {
    "fur": (159, 125, 90), "fur_dark": (124, 78, 64), "fur_light": (186, 152, 112), "cream": (215, 197, 166),
    "ear_in": (207, 145, 127), "nose": (214, 140, 140), "eye": (66, 33, 39), "eye_light": (255, 255, 255),
    "scar": (91, 51, 50), "blush": (214, 150, 140), "hand": (168, 116, 96), "foot": (136, 94, 74),
    "tail": (168, 116, 96),
}


# 그림 규칙(0203) 큰 보스 키 26~28px: 아래 좌표는 발 자리(16, 30)를 축으로 이만큼 줄여 찍는다.
K = 0.92


def draw(d, z, col):
    X = lambda x: 16 + (x - 16) * K
    Y = lambda y: 30 + (y - 30) * K
    E = lambda cx, cy, rx, ry, c: d.ellipse(
        [(X(cx) - rx * K) * z, (Y(cy) - ry * K) * z, (X(cx) + rx * K) * z, (Y(cy) + ry * K) * z], fill=c)
    L = lambda pts, w, c: d.line([(X(x) * z, Y(y) * z) for x, y in pts], fill=c, width=max(1, int(w * K * z)))
    # 굵은 꼬리: 엉덩이에서 뒤로 나가 끝이 살짝 올라간다(몸 윤곽 밖으로).
    E(6.2, 26.0, 2.6, 1.7, col("tail"))
    E(3.6, 24.8, 1.9, 1.3, col("tail"))
    E(1.9, 23.2, 1.2, 1.0, col("tail"))
    E(15.0, 21.0, 8.6, 8.2, col("fur"))                         # 통통한 몸(목 없이)
    E(18.0, 22.6, 4.6, 5.6, col("cream"))                       # 배
    E(19.2, 12.0, 6.8, 5.8, col("fur"))                         # 큰 머리(볼이 통통)
    E(13.6, 6.4, 3.4, 3.4, col("fur_dark"))                     # 먼 귀(둥글고 크다)
    E(13.8, 6.4, 2.2, 2.2, col("ear_in"))
    E(21.4, 5.2, 3.6, 3.6, col("fur"))                          # 가까운 귀
    E(21.6, 5.3, 2.4, 2.4, col("ear_in"))
    E(24.4, 13.8, 2.8, 2.2, col("cream"))                       # 주둥이
    E(26.8, 13.2, 1.0, 0.9, col("nose"))
    E(21.6, 11.6, 1.1, 0.8, col("eye"))                         # 졸린 눈(가로로 길다)
    E(21.2, 11.3, 0.35, 0.3, col("eye_light"))
    E(21.4, 14.6, 1.1, 0.6, col("blush"))
    L([(16.6, 7.6), (18.6, 9.6)], 0.5, col("scar"))             # 이마의 꿰맨 X 흉터
    L([(18.6, 7.6), (16.6, 9.6)], 0.5, col("scar"))
    E(22.0, 19.6, 1.5, 1.3, col("hand"))                        # 가슴 앞에 모은 작은 손
    E(12.0, 29.0, 3.4, 1.4, col("foot"))                        # 발
    E(19.6, 29.2, 2.8, 1.2, col("foot"))


def silhouette():
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    m = silhouette()
    sil = np.full((CELL, CELL, 3), 255, np.uint8)
    sil[m] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 16, CELL * 16), Image.NEAREST).save(OUT / "ref_bori_silhouette.png")
    z = 24
    big = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
    draw(ImageDraw.Draw(big), z, lambda k: COLORS[k])
    big.save(OUT / "ref_bori_blocking.png")
    # 크기 기준: 게임 크기 Naïl 서기(24칸, 발 22)와 보리 실루엣(32칸, 발 30)을 발 줄을 맞춰 나란히, 8배.
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    board = Image.new("RGBA", (24 + 8 + CELL + 8, CELL + 8), (255, 0, 255, 255))
    board.alpha_composite(nail, (4, 4 + 30 - 22))
    shape = np.zeros((CELL, CELL, 4), np.uint8)
    shape[m] = (30, 24, 34, 255)
    board.alpha_composite(Image.fromarray(shape, "RGBA"), (4 + 24 + 8, 4))
    board.resize((board.width * 8, board.height * 8), Image.NEAREST).save(OUT / "ref_bori_scale.png")
    ys, xs = np.nonzero(m)
    print(f"보리 실루엣: 키 {ys.max() - ys.min() + 1}px, 폭 {xs.max() - xs.min() + 1}px (Naïl 몸 16px)")


if __name__ == "__main__":
    main()
