"""보리(들쥐 대장 → 돌아온 보리) 스프라이트 시트(40×40 칸, 발바닥 38, 0206 C4). 큰 보스라 몸 키가 플레이어 셋(16px)보다
한눈에 크다(그림 규칙 0203: 키 26~28px). 칸은 가장 길게 뻗는 돌진·쓰러짐이 들어가게 40.

실행: python3 tools/art/bori_ai.py [출력 경로]   (기본 assets/art/chief.png)

원본: `sources/ai/sprites/bori_v1.png`(✦ 지움). 크기 기준 그림(`bori_silhouette.py`의 `ref_bori_scale` — 게임 크기 Naïl 옆
보리 실루엣 8배)을 붙여 받았다. 블록 약 7.9px, 고른 격자가 아니라 도토처럼 원본 외곽선을 걷고(`strip`) 서기 칸의 키를
줄인다(`body_h` = 외곽선 안 키 25 → 외곽선까지 27).

칸 배치(줄마다 같은 뜻):
  칸 0·1 서기, 2·3 걷기, 4 웅크림(돌진 예고), 5 돌진, 6 맞음, 7 부르기(부하를 부른다), 8 기뻐하기, 9 쓰러짐
  줄 0 돌아온 보리(쉼터·마을 이웃) · 줄 1 물든 대장(튜토리얼 보스: 회보라 털·붉은 눈·목에 감긴 보라 실 —
  줄 0의 색을 바꾸고 실은 코드로 그린다, 초상화 `chief_glare`)
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍)
"""
import colorsys
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import walk_ai  # noqa: E402
from sprite_ai import OUTLINE, Source, Spec, convert, lum, ramp, seat, sheet4, step_cycle  # noqa: E402

CELL, FOOT = 40, 38
NAMES = ["idle_a", "idle_b", "walk_a", "walk_b", "crouch", "charge", "hurt", "call", "happy", "down"]
BOXES = [
    (21, 34, 175, 240), (230, 38, 376, 240), (430, 29, 585, 240), (631, 29, 790, 240), (827, 64, 1003, 240),
    (17, 316, 210, 508), (234, 303, 397, 513), (430, 294, 594, 513), (627, 303, 803, 513), (819, 345, 999, 517),
]
SPEC = Spec(
    sources=[
        Source(
            HERE / "sources" / "ai" / "sprites" / "bori_v1.png",
            dict(zip(NAMES, BOXES)),
            groups=dict.fromkeys(NAMES, "idle_a"),
            tidy=0.15,
            strip=8,
        )
    ],
    palette_ref=(0, "idle_a"),
    body_h=25,
)
ORDER = NAMES + walk_ai.NAMES     # 칸 10~13 = 걷기 네 칸(`bori_walk_v1.png`, 0229)

# 물든 대장(0032): 초상화 chief_glare의 회보라 털, 붉게 빛나는 눈, 머리·목의 보라 실. 귓속 분홍·흉터는 그대로.
TAINT_FUR = [(52, 46, 66), (82, 76, 98), (112, 106, 128), (146, 140, 160), (196, 190, 206)]
TAINT_OUTLINE = (24, 16, 34)
EYE_RED = [(110, 16, 22), (214, 40, 40), (255, 140, 120)]
THREAD = [(96, 52, 160), (168, 110, 230)]




def taint(cell):
    """돌아온 보리 칸 → 물든 대장 칸. 눈 = 빛점(흰 점) 둘레 2칸 안의 어두운 점. 분홍(귓속·코·손)·흉터는 그대로."""
    out = cell.copy()
    rgb = cell[..., :3].astype(int)
    on = cell[..., 3] > 0
    white = on & (rgb.sum(-1) > 700)
    near = white.copy()
    for _ in range(2):
        g = near.copy()
        g[1:] |= near[:-1]
        g[:-1] |= near[1:]
        g[:, 1:] |= near[:, :-1]
        g[:, :-1] |= near[:, 1:]
        near = g
    for y, x in zip(*np.nonzero(on)):
        key = tuple(int(v) for v in rgb[y, x])
        L = lum(key) / 255
        if key == OUTLINE:
            c = TAINT_OUTLINE
        elif white[y, x]:
            c = key
        elif near[y, x] and L < 0.3:                                     # 눈
            c = ramp(EYE_RED, L * 3)
        else:
            h, l, s = colorsys.rgb_to_hls(*(v / 255 for v in key))
            if s > 0.35 and (h > 0.9 or h < 0.04) and L > 0.45:         # 분홍 귓속·코·손
                c = key
            else:                                                        # 털·배·흉터
                c = ramp(TAINT_FUR, L * 1.1)
        out[y, x, :3] = c
    return out


def threads(cell):
    """물든 대장의 보라 실(초상화 chief_glare의 목에 감긴 실): 몸 위 45% 줄, 등 쪽 털 위에만 한 가닥.
    얼굴(앞쪽 40%)·배(밝은 털)·외곽선은 건드리지 않는다 — 처음엔 두 줄을 몸 전체에 그었더니 눈을 가로질렀다."""
    out = cell.copy()
    on = cell[..., 3] > 0
    ys = np.nonzero(on.any(1))[0]
    xs_all = np.nonzero(on.any(0))[0]
    top, bot = int(ys.min()), int(ys.max())
    x0, x1 = int(xs_all.min()), int(xs_all.max())
    face = x0 + (x1 - x0) * 0.6
    y = top + int((bot - top + 1) * 0.45)
    for x in range(x0, x1 + 1):
        if x >= face or not on[y, x]:
            continue
        key = tuple(int(v) for v in cell[y, x, :3])
        if key == TAINT_OUTLINE or lum(key) > 150:
            continue
        out[y, x, :3] = THREAD[(x // 2) % 2]
    return out


def build():
    spr = convert(SPEC)
    cells = {k: seat(spr[k], CELL, FOOT) for k in NAMES}
    cells.update(walk_ai.cells("bori", SPEC.sources[0], CELL, FOOT))
    # 걷기는 첫 칸 하나에서 발만 딛게(0241): 따로 받은 걷기 넷(`bori_walk_v1.png`)은 넷째 칸만 키·몸이 커서 걸음마다 몸이 튀었다.
    steps = step_cycle(cells[walk_ai.NAMES[0]], FOOT - 1, reach=2)
    cells.update(zip(walk_ai.NAMES, steps))
    return sheet4([(cells[k], threads(taint(cells[k]))) for k in ORDER])


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/chief.png"
    Image.fromarray(build(), "RGBA").save(out)
