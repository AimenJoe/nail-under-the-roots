"""Naïl 스프라이트 시트(32×32 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)

왜 이 길인가(0203 → 0206): 16칸 손 그림은 "두더지 같지 않다"(사장). 원본을 28px 높이로 줄인 것을 사장이 골랐다.
줄일 때의 함정 셋을 피한다:
  - 마젠타 바탕이 가장자리에 섞여 보라·청록 얼룩이 된다 → 마젠타스러움(min(R,B) − G)이 높은 점을 버리고 한 겹 깎는다.
  - 평균으로 줄이면 줄무늬(목도리 주황·빨강·짙은 줄)가 자줏빛으로 섞인다 → 원본에서 색을 먼저 고르고, 칸마다 **다수결**.
  - 색을 MEDIANCUT으로 고르면 넓은 갈색이 색을 다 가져가 좁은 주황이 없어진다 → MAXCOVERAGE. 모든 칸이 **같은 색표**(기준 칸에서
    골라 — 다섯 칸을 합쳐 고르면 주황이 또 밀렸다). 칸마다 따로 고르면 털빛이 칸마다 달라진다.
배율도 하나(기준 칸의 몸 키 → `BODY_H`)라 칸마다 몸 크기가 같다. 발바닥(가장 아래 칠한 줄)은 `FOOT`, 가로는 가운데.

칸 배치는 16칸 시절(nail.py)과 같다(게임 코드는 칸 크기 `art::NAIL_CELL`만 안다):
  줄 0: 서기 2, 걷기 4 / 줄 1: 치기 3, 맞음, 쓰러짐, 들기 3, 거두기 / 줄 2: 흙더미 2, 그림자(16칸 그림을 칸 아래 가운데에)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 번쩍·다 모인 몸빛)
원본에 없는 동작(줄 1)은 당분간 서기 칸을 옮기고 기울여 만든다(`derived`) — 동작 시트가 오면 `FRAMES`에 칸을 더해 바꾼다.
"""
import colorsys
import os
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import nail as legacy  # noqa: E402  흙더미·그림자 16칸 그림과 털빛 표(FURS)를 같이 쓴다

SRC = HERE / "sources" / "ai" / "sprites" / "nail_idle_walk_v1.png"
CELL = 32
FOOT = 30          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(16칸 시절 그대로).
BODY_H = 28        # 기준 칸의 몸 키(px)
NCOL = 20
OUTLINE = (43, 29, 42)

# 원본 칸(시트 좌표 x0, y0, x1, y1) → 쓸 곳. 원본의 위아래 줄이 맞닿아 y=208에서 가른다.
# 정면 칸(왼쪽 위·왼쪽 아래)은 3/4 오른쪽이 아니라 쓰지 않는다.
SRC_FRAMES = {
    "idle_a": (276, 0, 496, 208),
    "idle_b": (792, 8, 1008, 208),
    "walk_a": (276, 212, 496, 424),
    "walk_b": (536, 212, 752, 424),
    "walk_c": (792, 212, 1008, 424),
}
REF = "idle_a"


def magenta(a):
    return np.minimum(a[..., 0], a[..., 2]) - a[..., 1]


def solid_mask(a, erode=3):
    keep = (magenta(a) < 60).astype(np.uint8) * 255
    return np.asarray(Image.fromarray(keep).filter(ImageFilter.MinFilter(erode))) > 0


def body_h(a, keep):
    ys = np.nonzero(keep.any(1))[0]
    return ys.max() - ys.min() + 1


def shared_palette(arrs):
    """모든 칸 안쪽 점에서 MAXCOVERAGE로 NCOL색."""
    inner = np.concatenate([a[k] for a, k in arrs])
    side = int(np.sqrt(len(inner)))
    sq = inner[: side * side].reshape(side, side, 3).astype(np.uint8)
    return Image.fromarray(sq).quantize(NCOL, method=Image.Quantize.MAXCOVERAGE, dither=Image.Dither.NONE)


def shrink(a, keep, scale, pal):
    """원본 칸 → 다수결로 줄인 RGBA(외곽선 없음)."""
    ys, xs = np.nonzero(keep)
    a, keep = a[ys.min():ys.max() + 1, xs.min():xs.max() + 1], keep[ys.min():ys.max() + 1, xs.min():xs.max() + 1]
    idx = np.asarray(Image.fromarray(a.astype(np.uint8)).quantize(palette=pal, dither=Image.Dither.NONE))
    colors = np.array(pal.getpalette()[: NCOL * 3]).reshape(-1, 3)
    H, W = a.shape[:2]
    h, w = max(1, round(H * scale)), max(1, round(W * scale))
    out = np.zeros((h, w, 4), np.uint8)
    for oy in range(h):
        for ox in range(w):
            ya, yb = int(oy * H / h), max(int(oy * H / h) + 1, int((oy + 1) * H / h))
            xa, xb = int(ox * W / w), max(int(ox * W / w) + 1, int((ox + 1) * W / w))
            k = keep[ya:yb, xa:xb]
            if k.mean() < 0.5:
                continue
            c = np.bincount(idx[ya:yb, xa:xb][k], minlength=NCOL).argmax()
            out[oy, ox] = list(colors[c]) + [255]
    return out


def outline(rgba):
    A = rgba[..., 3] > 0
    p = np.pad(A, 1)
    o = (p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]) & ~A
    out = rgba.copy()
    out[o] = list(OUTLINE) + [255]
    return out


def seat(sprite, dx=0, dy=0):
    """CELL×CELL 칸에 발바닥 FOOT·가운데로(외곽선 한 겹 둘레 포함). dx·dy로 옮긴다."""
    h, w = sprite.shape[:2]
    cell = np.zeros((CELL, CELL, 4), np.uint8)
    top, left = FOOT - (h - 1) + dy, (CELL - w) // 2 + dx
    for y in range(h):
        for x in range(w):
            yy, xx = top + y, left + x
            if 0 <= yy < CELL and 0 <= xx < CELL and sprite[y, x, 3]:
                cell[yy, xx] = sprite[y, x]
    return outline(cell)


def shear(sprite, k):
    """윗몸을 앞(+)·뒤(−)로 기울인다: 위에서부터 줄마다 조금씩 민다(발 줄은 그대로)."""
    h = sprite.shape[0]
    out = np.zeros_like(sprite)
    for y in range(h):
        s = round(k * (h - 1 - y) / (h - 1))
        out[y] = np.roll(sprite[y], s, axis=0)
        if s > 0:
            out[y, :s] = 0
        elif s < 0:
            out[y, s:] = 0
    return out


def squash(sprite, rows):
    """몸을 위에서 `rows`줄 눌러 납작하게(발은 그대로): 가운데 줄을 뺀다."""
    h = sprite.shape[0]
    drop = set(range(h // 3, h // 3 + rows))
    return np.stack([sprite[y] for y in range(h) if y not in drop])


def derived(base):
    """원본에 없는 동작(줄 1): 서기 칸을 기울이고 옮겨 당분간 쓴다. 동작 시트가 오면 바꾼다."""
    fwd = shear(base, 3)
    return {
        "attack0": seat(fwd, dx=2),
        "attack1": seat(squash(fwd, 1), dx=2, dy=0),
        "attack2": seat(squash(shear(base, 4), 2), dx=3),
        "hurt": seat(shear(base, -3), dx=-2),
        "dead": seat(base[::-1], dy=-1),
        "windup0": seat(shear(base, -2), dy=-1),
        "windup1": seat(shear(base, -2), dy=-1),
        "windup2": seat(shear(base, -3), dy=-2),
        "settle": seat(squash(base, 1), dx=1),
    }


def small(grid_text):
    """16칸 그림(흙더미·그림자)을 32칸 아래 가운데에."""
    rows = legacy.outline(legacy.grid(grid_text))
    cell = np.zeros((CELL, CELL, 4), np.uint8)
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == ".":
                continue
            c = legacy.P[ch]
            c = c if len(c) == 4 else c + (255,)
            cell[CELL - 16 + y, 8 + x] = c
    return cell


def build():
    src = np.asarray(Image.open(SRC).convert("RGB")).astype(int)
    crops = {k: src[y0:y1, x0:x1] for k, (x0, y0, x1, y1) in SRC_FRAMES.items()}
    masks = {k: solid_mask(a) for k, a in crops.items()}
    # 색표는 기준 칸 하나에서(다섯 칸을 합쳐 고르면 넓은 갈색에 밀려 목도리 주황이 빠졌다), 다른 칸은 이 색표에 맞춘다.
    pal = shared_palette([(crops[REF], masks[REF])])
    scale = BODY_H / body_h(crops[REF], masks[REF])
    spr = {k: shrink(crops[k], masks[k], scale, pal) for k in crops}
    d = derived(spr[REF])
    rows = [
        [seat(spr["idle_a"]), seat(spr["idle_b"]), seat(spr["walk_a"]), seat(spr["walk_b"], dy=-1),
         seat(spr["walk_c"]), seat(spr["idle_b"], dy=-1)],
        [d["attack0"], d["attack1"], d["attack2"], d["hurt"], d["dead"],
         d["windup0"], d["windup1"], d["windup2"], d["settle"]],
        [small(legacy.MOUND_A), small(legacy.MOUND_B), small(legacy.SHADOW)],
    ]
    sheet = np.zeros((CELL * 6, CELL * 9, 4), np.uint8)
    for r, row in enumerate(rows):
        for c, cell in enumerate(row):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = cell
            sil = cell.copy()
            on = sil[..., 3] > 0
            if r == 2:
                on &= ~np.all(sil[..., :3] == (24, 16, 28), axis=-1)   # 그림자는 실루엣 없음
            sil[on] = [255, 255, 255, 255]
            sil[~on] = 0
            sheet[(r + 3) * CELL:(r + 4) * CELL, c * CELL:(c + 1) * CELL] = sil
    return sheet


def fur_variant(sheet, fur):
    """털빛(0048): 털 갈색만 그 털빛의 4단(d·b·h·l)에 밝기 순으로 옮긴다. 목도리·분홍·눈·흙은 그대로."""
    if fur == 0:
        return sheet
    target = [legacy.FURS[fur][k] for k in "dbhl"]
    base = [legacy.P[k] for k in "dbhl"]
    lum = lambda c: 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
    out = sheet.copy()
    flat = out[: CELL * 3].reshape(-1, 4)
    cache = {}
    for i, px in enumerate(flat):
        if px[3] == 0:
            continue
        key = tuple(int(v) for v in px[:3])
        if key not in cache:
            r, g, b = (v / 255 for v in key)
            hh, ll, ss = colorsys.rgb_to_hls(r, g, b)
            is_fur = 0.03 <= hh <= 0.12 and 0.12 <= ss <= 0.55 and 0.15 <= ll <= 0.75 and key != OUTLINE
            if not is_fur:
                cache[key] = None
            else:
                # 기본 털 4단 밝기 사이 어디쯤인가 → 같은 자리의 털빛 색
                L = lum(key)
                bl = [lum(c) for c in base]
                t = np.interp(L, bl, range(4))
                i0, f = int(min(t, 2.999)), t - int(min(t, 2.999))
                c0, c1 = target[i0], target[i0 + 1]
                cache[key] = tuple(int(c0[j] * (1 - f) + c1[j] * f) for j in range(3))
        if cache[key] is not None:
            flat[i, :3] = cache[key]
    return out


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/nail.png"
    sheet = build()
    Image.fromarray(sheet, "RGBA").save(out)
    for n in legacy.FURS:
        Image.fromarray(fur_variant(sheet, n), "RGBA").save(os.path.join(os.path.dirname(out) or ".", f"nail_fur{n}.png"))
