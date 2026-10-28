"""Naïl 스프라이트 시트(32×32 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)

왜 이 길인가(0203 → 0206): 16칸 손 그림은 "두더지 같지 않다"(사장). 원본을 28px 높이로 줄인 것을 사장이 골랐다.
줄일 때의 함정 셋을 피한다:
  - 마젠타 바탕이 가장자리에 섞여 보라·청록 얼룩이 된다 → 마젠타스러움(min(R,B) − G)이 높은 점을 버리고 한 겹 깎는다.
  - 평균으로 줄이면 줄무늬(목도리 주황·빨강·짙은 줄)가 자줏빛으로 섞인다 → 원본에서 색을 먼저 고르고, 칸마다 **다수결**.
  - 색을 MEDIANCUT으로 고르면 넓은 갈색이 색을 다 가져가 좁은 주황이 없어지고, MAXCOVERAGE는 원본에 없는 색(자줏빛)을
    만들었다 → 원본이 **실제로 쓴 색**을 많이 쓴 순서로, 가까운 색은 합쳐 고른다. 모든 칸이 **같은 색표**(기준 칸에서).
    칸마다 따로 고르면 털빛이 칸마다 달라진다.
배율도 하나(기준 칸의 몸 키 → `BODY_H`)라 칸마다 몸 크기가 같다. 발바닥(가장 아래 칠한 줄)은 `FOOT`, 가로는 가운데.

칸 배치는 16칸 시절(nail.py)과 같다(게임 코드는 칸 크기 `art::NAIL_CELL`만 안다):
  줄 0: 서기 2, 걷기 4 / 줄 1: 치기 3, 맞음, 쓰러짐, 들기 3, 거두기 / 줄 2: 흙더미 2, 그림자(16칸 그림을 칸 아래 가운데에)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 번쩍·다 모인 몸빛)
줄 1은 시트 B(`SRC_B`, 3×3: 들기 A·치기 A·들기 B / 치기 B·크게 들기·내려찍기 / 거두기·맞음·쓰러짐)에서.
시트 B의 함정: 그리지 말라 한 휘두름 호(살구 갈색)·흙먼지를 그려 왔다 → 정한 네모 안의 그 색만 지우고(`ERASE`),
몸에서 떨어진 작은 조각은 버린다(`main_parts`). 칸 안 가로 자리는 몸통 털의 가운데로 맞춘다(`fur_cx`) — 겉 네모로
가운데에 두면 발톱을 뻗는 칸마다 몸이 옆으로 뛴다.
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

SRC = HERE / "sources" / "ai" / "sprites" / "nail_s1_v2.png"   # 시안 1 몸 + 둥근 시트의 눈·볼(0206, ✦는 지움)
CELL = 24          # 몸 16px + 뻗은 발톱이 들어가는 칸(0206: 32칸·몸 28은 월드에 비해 컸다)
FOOT = 22          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(16칸 시절 그대로).
BODY_H = 16        # 기준 칸의 몸 키(px) — 타일·들쥐와 같은 크기(사장: 28은 4배 넓이라 컸다)
NCOL = 20
OUTLINE = (43, 29, 42)

# 원본 칸(시트 좌표 x0, y0, x1, y1). 원본은 줄마다 그린 크기가 다르다(서기 칸이 걷기 칸보다 크다) →
# 배율은 줄(무리)마다 그 무리의 첫 칸 몸 키로 맞춘다(칸마다 맞추면 걷기의 들썩임이 사라진다).
SRC_FRAMES = {               # 원본의 발밑 보라 줄(윗줄 y 386~391, 아랫줄 661~666)은 칸 밖으로 뺀다
    "idle_a": (39, 7, 463, 385),
    "idle_b": (554, 17, 982, 385),
    "walk_a": (6, 419, 247, 660),
    "walk_b": (265, 419, 499, 660),
    "walk_c": (523, 419, 754, 660),
    "walk_d": (777, 419, 1013, 660),
}
GROUP = {"idle_a": "idle_a", "idle_b": "idle_a", "walk_a": "walk_a", "walk_b": "walk_a", "walk_c": "walk_a", "walk_d": "walk_a"}
REF = "idle_a"

# 시트 B(동작): 눈 크기가 아홉 칸 모두 같아(같은 배율로 그렸다) 배율 하나 — 똑바로 선 들기 A 칸의 몸 키를 BODY_H로.
SRC_B = HERE / "sources" / "ai" / "sprites" / "nail_s1_v2_b.png"
B_FRAMES = {
    "windup0": (9, 5, 331, 295),
    "attack0": (350, 5, 669, 295),
    "windup1": (692, 5, 1022, 295),
    "attack1": (9, 317, 335, 604),
    "windup2": (378, 314, 658, 607),
    "attack2": (722, 320, 1022, 608),
    "settle": (9, 626, 335, 913),
    "hurt": (353, 621, 668, 913),
    "dead": (698, 678, 1022, 913),
}
B_REF = "windup0"
ARC = (192, 145, 108)   # 휘두름 호의 살구 갈색(발톱 크림·주둥이·코와는 RGB 차 합 100 넘게 떨어진다)
ERASE = {"attack0": [(455, 170, 669, 295)], "attack1": [(255, 317, 335, 604)]}   # 호가 지나가는 네모(시트 좌표)


def magenta(a):
    return np.minimum(a[..., 0], a[..., 2]) - a[..., 1]


def solid_mask(a, erode=3):
    keep = (magenta(a) < 60).astype(np.uint8) * 255
    return np.asarray(Image.fromarray(keep).filter(ImageFilter.MinFilter(erode))) > 0


def erase_arc(a, keep, box, frame):
    """정한 네모 안에서 호 색(ARC ± 60)만 버린다."""
    x0, y0 = frame[0], frame[1]
    for bx0, by0, bx1, by1 in box:
        sub = a[by0 - y0:by1 - y0, bx0 - x0:bx1 - x0]
        near = np.abs(sub - ARC).sum(-1) < 60
        keep[by0 - y0:by1 - y0, bx0 - x0:bx1 - x0] &= ~near
    return keep


def main_parts(keep, frac=0.02):
    """가장 큰 덩어리의 frac보다 작은 조각(흙먼지·호 찌꺼기)을 버린다."""
    lab = np.zeros(keep.shape, np.int32)
    sizes = [0]
    H, W = keep.shape
    for y0, x0 in zip(*np.nonzero(keep)):
        if lab[y0, x0]:
            continue
        n = len(sizes)
        lab[y0, x0] = n
        stack, size = [(y0, x0)], 0
        while stack:
            y, x = stack.pop()
            size += 1
            for yy, xx in ((y + 1, x), (y - 1, x), (y, x + 1), (y, x - 1)):
                if 0 <= yy < H and 0 <= xx < W and keep[yy, xx] and not lab[yy, xx]:
                    lab[yy, xx] = n
                    stack.append((yy, xx))
        sizes.append(size)
    sizes = np.array(sizes)
    return (sizes >= sizes.max() * frac)[lab] & keep


def body_h(a, keep):
    ys = np.nonzero(keep.any(1))[0]
    return ys.max() - ys.min() + 1


def shared_palette(arrs, near=36):
    """원본이 실제로 쓴 색에서 NCOL색: 많이 쓴 색부터, 이미 고른 색과 가까우면(RGB 차 합 < near) 건너뛴다.
    PIL의 MAXCOVERAGE·MEDIANCUT은 원본에 없는 색을 만들었다(가장 많은 갈색 #654838 → 자줏빛 #722d3f).
    격자가 살아 있는 원본은 색 수가 적어 이 방식이 원래 색을 그대로 지킨다."""
    inner = np.concatenate([a[k] for a, k in arrs])
    cols, counts = np.unique(inner.reshape(-1, 3), axis=0, return_counts=True)
    chosen = []
    for i in np.argsort(-counts):
        c = cols[i]
        if all(np.abs(c - q).sum() >= near for q in chosen):
            chosen.append(c)
        if len(chosen) == NCOL:
            break
    while len(chosen) < NCOL:
        chosen.append(chosen[-1])
    pal = Image.new("P", (1, 1))
    pal.putpalette([int(v) for c in chosen for v in c] + [0] * (768 - 3 * NCOL))
    return pal


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


def is_fur(key):
    r, g, b = (v / 255 for v in key)
    hh, ll, ss = colorsys.rgb_to_hls(r, g, b)
    return 0.03 <= hh <= 0.12 and 0.12 <= ss <= 0.55 and 0.15 <= ll <= 0.75 and tuple(key) != OUTLINE


def fur_cx(sprite):
    """몸통 털(갈색) 점들의 가로 가운데."""
    ys, xs = np.nonzero(sprite[..., 3])
    fx = [x for y, x in zip(ys, xs) if is_fur(tuple(int(v) for v in sprite[y, x, :3]))]
    return float(np.median(fx))


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


def small(grid_text):
    """16칸 그림(흙더미·그림자)을 칸 아래 가운데에."""
    rows = legacy.outline(legacy.grid(grid_text))
    cell = np.zeros((CELL, CELL, 4), np.uint8)
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == ".":
                continue
            c = legacy.P[ch]
            c = c if len(c) == 4 else c + (255,)
            cell[CELL - 16 + y, (CELL - 16) // 2 + x] = c
    return cell


def build():
    src = np.asarray(Image.open(SRC).convert("RGB")).astype(int)
    crops = {k: src[y0:y1, x0:x1] for k, (x0, y0, x1, y1) in SRC_FRAMES.items()}
    masks = {k: solid_mask(a) for k, a in crops.items()}
    # 색표는 기준 칸 하나에서(다섯 칸을 합쳐 고르면 넓은 갈색에 밀려 목도리 주황이 빠졌다), 다른 칸은 이 색표에 맞춘다.
    pal = shared_palette([(crops[REF], masks[REF])])
    scale = {g: BODY_H / body_h(crops[g], masks[g]) for g in set(GROUP.values())}
    spr = {k: shrink(crops[k], masks[k], scale[GROUP[k]], pal) for k in crops}
    src_b = np.asarray(Image.open(SRC_B).convert("RGB")).astype(int)
    b_crops = {k: src_b[y0:y1, x0:x1] for k, (x0, y0, x1, y1) in B_FRAMES.items()}
    b_masks = {}
    for k, a in b_crops.items():
        keep = solid_mask(a)
        if k in ERASE:
            keep = erase_arc(a, keep, ERASE[k], B_FRAMES[k])
        b_masks[k] = main_parts(keep)
    b_scale = BODY_H / body_h(b_crops[B_REF], b_masks[B_REF])
    act = {k: shrink(b_crops[k], b_masks[k], b_scale, pal) for k in b_crops}
    # 칸 안 가로 자리: 서기 칸을 가운데에 둔 그 몸통 자리에 모두 맞춘다.
    w0 = spr[REF].shape[1]
    anchor = (CELL - w0) // 2 + fur_cx(spr[REF])
    put = lambda sp: seat(sp, dx=round(anchor - ((CELL - sp.shape[1]) // 2 + fur_cx(sp))))
    rows = [
        [seat(spr["idle_a"]), seat(spr["idle_b"]), seat(spr["walk_a"]), seat(spr["walk_b"]),
         seat(spr["walk_c"]), seat(spr["walk_d"])],
        [put(act[k]) for k in ("attack0", "attack1", "attack2", "hurt", "dead", "windup0", "windup1", "windup2", "settle")],
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
            if not is_fur(key):
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
