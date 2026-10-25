"""Naïl(아기 두더지) 스프라이트 시트 생성기. 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/nail.py [출력 경로]  (기본 assets/art/nail.png)
칸 16×16, 오른쪽을 본다(왼쪽은 게임에서 좌우 반전). 발바닥 y=14, 그림자는 게임이 그린다.
  줄 0: idle 2, walk 4
  줄 1: attack 3(1타·2타·마지막 타), hurt, dead
  줄 2: 흙더미 2(굴파기), 그림자
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


def attack(a, i):
    # 발톱은 늘 발 높이(12줄 이하)에서. 코 바로 밑에 크림색을 두면 다시 이빨로 읽힌다.
    # 머리는 옮기지 않는다(코가 칸 끝에 닿아 있다). 앞으로 나가는 맛은 게임의 전진(lunge)이 낸다.
    f = list(a)
    if i == 0:  # 앞발 쭉
        put(f, 11, 12, 'bpccc')
    elif i == 1:  # 낮게 긁기
        put(f, 11, 12, 'bb...')
        put(f, 11, 13, 'bpccc')
    else:  # 마지막 타: 눈 질끈(가로선), 두 발 다
        put(f, 9, 7, 'bb')
        put(f, 9, 8, 'ee')
        put(f, 10, 12, 'bbpccc')
        put(f, 11, 13, 'bpccc')
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


def paint(img, ox, oy, rows, silhouette=False):
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == '.':
                continue
            c = P[ch]
            if silhouette and ch not in FILL_NO_OUTLINE:
                c = (255, 255, 255)
            if len(c) == 3:
                c = c + (255,)
            img.putpixel((ox + x, oy + y), c)


def sheet():
    a = grid(BASE)
    rows = [
        [a, idle_b(a), walk(a, 0), walk(a, 1), walk(a, 2), walk(a, 3)],
        [attack(a, 0), attack(a, 1), attack(a, 2), hurt(a), dead(a)],
        [grid(MOUND_A), grid(MOUND_B), grid(SHADOW)],
    ]
    img = Image.new('RGBA', (16 * 6, 16 * 6), (0, 0, 0, 0))
    for ry, row in enumerate(rows):
        for rx, f in enumerate(row):
            f = outline(f)
            paint(img, rx * 16, ry * 16, f)
            paint(img, rx * 16, (ry + 3) * 16, f, silhouette=True)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/nail.png')
