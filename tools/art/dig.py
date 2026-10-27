"""파 내려가기 그림(0156): 땅 단면 전환에서 화면 가운데 크게 나오는 Naïl·모모, 32×32 × 6칸.
이 파일이 원본이고 assets/art/dig.png는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/dig.py [출력 경로]   (기본 assets/art/dig.png)
줄 0 Naïl 기본 털, 줄 1~3 털빛 1~3(nail.py FURS), 줄 4 돌아온 모모. 칸 0~5 = 한 번 긁는 동작(초당 12칸으로 돈다).

그리는 법:
  - 몸은 타원 몇 개를 합친 덩어리. 칸마다 테두리까지의 거리로 기울기를 어림해 왼쪽 위 빛으로 세 단(d·b·h)을 고르고,
    왼쪽 아래(배·주둥이 쪽)는 밝은 털(l). 외곽선(o)은 nail.py처럼 자동.
  - 얼굴(눈·볼·코)과 앞발(분홍 손바닥 + 크림 발톱 다섯)은 글자 격자를 손으로 찍어 얹는다. 도형으로 그리면 작은 부위가 뭉개진다(0070).
  - 머리를 아래로 한 옆모습: 위가 엉덩이·꼬리, 아래가 코, 얼굴은 오른쪽을 본다.
"""
import math
import sys

from PIL import Image

S = 32
NAIL = {
    'o': (43, 29, 42), 'd': (92, 62, 52), 'b': (128, 92, 74), 'h': (166, 128, 102), 'l': (206, 172, 142),
    'p': (242, 152, 164), 'q': (200, 104, 124), 'c': (248, 238, 216), 'C': (206, 190, 164),
    'e': (26, 16, 32), 'w': (255, 255, 255), 'r': (236, 132, 132),
}


def blank():
    return [['.'] * S for _ in range(S)]


def put(g, x, y, rows):
    """rows(문자열 목록)를 (x, y)부터 얹는다. '.'과 ' '는 건너뛴다."""
    for j, r in enumerate(rows):
        for i, ch in enumerate(r):
            if ch not in '. ' and 0 <= x + i < S and 0 <= y + j < S:
                g[y + j][x + i] = ch


def body(ellipses, light=(-0.6, -0.8), belly=None):
    """타원 (cx, cy, rx, ry) 합집합을 세 단 명암으로. belly(x, y) -> bool 이면 그 칸은 밝은 털."""
    g = blank()
    inside = [[any(((x + .5 - cx) / rx) ** 2 + ((y + .5 - cy) / ry) ** 2 <= 1 for cx, cy, rx, ry in ellipses)
               for x in range(S)] for y in range(S)]
    for y in range(S):
        for x in range(S):
            if not inside[y][x]:
                continue
            # 기울기: 가까운 테두리 방향(바깥으로 향하는 쪽)을 둘레 칸 수로 어림.
            nx = ny = 0.0
            for dy in range(-3, 4):
                for dx in range(-3, 4):
                    xx, yy = x + dx, y + dy
                    out = not (0 <= xx < S and 0 <= yy < S and inside[yy][xx])
                    if out:
                        w = 1.0 / (dx * dx + dy * dy)
                        nx += dx * w
                        ny += dy * w
            n = math.hypot(nx, ny)
            k = 0.0 if n == 0 else (nx * light[0] + ny * light[1]) / n
            edge = n > 0.35
            if belly and belly(x, y):
                ch = 'l'
            elif edge and k > 0.35:
                ch = 'h'
            elif edge and k < -0.25:
                ch = 'd'
            else:
                ch = 'b'
            g[y][x] = ch
    return g


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
# 엉덩이(위) → 어깨(가장 넓다) → 머리(아래로 좁아지는 주둥이).
NAIL_BODY = [(15.5, 9.0, 6.5, 6.5), (15.5, 15.0, 8.0, 6.5), (15.5, 21.0, 6.5, 5.0), (15.5, 25.0, 4.0, 3.5)]

# 얼굴: 오른쪽 아래(눈은 2×2 + 반사, 볼 홍조, 코는 둥근 분홍).
FACE = [
    "..we",
    "..ee",
    ".r..",
]
NOSE = [
    ".pp.",
    "pppq",
    ".qq.",
]
# 앞발(삽): 분홍 손바닥 + 크림 발톱 다섯. 가까운(왼) 발은 밝게, 먼(오른) 발은 그늘 색(q·C).
PAW_DOWN = [      # 아래로 내리긋는 손(발톱이 아래)
    ".bpp.",
    "bpppp",
    "ppppp",
    "pqppq",
    "c.c.c",
    "c.c.c",
]
PAW_SIDE = [      # 옆으로 밀어내는 손(발톱이 바깥 = 왼쪽, 오른쪽 끝이 팔로 몸에 붙는다)
    "..ppbb.",
    "ccpppbb",
    "..ppppb",
    "ccqpppb",
    "...qqb.",
]
PAW_UP = [        # 위로 당겨 올린 손(다음에 내리긋기 전, 발톱이 위)
    "c.c.c",
    "ppppp",
    "bpppq",
    ".bbb.",
]


def mirror(rows, far=True):
    """오른쪽 발: 좌우를 뒤집고, 먼 쪽이라 그늘 색으로."""
    m = {'p': 'q', 'c': 'C', 'b': 'd', 'h': 'b'} if far else {}
    return [''.join(m.get(ch, ch) for ch in reversed(r)) for r in rows]


def hind(g, left_up):
    """뒷발: 엉덩이 양옆에서 흙을 찬다(위로 찬 발은 발바닥이 보인다)."""
    if left_up:
        put(g, 7, 3, ["pp", "qp"])
        put(g, 22, 6, ["bq"])
    else:
        put(g, 8, 6, ["qb"])
        put(g, 23, 3, ["pp", "pq"])


def nail_frame(i):
    # 칸마다 몸을 조금 늘이고 줄인다(1·4칸은 밀며 한 줄 내려감).
    dy = [0, 0, 1, 0, 0, 1][i]
    ell = [(cx, cy + dy * (k >= 2), rx, ry) for k, (cx, cy, rx, ry) in enumerate(NAIL_BODY)]
    g = body(ell, belly=lambda x, y: x < 13 and y > 20 + dy and (x - 13) ** 2 + (y - 27) ** 2 < 40)
    put(g, 17, 21 + dy, FACE)
    put(g, 14, 27 + dy, NOSE)
    put(g, 14, 1, ["qq"])  # 짧은 꼬리
    L, R = (3, 15), (24, 15)
    poses = [
        ('up', 'down'), ('down', 'up'), ('side', 'up'),
        ('up', 'down'), ('up', 'side'), ('down', 'down'),
    ][i]
    for side, pose in zip(('L', 'R'), poses):
        far = side == 'R'
        base = {'down': PAW_DOWN, 'side': PAW_SIDE, 'up': PAW_UP}[pose]
        rows = mirror(base) if far else base
        if pose == 'down':
            x, y = (7, 22) if not far else (20, 22)
        elif pose == 'side':
            x, y = (2, 16) if not far else (23, 16)
        else:
            x, y = (5, 11) if not far else (22, 11)
        put(g, x, y + dy, rows)
    hind(g, i % 2 == 0)
    return outline(g)


# ---- 돌아온 모모(굴토끼) ----
MOMO = {
    'o': (43, 29, 42), 'd': (104, 84, 74), 'b': (150, 124, 106), 'h': (190, 164, 140), 'l': (228, 212, 192),
    'p': (240, 150, 164), 'q': (196, 104, 124), 'c': (248, 238, 218), 'C': (214, 200, 182),
    'n': (40, 26, 40), 'W': (255, 255, 255), 'r': (236, 132, 140), 'F': (255, 168, 196), 'Y': (255, 226, 120),
}
# 몸은 Naïl보다 조금 왼쪽(오른쪽에 눕힌 귀 자리), 엉덩이가 둥글고 머리가 작다.
MOMO_BODY = [(14.0, 9.5, 6.5, 6.0), (14.0, 15.5, 7.0, 6.5), (14.5, 21.5, 6.0, 5.0), (15.0, 25.5, 3.8, 3.0)]
# 귀 둘: 머리(아래)에서 등 쪽(위)으로 눕혔다. 몸에서 한 칸 떼어 외곽선이 사이에 들어간다.
EAR = [(24.5, 15.0, 2.6, 6.5), (24.2, 8.0, 1.6, 3.0)]
MOMO_FACE = [
    "..nW",
    "..nn",
    ".r..",
]
MOMO_NOSE = [
    ".p.",
    "pqp",
]
FRONT = [         # 작은 앞발(밝은 털 + 분홍 발바닥)
    ".ll",
    "lll",
    "pp.",
]
HIND_UP = [       # 옆으로 길게 찬 큰 뒷발(분홍 발바닥이 보인다)
    "lllb",
    "pppl",
]
HIND_DOWN = [     # 엉덩이 옆에 붙인 뒷발
    "bb",
    "ll",
]


def momo_frame(i):
    dy = [0, 0, 1, 0, 0, 1][i]
    ell = [(cx, cy + dy * (k >= 2), rx, ry) for k, (cx, cy, rx, ry) in enumerate(MOMO_BODY)]
    g = body(ell, belly=lambda x, y: x < 12 and y > 19 + dy and (x - 12) ** 2 + (y - 26) ** 2 < 36)
    ear = body([(cx, cy + dy, rx, ry) for cx, cy, rx, ry in EAR])
    for y in range(S):
        for x in range(S):
            if ear[y][x] != '.':
                # 귀 안쪽 한 줄은 분홍.
                inner = (x == 25 and 8 + dy <= y <= 19 + dy) or (x == 24 and 11 + dy <= y <= 17 + dy)
                g[y][x] = 'p' if inner else ear[y][x]
    put(g, 21, 20 + dy, ["FYF", ".F."])  # 귀 밑동의 꽃
    put(g, 15, 21 + dy, MOMO_FACE)
    put(g, 14, 27 + dy, MOMO_NOSE)
    put(g, 13, 2, ["cc", "ccc"] if i % 2 == 0 else ["ccc", ".cc"])  # 솜뭉치 꼬리
    # 앞발: 번갈아 턱 밑을 긁는다.
    lf = [(8, 25), (9, 27), (7, 23), (8, 25), (9, 27), (9, 26)][i]
    rf = [(19, 27), (18, 25), (19, 27), (17, 24), (18, 26), (17, 26)][i]
    put(g, lf[0], lf[1] + dy, FRONT)
    put(g, rf[0], rf[1] + dy, mirror(FRONT))
    # 뒷발: 번갈아 위로 크게 찬다.
    if i % 3 == 0:
        put(g, 4, 6, HIND_UP)
        put(g, 19, 9, mirror(HIND_DOWN))
    elif i % 3 == 1:
        put(g, 7, 9, HIND_DOWN)
        put(g, 20, 5, mirror(HIND_UP))
    else:
        put(g, 6, 7, HIND_DOWN)
        put(g, 20, 8, mirror(HIND_DOWN))
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
