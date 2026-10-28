"""누룩 할매(오래된 균사, 가장 깊은 곳의 주인 · 마을 이웃) 실루엣·색 안내·크기 기준(0206과 같은 길: 실루엣 먼저). 나노바나나에
붙일 참고 그림 셋.

실행: python3 tools/art/halmae_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_halmae_silhouette.png: 96칸(가장 깊은 곳 크기) 실루엣, 8배.
  - ref_halmae_blocking.png: 같은 모양을 크게 칠한 색 안내(캐릭터 시트 `sheets/halmae.png`·일러스트 `illust/halmae_deep.png` 색:
    흰 솜 같은 균사 더미·연보라 그늘·갈색 버섯 갓 셋·크림 대·감은 눈·볼 홍조·작은 웃음·밑동의 보라 뿌리실).
  - ref_halmae_scale.png: 게임 크기 Naïl(몸 16px) 옆에 할매 실루엣, 마젠타 바탕 4배(점 크기·몸 크기 기준).

왜: 할매는 싸우지 않는다(0162) — 때릴 몸이 아니라 "실이 모이는 곳"으로 읽혀야 해서 화면 한가운데 크게(폭 약 84px, Naïl의
다섯 배) 앉는다. 표지는 셋: 구름처럼 울퉁불퉁한 솜 더미(매끈한 공이 아니다), 머리 위 버섯 갓 셋(가운데가 가장 크다),
밑동에서 바닥으로 퍼지는 보라 뿌리실(매듭으로 이어지는 실의 시작). 마을 이웃 크기(32칸)는 같은 원본을 따로 줄인다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL = 96

COLORS = {
    "puff": (236, 232, 246), "puff_shade": (184, 176, 212), "puff_light": (252, 250, 255), "face": (246, 244, 252),
    "cap": (150, 104, 60), "cap_dark": (110, 74, 42), "stem": (226, 216, 190), "eye": (60, 44, 70),
    "blush": (240, 150, 160), "root": (150, 110, 200), "root_light": (196, 170, 232),
}

PUFFS = [(14, 66, 9), (12, 76, 8), (20, 56, 9), (30, 48, 9), (44, 44, 10), (58, 46, 9), (70, 52, 9), (80, 62, 9),
         (84, 74, 8), (78, 84, 8), (64, 88, 8), (48, 89, 8), (32, 88, 8), (18, 84, 8)]


def draw(d, z, col):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    # 밑동의 보라 뿌리실: 바닥으로 퍼지는 갈래(앞쪽은 더미 아래로 나온다).
    for x0, x1, y1 in ((20, 1, 92), (32, 14, 95), (64, 82, 95), (76, 95, 91)):
        P([(x0 - 3, 82), (x0 + 3, 82), (x1 + 2, y1), (x1 - 2, y1)], col("root"))
        P([(x1 - 2, y1), (x1 + 2, y1), (x1 + (x1 - x0) * 0.2 + 1.5, y1 - 3), (x1 + (x1 - x0) * 0.2 - 1.5, y1 - 3)], col("root"))
    # 버섯 갓 셋(뒤에서 솟는다, 가운데가 가장 크다): 대 → 갓.
    for cx, top, rx, ry in ((30, 26, 9, 6), (48, 16, 11, 7), (65, 28, 8, 5.5)):
        P([(cx - 2.2, top + 2), (cx + 2.2, top + 2), (cx + 2.6, 50), (cx - 2.6, 50)], col("stem"))
        E(cx, top, rx, ry, col("cap"))
        E(cx, top + ry * 0.55, rx, ry * 0.35, col("cap_dark"))
    # 솜 더미: 가운데 큰 덩어리 + 둘레의 몽실몽실한 혹(매끈한 공이 아니다).
    E(48, 68, 38, 24, col("puff_shade"))
    for x, y, r in PUFFS:
        E(x, y, r, r * 0.9, col("puff_shade"))
    E(46, 64, 33, 20, col("puff"))
    for x, y, r in PUFFS[:9]:
        E(x - 1, y - 1, r - 2, (r - 2) * 0.9, col("puff"))
    E(36, 54, 14, 7, col("puff_light"))
    # 얼굴: 밝은 둥근 자리 + 감은 눈(아래로 굽은 선) + 볼 + 작은 웃음. 입은 웃음 한 줄만.
    E(50, 68, 15, 12, col("face"))
    for ex in (43, 57):
        P([(ex - 3.5, 65.6), (ex, 67.4), (ex + 3.5, 65.6), (ex + 3.5, 66.8), (ex, 68.6), (ex - 3.5, 66.8)], col("eye"))
        E(ex, 71.6, 2.4, 1.2, col("blush"))
    P([(47.6, 73.4), (50, 74.6), (52.4, 73.4), (52.4, 74.2), (50, 75.4), (47.6, 74.2)], col("eye"))


def shape():
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    m = shape()
    sil = np.full((CELL, CELL, 3), 255, np.uint8)
    sil[m] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 8, CELL * 8), Image.NEAREST).save(OUT / "ref_halmae_silhouette.png")
    z = 10
    big = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
    draw(ImageDraw.Draw(big), z, lambda k: COLORS[k])
    big.save(OUT / "ref_halmae_blocking.png")
    # 크기 기준: 게임 크기 Naïl 서기(24칸, 발 22) 옆에 할매(96칸, 밑동 95), 발 줄을 맞춰 4배.
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    board = Image.new("RGBA", (4 + 24 + 8 + CELL + 4, CELL + 8), (255, 0, 255, 255))
    board.alpha_composite(nail, (4, 4 + 95 - 22))
    a = np.zeros((CELL, CELL, 4), np.uint8)
    a[m] = (30, 24, 34, 255)
    board.alpha_composite(Image.fromarray(a, "RGBA"), (4 + 24 + 8, 4))
    board.resize((board.width * 4, board.height * 4), Image.NEAREST).save(OUT / "ref_halmae_scale.png")
    ys, xs = np.nonzero(m)
    print(f"할매: 키 {ys.max() - ys.min() + 1}px, 폭 {xs.max() - xs.min() + 1}px (Naïl 몸 16px)")


if __name__ == "__main__":
    main()
