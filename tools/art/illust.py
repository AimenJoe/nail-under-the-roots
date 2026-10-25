"""컷씬 일러스트 생성기 — 임시 그림(docs/decisions/0039). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/illust.py [출력 폴더]  (기본 assets/art/illust)
사람이 그린 그림으로 바꿀 때는 같은 이름의 PNG를 넣는다(사양: docs/art/cutscene-illustrations.md).

크기: 화면에 보이는 창 640×360(게임 320×180의 2배 밀도). 패닝하는 그림은 더 크다(src/illust.rs의 pan과 맞출 것).
  great_tree   640×560  위(나무) → 아래(땅속)로 패닝
  halmae_deep  704×360  오른쪽으로 64px 패닝
  last_bloom   640×440  위(꽃) → 아래(밑동)로 패닝
  나머지       640×360
대화창·내레이션 띠가 보이는 창의 y 248 아래를 덮는다 → 얼굴·중요한 것은 그 위에.

그리는 법(픽셀 그림 흉내):
  - 하늘·흙은 계단식 그라디언트(띠), 1px 디더는 쓰지 않는다(홀수 화면 배율에서 1.5배로 늘어나 얼룩진다).
  - 몸은 마스크 하나에 구(球) 음영 띠를 입힌다(왼쪽 위 빛). 캐릭터마다 바깥에 어두운 외곽선 1~2px.
  - 실·뿌리는 베지어 곡선을 굵기를 줄여 가며 긋는다.
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


def shade(cv, mask, ramp, cx=None, cy=None, rx=None, ry=None, bias=0.0):
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
    for i, c in enumerate(ramp):
        cv.paint(mask & (idx == i), c)


def outline(cv, mask, rgb=OUTLINE, r=2):
    cv.paint(dilate(mask, r) & ~mask, rgb)


def bands(cv, y0, y1, top, bottom, n, x0=0, x1=None):
    """y0..y1 세로 계단 그라디언트."""
    x1 = cv.w if x1 is None else x1
    for i in range(n):
        t = i / max(1, n - 1)
        c = tuple(top[j] + (bottom[j] - top[j]) * t for j in range(3))
        a = y0 + (y1 - y0) * i / n
        b = y0 + (y1 - y0) * (i + 1) / n
        m = (cv.yy >= a) & (cv.yy < b) & (cv.xx >= x0) & (cv.xx < x1)
        cv.paint(m, c)


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

def mole(cv, cx, cy, s, facing=1, eyes='open', ramp=FUR, paws=True):
    """두더지: 둥근 몸 + 분홍 주둥이 + 큰 앞발·크림 발톱. s = 몸 가로 반지름."""
    f = facing
    body = ellipse(cv, cx, cy, s, s * 0.8)
    head = ellipse(cv, cx + f * s * 0.55, cy - s * 0.05, s * 0.6, s * 0.55)
    allm = body | head
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
    sil = allm | snout | nose | pawm | clawm
    outline(cv, sil, r=2 if s > 30 else 1)
    shade(cv, allm, ramp, cx + f * s * 0.2, cy, s * 1.2, s * 0.9)
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


def flower(cv, cx, cy, r, ramp=BLOSSOM, center=(250, 210, 120)):
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

def great_tree():
    rng = random.Random(11)
    cv = Canvas(640, 560)
    ground = 300
    bands(cv, 0, ground, (46, 38, 74), (206, 150, 126), 10)
    # 먼 언덕
    hill = poly(cv, [(0, 262)] + [(x, 258 - 14 * math.sin(x / 70) - 8 * math.sin(x / 23)) for x in range(0, 641, 16)] + [(640, ground), (0, ground)])
    cv.paint(hill, (92, 78, 96))
    # 땅속 흙 띠
    bands(cv, ground, 560, (92, 64, 50), (30, 20, 26), 9)
    speckle(cv, rng, (0, ground + 10, 640, 560), 140, [(70, 50, 42), (110, 82, 62), (54, 38, 36)])
    # 풀 줄
    grass = (cv.yy >= ground - 6) & (cv.yy < ground + 4)
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
        cv.paint(dilate(m) & ~m, VIOLET[0])
        cv.paint(m, VIOLET[2])
    for _ in range(14):
        p0 = (rng.uniform(80, 560), rng.uniform(380, 470))
        wavy_thread(cv, p0, (p0[0] + rng.uniform(-120, 120), 560), rng, 5, 2, VIOLET)
    # 어둠 속 붉은 눈
    for x, y in ((92, 520), (548, 506), (420, 540)):
        for dx in (0, 9):
            cv.paint(ellipse(cv, x + dx, y, 2.2, 1.6), (230, 60, 90))
    # 나무 줄기
    trunk = poly(cv, [(282, ground + 4), (298, 170), (306, 90), (334, 90), (342, 170), (362, ground + 4)])
    branches = []
    for (sx, sy, ex, ey, w) in ((310, 150, 170, 60, 14), (330, 140, 470, 50, 14), (320, 100, 250, 20, 10),
                                (325, 96, 400, 14, 10), (300, 190, 200, 140, 9), (340, 180, 450, 130, 9)):
        pts = bezier((sx, sy), ((sx + ex) / 2, sy - 10), (ex, (sy + ey) / 2), (ex, ey))
        branches.append(pts)
        trunk |= stroke(cv, pts, w, 3)
    outline(cv, trunk, (30, 20, 24), 2)
    shade(cv, trunk, BARK, 322, 200, 60, 140)
    # 줄기 결
    for k in range(5):
        x = 300 + k * 9
        cv.paint(stroke(cv, [(x, 290), (x + 3, 200), (x + 1, 130)], 1) & trunk, BARK[0])
    # 몇 안 남은 마른 잎
    for pts in branches:
        for (x, y) in pts[-12::4]:
            if rng.random() < 0.7:
                m = ellipse(cv, x + rng.uniform(-8, 8), y + rng.uniform(-6, 6), 5, 3, rot=rng.uniform(0, 3))
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
    sil = body | head | ears | tail | snout | paws
    outline(cv, sil, r=2)
    shade(cv, tail, PINK[:3])
    shade(cv, body | head, RFUR, cx, cy - 20, 100, 110)
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
    chest = ellipse(cv, cx, cy + 120, 120, 70)
    sil = ear_l | ear_r | head | cheeks | chest
    outline(cv, sil, r=2)
    shade(cv, chest, RABBIT, cx, cy + 110, 130, 80)
    shade(cv, ear_l, RABBIT)
    shade(cv, ear_r, RABBIT)
    cv.paint(ellipse(cv, cx - 44, cy - 124, 11, 66, rot=-0.18) | ellipse(cv, cx + 40, cy - 120, 11, 66, rot=0.22), PINK[1])
    shade(cv, head | cheeks, RABBIT, cx, cy, 100, 90)
    muzzle = ellipse(cv, cx, cy + 36, 40, 28)
    shade(cv, muzzle, RABBIT_LIGHT)
    cv.paint(poly(cv, [(cx - 8, cy + 22), (cx + 8, cy + 22), (cx, cy + 32)]), PINK[1])
    cv.paint(stroke(cv, [(cx, cy + 32), (cx, cy + 40), (cx - 8, cy + 46)], 2) | stroke(cv, [(cx, cy + 40), (cx + 8, cy + 46)], 2), (96, 60, 64))
    # 초록 눈(돌아온 눈)
    for ex in (cx - 38, cx + 38):
        cv.paint(ellipse(cv, ex, cy - 6, 14, 17), (22, 30, 26))
        cv.paint(ellipse(cv, ex, cy - 2, 10, 12), (86, 164, 92))
        cv.paint(ellipse(cv, ex, cy + 2, 7, 7), (38, 80, 46))
        cv.paint(ellipse(cv, ex - 5, cy - 10, 4, 4), (255, 255, 255))
        cv.paint(ellipse(cv, ex + 5, cy + 6, 2, 2), (230, 255, 230))
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


def halmae_deep():
    rng = random.Random(55)
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
    # 작은 Naïl(왼쪽 아래, 할매를 올려다본다)
    mole(cv, 210, 236, 22, facing=1, eyes='open')
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


def molehill():
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


SCENES = {
    'great_tree': great_tree,
    'mother_burrow': mother_burrow,
    'bori_freed': bori_freed,
    'momo_bloom': momo_bloom,
    'halmae_deep': halmae_deep,
    'last_bloom': last_bloom,
    'sprout': sprout,
    'molehill': molehill,
}


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else 'assets/art/illust'
    only = sys.argv[2:]
    os.makedirs(out, exist_ok=True)
    for name, fn in SCENES.items():
        if only and name not in only:
            continue
        im = fn().image()
        path = os.path.join(out, name + '.png')
        im.save(path, optimize=True)
        print(f'{path}: {im.size[0]}×{im.size[1]}, {os.path.getsize(path) // 1024}KB')


if __name__ == '__main__':
    main()
