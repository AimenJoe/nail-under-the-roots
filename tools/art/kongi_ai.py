"""콩이(아르마딜로) 스프라이트 시트(32×32 칸, 발바닥 30, 0206 C4 → 0238 다시). 이름 있는 보스(키 약 22px), 몸이 옆으로 길어
칸은 32.

실행: python3 tools/art/kongi_ai.py [출력 경로]   (기본 assets/art/kongi.png)

원본: `sources/ai/sprites/kongi_v2.png`(0238, 사장 고름 3번 — 모래색 등딱지·큰 귀·꼬리, ✦ 지움). 걷기 넷도 같은 시트 안.
옛 원본 `kongi_v1.png`(0217)에 대한 아래 설명(공 배율·띠)은 그대로 맞다. 크기 기준 그림(`kongi_silhouette.py`의 `ref_kongi_scale` — 게임 크기 Naïl 옆
서기·공 실루엣 8배)을 붙여 받았다. 블록 약 8~9px, 고른 격자가 아니라 원본 외곽선을 걷고(`strip`) 줄인다. 공은 서 있는 키와 거의
같은 지름으로 와서(실루엣은 16px로 줬다) 서기 배율로 줄이면 공이 지름 20px — 공 두 칸은 따로 공 지름으로 줄인다(`BALL`).

칸 배치(critters 콩이 줄과 같은 뜻, `art::BIG_SHEETS`로 `Art::critter`가 콩이 두 줄을 여기로 돌린다):
  칸 0·1 서기, 2·3 걷기, 4·5 공(띠가 엇갈린 두 장 — 구르기), 6 맞음, 7 쓰러짐
  줄 0 돌아온 콩이 · 줄 1 물든 콩이(보스: 회색 살갗·푸른 회색 등딱지·옅은 파랑으로 빛나는 눈·보라 실 — 줄 0의 색을 바꾸고
  실은 코드로, 초상화 `kongi_tainted_glare`)
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import colorsys
import sys
from collections import Counter
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import walk_ai  # noqa: E402
from sprite_ai import OUTLINE, Source, Spec, convert, lum, ramp, seat, sheet4, step_cycle  # noqa: E402

CELL, FOOT = 32, 30
SRC = HERE / "sources" / "ai" / "sprites" / "kongi_v2.png"
# 원본(0238, 사장 고름 1단계 3번 → 시트 한 장, ✦ 지움): 윗줄 서기·서기(숨)·맞음·뒤집힘(+ 같은 셋 한 번 더, 쓰지 않음),
# 아랫줄 걷기 넷·공 둘·반쯤 말기·펴기(뒤 둘은 쓰지 않음). 칸 상자는 redo_concepts.components로 찾은 값.
# 원본이 작다(키 약 100px, 블록 약 4~5px) — 외곽선 걷기는 4px.
STAND = ["idle_a", "idle_b", "walk_a", "walk_b", "hurt", "down"] + walk_ai.NAMES
STAND_BOXES = [(15, 7, 136, 110), (153, 7, 273, 110), (12, 133, 133, 229), (271, 130, 389, 227),
               (284, 7, 405, 111), (429, 27, 552, 112),
               (12, 133, 133, 229), (145, 131, 262, 229), (271, 130, 389, 227), (387, 131, 503, 231)]
BALL = ["ball_a", "ball_b"]
BALL_BOXES = [(519, 141, 606, 231), (640, 141, 728, 231)]
STRIP = 4
SPEC = Spec(
    sources=[Source(SRC, dict(zip(STAND, STAND_BOXES)),
                    groups={**dict.fromkeys(STAND[:6], "idle_a"), **dict.fromkeys(walk_ai.NAMES, walk_ai.NAMES[0])},
                    tidy=0.15, strip=STRIP)],
    palette_ref=(0, "idle_a"),
    body_h=21,      # 서기 칸 외곽선 안 키(귀 끝~발) → 외곽선까지 22~23(Naïl 22보다 조금 크게 — 길의 주인)
)
SPEC_BALL = Spec(
    sources=[Source(SRC, dict(zip(BALL, BALL_BOXES)), groups=dict.fromkeys(BALL, "ball_a"), tidy=0.15, strip=STRIP)],
    palette_ref=(0, "ball_a"),
    body_h=16,      # 공 지름(외곽선 안) → 외곽선까지 17~18
)
ORDER = ["idle_a", "idle_b", "walk_a", "walk_b", "ball_a", "ball_b", "hurt", "down"] + walk_ai.NAMES   # 8~11 걷기 넷(0229)

# 물든 콩이(0054): 초상화 kongi_tainted_glare — 살갗은 회색 모브, 등딱지는 푸른 회색, 눈은 옅은 파랑으로 빛난다, 보라 실.
TAINT_SKIN = [(70, 58, 68), (104, 92, 102), (138, 126, 134), (172, 162, 168), (206, 198, 204)]
TAINT_SHELL = [(44, 50, 60), (66, 76, 88), (92, 104, 116), (122, 134, 146), (160, 170, 180)]
TAINT_OUTLINE = (24, 18, 32)
EYE_BLUE = [(40, 90, 140), (120, 190, 240), (210, 240, 255)]
THREAD = [(96, 52, 160), (168, 110, 230)]
BAND_INK = (92, 70, 50)          # 원본 띠의 갈색(서기 칸 팔레트에서 가장 가까운 색을 쓴다)




def eyes(cell):
    """눈 = 몸 앞쪽 45%, 키의 35~75% 줄 안의 아주 어두운 점(귀 속 그늘·발가락은 줄 밖, 외곽선 색은 뺀다 — 눈은 외곽선보다 조금 더 어두운 제 색으로 온다). 원본의 흰
    빛점은 32칸에서 사라져 보리처럼 빛점 둘레로 찾을 수 없다."""
    on = cell[..., 3] > 0
    xs = np.nonzero(on.any(0))[0]
    ys = np.nonzero(on.any(1))[0]
    front = xs.min() + (xs.max() - xs.min()) * 0.55
    y0, y1 = ys.min() + (ys.max() - ys.min()) * 0.35, ys.min() + (ys.max() - ys.min()) * 0.75
    return [(y, x) for y, x in zip(*np.nonzero(on)) if x >= front and y0 <= y <= y1
            and tuple(int(v) for v in cell[y, x, :3]) != OUTLINE and lum(cell[y, x, :3]) < 45]


def taint(cell):
    """돌아온 콩이 칸 → 물든 콩이 칸. 분홍 쪽(살갗·귀·볼)은 회색 모브, 올리브 쪽(등딱지·띠·꼬리)은 푸른 회색,
    눈(`eyes`)은 옅은 파랑으로 빛난다(맨 위 점이 가장 밝다)."""
    out = cell.copy()
    rgb = cell[..., :3].astype(int)
    on = cell[..., 3] > 0
    for y, x in zip(*np.nonzero(on)):
        key = tuple(int(v) for v in rgb[y, x])
        L = lum(key) / 255
        if key == OUTLINE:
            c = TAINT_OUTLINE
        else:
            h, _, _ = colorsys.rgb_to_hls(*(v / 255 for v in key))
            pink = h < 0.07 or h > 0.85
            c = ramp(TAINT_SKIN if pink else TAINT_SHELL, L * 1.1)
        out[y, x, :3] = c
    eye = eyes(cell)
    top = min((y for y, _ in eye), default=0)
    for y, x in eye:
        out[y, x, :3] = EYE_BLUE[2] if y == top else EYE_BLUE[1]
    return out


def thread(cell):
    """보라 실 한 가닥: 몸 위 45% 줄을 가로질러(서기는 얼굴 앞쪽 45%를 비운다 — 35%면 눈에 닿았다, 공은 다). 외곽선·밝은 점은 그대로."""
    out = cell.copy()
    on = cell[..., 3] > 0
    ys = np.nonzero(on.any(1))[0]
    xs = np.nonzero(on.any(0))[0]
    y = int(ys.min()) + int((ys.max() - ys.min() + 1) * 0.45)
    x0, x1 = int(xs.min()), int(xs.max())
    wide = (x1 - x0) > (ys.max() - ys.min()) * 1.05   # 서 있는 칸(새 콩이는 몸이 덜 길다, 0238). 공은 폭 ≈ 키
    face = x0 + (x1 - x0) * 0.55 if wide else x1 + 1
    for x in range(x0, x1 + 1):
        if x >= face or not on[y, x]:
            continue
        key = tuple(int(v) for v in cell[y, x, :3])
        if key == TAINT_OUTLINE or lum(key) > 180:
            continue
        out[y, x, :3] = THREAD[(x // 2) % 2]
    return out


def bands(cell, box, pal):
    """공의 띠를 원본에서 다시 옮긴다. 띠는 원본에서 4px 굵기의 갈색 줄이라 공 배율(블록 약 11px)로 줄이면 사라진다(공 A는
    민무늬가 됐다). 원본의 띠 점(밝기 50~125의 갈색: 붉은 기가 푸른 기보다 25 넘게 많다)을 몸의 테두리 상자끼리 맞춰 옮기고,
    외곽선 안쪽에만 칠한다. 띠 색 = 서기 칸 팔레트에서 가장 가까운 색(공 A 칸에는 바탕색 하나뿐이라 제 칸에서 고르면 안
    보인다). 공의 띠는 원본도 곧은 줄(세로·비스듬)이라 그대로 옮긴다 — 서기 칸은 `arcs`."""
    src = np.asarray(Image.open(SRC).convert("RGB")).astype(int)
    x0, y0, x1, y1 = box
    r = src[y0:y1, x0:x1]
    body = ~((r[..., 0] > 200) & (r[..., 1] < 60))
    L = (r * [0.299, 0.587, 0.114]).sum(-1)
    band = body & (L > 50) & (L < 125) & (r[..., 0] - r[..., 2] > 25) & (r[..., 1] >= r[..., 2])
    rows = np.nonzero(body.sum(1) >= 6)[0]                     # 반짝이 같은 작은 조각은 상자에서 뺀다
    cols = np.nonzero(body.sum(0) >= 6)[0]
    out = cell.copy()
    on = cell[..., 3] > 0
    cys, cxs = np.nonzero(on)
    sy = (cys.max() - cys.min() + 1) / (rows.max() - rows.min() + 1)
    sx = (cxs.max() - cxs.min() + 1) / (cols.max() - cols.min() + 1)
    want = tuple(int(v) for v in np.median(r[band], 0))
    ink = min(pal, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, want)))
    hit = np.zeros(on.shape, int)
    for y, x in zip(*np.nonzero(band)):
        ty = int(cys.min() + (y - rows.min()) * sy)
        tx = int(cxs.min() + (x - cols.min()) * sx)
        if 0 <= ty < hit.shape[0] and 0 <= tx < hit.shape[1]:
            hit[ty, tx] += 1
    for y, x in zip(*np.nonzero(hit >= 6)):
        key = tuple(int(v) for v in cell[y, x, :3])
        if not on[y, x] or key == OUTLINE:
            continue
        out[y, x, :3] = ink
    return out


BAND_AT = (0.18, 0.48, 0.8)     # 등딱지 가운데 줄에서 꼬리 쪽 테두리(0) ~ 머리 쪽 끝(1) 사이 띠 자리
BAND_BOW = 1.5                  # 띠 양 끝이 머리 쪽으로 휘는 폭(px). 띠 사이가 3px이라 2면 이웃 띠와 엉킨다


def arcs(cell, ink):
    """서기 칸의 등딱지 띠: 원본에서 옮기면(`bands`처럼) 띠가 흩어지거나 거의 곧은 세로줄로 남는다 — 원본의 띠도 높이 약
    100px에 10px만 휘어 게임 크기(띠 높이 10px)에선 1px 꺾임도 안 된다(사장: "띠가 곡선이 아닌 직선처럼 보인다"). 그래서
    줄인 칸에 띠를 새로 긋는다: 등딱지(분홍·외곽선이 아닌 가장 많은 색) 가운데 줄에서 `BAND_AT` 자리마다, 그 세로 칸의
    등딱지 위~아래를 따라 원본과 같은 "(" 모양(가운데가 꼬리 쪽으로 불룩, 양 끝이 머리 쪽으로 `BAND_BOW`px)으로.
    칸마다 같은 자리라 서기·걷기를 오갈 때 띠 수가 바뀌어 깜빡이지 않는다. 줄이며 생긴 띠 색 점은 먼저 등딱지 색으로 덮는다
    (줄마다 꼬리 쪽 맨 끝 점 = 테두리 그늘은 둔다)."""
    out = cell.copy()
    on = cell[..., 3] > 0
    rgb = cell[..., :3].astype(int)
    count = Counter(tuple(int(v) for v in rgb[y, x]) for y, x in zip(*np.nonzero(on)))
    shell = max((c for c in count if c not in (OUTLINE, ink) and not pinkish(c)), key=lambda c: count[c])
    S = on & (np.all(rgb == shell, -1) | np.all(rgb == ink, -1))
    ys = np.nonzero(S.any(1))[0]
    for y in ys:
        row = np.nonzero(S[y])[0]
        for x in row[1:]:
            if tuple(int(v) for v in rgb[y, x]) == ink:
                out[y, x, :3] = shell
    mid = int(ys.min() + (ys.max() - ys.min()) * 0.5)
    row = np.nonzero(S[mid])[0]
    for f in BAND_AT:
        xm = int(round(row.min() + (row.max() - row.min()) * f))
        col = np.nonzero(S[:, xm])[0]
        top = bot = int(col.min())
        while bot + 1 < S.shape[0] and S[bot + 1, xm]:
            bot += 1
        half = max(1, (bot - top) / 2)
        for y in range(top, bot + 1):
            t = (y - (top + bot) / 2) / half
            x = int(round(xm + BAND_BOW * t * t))
            if S[y, x]:
                out[y, x, :3] = ink
    return out


def ink_lines(cell):
    """몸 안쪽 선(다리 사이·귀 테두리)이 순검정으로 온다(0238 원본) — 외곽선 색으로 맞춘다(검은 점이 배에 얼룩처럼 보였다)."""
    out = cell.copy()
    dark = (cell[..., 3] > 0) & (cell[..., :3].astype(int).sum(-1) < 60)
    out[dark, :3] = OUTLINE
    return out


def pinkish(c):
    h, _, _ = colorsys.rgb_to_hls(*(v / 255 for v in c))
    return h < 0.07 or h > 0.85


def build():
    spr = convert(SPEC)
    spr.update(convert(SPEC_BALL))
    cells = {k: ink_lines(seat(spr[k], CELL, FOOT)) for k in ORDER}
    stand = cells["idle_a"]
    pal = {tuple(int(v) for v in p[:3]) for p in stand[stand[..., 3] > 0]} - {OUTLINE}
    for k, box in zip(BALL, BALL_BOXES):
        cells[k] = bands(cells[k], box, pal)
    ink = min(pal, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, BAND_INK)))
    for k in STAND:
        cells[k] = arcs(cells[k], ink)
    # 걷기는 첫 칸 하나에서 발만 딛게(0241): 원본 네 칸은 등딱지 크기·머리 높이·눈이 칸마다 달라 걸을 때 몸이 끓었다.
    # 두 칸 걷기(walk_a·b)도 같은 순환의 두 디딤 칸으로.
    steps = step_cycle(cells[walk_ai.NAMES[0]], FOOT - 2)
    cells.update(zip(walk_ai.NAMES, steps))
    cells["walk_a"], cells["walk_b"] = steps[0], steps[2]
    return sheet4([(cells[k], thread(taint(cells[k]))) for k in ORDER])


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/kongi.png"
    Image.fromarray(build(), "RGBA").save(out)
