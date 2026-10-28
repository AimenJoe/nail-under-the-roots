"""토끼풀(어미 굴토끼) 스프라이트 시트(40×40 칸, 발바닥 38, 0206 C4). 모모의 어미 — 모모(키 23px)보다 한눈에 크고 무겁다.
칸은 가장 길게 뻗는 덮치기가 들어가게 40(대장·도토와 같다).

실행: python3 tools/art/clover_ai.py [출력 경로]   (기본 assets/art/clover.png)

원본: `sources/ai/sprites/clover_v1.png`(✦ 지움). 크기 기준 그림(`clover_silhouette.py`의 `ref_clover_scale` — 게임 크기 Naïl·모모
옆 토끼풀 서기·덮치기 실루엣 8배)을 붙여 받았다. 블록 약 6.5px, 고른 격자가 아니라 원본 외곽선을 걷고(`strip`) 줄인다.
서기 칸의 외곽선 안 키 26(외곽선까지 28) — 27로 하면 덮치기(원본 폭 252px)가 40칸을 넘는다.

칸 배치(critters 토끼풀 줄과 같은 뜻, `art::BIG_SHEETS`로 `Art::critter`가 토끼풀 두 줄을 여기로 돌린다):
  칸 0·1 서기, 2·3 깡충(모으기·공중), 4 웅크림(발 구르기 예고·품기), 5 덮치기, 6 맞음, 7 쓰러짐
  줄 0 돌아온 토끼풀 · 줄 1 물든 토끼풀(보스: 보랏빛 모브 털·초록으로 빛나는 눈·보라 실 — 줄 0의 색을 바꾸고 실은 코드로,
  초상화 `clover_tainted_glare`. 귀 밑 토끼풀 꽃과 잎은 초상화처럼 그대로)
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍)
"""
import colorsys
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import OUTLINE, Source, Spec, convert, seat  # noqa: E402

CELL, FOOT = 40, 38
SRC = HERE / "sources" / "ai" / "sprites" / "clover_v1.png"
NAMES = ["idle_a", "idle_b", "hop_a", "hop_b", "crouch", "pounce", "hurt", "down"]
BOXES = [
    (21, 64, 235, 244), (277, 68, 491, 244), (533, 55, 751, 244), (780, 47, 1012, 218),
    (12, 315, 244, 500), (256, 324, 508, 478), (529, 298, 756, 495), (768, 367, 1007, 487),
]
SPEC = Spec(
    sources=[Source(SRC, dict(zip(NAMES, BOXES)), groups=dict.fromkeys(NAMES, "idle_a"), tidy=0.15, strip=6,
                   pink=True)],
    palette_ref=(0, "idle_a"),
    body_h=26,
)

# 물든 토끼풀(0165): 초상화 clover_tainted_glare — 보랏빛 모브 털, 옅은 라일락 가슴·주둥이, 초록으로 빛나는 눈(모모와 같은
# 가족 빛), 몸을 감은 보라 실. 꽃·잎(채도 높은 분홍·초록)은 그대로.
TAINT_FUR = [(58, 44, 64), (86, 68, 92), (114, 94, 120), (146, 126, 152), (190, 174, 196), (222, 210, 226)]
TAINT_OUTLINE = (26, 16, 34)
EYE_GREEN = [(40, 120, 40), (110, 230, 80), (210, 255, 170)]
THREAD = [(96, 52, 160), (168, 110, 230)]


def lum(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def ramp(colors, t):
    return colors[min(len(colors) - 1, max(0, int(t * len(colors))))]


def eyes(cell):
    """눈 = 몸 앞쪽 40%, 키의 20~70% 줄 안의 아주 어두운 점(외곽선 색은 뺀다 — 콩이처럼 눈은 제 어두운 색으로 온다) 가운데
    가장 큰 덩어리(8방향). 귀 밑동의 어두운 점이 따로 잡혀 물든 줄에서 초록 점이 하나 더 생겼다."""
    on = cell[..., 3] > 0
    xs = np.nonzero(on.any(0))[0]
    ys = np.nonzero(on.any(1))[0]
    front = xs.min() + (xs.max() - xs.min()) * 0.6
    y0, y1 = ys.min() + (ys.max() - ys.min()) * 0.2, ys.min() + (ys.max() - ys.min()) * 0.7
    dark = {(int(y), int(x)) for y, x in zip(*np.nonzero(on)) if x >= front and y0 <= y <= y1
            and tuple(int(v) for v in cell[y, x, :3]) != OUTLINE and lum(cell[y, x, :3]) < 50}
    best = []
    while dark:
        stack = [dark.pop()]
        comp = []
        while stack:
            y, x = stack.pop()
            comp.append((y, x))
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    if (y + dy, x + dx) in dark:
                        dark.remove((y + dy, x + dx))
                        stack.append((y + dy, x + dx))
        best = max(best, comp, key=len)
    return best


def sparkle(cell):
    """돌아온 토끼풀의 눈 빛점: 원본의 흰 빛점은 40칸에서 사라진다 — 눈 덩어리(4점 넘게)의 왼쪽 위 점을 희게."""
    out = cell.copy()
    eye = eyes(cell)
    if len(eye) >= 4:
        y, x = min(eye)
        out[y, x, :3] = (255, 255, 255)
    return out


def taint(cell):
    """돌아온 토끼풀 칸 → 물든 토끼풀 칸. 털·가슴·귓속·볼은 밝기대로 모브 6단, 꽃(채도 0.45 넘는 진분홍)·잎(채도 0.2 넘는 초록 — 잎은 채도가 낮다)은 그대로,
    눈은 초록으로 빛난다(맨 위 점이 가장 밝다)."""
    out = cell.copy()
    rgb = cell[..., :3].astype(int)
    on = cell[..., 3] > 0
    for y, x in zip(*np.nonzero(on)):
        key = tuple(int(v) for v in rgb[y, x])
        if key == OUTLINE:
            out[y, x, :3] = TAINT_OUTLINE
            continue
        h, l, s = colorsys.rgb_to_hls(*(v / 255 for v in key))
        flower = (s > 0.45 and (h > 0.85 or h < 0.02) or s > 0.2 and 0.18 < h < 0.45) and 0.2 < l < 0.8   # 꽃 · 잎
        if not flower:
            out[y, x, :3] = ramp(TAINT_FUR, lum(key) / 255 * 1.05)
    eye = eyes(cell)
    top = min((y for y, _ in eye), default=0)
    for y, x in eye:
        out[y, x, :3] = EYE_GREEN[2] if y == top else EYE_GREEN[1]
    return out


def thread(cell):
    """보라 실 한 가닥: 몸 위 55% 줄을 가로질러, 머리 쪽 40%는 비운다(눈·꽃을 지나지 않게). 외곽선·밝은 점(가슴·꼬리)은 그대로."""
    out = cell.copy()
    on = cell[..., 3] > 0
    ys = np.nonzero(on.any(1))[0]
    xs = np.nonzero(on.any(0))[0]
    y = int(ys.min()) + int((ys.max() - ys.min() + 1) * 0.55)
    x0, x1 = int(xs.min()), int(xs.max())
    face = x0 + (x1 - x0) * 0.6
    for x in range(x0, x1 + 1):
        if x >= face or not on[y, x]:
            continue
        key = tuple(int(v) for v in cell[y, x, :3])
        if key == TAINT_OUTLINE or lum(key) > 170:
            continue
        out[y, x, :3] = THREAD[(x // 2) % 2]
    return out


def build():
    spr = convert(SPEC)
    cells = {k: sparkle(seat(spr[k], CELL, FOOT)) for k in NAMES}
    sheet = np.zeros((CELL * 4, CELL * len(NAMES), 4), np.uint8)
    for c, k in enumerate(NAMES):
        for r, cell in enumerate((cells[k], thread(taint(cells[k])))):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = cell
            sil = np.zeros_like(cell)
            sil[cell[..., 3] > 0] = [255, 255, 255, 255]
            sheet[(r + 2) * CELL:(r + 3) * CELL, c * CELL:(c + 1) * CELL] = sil
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/clover.png"
    Image.fromarray(build(), "RGBA").save(out)
