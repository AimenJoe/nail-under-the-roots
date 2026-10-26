"""나노바나나로 그린 일러스트 원본(tools/art/sources/ai/illust/<장면>.png)을 게임 그림으로 만든다(docs/decisions/0125).

illust.py의 main()이 부른다. 원본이 있으면 그 장면을 이 그림으로 바꿔 끼우고, 없으면 코드로 그린 그림 그대로 둔다.
PNG를 손으로 고치지 않고, 원본에 할 일을 모두 여기에 둔다(같은 원본 = 같은 결과):
  1. 오른쪽 아래 ✦(Gemini 앱 로고)를 장면마다 적어 둔 자리에서 둘레 색으로 메운다(STAR).
     어두운 그림에서 "배경보다 밝은 점"으로 찾으면 뿌리·실까지 별로 알고 지웠다.
  2. 팔레트로 줄인다(디더 없이): 채도 높은 색(보라 실·빛나는 눈)에 VIVID_COLORS칸쯤을 색상별 묶음으로 따로 떼고 나머지 REST_COLORS칸.
     한 덩어리로 줄이면 보라 실이 갈색에 묶여 칙칙한 자줏빛이 됐고, 점이 적은 빨간 눈은 묶음 없이는 사라졌다.
  3. 게임 칸(그림 크기의 절반, 일러스트는 게임 점의 2배 밀도)마다 그 자리에서 가장 흔한 색을 고른다(평균이면 덩어리 경계가 흐려진다).
     다만 칸의 VIVID_SHARE(15%) 넘게 채도 높은 색이면 그 색을 고른다: 작은 빨간 눈(한두 칸)이 흙색에 밀려 사라졌다.
  4. 2배로 키운다(다른 일러스트와 같은 2×2 점).
나노바나나 도트는 진짜 격자가 아니다(한 점이 3~5px로 들쭉날쭉, 자기상관으로 주기가 안 나온다). 그래서 격자를 찾지 않고 칸마다 고른다.
"""
import colorsys
import os

import numpy as np
from PIL import Image

import ref_pack

ROOT = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(ROOT, 'sources', 'ai', 'illust')
VIVID_COLORS = 16
REST_COLORS = 48
VIVID_SHARE = 0.15
# 장면 → 게임 그림 크기(src/illust.rs가 기대하는 크기, docs/art/nano-banana-prompts.md 4의 괄호).
SIZE = {
    'great_tree': (640, 560), 'mother_burrow': (640, 360), 'bori_freed': (640, 360), 'momo_bloom': (640, 360),
    'halmae_deep': (704, 360), 'last_bloom': (640, 440), 'sprout': (640, 360), 'molehill': (640, 360),
    'doto_memory': (640, 360), 'momo_deep': (704, 360), 'kongi_door': (640, 360), 'title': (640, 360),
}
# 그림 안의 큰 네 꼭짓점 반짝이(가운데 x, y, 반지름, 원본 px): ✦ 로고와 똑같은 모양이라 로고처럼 보여 지운다(사장).
# 작은 점 반짝이는 남긴다. 밝은 덩어리(채도 낮고 아주 밝은 점 25개 넘음)를 찾아 구름을 빼고 골랐다.
SPARKLES = {
    'momo_bloom': [(326, 501, 11), (804, 456, 9), (710, 510, 11), (279, 456, 9)],
}
# 세로로 남을 때 자를 높이(0 위·1 아래). 없으면 가운데.
CROP_Y = {}
# 장면 → ✦ 자리(가운데 x, y, 반지름, 원본 px).
STAR = {
    'great_tree': (912, 785, 21),
    # 16:9(1024×572) 그림은 늘 이 자리(수수 시트도 같다).
    'mother_burrow': (935, 483, 19),
    'bori_freed': (935, 483, 19),
    'momo_bloom': (935, 483, 19),
    'sprout': (935, 483, 19),
    # 21:9(1024×434)
    'halmae_deep': (946, 356, 17),
    # 3:2(1024×687)
    'last_bloom': (926, 592, 18),
}


def _vivid(rgb):
    h, s, v = colorsys.rgb_to_hsv(*(c / 255 for c in rgb))
    return s > 0.42 and v > 0.35


def _palette(points, n):
    if not points:
        return []
    img = Image.new('RGB', (len(points), 1))
    img.putdata(points)
    q = img.quantize(colors=n, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    used = sorted(set(q.getdata()))
    raw = q.getpalette()
    return [tuple(raw[i * 3:i * 3 + 3]) for i in used]


def _vivid_palette(points, n, bins=12, min_points=30):
    """채도 높은 색을 색상별 묶음(12)으로 나눠 묶음마다 팔레트를 만든다. 점이 min_points 넘는 묶음은 적어도 두 칸.
    한 덩어리로 나누면 언덕의 올리브 초록이 칸을 다 차지해, 점 231개뿐인 빨간 눈이 다른 색에 묶여 사라졌다."""
    groups = {}
    for p in points:
        h = colorsys.rgb_to_hsv(*(c / 255 for c in p))[0]
        groups.setdefault(int(h * bins) % bins, []).append(p)
    groups = {k: v for k, v in groups.items() if len(v) >= min_points}
    total = sum(len(v) for v in groups.values()) or 1
    out = []
    for k in sorted(groups):
        out += _palette(groups[k], max(2, round(n * len(groups[k]) / total)))
    return out


def _crop_to(img, aspect, y_at=0.5):
    """게임 그림 비율로 자른다(나노바나나는 고를 수 있는 비율이 정해져 있다: 2:1 장면은 21:9로 받는다).
    2% 안쪽 차이(16:9 원본 1024×572와 640×360)는 자르지 않는다. 가로로 남으면 가운데, 세로로 남으면 y_at(0 위·1 아래)에서."""
    w, h = img.size
    if abs(w / h - aspect) / aspect <= 0.02:
        return img
    if w / h > aspect:
        nw = round(h * aspect)
        x0 = (w - nw) // 2
        return img.crop((x0, 0, x0 + nw, h))
    nh = round(w / aspect)
    y0 = round((h - nh) * y_at)
    return img.crop((0, y0, w, y0 + nh))


def scene(name, size=None):
    """원본이 있으면 size(게임 그림 크기, 짝수, 없으면 SIZE)의 그림, 없으면 None."""
    size = size or SIZE.get(name)
    path = os.path.join(SRC, f'{name}.png')
    if not os.path.exists(path):
        return None
    img = Image.open(path).convert('RGB')
    if name in STAR:
        ref_pack.erase_star_at(img, *STAR[name])
    for star in SPARKLES.get(name, ()):
        ref_pack.erase_star_at(img, *star)
    img = _crop_to(img, size[0] / size[1], CROP_Y.get(name, 0.5))
    w, h = img.size
    # 팔레트: 나머지 색은 점을 고르게 뽑아(전부 쓰면 느리다), 채도 높은 색은 원본 점을 전부 쓴다.
    # 4칸마다 뽑았더니 빨간 눈(눈 하나 3×2px)이 빠져 팔레트에 빨강이 없었다.
    allpx = list(img.getdata())
    vivid = [p for p in allpx if _vivid(p)]
    rest = [p for p in allpx[::16] if not _vivid(p)]
    entries = _vivid_palette(vivid, VIVID_COLORS) + _palette(rest, REST_COLORS)
    pal = Image.new('P', (1, 1))
    flat = [v for c in entries for v in c]
    pal.putpalette(flat + flat[-3:] * (256 - len(entries)))
    idx = np.asarray(img.quantize(palette=pal, dither=Image.Dither.NONE))
    is_vivid = np.array([_vivid(c) for c in entries] + [False] * (256 - len(entries)))
    k = len(entries)
    gw, gh = size[0] // 2, size[1] // 2
    out = np.zeros((gh, gw), np.int32)
    for j in range(gh):
        y0, y1 = j * h // gh, (j + 1) * h // gh
        for i in range(gw):
            x0, x1 = i * w // gw, (i + 1) * w // gw
            counts = np.bincount(idx[y0:y1, x0:x1].ravel(), minlength=k)[:k]
            vc = counts * is_vivid[:k]
            if vc.sum() > VIVID_SHARE * counts.sum():
                out[j, i] = vc.argmax()
            else:
                out[j, i] = counts.argmax()
    colors = np.array(entries, np.uint8)
    return Image.fromarray(colors[out], 'RGB').resize(size, Image.NEAREST)
