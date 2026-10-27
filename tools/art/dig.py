"""파 내려가기 그림(0156): 땅 단면 전환에서 화면 가운데 크게 나오는 Naïl·모모, 32×32 × 6칸.
이 파일이 원본이고 assets/art/dig.png는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/dig.py [출력 경로]   (기본 assets/art/dig.png)
줄 0 Naïl 기본 털, 줄 1~3 털빛 1~3(nail.py FURS), 줄 4 돌아온 모모. 칸 0~5 = 한 번 긁는 동작(초당 12칸으로 돈다).

카메라(사장 지시 2026-10-27): 머리가 아래(땅 쪽), 등이 위(우리 쪽). 위에서 내려다보는 뒷모습이라 얼굴은 안 보이고
등·엉덩이·꼬리·뒷발바닥이 보인다. 앞발은 어깨 양옆에서 땅을 긁어 뒤(위)로 흙을 넘긴다.

그리는 법:
  - 몸은 타원 몇 개를 합친 덩어리. 음영은 그 줄의 몸통 폭에서 어디쯤인가로 정한다(왼쪽 가장자리 밝음 h, 가운데 b,
    오른쪽 가장자리 그늘 d) → 원통처럼 읽힌다. 등 가운데에는 털 결 점(h)을 드문드문.
  - 앞발(삽: 분홍 손바닥 + 크림 발톱)·뒷발·꼬리·귀·꽃은 글자 격자를 손으로 찍어 얹는다. 도형으로 그리면 작은 부위가 뭉개진다(0070).
  - 외곽선(o)은 nail.py처럼 자동.
"""
import sys

from PIL import Image

S = 32
NAIL = {
    'o': (43, 29, 42), 'd': (92, 62, 52), 'b': (128, 92, 74), 'h': (166, 128, 102), 'l': (206, 172, 142),
    'p': (242, 152, 164), 'q': (200, 104, 124), 'c': (248, 238, 216), 'C': (206, 190, 164),
    'e': (26, 16, 32), 'w': (255, 255, 255), 'r': (236, 132, 132),
}
MOMO = {
    'o': (43, 29, 42), 'd': (104, 84, 74), 'b': (150, 124, 106), 'h': (190, 164, 140), 'l': (228, 212, 192),
    'p': (240, 150, 164), 'q': (196, 104, 124), 'c': (248, 238, 218), 'C': (214, 200, 182),
    'n': (40, 26, 40), 'W': (255, 255, 255), 'r': (236, 132, 140), 'F': (255, 168, 196), 'Y': (255, 226, 120),
}


def blank():
    return [['.'] * S for _ in range(S)]


def put(g, x, y, rows):
    """rows(문자열 목록)를 (x, y)부터 얹는다. '.'과 ' '는 건너뛴다."""
    for j, r in enumerate(rows):
        for i, ch in enumerate(r):
            if ch not in '. ' and 0 <= x + i < S and 0 <= y + j < S:
                g[y + j][x + i] = ch


def flip(rows):
    return [r[::-1] for r in rows]


def inside_of(ellipses):
    return [[any(((x + .5 - cx) / rx) ** 2 + ((y + .5 - cy) / ry) ** 2 <= 1 for cx, cy, rx, ry in ellipses)
             for x in range(S)] for y in range(S)]


def back(ellipses, grain=(), hump=None):
    """등을 위에서 본 몸통: 줄마다 몸통 폭의 왼쪽 22%는 밝음(h), 오른쪽 26%는 그늘(d), 가운데 털(b).
    맨 아래 두 줄(머리 끝)과 맨 위 줄(엉덩이 끝)은 그늘로 돌려 둥글게. grain = 털 결 점 (x, y)."""
    ins = inside_of(ellipses)
    g = blank()
    ys = [y for y in range(S) if any(ins[y])]
    for y in ys:
        xs = [x for x in range(S) if ins[y][x]]
        x0, x1 = xs[0], xs[-1]
        w = x1 - x0 + 1
        for x in xs:
            t = (x - x0 + 0.5) / w
            if y >= ys[-1] - 1 or y == ys[0]:
                ch = 'd' if t > 0.3 else 'b'
            elif t < 0.22:
                ch = 'h'
            elif t > 0.74:
                ch = 'd'
            else:
                ch = 'b'
            g[y][x] = ch
    if hump:
        # 어깨의 둥근 하이라이트(왼쪽 위 빛): 등이 부풀어 보인다.
        hx, hy, hr = hump
        for y in range(S):
            for x in range(S):
                if g[y][x] == 'b' and (x + .5 - hx) ** 2 + (y + .5 - hy) ** 2 <= hr * hr:
                    g[y][x] = 'h'
    for x, y in grain:
        if g[y][x] == 'b':
            g[y][x] = 'h'
    return g


def head_shade(g, crease_y):
    """머리(아래쪽)는 땅에 박히며 기울어 더 어둡다: crease_y 줄 가운데에 목 주름(d), 그 아래는 밝음(h)을 털(b)로."""
    xs = [x for x in range(S) if g[crease_y][x] != '.']
    if xs:
        x0, x1 = xs[0], xs[-1]
        for x in range(x0 + 2, x1 - 1):
            if g[crease_y][x] in 'bh':
                g[crease_y][x] = 'd'
    for y in range(crease_y + 1, S):
        for x in range(S):
            if g[y][x] == 'h':
                g[y][x] = 'b'


def attach_x(ells, y, width, side, overlap=1):
    """y줄에서 몸통 가장자리에 width 폭의 부위를 overlap만큼 겹쳐 붙일 x."""
    ins = inside_of(ells)
    xs = [x for x in range(S) if ins[min(max(y, 0), S - 1)][x]]
    if not xs:
        return 0 if side == 'L' else S - width
    return xs[0] - width + overlap if side == 'L' else xs[-1] + 1 - overlap


def outline(g):
    out = [r[:] for r in g]
    for y in range(S):
        for x in range(S):
            if g[y][x] != '.':
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                xx, yy = x + dx, y + dy
                if 0 <= xx < S and 0 <= yy < S and g[yy][xx] != '.':
                    out[y][x] = 'o'
                    break
    return out


# ---- Naïl ----
# 위에서 본 두더지: 엉덩이(위, 둥글고 좁다) → 어깨(가장 넓다) → 머리(아래, 좁아지며 땅에 박힌다).
NAIL_BODY = [(15.5, 9.5, 6.5, 6.5), (15.5, 15.5, 8.0, 7.0), (15.5, 21.5, 7.0, 5.5), (15.5, 26.0, 5.0, 3.5)]
NAIL_GRAIN = [(13, 7), (17, 9), (12, 12), (18, 13), (14, 16), (17, 18), (13, 20), (16, 22)]
TAIL = ["q", "p"]
# 뒷발(위에서): 엉덩이 양옆. 찬 발은 발바닥(분홍)이 보인다.
HIND_KICK = [".ppb", "pppb", "qqbb"]
HIND_REST = ["bb", "db"]
# 앞발(삽, 왼쪽 것. 오른쪽은 뒤집는다): 어깨에서 나온 팔(b)이 몸에 붙고, 넓은 분홍 손바닥에 크림 발톱.
PAW_DOWN = [      # 내리긋기(코 옆, 발톱이 아래)
    "...dbbb",
    "..qpppb",
    ".qppppb",
    "qpppppd",
    "pppppq.",
    "c.c.c.c",
    "c.c.c..",
]
PAW_OUT = [       # 바깥으로 흙을 밀어냄: 손이 옆으로 누워 발톱이 바깥(왼쪽)을 향한다
    "....pbbb",
    "..cpppbb",
    "c..ppppb",
    "c.qpppb.",
    "..cqqp..",
    ".c......",
]
PAW_UP = [        # 뒤(위)로 당겨 올림: 손등(털)이 보이고 발톱이 위를 향한다
    ".c.c.c",
    "cbpppb",
    "bppppb",
    "bpppqb",
    ".bqqb.",
]


def nail_frame(i):
    """여섯 칸: 0 둘 다 뒤로 당김, 1 왼손 내리긋기, 2 왼손 밀기, 3 오른손 내리긋기, 4 오른손 밀기, 5 둘 다 내리긋기.
    밀거나 둘 다 긋는 칸은 몸 아래쪽이 한 줄 내려가(파고든다) 머리가 땅에 더 박힌다."""
    dy = [0, 0, 1, 0, 1, 1][i]
    ell = [(cx, cy + dy * (k >= 2), rx, ry) for k, (cx, cy, rx, ry) in enumerate(NAIL_BODY)]
    g = back(ell, [(x, y + dy * (y > 14)) for x, y in NAIL_GRAIN], hump=(12.5, 13.5, 4.2))
    head_shade(g, 23 + dy)
    put(g, 15, 1, TAIL)
    up_left = i % 2 == 0
    for side in ('L', 'R'):
        kick = up_left == (side == 'L')
        rows = HIND_KICK if kick else HIND_REST
        if side == 'R':
            rows = flip(rows)
        put(g, attach_x(ell, 6, len(rows[0]), side), 4 if kick else 5, rows)
    poses = [('up', 'up'), ('down', 'up'), ('out', 'up'), ('up', 'down'), ('up', 'out'), ('down', 'down')][i]
    for side, pose in zip(('L', 'R'), poses):
        rows = {'down': PAW_DOWN, 'out': PAW_OUT, 'up': PAW_UP}[pose]
        y = {'down': 18, 'out': 16, 'up': 12}[pose] + dy
        ov = {'down': 3, 'out': 3, 'up': 2}[pose]
        if side == 'R':
            rows = flip(rows)
        put(g, attach_x(ell, y + 1, len(rows[0]), side, ov), y, rows)
    return outline(g)


# ---- 돌아온 모모(굴토끼) ----
# 위에서 본 토끼: 둥근 엉덩이에 솜뭉치 꼬리, 긴 귀 둘이 등 위에 나란히 눕는다(머리에서 엉덩이 쪽으로).
# 귀 밑동(머리 옆)에 꽃. 작은 앞발은 어깨 옆에서 긁고, 큰 뒷발은 엉덩이 양옆에서 찬다.
MOMO_BODY = [(15.5, 9.5, 7.0, 6.5), (15.5, 15.5, 7.5, 6.5), (15.5, 21.5, 6.5, 5.0), (15.5, 25.5, 4.5, 3.0)]
MOMO_GRAIN = [(12, 8), (19, 9), (13, 14), (18, 16), (12, 20), (18, 21)]
COTTON = ["cc", "ccc", "cc"]
# 귀(왼쪽 것): 등 위에 누운 긴 타원, 안쪽 분홍. 아래(머리 쪽)가 밑동.
EAR = [
    "..hhh.",
    ".hbbbd",
    "hbppbd",
    "hbppbd",
    "hbppbd",
    "hbppbd",
    "hbppbd",
    "hbppbd",
    ".hbpbd",
    "..bbd.",
]
FLOWER = ["FYF", ".F."]
M_FRONT = [       # 작은 앞발(밝은 털 + 분홍 발바닥)
    ".ll",
    "lpp",
    "lp.",
]
M_HIND_KICK = [".ll", "lpp", "lpp", "lll"]
M_HIND_REST = ["ll"]


def momo_frame(i):
    dy = [0, 0, 1, 0, 1, 1][i]
    ell = [(cx, cy + dy * (k >= 2), rx, ry) for k, (cx, cy, rx, ry) in enumerate(MOMO_BODY)]
    g = back(ell, [(x, y + dy * (y > 14)) for x, y in MOMO_GRAIN], hump=(12.5, 13.0, 3.5))
    head_shade(g, 23 + dy)
    put(g, 14, 1, COTTON if i % 2 == 0 else [".cc", "ccc", ".cc"])
    put(g, 8, 10 + dy, EAR)
    put(g, 18, 10 + dy, flip(EAR))
    put(g, attach_x(ell, 20 + dy, 3, 'L', 2), 19 + dy, FLOWER)
    put(g, attach_x(ell, 20 + dy, 3, 'R', 2), 19 + dy, FLOWER)
    fy = [22, 24, 23, 22, 24, 24][i], [24, 22, 24, 23, 22, 24][i]
    put(g, attach_x(ell, fy[0] + dy + 1, 3, 'L', 1), fy[0] + dy, M_FRONT)
    put(g, attach_x(ell, fy[1] + dy + 1, 3, 'R', 1), fy[1] + dy, flip(M_FRONT))
    up_left = i % 2 == 0
    for side in ('L', 'R'):
        kick = up_left == (side == 'L')
        rows = M_HIND_KICK if kick else M_HIND_REST
        if side == 'R':
            rows = flip(rows)
        put(g, attach_x(ell, 6, len(rows[0]), side), 3 if kick else 5, rows)
    return outline(g)


def paint(img, ox, oy, g, pal):
    for y in range(S):
        for x in range(S):
            ch = g[y][x]
            if ch != '.':
                img.putpixel((ox + x, oy + y), pal[ch] + (255,))


def sheet():
    sys.path.insert(0, __file__.rsplit('/', 1)[0])
    from nail import FURS
    rows = [NAIL] + [dict(NAIL, **FURS[k]) for k in (1, 2, 3)]
    img = Image.new('RGBA', (S * 6, S * 5), (0, 0, 0, 0))
    frames = [nail_frame(i) for i in range(6)]
    for r, pal in enumerate(rows):
        for i, g in enumerate(frames):
            paint(img, i * S, r * S, g, pal)
    for i in range(6):
        paint(img, i * S, 4 * S, momo_frame(i), MOMO)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/dig.png')
