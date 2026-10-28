"""(얼림, 0216: 지금 chief.png는 `tools/art/bori_ai.py`가 나노바나나 원본에서 만든다.)
들쥐 대장(튜토리얼 보스, docs/decisions/0032) 스프라이트 생성기 — 코드 그림 시절의 원본.

실행: python3 tools/art/chief.py [출력 경로]  (기본 assets/art/chief.png)
칸 32×32(졸개의 두 배 크기, 같은 픽셀 밀도), 오른쪽을 본다. 발바닥 y=30.
  줄 0 물든 대장: 0·1 서 있기, 2·3 걷기, 4 예고(웅크림), 5 공격(돌진·뛰기), 6 맞음, 7 울부짖기(부하 부르기)
  줄 1 돌아온 보리(쉼터의 이웃): 0·1 서 있기, 2·3 걷기, 4 기뻐하기
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍임)

critters.py의 문법을 그대로 쓴다(채움 글자 + 자동 외곽선, 3단 털, 물든 표시 = 붉은 눈 + 보라 뿌리).
대장다움은 크기와 실루엣으로: 들쥐의 둥근 귀 하나 + 한쪽이 찢긴 귀, 눈 위 흉터, 굵은 꼬리, 머리에 뿌리 왕관.
돌아온 보리는 같은 몸에서 뿌리를 걷어 내고(털로 덮음) 눈을 검게, 흉터는 남긴다(지나온 일).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import critters as c  # noqa: E402

N = 32
c.N = N   # critters의 도우미(shift·outline·paint …)는 모듈 전역 N을 본다. 이 생성기는 32칸으로 쓴다.

# 들쥐 털(critters FIELD)보다 한 단 짙고 붉은 갈색: 늙은 대장.
FUR = [(92, 62, 50), (134, 94, 70), (174, 134, 98), (220, 192, 156)]

BASE = [
    "................................",
    "................................",
    "...........V.....V..............",
    "...........vV...Vv..............",
    "............v...v...............",
    "............vVVVv....dd.........",
    "..........dd.vvv....dqqd........",
    ".........dqqd.v....dqppqd.......",
    ".........dqpd.v...dqpppqd.......",
    "..........dd.hhhhhdqpppqd.......",
    "...........hhhhhhhhdqppd........",
    "..........hhbbbbbbbbddd.........",
    ".........hhbbvVbbbbbbbbh........",
    "........hhbbvvbbbbbbbbbbh.......",
    ".......hhbbvvbbbbbbbbkbbbb......",
    ".......hbbvvbbbbbbbbbbkwebbb....",
    "......hbbvvbbbbbbbbbbbkeebbbl...",
    "......hbbvbbbbbbbbbbbbbbbbblll..",
    ".....hbbbvbbbbbbbbbbbbbbbllllln.",
    ".....hbbbbbbbbbbbbbbbbbbbllllpn.",
    ".....hbbbbbbbbbbbbbbbbbbbblcl...",
    ".....bbbbbbbbbbbbbbbbbbbbll.....",
    "pp...bbbbbbbbbbbbbbbbbblllll....",
    ".pp..dbbbbbbbbbbbbbbbblllllll...",
    "..pp.dbbbbbbbbbbbbbbbbllllllb...",
    "...ppdbbbbbbbbbbbbbbbbllllllb...",
    "....pddbbbbbbbbbbbbbbbbllllbb...",
    ".....pddbbbbbbbbbbbbbbbbbbbd....",
    "......pddddbbbbbbbbbbbbbbdd.....",
    "........dddddddddddddddddd......",
    ".........ppp..ppp...ppp..ppp....",
    "................................",
]


def grid(rows):
    rows = [r.ljust(N, '.')[:N] for r in rows]
    assert len(rows) == N, len(rows)
    return [list(r) for r in rows]


def details(g):
    """큰 칸이라 눈·흉터·뿌리는 따로 찍는다(밑그림 위에 덧칠)."""
    # 눈: 3×3 붉은 빛 + 왼쪽 위 반사. 졸개(2×2)보다 크게.
    for x, y in c.find(g, 'kwe'):
        g[y][x] = 'b'
    c.put(g, 22, 14, 'wee')
    c.put(g, 22, 15, 'eee')
    c.put(g, 22, 16, 'eee')
    # 흉터: 눈 위아래로 지나가는 밝은 줄(어두운 털 위에서 읽히게 크림색).
    c.put(g, 20, 12, 'c')
    c.put(g, 21, 13, 'c')
    c.put(g, 25, 17, 'c')
    # 등의 뿌리: 왕관에서 내려와 두 갈래로 등을 감는다. 마디는 밝게.
    c.put(g, 13, 13, 'V')
    c.put(g, 10, 16, 'V')
    c.put(g, 16, 11, 'v')
    c.put(g, 17, 12, 'vV')
    c.put(g, 18, 13, 'v')
    c.put(g, 19, 14, 'v')
    c.put(g, 8, 19, 'v')
    c.put(g, 7, 20, 'vV')
    c.put(g, 6, 21, 'v')
    return g


def cleanse(g):
    """돌아온 보리: 뿌리를 털로, 붉은 눈을 검은 눈으로(반사는 흰색), 머리 위 뿌리 왕관은 지운다."""
    f = c.copy(g)
    top = min(y for y in range(N) for x in range(N) if f[y][x] in 'hb')
    for y in range(N):
        for x in range(N):
            ch = f[y][x]
            if ch in 'vV':
                f[y][x] = '.' if y < top else 'b'
            elif ch == 'e':
                f[y][x] = 'n'
            elif ch == 'w':
                f[y][x] = 'W'
    return f


def howl(g):
    """울부짖기: 몸을 한 칸 세우고(발은 땅에) 입을 크게 벌린다, 뿌리가 밝게."""
    f = c.shift(g, dy=-1, rows=range(0, 30))
    c.put(f, 25, 19, 'nnn')
    c.put(f, 25, 20, 'nqq')
    c.put(f, 26, 21, 'c')
    for x, y in c.find(f, 'v'):
        f[y][x] = 'V'
    return f


def happy(g):
    """기뻐하기: 눈을 ^로 감고 볼에 홍조."""
    f = c.copy(g)
    for x, y in c.find(f, 'nW'):
        if y < 18:
            f[y][x] = 'b'
    c.put(f, 22, 15, 'n')
    c.put(f, 23, 14, 'n')
    c.put(f, 24, 15, 'n')
    c.put(f, 21, 18, 'rr')
    return f


def walk(g, i):
    f = c.copy(g)
    if i == 1:
        f = c.shift(f, dy=-1, rows=range(0, 30))
        for x in range(N):
            f[30][x] = '.'
        c.put(f, 10, 30, 'ppp')
        c.put(f, 15, 30, 'ppp')
        c.put(f, 22, 30, 'ppp')
        c.put(f, 27, 30, 'ppp')
    return f


def breathe(g):
    top = min(y for y in range(N) for x in range(N) if g[y][x] in 'hb')
    return c.shift(g, dy=1, rows=range(0, top + 1))


def crouch(g):
    f = c.shift(g, dy=1, rows=range(0, 30))
    for x, y in c.find(f, 'v'):
        f[y][x] = 'V'
    return f


def lunge(g):
    return c.shift(g, dx=2, rows=range(0, 30))


def hurt(g):
    f = c.shift(g, dx=-1)
    c.squint(f)
    return f


def sheet():
    base = details(grid(BASE))
    bori = cleanse(base)
    rows = [
        ([base, breathe(base), walk(base, 0), walk(base, 1), crouch(base), lunge(base), hurt(base), howl(base)], True),
        ([bori, breathe(bori), walk(bori, 0), walk(bori, 1), happy(bori)], False),
    ]
    from PIL import Image
    img = Image.new('RGBA', (N * 8, N * len(rows) * 2), (0, 0, 0, 0))
    for ry, (frames, tainted) in enumerate(rows):
        for rx, g in enumerate(frames):
            g = c.outline(g)
            c.paint(img, rx * N, ry * N, g, FUR, tainted=tainted)
            c.paint(img, rx * N, (ry + len(rows)) * N, g, FUR, silhouette=True)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/chief.png')
