"""나노바나나로 그린 초상화 원본(tools/art/sources/ai/*.png)을 대화창 초상화 칸(88×88)으로 만든다(docs/decisions/0107·0108).

portraits.py의 sheet()가 부른다. 원본이 있으면 그 칸을 이 그림으로 바꿔 끼우고, 없으면 직접 그린 그림 그대로 둔다.
PNG를 손으로 고치지 않고, 원본에 할 일을 모두 여기에 둔다(같은 원본 = 같은 결과):
  1. 오른쪽 아래 ✦(Gemini 앱 로고)를 둘레 색으로 메운다(네 꼭짓점 별 모양, 원본 512px에서 늘 같은 자리).
  2. 고칠 것(눈 속 따뜻한 빛 → 연보라 등)을 원본별 표대로.
  3. 배경(가장자리와 이어진 밝은 베이지)을 투명하게, 몸과 떨어진 반짝이·티끌은 지운다.
  4. 88×88로 줄이고(덩어리 평균), 같은 캐릭터 표정 넷을 한 팔레트로 맞춘다(디더 없이).
  5. Naïl 털빛 1~3: 털 색만 portraits.FUR_VARIANTS 경사로 옮긴다(목도리·주둥이·앞발·발톱·눈은 그대로).
"""
import colorsys
import os

from PIL import Image

ROOT = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(ROOT, 'sources', 'ai')
W = H = 88
# 표정 순서 = 칸 순서(portraits.py 칸 0~3).
FACES = ('normal', 'happy', 'worried', 'determined')
# 한 캐릭터 표정들을 맞출 팔레트 색 수. 32·48색이면 눈의 보랏빛과 목도리 주황이 털 갈색에 묻혔다.
# 눈(보랏빛)은 점이 적어 팔레트를 나눌 때 사라지므로 EYE_COLORS칸을 따로 떼어 둔다.
PALETTE = 64
EYE_COLORS = 8
# Gemini 앱 로고(✦): 원본 512px 기준 가운데·반지름(네 꼭짓점 별).
STAR = (451.5, 452.5, 12.5)


def _hsv(c):
    return colorsys.rgb_to_hsv(c[0] / 255, c[1] / 255, c[2] / 255)


def _erase_star(img):
    """✦ 자리를 둘레 색으로 메운다: 별 밖의 이웃 색 가운데 가장 흔한 것을 안쪽으로 번져 채운다."""
    cx, cy, r = STAR
    sx, sy = img.width / 512, img.height / 512
    cx, cy, r = cx * sx, cy * sy, r * sx
    px = img.load()
    hole = set()
    for y in range(int(cy - r) - 1, int(cy + r) + 2):
        for x in range(int(cx - r) - 1, int(cx + r) + 2):
            dx, dy = abs(x - cx) / r, abs(y - cy) / r
            if dx ** (2 / 3) + dy ** (2 / 3) <= 1.15:
                hole.add((x, y))
    while hole:
        filled = []
        for (x, y) in hole:
            near = [px[x + i, y + j] for i in (-1, 0, 1) for j in (-1, 0, 1)
                    if (i or j) and (x + i, y + j) not in hole]
            if len(near) >= 3:
                filled.append(((x, y), max(set(near), key=near.count)))
        if not filled:
            break
        for (p, c) in filled:
            px[p] = c
            hole.discard(p)


def _eyes_to_lavender(img, boxes):
    """눈 속 주황·노랑 빛을 보통 표정 눈의 연보라로(결의 표정만 눈동자가 호박색으로 나왔다).
    털(채도 0.39~0.52·밝기 0.47~0.63)과 가르려고 채도가 높거나(주황) 아주 밝은(노랑) 점만, 흰 반사는 위쪽만 남긴다."""
    lo, hi = (88, 70, 112), (176, 162, 212)
    px = img.load()
    for (x0, y0, x1, y1), white_from in boxes:
        for y in range(y0, y1):
            for x in range(x0, x1):
                h, s, v = _hsv(px[x, y])
                warm = h < 0.2 and s > 0.12 and (s > 0.55 or v > 0.7)
                low_white = s <= 0.12 and v > 0.85 and y >= white_from
                if warm or low_white:
                    t = min(1.0, max(0.0, (v - 0.4) / 0.6))
                    px[x, y] = tuple(int(lo[i] + (hi[i] - lo[i]) * t) for i in range(3))


# 원본별로 더 고칠 것(원본 512px 좌표).
FIXES = {
    'nail_determined': lambda img: _eyes_to_lavender(
        img, [((226, 250, 284, 288), 272), ((400, 244, 430, 266), 258)]),
}


def _character_mask(img, tol=34):
    """가장자리와 이어진 배경(밝은 베이지)을 빼고, 남은 것 가운데 가장 큰 덩어리(캐릭터)만 참."""
    w, h = img.size
    px = img.load()
    corners = [px[0, 0], px[w - 1, 0], px[0, h // 2], px[w - 1, h // 2]]
    bg = tuple(sum(c[i] for c in corners) // len(corners) for i in range(3))
    near_bg = lambda c: sum((c[i] - bg[i]) ** 2 for i in range(3)) <= tol * tol
    is_bg = [[False] * w for _ in range(h)]
    stack = [(x, 0) for x in range(w)] + [(x, h - 1) for x in range(w)]
    stack += [(0, y) for y in range(h)] + [(w - 1, y) for y in range(h)]
    while stack:
        x, y = stack.pop()
        if not (0 <= x < w and 0 <= y < h) or is_bg[y][x] or not near_bg(px[x, y]):
            continue
        is_bg[y][x] = True
        stack += [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
    # 남은 것의 덩어리: 가장 큰 것만(배경 위 반짝이·티끌을 지운다).
    label = [[0] * w for _ in range(h)]
    best, best_n, n = 0, 0, 0
    for y0 in range(h):
        for x0 in range(w):
            if is_bg[y0][x0] or label[y0][x0]:
                continue
            n += 1
            count, stack = 0, [(x0, y0)]
            label[y0][x0] = n
            while stack:
                x, y = stack.pop()
                count += 1
                for (i, j) in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    xx, yy = x + i, y + j
                    if 0 <= xx < w and 0 <= yy < h and not is_bg[yy][xx] and not label[yy][xx]:
                        label[yy][xx] = n
                        stack.append((xx, yy))
            if count > best_n:
                best, best_n = n, count
    return [[label[y][x] == best for x in range(w)] for y in range(h)]


def _shrink(img, mask):
    """88×88로 줄인다: 칸마다 캐릭터 점의 평균 색, 칸의 절반 넘게 캐릭터면 불투명."""
    w, h = img.size
    px = img.load()
    out = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    o = out.load()
    for oy in range(H):
        y0, y1 = oy * h // H, (oy + 1) * h // H
        for ox in range(W):
            x0, x1 = ox * w // W, (ox + 1) * w // W
            acc, cnt, tot = [0, 0, 0], 0, 0
            for y in range(y0, y1):
                for x in range(x0, x1):
                    tot += 1
                    if mask[y][x]:
                        c = px[x, y]
                        acc[0] += c[0]
                        acc[1] += c[1]
                        acc[2] += c[2]
                        cnt += 1
            if cnt * 2 > tot:
                o[ox, oy] = (acc[0] // cnt, acc[1] // cnt, acc[2] // cnt, 255)
    return out


def _shared_palette(cells):
    """표정들을 한 팔레트로(디더 없이). 투명은 그대로."""
    strip = Image.new('RGB', (W * len(cells), H), (0, 0, 0))
    for i, c in enumerate(cells):
        flat = Image.new('RGB', (W, H), (0, 0, 0))
        flat.paste(c, (0, 0), c)
        strip.paste(flat, (i * W, 0))
    # 투명 칸의 검정이 팔레트를 먹지 않게, 불투명 점만 모아 팔레트를 만든다.
    opaque = [c.getpixel((x, y))[:3] for c in cells for y in range(H) for x in range(W)
              if c.getpixel((x, y))[3]]

    def is_eye(p):
        h, s, v = _hsv(p)
        return 0.6 <= h <= 0.85 and s > 0.12 and v > 0.2

    def colors(points, n):
        if not points:
            return []
        img = Image.new('RGB', (len(points), 1))
        img.putdata(points)
        q = img.quantize(colors=n, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
        used = sorted({i for i in q.getdata()})
        raw = q.getpalette()
        return [tuple(raw[i * 3:i * 3 + 3]) for i in used]

    eye = [p for p in opaque if is_eye(p)]
    rest = [p for p in opaque if not is_eye(p)]
    entries = colors(eye, EYE_COLORS) + colors(rest, PALETTE - EYE_COLORS)
    pal = Image.new('P', (1, 1))
    flat_pal = [v for c in entries for v in c]
    pal.putpalette(flat_pal + flat_pal[-3:] * (256 - len(entries)))
    out = []
    for c in cells:
        flat = Image.new('RGB', (W, H), (0, 0, 0))
        flat.paste(c, (0, 0), c)
        q = flat.quantize(palette=pal, dither=Image.Dither.NONE).convert('RGBA')
        q.putalpha(c.getchannel('A'))
        out.append(q)
    return out


def _is_fur(c):
    """털 갈색인가. 목도리·매듭(붉은 주황), 주둥이(밝은 베이지와 그 붉은 그늘), 앞발·볼(분홍), 눈(보라), 선(자두빛)은 아니다.
    88×88 팔레트를 위치와 함께 재 보니: 털은 밝은 단일수록 노란 쪽(색상 0.07~0.09, 밝기 ≤0.64), 어두운 단은 붉은 쪽(0.01~0.06).
    주둥이 그늘(0.05·밝기 0.62)과 매듭(0.04·0.63)은 밝은데 붉은 쪽이라 털이 아니다."""
    h, s, v = _hsv(c)
    # 밝은 결(밝기 0.66~0.8)도 털이다(잿빛 결의 표정 이마에 주황 획이 남았다). 같은 밝기의 주둥이·발톱은 자리로 지킨다(_cream).
    if not (0.25 <= s <= 0.5 and v <= 0.8):
        return False
    return 0.065 <= h <= 0.12 or (h < 0.065 and v < 0.55)


def _cream(cell):
    """털빛 변형에서 지킬 자리: 밝은 베이지(색상 0.08~0.14, 채도 0.2~0.42, 밝기 ≥0.7) 덩어리 가운데 주둥이(가장 큰 것)와 아래쪽 발톱.
    가장 큰 덩어리(주둥이)는 줄마다 왼쪽~오른쪽 끝까지 채우고(가운데 단·주름이 털과 같은 색이라), 모두 한 칸 넓힌다.
    색만으로 가르면 주둥이에 털색 얼룩이, 주둥이 아래 주름에 회색 줄이 생겼다."""
    px = cell.load()
    light = set()
    for y in range(H):
        for x in range(W):
            p = px[x, y]
            if p[3]:
                h, s, v = _hsv(p)
                if 0.08 <= h <= 0.14 and 0.2 <= s <= 0.42 and v >= 0.7:
                    light.add((x, y))
    comps, seen = [], set()
    for start in light:
        if start in seen:
            continue
        comp, stack = set(), [start]
        seen.add(start)
        while stack:
            x, y = stack.pop()
            comp.add((x, y))
            for i in (-1, 0, 1):
                for j in (-1, 0, 1):
                    q = (x + i, y + j)
                    if q in light and q not in seen:
                        seen.add(q)
                        stack.append(q)
        comps.append(comp)
    # 발톱은 아래쪽(앞발 줄) 덩어리만. 이마의 밝은 결도 같은 색이라 위쪽 작은 덩어리는 털로 둔다(결의 표정 이마에 주황 조각이 남았다).
    area = set()
    for comp in comps:
        if sum(p[1] for p in comp) / len(comp) >= H * 0.68:
            area |= comp
    if comps:
        muzzle = max(comps, key=len)
        area |= muzzle
        for y in {p[1] for p in muzzle}:
            xs = [p[0] for p in muzzle if p[1] == y]
            area.update((x, y) for x in range(min(xs), max(xs) + 1))
    return {(x + i, y + j) for (x, y) in area for i in (-1, 0, 1) for j in (-1, 0, 1)}


def fur_variant(cell, ramp):
    """털 색만 경사 ramp(어두움 → 밝음 5단)로 옮긴다. 털 색의 밝기 순서는 그대로. 주둥이·발톱 자리는 건드리지 않는다."""
    px = cell.load()
    keep = _cream(cell)
    furs = sorted({px[x, y][:3] for y in range(H) for x in range(W) if px[x, y][3] and _is_fur(px[x, y])},
                  key=lambda c: 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2])
    if not furs:
        return cell.copy()
    lum = lambda c: 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
    lo, hi = lum(furs[0]), lum(furs[-1])
    remap = {}
    for c in furs:
        t = (lum(c) - lo) / max(1.0, hi - lo) * (len(ramp) - 1)
        i = min(len(ramp) - 2, int(t))
        f = t - i
        remap[c] = tuple(int(ramp[i][k] + (ramp[i + 1][k] - ramp[i][k]) * f) for k in range(3))
    out = cell.copy()
    o = out.load()
    for y in range(H):
        for x in range(W):
            c = o[x, y]
            if c[3] and c[:3] in remap and (x, y) not in keep:
                o[x, y] = remap[c[:3]] + (c[3],)
    return out


def character(name):
    """name_<표정>.png 넷을 칸 그림 넷으로. 하나라도 없으면 None(직접 그린 그림을 쓴다)."""
    paths = [os.path.join(SRC, f'{name}_{f}.png') for f in FACES]
    if not all(os.path.exists(p) for p in paths):
        return None
    cells = []
    for f, p in zip(FACES, paths):
        img = Image.open(p).convert('RGB')
        _erase_star(img)
        fix = FIXES.get(f'{name}_{f}')
        if fix:
            fix(img)
        cells.append(_shrink(img, _character_mask(img)))
    return _shared_palette(cells)


def cells(fur_variants):
    """{칸 번호: 88×88 RGBA}. Naïl: 칸 0~3, 털빛 k는 칸 18 + (k-1)*4부터."""
    out = {}
    nail = character('nail')
    if nail:
        for i, c in enumerate(nail):
            out[i] = c
        for k, ramp in fur_variants.items():
            for i, c in enumerate(nail):
                out[18 + (k - 1) * 4 + i] = fur_variant(c, ramp)
    return out
