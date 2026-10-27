"""Naïl(아기 두더지) 스프라이트 시트 생성기. 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/nail.py [출력 경로]  (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)
칸 16×16, 오른쪽을 본다(왼쪽은 게임에서 좌우 반전). 발바닥 y=14, 그림자는 게임이 그린다.
  줄 0: idle 2, walk 4
  줄 1: 치기 3(앞발·뒷발·두 발), hurt, dead, 들기 3(앞발·뒷발·두 발), 마지막 타 거두기
        (한 타 = 들기 → 치기 → 거두기, docs/decisions/0082. 1·2타는 가까운 발·먼 발을 번갈아, 마지막 타는 두 발)
  줄 2: 흙더미 2(굴파기), 그림자, 3~6열 파 내려가기 4칸(땅 단면 전환, 0156)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 플래시용)
그리는 법: 채움 색만 문자로 적고 외곽선('o')은 자동으로 두른다. 그래서 부위를 옮겨도 외곽선이 깨지지 않는다.
"""
import sys
from PIL import Image

# ---- 팔레트: 따뜻한 갈색 털, 분홍 코·손바닥, 크림색 발톱 ----
P = {
    'o': (43, 29, 42),      # 외곽선(짙은 자두색)
    'd': (92, 62, 52),      # 털 그림자
    'b': (128, 92, 74),     # 털
    'h': (166, 128, 102),   # 털 하이라이트
    'l': (206, 172, 142),   # 주둥이·배(밝은 털)
    'p': (242, 152, 164),   # 코·손바닥
    'q': (200, 104, 124),   # 분홍 그림자
    'c': (248, 238, 216),   # 발톱
    'C': (206, 190, 164),   # 발톱 그림자
    'e': (26, 16, 32),      # 눈
    'w': (255, 255, 255),   # 눈 하이라이트
    'r': (236, 132, 132),   # 볼 홍조
    'm': (118, 84, 58),     # 흙
    'n': (156, 114, 80),    # 흙 밝은 면
    'k': (84, 58, 42),      # 흙 어두운 면
    's': (24, 16, 28, 90),  # 그림자(반투명)
}
FILL_NO_OUTLINE = {'s'}

# 털빛(저널 보상, docs/decisions/0048): 털 네 단계(d·b·h·l)만 바꾼다. 코·눈·발톱·흙은 그대로.
# 물든 적보다 밝게(값 위계, critters.py 머리말) — 밤빛도 물든 들쥐보다 밝은 쪽으로 맞췄다.
FURS = {
    1: {'d': (72, 70, 78), 'b': (108, 106, 116), 'h': (146, 144, 154), 'l': (198, 194, 198)},   # 잿빛
    2: {'d': (70, 44, 38), 'b': (104, 66, 54), 'h': (140, 96, 76), 'l': (190, 152, 124)},     # 밤빛
    3: {'d': (150, 112, 70), 'b': (190, 150, 98), 'h': (220, 186, 132), 'l': (242, 222, 184)},  # 모래빛
}

# 채움만. 외곽선 자동. 오른쪽을 본다.
# 얼굴(v2, 2026-10-25): 동그란 분홍 코 하나 + 2×2 눈(하이라이트) + 볼 홍조. 입은 그리지 않는다.
# v1은 주둥이 위아래로 크림색 발톱이 붙어 "벌린 입과 이빨"로 읽혔다(징그럽다는 피드백). 서 있을 때 발톱은 발 줄(13)에만,
# 코 밑(14~15열, 11~13줄)은 비워 턱선을 남긴다. 공격 프레임만 앞발이 앞으로 나온다.
BASE = """
................
................
................
.....hhhhh......
...hhhhbbbbb....
..hhbbbbbbbbb...
..hbbbbbbbbbbb..
.hbbbbbbbwebb...
.bbbbbbbbeebbl..
.dbbbbbbbrbbllpp
.ddbbbbbbbbbllpq
.dddbbbbbbbbbb..
..dddbbbbbbbbb..
...pdddddddpcc..
................
................
"""

MOUND_A = """
................
................
................
................
................
................
................
................
................
................
......nn........
....mnnnnmk.....
..kmmnnnmmmmk...
.kmmmmmmmmmmmk..
................
................
"""

MOUND_B = """
................
................
................
................
................
................
................
................
................
..n........n....
.......n........
.....mnnnmk.....
..kmmnnnnmmmk...
.kmmmmmmmmmmmk..
................
................
"""

# 파 내려가기(0156): 머리를 아래로 한 옆모습(땅 단면 전환에서 쓴다). 위가 엉덩이·꼬리, 아래가 코.
# 빛은 그대로 왼쪽 위라 왼쪽 가장자리가 밝고(h) 오른쪽이 어둡다(d). 배(밝은 털)는 왼쪽 아래, 눈은 오른쪽 아래.
# 동작은 `dig`가 앞발(삽 같은 손바닥 + 크림 발톱)을 번갈아 내리긋고, 뒷발이 위에서 흙을 찬다.
DIG = """
................
......qq........
.....hbbb.......
....hbbbbb......
....hbbbbbd.....
...hbbbbbbbd....
...hbbbbbbbd....
..hbbbbbbbbbd...
..hbbbbbbbbbd...
..hbbbbbbbbdd...
...hbbbbbwed....
...lbbbbbeed....
....llbbbrd.....
.....llbbd......
......lll.......
.......pp.......
"""

SHADOW = """
................
................
................
................
................
................
................
................
................
................
................
................
................
................
...ssssssssss...
..ssssssssssss..
"""

BLANK = '.' * 16


def grid(s):
    rows = [r[:16].ljust(16, '.') for r in s.strip('\n').split('\n')]
    assert len(rows) == 16, len(rows)
    return rows


def put(rows, x, y, s):
    r = list(rows[y])
    for i, ch in enumerate(s):
        if 0 <= x + i < 16 and ch != ' ':
            r[x + i] = ch
    rows[y] = ''.join(r)


def shift(rows, dx=0, dy=0, y_from=0, y_to=16):
    """y_from..y_to 줄만 dx만큼 옮기고, 전체를 dy만큼 옮긴다."""
    out = list(rows)
    for y in range(y_from, y_to):
        r = out[y]
        out[y] = ('.' * dx + r)[:16] if dx >= 0 else (r[-dx:] + '.' * -dx)
    if dy < 0:
        out = out[-dy:] + [BLANK] * -dy
    elif dy > 0:
        out = [BLANK] * dy + out[:-dy]
    return out


def outline(rows):
    """채운 칸에 4방향으로 닿은 빈 칸을 외곽선으로."""
    g = [list(r) for r in rows]
    out = [list(r) for r in rows]
    for y in range(16):
        for x in range(16):
            if g[y][x] != '.':
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < 16 and 0 <= ny < 16 and g[ny][nx] not in '.' and g[ny][nx] not in FILL_NO_OUTLINE:
                    out[y][x] = 'o'
                    break
    return [''.join(r) for r in out]


# ---- 동작 ----
def idle_b(a):
    # 숨쉬기: 등 한 줄을 빼서 납작하게.
    return [BLANK] + a[0:5] + a[6:]


def walk(a, i):
    f = list(a)
    if i in (1, 3):
        f = shift(f, dy=-1)  # 통통 튀기
    if i == 1:  # 앞발 내밀기
        f = shift(f, dx=1, y_from=13, y_to=14)
    if i == 3:  # 뒷발 차기
        put(f, 1, 12, 'pp')
    return f


def bob(a, dy):
    """윗몸(3~11줄)만 dy줄 옮긴다(0082). 발 줄(12~13)은 땅에 그대로 → 상체가 들썩인다.
    dy=-1: 들기(몸이 한 줄 늘어난다, 11줄을 한 번 더). dy=+1: 내리찍기(몸이 한 줄 눌린다, 옛 12줄이 덮인다)."""
    f = list(a)
    if dy < 0:
        for y in range(2, 11):
            f[y] = a[y + 1]
    elif dy > 0:
        for y in range(12, 3, -1):
            f[y] = a[y - 1]
        f[3] = BLANK
    return f


def eyes_shut(f, dy=0):
    # 눈 질끈(가로선). 몸을 옮긴 만큼 눈 줄도 옮긴다.
    put(f, 9, 7 + dy, 'bb')
    put(f, 9, 8 + dy, 'ee')


def attack(a, i):
    """치기(0082): 0 = 가까운 앞발(밝게, 가운데 높이로 쭉), 1 = 먼 앞발(그늘 색, 낮게 긁기), 2 = 두 발 + 내리찍기.
    발톱은 늘 발 높이(12줄 이하)에서. 코 바로 밑(14~15열, 코 다음 줄)에 크림색을 두면 다시 이빨로 읽힌다.
    머리를 앞으로 옮기지 않는다(코가 칸 끝에 닿아 있다). 앞으로 나가는 맛은 게임의 전진(lunge)이 낸다."""
    f = list(a)
    if i == 0:  # 가까운 발 쭉, 먼 발은 땅에
        put(f, 11, 12, 'bpccc')
    elif i == 1:  # 먼 발 낮게 긁기(그늘 색), 가까운 발은 뒤로 당겨 땅에
        put(f, 11, 12, 'bb...')
        put(f, 9, 13, 'dpdd')
        put(f, 12, 13, 'qCCC')
    else:  # 마지막 타: 윗몸을 한 줄 내리찍고 두 발 다(가까운 발은 땅을 따라 멀리, 먼 발은 몸 가장자리)
        f = bob(f, 1)
        eyes_shut(f, 1)
        put(f, 11, 12, 'qCC')
        put(f, 11, 13, 'bpccc')
    return f


def windup(a, i):
    """들기(0082): 윗몸을 한 줄 들어 올리고 칠 발을 땅에서 뗀다. 짧다(1·2타 약 1프레임, 마지막 타 3프레임).
    들었다가 내려오는 한 줄이 "들썩"이다. 뗀 발은 발 높이(12줄)에 둔다 — 몸 가운데(눈 밑)에 분홍·크림을 두면
    입으로 읽힌다(2026-10-26 확대 확인)."""
    f = bob(a, -1)
    if i == 0:  # 가까운 발을 들어 뒤로 젖힌다
        put(f, 11, 13, 'd...')
        put(f, 10, 12, 'pcc')
    elif i == 1:  # 먼 발(그늘 색)을 든다. 가까운 발은 땅에
        put(f, 9, 12, 'qCC')
    else:  # 두 발 다 든다
        put(f, 11, 13, 'd...')
        put(f, 8, 12, 'qCpcc')
    return f


def settle(a):
    """마지막 타 거두기: 몸은 제자리, 두 발은 아직 앞에(후딜레이 동안 자세를 지킨다)."""
    f = list(a)
    put(f, 11, 12, 'bqC')
    put(f, 11, 13, 'bpcc')
    return f


def hurt(a):
    f = shift(a, dx=-1)
    put(f, 8, 7, 'eb')  # > 모양 눈
    put(f, 8, 8, 'be')
    put(f, 8, 9, 'eb')
    return f


def dead(a):
    f = shift(list(reversed(a)), dy=-2)  # 뒤집혀 누움
    # 눈(원래 7~8줄 → 5~6줄)을 × 눈으로. 검은 2×2로 두면 벌린 입처럼 보인다.
    put(f, 9, 4, 'ebe')
    put(f, 9, 5, 'beb')
    put(f, 9, 6, 'ebe')
    return f


def dig(a, i):
    """파 내려가기 네 칸(0156). 두더지 앞발은 몸 옆으로 튀어나온 넓은 삽(분홍 손바닥 + 크림 발톱)이다.
    0 가까운(왼) 앞발로 코 옆을 내리긋고 먼 발은 위로 당김, 1 두 앞발을 코 앞에 모아 파기,
    2 먼(오른, 그늘 색) 앞발로 내리긋기, 3 두 앞발을 옆으로 크게 벌려 흙을 밀어냄. 뒷발은 위에서 번갈아 찬다."""
    f = list(a)
    if i == 0:
        put(f, 1, 11, 'ppb')
        put(f, 0, 12, 'ppc')
        put(f, 0, 13, 'cc')
        put(f, 12, 6, 'qq')
        put(f, 13, 7, 'qC')
        put(f, 3, 3, 'p')
    elif i == 1:
        put(f, 2, 12, 'pp')
        put(f, 3, 13, 'pc')
        put(f, 4, 14, 'c')
        put(f, 10, 12, 'qq')
        put(f, 9, 13, 'qC')
        put(f, 9, 14, 'C')
        put(f, 10, 3, 'p')
    elif i == 2:
        put(f, 12, 10, 'dqq')
        put(f, 12, 11, 'qqC')
        put(f, 13, 12, 'CC')
        put(f, 1, 6, 'pp')
        put(f, 1, 7, 'cp')
        put(f, 10, 3, 'p')
    else:
        put(f, 0, 9, 'c')
        put(f, 0, 10, 'cpp')
        put(f, 1, 11, 'pp')
        put(f, 13, 9, 'qC')
        put(f, 13, 10, 'qqC')
        put(f, 3, 3, 'p')
        put(f, 10, 3, 'p')
    return f


def paint(img, ox, oy, rows, silhouette=False, pal=P):
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == '.':
                continue
            c = pal[ch]
            if silhouette and ch not in FILL_NO_OUTLINE:
                c = (255, 255, 255)
            if len(c) == 3:
                c = c + (255,)
            img.putpixel((ox + x, oy + y), c)


def sheet(fur=0):
    pal = dict(P, **FURS.get(fur, {}))
    a = grid(BASE)
    rows = [
        [a, idle_b(a), walk(a, 0), walk(a, 1), walk(a, 2), walk(a, 3)],
        [attack(a, 0), attack(a, 1), attack(a, 2), hurt(a), dead(a),
         windup(a, 0), windup(a, 1), windup(a, 2), settle(a)],
        # 줄 2: 흙더미 2·그림자, 3~6열 = 파 내려가기 4칸(0156)
        [grid(MOUND_A), grid(MOUND_B), grid(SHADOW)] + [dig(grid(DIG), i) for i in range(4)],
    ]
    img = Image.new('RGBA', (16 * max(len(r) for r in rows), 16 * 6), (0, 0, 0, 0))
    for ry, row in enumerate(rows):
        for rx, f in enumerate(row):
            f = outline(f)
            paint(img, rx * 16, ry * 16, f, pal=pal)
            paint(img, rx * 16, (ry + 3) * 16, f, silhouette=True, pal=pal)
    return img


if __name__ == '__main__':
    import os
    out = sys.argv[1] if len(sys.argv) > 1 else 'assets/art/nail.png'
    sheet().save(out)
    for n in FURS:
        sheet(n).save(os.path.join(os.path.dirname(out), f'nail_fur{n}.png'))
