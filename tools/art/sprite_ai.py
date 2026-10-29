"""나노바나나 스프라이트 시트 → 게임 도트 변환기(공용, 0206). 생물마다 `Spec` 하나를 적고 `convert`를 부른다.

쓰는 법(예: `nail_ai.py`):
    spec = Spec(sources=[Source(경로, {"idle_a": (x0, y0, x1, y1), ...}, groups={"idle_a": "idle_a", ...})],
                palette_ref=(0, "idle_a"), body_h=16, limbs=Limbs(...), swing={...}, breathe=("idle_a", "idle_b"))
    spr = convert(spec)                # 이름 → 줄인 RGBA(외곽선 없음)
    cell = seat(spr["idle_a"], 24, 22)  # 칸에 앉히고 외곽선

Naïl을 옮기며 부딪힌 함정과 답(0206):
  - 마젠타 바탕이 가장자리에 섞여 보라·청록 얼룩이 된다 → 마젠타스러움(min(R,B) − G)이 높은 점을 버리고 한 겹 깎는다.
  - 평균으로 줄이면 줄무늬(목도리 주황·빨강·짙은 줄)가 자줏빛으로 섞인다 → 원본에서 색을 먼저 고르고, 칸마다 **다수결**.
  - MEDIANCUT은 넓은 갈색이 색을 다 가져가 좁은 주황이 없어지고, MAXCOVERAGE는 원본에 없는 색(자줏빛)을 만들었다
    → 원본이 **실제로 쓴 색**을 많이 쓴 순서로, 가까운 색은 합쳐 고른다. 모든 칸이 **같은 색표**(기준 칸 하나에서).
  - 배율은 원본 줄(무리)마다 그 무리 첫 칸의 몸 키로(칸마다 맞추면 걷기의 들썩임이 사라진다).
  - "효과는 그리지 말라"고 해도 휘두름 호·흙먼지를 그려 온다 → 정한 네모 안의 그 색만 지우고(`Source.erase`),
    몸에서 떨어진 작은 조각은 버린다(`main_parts`).
  - 칸 안 가로 자리는 몸통 털의 가운데로(`fur_cx`) — 겉 네모 가운데로 두면 발톱을 뻗는 칸마다 몸이 옆으로 뛴다.
  - 손발을 검게 그려 온다 → `Limbs`로 초상화 색에(발톱에 닿은 덩어리 = 손, 아래 세 줄 = 발, 머리 둘레 점 = 털 그늘).
  - 걷기에서 손이 그대로다 → `swing`(빈 자리는 몸과 손 사이만 털로 메워 팔이 붙어 있게).
  - 서기 두 칸을 따로 그려 와 숨쉬기가 끊긴다 → `breathe`로 한 칸에서 둘째 칸을 만든다.
"""
import colorsys
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

NCOL = 20
OUTLINE = (43, 29, 42)


@dataclass
class Source:
    """원본 시트 한 장. frames: 이름 → 시트 좌표 (x0, y0, x1, y1). groups: 이름 → 배율 기준 칸 이름(같은 무리는 같은 배율).
    erase: 이름 → 효과(호)가 지나가는 네모 목록(시트 좌표) — 그 안의 `Spec.arc` 색만 지운다.
    tidy: 가장 큰 덩어리의 이 비율보다 작은 떨어진 조각(흙먼지·옆 칸에서 넘어온 귀·얼굴)을 버린다. 0이면 안 버린다.
    strip: 원본이 이미 굵은 외곽선이 있는 도트(나노바나나가 게임 크기 그림을 다시 그린 것)면 바탕에서 이 px 안의
    어두운 점을 걷어낸다 — 그대로 줄이면 원본 외곽선 위에 `outline`이 한 겹 더 붙어 선이 두 겹이 됐다.
    pink: 그림에 마젠타에 가까운 진분홍(꽃 등)이 있으면 True — 바탕을 "R≈B인 마젠타"로만 본다(`magenta`만 보면 진분홍도
    바탕으로 지워진다, 토끼풀의 꽃 0218)."""
    path: Path
    frames: dict
    groups: dict
    erase: dict = field(default_factory=dict)
    tidy: float = 0.0
    strip: int = 0
    pink: bool = False


@dataclass
class Limbs:
    """검게 온 손발을 바꿀 색. limb = 원본 검은 손발이 색표에서 받은 색, claw = 발톱 크림."""
    limb: tuple
    paw: tuple
    sole: tuple
    arm: tuple
    shade: tuple
    claw: tuple


@dataclass
class Spec:
    sources: list
    palette_ref: tuple            # (원본 번호, 칸 이름): 색표를 고를 칸
    body_h: int                   # 배율 기준 칸의 몸 키(px)
    arc: tuple = (192, 145, 108)  # 지울 효과 색
    limbs: Limbs = None
    swing: dict = field(default_factory=dict)   # 이름 → (dx, dy) 앞발 흔들기
    breathe: tuple = None         # (들이쉰 칸, 만들 내쉰 칸)


def magenta(a):
    return np.minimum(a[..., 0], a[..., 2]) - a[..., 1]


def solid_mask(a, erode=3, pink=False):
    bg = magenta(a) >= 60
    if pink:                                # 진분홍(R이 B보다 한참 크다)은 바탕이 아니다
        bg &= np.abs(a[..., 0] - a[..., 2]) < 40
    keep = (~bg).astype(np.uint8) * 255
    return np.asarray(Image.fromarray(keep).filter(ImageFilter.MinFilter(erode))) > 0


def erase_color(a, keep, boxes, frame, color, tol=60):
    """정한 네모 안에서 color ± tol(RGB 차 합)만 버린다."""
    x0, y0 = frame[0], frame[1]
    for bx0, by0, bx1, by1 in boxes:
        sub = a[by0 - y0:by1 - y0, bx0 - x0:bx1 - x0]
        near = np.abs(sub - color).sum(-1) < tol
        keep[by0 - y0:by1 - y0, bx0 - x0:bx1 - x0] &= ~near
    return keep


def strip_outline(a, keep, px):
    """바탕에서 px 안쪽의 어두운 점(RGB 합 < 200)을 버린다: 원본의 외곽선."""
    near = ~keep
    for _ in range(px):
        g = near.copy()
        g[1:] |= near[:-1]
        g[:-1] |= near[1:]
        g[:, 1:] |= near[:, :-1]
        g[:, :-1] |= near[:, 1:]
        near = g
    return keep & ~(near & (a.sum(-1) < 200))


def grow(mask, seeds):
    """mask 안에서 seeds에 이어진 점만(4방향)."""
    out = seeds.copy()
    stack = list(zip(*np.nonzero(seeds)))
    h, w = mask.shape
    while stack:
        y, x = stack.pop()
        for yy, xx in ((y + 1, x), (y - 1, x), (y, x + 1), (y, x - 1)):
            if 0 <= yy < h and 0 <= xx < w and mask[yy, xx] and not out[yy, xx]:
                out[yy, xx] = True
                stack.append((yy, xx))
    return out


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
    full = np.array(pal.getpalette()[:768]).reshape(-1, 3)
    colors = full[:NCOL]
    # 색표 뒤쪽은 0(검정)으로 채워져 있어, 색표에 짙은 색이 없으면 짙은 점이 그쪽으로 간다(모모 깡충 원본, 0229).
    # 표는 그대로 두고(그 점들은 표 안의 색에 표를 보태지 않는다), 다수결에서 이긴 칸만 표 안의 가장 가까운 색으로 접는다.
    # 표를 미리 접으면 짙은 점이 외곽선 색에 표를 보태 도토 칸 8점이 바뀌었다(0230 R17).
    fold = np.array([((colors - c) ** 2).sum(-1).argmin() for c in full])
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
            c = fold[np.bincount(idx[ya:yb, xa:xb][k], minlength=NCOL).argmax()]
            out[oy, ox] = list(colors[c]) + [255]
    return out


def lum(c):
    """밝기(BT.601) — 물든 모습 색 바꾸기가 쓴다."""
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def ramp(colors, t):
    """t(0..1) 자리의 색(단 사이를 섞지 않고 가까운 단)."""
    return colors[min(len(colors) - 1, max(0, int(t * len(colors))))]


def silhouette(cell):
    """칸의 흰 실루엣(맞을 때 번쩍임·다 모였을 때 빛, 게임이 + 2줄로 읽는다)."""
    sil = np.zeros_like(cell)
    sil[cell[..., 3] > 0] = [255, 255, 255, 255]
    return sil


def sheet4(cols):
    """칸 [(돌아온, 물든)] → 네 줄 시트: 돌아온·물든 + 각각의 흰 실루엣(게임 `art::BigSheet`, 0206). 물든 모습이 없는
    시트(이웃·잡몹)는 같은 칸을 둘 넣는다. 0230 R17: 변환기마다 같은 붙이기를 손으로 하던 것."""
    size = cols[0][0].shape[0]
    sheet = np.zeros((size * 4, size * len(cols), 4), np.uint8)
    for c, (back, tainted) in enumerate(cols):
        for r, im in enumerate((back, tainted, silhouette(back), silhouette(tainted))):
            sheet[r * size:(r + 1) * size, c * size:(c + 1) * size] = im
    return sheet


def outline(rgba):
    A = rgba[..., 3] > 0
    p = np.pad(A, 1)
    o = (p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]) & ~A
    out = rgba.copy()
    out[o] = list(OUTLINE) + [255]
    return out


def _dilate(mask, steps=1):
    """체비쇼프 거리 steps까지 넓힌다."""
    for _ in range(steps):
        g = mask.copy()
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                g |= np.roll(np.roll(mask, dy, 0), dx, 1)
        mask = g
    return mask


def paws(sprite, c):
    """검은 손발 → c(Limbs)의 색. 아래 세 줄은 발(sole), 그 위는 발톱에서 2칸 안이 손(paw), 더 먼 팔은 털색(arm).
    손발 = 검은 덩어리 중 크림 발톱에 닿거나 아래 세 줄에 걸친 것. 머리 둘레의 검은 점은 털 그늘(shade) — 검게 두면
    외곽선에 붙어 선이 두 겹으로 굵어 보였다. 팔까지 분홍이면 코 바로 아래 앞발이 코와 한 덩어리로 굵어 보였다."""
    out = sprite.copy()
    h = out.shape[0]
    rgb = out[..., :3].astype(int)
    on = (out[..., 3] > 0) & (np.abs(rgb - c.limb).sum(-1) < 20)
    claw = (out[..., 3] > 0) & (np.abs(rgb - c.claw).sum(-1) < 8)
    seeds = _dilate(claw)
    seeds[h - 3:] = True
    limb = grow(on, seeds & on)
    out[on & ~limb, :3] = c.shade
    on = limb
    near2 = _dilate(claw, 2)
    rows = np.arange(h)[:, None] >= h - 3
    out[on & rows, :3] = c.sole
    out[on & ~rows & near2, :3] = c.paw
    out[on & ~rows & ~near2, :3] = c.arm
    return out


def swing(sprite, move, c):
    """앞발(가운데 오른쪽의 손·발톱, 발 줄 위)은 move = (dx, dy)만큼, 뒷발톱(왼쪽)은 (−dx, 0)만큼 옮긴다.
    빈 자리는 비운다. 다만 같은 줄에서 몸과 옮긴 손 사이에 낀 빈 자리는 몸 쪽으로 처음 만나는 털 색으로 메운다 —
    팔이 늘어난 것처럼 붙어 있게(떨어지면 외곽선이 따로 둘러 공중에 뜬다). 손이 떠난 아래 줄은 털로 메우지 않는다.
    들어 올린 앞발의 윗줄이 얼굴에 닿으면 그 점을 외곽선 색으로 — 코·주둥이와 색이 비슷해 한 덩어리로 보였다."""
    dx, dy = move
    if (dx, dy) == (0, 0):
        return sprite
    h, w = sprite.shape[:2]
    rgb = sprite[..., :3].astype(int)
    hand = (sprite[..., 3] > 0) & ((np.abs(rgb - c.paw).sum(-1) < 8) | (np.abs(rgb - c.claw).sum(-1) < 8))
    hand[h - 3:] = False
    out = sprite.copy()
    for side, (mx, my) in ((np.arange(w) >= w // 2, (dx, dy)), (np.arange(w) < w // 2, (-dx, 0))):
        m = hand & side[None, :]
        ys, xs = np.nonzero(m)
        px = sprite[ys, xs].copy()
        out[ys, xs] = 0
        moved = np.zeros_like(hand)
        for (y, x), col in zip(zip(ys, xs), px):
            if 0 <= x + mx < w and 0 <= y + my < h:
                out[y + my, x + mx] = col
                moved[y + my, x + mx] = True
        # 몸 쪽(가운데) 방향: 오른쪽 앞발은 왼쪽(-1), 왼쪽 뒷발톱은 오른쪽(+1)
        inward = -1 if side[-1] else 1
        for y, x in zip(ys, xs):
            if out[y, x, 3]:
                continue
            outer = moved[y, x + 1:] if inward < 0 else moved[y, :x]
            n = x + inward
            while 0 <= n < w and not out[y, n, 3]:
                n += inward
            if outer.any() and 0 <= n < w:
                out[y, x] = out[y, n]
        if my < 0:
            for y, x in zip(*np.nonzero(moved)):
                if y > 0 and out[y - 1, x, 3] and not moved[y - 1, x] and tuple(out[y, x, :3]) == c.paw:
                    out[y, x] = list(OUTLINE) + [255]
    return out


def breathe(sprite):
    """숨 들이쉰 서기 칸 → 내쉰 칸: 아랫몸에서 바로 아래 줄과 가장 닮은 줄 하나를 빼 1px 낮춘다(눈·코·발은 그대로)."""
    h = sprite.shape[0]
    rows = range(h // 2, h - 4)
    drop = min(rows, key=lambda r: int((sprite[r] != sprite[r + 1]).any(-1).sum()))
    return np.concatenate([sprite[:drop], sprite[drop + 1:]])


def is_fur(key):
    r, g, b = (v / 255 for v in key)
    hh, ll, ss = colorsys.rgb_to_hls(r, g, b)
    return 0.03 <= hh <= 0.12 and 0.12 <= ss <= 0.55 and 0.15 <= ll <= 0.75 and tuple(key) != OUTLINE


def fur_cx(sprite):
    """몸통 털(갈색) 점들의 가로 가운데."""
    ys, xs = np.nonzero(sprite[..., 3])
    fx = [x for y, x in zip(ys, xs) if is_fur(tuple(int(v) for v in sprite[y, x, :3]))]
    return float(np.median(fx))


def seat(sprite, cell, foot, dx=0, dy=0):
    """cell×cell 칸에 발바닥 foot·가운데로(외곽선 한 겹 둘레 포함). dx·dy로 옮긴다."""
    h, w = sprite.shape[:2]
    out = np.zeros((cell, cell, 4), np.uint8)
    top, left = foot - (h - 1) + dy, (cell - w) // 2 + dx
    for y in range(h):
        for x in range(w):
            yy, xx = top + y, left + x
            if 0 <= yy < cell and 0 <= xx < cell and sprite[y, x, 3]:
                out[yy, xx] = sprite[y, x]
    return outline(out)


def seat_on_fur(sprite, ref, cell, foot):
    """몸통 털의 가운데를 ref(겉 네모로 가운데에 앉힌 기준 칸)의 몸통 자리에 맞춰 앉힌다."""
    anchor = (cell - ref.shape[1]) // 2 + fur_cx(ref)
    return seat(sprite, cell, foot, dx=round(anchor - ((cell - sprite.shape[1]) // 2 + fur_cx(sprite))))


def convert(spec):
    """Spec → 이름 → 줄인 RGBA(외곽선 없음, 칸에 앉히기 전)."""
    loaded = []
    for s in spec.sources:
        img = np.asarray(Image.open(s.path).convert("RGB")).astype(int)
        crops = {k: img[y0:y1, x0:x1] for k, (x0, y0, x1, y1) in s.frames.items()}
        masks = {}
        for k, a in crops.items():
            keep = solid_mask(a, pink=s.pink)
            if k in s.erase:
                keep = erase_color(a, keep, s.erase[k], s.frames[k], spec.arc)
            if s.strip:
                keep = strip_outline(a, keep, s.strip)
            masks[k] = main_parts(keep, s.tidy) if s.tidy else keep
        loaded.append((s, crops, masks))
    si, name = spec.palette_ref
    _, crops, masks = loaded[si]
    pal = shared_palette([(crops[name], masks[name])])
    out = {}
    for s, crops, masks in loaded:
        scale = {g: spec.body_h / body_h(crops[g], masks[g]) for g in set(s.groups.values())}
        for k in crops:
            sp = shrink(crops[k], masks[k], scale[s.groups[k]], pal)
            if spec.limbs:
                sp = paws(sp, spec.limbs)
                sp = swing(sp, spec.swing.get(k, (0, 0)), spec.limbs)
            out[k] = sp
    if spec.breathe:
        a, b = spec.breathe
        out[b] = breathe(out[a])
    return out
