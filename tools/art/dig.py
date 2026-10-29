"""파 내려가기 그림(0156): 땅 단면 전환에서 화면 가운데 크게 나오는 Naïl·모모, 32×32 × 6칸.
이 파일이 원본이고 assets/art/dig.png는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/dig.py [출력 경로]   (기본 assets/art/dig.png)
줄 0 Naïl 기본 털, 줄 1~3 털빛 1~3(nail.py FURS), 줄 4 돌아온 모모, 줄 5 돌아온 도토(0195, 임시: 모모의 몸 + 청설모 털빛·큰 꼬리·갓).
칸 0~5 긁기 한 번(초당 12칸), 6 고개 들어 카메라 보기, 7 윙크, 8~15 몸통 축으로 한 바퀴(코르크 마개처럼 돌며 내려감).
장면 순서(render.rs `dig_col`): 파다가 → 우리를 보고 윙크 → 뱅글뱅글 돌며 내려간다(사장 제안 2026-10-27).

카메라(사장 지시 2026-10-27): 머리가 아래(땅 쪽), 등이 위(우리 쪽). 위에서 내려다보는 뒷모습이라 얼굴은 안 보이고
등·엉덩이·꼬리·뒷발바닥이 보인다. 앞발은 어깨 양옆에서 땅을 긁어 뒤(위)로 흙을 넘긴다.

그리는 법:
  - 몸은 타원 몇 개를 합친 덩어리. 음영은 그 줄의 몸통 폭에서 어디쯤인가로 정한다(왼쪽 가장자리 밝음 h, 가운데 b,
    오른쪽 가장자리 그늘 d) → 원통처럼 읽힌다. 등 가운데에는 털 결 점(h)을 드문드문.
  - 앞발(삽: 분홍 손바닥 + 크림 발톱)·뒷발·꼬리·귀·꽃은 글자 격자를 손으로 찍어 얹는다. 도형으로 그리면 작은 부위가 뭉개진다(0070).
  - 외곽선(o)은 nail.py처럼 자동.
"""
import math
import sys

from PIL import Image

S = 32
NAIL = {
    'o': (43, 29, 42), 'd': (92, 62, 52), 'b': (128, 92, 74), 'h': (166, 128, 102), 'l': (206, 172, 142),
    'p': (242, 152, 164), 'q': (200, 104, 124), 'c': (248, 238, 216), 'C': (206, 190, 164),
    'e': (26, 16, 32), 'w': (255, 255, 255), 'r': (236, 132, 132),
    # 주황 목도리(0206 새 옷: 게임 스프라이트와 같은 색)
    'j': (238, 128, 60), 'k': (214, 92, 29), 'K': (174, 66, 19),
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


def back(ellipses, grain=(), hump=None, phase=0.0, belly=1.0):
    """등을 위에서 본 몸통: 줄마다 몸통 폭의 왼쪽 22%는 밝음(h), 오른쪽 26%는 그늘(d), 가운데 털(b).
    맨 아래 두 줄(머리 끝)과 맨 위 줄(엉덩이 끝)은 그늘로 돌려 둥글게. grain = 털 결 점 (x, y).
    phase(라디안)는 몸통 축으로 돈 각: 0 = 등이 우리 쪽. 원통이라 가로 자리 t → 표면 각 θ = asin(2t−1)이고,
    재질 각 θ+phase가 뒤(π) 근처면 배의 밝은 털(l, 폭 belly 라디안)이 보인다. 털 결 점도 같이 돈다."""
    ins = inside_of(ellipses)
    g = blank()
    ys = [y for y in range(S) if any(ins[y])]
    for y in ys:
        xs = [x for x in range(S) if ins[y][x]]
        x0, x1 = xs[0], xs[-1]
        w = x1 - x0 + 1
        for x in xs:
            t = (x - x0 + 0.5) / w
            theta = math.asin(max(-1.0, min(1.0, 2 * t - 1)))
            a = (theta + phase) % (2 * math.pi) - math.pi   # 배(재질 각 π) 가운데가 0
            if y >= ys[-1] - 1 or y == ys[0]:
                ch = 'd' if t > 0.3 else 'b'
            elif abs(a) < belly / 2 and ys[0] + 3 < y < ys[-1] - 2:
                ch = 'l'
            elif t < 0.22:
                ch = 'h'
            elif t > 0.74:
                ch = 'd'
            else:
                ch = 'b'
            g[y][x] = ch
    if phase:
        # 돌고 있으면 털 결 점을 재질 각에 따라 옮긴다(앞쪽에 있을 때만).
        moved = []
        for x, y in grain:
            if y < ys[0] or y > ys[-1]:
                continue
            xs = [xx for xx in range(S) if ins[y][xx]]
            if not xs:
                continue
            x0, x1 = xs[0], xs[-1]
            w = x1 - x0 + 1
            am = math.asin(max(-1.0, min(1.0, 2 * ((x - x0 + 0.5) / w) - 1)))
            sa = am - phase
            if math.cos(sa) > 0.1:
                moved.append((x0 + int((math.sin(sa) + 1) / 2 * w), y))
        grain = moved
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


def spin_x(ells, y, am, phase, width):
    """돌고 있는 몸통의 재질 각 am(±π/2 = 양옆)에 붙은 부위의 화면 x(너비 width 가운데 맞춤). 뒤로 돌아가 있으면 None."""
    ins = inside_of(ells)
    xs = [x for x in range(S) if ins[min(max(y, 0), S - 1)][x]]
    if not xs:
        return None
    sa = am - phase
    if math.cos(sa) < -0.15:
        return None
    x0, x1 = xs[0], xs[-1]
    cx = x0 + (math.sin(sa) + 1) / 2 * (x1 - x0 + 1)
    return int(round(cx - width / 2))


def attach_x(ells, y, width, side, overlap=1):
    """y줄에서 몸통 가장자리에 width 폭의 부위를 overlap만큼 겹쳐 붙일 x."""
    ins = inside_of(ells)
    xs = [x for x in range(S) if ins[min(max(y, 0), S - 1)][x]]
    if not xs:
        return 0 if side == 'L' else S - width
    return xs[0] - width + overlap if side == 'L' else xs[-1] + 1 - overlap


def scarf(g, y, tail=None):
    """목(y−1~y+1, 세 줄 — 게임 스프라이트처럼 몸의 5분의 1쯤)을 두른 주황 목도리(0206 새 옷). 몸통 털(bhdl)만 칠해 앞발은 위에 남는다. 몸통 폭의 왼쪽은 밝게,
    오른쪽은 그늘로(원통). tail = (x, 위 끝 y): 등 위로 늘어진 끝자락 — 뒷모습에서 목도리가 보이는 자리."""
    for yy in (y - 1, y, y + 1):
        xs = [x for x in range(S) if g[yy][x] in 'bhdl']
        if not xs:
            continue
        x0, x1 = xs[0], xs[-1]
        for x in xs:
            t = (x - x0 + 0.5) / (x1 - x0 + 1)
            g[yy][x] = 'j' if t < 0.22 else 'K' if t > 0.74 else 'k'
    if tail:
        put(g, tail[0], tail[1], SCARF_TAIL)


SCARF_TAIL = [".k", "jk", "jk", "kK", "kK"]


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
    scarf(g, 21 + dy, tail=(14, 15 + dy))
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


# 고개를 들어 우리를 보는 머리(아래쪽 몸 위에 얹는다): 큰 눈 둘 + 반사, 볼 홍조, 둥근 분홍 코.
LOOK_HEAD = [
    "...dbbbbbbd...",
    "..bbbbbbbbbb..",
    ".bbbbbbbbbbbb.",
    ".bbwebbbbbweb.",
    ".bbeebbbbbeeb.",
    ".brbbbbbbbbrb.",
    "..bbbbpppbbb..",
    "..dbbbpqpbbd..",
    "...dbbbqbbd...",
    "....dddddd....",
]
WINK = ["bb", "ee"]   # 오른눈 감기(가로선)


def nail_look(wink):
    g = nail_frame(0)
    # outline()을 거친 뒤라 외곽선 포함. 머리를 다시 얹고 외곽선을 다시 두른다.
    g = [[ch if ch != 'o' else '.' for ch in row] for row in g]
    scarf(g, 18)      # 고개를 들면 목은 머리 바로 위
    put(g, 9, 19, LOOK_HEAD)
    if wink:
        put(g, 19, 22, WINK)
    return outline(g)


def nail_spin(k):
    """8칸 중 k번째: 몸통 축으로 k/8 바퀴. 앞발은 양옆(±π/2)에 붙어 돌고, 뒷발·꼬리도 함께."""
    phase = k / 8 * 2 * math.pi
    g = back(NAIL_BODY, NAIL_GRAIN, hump=(12.5, 13.5, 4.2), phase=phase, belly=1.4)
    head_shade(g, 23)
    tx = spin_x(NAIL_BODY, 19, -0.3, phase, 2)      # 끝자락은 등 가운데 조금 왼쪽에 붙어 함께 돈다
    scarf(g, 21, tail=None if tx is None else (tx, 15))
    put(g, 15, 1, TAIL)
    for am, rows in ((-math.pi / 2, PAW_DOWN), (math.pi / 2, flip(PAW_DOWN))):
        x = spin_x(NAIL_BODY, 20, am, phase, len(rows[0]))
        if x is not None:
            put(g, x, 18, rows)
    for am, rows in ((-math.pi / 2, HIND_REST), (math.pi / 2, flip(HIND_REST))):
        x = spin_x(NAIL_BODY, 6, am, phase, len(rows[0]))
        if x is not None:
            put(g, x, 5, rows)
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


M_LOOK_HEAD = [
    "...dbbbbbbd...",
    "..bbbbbbbbbb..",
    ".bbbbbbbbbbbb.",
    ".bbWnbbbbbWnb.",
    ".bbnnbbbbbnnb.",
    ".brbbbbbbbbrb.",
    "..bbbbbppbbb..",
    "..dbbbbqqbbd..",
    "...dbbbbbbd...",
    "....dddddd....",
]
M_WINK = ["bb", "nn"]


def momo_look(wink):
    g = momo_frame(0)
    g = [[ch if ch != 'o' else '.' for ch in row] for row in g]
    put(g, 9, 19, M_LOOK_HEAD)
    if wink:
        put(g, 19, 22, M_WINK)
    return outline(g)


def momo_spin(k):
    phase = k / 8 * 2 * math.pi
    g = back(MOMO_BODY, MOMO_GRAIN, hump=(12.5, 13.0, 3.5), phase=phase, belly=1.4)
    head_shade(g, 23)
    put(g, 14, 1, COTTON)
    # 귀 둘은 등 위(재질 각 ±0.55)에 누워 함께 돈다. 뒤로 가면 안 보인다.
    for am, rows in ((-0.55, EAR), (0.55, flip(EAR))):
        x = spin_x(MOMO_BODY, 15, am, phase, len(rows[0]))
        if x is not None:
            put(g, x, 10, rows)
            put(g, x + 1, 19, FLOWER)
    for am, rows in ((-math.pi / 2, M_FRONT), (math.pi / 2, flip(M_FRONT))):
        x = spin_x(MOMO_BODY, 23, am, phase, 3)
        if x is not None:
            put(g, x, 22, rows)
    for am, rows in ((-math.pi / 2, M_HIND_REST), (math.pi / 2, flip(M_HIND_REST))):
        x = spin_x(MOMO_BODY, 6, am, phase, 2)
        if x is not None:
            put(g, x, 5, rows)
    return outline(g)


# 도토(0195 → 2026-10-29 다시): 위에서 본 청설모. 모모의 몸(둥근 등)에 청설모 털빛을 쓰되, 토끼 귀 대신
# 머리 옆에 끝에 술이 달린 작은 귀 둘, 솜꼬리 대신 엉덩이에서 등을 덮으며 올라가는 큰 꼬리(사장: "귀가 너무 크고 꼬리가 너무 작다").
DOTO = dict(MOMO, **{
    'd': (120, 54, 34), 'b': (172, 84, 46), 'h': (214, 126, 72), 'l': (238, 204, 164),
    'p': (226, 150, 120), 'q': (176, 96, 80), 'c': (226, 140, 84), 'C': (190, 104, 60),
    'T': (246, 196, 140), 'F': (104, 70, 44), 'Y': (204, 146, 76),
})
# 작은 귀(왼쪽): 끝에 짙은 술(F), 안쪽 분홍.
D_EAR = [".F.", "FbF", "bpb", "bpb", ".b."]
# 큰 꼬리: 엉덩이(위쪽 끝)에서 등 위로 넓게 부풀어 오른다. c 털, C 결, T 밝은 끝 털.
D_TAIL = [
    ".....TTTTT.....",
    "...TTcccccTT...",
    "..TccCCcCCccT..",
    ".TcCCCcCCCCcCT.",
    ".TcCCcCCCcCCcT.",
    "TcCCCCcCCCCCcCT",
    "TcCCcCCCCcCCcCT",
    "TccCCCCcCCCCccT",
    ".TcCCcCCCCcCcT.",
    ".TccCCCCcCCccT.",
    "..TccCCcCCccT..",
    "...TTcccccTT...",
    ".....TTTTT.....",
]

def doto_body(i):
    """모모 걸음과 같은 들썩임·발, 귀·꽃·솜꼬리는 없이."""
    dy = [0, 0, 1, 0, 1, 1][i]
    ell = [(cx, cy + dy * (k >= 2), rx, ry) for k, (cx, cy, rx, ry) in enumerate(MOMO_BODY)]
    g = back(ell, [(x, y + dy * (y > 14)) for x, y in MOMO_GRAIN], hump=(12.5, 13.0, 3.5))
    head_shade(g, 23 + dy)
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
    return g, ell, dy


def doto_ears(g, ell, dy):
    put(g, attach_x(ell, 24 + dy, 3, 'L', 2), 21 + dy, D_EAR)
    put(g, attach_x(ell, 24 + dy, 3, 'R', 2), 21 + dy, flip(D_EAR))


def doto_frame(i):
    g, ell, dy = doto_body(i)
    doto_ears(g, ell, dy)
    # 꼬리는 걸음마다 좌우로 1px 살랑.
    put(g, 8 + [0, 1, 0, -1, 0, 1][i], 0, D_TAIL)
    return outline(g)


def doto_look(wink):
    g, ell, dy = doto_body(0)
    put(g, 9, 19, M_LOOK_HEAD)
    if wink:
        put(g, 19, 22, M_WINK)
    put(g, 7, 17, D_EAR)
    put(g, 22, 17, flip(D_EAR))
    put(g, 8, 0, D_TAIL)
    return outline(g)


def doto_spin(k):
    phase = k / 8 * 2 * math.pi
    g = back(MOMO_BODY, MOMO_GRAIN, hump=(12.5, 13.0, 3.5), phase=phase, belly=1.4)
    head_shade(g, 23)
    for am, rows in ((-1.1, D_EAR), (1.1, flip(D_EAR))):
        x = spin_x(MOMO_BODY, 24, am, phase, len(rows[0]))
        if x is not None:
            put(g, x, 21, rows)
    for am, rows in ((-math.pi / 2, M_FRONT), (math.pi / 2, flip(M_FRONT))):
        x = spin_x(MOMO_BODY, 23, am, phase, 3)
        if x is not None:
            put(g, x, 22, rows)
    # 큰 꼬리는 등 가운데(재질 각 0)에 붙어 돈다 — 뒤로 가면 몸 너머로 끝만 보인다.
    x = spin_x(MOMO_BODY, 6, 0.0, phase, len(D_TAIL[0]))
    put(g, 8 if x is None else x, 0, D_TAIL if x is not None else D_TAIL[:4])
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
    img = Image.new('RGBA', (S * 16, S * 6), (0, 0, 0, 0))
    frames = [nail_frame(i) for i in range(6)] + [nail_look(False), nail_look(True)] + [nail_spin(k) for k in range(8)]
    for r, pal in enumerate(rows):
        for i, g in enumerate(frames):
            paint(img, i * S, r * S, g, pal)
    mframes = [momo_frame(i) for i in range(6)] + [momo_look(False), momo_look(True)] + [momo_spin(k) for k in range(8)]
    for i, g in enumerate(mframes):
        paint(img, i * S, 4 * S, g, MOMO)
    dframes = [doto_frame(i) for i in range(6)] + [doto_look(False), doto_look(True)] + [doto_spin(k) for k in range(8)]
    for i, g in enumerate(dframes):
        paint(img, i * S, 5 * S, g, DOTO)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/dig.png')
