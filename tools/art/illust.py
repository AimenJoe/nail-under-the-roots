"""컷씬 일러스트 생성기 — 임시 그림(docs/decisions/0039). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/illust.py [출력 폴더]  (기본 assets/art/illust)
사람이 그린 그림으로 바꿀 때는 같은 이름의 PNG를 넣는다(사양: docs/art/cutscene-illustrations.md).

크기: 화면에 보이는 창 640×360(게임 320×180의 2배 밀도). 패닝하는 그림은 더 크다(src/illust.rs의 pan과 맞출 것).
  great_tree   640×560  위(나무) → 아래(땅속)로 패닝
  halmae_deep  704×360  오른쪽으로 64px 패닝
  last_bloom   640×440  위(꽃) → 아래(밑동)로 패닝
  나머지       640×360(doto_memory 포함 — 도토가 기억해 내는 장면, 0053)
대화창·내레이션 띠가 보이는 창의 y 248 아래를 덮는다 → 얼굴·중요한 것은 그 위에.

그리는 법(픽셀 그림 흉내):
  - 하늘·흙은 계단식 그라디언트(띠), 1px 디더는 쓰지 않는다(홀수 화면 배율에서 1.5배로 늘어나 얼룩진다).
    띠 경계만 2×2 칸 바둑판으로 섞는다(0049): 2×2 = 게임 1px이라 어느 배율에서도 고르게 찍힌다.
  - 몸은 마스크 하나에 구(球) 음영 띠를 입힌다(왼쪽 위 빛). 캐릭터마다 바깥에 어두운 외곽선 1~2px.
  - 실·뿌리는 베지어 곡선을 굵기를 줄여 가며 긋는다.
비교 시험: NAIL_ART_V2=1(색조 이동·조약돌·풀 포기·노을 가장자리 빛), NAIL_ART_V4=1(v2 + 털 뭉치·삐침·Naïl 목도리, 0072).
"""
import math
import os
import random
import sys

import numpy as np
from PIL import Image, ImageDraw

OUTLINE = (43, 29, 42)
LIGHT = np.array([-0.55, -0.65, 0.52])
LIGHT = LIGHT / np.linalg.norm(LIGHT)

# 재질 띠(어두움 → 밝음). 초상화(tools/art/portraits.py) 팔레트와 같은 계열.
FUR = [(58, 38, 42), (88, 60, 54), (122, 88, 72), (152, 114, 90), (184, 146, 116)]
MOM_FUR = [(50, 34, 40), (78, 54, 52), (108, 78, 66), (136, 104, 84), (164, 132, 108)]
PINK = [(160, 78, 100), (206, 112, 132), (240, 150, 164), (255, 198, 204)]
CLAW = [(176, 160, 138), (214, 200, 176), (244, 236, 216)]
RFUR = [(62, 40, 34), (98, 66, 50), (134, 94, 70), (170, 128, 94), (204, 164, 124)]
RMUZ = [(150, 118, 100), (188, 158, 130), (216, 192, 160), (238, 220, 192)]
RABBIT = [(70, 56, 58), (104, 84, 80), (138, 114, 102), (170, 146, 128), (200, 180, 160)]
RABBIT_LIGHT = [(170, 150, 136), (204, 188, 172), (230, 220, 206)]
BARK = [(46, 30, 30), (70, 48, 40), (96, 68, 52), (124, 92, 68)]
VIOLET = [(54, 28, 70), (92, 50, 116), (136, 84, 164), (182, 132, 206)]
THREAD_WHITE = [(150, 150, 170), (200, 200, 214), (236, 236, 244), (255, 255, 255)]
GREEN = [(40, 70, 46), (62, 104, 58), (92, 140, 70), (132, 176, 88)]
BLOSSOM = [(186, 96, 124), (226, 136, 160), (246, 178, 194), (255, 222, 230)]
CAP = [(112, 78, 54), (156, 112, 70), (196, 150, 92), (226, 190, 132)]
# 도토(청설모): 붉은 밤색 털, 크림 배(portraits.py dfur·dlight와 같은 계열). 도토리 갓.
DFUR = [(70, 36, 30), (112, 62, 42), (160, 96, 60), (200, 134, 86), (230, 170, 116)]
DLIGHT = [(206, 180, 156), (232, 212, 190), (248, 238, 222)]
ACORN = [(62, 40, 26), (98, 66, 40), (138, 98, 60), (176, 134, 84)]

# v2(비교 시험, 기본 꺼짐): NAIL_ART_V2=1. 초상화 v2와 같은 색조 이동 + 덩어리(조약돌·풀·나무껍질)·노을 가장자리 빛.
# Naïl의 뜨개 목도리(portraits.py RAMP['scarf']와 같은 색)
SCARF = [(104, 34, 44), (150, 52, 54), (196, 82, 66), (228, 122, 86), (246, 166, 120)]

_BASE = dict(FUR=FUR, MOM_FUR=MOM_FUR, PINK=PINK, RFUR=RFUR, RMUZ=RMUZ, RABBIT=RABBIT, BARK=BARK, VIOLET=VIOLET,
             GREEN=GREEN, BLOSSOM=BLOSSOM, CAP=CAP, DFUR=DFUR, ACORN=ACORN, SCARF=SCARF)
SUNSET_RIM = (226, 160, 118)
# 그림 방식. 환경 변수가 있으면 모든 장면에(비교용), 없으면 장면마다 SCENE_STYLE(0072·0073).
#   NAIL_ART_V2=1: 색조 이동·조약돌·풀 포기·노을 가장자리 빛 / NAIL_ART_V4=1: v2 + 초상화 v4의 털 규칙(0071)
V2_ENV = os.environ.get('NAIL_ART_V2') == '1'
V4_ENV = os.environ.get('NAIL_ART_V4') == '1'


def set_style(v2, v4):
    """경사 표와 V2·V4를 한 장면 동안 바꾼다(장면 함수는 전역을 그릴 때 읽는다)."""
    global V2, V4, FUR_RAMPS
    V4 = v4
    V2 = v2 or v4
    if V2:
        from portraits import hue_shift
        globals().update({k: hue_shift(v) for k, v in _BASE.items()})
    else:
        globals().update(_BASE)
    FUR_RAMPS = (FUR, MOM_FUR, RFUR, RABBIT, DFUR)


set_style(V2_ENV, V4_ENV)
_lu = math.hypot(LIGHT[0], LIGHT[1])
_U = (LIGHT[0] / _lu, LIGHT[1] / _lu)      # 화면에서 빛 쪽
_V = (-_U[1], _U[0])


# ---- 캔버스와 마스크 -----------------------------------------------------------

class Canvas:
    def __init__(self, w, h, rgb=(0, 0, 0)):
        self.w, self.h = w, h
        self.px = np.zeros((h, w, 3), dtype=np.float32)
        self.px[:] = rgb
        self.yy, self.xx = np.mgrid[0:h, 0:w].astype(np.float32)

    def paint(self, mask, rgb, alpha=1.0):
        c = np.array(rgb, dtype=np.float32)
        if alpha >= 1.0:
            self.px[mask] = c
        else:
            self.px[mask] = self.px[mask] * (1 - alpha) + c * alpha

    def image(self):
        return Image.fromarray(np.clip(self.px + 0.5, 0, 255).astype(np.uint8), 'RGB')


def ellipse(cv, cx, cy, rx, ry, rot=0.0):
    x = cv.xx - cx
    y = cv.yy - cy
    if rot:
        c, s = math.cos(rot), math.sin(rot)
        x, y = x * c + y * s, -x * s + y * c
    return (x / rx) ** 2 + (y / ry) ** 2 <= 1.0


def poly(cv, pts):
    im = Image.new('L', (cv.w, cv.h), 0)
    ImageDraw.Draw(im).polygon([(round(x), round(y)) for x, y in pts], fill=255)
    return np.array(im) > 0


def bezier(p0, p1, p2, p3, n=48):
    out = []
    for i in range(n + 1):
        t = i / n
        u = 1 - t
        x = u ** 3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t ** 3 * p3[0]
        y = u ** 3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t ** 3 * p3[1]
        out.append((x, y))
    return out


def stroke(cv, pts, w0, w1=None):
    """굵기 w0 → w1로 가늘어지는 선(마스크)."""
    w1 = w0 if w1 is None else w1
    im = Image.new('L', (cv.w, cv.h), 0)
    d = ImageDraw.Draw(im)
    n = len(pts) - 1
    for i in range(n):
        w = w0 + (w1 - w0) * i / max(1, n)
        a, b = pts[i], pts[i + 1]
        d.line([(round(a[0]), round(a[1])), (round(b[0]), round(b[1]))], fill=255, width=max(1, round(w)))
        if w >= 3:
            r = w / 2
            d.ellipse([b[0] - r, b[1] - r, b[0] + r, b[1] + r], fill=255)
    return np.array(im) > 0


def dilate(m, r=1):
    out = m.copy()
    for _ in range(r):
        o = out.copy()
        o[1:, :] |= out[:-1, :]
        o[:-1, :] |= out[1:, :]
        o[:, 1:] |= out[:, :-1]
        o[:, :-1] |= out[:, 1:]
        out = o
    return out


def shade(cv, mask, ramp, cx=None, cy=None, rx=None, ry=None, bias=0.0, clumps=False):
    """마스크에 구 음영 띠. 중심·반지름을 안 주면 마스크 상자로."""
    ys, xs = np.nonzero(mask)
    if len(xs) == 0:
        return
    if cx is None:
        cx, cy = (xs.min() + xs.max()) / 2, (ys.min() + ys.max()) / 2
        rx, ry = max(1, (xs.max() - xs.min()) / 2), max(1, (ys.max() - ys.min()) / 2)
    nx = np.clip((cv.xx - cx) / rx, -1, 1)
    ny = np.clip((cv.yy - cy) / ry, -1, 1)
    nz = np.sqrt(np.clip(1 - nx * nx - ny * ny, 0, 1))
    k = nx * LIGHT[0] + ny * LIGHT[1] + nz * LIGHT[2] + bias
    k = np.clip((k + 0.35) / 1.25, 0, 0.999)
    idx = (k * len(ramp)).astype(int)
    if V4 and (clumps or any(ramp is r for r in FUR_RAMPS)):
        n = len(ramp)
        idx[(idx == n - 1) & (k < 0.9)] = n - 2       # 가장 밝은 단은 빛 바로 밑 작은 덩어리만
        idx = fur_clumps(cv, mask, idx, max(rx, ry))
    for i, c in enumerate(ramp):
        cv.paint(mask & (idx == i), c)


def outline(cv, mask, rgb=OUTLINE, r=2):
    cv.paint(dilate(mask, r) & ~mask, rgb)


# ---- v4: 털이 대머리·피부로 읽히지 않게(초상화 0071과 같은 규칙, 마스크판) ------------------------------
def fur_clumps(cv, mask, idx, size):
    """단 경계를 털 뭉치로: 빛 방향으로 비대칭 톱니만큼 밀린 자리의 단을 가져온다. 경계만 휜다.
    일러스트는 인물 크기가 제각각이라 톱니 주기를 크기에 맞춘다."""
    period = float(np.clip(size * 0.09, 4.6, 11.0))
    amp = period * 0.75
    a = cv.xx * _V[0] + cv.yy * _V[1]
    b = cv.xx * _U[0] + cv.yy * _U[1]
    k = amp * (((a / period + b * 0.12 * 4.6 / period) % 1.0) - 0.5)
    qx = np.clip(np.rint(cv.xx + _U[0] * k), 0, cv.w - 1).astype(int)
    qy = np.clip(np.rint(cv.yy + _U[1] * k), 0, cv.h - 1).astype(int)
    ok = mask & mask[qy, qx]
    return np.where(ok, idx[qy, qx], idx)


# 삐침: 크기에 비례하는 삼각형(일러스트는 인물이 초상화보다 몇 배 커서, 1px 도장은 바깥선에 묻혀 가는 송곳털이 된다).
# 길이·간격은 일부러 섞는다(같은 크기가 고르게 서면 톱니·가시).
TUFT_LEN = (1.0, 0.6, 1.4, 0.7, 1.1, 0.8, 1.5, 0.6, 1.2)
TUFT_GAP = (1.0, 1.4, 1.1, 1.6, 0.9, 1.3, 1.8, 1.1)


def tufted(cv, mask, size, facing=1):
    """털 덩어리 마스크에 삐침을 더해 돌려준다. 윗선은 끝이 뒤쪽 위, 뒤쪽 옆선은 끝이 뒤쪽 아래(털이 코에서 뒤로 자란다).
    facing 0이면(정면) 윗선은 바깥쪽으로 갈라지고 양옆 모두. 좁은 줄(귀·꼬리 끝)은 건너뛴다."""
    if not V4:
        return mask
    out = mask.copy()
    ys, xs = np.nonzero(mask)
    if len(xs) == 0:
        return out
    L0, G0 = max(2.5, size * 0.075), max(5.0, size * 0.16)
    min_run = size * 0.35
    midx = (xs.min() + xs.max()) / 2

    def run(y, x):
        row = mask[y]
        l = x
        while l > 0 and row[l - 1]:
            l -= 1
        r = x
        while r < cv.w - 1 and row[r + 1]:
            r += 1
        return r - l + 1
    tris = []
    i, x = 0, float(xs.min()) + G0 * 0.5
    while x <= xs.max():                              # 윗선
        xi = int(x)
        col = np.nonzero(mask[:, xi])[0]
        if len(col) and col.min() + 1 < cv.h and run(col.min() + 1, xi) >= min_run:
            top = col.min()
            L = L0 * TUFT_LEN[i % len(TUFT_LEN)]
            back = (-1 if facing > 0 else 1) if facing else (-1 if xi < midx else 1)
            tris.append([(xi - L * 0.75, top + 2), (xi + L * 0.75, top + 2), (xi + back * L * 0.9, top - L * 0.7)])
            i += 1
        x += G0 * TUFT_GAP[i % len(TUFT_GAP)]
    for side in (((-1 if facing > 0 else 1),) if facing else (-1, 1)):   # 뒤쪽 옆선(정면이면 양옆)
        i, y = 0, float(ys.min()) + G0
        while y < ys.max() - G0:
            yi = int(y)
            row = np.nonzero(mask[yi])[0]
            if len(row):
                x = row.min() if side < 0 else row.max()
                if run(yi, x) >= min_run:
                    L = L0 * TUFT_LEN[(i + 4) % len(TUFT_LEN)]
                    tris.append([(x - side * 2, yi - L * 0.75), (x - side * 2, yi + L * 0.75), (x + side * L * 0.7, yi + L * 0.9)])
                    i += 1
            y += G0 * TUFT_GAP[(i + 3) % len(TUFT_GAP)]
    if tris:
        im = Image.new('L', (cv.w, cv.h), 0)
        d = ImageDraw.Draw(im)
        for t in tris:
            d.polygon([(round(px), round(py)) for px, py in t], fill=255)
        out |= np.array(im) > 0
    return out


def lit_outline(cv, sil, fur, ramp, r=2):
    """v4: 바깥선을 두르고, 털 덩어리의 빛 쪽(왼쪽 위) 바깥선만 털 둘째 단으로(빛 쪽 실루엣이 가장 어두우면 빛과 어긋난다)."""
    outline(cv, sil, r=r)
    if not V4:
        return
    ring = dilate(sil, r) & ~sil
    lit = np.zeros_like(fur)
    for i in range(1, r + 1):
        sh = np.zeros_like(fur)
        sh[:-i, :-i] = fur[i:, i:]          # 오른쪽 아래 i칸이 털이면 빛 쪽 바깥
        lit |= sh
    cv.paint(ring & lit, ramp[1])


def bands(cv, y0, y1, top, bottom, n, x0=0, x1=None, blend=True):
    """y0..y1 세로 계단 그라디언트. 띠 경계 위아래 4px은 2×2 칸 바둑판으로 두 색을 섞는다(띠 자국을 누그러뜨린다)."""
    x1 = cv.w if x1 is None else x1
    cols = []
    edges = []
    for i in range(n):
        t = i / max(1, n - 1)
        c = tuple(top[j] + (bottom[j] - top[j]) * t for j in range(3))
        a = y0 + (y1 - y0) * i / n
        b = y0 + (y1 - y0) * (i + 1) / n
        m = (cv.yy >= a) & (cv.yy < b) & (cv.xx >= x0) & (cv.xx < x1)
        cv.paint(m, c)
        cols.append(c)
        edges.append(b)
    if not blend:
        return
    cell = ((cv.xx // 2) + (cv.yy // 2)) % 2 == 0
    inx = (cv.xx >= x0) & (cv.xx < x1)
    for i in range(n - 1):
        e = int(round(edges[i] / 2)) * 2
        # 경계 위 한 줄(2px)은 아래 색을 반만, 경계 아래 한 줄은 위 색을 반만.
        up = (cv.yy >= e - 2) & (cv.yy < e) & inx & cell
        down = (cv.yy >= e) & (cv.yy < e + 2) & inx & cell
        cv.paint(up, cols[i + 1])
        cv.paint(down, cols[i])


def glow(cv, cx, cy, r, rgb, strength=0.5, steps=4):
    """계단식 빛 번짐."""
    for i in range(steps):
        rr = r * (1 - i / steps)
        m = ellipse(cv, cx, cy, rr, rr)
        cv.paint(m, rgb, strength / steps)


def speckle(cv, rng, region, n, colors, size=(2, 4)):
    for _ in range(n):
        x = rng.uniform(region[0], region[2])
        y = rng.uniform(region[1], region[3])
        s = rng.uniform(*size)
        cv.paint(ellipse(cv, x, y, s, s * 0.7), rng.choice(colors))


def wavy_thread(cv, p0, p3, rng, amp, width, ramp, wraps=0, base=None):
    """흔들리는 실. base(뿌리 선)가 있으면 그 둘레를 감는다."""
    pts = bezier(p0, (p0[0] + rng.uniform(-60, 60), p0[1] + rng.uniform(20, 80)),
                 (p3[0] + rng.uniform(-60, 60), p3[1] - rng.uniform(20, 80)), p3, n=80)
    out = []
    ph = rng.uniform(0, 6.28)
    for i, (x, y) in enumerate(pts):
        t = i / (len(pts) - 1)
        wob = math.sin(t * (wraps * 6.28 + 9) + ph) * amp
        out.append((x + wob, y))
    m = stroke(cv, out, width, max(1, width * 0.5))
    cv.paint(dilate(m, 1) & ~m, ramp[0])
    cv.paint(m, ramp[2])
    hi = stroke(cv, [(x - 1, y - 1) for x, y in out[::2]], 1)
    cv.paint(hi & m, ramp[3])
    return out


# ---- 인물 -------------------------------------------------------------------

def mole(cv, cx, cy, s, facing=1, eyes='open', ramp=None, paws=True):
    """두더지: 둥근 몸 + 분홍 주둥이 + 큰 앞발·크림 발톱. s = 몸 가로 반지름. ramp 없으면 Naïl 털(FUR)."""
    ramp = FUR if ramp is None else ramp
    f = facing
    body = ellipse(cv, cx, cy, s, s * 0.8)
    head = ellipse(cv, cx + f * s * 0.55, cy - s * 0.05, s * 0.6, s * 0.55)
    allm = body | head
    nail_ = V4 and ramp is FUR            # Naïl(엄마는 목도리를 떠 준 쪽이라 없다)
    if nail_:
        # 정수리 한 가닥: 얼굴 쪽으로 기운 잎 모양
        hx, hy = cx + f * s * 0.42, cy - s * 0.05 - s * 0.55
        allm |= poly(cv, [(hx - f * s * 0.08, hy + s * 0.06), (hx + f * s * 0.06, hy + s * 0.04),
                          (hx + f * s * 0.12, hy - s * 0.16), (hx + f * s * 0.02, hy - s * 0.08)])
    allm = tufted(cv, allm, s, f)
    snout = ellipse(cv, cx + f * s * 1.08, cy + s * 0.05, s * 0.28, s * 0.18)
    nose = ellipse(cv, cx + f * s * 1.3, cy + s * 0.02, s * 0.11, s * 0.1)
    pawm = np.zeros_like(body)
    claws = []
    if paws:
        for dy, dx in ((0.55, 0.62), (0.62, 0.25)):
            px, py = cx + f * s * dx, cy + s * dy
            pawm |= ellipse(cv, px, py, s * 0.24, s * 0.16)
            for k in range(3):
                claws.append(ellipse(cv, px + f * s * (0.18 + 0.02 * k), py - s * 0.1 + k * s * 0.09,
                                     s * 0.09, s * 0.045))
    clawm = np.zeros_like(body)
    for c in claws:
        clawm |= c
    scarf = tails = np.zeros_like(body)
    if nail_:
        # 엄마가 떠 준 목도리: 머리와 몸 사이 목을 감은 띠(앞으로 살짝 휜다) + 앞 아래 매듭 + 뒤로 늘어진 꼬리
        # 옆에서 보면 목이 없어서, 머리 둘레(몸과 맞닿은 쪽)를 감은 깃처럼 그린다. 매듭은 머리 뒤 아래, 꼬리는 아래로.
        hx, hy, hrx, hry = cx + f * s * 0.55, cy - s * 0.05, s * 0.6, s * 0.55
        w = s * 0.2
        collar = ellipse(cv, hx, hy, hrx + w, hry + w) & ~ellipse(cv, hx, hy, hrx, hry) & body
        scarf = collar & (cv.yy > hy - hry * 0.7)
        th = math.radians(112)                       # 매듭: 머리 뒤 아래(앞발에 가리지 않게)
        kx, ky = hx + f * math.cos(th) * (hrx + w * 0.5), hy + math.sin(th) * (hry + w * 0.5)
        knot = ellipse(cv, kx, ky, s * 0.1, s * 0.085)
        tails = knot | stroke(cv, bezier((kx, ky), (kx - f * s * 0.04, ky + s * 0.14),
                                         (kx - f * s * 0.12, ky + s * 0.22), (kx - f * s * 0.2, ky + s * 0.34)),
                              max(2, s * 0.12), max(2, s * 0.07))
        x0 = kx
    sil = allm | snout | nose | pawm | clawm | tails
    lit_outline(cv, sil, allm, ramp, r=2 if s > 30 else 1)
    shade(cv, allm, ramp, cx + f * s * 0.2, cy, s * 1.2, s * 0.9)
    if nail_:
        # 원통처럼: 빛 쪽이 밝다. 뜨개 골은 4칸마다 한 줄(2칸 줄무늬는 소음).
        shade(cv, scarf, SCARF[:4], hx, hy, hrx + w, hry + w)
        ang = np.arctan2(cv.yy - hy, (cv.xx - hx) * f)
        cv.paint(scarf & ((np.floor(ang * (hrx + w) / 4.0) % 2) == 0) & ((np.floor(ang * (hrx + w) / 2.0) % 2) == 0), SCARF[0], 0.4)
        shade(cv, tails, SCARF[:4], kx, ky + s * 0.1, s * 0.25, s * 0.3)
    shade(cv, snout | nose, PINK)
    cv.paint(nose, PINK[0])
    cv.paint(ellipse(cv, cx + f * s * 1.27, cy - s * 0.02, s * 0.04, s * 0.035), PINK[3])
    shade(cv, pawm, PINK, bias=0.15)
    cv.paint(clawm, CLAW[2])
    ex, ey = cx + f * s * 0.72, cy - s * 0.2
    er = max(1.5, s * 0.075)
    if eyes == 'open':
        cv.paint(ellipse(cv, ex, ey, er, er * 1.1), (22, 14, 30))
        cv.paint(ellipse(cv, ex - er * 0.35, ey - er * 0.4, max(1, er * 0.4), max(1, er * 0.4)), (255, 255, 255))
    else:
        arc = [(ex - er * 1.2 + i * er * 0.24, ey + abs(i - 5) * er * 0.12) for i in range(11)]
        cv.paint(stroke(cv, arc, max(1, er * 0.5)), (40, 26, 36))
    # 볼 홍조
    cv.paint(ellipse(cv, cx + f * s * 0.85, cy + s * 0.12, s * 0.12, s * 0.06), (236, 132, 140), 0.55)
    return sil


def silhouette_critter(cv, kind, cx, cy, s, rgb, facing=1):
    """아주 작은 실루엣(먼 배경). kind: mole·mouse·rabbit·raccoon."""
    f = facing
    m = ellipse(cv, cx, cy, s, s * 0.75)
    if kind == 'mouse':
        m |= ellipse(cv, cx + f * s * 0.8, cy - s * 0.3, s * 0.5, s * 0.45)
        m |= ellipse(cv, cx + f * s * 0.6, cy - s * 0.85, s * 0.3, s * 0.3)
        m |= stroke(cv, bezier((cx - f * s, cy), (cx - f * s * 1.8, cy), (cx - f * s * 2, cy - s), (cx - f * s * 2.3, cy - s * 0.6)), 2)
    elif kind == 'rabbit':
        m |= ellipse(cv, cx + f * s * 0.6, cy - s * 0.6, s * 0.5, s * 0.5)
        m |= ellipse(cv, cx + f * s * 0.5, cy - s * 1.5, s * 0.16, s * 0.6, rot=-0.2 * f)
        m |= ellipse(cv, cx + f * s * 0.8, cy - s * 1.45, s * 0.16, s * 0.6, rot=0.25 * f)
    elif kind == 'raccoon':
        m |= ellipse(cv, cx + f * s * 0.7, cy - s * 0.5, s * 0.55, s * 0.5)
        m |= ellipse(cv, cx + f * s * 0.5, cy - s * 1.0, s * 0.18, s * 0.22)
        m |= ellipse(cv, cx + f * s * 1.0, cy - s * 0.98, s * 0.18, s * 0.22)
        m |= ellipse(cv, cx - f * s * 1.2, cy + s * 0.1, s * 0.6, s * 0.25, rot=0.3 * f)
    else:
        m |= ellipse(cv, cx + f * s * 1.05, cy + s * 0.1, s * 0.35, s * 0.25)
    cv.paint(m, rgb)
    return m


def flower(cv, cx, cy, r, ramp=None, center=(250, 210, 120)):
    ramp = BLOSSOM if ramp is None else ramp
    m = np.zeros((cv.h, cv.w), dtype=bool)
    for k in range(5):
        a = -math.pi / 2 + k * 2 * math.pi / 5
        m |= ellipse(cv, cx + math.cos(a) * r * 0.62, cy + math.sin(a) * r * 0.62, r * 0.48, r * 0.48)
    outline(cv, m, ramp[0], 1)
    shade(cv, m, ramp[1:], cx, cy, r * 1.1, r * 1.1)
    cv.paint(ellipse(cv, cx, cy, r * 0.28, r * 0.28), center)
    return m


def blossom_cluster(cv, rng, cx, cy, r, n):
    for _ in range(n):
        a = rng.uniform(0, 6.28)
        d = rng.uniform(0, r)
        x, y = cx + math.cos(a) * d, cy + math.sin(a) * d * 0.7
        rr = rng.uniform(5, 9)
        m = ellipse(cv, x, y, rr, rr * 0.85)
        shade(cv, m, BLOSSOM, x, y, rr, rr)


# ---- 장면 -------------------------------------------------------------------

def pebbles(cv, rng, region, n, base):
    """v2 조약돌: 덩어리 하나 + 오른쪽 아래 그늘 + 왼쪽 위 2×2 빛(흩뿌린 얼룩 대신)."""
    for _ in range(n):
        x, y = rng.uniform(region[0], region[2]), rng.uniform(region[1], region[3])
        r = rng.uniform(2.5, 5.5)
        m = ellipse(cv, x, y, r, r * 0.7)
        c = rng.choice(base)
        cv.paint(m, c)
        cv.paint(m & ~ellipse(cv, x - 1.2, y - 1.0, r, r * 0.7), tuple(v * 0.62 for v in c))
        cv.paint(m & ellipse(cv, x - r * 0.35, y - r * 0.3, 1.4, 1.1), tuple(min(255, v * 1.35) for v in c))


def grass_tufts(cv, rng, y, x0, x1, ramp):
    """v2 풀: 포기마다 잎 셋(어두운 밑 → 밝은 끝)."""
    x = x0
    while x < x1:
        for k, lean in enumerate((-3, 0, 3)):
            h = rng.randint(5, 13)
            tip = (x + lean + rng.uniform(-1, 1), y - h)
            cv.paint(stroke(cv, [(x, y + 1), tip], 3, 1), ramp[1])
            cv.paint(stroke(cv, [(x + lean * 0.4, y - h * 0.45), tip], 1), ramp[3] if k == 1 else ramp[2])
        x += rng.randint(5, 9)


def rim_light(cv, mask, rgb, width=2, alpha=1.0, within=None):
    """v2 가장자리 빛: 빛 쪽(왼쪽 위)으로 비어 있는 가장자리 width px만 rgb로. within이 있으면 그 안만 칠한다
    (가장자리는 mask 전체 윤곽으로 재므로, 줄기와 가지가 이어진 곳에 선이 생기지 않는다)."""
    edge = np.zeros_like(mask)
    for i in range(1, width + 1):
        s = np.zeros_like(mask)
        s[i:, i:] = mask[:-i, :-i]   # 오른쪽 아래로 민 마스크: 원래 칸의 왼쪽 위 이웃이 비었으면 False
        edge |= mask & ~s
    if within is not None:
        edge &= within
    cv.paint(edge, rgb, alpha)


def great_tree():
    rng = random.Random(11)
    cv = Canvas(640, 560)
    ground = 300
    bands(cv, 0, ground, (46, 38, 74), (206, 150, 126), 10)
    if V2:
        # 대기 원근: 먼 산줄기 한 겹 더(하늘색 쪽으로 밝게)
        far = poly(cv, [(0, 236)] + [(x, 232 - 22 * math.sin(x / 110 + 1.3) - 9 * math.sin(x / 37)) for x in range(0, 641, 16)] + [(640, ground), (0, ground)])
        cv.paint(far, (146, 110, 128))
    # 먼 언덕
    hill = poly(cv, [(0, 262)] + [(x, 258 - 14 * math.sin(x / 70) - 8 * math.sin(x / 23)) for x in range(0, 641, 16)] + [(640, ground), (0, ground)])
    cv.paint(hill, (92, 78, 96))
    # 땅속 흙 띠
    bands(cv, ground, 560, (92, 64, 50), (30, 20, 26), 9)
    if V2:
        pebbles(cv, rng, (0, ground + 14, 640, 556), 110, [(104, 80, 66), (88, 70, 64), (120, 96, 74)])
    else:
        speckle(cv, rng, (0, ground + 10, 640, 560), 140, [(70, 50, 42), (110, 82, 62), (54, 38, 36)])
    # 풀 줄
    grass = (cv.yy >= ground - 6) & (cv.yy < ground + 4)
    if V2:
        cv.paint(grass, (72, 92, 62))
        grass_tufts(cv, rng, ground, 0, 640, hue_shift([(52, 70, 50), (74, 96, 60), (98, 122, 72), (140, 156, 90)]))
    else:
        cv.paint(grass, (88, 104, 66))
        for x in range(0, 640, 6):
            h = rng.randint(4, 12)
            cv.paint(stroke(cv, [(x, ground), (x + rng.randint(-2, 2), ground - h)], 2), (104, 124, 76))
    # 뿌리(갈색, 진짜 뿌리)
    root_paths = []
    for k in range(9):
        sx = 300 + k * 6
        ex = rng.uniform(30, 610)
        ey = rng.uniform(420, 560)
        pts = bezier((sx, ground), (sx + (ex - sx) * 0.2, ground + 60), (ex, ey - 100), (ex, ey))
        m = stroke(cv, pts, 16 - k * 0.6, 2)
        outline(cv, m, (34, 22, 24), 1)
        cv.paint(m, BARK[2])
        cv.paint(stroke(cv, [(x - 2, y) for x, y in pts], 3, 1) & m, BARK[3])
        root_paths.append(pts)
    # 보라 실이 뿌리를 감고 더 깊이 번진다
    for pts in root_paths[::2]:
        out = []
        for i, (x, y) in enumerate(pts[10:]):
            t = i / 38
            out.append((x + math.sin(t * 30) * 6, y))
        m = stroke(cv, out, 3, 1)
        if V2:   # 실 둘레 은은한 빛(땅속에서 실이 스스로 빛나는 느낌)
            cv.paint(dilate(m, 4) & ~m, VIOLET[1], 0.22)
        cv.paint(dilate(m) & ~m, VIOLET[0])
        cv.paint(m, VIOLET[2])
    for _ in range(14):
        p0 = (rng.uniform(80, 560), rng.uniform(380, 470))
        wavy_thread(cv, p0, (p0[0] + rng.uniform(-120, 120), 560), rng, 5, 2, VIOLET)
    # 어둠 속 붉은 눈
    for x, y in ((92, 520), (548, 506), (420, 540)):
        for dx in (0, 9):
            cv.paint(ellipse(cv, x + dx, y, 2.2, 1.6), (230, 60, 90))
    # 나무 줄기: 밑동이 벌어진 줄기 + 갈라지며 가늘어지는 가지(0049). 예전엔 굵기가 같은 곡선 여섯 개라 사슴뿔처럼 보였다.
    trunk = poly(cv, [(262, ground + 6), (286, ground - 20), (298, 170), (304, 96), (336, 96), (342, 170),
                      (354, ground - 20), (380, ground + 6)])
    tips = []

    def grow(x, y, ang, length, width, depth):
        """가지 하나: 끝으로 갈수록 가늘고, 늙은 나무라 끝이 살짝 처진다. 끝 쪽에서 둘(가끔 셋)로 갈라진다."""
        nonlocal trunk
        droop = 0.18 * (1 - depth / 4)
        ex, ey = x + math.cos(ang) * length, y + math.sin(ang) * length
        bend = rng.uniform(-0.25, 0.25)
        c1 = (x + math.cos(ang + bend) * length * 0.4, y + math.sin(ang + bend) * length * 0.4)
        c2 = (x + math.cos(ang + droop) * length * 0.75, y + math.sin(ang + droop) * length * 0.75 + length * droop * 0.3)
        pts = bezier((x, y), c1, c2, (ex, ey + length * droop * 0.25), 24)
        trunk |= stroke(cv, pts, width, max(1.5, width * 0.62))
        if depth == 0:
            tips.append(pts[-1])
            return
        kids = 3 if rng.random() < 0.3 else 2
        for k in range(kids):
            t = rng.uniform(0.62, 0.95)
            sx, sy = pts[int(t * (len(pts) - 1))]
            spread = rng.uniform(0.32, 0.62) * (1 if k % 2 == 0 else -1) + (0.1 if k == 2 else 0)
            grow(sx, sy, ang + spread, length * rng.uniform(0.58, 0.74), width * 0.62, depth - 1)

    for (sx, sy, ang, length, width) in ((310, 168, -2.45, 92, 14), (330, 160, -0.72, 96, 14),
                                         (318, 104, -1.95, 74, 11), (324, 100, -1.15, 78, 11),
                                         (302, 206, -2.85, 64, 9), (340, 200, -0.32, 66, 9)):
        grow(sx, sy, ang, length, width, 3)
    outline(cv, trunk, (30, 20, 24), 2)
    # 음영: 줄기는 좁은 구(통나무처럼), 가지는 넓은 구(먼 가지가 새까매지지 않게).
    branch_only = trunk & ~poly(cv, [(262, ground + 6), (286, ground - 20), (298, 170), (304, 96), (336, 96),
                                     (342, 170), (354, ground - 20), (380, ground + 6)])
    shade(cv, trunk, BARK, 322, 200, 60, 140)
    shade(cv, branch_only, BARK, 300, 110, 240, 170, bias=0.1)
    # 줄기 결
    if V2:
        # 껍질: 곧은 선 대신 짧은 갈라짐 덩어리(어두운 틈 + 오른쪽 1px 밝은 턱), 노을 쪽 가장자리 빛
        for _ in range(34):
            x = rng.uniform(296, 348)
            y = rng.uniform(110, 296)
            ln = rng.uniform(8, 22)
            crack = stroke(cv, [(x, y), (x + rng.uniform(-2, 2), y + ln)], 2) & trunk
            cv.paint(crack, BARK[0])
            cv.paint(stroke(cv, [(x + 2, y + 1), (x + 2, y + ln - 1)], 1) & trunk & ~crack, BARK[3])
        # 가는 가지까지 2px이면 주황 테두리처럼 보인다: 줄기만 2px, 가지는 1px을 옅게.
        rim_light(cv, trunk, SUNSET_RIM, 2, within=trunk & ~branch_only)
        rim_light(cv, trunk, SUNSET_RIM, 1, 0.45, within=branch_only)
    else:
        for k in range(5):
            x = 300 + k * 9
            cv.paint(stroke(cv, [(x, 290), (x + 3, 200), (x + 1, 130)], 1) & trunk, BARK[0])
    # 몇 안 남은 마른 잎(가지 끝 몇 군데에만)
    for (x, y) in tips:
        if rng.random() < 0.35:
            m = ellipse(cv, x + rng.uniform(-3, 3), y + rng.uniform(1, 5), 5, 3, rot=rng.uniform(0, 3))
            cv.paint(dilate(m) & ~m, (70, 46, 34))
            cv.paint(m, rng.choice([(176, 132, 74), (150, 104, 60), (196, 156, 92)]))
    return cv


def mother_burrow():
    rng = random.Random(22)
    cv = Canvas(640, 360)
    bands(cv, 0, 360, (60, 42, 38), (28, 20, 24), 6)
    speckle(cv, rng, (0, 0, 640, 360), 160, [(48, 34, 32), (82, 60, 50), (40, 28, 30)])
    # 굴 방(밝은 흙)과 오른쪽 위로 난 굴길
    room = ellipse(cv, 260, 190, 230, 120)
    tunnel = stroke(cv, bezier((380, 170), (470, 150), (520, 90), (600, 0)), 90, 70)
    cav = room | tunnel
    # 굴 안은 밝은 흙(아래로 어두워짐), 밖은 그대로 어두운 흙.
    lit = Canvas(640, 360)
    bands(lit, 60, 360, (150, 112, 84), (90, 66, 56), 7)
    cv.px[cav] = lit.px[cav]
    outline(cv, cav, (36, 24, 26), 3)
    # 굴길 끝의 빛: 굴길 안만 위로 갈수록 밝게(넓은 계단 셋).
    for i, (r, al) in enumerate(((260, 0.10), (180, 0.14), (110, 0.2))):
        cv.paint(ellipse(cv, 640, -20, r, r) & tunnel, (255, 238, 196), al)
    # 바닥 둥지(마른 풀)
    nest = ellipse(cv, 170, 268, 130, 34) & cav
    cv.paint(nest, (132, 104, 64))
    for _ in range(70):
        x = rng.uniform(60, 290)
        y = rng.uniform(250, 290)
        a = rng.uniform(-0.6, 0.6)
        pts = [(x, y), (x + math.cos(a) * 18, y + math.sin(a) * 6)]
        cv.paint(stroke(cv, pts, 2) & cav, rng.choice([(196, 166, 96), (166, 132, 76), (214, 186, 118)]))
    # 엄마(크게, 뒤) — 코로 Naïl을 민다
    mole(cv, 210, 180, 92, facing=1, eyes='closed', ramp=MOM_FUR, paws=True)
    # Naïl(작게, 앞) — 굴길 쪽
    mole(cv, 372, 196, 40, facing=1, eyes='open')
    # 엄마 코끝과 Naïl 엉덩이 사이 작은 흙먼지
    for k in range(4):
        cv.paint(ellipse(cv, 330 + k * 4, 214 - k * 5, 3, 2), (180, 150, 120))
    return cv


def bori_freed():
    rng = random.Random(33)
    cv = Canvas(640, 360)
    bands(cv, 0, 200, (112, 96, 128), (220, 176, 140), 8)
    bands(cv, 200, 360, (106, 128, 74), (60, 84, 52), 6)
    for _ in range(60):
        x, y = rng.uniform(0, 640), rng.uniform(210, 360)
        flower(cv, x, y, rng.uniform(2, 4), [(150, 120, 150), (230, 200, 230), (250, 240, 250), (255, 255, 255)], (250, 220, 120))
    # 공터 둘레 덤불(지평선 위, 어두운 초록)
    for (x, r) in ((30, 80), (130, 60), (510, 64), (610, 84)):
        bush = ellipse(cv, x, 206, r, r * 0.55) & (cv.yy <= 210)
        cv.paint(bush, (64, 84, 62))
        cv.paint(ellipse(cv, x - r * 0.3, 190, r * 0.4, r * 0.2) & bush, (84, 106, 74))
    # 보리(들쥐 대장): 앉아서 눈을 감고 숨을 고른다
    cx, cy = 320, 190
    body = ellipse(cv, cx, cy + 10, 78, 62)
    head = ellipse(cv, cx + 14, cy - 58, 54, 48)
    ears = ellipse(cv, cx - 30, cy - 104, 30, 30) | ellipse(cv, cx + 60, cy - 104, 30, 30)
    tail = stroke(cv, bezier((cx - 70, cy + 50), (cx - 150, cy + 60), (cx - 170, cy - 10), (cx - 140, cy - 40)), 8, 3)
    snout = ellipse(cv, cx + 52, cy - 44, 24, 18)
    paws = ellipse(cv, cx + 30, cy + 30, 20, 14) | ellipse(cv, cx - 10, cy + 36, 20, 14)
    furm = tufted(cv, body | head, 90, 1)
    sil = furm | ears | tail | snout | paws
    lit_outline(cv, sil, furm, RFUR, r=2)
    shade(cv, tail, PINK[:3])
    shade(cv, furm, RFUR, cx, cy - 20, 100, 110)
    shade(cv, ears, RFUR)
    cv.paint(ellipse(cv, cx - 30, cy - 102, 18, 18) | ellipse(cv, cx + 60, cy - 102, 18, 18), PINK[1])
    shade(cv, snout, RMUZ)
    cv.paint(ellipse(cv, cx + 74, cy - 48, 7, 6), PINK[0])
    shade(cv, paws, RMUZ, bias=0.2)
    # 흉터
    cv.paint(stroke(cv, [(cx - 12, cy - 86), (cx + 6, cy - 70)], 3), (226, 200, 188))
    # 감은 눈(편안)
    for ex in (cx + 20,):
        arc = [(ex - 9 + i * 1.8, cy - 62 + abs(i - 5) * 0.9) for i in range(11)]
        cv.paint(stroke(cv, arc, 3), (40, 26, 30))
    # 풀려 올라가는 보라 실: 몸 가장자리에서 바깥 위로, 끝은 끊겨 반짝이가 된다(얼굴은 가리지 않는다).
    for k in range(7):
        side = -1 if k % 2 == 0 else 1
        p0 = (cx + side * rng.uniform(60, 80), cy + rng.uniform(-10, 50))
        p3 = (p0[0] + side * rng.uniform(40, 110), rng.uniform(20, 90))
        pts = bezier(p0, (p0[0] + side * 30, p0[1] - 30), (p3[0] - side * 20, p3[1] + 50), p3, n=40)
        pts = [(x + math.sin(i * 0.5) * 3, y) for i, (x, y) in enumerate(pts)]
        m = stroke(cv, pts[:26], 2, 1)
        cv.paint(dilate(m) & ~m, VIOLET[0])
        cv.paint(m, VIOLET[2])
        for (x, y) in pts[28::4]:
            cv.paint(ellipse(cv, x, y, 1.6, 1.6), (230, 210, 255))
    return cv


def momo_bloom():
    rng = random.Random(44)
    cv = Canvas(640, 360)
    bands(cv, 0, 360, (90, 70, 110), (210, 160, 180), 9)
    # 뒤 빛망울
    for _ in range(26):
        x, y, r = rng.uniform(0, 640), rng.uniform(0, 300), rng.uniform(8, 26)
        cv.paint(ellipse(cv, x, y, r, r), (255, 230, 240), 0.12)
    cx, cy = 320, 176
    # 귀(길게, 위로)
    ear_l = ellipse(cv, cx - 44, cy - 128, 24, 84, rot=-0.18)
    ear_r = ellipse(cv, cx + 40, cy - 124, 24, 84, rot=0.22)
    head = ellipse(cv, cx, cy - 6, 92, 80)
    cheeks = ellipse(cv, cx - 50, cy + 30, 46, 34) | ellipse(cv, cx + 50, cy + 30, 46, 34)
    # 몸: 머리 밑으로 이어지는 어깨(목 없이 한 덩어리 — 공 두 개를 쌓은 눈사람처럼 보이지 않게, 0049)
    chest = ellipse(cv, cx, cy + 116, 128, 74) | poly(cv, [(cx - 92, cy + 30), (cx + 92, cy + 30), (cx + 120, cy + 100), (cx - 120, cy + 100)])
    furm = tufted(cv, head | cheeks | chest, 100, 0)
    extra = furm & ~(head | cheeks | chest)
    chest = chest | extra
    sil = ear_l | ear_r | furm
    lit_outline(cv, sil, furm, RABBIT, r=2)
    shade(cv, chest, RABBIT, cx, cy + 110, 130, 80)
    shade(cv, ear_l, RABBIT)
    shade(cv, ear_r, RABBIT)
    cv.paint(ellipse(cv, cx - 44, cy - 124, 11, 66, rot=-0.18) | ellipse(cv, cx + 40, cy - 120, 11, 66, rot=0.22), PINK[1])
    # 가슴 털(밝은 털 덩어리, 위가 들쭉날쭉) + 앞발 둘
    bib = ellipse(cv, cx, cy + 112, 58, 46)
    shade(cv, bib, RABBIT_LIGHT, cx, cy + 100, 70, 60)
    for k in range(7):
        x = cx - 48 + k * 16
        cv.paint(poly(cv, [(x - 8, cy + 76), (x + 8, cy + 76), (x, cy + 66)]), RABBIT_LIGHT[1])
    for px in (cx - 34, cx + 34):
        paw = ellipse(cv, px, cy + 150, 22, 15)
        outline(cv, paw, r=2)
        shade(cv, paw, RABBIT_LIGHT, px, cy + 146, 24, 16)
        for k in (-8, 0, 8):
            cv.paint(stroke(cv, [(px + k, cy + 141), (px + k, cy + 150)], 1), RABBIT[1])
    shade(cv, head | cheeks, RABBIT, cx, cy, 100, 90)
    muzzle = ellipse(cv, cx, cy + 36, 40, 28)
    shade(cv, muzzle, RABBIT_LIGHT)
    cv.paint(poly(cv, [(cx - 8, cy + 22), (cx + 8, cy + 22), (cx, cy + 32)]), PINK[1])
    cv.paint(stroke(cv, [(cx, cy + 32), (cx, cy + 40), (cx - 8, cy + 46)], 2) | stroke(cv, [(cx, cy + 40), (cx + 8, cy + 46)], 2), (96, 60, 64))
    # 눈: 초록 빛이 걷힌 맑은 검은 눈(스프라이트·초상화와 같다, 0046·0049). 아래쪽에 따뜻한 반사광.
    for ex in (cx - 38, cx + 38):
        cv.paint(ellipse(cv, ex, cy - 6, 14, 17), (22, 14, 30))
        cv.paint(ellipse(cv, ex, cy + 4, 10, 7), (70, 56, 88))
        cv.paint(ellipse(cv, ex, cy + 7, 7, 4), (120, 100, 132))
        cv.paint(ellipse(cv, ex - 5, cy - 11, 5, 5), (255, 255, 255))
        cv.paint(ellipse(cv, ex + 6, cy + 2, 2, 2), (255, 255, 255))
        # 속눈썹(바깥 눈꼬리)
        side = -1 if ex < cx else 1
        cv.paint(stroke(cv, [(ex + side * 12, cy - 16), (ex + side * 18, cy - 20)], 2), (22, 14, 30))
    # 볼 홍조
    for ex in (cx - 62, cx + 62):
        cv.paint(ellipse(cv, ex, cy + 26, 14, 7), (240, 140, 150), 0.5)
    # 귀 밑동의 꽃(활짝)
    flower(cv, cx + 66, cy - 74, 22)
    flower(cv, cx + 92, cy - 58, 13)
    for k in range(3):
        cv.paint(ellipse(cv, cx + 52 + k * 12, cy - 50 + k * 4, 7, 4, rot=0.6), GREEN[2])
    # 떨어져 나가는 마지막 보라 실
    for _ in range(5):
        p0 = (cx + rng.uniform(-90, 90), cy + rng.uniform(60, 110))
        wavy_thread(cv, p0, (p0[0] + rng.uniform(-60, 60), 360), rng, 3, 2, VIOLET)
    return cv


def deep_place(rng):
    """가장 깊은 곳: 사방으로 뻗은 실 + 할매(실 덩어리와 버섯 갓). halmae_deep·momo_deep이 같이 쓴다."""
    cv = Canvas(704, 360)
    bands(cv, 0, 360, (16, 14, 30), (34, 24, 44), 6)
    speckle(cv, rng, (0, 0, 704, 360), 90, [(28, 22, 40), (40, 30, 52)])
    cx, cy = 384, 150
    glow(cv, cx, cy, 260, (200, 180, 230), 0.45, 6)
    # 사방으로 뻗은 실(흰빛 + 보라)
    for k in range(46):
        a = k / 46 * 6.28 + rng.uniform(-0.05, 0.05)
        d = rng.uniform(380, 520)
        end = (cx + math.cos(a) * d, cy + math.sin(a) * d * 0.7)
        p0 = (cx + math.cos(a) * 60, cy + math.sin(a) * 40)
        ramp = THREAD_WHITE if k % 3 else VIOLET
        pts = bezier(p0, (p0[0] + rng.uniform(-50, 50), p0[1] + rng.uniform(-30, 30)),
                     (end[0] + rng.uniform(-80, 80), end[1] + rng.uniform(-60, 60)), end, n=60)
        m = stroke(cv, pts, 4 if k % 5 == 0 else 2, 1)
        cv.paint(m, ramp[1] if k % 3 else ramp[2])
    # 할매: 둥글게 엉킨 실 덩어리 + 버섯 갓 무리
    mass = ellipse(cv, cx, cy + 20, 120, 80)
    outline(cv, mass, (60, 50, 80), 2)
    shade(cv, mass, [(110, 100, 140), (150, 140, 176), (190, 182, 210), (226, 222, 240)], cx, cy + 10, 130, 90)
    for _ in range(80):
        a = rng.uniform(0, 6.28)
        d = rng.uniform(10, 110)
        x, y = cx + math.cos(a) * d, cy + 20 + math.sin(a) * d * 0.65
        cv.paint(stroke(cv, bezier((x, y), (x + 8, y - 6), (x + 14, y + 6), (x + 20, y)), 1) & mass, (240, 238, 250))
    caps = [(cx - 70, cy - 46, 34), (cx - 20, cy - 70, 46), (cx + 46, cy - 52, 38), (cx + 92, cy - 20, 26), (cx - 104, cy - 6, 24)]
    for (x, y, r) in caps:
        stem = poly(cv, [(x - r * 0.25, y), (x + r * 0.25, y), (x + r * 0.3, y + r * 1.1), (x - r * 0.3, y + r * 1.1)])
        cap = ellipse(cv, x, y, r, r * 0.55) & (cv.yy <= y + r * 0.12)
        outline(cv, stem | cap, (54, 40, 40), 2)
        shade(cv, stem, [(170, 156, 140), (210, 198, 180), (236, 228, 214)])
        shade(cv, cap, CAP, x, y, r, r * 0.6)
        for _ in range(4):
            cv.paint(ellipse(cv, x + rng.uniform(-r * 0.6, r * 0.6), y - rng.uniform(0, r * 0.3), 2.5, 2), CAP[0])
    # 다정한 얼굴(가장 큰 갓 아래 덩어리에): 감은 눈 + 작은 웃음
    fx, fy = cx - 6, cy + 24
    for ex in (fx - 26, fx + 26):
        arc = [(ex - 10 + i * 2, fy - abs(i - 5) * 0.9) for i in range(11)]
        cv.paint(stroke(cv, arc, 3), (70, 56, 90))
    smile = [(fx - 10 + i * 2, fy + 22 + (5 - abs(i - 5)) * 0.8) for i in range(11)]
    cv.paint(stroke(cv, smile, 3), (70, 56, 90))
    for ex in (fx - 40, fx + 40):
        cv.paint(ellipse(cv, ex, fy + 14, 9, 5), (236, 160, 176), 0.6)
    return cv


def halmae_deep():
    cv = deep_place(random.Random(55))
    # 작은 Naïl(왼쪽 아래, 할매를 올려다본다)
    mole(cv, 210, 236, 22, facing=1, eyes='open')
    return cv


def rabbit(cv, cx, cy, s, facing=1, eyes='open'):
    """작은 굴토끼(모모): 둥근 몸 + 머리 + 긴 귀 둘, 귀 밑동에 꽃. s = 몸 가로 반지름."""
    f = facing
    body = ellipse(cv, cx, cy, s, s * 0.82)
    head = ellipse(cv, cx + f * s * 0.5, cy - s * 0.55, s * 0.62, s * 0.56)
    ear_a = ellipse(cv, cx + f * s * 0.28, cy - s * 1.55, s * 0.2, s * 0.68, rot=-0.22 * f)
    ear_b = ellipse(cv, cx + f * s * 0.72, cy - s * 1.5, s * 0.2, s * 0.68, rot=0.28 * f)
    tail = ellipse(cv, cx - f * s * 0.95, cy + s * 0.1, s * 0.26, s * 0.24)
    furm = tufted(cv, body | head, s, f)
    body, head = furm, furm
    sil = body | head | ear_a | ear_b | tail
    lit_outline(cv, sil, furm, RABBIT, r=1)
    shade(cv, body | head | ear_a | ear_b, RABBIT, cx + f * s * 0.2, cy - s * 0.3, s * 1.3, s * 1.2)
    cv.paint(ellipse(cv, cx + f * s * 0.28, cy - s * 1.5, s * 0.08, s * 0.48, rot=-0.22 * f)
             | ellipse(cv, cx + f * s * 0.72, cy - s * 1.45, s * 0.08, s * 0.48, rot=0.28 * f), PINK[1])
    shade(cv, tail, RABBIT_LIGHT)
    cv.paint(ellipse(cv, cx + f * s * 0.35, cy + s * 0.25, s * 0.42, s * 0.36), RABBIT_LIGHT[1])
    ex, ey = cx + f * s * 0.78, cy - s * 0.62
    er = max(1.5, s * 0.09)
    if eyes == 'open':
        cv.paint(ellipse(cv, ex, ey, er, er * 1.15), (22, 14, 30))
        cv.paint(ellipse(cv, ex - er * 0.35, ey - er * 0.45, max(1, er * 0.4), max(1, er * 0.4)), (255, 255, 255))
    cv.paint(ellipse(cv, cx + f * s * 1.08, cy - s * 0.45, s * 0.08, s * 0.07), PINK[0])
    cv.paint(ellipse(cv, cx + f * s * 0.82, cy - s * 0.32, s * 0.12, s * 0.06), (236, 132, 140), 0.55)
    flower(cv, cx + f * s * 0.5, cy - s * 1.05, s * 0.24)
    return sil


def momo_deep():
    """모모로 가장 깊은 곳(0056): 실에 걸린 굴토끼 털 뭉치와 시든 꽃잎, 그것을 보는 작은 모모."""
    rng = random.Random(55)
    cv = deep_place(rng)
    # 실에 걸린 털 뭉치(굴토끼 가족)
    for (x, y) in ((132, 92), (560, 70), (610, 238), (470, 300), (88, 300)):
        for _ in range(7):
            cv.paint(ellipse(cv, x + rng.uniform(-6, 6), y + rng.uniform(-4, 4), rng.uniform(3, 5), rng.uniform(2, 4)),
                     rng.choice(RABBIT_LIGHT))
        cv.paint(ellipse(cv, x, y, 3, 2), RABBIT[3])
    # 시든 꽃잎 하나: 실 끝에 늘어져 매달렸다(가장자리가 갈색)
    px, py = 262, 176
    cv.paint(stroke(cv, [(px - 18, py - 40), (px - 6, py - 18), (px, py - 6)], 1), THREAD_WHITE[2])
    petal = ellipse(cv, px + 2, py + 6, 7, 12, rot=0.5)
    outline(cv, petal, (90, 50, 60), 1)
    shade(cv, petal, [(150, 96, 100), (196, 140, 146), (226, 176, 180)], px, py, 8, 12)
    cv.paint(ellipse(cv, px + 6, py + 14, 3, 3) & petal, (140, 100, 80))
    # 작은 모모(왼쪽 아래, 꽃잎을 올려다본다)
    rabbit(cv, 210, 250, 20, facing=1)
    return cv


def last_bloom():
    rng = random.Random(66)
    cv = Canvas(640, 440)
    bands(cv, 0, 340, (250, 212, 170), (176, 196, 222), 9)
    bands(cv, 340, 440, (120, 150, 86), (78, 108, 62), 5)
    glow(cv, 520, 60, 140, (255, 250, 220), 0.5, 5)
    trunk = poly(cv, [(286, 350), (300, 200), (306, 140), (334, 140), (340, 200), (356, 350)])
    for (sx, sy, ex, ey, w) in ((310, 190, 160, 110, 14), (330, 180, 480, 100, 14), (318, 150, 240, 50, 10),
                                (324, 146, 410, 40, 10)):
        trunk |= stroke(cv, bezier((sx, sy), ((sx + ex) / 2, sy - 10), (ex, (sy + ey) / 2), (ex, ey)), w, 3)
    outline(cv, trunk, (40, 28, 30), 2)
    shade(cv, trunk, BARK, 322, 240, 60, 140)
    # 꽃 구름
    for (x, y, r, n) in ((320, 90, 150, 160), (160, 120, 80, 70), (480, 110, 90, 80), (250, 50, 70, 50), (400, 40, 80, 60)):
        blossom_cluster(cv, rng, x, y, r, n)
    # 밑동의 이웃들(실루엣): 두더지·라쿤·들쥐·굴토끼
    dark = (70, 72, 70)
    silhouette_critter(cv, 'raccoon', 230, 336, 16, dark, 1)
    silhouette_critter(cv, 'mouse', 410, 340, 12, dark, -1)
    silhouette_critter(cv, 'rabbit', 370, 338, 12, dark, -1)
    mole(cv, 296, 340, 16, facing=1)
    for _ in range(40):
        x, y = rng.uniform(0, 640), rng.uniform(340, 440)
        flower(cv, x, y, rng.uniform(2, 4), BLOSSOM, (250, 220, 140))
    return cv


def sprout():
    rng = random.Random(77)
    cv = Canvas(640, 360)
    bands(cv, 0, 360, (54, 40, 40), (22, 16, 22), 7)
    speckle(cv, rng, (0, 0, 640, 360), 120, [(44, 32, 34), (70, 52, 46)])
    glow(cv, 320, 0, 220, (255, 240, 200), 0.3, 5)
    cx, cy = 320, 200
    glow(cv, cx, cy - 20, 150, (230, 240, 230), 0.45, 5)
    # 흰 실: 아래에서 올라와 씨앗을 손처럼 감싼다
    for k in range(9):
        a = math.pi * (0.15 + 0.7 * k / 8)
        p3 = (cx + math.cos(a) * 46, cy + 10 + math.sin(a) * 12)
        p0 = (cx + (k - 4) * 70, 360)
        pts = bezier(p0, (p0[0], 300), (p3[0] + (k - 4) * 10, cy + 70), p3)
        m = stroke(cv, pts, 4, 2)
        cv.paint(dilate(m) & ~m, THREAD_WHITE[0])
        cv.paint(m, THREAD_WHITE[2])
        cv.paint(stroke(cv, [(x - 1, y) for x, y in pts[::3]], 1) & m, THREAD_WHITE[3])
    # 씨앗(갈라진 껍질)
    seed = ellipse(cv, cx, cy, 36, 26)
    outline(cv, seed, (40, 26, 22), 2)
    shade(cv, seed, [(90, 56, 36), (130, 86, 52), (170, 120, 72), (204, 158, 98)])
    cv.paint(stroke(cv, [(cx - 6, cy - 26), (cx + 2, cy - 12), (cx - 4, cy)], 2), (60, 38, 28))
    # 새싹: 줄기 + 떡잎 둘
    stem = stroke(cv, bezier((cx, cy - 20), (cx - 4, cy - 50), (cx + 6, cy - 80), (cx, cy - 110)), 6, 4)
    leaf_l = ellipse(cv, cx - 30, cy - 120, 30, 13, rot=-0.45)
    leaf_r = ellipse(cv, cx + 30, cy - 124, 30, 13, rot=0.45)
    sp = stem | leaf_l | leaf_r
    outline(cv, sp, (30, 52, 34), 2)
    shade(cv, stem, GREEN[1:])
    shade(cv, leaf_l, GREEN)
    shade(cv, leaf_r, GREEN)
    cv.paint(stroke(cv, [(cx - 52, cy - 110), (cx - 10, cy - 124)], 1) & leaf_l, GREEN[0])
    cv.paint(stroke(cv, [(cx + 52, cy - 114), (cx + 10, cy - 128)], 1) & leaf_r, GREEN[0])
    return cv


SOURCES = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'sources')


def pastel_bg(cv, name, colors=18, edge=30, margin=0.04):
    """바탕을 파스텔·수채 그림에서(0073): tools/pixel의 --cel과 같은 길(단순화 → 면 나누기 → 다수결 축소 → 큰 면 1px 선).
    원본은 tools/art/sources/(CC0·퍼블릭 도메인, 출처는 assets/LICENSES.md). 인물과 같은 칸 크기로 만든다(믹셀 피하기)."""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'pixel'))
    from pxl import cel_pixelate, prepare
    src = prepare(Image.open(os.path.join(SOURCES, name)).convert('RGB'), cv.w, cv.h, margin, 1.1, 1.1)
    img, _ = cel_pixelate(src, cv.w, cv.h, colors, edge)
    cv.px[:] = np.asarray(img, dtype=np.float32)


def ambient_tint(cv, base, amount=0.15):
    """바탕 위에 얹은 것(바탕과 달라진 칸)에 바탕의 평균 빛을 조금 섞는다: 그림 속 공기에 함께 들어가 스티커처럼 뜨지 않게."""
    changed = np.abs(cv.px - base).sum(-1) > 0
    amb = base.reshape(-1, 3).mean(0)
    cv.px[changed] = cv.px[changed] * (1 - amount) + amb * amount


def ground_shadow(cv, cx, cy, rx, ry, k=0.62):
    """인물 발밑 그림자: 바탕을 어둡게 눌러 땅에 붙인다."""
    m = ellipse(cv, cx, cy, rx, ry)
    cv.px[m] = cv.px[m] * k


def molehill():
    if V4:
        return molehill_pastel()
    return molehill_v1()


def molehill_pastel():
    """흙더미(0073): 바탕은 Erzsébet Korb「도나우 풍경」(파스텔, ca. 1920–22, CC0). 길 끝 언덕에 마을 둥근 문,
    길 끝에 새싹이 자란 어린 나무, 앞쪽 들판에 새 두더지 둔덕, 왼쪽 길섶에서 고개 내민 엄마 두더지."""
    rng = random.Random(88)
    cv = Canvas(640, 360)
    pastel_bg(cv, 'korb_danube.jpg')
    base = cv.px.copy()
    # 마을: 길 끝 언덕(지평선 아래 들판 띠)에 둥근 문·창
    for (x, y) in ((206, 238), (258, 230), (384, 230), (444, 240)):
        ground_shadow(cv, x, y + 8, 18, 3)
        door = ellipse(cv, x, y, 12, 14) & (cv.yy <= y + 7)
        outline(cv, door, (60, 50, 40), 1)
        cv.paint(door, (110, 74, 52))
        cv.paint(ellipse(cv, x, y - 2, 6, 7) & door, (84, 54, 40))
        cv.paint(ellipse(cv, x + 22, y - 4, 4, 4), (255, 220, 140))
    # 어린 나무: 길이 모이는 곳
    tx, ty = 316, 246
    ground_shadow(cv, tx, ty + 2, 22, 4)
    trunk = stroke(cv, bezier((tx, ty), (tx - 4, ty - 40), (tx + 6, ty - 80), (tx, ty - 120)), 8, 4)
    outline(cv, trunk, (50, 36, 30), 1)
    shade(cv, trunk, BARK, tx, ty - 60, 10, 60)
    crown = [(tx - 20, ty - 120, 26), (tx + 22, ty - 128, 28), (tx, ty - 148, 26), (tx - 34, ty - 96, 18), (tx + 36, ty - 100, 18)]
    cm = np.zeros((cv.h, cv.w), bool)
    for (x, y, r) in crown:
        cm |= ellipse(cv, x, y, r, r * 0.8)
    cm = tufted(cv, cm, 40, 0)                # 잎도 털처럼: 가장자리 삐침 + 단 경계 톱니(매끈한 공은 풍선으로 읽힌다)
    outline(cv, cm, (40, 70, 44), 1)
    shade(cv, cm, GREEN, tx, ty - 126, 52, 40, clumps=True)
    for _ in range(6):
        flower(cv, rng.uniform(tx - 40, tx + 40), rng.uniform(ty - 160, ty - 100), 4, BLOSSOM, (250, 220, 140))
    # 앞쪽 새 두더지 둔덕(오른쪽 들판)
    ground_shadow(cv, 476, 306, 76, 8)
    hillm = ellipse(cv, 470, 300, 70, 34) & (cv.yy <= 304)
    outline(cv, hillm, (60, 40, 32), 2)
    shade(cv, hillm, [(98, 66, 48), (132, 92, 64), (164, 120, 84), (190, 150, 108)], 470, 300, 70, 40)
    speckle(cv, rng, (420, 272, 520, 300), 14, [(84, 56, 42), (150, 110, 80)], size=(2, 3))
    # 왼쪽 길섶, 땅에서 고개를 내민 엄마 두더지 — 둔덕을 바라본다
    hole = ellipse(cv, 176, 246, 28, 8)
    cv.paint(hole, (70, 46, 34))
    mole(cv, 176, 234, 14, facing=1, eyes='open', ramp=MOM_FUR, paws=False)
    cv.paint(ellipse(cv, 176, 248, 32, 7) & (cv.yy >= 246), (96, 66, 48))
    for _ in range(18):
        x, y = rng.uniform(0, 640), rng.uniform(262, 360)
        flower(cv, x, y, rng.uniform(2, 3.5), [(200, 180, 90), (240, 220, 120), (255, 240, 170), (255, 250, 220)], (240, 160, 80))
    ambient_tint(cv, base)
    return cv


def molehill_v1():
    rng = random.Random(88)
    cv = Canvas(640, 360)
    bands(cv, 0, 200, (150, 196, 230), (236, 232, 206), 8)
    # 먼 언덕과 마을이 들어선 둔덕
    far = poly(cv, [(0, 180)] + [(x, 170 - 12 * math.sin(x / 90)) for x in range(0, 641, 16)] + [(640, 220), (0, 220)])
    cv.paint(far, (150, 178, 120))
    bank = ellipse(cv, 330, 250, 260, 90)
    cv.paint(bank, (112, 146, 84))
    bands(cv, 220, 360, (118, 152, 86), (84, 120, 62), 5)
    cv.paint(bank & (cv.yy < 250), (124, 160, 90))
    # 마을: 둥근 문·창
    for (x, y) in ((230, 214), (300, 224), (400, 216), (460, 230)):
        door = ellipse(cv, x, y, 12, 14) & (cv.yy <= y + 6)
        outline(cv, door, (60, 50, 40), 1)
        cv.paint(door, (110, 74, 52))
        cv.paint(ellipse(cv, x + 24, y - 4, 5, 5), (255, 220, 140))
    # 어린 나무(새싹이 자란 것)
    trunk = stroke(cv, bezier((330, 200), (326, 160), (336, 120), (330, 80)), 8, 4)
    outline(cv, trunk, (50, 36, 30), 1)
    cv.paint(trunk, BARK[2])
    for (x, y, r) in ((310, 80, 26), (352, 72, 28), (330, 52, 26), (296, 104, 18), (366, 100, 18)):
        m = ellipse(cv, x, y, r, r * 0.8)
        outline(cv, m, (40, 70, 44), 1)
        shade(cv, m, GREEN, x, y, r, r * 0.8)
    for _ in range(6):
        flower(cv, rng.uniform(296, 366), rng.uniform(50, 110), 4, BLOSSOM, (250, 220, 140))
    # 앞쪽 새 두더지 둔덕
    hillm = ellipse(cv, 470, 300, 70, 34) & (cv.yy <= 304)
    outline(cv, hillm, (60, 40, 32), 2)
    shade(cv, hillm, [(98, 66, 48), (132, 92, 64), (164, 120, 84), (190, 150, 108)], 470, 300, 70, 40)
    speckle(cv, rng, (420, 272, 520, 300), 14, [(84, 56, 42), (150, 110, 80)], size=(2, 3))
    # 멀리 왼쪽, 땅에서 고개를 내민 엄마 두더지(작게) — 둔덕을 바라본다
    m = ellipse(cv, 120, 236, 26, 14) & (cv.yy <= 238)
    cv.paint(m, (100, 76, 56))
    mole(cv, 120, 226, 13, facing=1, eyes='open', ramp=MOM_FUR, paws=False)
    cv.paint(ellipse(cv, 120, 238, 30, 8) & (cv.yy >= 236), (96, 66, 48))
    for _ in range(40):
        x, y = rng.uniform(0, 640), rng.uniform(240, 360)
        flower(cv, x, y, rng.uniform(2, 3.5), [(200, 180, 90), (240, 220, 120), (255, 240, 170), (255, 250, 220)], (240, 160, 80))
    return cv


def doto_memory():
    """도토가 기억해 낸다(0053): 아주 오래전, 가장 깊은 곳. 어린 도토가 흰 실 곁에 빛나는 씨앗을 묻는다.
    기억이라 색을 바래게(따뜻한 세피아) 하고 가장자리를 어둡게 눌렀다. 중요한 것(도토·씨앗)은 y 248 위."""
    rng = random.Random(99)
    cv = Canvas(640, 360)
    bands(cv, 0, 360, (58, 44, 36), (28, 20, 20), 7)
    speckle(cv, rng, (0, 0, 640, 360), 110, [(48, 36, 30), (72, 56, 44)])
    sx, sy = 372, 206
    glow(cv, sx, sy, 200, (250, 226, 170), 0.2, 5)
    # 흰 실: 오른쪽 어둠에서 와서 씨앗 자리 둘레에 닿는다(누룩 할매의 실 — 그때는 흰빛)
    for k in range(9):
        y0 = 40 + k * 36 + rng.uniform(-10, 10)
        end = (sx + 26 + rng.uniform(-6, 10), sy + 6 + (k - 4) * 4)
        wavy_thread(cv, (660, y0), end, rng, 3, 2 if k % 3 else 3, THREAD_WHITE)
    # 판 구덩이와 흙더미
    hole = ellipse(cv, sx, sy + 12, 34, 11)
    cv.paint(hole, (30, 20, 18))
    heap = ellipse(cv, sx - 64, sy + 22, 30, 14) & (cv.yy <= sy + 26)
    outline(cv, heap, (34, 24, 20), 2)
    shade(cv, heap, [(76, 54, 40), (104, 76, 54), (132, 98, 68)])
    # 씨앗(빛난다)
    glow(cv, sx, sy + 4, 34, (255, 244, 200), 0.6, 4)
    seed = ellipse(cv, sx, sy + 4, 13, 10)
    outline(cv, seed, (60, 38, 24), 2)
    shade(cv, seed, [(150, 104, 60), (196, 150, 92), (232, 196, 130), (250, 232, 180)])
    # 꼬리(뒤): 몸 뒤에서 솟아 머리 위로 말린 굵은 털 곡선 하나(공을 쌓으면 눈사람처럼 보였다)
    tpts = bezier((262, 226), (150, 200), (150, 70), (262, 64), n=80)
    tail = stroke(cv, tpts, 52, 40)
    tail |= ellipse(cv, 258, 72, 34, 26)
    for i in range(0, len(tpts), 4):                 # 털 끝이 삐죽(바깥쪽으로 작은 털 뭉치)
        x, y = tpts[i]
        tail |= ellipse(cv, x + rng.uniform(-22, 22), y + rng.uniform(-22, 22), 9, 7)
    tail = tufted(cv, tail, 60, 1)
    lit_outline(cv, tail, tail, DFUR, 2)
    shade(cv, tail, DFUR, 200, 140, 80, 110, bias=-0.08)
    for i in range(6, len(tpts) - 4, 5):
        x, y = tpts[i]
        m = stroke(cv, bezier((x - 10, y + 6), (x - 4, y), (x + 4, y - 2), (x + 12, y + 2)), 1) & tail
        cv.paint(m, DFUR[1])
    # 몸·목·머리를 한 덩어리로, 귀는 모자 뒤로 솟는다
    body = ellipse(cv, 286, 198, 48, 38, rot=0.2)
    neck = ellipse(cv, 312, 170, 28, 24)
    head = ellipse(cv, 330, 146, 30, 26)
    ear = ellipse(cv, 340, 104, 8, 18, rot=0.2)
    furm = tufted(cv, body | neck | head, 50, 1)
    fig = furm | ear
    lit_outline(cv, fig, furm, DFUR, 2)
    shade(cv, fig & ~ear, DFUR, 300, 170, 70, 64, bias=0.05)
    shade(cv, ear, DFUR)
    cv.paint(ellipse(cv, 341, 108, 3, 10, rot=0.2), (180, 110, 110))
    belly = ellipse(cv, 318, 200, 20, 26, rot=0.2) & body
    shade(cv, belly, DLIGHT)
    muzzle = ellipse(cv, 352, 156, 14, 11) & head
    shade(cv, muzzle, DLIGHT)
    cv.paint(ellipse(cv, 364, 151, 4, 3), (180, 100, 110))
    # 귀 술
    for k in range(4):
        cv.paint(stroke(cv, [(338 + k * 2, 88), (337 + k * 3, 76 - k)], 2), DFUR[0])
    # 도토리 갓 모자
    cap = ellipse(cv, 322, 118, 26, 11) & (cv.yy <= 121)
    stem = stroke(cv, [(322, 108), (324, 100), (328, 96)], 3, 2)
    outline(cv, cap | stem, (40, 26, 20), 2)
    shade(cv, cap, ACORN, 322, 114, 26, 10)
    cv.paint(stem, ACORN[1])
    for (x, y) in ((306, 113), (314, 110), (322, 109), (330, 110), (338, 113), (310, 117), (326, 116), (318, 114)):
        cv.paint(ellipse(cv, x, y, 2, 1.5), ACORN[0])
    # 앞발 둘: 씨앗 위로 흙을 덮는다
    for (px, py) in ((346, 198), (360, 192)):
        paw = ellipse(cv, px, py, 9, 7)
        outline(cv, paw, OUTLINE, 1)
        shade(cv, paw, DFUR[1:])
    # 감은 눈(흐뭇함) + 볼
    eye = [(334 + i * 2, 140 - (3 - abs(i - 3)) * 1.2) for i in range(7)]
    cv.paint(stroke(cv, eye, 3), (40, 26, 30))
    cv.paint(ellipse(cv, 342, 156, 7, 4), (236, 150, 150), 0.55)
    # 흙 부스러기가 떨어진다
    speckle(cv, rng, (330, 196, 400, 214), 10, [(92, 66, 48), (124, 92, 66)], size=(1.5, 2.5))
    # 기억의 바램: 세피아로 끌고 가장자리를 누른다
    g = cv.px.mean(axis=2, keepdims=True)
    sep = np.concatenate([g * 1.08 + 14, g * 0.94 + 6, g * 0.76], axis=2)
    cv.px = cv.px * 0.55 + sep * 0.45
    d = np.sqrt(((cv.xx - 330) / 380) ** 2 + ((cv.yy - 170) / 250) ** 2)
    k = np.clip((d - 0.55) / 0.45, 0, 1)
    step = (np.floor(k * 3) / 3)[..., None]
    cv.px = cv.px * (1 - 0.45 * step)
    return cv


SCENES = {
    'great_tree': great_tree,
    'mother_burrow': mother_burrow,
    'bori_freed': bori_freed,
    'momo_bloom': momo_bloom,
    'halmae_deep': halmae_deep,
    'momo_deep': momo_deep,
    'last_bloom': last_bloom,
    'sprout': sprout,
    'molehill': molehill,
    'doto_memory': doto_memory,
}


# 장면마다 그림 방식(v2, v4). 하나씩 바꾼다(0073): 파스텔 바탕으로 바꾼 장면은 v4.
SCENE_STYLE = {'molehill': (True, True)}


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else 'assets/art/illust'
    only = sys.argv[2:]
    os.makedirs(out, exist_ok=True)
    for name, fn in SCENES.items():
        if only and name not in only:
            continue
        set_style(*((V2_ENV, V4_ENV) if (V2_ENV or V4_ENV) else SCENE_STYLE.get(name, (False, False))))
        im = fn().image()
        path = os.path.join(out, name + '.png')
        im.save(path, optimize=True)
        print(f'{path}: {im.size[0]}×{im.size[1]}, {os.path.getsize(path) // 1024}KB')


if __name__ == '__main__':
    main()
