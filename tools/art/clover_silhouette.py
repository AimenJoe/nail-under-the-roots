"""토끼풀(어미 굴토끼, 굴토끼 옛 굴의 주인) 실루엣·색 안내·크기 기준(0206과 같은 길: 실루엣 먼저). 나노바나나에 붙일 참고 그림 셋.

실행: python3 tools/art/clover_silhouette.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_clover_silhouette.png: 40칸 서기 + 덮치기 실루엣, 16배.
  - ref_clover_blocking.png: 같은 모양을 크게 칠한 색 안내(캐릭터 시트 `sheets/clover.png`·초상화 `clover_normal` 색: 회갈색 털·
    크림 가슴 털·분홍 귓속·귀 밑동의 분홍 토끼풀 꽃·큰 솜꼬리·까만 눈·볼 홍조).
  - ref_clover_scale.png: 게임 크기 Naïl(몸 16px)·모모(키 23px) 옆에 토끼풀 서기·덮치기 실루엣, 마젠타 바탕 8배(점 크기·몸 크기 기준).

왜: 토끼풀은 모모의 어미라 같은 회갈색 굴토끼다 — 모모와 갈리는 것은 크기와 무게(0165 "모모보다 크고 무겁다"), 그리고
귀: 한 귀는 등 뒤로 눕고(경계) 한 귀는 얼굴 옆으로 처진다(캐릭터 시트의 처진 귀). 셋째 표지는 귀 밑동의 토끼풀 꽃(이름).
칸은 40(대장·도토와 같다): 서기 키 27px로 모모(23)보다 한눈에 크고, 덮치기(뒷발을 뒤로 뻗고 앞발을 내민다)가 36px로 길다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL = 40

COLORS = {
    "fur": (122, 108, 100), "fur_dark": (88, 76, 72), "cream": (232, 220, 200), "ear_in": (214, 140, 150),
    "eye": (36, 26, 30), "eye_light": (255, 255, 255), "blush": (230, 150, 150), "nose": (196, 120, 130),
    "flower": (222, 120, 160), "flower_dark": (176, 80, 120), "leaf": (110, 150, 80),
}


def tools(d, z):
    E = lambda cx, cy, rx, ry, c: d.ellipse([(cx - rx) * z, (cy - ry) * z, (cx + rx) * z, (cy + ry) * z], fill=c)
    P = lambda pts, c: d.polygon([(x * z, y * z) for x, y in pts], fill=c)
    return E, P


def draw_stand(d, z, col):
    E, P = tools(d, z)
    P([(25.0, 17.4), (19.6, 11.2), (16.6, 9.8), (15.4, 11.2), (17.0, 14.0), (21.0, 19.8)], col("fur_dark"))   # 먼 귀: 등 뒤로 눕힘(경계), 넓은 잎 모양
    E(7.4, 30.6, 3.4, 3.2, col("cream"))                                           # 솜꼬리
    E(15.6, 30.0, 9.2, 7.6, col("fur"))                                            # 무거운 몸
    E(12.6, 37.0, 5.4, 1.6, col("fur_dark"))                                       # 큰 뒷발
    E(24.6, 37.0, 2.4, 1.4, col("fur_dark"))                                       # 앞발
    E(23.2, 30.2, 3.6, 5.2, col("cream"))                                          # 가슴 털
    E(26.2, 21.6, 6.0, 5.4, col("fur"))                                            # 머리
    E(29.6, 24.0, 2.6, 2.0, col("cream"))                                          # 주둥이
    E(31.6, 23.0, 0.8, 0.6, col("nose"))
    P([(24.4, 17.0), (20.6, 19.4), (20.2, 27.6), (22.8, 28.6), (25.2, 20.4)], col("fur"))   # 가까운 귀: 얼굴 옆으로 처짐
    P([(23.6, 19.2), (21.6, 20.4), (21.4, 26.4), (22.6, 26.8), (24.0, 20.6)], col("ear_in"))
    E(24.6, 16.8, 1.8, 1.6, col("flower"))                                         # 귀 밑동 토끼풀 꽃
    E(24.2, 17.2, 0.8, 0.7, col("flower_dark"))
    E(22.6, 17.8, 1.1, 0.7, col("leaf"))
    E(28.2, 20.8, 1.2, 1.3, col("eye"))                                            # 까만 눈 + 빛점
    E(27.9, 20.3, 0.4, 0.4, col("eye_light"))
    E(28.8, 23.4, 1.0, 0.5, col("blush"))


def draw_pounce(d, z, col):
    E, P = tools(d, z)
    P([(9.4, 29.6), (1.6, 32.6), (2.2, 35.2), (11.0, 33.6)], col("fur_dark"))     # 뒤로 뻗은 큰 뒷발
    E(7.6, 26.6, 3.0, 2.8, col("cream"))                                           # 솜꼬리
    P([(29.4, 20.2), (21.6, 15.8), (18.0, 15.8), (17.6, 17.6), (21.0, 19.4), (27.0, 22.6)], col("fur_dark"))   # 먼 귀: 뒤로 날린다
    E(19.0, 27.4, 11.6, 5.6, col("fur"))                                           # 길게 뻗은 몸
    E(25.0, 30.0, 4.0, 2.6, col("cream"))
    P([(27.0, 30.0), (36.8, 32.0), (36.6, 34.6), (26.4, 33.4)], col("fur_dark"))  # 내민 앞발
    E(31.2, 24.4, 5.4, 4.8, col("fur"))                                            # 머리
    E(34.4, 26.4, 2.4, 1.8, col("cream"))
    E(36.4, 25.6, 0.8, 0.6, col("nose"))
    P([(29.4, 21.0), (22.4, 21.2), (19.8, 22.6), (21.0, 24.6), (28.4, 23.8)], col("fur"))       # 가까운 귀(처진 귀가 뒤로 날린다)
    E(29.6, 20.6, 1.6, 1.4, col("flower"))
    E(33.0, 23.6, 1.1, 1.2, col("eye"))
    E(32.7, 23.1, 0.4, 0.4, col("eye_light"))


def shape(draw_fn):
    z = 8
    im = Image.new("L", (CELL * z, CELL * z), 0)
    draw_fn(ImageDraw.Draw(im), z, lambda k: 255)
    return np.asarray(im.resize((CELL, CELL), Image.BOX)) > 110


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    stand, pounce = shape(draw_stand), shape(draw_pounce)
    sil = np.full((CELL, CELL * 2, 3), 255, np.uint8)
    sil[:, :CELL][stand] = (30, 24, 34)
    sil[:, CELL:][pounce] = (30, 24, 34)
    Image.fromarray(sil).resize((CELL * 32, CELL * 16), Image.NEAREST).save(OUT / "ref_clover_silhouette.png")
    z = 24
    big = Image.new("RGB", (CELL * z * 2, CELL * z), (255, 255, 255))
    d = ImageDraw.Draw(big)
    draw_stand(d, z, lambda k: COLORS[k])
    off = Image.new("RGB", (CELL * z, CELL * z), (255, 255, 255))
    draw_pounce(ImageDraw.Draw(off), z, lambda k: COLORS[k])
    big.paste(off, (CELL * z, 0))
    big.save(OUT / "ref_clover_blocking.png")
    # 크기 기준: 게임 크기 Naïl 서기(24칸, 발 22)·모모 서기(32칸, 발 30) 옆에 토끼풀 서기·덮치기(40칸, 발 38), 발 줄을 맞춰 8배.
    art = HERE.parents[1] / "assets" / "art"
    nail = Image.open(art / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    momo = Image.open(art / "momo.png").convert("RGBA").crop((0, 0, 32, 32))
    board = Image.new("RGBA", (4 + 24 + 4 + 32 + 6 + CELL + 6 + CELL + 4, CELL + 8), (255, 0, 255, 255))
    board.alpha_composite(nail, (4, 4 + 38 - 22))
    board.alpha_composite(momo, (4 + 24 + 4, 4 + 38 - 30))
    for i, m in enumerate((stand, pounce)):
        a = np.zeros((CELL, CELL, 4), np.uint8)
        a[m] = (30, 24, 34, 255)
        board.alpha_composite(Image.fromarray(a, "RGBA"), (4 + 24 + 4 + 32 + 6 + i * (CELL + 6), 4))
    board.resize((board.width * 8, board.height * 8), Image.NEAREST).save(OUT / "ref_clover_scale.png")
    for name, m in (("서기", stand), ("덮치기", pounce)):
        ys, xs = np.nonzero(m)
        print(f"토끼풀 {name}: 키 {ys.max() - ys.min() + 1}px, 폭 {xs.max() - xs.min() + 1}px (Naïl 몸 16px, 모모 키 23px)")


if __name__ == "__main__":
    main()
