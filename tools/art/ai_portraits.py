"""나노바나나로 그린 초상화 원본(tools/art/sources/ai/*.png)을 대화창 초상화 칸(88×88)으로 만든다(docs/decisions/0107·0108).

portraits.py의 sheet()가 부른다. 원본이 있으면 그 칸을 이 그림으로 바꿔 끼우고, 없으면 직접 그린 그림 그대로 둔다.
PNG를 손으로 고치지 않고, 원본에 할 일을 모두 여기에 둔다(같은 원본 = 같은 결과):
  1. 오른쪽 아래 ✦(Gemini 앱 로고)를 둘레 색으로 메운다(네 꼭짓점 별 모양, 원본 512px에서 늘 같은 자리).
  2. 고칠 것(눈 속 따뜻한 빛 → 연보라 등)을 원본별 표대로.
  3. 배경(가장자리와 이어진 밝은 베이지)을 투명하게, 몸과 떨어진 반짝이·티끌은 지운다.
  4. 88×88로 줄이고(덩어리 평균), 같은 캐릭터 표정 넷을 한 팔레트로 맞춘다(디더 없이).
  5. Naïl 털빛 1~3: 털 색만 portraits.FUR_VARIANTS 경사로 옮긴다(목도리·주둥이·앞발·발톱·눈은 그대로).
  6. 틀 맞추기(FIT): 캐릭터마다 그림 속 여백이 달라 칸에서 크기가 달라 보인다(포포는 높이 71%, Naïl은 94%).
     표정들을 합친 캐릭터 영역으로 정사각형을 잘라, 높이가 칸의 FIT 비율이 되고 가슴 아래는 칸 밖으로 조금 나가게 한다.
     FIT가 1보다 크면 위(귀 끝)를 칸 맨 위에 맞추고 가슴 아래를 칸 밖으로 더 내보낸다(모모의 긴 귀).
"""
import colorsys
import os

from PIL import Image

ROOT = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(ROOT, 'sources', 'ai')
W = H = 88
# 캐릭터: 원본 이름 → (첫 칸, 표정 순서 = 칸 순서, 틀 맞추기). 틀이 None이면 원본 그대로(Naïl이 기준).
# 칸 번호는 portraits.py 맨 위 표, 표정 이름은 원본 파일 이름 `<이름>_<표정>.png`.
# vivid: 채도 높은 작은 색에 따로 뗄 팔레트 칸 수. 포포의 노란 연필이 회색 털에 밀려 황토색이 돼서 켰다.
# Naïl은 0: 켜면 나머지 색의 나뉨이 달라져 주둥이 중간 단이 털 색과 묶이고, 털빛 변형의 주둥이가 물들었다.
CHARACTERS = {
    'nail': (0, ('normal', 'happy', 'worried', 'determined'), None, 0),
    'popo': (5, ('normal', 'happy'), 0.92, 16),
    # 모모는 귀가 길어 높이로 맞추면 얼굴이 작아진다. 1보다 크게 잡아 귀 끝을 칸 맨 위에 두고 가슴을 칸 아래로 내보낸다
    # (처음엔 귀 끝을 잘랐는데 거슬렸다, 사장). 1.22에서 얼굴이 Naïl·포포와 비슷하다.
    'momo': (12, ('normal', 'happy', 'worried'), 1.22, 16),
    # 물든 모모: 돌아온 모모 보통 표정을 고쳐 그려 틀이 1px까지 같다 → 같은 틀. 보라 실은 눈 팔레트(보랏빛) 칸을, 초록 눈은 채도 칸을 쓴다.
    'momo_tainted': (7, ('normal', 'pained'), 1.22, 16),
    # 보리: 원본이 칸을 거의 채워(Naïl처럼) 틀은 그대로. 선도 짙은 자두빛이라 따로 고치지 않는다.
    'bori': (10, ('normal', 'happy'), None, 16),
    # 들쥐 대장 = 물든 보리(0032). 보리 보통 표정을 고쳐 그려 틀이 같다.
    'chief': (9, ('glare',), None, 16),
    # 누룩 할매: 몸이 칸 아래로 이어지고 흰 몸이 배경 베이지와 가까워 아래 끝 번지기를 끈다(CHEST_AT_BOTTOM). 틀은 그대로.
    'halmae': (15, ('normal', 'happy'), None, 16),
}
# 크림색 가슴이 원본 아래 끝에 닿는 캐릭터: 배경 지우기를 아래 끝에서 시작하지 않는다(_character_mask).
CHEST_AT_BOTTOM = {'momo', 'momo_tainted', 'halmae'}
# 몸에 둘러싸인 배경 틈을 지울 캐릭터(_clear_holes): 할매 갓·대 사이.
HOLES = {'halmae'}
# 틀 맞출 때 가슴 아래를 칸 밖으로 보내는 몫(잘린 가슴 = Naïl의 칸 밖으로 나간 앞발과 같은 틀).
FIT_CUT = 0.02
# 틀이 1보다 클 때 귀 끝 위에 남길 여백 몫.
FIT_TOP = 0.01
# 한 캐릭터 표정들을 맞출 팔레트: 눈(보랏빛) EYE_COLORS + 채도 높은 색(캐릭터 표의 vivid) + 나머지 REST_COLORS.
# 32·48색 한 덩어리면 Naïl 눈의 보랏빛과 목도리 주황이 털 갈색에 묻혔다(눈은 점이 적어 팔레트를 나눌 때 사라진다).
EYE_COLORS = 8
REST_COLORS = 56
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


def _plum_lines(img, keep):
    """거의 검은 선을 Naïl·포포와 같은 자두빛 선으로(모모 원본만 선이 검정으로 나왔다). 눈 상자 keep 안(눈동자·속눈썹)은 그대로.
    좌표는 원본 1024px 기준. 밝기 순서는 지켜서 선 가장자리의 섞인 점도 자두빛 쪽으로 옮긴다."""
    lo, hi, top = (40, 24, 38), (64, 42, 52), 56
    sx = img.width / 1024
    boxes = [tuple(int(v * sx) for v in b) for b in keep]
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            c = px[x, y]
            m = max(c)
            if m < top and not any(x0 <= x < x1 and y0 <= y < y1 for (x0, y0, x1, y1) in boxes):
                t = m / top
                px[x, y] = tuple(int(lo[i] + (hi[i] - lo[i]) * t) for i in range(3))


# 모모 눈 상자(원본 1024px): 왼눈·오른눈. 세 표정 모두 같은 자리다.
MOMO_EYES = [(410, 470, 580, 640), (680, 480, 800, 640)]
# 물든 모모 괴로운 표정의 눈물(원본 1024px). 흰빛 물들이기에서 뺀다.
MOMO_TEAR = (395, 590, 465, 675)
# 들쥐 대장(물든 보리)에서 흰빛 물들이기를 뺄 자리(원본 1024px): 수염 왼쪽·오른쪽, 눈 둘.
CHIEF_KEEP = [(395, 520, 570, 710), (750, 510, 910, 700), (430, 410, 570, 510), (660, 420, 760, 500)]
# 몸 둘레에 빛 번짐이 붙은 캐릭터(빛나는 실): 흰빛을 보랏빛으로 물들인다(_tint_glow).
HALO = {
    'momo_tainted': MOMO_EYES + [MOMO_TEAR],
    'chief': CHIEF_KEEP,
}
# 보랏빛 눈 팔레트(EYE_COLORS)를 떼지 않는 캐릭터: 들쥐 대장은 눈이 빨강이고 털이 보라 기운 잿빛이라,
# 떼면 털이 그 칸을 다 차지해 보라 실이 털 색과 묶였다(흰 줄로 보였다). 실은 채도 칸으로 간다.
NO_EYE_SPLIT = {'chief'}

def _glow_red_eyes(img, boxes):
    """눈 상자 안의 빨강을 숯불처럼 밝고 진하게(들쥐 대장). 원본 눈은 검붉은 바탕에 빨간 점이라 88px에서 빛나는 눈으로 읽히지 않았다.
    실에 묶인 동물 = 작게 빛나는 빨간 눈이 이야기 약속이다(nano-banana-prompts.md 세계 규칙). 빨강(색상 0.95~0.05, 채도 > 0.4)만 옮기고 흰 반사·검은 선은 그대로."""
    sx = img.width / 1024
    px = img.load()
    for (x0, y0, x1, y1) in boxes:
        for y in range(int(y0 * sx), int(y1 * sx)):
            for x in range(int(x0 * sx), int(x1 * sx)):
                h, s, v = _hsv(px[x, y])
                if (h < 0.05 or h > 0.95) and s > 0.4 and v > 0.18:
                    r, g, b = colorsys.hsv_to_rgb(0.99, 0.82, min(1.0, 0.62 + v * 0.6))
                    px[x, y] = (int(r * 255), int(g * 255), int(b * 255))


# 원본별로 더 고칠 것(Naïl은 원본 512px 좌표).
FIXES = {
    'nail_determined': lambda img: _eyes_to_lavender(
        img, [((226, 250, 284, 288), 272), ((400, 244, 430, 266), 258)]),
}
FIXES['chief_glare'] = lambda img: _glow_red_eyes(img, CHIEF_KEEP[2:])
for _face in ('normal', 'happy', 'worried', 'tainted_normal', 'tainted_pained'):
    FIXES[f'momo_{_face}'] = lambda img: _plum_lines(img, MOMO_EYES)


def _character_mask(img, tol=34, from_bottom=True):
    """가장자리와 이어진 배경(밝은 베이지)을 빼고, 남은 것 가운데 가장 큰 덩어리(캐릭터)만 참.
    from_bottom: 아래 끝에서도 배경을 번질지. Naïl은 앞발 사이 아래에 갇힌 배경이 있어 켜야 하고,
    모모는 크림색 가슴이 아래 끝에 닿고 배경 베이지와 가까워 켜면 가슴이 통째로 투명해진다(CHEST_AT_BOTTOM)."""
    w, h = img.size
    px = img.load()
    corners = [px[0, 0], px[w - 1, 0], px[0, h // 2], px[w - 1, h // 2]]
    bg = tuple(sum(c[i] for c in corners) // len(corners) for i in range(3))
    near_bg = lambda c: sum((c[i] - bg[i]) ** 2 for i in range(3)) <= tol * tol
    is_bg = [[False] * w for _ in range(h)]
    stack = [(x, 0) for x in range(w)] + ([(x, h - 1) for x in range(w)] if from_bottom else [])
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


def _tint_glow(img, mask, keep):
    """빛나는 실 둘레·가운데의 흰빛을 보랏빛으로. 원본의 흰빛은 실 빛과 밝은 배경 베이지가 섞인 것이라 어두운 대화창에서 하얗게 튀었다(사장).
    - 거의 배경색인 번짐(채도 < 0.12)은 어두운 보라(밝기 0.42): 어두운 바탕 위의 은은한 빛으로 읽힌다.
    - 그보다 짙은 흰빛(채도 < 0.3, 밝기 > 0.75)은 연보라(밝기 0.62~0.88, 밝을수록 밝게).
    번짐을 지우면(바깥에서 벗기기) 남은 실 고리가 칸 한 점(원본 약 12px)보다 가늘어져 88px로 줄일 때 귀 밖 고리가 사라졌다. 그래서 지우지 않고 물들인다.
    keep 상자(눈·눈물, 원본 1024px)는 그대로."""
    sx = img.width / 1024
    boxes = [tuple(int(v * sx) for v in b) for b in keep]
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            if not mask[y][x] or any(x0 <= x < x1 and y0 <= y < y1 for (x0, y0, x1, y1) in boxes):
                continue
            hh, s, v = _hsv(px[x, y])
            if s < 0.3 and v > 0.75:
                value = 0.42 if s < 0.12 else min(0.88, 0.62 + (v - 0.75))
                r, g, b = colorsys.hsv_to_rgb(0.76, 0.42, value)
                px[x, y] = (int(r * 255), int(g * 255), int(b * 255))


def _clear_holes(img, mask, tol=20, min_area=200):
    """캐릭터에 둘러싸여 가장자리와 이어지지 않은 배경 틈(배경색에 아주 가까운 덩어리)도 지운다.
    할매 머리 위 갓·대 사이 틈이 베이지로 남았다(사장). 크림색 몸이 배경과 가까운 캐릭터(모모 가슴)는 지워지면 안 되니 HOLES에 든 캐릭터만."""
    w, h = img.size
    px = img.load()
    corners = [px[0, 0], px[w - 1, 0], px[0, h // 2], px[w - 1, h // 2]]
    bg = tuple(sum(c[i] for c in corners) // len(corners) for i in range(3))
    near = lambda c: sum((c[i] - bg[i]) ** 2 for i in range(3)) <= tol * tol
    seen = [[False] * w for _ in range(h)]
    for y0 in range(h):
        for x0 in range(w):
            if seen[y0][x0] or not mask[y0][x0] or not near(px[x0, y0]):
                continue
            comp, stack = [], [(x0, y0)]
            seen[y0][x0] = True
            while stack:
                x, y = stack.pop()
                comp.append((x, y))
                for (i, j) in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    xx, yy = x + i, y + j
                    if 0 <= xx < w and 0 <= yy < h and not seen[yy][xx] and mask[yy][xx] and near(px[xx, yy]):
                        seen[yy][xx] = True
                        stack.append((xx, yy))
            if len(comp) >= min_area:
                for (x, y) in comp:
                    mask[y][x] = False


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


def _shared_palette(cells, vivid_colors, eye_split=True):
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

    def is_vivid(p):
        h, s, v = _hsv(p)
        return s > 0.45 and v > 0.45

    if not eye_split:
        is_eye = lambda p: False
    eye = [p for p in opaque if is_eye(p)]
    split = vivid_colors > 0
    vivid = [p for p in opaque if split and not is_eye(p) and is_vivid(p)]
    rest = [p for p in opaque if not is_eye(p) and not (split and is_vivid(p))]
    entries = colors(eye, EYE_COLORS) + colors(vivid, vivid_colors) + colors(rest, REST_COLORS)
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


def _fit(images, masks, frac):
    """표정들을 합친 캐릭터 영역으로 같은 정사각형을 잘라(표정끼리 틀이 같게) 높이가 frac이 되게 한다."""
    w, h = images[0].size
    ys = [y for m in masks for y in range(h) if any(m[y])]
    xs = [x for m in masks for x in range(w) if any(m[y][x] for y in range(0, h, 4))]
    top, bottom, left, right = min(ys), max(ys) + 1, min(xs), max(xs) + 1
    side = int((bottom - top) / frac)
    if frac > 1:
        # 키보다 칸이 작으면 위(귀 끝)를 칸 맨 위에 맞추고 남는 것은 가슴 아래로 내보낸다(귀가 잘리면 거슬린다, 사장).
        y0 = top - int(side * FIT_TOP)
        y1 = y0 + side
    else:
        y1 = bottom - int(side * FIT_CUT)
        y0 = y1 - side
    x0 = (left + right) // 2 - side // 2
    out_i, out_m = [], []
    for img, m in zip(images, masks):
        ci = Image.new('RGB', (side, side), img.getpixel((0, 0)))
        ci.paste(img.crop((max(0, x0), max(0, y0), min(w, x0 + side), min(h, y1))), (max(0, -x0), max(0, -y0)))
        cm = [[0 <= x0 + x < w and 0 <= y0 + y < h and m[y0 + y][x0 + x] for x in range(side)] for y in range(side)]
        out_i.append(ci)
        out_m.append(cm)
    return out_i, out_m


def character(name):
    """<이름>_<표정>.png를 칸 그림으로(표정 순서대로). 하나라도 없으면 None(직접 그린 그림을 쓴다)."""
    _, faces, fit, vivid = CHARACTERS[name]
    paths = [os.path.join(SRC, f'{name}_{f}.png') for f in faces]
    if not all(os.path.exists(p) for p in paths):
        return None
    images, masks = [], []
    for f, p in zip(faces, paths):
        img = Image.open(p).convert('RGB')
        _erase_star(img)
        fix = FIXES.get(f'{name}_{f}')
        if fix:
            fix(img)
        images.append(img)
        mask = _character_mask(img, from_bottom=name not in CHEST_AT_BOTTOM)
        if name in HOLES:
            _clear_holes(img, mask)
        if name in HALO:
            _tint_glow(img, mask, HALO[name])
        masks.append(mask)
    if fit:
        images, masks = _fit(images, masks, fit)
    return _shared_palette([_shrink(i, m) for i, m in zip(images, masks)], vivid, name not in NO_EYE_SPLIT)


def cells(fur_variants):
    """{칸 번호: 88×88 RGBA}. 원본이 다 있는 캐릭터만. Naïl 털빛 k는 칸 18 + (k-1)*4부터."""
    out = {}
    for name, (first, _, _, _) in CHARACTERS.items():
        made = character(name)
        if not made:
            continue
        for i, c in enumerate(made):
            out[first + i] = c
        if name == 'nail':
            for k, ramp in fur_variants.items():
                for i, c in enumerate(made):
                    out[18 + (k - 1) * 4 + i] = fur_variant(c, ramp)
    return out
