"""나노바나나 원본에서 오른쪽 아래 ✦(Gemini 앱 로고)를 한 번 지운다(docs/decisions/0133).

실행: python3 tools/art/strip_logo.py            # tools/art/sources/ai 아래 아직 안 지운 원본을 모두
      python3 tools/art/strip_logo.py <파일>...   # 새로 넣은 원본만
- 지운 파일에는 PNG 글 칸 `nail-logo-stripped`를 남기고, 다음에는 건너뛴다(두 번 지우면 그림이 상한다).
- ✦는 흰색을 일정한 투명도로 얹은 것이다(관측 = a·255 + (1-a)·원래). 그래서 a를 알면 원래 점을 그대로 되돌린다:
  원래 = (관측 - a·255) / (1 - a). 둘레 색으로 메우면 무늬 있는 자리(뿌리·옷 줄무늬)가 번졌다.
  a 지도(gemini_star_alpha.npz)는 바탕이 고른 1024² 초상화 다섯 장(별 한 번)에서 잰 것이다. 잰 값에는 별 밖에도 ±0.06 잡음이 있어
  가운데에서 SUPPORT(템플릿 px) 안만 쓴다: 전부 쓰면 81×81 네모 전체가 조금씩 바뀌고, 메우기가 그 잡음 자리까지 번져 할매 옷 무늬를 지웠다.
- 별의 크기·자리는 그림 넓이의 제곱근에 비례한다(가장자리에서 약 0.118·√(w·h), 반지름 약 0.024·√(w·h)).
  그림마다 자리(±4px)·크기(±4%)는 템플릿과 가장 닮은 곳(고주파끼리 상관)으로, 세기 n은 되돌린 뒤 별 모양이
  남지 않는 값(상관 0)으로 맞춘다. a = 1 - (1 - 템플릿)^n.
- n이 FILL_ABOVE(1.4)를 넘으면 나노바나나로 고친 그림이다: 고칠 때 앞의 ✦를 모델이 다시 그려(보랏빛·흐림) 흰색 얹기로는
  다 안 지워진다. 그때는 되돌린 뒤 별 자리를 둘레 색으로 메운다. 다만 무늬가 진한 자리는 메우기가 더 보기 싫어 되돌리기만 한다(BLEND_ONLY).
- 눈에 보이지 않는 SynthID 워터마크는 점 값 전체에 퍼져 있어 지울 수 없고, 출처 표시라 지우지 않는다.
  같은 까닭으로 원본의 보조 덩어리(C2PA 콘텐츠 자격 `caBX`, EXIF `zTXt`, `sBIT`)도 그대로 옮긴다(점을 고쳤으니 C2PA 해시는 맞지 않는다).
게임 그림을 만드는 단계(ai_portraits·ai_illust·ref_pack)는 ✦를 지우지 않는다. 원본이 이미 깨끗하다.
새 원본을 넣으면 이 스크립트를 돌리고, 찍힌 n·방법을 보고 별 자리를 크게 봐서 확인한다.
"""
import io
import os
import struct
import sys

import numpy as np
from PIL import Image, PngImagePlugin

import ref_pack

ROOT = os.path.dirname(os.path.abspath(__file__))
AI = os.path.join(ROOT, 'sources', 'ai')
MARK = 'nail-logo-stripped'
_T = np.load(os.path.join(ROOT, 'gemini_star_alpha.npz'))
CENTER = _T['center']           # 템플릿 안의 별 가운데(칸 좌표)
SUPPORT = 27                    # 별 끝(반지름 약 24) + 빛 테두리. 그 밖의 잰 값은 잡음(평균 |a| 0.006)
_yy, _xx = np.mgrid[:_T['alpha'].shape[0], :_T['alpha'].shape[1]]
ALPHA = np.where(np.hypot(_xx - CENTER[0], _yy - CENTER[1]) <= SUPPORT, _T['alpha'], 0)   # 1024² 원본 기준, 별 한 번의 a
FILL_A = 0.03                   # 메울 자리: 별 몸(a가 이보다 큰 곳)을 조금 키운 것
EDGE = 1024 - (863 + CENTER)    # 1024² 원본의 오른쪽·아래 가장자리에서 별 가운데까지
FILL_ABOVE = 1.4
# 고친 그림이지만 메우지 않고 되돌리기만 하는 원본(sources/ai 기준 경로, 확장자 없이): 무늬 위에서 메우기가 덩어리져 보였다.
BLEND_ONLY = {'illust/mother_burrow', 'sheets/kongi'}
# 늘 메우는 원본(이름 앞부분): 실의 기억(0273)은 오른쪽 아래가 늘 빈 흙이라 메우기가 보이지 않는다. 고친 판은 n이
# FILL_ABOVE 아래로 나와도(1.09) 모델이 다시 그린 ✦ 자국이 게임 그림의 어두운 흙에 남았다.
FILL_ALWAYS = ('illust/mem_',)
# 메운 뒤 별 자리를 둘레 흙의 평면으로 한 번 더 덮는 원본: 나노바나나로 고친 판(mem_alone)은 다시 그린 ✦의 옅은 얼룩이 남았다.
# 흙에 결이 있는 그림(mem_forest·mem_drought)에서는 평면 자리가 둥글게 밝아 보여 쓰지 않는다.
SMOOTH = {'illust/mem_alone', 'illust/mem_first_sprout', 'illust/mem_dyed_thread'}


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
    if (n <= FILL_ABOVE and not name.startswith(FILL_ALWAYS)) or name in BLEND_ONLY:
        return out, n, 'unblend'
    # 별보다 조금 크게(템플릿 반지름 48에 3) 메운다: 모델이 다시 그린 별은 가장자리가 번져 있다.
    # 늘 메우는 원본은 빈 흙이라 넓게(12): 고친 판은 앞 판의 ✦가 조금 어긋난 자리에 다시 그려져 가장자리 자국이 남았다(mem_alone).
    grow = 12 if name.startswith(FILL_ALWAYS) else 3
    mask = _alpha(w, h, box, dx, dy, m * (1 + grow / 48)) > FILL_A
    ref_pack.fill_hole(out, {(x0 + x, y0 + y) for y, x in zip(*np.nonzero(mask))})
    if name in SMOOTH:
        # 빈 흙 자리: 메운 뒤에도 다시 그린 ✦의 옅은 얼룩이 남았다(mem_alone) → 별 자리(+여유)를 둘레 고리에서 잰
        # 고른 기울기(평면)로 덮는다. 무늬 없는 어두운 흙이라 평면이 둘레와 이어진다.
        ys, xs = np.nonzero(mask)
        cx, cy = x0 + xs.mean(), y0 + ys.mean()
        r = max(xs.max() - xs.min(), ys.max() - ys.min()) / 2 + 6
        out = _smooth_disk(out, cx, cy, r)
        return out, n, 'unblend+fill+smooth'
    return out, n, 'unblend+fill'


def _smooth_disk(img, cx, cy, r, ring=10):
    """(cx, cy) 반지름 r 원을 둘레 고리(r..r+ring)에 맞춘 색 평면으로 바꾸고, 가장자리 4px는 섞는다."""
    a = np.asarray(img).astype(float)
    h, w, _ = a.shape
    yy, xx = np.mgrid[:h, :w]
    dist = np.hypot(xx - cx, yy - cy)
    ringm = (dist > r) & (dist <= r + ring)
    A = np.stack([xx[ringm], yy[ringm], np.ones(ringm.sum())], 1)
    inside = dist <= r + 4
    for c in range(3):
        coef, *_ = np.linalg.lstsq(A, a[..., c][ringm], rcond=None)
        plane = coef[0] * xx + coef[1] * yy + coef[2]
        k = np.clip((r + 4 - dist) / 4, 0, 1)
        a[..., c] = np.where(inside, k * plane + (1 - k) * a[..., c], a[..., c])
    return Image.fromarray(np.clip(np.round(a), 0, 255).astype(np.uint8))


def _chunks(data):
    i, out = 8, []
    while i < len(data):
        n, = struct.unpack('>I', data[i:i + 4])
        out.append((data[i + 4:i + 8], data[i:i + 12 + n]))
        i += 12 + n
    return out


def _keep_extra_chunks(original, saved):
    """saved(PIL이 쓴 PNG)의 IHDR 뒤에 original의 보조 덩어리(그림 점·팔레트·글 칸이 아닌 것)를 그대로 끼운다."""
    extra = [raw for t, raw in _chunks(original) if t not in (b'IHDR', b'PLTE', b'IDAT', b'IEND', b'tEXt', b'iTXt')]
    parts = _chunks(saved)
    return saved[:8] + parts[0][1] + b''.join(extra) + b''.join(raw for _, raw in parts[1:])


def strip(path):
    img = Image.open(path)
    name = os.path.splitext(os.path.relpath(path, AI).replace(os.sep, '/'))[0]
    if img.info.get(MARK) or name.endswith('_palette'):
        return 'skip'
    original = open(path, 'rb').read()
    out, n, how = clean(img.convert('RGB'), name)
    info = PngImagePlugin.PngInfo()
    info.add_text(MARK, '1')
    buf = io.BytesIO()
    out.save(buf, format='PNG', pnginfo=info, optimize=True)
    with open(path, 'wb') as f:
        f.write(_keep_extra_chunks(original, buf.getvalue()))
    return f'{how} n={n:.2f}'


def main(paths):
    if not paths:
        paths = [os.path.join(d, f) for d, _, fs in os.walk(AI) for f in sorted(fs) if f.endswith('.png')]
    for p in sorted(paths):
        print(os.path.relpath(p, AI), strip(os.path.abspath(p)))


if __name__ == '__main__':
    main(sys.argv[1:])
