"""큰 나무(마을, 0045 → 0238): 나노바나나 원본 한 장(시든 나무 | 다 핀 나무)을 잎 단계 다섯 칸으로.

실행: python3 tools/art/tree_ai.py [출력 경로]   (기본 assets/art/tree.png)

원본 `sources/ai/sprites/tree_v1.png`(✦ 지움): 왼쪽 시든 나무(빛나는 구멍·마른 잎·보라 실), 오른쪽 다 핀 나무(잎 덩어리·분홍 꽃).
둘 다 벼랑 위에 그려 왔지만 줄기가 서로 다르다("같은 줄기로" 청해도 안 됐다). 그래서
  - 줄기·뿌리·구멍은 늘 시든 나무 하나(단계마다 같은 나무),
  - 잎은 다 핀 나무에서 덩어리(`CLUMPS`개)로 잘라 단계마다 얹는다: 1 → 3, 2 → 7, 3 → 10, 4 → 12(아래부터, 0045의 수),
  - 보라 실은 첫 잎이 돌아올 때(단계 1부터) 지운다.
벼랑은 자른다: 마을 지도의 나무 자리가 이미 벽 둔덕(###Y###)이라 그 위에 뿌리 끝이 얹힌다(`CUT`).
원본 점 블록이 약 4.5px라 키 `HEIGHT`로 줄이면 거의 원래 점 하나 = 게임 점 하나 — 다수결 줄이기(`sprite_ai.shrink`)로 옮긴다.

칸: 가로 `W`·세로 `H` 다섯 칸 한 줄. 밑동(줄기 아래 가운데)이 칸의 (`W` / 2, `BASE`)에 온다 — 게임(`art::TREE_*`)이 이 점을 지도의 Y 자리에 맞춘다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sprite_ai  # noqa: E402
from sprite_ai import OUTLINE, lum, outline, ramp, shared_palette, shrink  # noqa: E402

SRC = HERE / "sources" / "ai" / "sprites" / "tree_v1.png"
HALF = 512          # 왼쪽 시든 나무 | 오른쪽 다 핀 나무
CUT = 452           # 이 줄 아래(벼랑 앞면)는 버린다
BASE_SRC = 418      # 원본에서 밑동(구멍 아래) 줄
HEIGHT = 100        # 줄인 나무 키(잎 꼭대기 ~ 자른 줄)
W, H = 128, 112
CLUMPS = 12
STAGE_CLUMPS = [0, 3, 7, 10, 12]
NCOL = 32           # 나무는 갈색·초록·분홍·보라·빛 — 20색으로는 잎의 밝은 단이 빠졌다


def hls(a):
    import colorsys
    flat = a.reshape(-1, 3) / 255.0
    out = np.array([colorsys.rgb_to_hls(*p) for p in flat])
    return out.reshape(a.shape[:2] + (3,))


def body_mask(a):
    keep = sprite_ai.magenta(a) < 60
    keep[CUT:] = False
    return sprite_ai.main_parts(keep, 0.05)


def thread_mask(a, keep):
    h = hls(a)
    # 밝기 조건이 없으면 자두빛 외곽선(색상 0.86·밝기 0.11)까지 실로 잡혔다.
    return keep & (h[..., 0] > 0.68) & (h[..., 0] < 0.9) & (h[..., 2] > 0.25) & (h[..., 1] > 0.28)


def unthread(a, keep):
    """보라 실 점을 둘레(실이 아닌 점)의 평균 색으로 메운다 — 안쪽으로 몇 번."""
    a = a.copy()
    t = thread_mask(a, keep)
    for _ in range(8):
        if not t.any():
            break
        ys, xs = np.nonzero(t)
        nt = t.copy()
        for y, x in zip(ys, xs):
            y0, y1, x0, x1 = max(0, y - 2), y + 3, max(0, x - 2), x + 3
            ok = keep[y0:y1, x0:x1] & ~t[y0:y1, x0:x1]
            if ok.sum() >= 3:
                a[y, x] = a[y0:y1, x0:x1][ok].mean(0)
                nt[y, x] = False
        t = nt
    return a


def glow_mask(a, keep):
    """구멍 속 불빛(밝은 주황·노랑) — 드물어 많이 쓴 색 순서로는 색표에 못 든다."""
    h = hls(a)
    return keep & (h[..., 0] > 0.04) & (h[..., 0] < 0.16) & (h[..., 1] > 0.55) & (h[..., 2] > 0.6)


def blossom_mask(a):
    h = hls(a)
    return (h[..., 0] > 0.88) & (h[..., 1] > 0.6) & (h[..., 2] > 0.4)


def must_have(pal, groups):
    """색표 끝 몇 칸을 드문 색(무리마다 가장 많이 쓴 색 둘)으로 바꾼다."""
    cols = np.array(pal.getpalette()[:NCOL * 3]).reshape(-1, 3)
    extra = []
    for g in groups:
        if len(g) == 0:
            continue
        c, n = np.unique(g.reshape(-1, 3), axis=0, return_counts=True)
        extra += [c[i] for i in np.argsort(-n)[:2]]
    cols[NCOL - len(extra):] = extra
    out = Image.new("P", (1, 1))
    out.putpalette([int(v) for c in cols for v in c] + [0] * (768 - 3 * NCOL))
    return out


def crown(a, keep):
    """다 핀 나무의 잎(초록) + 그 둘레(외곽선·꽃·잎 사이 가지) 마스크와, 덩어리 번호(아래 → 위)."""
    h = hls(a)
    green = keep & (h[..., 0] > 0.18) & (h[..., 0] < 0.45) & (h[..., 2] > 0.25)
    green = sprite_ai.main_parts(green, 0.01)
    m = green.copy()
    for _ in range(6):                       # 덩어리 외곽선·꽃이 잎 둘레 약 6px 안
        m = sprite_ai._dilate(m)
    m &= keep
    ys, xs = np.nonzero(m)
    # 잎 바닥 줄보다 아래(줄기 쪽)는 잎이 아니다.
    bottom = np.nonzero(green.any(1))[0].max()
    m[bottom + 4:] = False
    # 덩어리: 초록 점 위치 k-평균(정해진 시작점 — 같은 원본이면 같은 결과).
    gy, gx = np.nonzero(green)
    pts = np.stack([gx, gy], 1).astype(float)
    order = np.argsort(pts[:, 1] * 10000 + pts[:, 0])
    centers = pts[order[np.linspace(0, len(order) - 1, CLUMPS).astype(int)]]
    for _ in range(25):
        d = ((pts[:, None, :] - centers[None]) ** 2).sum(-1)
        lab = d.argmin(1)
        centers = np.array([pts[lab == k].mean(0) if (lab == k).any() else centers[k] for k in range(CLUMPS)])
    rank = np.argsort(-centers[:, 1])        # 아래(큰 y)부터
    ys, xs = np.nonzero(m)
    d = ((np.stack([xs, ys], 1)[:, None, :] - centers[None]) ** 2).sum(-1)
    clump = np.full(m.shape, -1)
    clump[ys, xs] = np.argsort(rank)[d.argmin(1)]
    return m, clump


def build():
    img = np.asarray(Image.open(SRC).convert("RGB")).astype(int)
    dead, live = img[:, :HALF], img[:, HALF:]
    dk, lk = body_mask(dead), body_mask(live)
    # 두 나무 맞추기: 벼랑 앞면 위쪽 줄(CUT 바로 위)의 가로 가운데끼리.
    def cx(k):
        xs = np.nonzero(k[CUT - 12:CUT].any(0))[0]
        return (xs.min() + xs.max()) / 2
    shift = int(round(cx(dk) - cx(lk)))
    cm, clump = crown(live, lk)
    clean = unthread(dead, dk)
    # 한 장의 큰 캔버스(원본 좌표)에 단계마다 합친다.
    top = min(np.nonzero(dk.any(1))[0].min(), np.nonzero(cm.any(1))[0].min())
    stages = []
    for n in STAGE_CLUMPS:
        a = (dead if n == 0 else clean).copy()
        k = dk.copy()
        on = cm & (clump < n)
        ys, xs = np.nonzero(on)
        xs2 = xs + shift
        ok = (xs2 >= 0) & (xs2 < HALF)
        a[ys[ok], xs2[ok]] = live[ys[ok], xs[ok]]
        k[ys[ok], xs2[ok]] = True
        stages.append((a[top:CUT], k[top:CUT]))
    # 색 수는 공용 변환기의 전역 — 나무만 32색으로 쓰고 되돌린다(재현 검사가 한 프로세스에서 다른 시트도 만든다).
    keep_ncol, sprite_ai.NCOL = sprite_ai.NCOL, NCOL
    try:
        return grade(_build(img, dead, live, dk, cm, clump, clean, stages, top))
    finally:
        sprite_ai.NCOL = keep_ncol


def _build(img, dead, live, dk, cm, clump, clean, stages, top):
    pal = shared_palette([(img[:, :HALF][top:CUT], dk[top:CUT]), (stages[-1][0], stages[-1][1])], near=24)
    pal = must_have(pal, [
        dead[thread_mask(dead, dk)],                         # 보라 실
        dead[glow_mask(dead, dk)],                           # 구멍 속 불빛
        live[cm & blossom_mask(live)],                       # 분홍 꽃
    ])
    scale = HEIGHT / (CUT - top)
    xs = np.nonzero(np.any([k.any(0) for _, k in stages], 0))[0]
    x0 = xs.min()
    base_x = (np.nonzero(dk[BASE_SRC - 30:BASE_SRC].any(0))[0].mean())  # 밑동 둘레 가운데
    sheet = np.zeros((H, W * len(stages), 4), np.uint8)
    for i, (a, k) in enumerate(stages):
        # 모든 단계를 같은 상자(x0.., top..CUT)로 줄여 칸 안 자리가 같게: 상자 밖은 비워 두고 왼쪽 위에 한 점 찍지 않는다.
        box = np.zeros_like(k)
        box[:, x0:] = k[:, x0:]
        a2, k2 = a.copy(), box
        # shrink는 마스크의 겉 네모로 자르므로, 단계마다 겉 네모가 같도록 왼쪽 위·오른쪽 아래 모서리를 잠깐 켠다.
        corners = [(0, x0), (k2.shape[0] - 1, xs.max())]
        saved = [(y, x, k2[y, x]) for y, x in corners]
        for y, x in corners:
            k2[y, x] = True
        sp = shrink(a2, k2, scale, pal)
        for (y, x, v), (yy, xx) in zip(saved, [(0, 0), (sp.shape[0] - 1, sp.shape[1] - 1)]):
            if not v:
                sp[yy, xx] = 0
        if i == 0:
            sp = threads_back(sp, dead[top:CUT], dk[top:CUT], x0, xs.max())
        sp = outline_body(sp)
        bx = int(round((base_x - x0) * scale))
        by = int(round((BASE_SRC - top) * scale))
        ox, oy = W // 2 - bx, BASE - by
        h, w = sp.shape[:2]
        cell = np.zeros((H, W, 4), np.uint8)
        for y in range(h):
            for x in range(w):
                yy, xx = oy + y, ox + x
                if 0 <= yy < H and 0 <= xx < W and sp[y, x, 3]:
                    cell[yy, xx] = sp[y, x]
        sheet[:, i * W:(i + 1) * W] = cell
    return sheet


# 마을 톤(0238 덧붙임, 사장 "나무 톤이 좀 튄다"): 원본 잎은 채도 0.8~1.0의 쨍한 초록, 줄기는 자줏빛 짙은 갈색이라
# 마을(풀·작은 나무·덤불 채도 0.3~0.5, 둔덕 흙 갈색)에서 튀었다. 게임 화면에서 잰 마을 색 단계로 밝기 순서대로 옮긴다.
LEAF = [(26, 52, 44), (36, 93, 68), (60, 138, 78), (82, 166, 76), (98, 200, 91), (150, 214, 120)]
BARK = [(52, 36, 38), (78, 56, 44), (98, 70, 52), (118, 84, 62), (142, 106, 78)]


def grade(sheet):
    """잎(초록)·줄기(갈색·자줏빛 갈색)를 마을 단계로. 꽃 분홍·불빛·보라 실·외곽선은 그대로, 거의 검정은 외곽선 색."""
    import colorsys
    out = sheet.copy()
    on = out[..., 3] > 0
    keys = {tuple(int(v) for v in out[y, x, :3]) for y, x in zip(*np.nonzero(on))}
    leaf, bark, remap = [], [], {}
    for k in keys:
        h, l, s_ = colorsys.rgb_to_hls(*(v / 255 for v in k))
        if l < 0.07:
            remap[k] = OUTLINE
        elif 0.18 <= h <= 0.52 and s_ > 0.2:
            leaf.append(k)
        elif (h >= 0.82 or h <= 0.12) and s_ < 0.62 and l < 0.5 and k != OUTLINE:
            bark.append(k)
    for group, steps in ((leaf, LEAF), (bark, BARK)):
        if not group:
            continue
        ls = [lum(c) for c in group]
        lo, hi = min(ls), max(ls)
        for c, L in zip(group, ls):
            remap[c] = ramp(steps, (L - lo) / max(1.0, hi - lo))
    for y, x in zip(*np.nonzero(on)):
        k = tuple(int(v) for v in out[y, x, :3])
        if k in remap:
            out[y, x, :3] = remap[k]
    return out


BASE = 96           # 칸 위에서 밑동 줄(잎 꼭대기가 칸 위 끝에 닿지 않게 — 86이면 다 핀 나무가 6px 잘렸다)


def threads_back(sp, a, k, x0, x1):
    """보라 실은 원본에서 1~2점 굵기라 다수결로 줄이면 사라진다 → 원본 실 점이 한 칸의 1/5 넘게 지나가면 그 칸을 실 색으로."""
    t = thread_mask(a, k)[:, x0:x1 + 1].astype(np.uint8) * 255
    small = np.asarray(Image.fromarray(t).resize((sp.shape[1], sp.shape[0]), Image.BOX)) > 30
    c, n = np.unique(a[thread_mask(a, k)].reshape(-1, 3), axis=0, return_counts=True)
    ink = c[np.argmax(n)]
    out = sp.copy()
    on = small & (out[..., 3] > 0)
    out[on, :3] = ink
    return out


def outline_body(sp):
    """겉 테두리만 외곽선 한 겹(원본 외곽선이 줄이며 끊긴 곳을 잇는다). 칸 크기는 그대로."""
    pad = np.zeros((sp.shape[0] + 2, sp.shape[1] + 2, 4), np.uint8)
    pad[1:-1, 1:-1] = sp
    return outline(pad)


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else str(HERE.parents[1] / "assets" / "art" / "tree.png")
    Image.fromarray(build(), "RGBA").save(out)
    print(out)
