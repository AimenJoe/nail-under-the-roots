"""초상화 A안(그림책 수채) 시험: Naïl 보통 표정 한 칸(2026-10-26, 사장이 고른 방향). 게임 그림에는 아직 쓰지 않는다.
실행: NAIL_ART_V4=1 python3 tools/art/trials/nail_book.py 출력.png  (레포 루트에서). portraits.py의 캔버스·음영·외곽선·털 단계를 빌려 쓴다."""
import math, sys, os
sys.path.insert(0, 'tools/art')
import portraits as P
from PIL import Image

# 수채 팔레트: 밝기를 위로 눌러 담고(가장 어두운 단도 중간 밝기쯤) 채도를 낮춘다. 털은 따뜻한 회갈색 벨벳.
def lift(ramp, lo=0.30, sat=0.8):
    import colorsys
    out = []
    for r, g, b in ramp:
        h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
        l = lo + (1 - lo) * l
        rr, gg, bb = colorsys.hls_to_rgb(h, l, s * sat)
        out.append((round(rr * 255), round(gg * 255), round(bb * 255)))
    return out
P.RAMP['bfurA'] = lift(P.hue_shift([(66, 44, 40), (100, 70, 56), (136, 98, 74), (170, 130, 98), (204, 168, 132)]), 0.1, 0.95)
P.RAMP['pinkA'] = lift(P.RAMP['pink'], 0.16, 0.9)
P.RAMP['scarfA'] = lift(P.RAMP['scarf'], 0.08, 0.95)
P.RAMP['clawA'] = lift(P.RAMP['claw'], 0.3, 0.7)
P.RAMP['bellyA'] = lift(P.RAMP['muzzle'], 0.2, 0.9)
P.FURRY = P.FURRY + ('bfurA',)
INK = (96, 70, 62)            # 외곽선: 검정 대신 옅은 갈색 잉크
EYE = (34, 24, 30)

def draw():
    cv = P.Canvas()
    # 한 덩어리 몸: 아래 왼쪽의 큰 몸에서 오른쪽 위 머리로 목 없이 이어진다(공 둘을 쌓으면 눈사람으로 읽힌다).
    def body(x, y):
        # 두 타원 사이를 매끈하게 잇는 거리장(작을수록 안쪽): 몸(34,64,30,24), 머리(54,42,20,17)
        d1 = math.hypot((x - 34) / 30, (y - 64) / 24)
        d2 = math.hypot((x - 55) / 23, (y - 39) / 19.5)
        k = 0.35
        h = max(0.0, min(1.0, 0.5 + 0.5 * (d2 - d1) / k))
        return d2 * (1 - h) + d1 * h - k * h * (1 - h)      # 부드러운 최솟값
    for y in range(88):
        for x in range(88):
            if body(x + 0.5, y + 0.5) <= 1.0:
                nx = (x + 0.5 - 44) / 34
                ny = (y + 0.5 - 50) / 30
                cv.put(x, y, 'bfurA', P.shade_level(nx, ny, 5, x, y, 0.08))
    # 배 털: 턱 밑에서 가슴까지 밝은 크림(부피가 생긴다)
    for y in range(44, 80):
        for x in range(46, 76):
            if cv.mat[y][x] and math.hypot((x + 0.5 - 62) / 12, (y + 0.5 - 60) / 14) <= 1.0:
                nx, ny = (x + 0.5 - 60) / 14, (y + 0.5 - 56) / 16
                cv.put(x, y, 'bellyA', P.shade_level(nx, ny, 4, x, y, 0.15))
    # 주둥이: 둥근 살덩이(원뿔은 당근·뿔로 읽혔다), 털에서 분홍으로
    P.ellipse(cv, 76, 44, 9.5, 5.8, 'pinkA', bias=0.1)
    P.ellipse(cv, 84.5, 44.5, 2.8, 2.8, 'pinkA', bias=-0.15)    # 코끝
    # 정수리 한 가닥(앞으로 기운 잎)
    for dy, xs in ((-5, (49,)), (-4, (47, 48, 49)), (-3, (46, 47, 48)), (-2, (45, 46, 47, 48)), (-1, (44, 45, 46, 47, 48))):
        for x in xs:
            cv.put(x - 1, 20 + dy, 'bfurA', 4 if x <= xs[0] + 1 else 3)
    # 목도리: 머리와 몸 사이를 감은 깃(옆모습엔 목이 없다) + 매듭에서 늘어진 꼬리
    for y in range(28, 76):
        for x in range(20, 70):
            if cv.mat[y][x] is None:
                continue
            dx, dy = (x + 0.5 - 55) / 23, (y + 0.5 - 39) / 19.5
            r = math.hypot(dx, dy)
            if 1.0 <= r <= 1.42 and y > 32 and x < 68:
                lvl = 3 if dy < -0.2 else (2 if dy < 0.55 else 1)
                if (int(math.degrees(math.atan2(dy, dx))) // 14) % 2 == 0:
                    lvl = max(0, lvl - 1)                       # 뜨개 골
                cv.put(x, y, 'scarfA', lvl)
    for y in range(58, 76):                                    # 꼬리 하나(가슴 앞으로, 짧게) + 끝 술
        cx = 47 - (y - 58) * 0.45
        for x in range(int(cx - 3), int(cx + 3) + 1):
            cv.put(x, y, 'scarfA', 2 if x < cx else 1)
    for k in range(4):
        cv.put(int(47 - 18 * 0.45) - 3 + k * 2, 76, 'scarfA', 0)
        cv.put(int(47 - 18 * 0.45) - 3 + k * 2, 77, 'scarfA', 1)
    # 앞발: 가슴 앞에 든 큰 삽 발(분홍) + 앞을 보는 짧고 평평한 발톱 넷
    P.ellipse(cv, 63, 66, 9, 7.5, 'pinkA', bias=0.08)
    for k in range(4):
        y0 = 59 + k * 3
        for dx, lv in ((0, 2), (1, 2), (2, 1)):
            cv.put(72 + dx - (1 if k in (0, 3) else 0), y0, 'clawA', lv)
            cv.put(72 + dx - (1 if k in (0, 3) else 0), y0 + 1, 'clawA', max(0, lv - 1))
    P.fur_clumps(cv)
    P.silhouette_tufts(cv)
    # 수채의 "고인 물감" 테두리: 그늘 쪽 단 경계(밝은 단 → 어두운 단으로 넘어가는 칸)를 한 단 더 어둡게
    marks = []
    for y in range(1, 87):
        for x in range(1, 87):
            m = cv.mat[y][x]
            if not m or m[0] != 'bfurA':
                continue
            for ddx, ddy in ((-1, 0), (0, -1)):
                n = cv.mat[y + ddy][x + ddx]
                if n and n[0] == 'bfurA' and n[1] > m[1] and m[1] >= 1:
                    marks.append((x, y, m[1] - 1))
                    break
    for x, y, l in marks:
        cv.mat[y][x] = ('bfurA', l)
    # 외곽선: 그늘 쪽만 옅은 갈색 잉크, 빛 쪽(왼쪽·위)은 털 둘째 단(지워진 듯한 가장자리)
    out = []
    for y in range(88):
        for x in range(88):
            if cv.mat[y][x] is not None:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < 88 and 0 <= ny < 88 and cv.mat[ny][nx] is not None:
                    mat, lvl = cv.mat[ny][nx]
                    lit = (dx, dy) in ((1, 0), (0, 1))
                    out.append((x, y, P.RAMP[mat][min(len(P.RAMP[mat]) - 1, 2)] if lit else INK))
                    break
    for x, y, c in out:
        cv.mat[y][x] = ('outline', 0)
        cv.col[y][x] = c
    # 눈: 작은 검은 구슬 + 반사 하나. 볼 홍조, 입은 코 밑 짧은 선
    eye = [(x, y) for y in range(31, 36) for x in range(59, 64) if (x - 61) ** 2 / 6.5 + (y - 33) ** 2 / 5.0 <= 1.0]
    for (x, y) in eye:
        cv.paint(x, y, EYE)
    for (x, y) in ((60, 32), (61, 32), (60, 33)):
        cv.paint(x, y, (255, 255, 255))                        # 큰 반사
    cv.paint(62, 35, (120, 104, 130))                          # 아래 옅은 반사
    for (x, y) in ((58, 29), (59, 29), (60, 28), (61, 28), (62, 28), (63, 29)):
        cv.paint(x, y, P.RAMP['bfurA'][0])                     # 눈 위 짧은 털결(눈썹 대신)
    for (x, y) in ((63, 40), (64, 40), (65, 40), (63, 41), (64, 41)):
        cv.paint(x, y, (232, 150, 150))
    for (x, y) in ((79, 50), (80, 50), (81, 49)):
        cv.paint(x, y, (150, 96, 92))
    return cv

cv = draw()
im = P.to_image(cv)
out = sys.argv[1]
im.save(out)
