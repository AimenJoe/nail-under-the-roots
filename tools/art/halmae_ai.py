"""누룩 할매(오래된 균사) 스프라이트(0206 C4, 0219): 원본 한 장에서 두 크기.

실행: python3 tools/art/halmae_ai.py [출력 경로]   (기본 assets/art/halmae.png)

원본: `sources/ai/sprites/halmae_v1.png`(✦ 지움). 크기 기준 그림(`halmae_silhouette.py`의 `ref_halmae_scale` — 게임 크기 Naïl 옆
할매 실루엣 4배)을 붙여 받았다. 칸 사이로 뿌리가 맞닿아 덩어리로는 못 나눠 열 경계(빈 열)에서 잘랐다.

시트(한 장, 두 줄):
  줄 0(y 0~95): 96×96 칸 여섯 — 가장 깊은 곳(0162). 몸(버섯 끝~밑동) 외곽선 안 키 84 → Naïl의 다섯 배쯤.
  줄 1(y 96~135): 40×40 칸 여섯 — 마을 이웃(0045). 같은 원본을 외곽선 안 키 34로(32칸 키 28은 얼굴이 사라졌다).
  칸: 0·1 숨(들숨·날숨), 2 당기기(질끈 눈, 뿌리가 팽팽), 3 움찔(매듭이 끊길 때, 걱정 눈썹), 4 놓아주기(눈을 뜬다), 5 기쁨(^ ^)
  흰 실루엣 줄은 없다 — 할매는 맞지 않는다(0162).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import OUTLINE, Source, Spec, convert, seat  # noqa: E402

SRC = HERE / "sources" / "ai" / "sprites" / "halmae_v1.png"
NAMES = ["breathe_in", "breathe_out", "pull", "flinch", "release", "happy"]
BOXES = [
    (2, 18, 335, 330), (346, 22, 679, 330), (685, 18, 1024, 343),
    (1, 357, 331, 687), (331, 357, 676, 687), (676, 357, 1024, 687),
]
BIG, BIG_FOOT, BIG_H = 96, 94, 84
SMALL, SMALL_FOOT, SMALL_H = 40, 38, 34
# 원본 아래 줄(움찔·놓아주기·기쁨)은 위 줄보다 4.5% 크게 왔다(버섯 끝~몸 밑 320px 대 306px). 같은 배율이면 표정마다 몸이
# 커졌다 작아진다 — 줄마다 기준 칸을 두고, 아래 줄은 그 비만큼 줄인다. 기준 칸의 상자 키(뿌리 끝까지)는 위 312·아래 330.
ROW_FIX = 306 / 320 * 330 / 312


def spec(h, names, ref):
    # tidy: 옆 칸에서 넘어온 뿌리 끝을 버린다. 색표는 늘 위 줄 들숨 칸에서(줄마다 고르면 아래 줄 볼이 갈색이 됐다) —
    # 그 칸을 둘째 원본으로 붙이고 그 결과는 버린다(`two_rows`).
    frames = {k: BOXES[NAMES.index(k)] for k in names}
    return Spec(
        sources=[Source(SRC, frames, groups=dict.fromkeys(names, ref), tidy=0.05, strip=4),
                 Source(SRC, {"pal": BOXES[0]}, groups={"pal": "pal"}, tidy=0.05, strip=4)],
        palette_ref=(1, "pal"),
        body_h=round(h),
    )


def two_rows(h):
    out = convert(spec(h, NAMES[:3], "breathe_in"))
    out.update(convert(spec(h * ROW_FIX, NAMES[3:], "release")))
    out.pop("pal")
    return out


EYE = (60, 44, 70)
# 마을 크기에서 다시 찍는 눈(칸 0~3 감은 눈 ‿, 4 뜬 눈, 5 기쁜 눈 ^): 눈 가운데에서의 (dy, dx).
GLYPHS = {
    "closed": [(0, -1), (0, 0)],
    "open": [(-1, 0), (0, 0)],
    "happy": [(0, -1), (-1, 0), (0, 1)],
}
FACE = ["closed", "closed", "closed", "closed", "open", "happy"]


def features(cell):
    """큰 칸에서 눈 둘·볼 둘의 가운데(칸 좌표, 몸 겉 네모 안의 비율). 눈 = 몸 가운데 40% 폭·키 45~85% 줄 안의 어두운 점
    중 쪽마다 맨 위 3줄(입이 섞이면 눈이 볼 높이로 내려갔다, 움찔 칸의 흩어진 실 조각은 가운데 폭 밖이라 빠진다), 볼 = 분홍 점. 왼쪽·오른쪽은 몸 가운데로 가른다."""
    on = cell[..., 3] > 0
    ys, xs = np.nonzero(on.any(1))[0], np.nonzero(on.any(0))[0]
    y0, y1, x0, x1 = ys.min(), ys.max(), xs.min(), xs.max()
    cx = (x0 + x1) / 2
    rgb = cell[..., :3].astype(int)
    lum = (rgb * [0.299, 0.587, 0.114]).sum(-1)
    pink = on & (rgb[..., 0] > 200) & (rgb[..., 1] < 200) & (rgb[..., 0] - rgb[..., 2] > 25)
    dark = on & (lum < 100)
    band = np.zeros_like(on)
    band[int(y0 + (y1 - y0) * 0.45):int(y0 + (y1 - y0) * 0.85), int(cx - (x1 - x0) * 0.2):int(cx + (x1 - x0) * 0.2)] = True
    rel = lambda y, x: ((y - y0) / (y1 - y0), (x - x0) / (x1 - x0))
    out = {}
    for name, m in (("eye", dark & band & ~pink), ("blush", pink)):
        for side, keep in (("l", lambda x: x < cx), ("r", lambda x: x >= cx)):
            pts = [(y, x) for y, x in zip(*np.nonzero(m)) if keep(x)]
            if pts and name == "eye":                    # 입(가운데 아래 어두운 점)을 빼려고 맨 위 3줄만
                top = min(y for y, _ in pts)
                pts = [(y, x) for y, x in pts if y <= top + 3]
            if pts:
                out[name + side] = rel(np.mean([p[0] for p in pts]), np.mean([p[1] for p in pts]))
    return out


def face(small, big, kind):
    """마을 크기 칸에 눈·볼을 다시 찍는다: 원본의 4px 눈선은 키 34로 줄이면 1px도 안 남아 얼굴이 볼만 남거나(칸 0~2·5)
    다 사라졌다(움찔·놓아주기). 큰 칸에서 찾은 자리를 몸 겉 네모 비율로 옮긴다. 큰 칸에 볼이 없으면 눈 아래로."""
    out = small.copy()
    on = small[..., 3] > 0
    ys, xs = np.nonzero(on.any(1))[0], np.nonzero(on.any(0))[0]
    y0, y1, x0, x1 = ys.min(), ys.max(), xs.min(), xs.max()
    at = lambda r: (int(round(y0 + r[0] * (y1 - y0))), int(round(x0 + r[1] * (x1 - x0))))
    f = features(big)
    blush = tuple(int(v) for v in big[big[..., 3] > 0][np.argmax(
        (big[big[..., 3] > 0][:, 0].astype(int) - big[big[..., 3] > 0][:, 1].astype(int)))][:3])
    rgb = small[..., :3].astype(int)
    for side in "lr":
        if "eye" + side not in f:
            continue
        ey, ex = at(f["eye" + side])
        by, bx = at(f["blush" + side]) if "blush" + side in f else (ey + 2, ex + (-1 if side == "l" else 1))
        ey = min(ey, by - 2)                                   # 눈과 볼이 한 줄에 겹치면 볼이 눈을 덮는다
        # 줄이며 남은 눈 찌꺼기(어두운 점)를 얼굴 바탕색으로 덮고 찍는다.
        box = (slice(ey - 2, by + 1), slice(ex - 3, ex + 4))
        sub = rgb[box]
        lums = (sub * [0.299, 0.587, 0.114]).sum(-1)
        light = [tuple(int(c) for c in v) for v in sub[lums > 180]]
        base = max(light, key=light.count, default=None)
        if base:
            for y, x in zip(*np.nonzero(lums < 150)):
                yy, xx = ey - 2 + y, ex - 3 + x
                if on[yy, xx] and tuple(int(v) for v in small[yy, xx, :3]) != OUTLINE:
                    out[yy, xx, :3] = base
        for dy, dx in GLYPHS[kind]:
            if on[ey + dy, ex + dx] and tuple(int(v) for v in small[ey + dy, ex + dx, :3]) != OUTLINE:
                out[ey + dy, ex + dx, :3] = EYE
        for dx in (0, 1 if side == "l" else -1):
            if on[by, bx + dx]:
                out[by, bx + dx, :3] = blush
    return out


def build():
    big, small = two_rows(BIG_H), two_rows(SMALL_H)
    sheet = np.zeros((BIG + SMALL, BIG * len(NAMES), 4), np.uint8)
    for c, k in enumerate(NAMES):
        sheet[:BIG, c * BIG:(c + 1) * BIG] = seat(big[k], BIG, BIG_FOOT)
        big_cell = sheet[:BIG, c * BIG:(c + 1) * BIG]
        sheet[BIG:, c * SMALL:(c + 1) * SMALL] = face(seat(small[k], SMALL, SMALL_FOOT), big_cell, FACE[c])
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/halmae.png"
    Image.fromarray(build(), "RGBA").save(out)
