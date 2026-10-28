"""Naïl(아기 두더지) 스프라이트 시트 생성기. 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/nail.py [출력 경로]  (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)
칸 16×16, 오른쪽을 본다(왼쪽은 게임에서 좌우 반전). 발바닥 y=14, 그림자는 게임이 그린다.
  줄 0: idle 2, walk 4
  줄 1: 치기 3(앞발·낮게 긁기·두 발), hurt, dead, 들기 3(앞발·낮은 발·두 발), 마지막 타 거두기
        (한 타 = 들기 → 치기 → 거두기, docs/decisions/0082)
  줄 2: 흙더미 2(굴파기), 그림자
  줄 3~5: 줄 0~2의 흰 실루엣(피격 플래시용)
그리는 법: 채움 색만 문자로 적고 외곽선('o')은 자동으로 두른다. 안쪽 선이 필요한 곳만 'o'를 직접 적는다.
새 옷(v2, 2026-10-28, 0203 C3): 초상화·나노바나나 시트(tools/art/sources/ai/sprites/nail_idle_walk_v1.png)의 생김새 —
둥근 털뭉치 + 주황 뜨개 목도리와 매듭 + 분홍 앞발·크림 발톱 — 를 16칸에 손으로 찍었다(시트를 줄이면 얼굴이 사라진다).
옛 길쭉한 옆모습은 tools/art/legacy/nail_v1.py.
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
    'f': (208, 102, 46),    # 목도리
    'g': (140, 54, 36),     # 목도리 그림자·매듭
    'e': (26, 16, 32),      # 눈
    'w': (255, 255, 255),   # 눈 하이라이트
    'r': (236, 132, 132),   # 볼 홍조
    'm': (118, 84, 58),     # 흙
    'n': (156, 114, 80),    # 흙 밝은 면
    'k': (84, 58, 42),      # 흙 어두운 면
    's': (24, 16, 28, 90),  # 그림자(반투명)
}
FILL_NO_OUTLINE = {'s'}

# 털빛(저널 보상, docs/decisions/0048): 털 네 단계(d·b·h·l)만 바꾼다. 코·눈·발톱·흙·목도리는 그대로.
# 물든 적보다 밝게(값 위계, critters.py 머리말) — 밤빛도 물든 들쥐보다 밝은 쪽으로 맞췄다.
FURS = {
    1: {'d': (72, 70, 78), 'b': (108, 106, 116), 'h': (146, 144, 154), 'l': (198, 194, 198)},   # 잿빛
    2: {'d': (70, 44, 38), 'b': (104, 66, 54), 'h': (140, 96, 76), 'l': (190, 152, 124)},     # 밤빛
    3: {'d': (150, 112, 70), 'b': (190, 150, 98), 'h': (220, 186, 132), 'l': (242, 222, 184)},  # 모래빛
}

# 채움만. 외곽선 자동. 오른쪽을 본다(3/4: 먼 눈이 보인다).
# 자리: 털 끝 1줄, 몸 2~12줄, 발 13줄(5~6·9~10열). 먼 눈 7열 4~5줄, 가까운 눈 11~12열(빛은 11,4), 볼 10,6, 코 13~14열 5~6줄.
# 목도리 8~9줄(매듭 10~11줄 7~9열). 뒷발(보는 쪽) 1~2열·앞발 13~14열, 9~11줄, 발톱 11줄. 입은 그리지 않는다.
BASE = """
................
.....hl..hh.....
....hllhhhhhh...
...hlhhhhhhbbb..
...hhhhebhbweb..
..hhhhhebbbeepp.
..hhhhbbbbrbbpq.
..hbbbbbbbbbbb..
..gffffffffffg..
.ppgffffffffgpp.
.qpoddddggddopq.
.ccodddggdddocc.
.....dddddd.....
.....pp..pp.....
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
FRONT = [(9, 13, '..'), (10, 13, '..'), (11, 13, '..')]   # 앞발 자리(줄, 열, 지우기)


def clear_front(f):
    for y, x, s in FRONT:
        put(f, x, y, s)
    put(f, 12, 10, 'd')
    put(f, 12, 11, 'd')
    put(f, 12, 9, 'f')


def idle_b(a):
    # 숨쉬기: 얼굴 밑 털 한 줄을 빼서 몸이 납작해진다(발은 그대로).
    return [BLANK] + a[0:7] + a[8:]


def walk(a, i):
    """0·2 = 디딤(발 둘이 땅), 1·3 = 통통 튀며 한 발을 앞으로 / 뒤로."""
    f = list(a)
    if i == 1:
        f = shift(f, dy=-1)
        put(f, 9, 12, '.pp')          # 앞발이 앞으로
    elif i == 2:
        put(f, 5, 13, '.pp..pp')      # 두 발이 한 칸 앞으로 모인다
    elif i == 3:
        f = shift(f, dy=-1)
        put(f, 4, 12, 'pp.')          # 뒷발 차기
    return f


def bob(a, dy):
    """윗몸(1~11줄)만 dy줄 옮긴다(0082). 발 줄(12~13)은 땅에 그대로 → 상체가 들썩인다."""
    f = list(a)
    if dy < 0:
        for y in range(0, 11):
            f[y] = a[y + 1]
    elif dy > 0:
        for y in range(12, 1, -1):
            f[y] = a[y - 1]
        f[1] = BLANK
    return f


def eyes_shut(f, dy=0):
    # 눈 질끈(가로선). 몸을 옮긴 만큼 눈 줄도 옮긴다.
    put(f, 11, 4 + dy, 'bb')
    put(f, 11, 5 + dy, 'ee')
    put(f, 7, 4 + dy, 'b')


def attack(a, i):
    """치기(0082): 0 = 앞발을 가운데 높이로 쭉, 1 = 앞발 낮게 긁기(그늘 색), 2 = 두 발 + 내리찍기.
    발톱은 발 높이에서. 코 바로 밑에 크림색을 두면 이빨로 읽힌다(v1) — 앞발은 목도리 줄 밑으로만."""
    f = list(a)
    clear_front(f)
    if i == 0:
        put(f, 13, 9, 'ppc')
        put(f, 13, 10, 'qcc')
    elif i == 1:
        put(f, 12, 11, 'qqcc')
        put(f, 12, 12, '.qc')
    else:
        f = bob(f, 1)
        eyes_shut(f, 1)
        put(f, 12, 11, 'ppcc')
        put(f, 12, 12, 'qqcc')
    return f


def windup(a, i):
    """들기(0082): 윗몸을 한 줄 들어 올리고 칠 발을 뒤로·위로. 들었다가 내려오는 한 줄이 "들썩"이다."""
    f = list(a)
    if i in (0, 2):
        clear_front(f)
    f = bob(f, -1)
    if i == 0:      # 앞발을 어깨 뒤로 젖힌다(목도리 위, 코와 떨어져)
        put(f, 12, 8, 'pc')
        put(f, 12, 9, 'q')
    elif i == 1:    # 앞발을 땅에서 뗀다
        put(f, 13, 11, '...')
        put(f, 13, 10, 'cc')
    else:           # 두 발 다 든다
        put(f, 12, 8, 'pc')
        put(f, 0, 6, 'cp')
        put(f, 1, 7, 'p')
    return f


def settle(a):
    """마지막 타 거두기: 몸은 제자리, 두 발은 아직 앞에(후딜레이 동안 자세를 지킨다)."""
    f = list(a)
    clear_front(f)
    put(f, 12, 11, 'ppcc')
    put(f, 12, 12, 'qqc')
    return f


def hurt(a):
    f = shift(a, dx=-1)
    put(f, 10, 3, 'eb')  # > 모양 눈
    put(f, 10, 4, 'be')
    put(f, 10, 5, 'eb')
    put(f, 6, 4, 'b')
    return f


def dead(a):
    f = shift(list(reversed(a)), dy=-2)  # 뒤집혀 누움
    # 눈(4~5줄 → 거꾸로 8~9줄)을 × 눈으로. 검은 2×2로 두면 벌린 입처럼 보인다.
    put(f, 10, 7, 'ebe')
    put(f, 10, 8, 'beb')
    put(f, 10, 9, 'ebe')
    put(f, 7, 8, 'b')
    put(f, 7, 9, 'b')
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
        [grid(MOUND_A), grid(MOUND_B), grid(SHADOW)],
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
