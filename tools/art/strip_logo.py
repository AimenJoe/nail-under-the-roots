"""나노바나나 원본에서 오른쪽 아래 ✦(Gemini 앱 로고)를 한 번 지운다(docs/decisions/0133).

실행: python3 tools/art/strip_logo.py            # tools/art/sources/ai 아래 아직 안 지운 원본을 모두
      python3 tools/art/strip_logo.py <파일>...   # 새로 넣은 원본만
- 지운 파일에는 PNG 글 칸 `nail-logo-stripped`를 남기고, 다음에는 건너뛴다(두 번 지우면 그림이 상한다).
- ✦는 흰색을 일정한 투명도로 얹은 것이다(관측 = a·255 + (1-a)·원래). 그래서 a를 알면 원래 점을 그대로 되돌린다:
  원래 = (관측 - a·255) / (1 - a). 둘레 색으로 메우면 무늬 있는 자리(뿌리·옷 줄무늬)가 번졌다.
  a 지도(gemini_star_alpha.npz)는 바탕이 고른 1024² 초상화 다섯 장(별 한 번)에서 잰 것이다.
- 별의 크기·자리는 그림 넓이의 제곱근에 비례한다(가장자리에서 0.1165·√(w·h), 반지름 약 0.024·√(w·h)).
  그림마다 자리(±4px)·크기(±4%)는 템플릿과 가장 닮은 곳(고주파끼리 상관)으로, 세기 n은 되돌린 뒤 별 모양이
  남지 않는 값(상관 0)으로 맞춘다. a = 1 - (1 - 템플릿)^n.
- n이 1을 넘으면(FILL_ABOVE) 나노바나나로 고친 그림이다: 고칠 때 앞의 ✦를 모델이 다시 그려(보랏빛·흐림) 흰색 얹기로는
  다 안 지워진다. 그때는 되돌린 뒤 별 자리를 둘레 색으로 메운다. 다만 무늬가 진한 자리는 메우기가 더 보기 싫어 되돌리기만 한다(BLEND_ONLY).
- 눈에 보이지 않는 SynthID 워터마크는 점 값 전체에 퍼져 있어 지울 수 없고, 출처 표시라 지우지 않는다.
게임 그림을 만드는 단계(ai_portraits·ai_illust·ref_pack)는 ✦를 지우지 않는다. 원본이 이미 깨끗하다.
새 원본을 넣으면 이 스크립트를 돌리고, 찍힌 n·방법을 보고 별 자리를 크게 봐서 확인한다.
"""
import os
import sys

import numpy as np
from PIL import Image, PngImagePlugin

import ref_pack

ROOT = os.path.dirname(os.path.abspath(__file__))
AI = os.path.join(ROOT, 'sources', 'ai')
MARK = 'nail-logo-stripped'
_T = np.load(os.path.join(ROOT, 'gemini_star_alpha.npz'))
ALPHA = _T['alpha']             # 1024² 원본 기준, 별 한 번의 a
CENTER = _T['center']           # ALPHA 안의 별 가운데(칸 좌표)
EDGE = 1024 - (863 + CENTER)    # 1024² 원본의 오른쪽·아래 가장자리에서 별 가운데까지
FILL_ABOVE = 1.3
# 고친 그림이지만 메우지 않고 되돌리기만 하는 원본(sources/ai 기준 경로, 확장자 없이): 무늬 위에서 메우기가 덩어리져 보였다.
BLEND_ONLY = {'illust/mother_burrow', 'sheets/kongi'}


def _sample(xs, ys):
    """ALPHA를 (xs, ys) 칸 좌표에서 쌍선형으로 읽는다. 밖은 0."""
    h, w = ALPHA.shape
    x0, y0 = np.floor(xs).astype(int), np.floor(ys).astype(int)
    fx, fy = xs - x0, ys - y0

    def at(y, x):
        ok = (y >= 0) & (y < h) & (x >= 0) & (x < w)
        return np.where(ok, ALPHA[np.clip(y, 0, h - 1), np.clip(x, 0, w - 1)], 0)
    return (at(y0, x0) * (1 - fx) * (1 - fy) + at(y0, x0 + 1) * fx * (1 - fy)
            + at(y0 + 1, x0) * (1 - fx) * fy + at(y0 + 1, x0 + 1) * fx * fy)


def _alpha(w, h, box, dx=0.0, dy=0.0, m=1.0, n=1.0):
    s = np.sqrt(w * h) / 1024
    cx, cy = w - EDGE[0] * s + dx, h - EDGE[1] * s + dy
    ys, xs = np.mgrid[box[1]:box[3], box[0]:box[2]].astype(float)
    t = _sample((xs - cx) / (s * m) + CENTER[0], (ys - cy) / (s * m) + CENTER[1])
    return 1 - (1 - t) ** n


def _box(w, h):
    s = np.sqrt(w * h) / 1024
    cx, cy, r = w - EDGE[0] * s, h - EDGE[1] * s, int(44 * s) + 4
    return (int(cx - r), int(cy - r), min(w, int(cx + r)), min(h, int(cy + r)))


def _hp(x, k=4):
    """고주파: 9×9 평균을 뺀다(무늬 있는 바탕에서도 별 모양만 남게)."""
    p = np.pad(x, k, mode='edge')
    return x - np.lib.stride_tricks.sliding_window_view(p, (2 * k + 1, 2 * k + 1)).mean((2, 3))


def _ncc(a, b):
    a, b = a - a.mean(), b - b.mean()
    return (a * b).sum() / np.sqrt((a * a).sum() * (b * b).sum() + 1e-9)


def _unblend(obs, a):
    return np.clip((obs - a[..., None] * 255) / (1 - a[..., None]), 0, 255)


def fit(rgb):
    """그림(h×w×3)에서 별의 자리·크기·세기를 맞춘다. → (box, dx, dy, m, n)"""
    h, w, _ = rgb.shape
    box = _box(w, h)
    obs = rgb[box[1]:box[3], box[0]:box[2]].astype(float)
    lum = _hp(obs.mean(2))
    best = max(((_ncc(lum, _hp(_alpha(w, h, box, dx, dy, m))), dx, dy, m)
                for dx in np.arange(-4, 4.01, 0.5) for dy in np.arange(-4, 4.01, 0.5)
                for m in (0.96, 0.98, 1.0, 1.02, 1.04)), key=lambda b: b[0])
    for st in (0.25, 0.125):
        _, dx, dy, m = best
        best = max([best] + [(_ncc(lum, _hp(_alpha(w, h, box, dx + i, dy + j, m + k))), dx + i, dy + j, m + k)
                             for i in (-st, 0, st) for j in (-st, 0, st) for k in (-0.01, 0, 0.01)],
                   key=lambda b: b[0])
    _, dx, dy, m = best
    one = _hp(_alpha(w, h, box, dx, dy, m))

    def left(n):   # 되돌린 뒤 남은 별 모양(양수 = 아직 밝은 별)
        return _ncc(_hp(_unblend(obs, _alpha(w, h, box, dx, dy, m, n)).mean(2)), one)
    lo, hi = 0.05, 10.0
    if left(hi) > 0:
        return box, dx, dy, m, hi
    for _ in range(40):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if left(mid) > 0 else (lo, mid)
    return box, dx, dy, m, (lo + hi) / 2


def clean(img, name):
    """img(RGB) → (✦를 지운 그림, n, 방법)"""
    rgb = np.asarray(img).copy()
    h, w, _ = rgb.shape
    box, dx, dy, m, n = fit(rgb)
    x0, y0, x1, y1 = box
    a = _alpha(w, h, box, dx, dy, m, n)
    rgb[y0:y1, x0:x1] = np.round(_unblend(rgb[y0:y1, x0:x1].astype(float), a)).astype(np.uint8)
    out = Image.fromarray(rgb)
    if n <= FILL_ABOVE or name in BLEND_ONLY:
        return out, n, 'unblend'
    # 별보다 조금 크게(템플릿 반지름 48에 3) 메운다: 모델이 다시 그린 별은 가장자리가 번져 있다.
    mask = _alpha(w, h, box, dx, dy, m * (1 + 3 / 48)) > 0.01
    ref_pack.fill_hole(out, {(x0 + x, y0 + y) for y, x in zip(*np.nonzero(mask))})
    return out, n, 'unblend+fill'


def strip(path):
    img = Image.open(path)
    name = os.path.splitext(os.path.relpath(path, AI).replace(os.sep, '/'))[0]
    if img.info.get(MARK) or name.endswith('_palette'):
        return 'skip'
    out, n, how = clean(img.convert('RGB'), name)
    info = PngImagePlugin.PngInfo()
    info.add_text(MARK, '1')
    out.save(path, pnginfo=info, optimize=True)
    return f'{how} n={n:.2f}'


def main(paths):
    if not paths:
        paths = [os.path.join(d, f) for d, _, fs in os.walk(AI) for f in sorted(fs) if f.endswith('.png')]
    for p in sorted(paths):
        print(os.path.relpath(p, AI), strip(os.path.abspath(p)))


if __name__ == '__main__':
    main(sys.argv[1:])
