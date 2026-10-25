"""적·이웃 스프라이트 시트 생성기(docs/decisions/0021). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/critters.py [출력 경로]  (기본 assets/art/critters.png)
칸 16×16, 오른쪽을 본다(왼쪽은 게임에서 좌우 반전). 발바닥 y=14 근처, 그림자는 게임이 그린다.
  줄 0 들쥐 · 1 생쥐 · 2 시궁쥐 · 3 다람쥐 · 4 청설모 · 5 허수아비 (species.rs의 sheet_row)
    칸 0·1 서 있기, 2·3 걷기, 4 예고(떨기·웅크림·조준), 5 공격(돌진·뛰기·던지기), 6 맞음
  줄 6 라쿤 포포(튜토리얼 안내): 0·1 서 있기, 2·3 걷기, 4 말하기(손 들기), 5 기뻐하기
  줄 7~13: 줄 0~6의 흰 실루엣(피격 번쩍임용)

그리는 법: 부위를 작은 타원으로 놓고(뒤에 있는 것부터), 타원 안 위치로 3단 음영(왼쪽 위 빛).
외곽선은 자동(이웃 재질의 가장 어두운 색 = 선택 외곽선). 뿌리에 물든 표시: 붉게 빛나는 눈 + 등에 보라 뿌리 가시.
"""
import sys

from PIL import Image

N = 16
# 재질 → (어두움, 보통, 밝음)
RAMPS = {
    # 들쥐: 흙빛 갈색
    'fm': [(96, 70, 58), (138, 104, 82), (176, 140, 108)],
    # 생쥐: 회색
    'hm': [(104, 100, 112), (150, 146, 156), (196, 192, 200)],
    # 시궁쥐: 짙은 회갈색
    'sr': [(70, 56, 58), (100, 84, 80), (134, 116, 106)],
    # 다람쥐: 주황 갈색
    'ch': [(136, 82, 48), (190, 124, 70), (224, 166, 104)],
    # 청설모: 잿빛 갈색
    'rs': [(76, 60, 60), (112, 92, 86), (150, 128, 116)],
    # 라쿤: 회색
    'rc': [(92, 92, 104), (140, 140, 150), (186, 186, 194)],
    'belly': [(178, 158, 136), (214, 196, 170), (236, 224, 204)],
    'grey_belly': [(176, 172, 176), (214, 210, 214), (238, 234, 236)],
    'pink': [(176, 92, 112), (226, 136, 152), (250, 186, 194)],
    'stripe_d': [(52, 34, 30), (66, 44, 36), (82, 56, 44)],
    'stripe_l': [(214, 200, 176), (238, 226, 204), (250, 244, 230)],
    'mask': [(30, 26, 36), (44, 40, 52), (60, 56, 70)],
    'white': [(206, 204, 206), (236, 234, 232), (252, 252, 250)],
    'root': [(58, 30, 70), (96, 48, 114), (140, 76, 160)],
    'straw': [(150, 112, 56), (196, 156, 84), (230, 196, 120)],
    'wood': [(78, 56, 40), (108, 80, 56), (140, 108, 76)],
    'cloth': [(110, 60, 64), (156, 88, 88), (196, 124, 116)],
}
EYE_GLOW = (255, 70, 110)
EYE_HI = (255, 214, 224)
EYE_DARK = (24, 16, 30)
EYE_WHITE = (255, 255, 255)
NOSE_DARK = (40, 30, 40)
BLUSH = (236, 126, 132)


class Cell:
    def __init__(self):
        self.m = [[None] * N for _ in range(N)]   # (재질, 단)
        self.c = [[None] * N for _ in range(N)]   # 덮어 칠한 색

    def put(self, x, y, mat, lv):
        x, y = int(x), int(y)
        if 0 <= x < N and 0 <= y < N:
            self.m[y][x] = (mat, lv)
            self.c[y][x] = None

    def paint(self, x, y, rgb):
        x, y = int(x), int(y)
        if 0 <= x < N and 0 <= y < N:
            if self.m[y][x] is None:
                self.m[y][x] = ('pink', 1)
            self.c[y][x] = rgb


def blob(cv, cx, cy, rx, ry, mat, flat=None):
    """음영 타원. 왼쪽 위가 밝고 오른쪽 아래가 어둡다."""
    for y in range(N):
        for x in range(N):
            dx, dy = (x + 0.5 - cx) / max(rx, 0.1), (y + 0.5 - cy) / max(ry, 0.1)
            if dx * dx + dy * dy <= 1.0:
                if flat is not None:
                    lv = flat
                else:
                    lit = -dx * 0.55 - dy * 0.8
                    lv = 2 if lit > 0.35 else (0 if lit < -0.45 else 1)
                cv.put(x, y, mat, lv)


def outline(cv):
    """빈 칸이 채운 칸과 4방향으로 닿으면 외곽선. 색은 닿은 재질의 가장 어두운 단보다 더 어둡게."""
    out = []
    for y in range(N):
        for x in range(N):
            if cv.m[y][x] is not None:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < N and 0 <= ny < N and cv.m[ny][nx] is not None:
                    mat = cv.m[ny][nx][0]
                    d = RAMPS[mat][0]
                    out.append((x, y, tuple(max(0, int(c * 0.55)) for c in d)))
                    break
    for x, y, c in out:
        cv.m[y][x] = ('outline', 0)
        cv.c[y][x] = c


def to_rgba(cv, silhouette=False):
    px = {}
    for y in range(N):
        for x in range(N):
            if cv.m[y][x] is None:
                continue
            if silhouette:
                px[(x, y)] = (255, 255, 255, 255)
                continue
            c = cv.c[y][x]
            if c is None:
                mat, lv = cv.m[y][x]
                c = RAMPS[mat][lv]
            px[(x, y)] = tuple(c) + (255,)
    return px


# ---- 쥐·다람쥐 ------------------------------------------------------------
SPECIES = {
    # body: (가로 반지름, 세로 반지름), head 반지름, ear 반지름, tail: thin/bushy, belly 재질
    'field_mouse': dict(fur='fm', belly='belly', body=(4.2, 3.0), head=3.0, ear=2.0, tail='thin', size=1.0),
    'house_mouse': dict(fur='hm', belly='grey_belly', body=(3.0, 2.2), head=2.6, ear=2.2, tail='thin', size=0.8),
    'sewer_rat': dict(fur='sr', belly='belly', body=(5.4, 3.6), head=3.2, ear=1.5, tail='thick', size=1.2),
    'chipmunk': dict(fur='ch', belly='belly', body=(3.8, 3.0), head=3.0, ear=1.3, tail='bushy', stripes=True, size=1.0),
    'red_squirrel': dict(fur='rs', belly='grey_belly', body=(3.8, 3.2), head=3.0, ear=1.4, tail='bushy_big', tufts=True, size=1.0),
}


def rodent(p, frame):
    cv = Cell()
    bw, bh = p['body']
    hr = p['head']
    cx, cy = 7.0, 14.0 - bh   # 몸 가운데(발이 14에 닿게)
    hx_off, hy_off = bw * 0.75, -bh * 0.55
    lean = 0.0
    if frame == 1:            # 숨쉬기
        bh *= 0.92
        cy += 0.3
    elif frame in (2, 3):     # 걷기: 통통
        cy -= 0.6 if frame == 3 else 0.0
    elif frame == 4:          # 예고: 몸을 낮추고 웅크림
        bw *= 1.08
        bh *= 0.82
        cy = 14.0 - bh
        lean = -0.6
    elif frame == 5:          # 공격: 앞으로 쭉
        bw *= 1.15
        bh *= 0.9
        cy = 14.0 - bh - 0.5
        lean = 1.0
    elif frame == 6:          # 맞음: 뒤로 젖힘
        lean = -1.2
        cy -= 0.5
    cx += lean
    hx, hy = cx + hx_off + lean * 0.5, cy + hy_off - (0.4 if frame == 5 else 0)
    fur, belly = p['fur'], p['belly']

    # 꼬리(맨 뒤)
    tx = cx - bw + 0.5
    if p['tail'] == 'thin' or p['tail'] == 'thick':
        w = 1 if p['tail'] == 'thin' else 2
        pts = [(tx - i, cy + 0.5 - (i * i) * 0.12 - (0.6 if frame in (2, 5) else 0)) for i in range(0, 5)]
        for (x, y) in pts:
            for k in range(w):
                cv.put(x, y + k, 'pink', 1 if k == 0 else 0)
    else:
        big = p['tail'] == 'bushy_big'
        rx, ry = (2.8, 5.0) if big else (2.4, 4.2)
        sway = (-0.5 if frame in (1, 3) else 0.0) + (1.0 if frame == 4 else 0.0)
        blob(cv, tx - 1.0 + sway * 0.3, cy - ry * 0.8 + sway, rx, ry, fur)
        blob(cv, tx - 0.4 + sway * 0.3, cy - ry * 1.5 + sway, rx * 0.8, ry * 0.5, fur)
    # 뒷발
    feet_y = 14
    back_foot = cx - bw * 0.5 + (1 if frame == 2 else 0)
    front_foot = cx + bw * 0.55 + (1 if frame == 3 else 0) + (1 if frame == 5 else 0)
    # 몸
    blob(cv, cx, cy, bw, bh, fur)
    # 배(아래 앞쪽)
    blob(cv, cx + bw * 0.35, cy + bh * 0.45, bw * 0.55, bh * 0.45, belly)
    # 줄무늬(다람쥐): 등 위에 검정·흰 줄
    if p.get('stripes'):
        for x in range(int(cx - bw + 1), int(cx + bw * 0.6)):
            for y in range(N):
                if cv.m[y][x] and cv.m[y][x][0] == fur:
                    top = y
                    break
            else:
                continue
            cv.put(x, top + 1, 'stripe_d', 1)
            cv.put(x, top + 2, 'stripe_l', 1)
    # 귀(머리 위 뒤쪽): 먼저 그리고 머리를 덮는다. 바깥은 어두운 단, 안은 분홍.
    er = p['ear']
    ex, ey = hx - hr * 0.45, hy - hr * 0.95
    blob(cv, ex, ey, er, er, fur, flat=0)
    if er >= 1.8:
        cv.put(ex, ey, 'pink', 2)
        cv.put(ex - 1, ey, 'pink', 1)
        cv.put(ex, ey + 1, 'pink', 1)
    elif er >= 1.4:
        cv.put(ex, ey, 'pink', 1)
    if p.get('tufts'):
        cv.put(ex, ey - er - 0.2, 'stripe_d', 1)
        cv.put(ex + 1, ey - er - 1.2, 'stripe_d', 1)
    # 머리
    blob(cv, hx, hy, hr, hr * 0.9, fur)
    # 주둥이
    blob(cv, hx + hr * 0.7, hy + hr * 0.35, hr * 0.55, hr * 0.42, belly)
    # 발
    for fx in (back_foot, front_foot):
        cv.put(fx, feet_y, 'pink', 0)
        cv.put(fx + 1, feet_y, 'pink', 1)
    # 코
    nx = hx + hr * 1.15
    cv.paint(nx, hy + hr * 0.15, NOSE_DARK)
    # 눈: 뿌리 독 = 붉게 빛남(2칸 + 반사). 맞을 때는 질끈(가로줄).
    ex2, ey2 = hx + hr * 0.25, hy - hr * 0.15
    if frame == 6:
        cv.paint(ex2, ey2, EYE_DARK)
        cv.paint(ex2 + 1, ey2, EYE_DARK)
    else:
        for ddx in (0, 1):
            for ddy in (0, 1):
                cv.paint(ex2 + ddx, ey2 + ddy, EYE_GLOW if ddy == 1 else EYE_DARK)
        cv.paint(ex2 + (1 if frame == 4 else 0), ey2, EYE_HI)
    # 뿌리 가시(등): 보라 뿌리 둘
    top_x = int(cx - bw * 0.55)
    for i, dx in enumerate((0, 1, 2)):
        x = top_x + dx
        for y in range(N):
            if cv.m[y][x] and cv.m[y][x][0] in (fur, 'stripe_d', 'stripe_l'):
                if i == 1:
                    cv.put(x, y - 1, 'root', 2)
                cv.put(x, y, 'root', 1)
                cv.put(x, y + 1, 'root', 0)
                break
    outline(cv)
    return cv


def dummy(frame):
    cv = Cell()
    lean = {6: -1.0}.get(frame, 0.0)
    for y in range(6, 15):
        cv.put(7 + lean * (14 - y) / 8, y, 'wood', 1)
    blob(cv, 7.5 + lean, 7.0, 4.0, 3.5, 'straw')
    blob(cv, 7.5 + lean, 3.0, 2.5, 2.3, 'cloth')
    cv.paint(8.5 + lean, 3, EYE_DARK)
    outline(cv)
    return cv


def raccoon(frame):
    """라쿤 포포: 서 있는 안내자. 동그란 몸, 검은 눈 가면, 줄무늬 꼬리. 눈은 맑다(물들지 않음)."""
    cv = Cell()
    bob = -0.5 if frame in (1, 3) else 0.0
    cx, cy = 7.0, 10.6 + bob
    # 꼬리(뒤): 회색·검정 고리
    for i, (x, y) in enumerate([(3, 12), (2, 11), (2, 10), (2, 9), (3, 8)]):
        mat = 'mask' if i % 2 == 0 else 'rc'
        cv.put(x, y + bob, mat, 1)
        cv.put(x + 1, y + bob, mat, 2 if mat == 'rc' else 1)
    # 몸
    blob(cv, cx, cy, 3.6, 3.6, 'rc')
    blob(cv, cx + 0.8, cy + 1.0, 2.2, 2.4, 'white', flat=1)
    # 발
    step = {2: (1, 0), 3: (0, 1)}.get(frame, (0, 0))
    cv.put(5 + step[0], 14, 'mask', 1)
    cv.put(9 + step[1], 14, 'mask', 1)
    # 팔(말하기: 오른팔을 위로)
    if frame == 4:
        cv.put(11, 8 + bob, 'rc', 1)
        cv.put(12, 7 + bob, 'rc', 1)
        cv.put(12, 6 + bob, 'mask', 1)
    elif frame == 5:
        cv.put(3, 8 + bob, 'rc', 1)
        cv.put(11, 8 + bob, 'rc', 1)
    else:
        cv.put(10, 11 + bob, 'mask', 1)
    # 머리
    hx, hy = 8.0, 5.0 + bob
    blob(cv, hx, hy, 3.6, 3.0, 'rc')
    # 귀
    blob(cv, hx - 2.5, hy - 2.6, 1.2, 1.2, 'rc', flat=0)
    blob(cv, hx + 2.0, hy - 2.8, 1.2, 1.2, 'rc', flat=0)
    cv.put(hx - 2.5, hy - 2.6, 'white', 2)
    cv.put(hx + 2.0, hy - 2.8, 'white', 2)
    # 흰 얼굴 + 검은 가면 띠
    blob(cv, hx + 0.5, hy + 1.2, 2.6, 1.4, 'white', flat=2)
    for x in range(int(hx - 2), int(hx + 3)):
        cv.put(x, hy, 'mask', 1)
    # 눈(가면 안의 맑은 눈 + 반사)
    if frame == 5:
        cv.paint(hx - 1, hy, EYE_WHITE)    # ^^ 대신 반짝
        cv.paint(hx + 2, hy, EYE_WHITE)
    else:
        cv.paint(hx - 1, hy, EYE_DARK)
        cv.paint(hx + 2, hy, EYE_DARK)
        cv.paint(hx - 1, hy - 1, EYE_WHITE)
        cv.paint(hx + 2, hy - 1, EYE_WHITE)
    # 코
    cv.paint(hx + 3, hy + 1, NOSE_DARK)
    cv.paint(hx + 2, hy + 2, BLUSH)
    outline(cv)
    return cv


def sheet():
    rows = []
    for name in ('field_mouse', 'house_mouse', 'sewer_rat', 'chipmunk', 'red_squirrel'):
        rows.append([rodent(SPECIES[name], f) for f in range(7)])
    rows.append([dummy(f) for f in range(7)])
    rows.append([raccoon(f) for f in range(6)])
    img = Image.new('RGBA', (N * 7, N * len(rows) * 2), (0, 0, 0, 0))
    for ry, row in enumerate(rows):
        for rx, cv in enumerate(row):
            for sil, oy in ((False, ry), (True, ry + len(rows))):
                for (x, y), c in to_rgba(cv, sil).items():
                    img.putpixel((rx * N + x, oy * N + y), c)
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/critters.png')
