"""적·이웃 스프라이트 시트 생성기 v2(docs/decisions/0021). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/critters.py [출력 경로]  (기본 assets/art/critters.png)
칸 16×16, 오른쪽을 본다(왼쪽은 게임에서 좌우 반전). 발바닥 y=14, 그림자는 게임이 그린다.
  줄 0 들쥐 · 1 생쥐 · 2 시궁쥐 · 3 다람쥐 · 4 청설모 · 5 허수아비 (species.rs의 sheet_row)
    칸 0·1 서 있기(숨쉬기), 2·3 걷기, 4 예고(떨기·웅크림·조준), 5 공격(돌진·뛰기·던지기), 6 맞음
  줄 6 라쿤 포포(튜토리얼 안내): 0·1 서 있기, 2·3 걷기, 4 말하기(손 들기), 5 기뻐하기
  줄 7 모모(물든 굴토끼, 히로인, docs/decisions/0029): 0·1 서 있기, 2·3 깡충, 4 웅크림, 5 뛰어 차기, 6 맞음
  줄 8 돌아온 모모(docs/decisions/0046): 같은 칸 순서. 꽃이 피고, 검은 눈, 따뜻한 털, 뿌리 없음
  줄 9 도토(물든 청설모, 둘째 길 보스, docs/decisions/0052): 같은 칸 순서. 도토리 갓 모자, 호박색 빛 눈
  줄 10 돌아온 도토: 검고 맑은 눈, 뿌리 없음, 모자는 그대로
  줄 11 땅강아지 수수(첫 쉼터의 장인, 싸우지 않는다, docs/decisions/0053): 0·1 서 있기, 2·3 걷기, 4 파기(삽발 들기), 5 기뻐하기
  줄 12 콩이(물든 세띠아르마딜로, 셋째 길 보스, docs/decisions/0054): 0·1 서 있기, 2·3 걷기, 4 웅크림(공), 5 구르기(공, 띠가 돈다), 6 맞음.
       물든 눈은 옅은 하늘빛(모모 초록·도토 호박색·졸개 붉은 눈과 다르다)
  줄 13 돌아온 콩이: 검고 맑은 눈, 뿌리 없음
  줄 14~18 새 졸개(0146): 땃쥐 · 고슴도치(4·5 가시 공) · 두꺼비(5 입 벌림, 혀는 게임이 그린다) ·
       박쥐(0~3 날갯짓, 4 접기, 5 내리꽂기) · 포자버섯(4 움츠림, 5 뿜기)
  줄 19~37: 줄 0~18의 흰 실루엣(피격 번쩍임용)

그리는 법(nail.py와 같다): 채움 색만 글자로 찍고 외곽선('o')은 자동으로 두른다. 종마다 기본 자세 한 장을 손으로 찍고,
동작은 그 자세를 옮기거나(통통·웅크림·내밀기) 몇 점만 바꿔 만든다(발, 눈, 꼬리).
v1(타원을 코드로 쌓은 것)은 윤곽이 울퉁불퉁하고 얼룩져 종이 안 읽혔다 → 손으로 찍은 큰 덩어리 + 3단 음영 + 또렷한 눈.

문법(모든 종이 같다):
  - 몸은 덩어리 하나, 위·왼쪽이 밝고(h) 아래가 어둡다(d). 배·주둥이는 밝은 털(l).
  - 뿌리 독에 물든 표시 = 붉게 빛나는 눈(e, 반사 w) + 등의 보라 뿌리(v 짙음, V 밝음). 물들지 않은 이웃(포포)은 검은 눈.
  - 실루엣으로 구별: 들쥐 = 둥근 귀 하나 + 가는 꼬리, 생쥐 = 작은 몸에 큰 귀, 시궁쥐 = 길고 구부정 + 굵은 꼬리 + 등 가시,
    다람쥐 = 등줄무늬 + 위로 선 꼬리, 청설모 = 귀 술 + 몸보다 큰 꼬리.
"""
import sys

from PIL import Image

N = 16
OUTLINE = (43, 29, 42)
# 물든 적의 외곽선: Naïl(따뜻한 자두색)보다 어둡고 차갑게.
OUTLINE_TAINTED = (26, 16, 38)

# 값(밝기) 위계: 무리 속에서 Naïl이 먼저 읽히게, 물든 적은 털색을 보라 쪽으로 기울이고 조금 어둡게.
# 들쥐가 Naïl과 같은 흙갈색이라 섞이면 안 보였다(사람 피드백 2026-10-25). 종 색은 남긴다(회색 생쥐, 주황 다람쥐).
TAINT = (84, 58, 112)
TAINT_MIX = 0.28
TAINT_VALUE = 0.82


def taint(rgb):
    return tuple(int((c * (1 - TAINT_MIX) + t * TAINT_MIX) * TAINT_VALUE) for c, t in zip(rgb, TAINT))

# 공통 색(종마다 털 색만 바꾼다)
COMMON = {
    'p': (240, 150, 164),   # 분홍(귀 안·코·발·꼬리)
    'q': (196, 104, 124),   # 분홍 그림자
    'e': (255, 64, 104),    # 물든 눈(붉은 빛)
    'w': (255, 214, 226),   # 눈 반사
    'n': (40, 26, 40),      # 코 끝·검은 눈
    'v': (98, 46, 120),     # 뿌리(짙음)
    'V': (156, 86, 178),    # 뿌리(밝음)
    'k': (52, 34, 32),      # 짙은 줄무늬
    'c': (248, 238, 218),   # 흰 줄무늬·이빨
    'W': (255, 255, 255),   # 흰 반사(포포 눈)
    'r': (236, 132, 140),   # 볼 홍조
    'g': (150, 230, 90),    # 모모의 물든 눈(초록 빛 — 졸개의 붉은 눈과 다르다)
    'G': (232, 255, 204),   # 그 반사
    'f': (176, 128, 140),   # 시든 꽃(귀 밑동, 돌아오면 핀다)
    'F': (255, 168, 196),   # 핀 꽃잎(돌아온 모모)
    'Y': (255, 226, 120),   # 꽃 가운데
    'A': (255, 168, 48),    # 도토의 물든 눈(호박색 빛 — 모모의 초록, 졸개의 붉은 눈과 다르다)
    'a': (255, 236, 180),   # 그 반사
    'B': (150, 220, 255),   # 콩이의 물든 눈(옅은 하늘빛)
    'u': (98, 64, 38),      # 도토리 갓(짙음)
    'U': (160, 112, 64),    # 도토리 갓(밝음)
}

FUR = {
    #           d(그늘)          b(털)            h(밝음)          l(배·주둥이)
    'field':  [(104, 72, 56), (146, 106, 78), (184, 146, 108), (222, 196, 160)],
    'house':  [(112, 108, 124), (156, 152, 166), (196, 194, 206), (234, 230, 232)],
    'sewer':  [(70, 58, 62), (102, 88, 88), (136, 120, 112), (178, 164, 148)],
    'chip':   [(146, 84, 44), (196, 124, 66), (230, 166, 100), (244, 222, 188)],
    'red':    [(84, 64, 66), (122, 98, 92), (160, 136, 124), (226, 218, 206)],
    'raccoon': [(96, 96, 110), (144, 144, 156), (188, 188, 198), (238, 236, 232)],
    'straw':  [(150, 110, 52), (198, 158, 84), (232, 200, 124), (110, 80, 56)],
    # 모모(물든 굴토끼): 따뜻한 회갈색이었을 털이 자두빛·잿빛으로. 초상화(portraits.py mfur)와 같은 계열.
    'momo':   [(58, 42, 66), (88, 64, 90), (118, 90, 116), (150, 124, 146)],
    # 돌아온 모모: 원래의 따뜻한 회갈색(0046).
    'momo_bloom': [(104, 84, 74), (150, 124, 106), (190, 164, 140), (228, 212, 192)],
    # 도토(청설모): 다른 청설모 졸개(잿빛 갈색)보다 붉은 밤색. 물든 줄은 taint로 보라 쪽.
    'doto':   [(112, 62, 42), (160, 96, 60), (200, 134, 86), (238, 216, 190)],
    # 수수(땅강아지): 벨벳 같은 짙은 갈색 + 금빛 윤기. 삽발(앞다리)은 짙은 줄(k)에 흰 이(c).
    'susu':   [(66, 46, 36), (108, 78, 56), (156, 120, 84), (214, 180, 132)],
    # 콩이(세띠아르마딜로): 잿빛 도는 황갈색 등딱지(띠 = d·h 번갈아), 분홍빛 도는 맨살(l·p).
    'kongi':  [(92, 80, 74), (138, 122, 106), (186, 168, 142), (222, 190, 170)],
    # 새 졸개(0146). 땃쥐 = 짙은 잿빛 갈색, 고슴도치 = 가시 갈색 + 크림 얼굴, 두꺼비 = 올리브, 박쥐 = 자두빛 잿색,
    # 포자버섯 = 팥빛 갓 + 크림 대.
    'shrew':  [(72, 58, 58), (108, 90, 86), (146, 128, 118), (196, 178, 160)],
    'hedgehog': [(92, 70, 56), (140, 110, 84), (196, 170, 130), (232, 212, 184)],
    'toad':   [(84, 82, 48), (122, 118, 66), (164, 158, 94), (206, 196, 140)],
    'bat':    [(56, 44, 58), (90, 72, 88), (128, 108, 120), (176, 150, 150)],
    'spore':  [(120, 52, 62), (170, 78, 86), (212, 118, 116), (236, 224, 206)],
    # 토끼풀(어미 굴토끼, 0165): 모모보다 짙은 회갈색. 물든 줄은 taint로 보라 쪽, 돌아온 줄·아기들은 그대로.
    'clover': [(92, 72, 66), (138, 112, 98), (178, 152, 132), (224, 206, 186)],
}


def grid(rows):
    rows = [r.ljust(N, '.')[:N] for r in rows]
    assert len(rows) == N, len(rows)
    return [list(r) for r in rows]


def copy(g):
    return [list(r) for r in g]


def shift(g, dx=0, dy=0, rows=None):
    """전체(또는 rows 줄만)를 옮긴다. 밖으로 나가면 잘린다."""
    out = [['.'] * N for _ in range(N)]
    for y in range(N):
        for x in range(N):
            if g[y][x] == '.':
                continue
            if rows is not None and y not in rows:
                ny, nx = y, x
            else:
                ny, nx = y + dy, x + dx
            if 0 <= nx < N and 0 <= ny < N:
                out[ny][nx] = g[y][x]
    return out


def put(g, x, y, s):
    for i, ch in enumerate(s):
        if 0 <= x + i < N and 0 <= y < N and ch != ' ':
            g[y][x + i] = ch


def find(g, chars):
    return [(x, y) for y in range(N) for x in range(N) if g[y][x] in chars]


def clear_row(g, y):
    g[y] = ['.'] * N


def squint(g):
    """맞음: 붉은 눈을 질끈(짙은 가로줄)."""
    eyes = find(g, 'ewgGWAaB')
    if not eyes:
        return
    top = min(y for _, y in eyes)
    for x, y in eyes:
        g[y][x] = 'n' if y == top else 'b'


def outline(g):
    out = copy(g)
    for y in range(N):
        for x in range(N):
            if g[y][x] != '.':
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < N and 0 <= ny < N and g[ny][nx] != '.':
                    out[y][x] = 'o'
                    break
    return out


# ---- 기본 자세(손으로 찍음) ----------------------------------------------------------
# 들쥐: 통통한 몸, 머리 위 둥근 귀 하나, 뾰족한 주둥이, 가는 꼬리가 뒤로 휘어 내려감.
FIELD = grid([
    "",
    "",
    "",
    "",
    "........dd",
    ".......dqpd",
    ".......dppd",
    "....hhhhhhhh",
    "...hhvVbbbbbhh",
    "..hbvvbbbbbwebh",
    "..bbbbbbbbbeebbp",
    "pdbbbbbbbbbbbll",
    "p.dbbbbbbbbblll",
    ".p.ddbbbbbbll",
    "....pp...pp",
    "",
])

# 생쥐: 작은 몸, 머리만 한 큰 귀 둘(먼 귀가 살짝 보임), 큰 눈.
HOUSE = grid([
    "",
    "",
    "",
    "",
    "",
    "......ddd",
    ".....dqqpd.d",
    ".....dqppddpd",
    "......hhhhhbd",
    "....hvVbbbbwbh",
    "...hbvbbbbbeebp",
    "p..dbbbbbbbblll",
    ".p.ddbbbbbblll",
    "..pp.dddddll",
    ".....pp..pp",
    "",
])

# 시궁쥐: 길고 구부정한 덩치, 작은 귀, 긴 주둥이, 등에 보라 뿌리 가시 셋, 굵은 꼬리.
SEWER = grid([
    "",
    "",
    "",
    ".....V.V",
    "....vVvVv..dd",
    "...hhvhvhhdqd",
    "..hbbbbbbbbhdh",
    ".hbbbbbbbbbbbbh",
    ".bbbbbbbbbbbwebh",
    "dbbbbbbbbbbbeebbp",
    "ddbbbbbbbbbbblll",
    "q.dbbbbbbbbbllll",
    "qq.ddbbbbbbbll",
    ".qq..ddddddd",
    "....pp....pp",
    "",
])

# 다람쥐: 짧은 몸, 등에 짙은·흰 줄무늬, 위로 말려 선 꼬리, 볼이 통통.
CHIP = grid([
    "",
    "..hh",
    ".hbbh",
    ".bbkbh",
    ".bbkbb....dd",
    "..bbkb...dqpd",
    "..dbbb..hhhhhh",
    "...dbkkkkkbbbbbh",
    "....hcccccbbwebh",
    "...bbkkkkkbbeebp",
    "...dbbbbbbbbblll",
    "....dbbbbbbblll",
    ".....ddbbbbll",
    "......dddddl",
    ".......pp.pp",
    "",
])

# 청설모: 잿빛 갈색, 귀 끝 술(짙은 털), 몸보다 큰 꼬리가 등 뒤로 높이 선다, 흰 배.
RED = grid([
    ".hhh",
    "hbbbh",
    "bbbbbh",
    "bbdbbb...k.k",
    "dbbdbb...dkd",
    ".dbbdb..dqpd",
    "..dbbbb.hhhhh",
    "...dbbhhbbbbbh",
    "....hvVbbbbwebh",
    "...hbvbbbbbeebbp",
    "...dbbbbbbbbblll",
    "....dbbbbbbblll",
    ".....ddbbbblll",
    "......ddddll",
    ".......pp.pp",
    "",
])

# 모모(물든 굴토끼): 긴 귀 둘(가까운 귀 안은 어두운 보라 q, 밑동에 시든 꽃 f), 동그란 머리, 초록 빛 눈(g/G),
# 하얀 솜꼬리(c), 몸에 보라 뿌리. 졸개(쥐)보다 몸이 둥글고 귀가 실루엣을 만든다.
MOMO = grid([
    ".......b....bb..",
    ".......hb..bhhb.",
    ".......bhb.bhqb.",
    "........bhbbhqb.",
    "........bhbbhqb.",
    ".........dbbbffb",
    ".......hhhhhhhf.",
    "......hhbbbbbbbh",
    ".....hbbbbbbGgbh",
    "....hbbbbbbbggbp",
    "..ccbvVbbbbbrlll",
    ".cccdvbbbbbblll.",
    "..cddbbbbbbll...",
    "....ddbbbbll....",
    ".....pp..pp.....",
    "",
])

# 돌아온 모모(0046): 같은 자세. 귀 밑동의 꽃이 피고(F 꽃잎, Y 가운데), 귀 안이 분홍, 눈은 검고 맑다(포포처럼 n + 반사 W),
# 몸의 보라 뿌리는 없다. 볼 홍조(r)는 그대로.
MOMO_BLOOM = grid([
    ".......b....bb..",
    ".......hb..bhhb.",
    ".......bhb.bhpb.",
    "........bhbbhpb.",
    "........bhbbhqbF",
    ".........dbbbFYF",
    ".......hhhhhhhF.",
    "......hhbbbbbbbh",
    ".....hbbbbbbnWbh",
    "....hbbbbbbbnnbp",
    "..ccbbbbbbbbrlll",
    ".cccdbbbbbbblll.",
    "..cddbbbbbbll...",
    "....ddbbbbll....",
    ".....pp..pp.....",
    "",
])

# 토끼풀(물든 어미 굴토끼, 0165): 모모보다 크고 무겁다. 귀는 뒤로 눕혔고(경계), 귀 밑동에 시든 꽃(f), 물든 눈은
# 모모와 같은 초록(g/G, 가족), 몸에 보라 뿌리(v/V), 솜꼬리(c). 오른쪽을 본다.
CLOVER = grid([
    "....bb..bb......",
    "...bhhbbhhb.....",
    "...bhqbbhqb.....",
    "....bqbbqbb.....",
    ".....bbbbbbb....",
    "......hhhhhhhf..",
    ".....hhbbbbbbbh.",
    "....hbbbbbbbGgbh",
    "...hbbbbbbbbggbp",
    "..hbbbbbbbbbrlll",
    ".ccbvVbbbbbbblll",
    "cccdvbbbbbbbblll",
    ".cddbbbbbbbbbll.",
    "..ddbbbbbbbbll..",
    "....pp..pp..pp..",
    "",
])

# 돌아온 토끼풀(0165): 같은 자세. 꽃이 피고(F·Y), 귀 안이 분홍, 눈은 검고 맑다(n + 반사 W), 보라 뿌리는 없다.
CLOVER_BACK = grid([
    "....bb..bb......",
    "...bhhbbhhb.....",
    "...bhpbbhpb.....",
    "....bpbbpbb.....",
    ".....bbbbbbb....",
    "......hhhhhhhF..",
    ".....hhbbbbbbFYF",
    "....hbbbbbbbWnbh",
    "...hbbbbbbbbnnbp",
    "..hbbbbbbbbbrlll",
    ".ccbbbbbbbbbblll",
    "cccdbbbbbbbbblll",
    ".cddbbbbbbbbbll.",
    "..ddbbbbbbbbll..",
    "....pp..pp..pp..",
    "",
])

# 아기 굴토끼(박하·민들레, 0165, 깨어난 모습): 작고 둥글다. 귀 둘, 검은 눈, 솜꼬리. 고치는 코드 그림(render.rs).
BABY = grid([
    "",
    "",
    "",
    "",
    "",
    "......bb.bb.....",
    "......bpbbpb....",
    ".......bbbb.....",
    "......hbbnWb....",
    ".....hbbbbbbb...",
    "....cbbbbbbrl...",
    "....cbbbbbbbl...",
    ".....dbbbbbl....",
    "......pp.pp.....",
    "",
    "",
])

# 도토(물든 청설모, 0052): 청설모 실루엣(큰 꼬리·귀 술) + 머리에 도토리 갓 모자(u/U). 물든 눈은 호박색(A/a), 몸에 보라 뿌리.
DOTO = grid([
    ".hhh",
    "hbbbh",
    "bbbbbh",
    "bbdbbb....k",
    "dbbdbb...uUUu",
    ".dbbdb..uUUUUu",
    "..dbbbb.hhhhhh",
    "...dbbhhbbbbbbh",
    "....hvVbbbbaAbh",
    "...hbvbbbbbAAbbp",
    "...dbbbbbbbbblll",
    "....dbbbbbbblll",
    ".....ddbbbblll",
    "......ddddll",
    ".......pp.pp",
    "",
])

# 돌아온 도토: 같은 자세, 뿌리 없음, 검고 맑은 눈(n + 반사 W), 볼 홍조 하나. 모자는 그대로.
DOTO_BACK = grid([
    ".hhh",
    "hbbbh",
    "bbbbbh",
    "bbdbbb....k",
    "dbbdbb...uUUu",
    ".dbbdb..uUUUUu",
    "..dbbbb.hhhhhh",
    "...dbbhhbbbbbbh",
    "....hbbbbbbWnbh",
    "...hbbbbbbrnnbbp",
    "...dbbbbbbbbblll",
    "....dbbbbbbblll",
    ".....ddbbbblll",
    "......ddddll",
    ".......pp.pp",
    "",
])

# 수수(땅강아지, 0053): 오른쪽을 본다. 앞은 둥근 투구 같은 가슴(벨벳, 윤기 h) + 작은 머리·구슬 눈(n·W)·짧은 더듬이(k),
# 뒤는 마디진 배(d 줄무늬)와 꼬리털 둘. 앞다리는 크고 짙은 삽발(k)에 흰 이(c) — 두더지 발과 같은 "파는 손"(수렴 진화).
SUSU = grid([
    "",
    "",
    "",
    "",
    "",
    "..........k..k",
    "...........kk",
    "......hhhhhk",
    "....hhhbbbbhh",
    "..hhbbbbbbbWnb",
    ".hbdbdbbbbbbnbb",
    "hbdbdbdbbbbbbkk",
    "bdbdbdbdbbbbkkhk",
    ".ddddddddbbkkhkh",
    "..p..p..p..pkk",
    "",
])

# 콩이(세띠아르마딜로, 0054): 오른쪽을 본다. 둥근 등딱지에 세로 띠(d·h 번갈아), 앞쪽에 작은 머리 방패·뾰족한 주둥이·작은 귀(l),
# 짧은 다리, 뒤에 짧고 굵은 꼬리. 물든 눈은 옅은 하늘빛(B), 등 위에 보라 뿌리.
KONGI = grid([
    "",
    "",
    "",
    "",
    "",
    "...hhhhhh",
    ".hhbhbhvhh",
    "hbhbhbVbhbh..l",
    "hbdbdbdbdbdbhhh",
    "dbdbdbdbdbdbbBll",
    "dbdbdbdbdbdbllln",
    ".dddddddddddl",
    "..l.l....l.l",
    "..pp.p...pp.p",
    "",
    "",
])

KONGI_BACK = grid([
    "",
    "",
    "",
    "",
    "",
    "...hhhhhh",
    ".hhbhbhbhh",
    "hbhbhbhbhbh..l",
    "hbdbdbdbdbdbhhh",
    "dbdbdbdbdbdbbnll",
    "dbdbdbdbdbdbrlln",
    ".dddddddddddl",
    "..l.l....l.l",
    "..pp.p...pp.p",
    "",
    "",
])

# 공으로 말린 콩이(웅크림·구르기). 띠 위상만 바꿔 구르는 느낌을 낸다.
KONGI_BALL = [grid([
    "",
    "",
    "",
    "",
    "",
    ".....hhhhh",
    "....h" + band[0:6] + "h",
    "...h" + band[1:9] + "h",
    "...h" + band[0:8] + "d",
    "..h" + band[1:11] + "d",
    "...d" + band[0:8] + "d",
    "...d" + band[1:9] + "d",
    "....d" + band[0:6] + "d",
    ".....ddddd",
    "",
    "",
]) for band in ("bdbdbdbdbdbd", "dbdbdbdbdbdb")]

# 땃쥐(0146): 낮고 긴 몸, 몸 밖으로 길게 나온 뾰족 주둥이(코 끝 n), 작은 눈, 귀는 털에 묻혔다, 짧은 꼬리.
SHREW = grid([
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "......hhhhh",
    "....hhvVbbbhh",
    "...hbvvbbbbbwhh",
    "dd.dbbbbbbbbebbn",
    "..ddbbbbbbbblll",
    "....ddbbbbbll",
    ".....pp...pp",
    "",
])

# 고슴도치(0146): 등을 덮은 가시(짙은 k·밝은 끝 h), 앞에 크림빛 얼굴(l)과 뾰족 코, 짧은 다리.
HEDGEHOG = grid([
    "",
    "",
    "",
    "",
    "",
    "....h.h.h.h",
    "...hkhkhkhkh",
    "..hkbkbkbkbkh",
    ".hkbkvkbkbkbkh",
    "hkbkbVbkbkbkbl",
    "kbkbkbkbkbkllw",
    "dbkbkbkbkbklleln",
    "ddbdbdbdbdbllll",
    ".ddddddddddll",
    "..pp......pp",
    "",
])

# 말린 고슴도치(웅크림·구르기): 가시 공. 콩이 공과 같은 틀, 가시 위상만 바꾼다.
HEDGEHOG_BALL = [grid([
    "",
    "",
    "",
    "",
    "",
    ".....h.h.h",
    "....h" + band[0:6] + "h",
    "...h" + band[1:9] + "h",
    "..h" + band[0:10] + "h",
    "..h" + band[1:11] + "d",
    "...d" + band[0:8] + "d",
    "...d" + band[1:9] + "d",
    "....d" + band[0:6] + "d",
    ".....ddddd",
    "",
    "",
]) for band in ("khkhkhkhkhkh", "hkhkhkhkhkhk")]

# 두꺼비(0146): 넓고 납작한 몸, 머리 위로 불룩한 눈, 앞쪽 긴 입선(k), 등에 사마귀(d)와 보라 뿌리, 벌린 다리.
TOAD = grid([
    "",
    "",
    "",
    "",
    "",
    "",
    "..........hh",
    ".........hwh",
    "....hhhhhheh",
    "..hhbbdbVbbbh",
    ".hbbdbvbbdbbbh",
    "hbbbbbbbbbkkkkk",
    "hbdbbbbbblllllh",
    ".ddbbbbblllll",
    "..pp.pp..pp.pp",
    "",
])

# 박쥐(0146): 늘 떠 있다(높이는 게임이 올린다). 작은 몸, 뾰족 귀 둘, 날개막(d)과 날개뼈(b).
BAT_UP = grid([
    "",
    "",
    "",
    ".d............d",
    ".dd....h.h...dd",
    ".ddd...hbhh.ddd",
    "..dddbhbbwbbdd",
    "...ddbbbbebbd",
    "....bbvbbbbb",
    ".....dbbbbl",
    "......dbbl",
    ".......pp",
    "",
    "",
    "",
    "",
])

BAT_DOWN = grid([
    "",
    "",
    "",
    "",
    ".......h.h",
    ".......hbhh",
    ".....bhbbwbb",
    "...ddbbbbebbdd",
    "..dddbbvbbbbddd",
    ".ddd..dbbbl..ddd",
    ".dd....dbbl...dd",
    ".d.....pp......d",
    "",
    "",
    "",
    "",
])

# 포자버섯(0146): 점박이 갓(c 점), 크림빛 대(l)에 물든 눈 둘, 밑동에 보라 뿌리. 움직이지 않는다.
SPORE = grid([
    "",
    "",
    "",
    "......hhhh",
    "....hhhchhhh",
    "...hhcchhhchh",
    "..hbbbbbbbcbbh",
    "..bbcbbbbbbbbb",
    "..dbbbbbbcbbbd",
    "...dddddddddd",
    ".....llllll",
    ".....lwllwl",
    ".....lelle",
    ".....llllll",
    "...vvpllllpvv",
    "",
])

# 허수아비: 말뚝 + 짚 몸 + 천 머리(단추 눈).
DUMMY = grid([
    "",
    "......cccc",
    ".....cccccc",
    ".....cnccnc",
    ".....cckkcc",
    "......dhhd",
    "..llllbbbbllll",
    "....bbbbbbbb",
    "...dbbbbbbbbd",
    "....dbbbbbbd",
    ".....dbbbbd",
    "......ll",
    "......ll",
    "......ll",
    ".....llll",
    "",
])

# 라쿤 포포: 서서 손을 흔드는 안내자. 동그란 몸, 흰 얼굴에 검은 가면, 맑은 눈, 줄무늬 꼬리.
RACCOON = grid([
    "",
    "....bb...bb",
    "...bnhb.bnhb",
    "...bhhhhhhhb",
    "..bhlllhlllhb",
    "..bkWnklkWnkb",
    "..bknnklknnkb",
    "...lllllnllll",
    "....llrlllrl",
    "...dbbbbbbbb",
    "..dbbbllllbbb",
    "nbdbbllllllbb",
    "bnbbbllllllbb",
    "nb.dbbbbbbbb",
    "....nn..nn",
    "",
])


SPECIES = [
    ('field', FIELD),
    ('house', HOUSE),
    ('sewer', SEWER),
    ('chip', CHIP),
    ('red', RED),
]


def feet_row(g):
    rows = [y for y, _ in [(y, 0) for y in range(N)] if any(c == 'p' for c in g[y]) and y >= 13]
    return max(rows) if rows else 14


def walk(g, i):
    f = copy(g)
    fy = feet_row(f)
    feet = [x for x in range(N) if f[fy][x] == 'p']
    if i == 1:
        f = shift(f, dy=-1, rows=range(0, fy))   # 몸이 통 튄다(발은 땅에)
        # 앞발을 앞으로 한 칸
        for x in sorted(feet, reverse=True)[:2]:
            f[fy][x] = '.'
        for x in sorted(feet, reverse=True)[:2]:
            if x + 1 < N:
                f[fy][x + 1] = 'p'
    return f


def breathe(g):
    """숨쉬기: 등 윗줄을 한 칸 내려 납작하게."""
    top = min(y for _, y in find(g, 'hbdlvVkc'))
    f = shift(g, dy=1, rows=range(0, top + 1))
    return f


def crouch(g):
    """예고: 몸을 낮추고(아래로 한 칸) 뿌리가 밝게 선다."""
    fy = feet_row(g)
    f = shift(g, dy=1, rows=range(0, fy))
    for x, y in find(f, 'v'):
        f[y][x] = 'V'
    return f


def lunge(g):
    """공격: 앞으로 한 칸 내밀고 뒷발은 남긴다."""
    fy = feet_row(g)
    f = shift(g, dx=1, rows=range(0, fy))
    return f


def hurt(g):
    f = shift(g, dx=-1)
    squint(f)
    return f


def frames_of(g):
    return [g, breathe(g), walk(g, 0), walk(g, 1), crouch(g), lunge(g), hurt(g)]


def raccoon_frames():
    g = RACCOON
    wave = copy(g)            # 말하기: 오른손을 든다
    put(wave, 13, 8, 'b')
    put(wave, 13, 7, 'b')
    put(wave, 14, 6, 'nb')
    happy = copy(g)           # 기뻐하기: 감은 눈(가면 위 흰 ^ 선) + 양손
    put(happy, 3, 5, 'kWWk')
    put(happy, 8, 5, 'kWWk')
    put(happy, 3, 6, 'kkkk')
    put(happy, 8, 6, 'kkkk')
    put(happy, 1, 9, 'b')
    put(happy, 13, 9, 'b')
    return [g, breathe(g), walk(g, 0), walk(g, 1), wave, happy]


def susu_frames():
    g = SUSU
    dig = shift(g, dy=1, rows=range(0, 14))   # 파기: 몸을 낮추고 삽발을 앞으로 든다
    for y in range(11, 15):
        for x in range(12, 16):
            if dig[y][x] in 'kh':
                dig[y][x] = 'b' if y < 13 and x < 14 else '.'
    put(dig, 13, 9, 'kh')
    put(dig, 13, 10, 'kkh')
    put(dig, 13, 11, 'kk')
    happy = copy(g)                            # 기뻐하기: 감은 눈(^), 더듬이가 선다
    put(happy, 11, 9, 'bk')
    put(happy, 12, 10, 'b')
    put(happy, 10, 4, 'k')
    put(happy, 13, 4, 'k')
    return [g, breathe(g), walk(g, 0), walk(g, 1), dig, happy]


def kongi_frames(g):
    """콩이: 서 있기·걷기는 보통, 예고·구르기는 공(띠가 엇갈린 두 장), 맞음은 공에서 몸을 편 채 뒤로."""
    roots = [(x, y) for x, y in find(g, 'vV')]
    balls = []
    for b in KONGI_BALL:
        bb = copy(b)
        for (x, y) in roots[:1]:
            if bb[y][x] != '.':
                bb[y][x] = 'v'
        balls.append(bb)
    return [g, breathe(g), walk(g, 0), walk(g, 1), balls[0], balls[1], hurt(g)]


def hedgehog_frames(g):
    """고슴도치: 콩이처럼 예고·구르기는 가시 공 두 장."""
    return [g, breathe(g), walk(g, 0), walk(g, 1), HEDGEHOG_BALL[0], HEDGEHOG_BALL[1], hurt(g)]


def toad_frames(g):
    """두꺼비: 공격 칸은 입을 벌린다(분홍 혀뿌리). 혀 줄은 게임이 그린다."""
    lash = copy(g)
    put(lash, 10, 11, 'kqqqp')
    put(lash, 10, 12, 'lkkkk')
    return [g, breathe(g), walk(g, 0), walk(g, 1), crouch(g), lash, hurt(g)]


def bat_frames():
    """박쥐: 0·1 서 있기 = 날갯짓, 2·3 걷기 = 날갯짓, 4 예고 = 날개를 모은다, 5 내리꽂기 = 날개를 뒤로."""
    up, down = BAT_UP, BAT_DOWN
    fold = copy(up)
    for y in range(N):
        for x in range(N):
            if fold[y][x] == 'd' and (x < 4 or x > 12):
                fold[y][x] = '.'
    for x, y in find(fold, 'v'):
        fold[y][x] = 'V'
    dive = shift(down, dx=1)
    return [up, down, up, down, fold, dive, hurt(up)]


def spore_frames(g):
    """포자버섯: 숨쉬기만, 4 움츠림(갓이 내려앉음), 5 뿜기(갓이 부풀고 점이 밝다)."""
    squash = shift(g, dy=1, rows=range(0, 10))
    puff = shift(g, dy=-1, rows=range(0, 10))
    for x, y in find(puff, 'c'):
        puff[y][x] = 'W'
    return [g, breathe(g), g, breathe(g), squash, puff, hurt(g)]


def paint(img, ox, oy, g, pal, silhouette=False, tainted=False):
    d, b, h, l = (taint(c) for c in pal) if tainted else pal
    colors = dict(COMMON, d=d, b=b, h=h, l=l, o=OUTLINE_TAINTED if tainted else OUTLINE)
    for y in range(N):
        for x in range(N):
            ch = g[y][x]
            if ch == '.':
                continue
            c = (255, 255, 255) if silhouette else colors[ch]
            img.putpixel((ox + x, oy + y), c + (255,))


def sheet():
    rows = []
    for name, g in SPECIES:
        rows.append((FUR[name], frames_of(g), True))
    rows.append((FUR['straw'], [DUMMY, DUMMY, DUMMY, DUMMY, DUMMY, lunge(DUMMY), shift(DUMMY, dx=-1)], False))
    rows.append((FUR['raccoon'], raccoon_frames(), False))
    rows.append((FUR['momo'], frames_of(MOMO), False))
    rows.append((FUR['momo_bloom'], frames_of(MOMO_BLOOM), False))
    rows.append((FUR['doto'], frames_of(DOTO), True))
    rows.append((FUR['doto'], frames_of(DOTO_BACK), False))
    rows.append((FUR['susu'], susu_frames(), False))
    rows.append((FUR['kongi'], kongi_frames(KONGI), True))
    rows.append((FUR['kongi'], kongi_frames(KONGI_BACK), False))
    rows.append((FUR['shrew'], frames_of(SHREW), True))
    rows.append((FUR['hedgehog'], hedgehog_frames(HEDGEHOG), True))
    rows.append((FUR['toad'], toad_frames(TOAD), True))
    rows.append((FUR['bat'], bat_frames(), True))
    rows.append((FUR['spore'], spore_frames(SPORE), True))
    rows.append((FUR['clover'], frames_of(CLOVER), True))
    rows.append((FUR['clover'], frames_of(CLOVER_BACK), False))
    rows.append((FUR['clover'], frames_of(BABY), False))
    img = Image.new('RGBA', (N * 7, N * len(rows) * 2), (0, 0, 0, 0))
    for ry, (pal, frames, tainted) in enumerate(rows):
        for rx, g in enumerate(frames):
            g = outline(g)
            paint(img, rx * N, ry * N, g, pal, tainted=tainted)
            paint(img, rx * N, (ry + len(rows)) * N, g, pal, silhouette=True)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/critters.png')
