"""콩이(아르마딜로) 스프라이트 시트(32×32 칸, 발바닥 30, 0206 C4). 이름 있는 보스(그림 규칙 0203 키 18~20px), 몸이 옆으로 길어
칸은 32.

실행: python3 tools/art/kongi_ai.py [출력 경로]   (기본 assets/art/kongi.png)

원본: `sources/ai/sprites/kongi_v1.png`(✦ 지움). 크기 기준 그림(`kongi_silhouette.py`의 `ref_kongi_scale` — 게임 크기 Naïl 옆
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
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import OUTLINE, Source, Spec, convert, seat  # noqa: E402

CELL, FOOT = 32, 30
SRC = HERE / "sources" / "ai" / "sprites" / "kongi_v1.png"
STAND = ["idle_a", "idle_b", "walk_a", "walk_b", "hurt", "down"]
STAND_BOXES = [(4, 42, 248, 227), (260, 47, 508, 227), (520, 47, 760, 227), (772, 47, 1016, 227),
               (516, 294, 756, 478), (772, 328, 1012, 474)]
BALL = ["ball_a", "ball_b"]
BALL_BOXES = [(42, 303, 214, 474), (298, 303, 470, 474)]
SPEC = Spec(
    sources=[Source(SRC, dict(zip(STAND, STAND_BOXES)), groups=dict.fromkeys(STAND, "idle_a"), tidy=0.15, strip=9)],
    palette_ref=(0, "idle_a"),
    body_h=19,      # 서기 칸 외곽선 안 키(귀 끝~발) → 외곽선까지 20~21
)
SPEC_BALL = Spec(
    sources=[Source(SRC, dict(zip(BALL, BALL_BOXES)), groups=dict.fromkeys(BALL, "ball_a"), tidy=0.15, strip=9)],
    palette_ref=(0, "ball_a"),
    body_h=15,      # 공 지름(외곽선 안) → 외곽선까지 16~17
)
ORDER = ["idle_a", "idle_b", "walk_a", "walk_b", "ball_a", "ball_b", "hurt", "down"]

# 물든 콩이(0054): 초상화 kongi_tainted_glare — 살갗은 회색 모브, 등딱지는 푸른 회색, 눈은 옅은 파랑으로 빛난다, 보라 실.
TAINT_SKIN = [(70, 58, 68), (104, 92, 102), (138, 126, 134), (172, 162, 168), (206, 198, 204)]
TAINT_SHELL = [(44, 50, 60), (66, 76, 88), (92, 104, 116), (122, 134, 146), (160, 170, 180)]
TAINT_OUTLINE = (24, 18, 32)
EYE_BLUE = [(40, 90, 140), (120, 190, 240), (210, 240, 255)]
THREAD = [(96, 52, 160), (168, 110, 230)]


def lum(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def ramp(colors, t):
    return colors[min(len(colors) - 1, max(0, int(t * len(colors))))]


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
    """보라 실 한 가닥: 몸 위 45% 줄을 가로질러(서기는 얼굴 앞쪽 35%를 비운다, 공은 다). 외곽선·밝은 점은 그대로."""
    out = cell.copy()
    on = cell[..., 3] > 0
    ys = np.nonzero(on.any(1))[0]
    xs = np.nonzero(on.any(0))[0]
    y = int(ys.min()) + int((ys.max() - ys.min() + 1) * 0.45)
    x0, x1 = int(xs.min()), int(xs.max())
    wide = (x1 - x0) > (ys.max() - ys.min()) * 1.2
    face = x0 + (x1 - x0) * 0.65 if wide else x1 + 1
    for x in range(x0, x1 + 1):
        if x >= face or not on[y, x]:
            continue
        key = tuple(int(v) for v in cell[y, x, :3])
        if key == TAINT_OUTLINE or lum(key) > 180:
            continue
        out[y, x, :3] = THREAD[(x // 2) % 2]
    return out


def bands(cell, box, pal):
    """공의 띠를 원본에서 다시 옮긴다. 띠는 원본에서 4px 굵기라 공 배율(블록 약 11px)로 줄이면 사라진다 — 원본의 띠 점(밝기
    60~120, 외곽선보다 밝고 등딱지보다 어둡다)을 공의 테두리 상자끼리 맞춰 옮기고 외곽선 안쪽 칸에만 칠한다(띠 색 = 서기 칸
    팔레트에서 가장 가까운 색 — 공 A 칸에는 바탕색 하나뿐이라 제 칸에서 고르면 안 보인다)."""
    src = np.asarray(Image.open(SRC).convert("RGB")).astype(int)
    x0, y0, x1, y1 = box
    r = src[y0:y1, x0:x1]
    mag = (r[..., 0] > 200) & (r[..., 1] < 60)
    L = (r * [0.299, 0.587, 0.114]).sum(-1)
    body = ~mag
    band = body & (L > 60) & (L < 120)
    bys, bxs = np.nonzero(body)
    out = cell.copy()
    on = cell[..., 3] > 0
    cys, cxs = np.nonzero(on)
    sy = (cys.max() - cys.min() + 1) / (bys.max() - bys.min() + 1)
    sx = (cxs.max() - cxs.min() + 1) / (bxs.max() - bxs.min() + 1)
    want = tuple(int(v) for v in np.median(r[band], 0))
    ink = min(pal, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, want)))
    hit = np.zeros(on.shape, int)
    for y, x in zip(*np.nonzero(band)):
        hit[int(cys.min() + (y - bys.min()) * sy), int(cxs.min() + (x - bxs.min()) * sx)] += 1
    for y, x in zip(*np.nonzero(hit >= 4)):
        if on[y, x] and tuple(int(v) for v in cell[y, x, :3]) != OUTLINE:
            out[y, x, :3] = ink
    return out


def build():
    spr = convert(SPEC)
    spr.update(convert(SPEC_BALL))
    cells = {k: seat(spr[k], CELL, FOOT) for k in ORDER}
    stand = cells["idle_a"]
    pal = {tuple(int(v) for v in p[:3]) for p in stand[stand[..., 3] > 0]} - {OUTLINE}
    for k, box in zip(BALL, BALL_BOXES):
        cells[k] = bands(cells[k], box, pal)
    sheet = np.zeros((CELL * 4, CELL * len(ORDER), 4), np.uint8)
    for c, k in enumerate(ORDER):
        for r, cell in enumerate((cells[k], thread(taint(cells[k])))):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = cell
            sil = np.zeros_like(cell)
            sil[cell[..., 3] > 0] = [255, 255, 255, 255]
            sheet[(r + 2) * CELL:(r + 3) * CELL, c * CELL:(c + 1) * CELL] = sil
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/kongi.png"
    Image.fromarray(build(), "RGBA").save(out)
