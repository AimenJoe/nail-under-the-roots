"""대화창 초상화 생성기(docs/decisions/0015). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/portraits.py [출력 경로]  (기본 assets/art/portraits.png)
칸 88×88, 게임 픽셀의 2배 밀도(대화창의 44×44 칸에 찍는다 → 화면 배율 4면 원본 1px = 화면 2px).
  칸 0~3: Naïl 보통·기쁨·걱정·결의
  칸 4  : 변해 버린 이웃(아직 정체를 모름: 그림자 + 빛나는 눈 + 뿌리)
  칸 5~6: 라쿤 포포(튜토리얼 안내) 보통·기쁨
  칸 7~8: 모모(굴토끼, 히로인) 물든 모습 — 차가움·괴로움(docs/decisions/0029)
  칸 9  : 들쥐 대장(튜토리얼 보스) 물든 모습 — 노려봄(docs/decisions/0032)
  칸 10~11: 돌아온 보리(같은 들쥐) 보통·기쁨. 흉터는 남는다

그리는 법(손 도트를 흉내 낸 절차):
  1. 부위를 타원으로 놓고 칸마다 재질을 정한다(뒤에 놓인 것부터).
  2. 재질마다 4~5단 색 경사(ramp). 타원을 구로 보고 왼쪽 위 빛으로 단을 고른다. 단 경계만 2×2 바둑판 디더.
  3. 털 결: 머리 윗선을 들쭉날쭉하게(털 뭉치), 등 쪽에 짧은 어두운 결.
  4. 선택 외곽선(sel-out): 바깥선은 이웃 재질의 가장 어두운 색, 빛 받는 쪽은 한 단 밝게. 검은 선을 피한다.
  5. 눈: 큰 흰 반사 + 작은 반사 + 아래쪽 푸른 반사광. 볼 홍조는 디더.
"""
import math
import sys

from PIL import Image

W = H = 88
LIGHT = (-0.55, -0.65, 0.52)
_l = math.sqrt(sum(c * c for c in LIGHT))
LIGHT = tuple(c / _l for c in LIGHT)

# ---- 재질별 색 경사(어두움 → 밝음). 게임 스프라이트(tools/art/nail.py) 팔레트를 늘린 것 ----
RAMP = {
    'fur':    [(58, 38, 42), (88, 60, 54), (122, 88, 72), (152, 114, 90), (184, 146, 116)],
    'muzzle': [(150, 112, 92), (184, 146, 118), (212, 180, 150), (234, 210, 182)],
    'pink':   [(160, 78, 100), (206, 112, 132), (240, 150, 164), (255, 198, 204)],
    'claw':   [(176, 160, 138), (214, 200, 176), (244, 236, 216), (255, 252, 240)],
    'eye':    [(22, 14, 30), (40, 28, 54), (70, 66, 112)],
    'shade':  [(26, 18, 36), (44, 30, 58), (66, 44, 82), (92, 62, 108)],     # 이웃 그림자 몸
    'root':   [(44, 30, 28), (78, 54, 44), (112, 80, 60), (146, 110, 80)],
    'glow':   [(150, 200, 60), (210, 240, 110), (250, 255, 200)],
    # 라쿤: 회색 털, 흰 얼굴, 검은 가면, 코
    'grey':   [(56, 54, 66), (88, 86, 100), (122, 120, 134), (156, 154, 166), (190, 188, 198)],
    'white':  [(170, 166, 172), (204, 200, 204), (230, 228, 228), (248, 246, 244)],
    'mask':   [(22, 18, 28), (36, 32, 44), (54, 50, 64), (74, 70, 86)],
    'nose':   [(18, 14, 22), (40, 34, 46), (84, 78, 96)],
    # 모모(물든 굴토끼): 원래 따뜻한 회갈색 털이 뿌리 독으로 자두빛·잿빛이 됐다. 귀 안쪽도 분홍이 아니라 어두운 보라.
    'mfur':   [(36, 26, 44), (58, 42, 66), (84, 62, 88), (112, 86, 112), (142, 114, 138)],
    'mlight': [(92, 70, 96), (116, 92, 118), (142, 116, 140), (166, 142, 162)],
    'mear':   [(50, 28, 56), (76, 42, 80), (104, 62, 104)],
    'vroot':  [(40, 20, 52), (70, 38, 88), (108, 62, 132), (150, 96, 174)],
    'mflower': [(96, 62, 78), (130, 88, 104), (162, 118, 130), (188, 150, 156)],
    # 보리(늙은 들쥐 대장): 붉은 기 도는 갈색. 물들면 자두빛으로 기울고 어두워진다(critters.py taint와 같은 방향).
    'rfur':   [(62, 40, 34), (98, 66, 50), (134, 94, 70), (170, 128, 94), (204, 164, 124)],
    'tfur':   [(42, 28, 40), (66, 46, 58), (92, 66, 76), (118, 88, 96), (144, 112, 118)],
    'rmuz':   [(150, 118, 100), (188, 158, 130), (216, 192, 160), (238, 220, 192)],
    'tmuz':   [(104, 84, 96), (132, 110, 120), (158, 136, 144), (182, 162, 166)],
    'scar':   [(150, 110, 104), (196, 160, 150), (226, 200, 188)],
}
RED_EYE = [(80, 14, 34), (190, 34, 72), (255, 84, 116), (255, 222, 230)]
GLOW_EYE = [(40, 70, 36), (110, 190, 70), (190, 245, 120), (240, 255, 210)]
OUTLINE_BASE = (43, 29, 42)
BLUSH = (236, 120, 128)
MOUTH = (96, 52, 60)
WHITE = (255, 255, 255)
BAYER2 = [[0.0, 0.5], [0.75, 0.25]]


class Canvas:
    def __init__(self):
        self.mat = [[None] * W for _ in range(H)]     # (재질, 단) 또는 None
        self.col = [[None] * W for _ in range(H)]     # 최종 색(덮어 그린 것)

    def put(self, x, y, mat, level):
        if 0 <= x < W and 0 <= y < H:
            self.mat[y][x] = (mat, level)
            self.col[y][x] = None

    def paint(self, x, y, rgb):
        if 0 <= x < W and 0 <= y < H:
            self.col[y][x] = rgb


def shade_level(nx, ny, n_levels, x, y, bias=0.0):
    """구 표면 법선으로 빛 세기 → 단. 경계는 2×2 디더."""
    nz = math.sqrt(max(0.0, 1.0 - nx * nx - ny * ny))
    i = nx * LIGHT[0] + ny * LIGHT[1] + nz * LIGHT[2]
    i = max(0.0, min(1.0, i * 0.8 + 0.12 + bias))
    v = i * (n_levels - 1)
    lo = int(v)
    frac = v - lo
    if lo >= n_levels - 1:
        return n_levels - 1
    return lo + (1 if frac > BAYER2[y % 2][x % 2] * 0.6 + 0.2 else 0)


def ellipse(cv, cx, cy, rx, ry, mat, bias=0.0, rough=None, clip=None):
    """구처럼 음영을 넣은 타원. rough(x,y)->반지름 배율로 가장자리를 들쭉날쭉하게."""
    levels = len(RAMP[mat])
    for y in range(int(cy - ry - 3), int(cy + ry + 4)):
        for x in range(int(cx - rx - 3), int(cx + rx + 4)):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            r2 = dx * dx + dy * dy
            lim = rough(x, y, math.atan2(dy, dx)) if rough else 1.0
            if r2 <= lim * lim and (clip is None or clip(x, y)):
                d = math.sqrt(r2) or 1.0
                k = min(1.0, math.sqrt(r2))
                cv.put(x, y, mat, shade_level(dx / d * k, dy / d * k, levels, x, y, bias))


def flat(cv, cx, cy, rx, ry, mat, level):
    for y in range(int(cy - ry - 1), int(cy + ry + 2)):
        for x in range(int(cx - rx - 1), int(cx + rx + 2)):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            if dx * dx + dy * dy <= 1.0:
                cv.put(x, y, mat, level)


def line(cv, pts, rgb, width=1):
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for i in range(n + 1):
            t = i / n
            x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            for oy in range(width):
                for ox in range(width):
                    cv.paint(int(round(x - width / 2 + ox + 0.5)), int(round(y - width / 2 + oy + 0.5)), rgb)


def dither_ellipse(cv, cx, cy, rx, ry, rgb, density=0.5):
    for y in range(int(cy - ry), int(cy + ry) + 1):
        for x in range(int(cx - rx), int(cx + rx) + 1):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            r2 = dx * dx + dy * dy
            if r2 <= 1.0 and cv.mat[y][x] is not None:
                core = r2 < 0.35
                if core or BAYER2[y % 2][x % 2] < density:
                    cv.paint(x, y, rgb)


def eye(cv, cx, cy, rx, ry, look=(0.6, 0.0)):
    """큰 눈: 짙은 자두색 → 아래쪽 푸른 반사광, 흰 반사 둘."""
    for y in range(int(cy - ry - 1), int(cy + ry + 2)):
        for x in range(int(cx - rx - 1), int(cx + rx + 2)):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            if dx * dx + dy * dy <= 1.0:
                lvl = 0 if dy < 0.25 else (1 if dy < 0.6 else 2)
                cv.paint(x, y, RAMP['eye'][lvl])
    hx, hy = cx - rx * 0.35 + look[0], cy - ry * 0.35 + look[1]
    for y in range(int(hy - 2), int(hy + 2)):
        for x in range(int(hx - 1.5), int(hx + 1.5)):
            if (x + 0.5 - hx) ** 2 + (y + 0.5 - hy) ** 2 <= 3.2:
                cv.paint(x, y, WHITE)
    cv.paint(int(cx + rx * 0.35 + look[0]), int(cy + ry * 0.4), WHITE)


def happy_eye(cv, cx, cy, rx):
    """^ 모양으로 감은 눈."""
    pts = [(cx - rx, cy + 1.5), (cx - rx * 0.5, cy - 1.5), (cx, cy - 2.5), (cx + rx * 0.5, cy - 1.5), (cx + rx, cy + 1.5)]
    line(cv, pts, RAMP['eye'][0], 2)


def outline(cv):
    """선택 외곽선: 빈 칸이 채운 칸과 닿으면 그 재질의 가장 어두운 색(빛 쪽은 한 단 밝게)."""
    out = []
    for y in range(H):
        for x in range(W):
            if cv.mat[y][x] is not None:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < W and 0 <= ny < H and cv.mat[ny][nx] is not None:
                    mat = cv.mat[ny][nx][0]
                    ramp = RAMP[mat]
                    lit = (dx, dy) in ((1, 0), (0, 1))   # 빈 칸이 왼쪽/위 → 빛 받는 쪽
                    c = ramp[1] if lit and len(ramp) > 3 else ramp[0]
                    if mat in ('fur', 'shade', 'root') and not lit:
                        c = OUTLINE_BASE if mat == 'fur' else ramp[0]
                    out.append((x, y, c))
                    break
    for x, y, c in out:
        cv.mat[y][x] = ('outline', 0)
        cv.col[y][x] = c


def inner_edges(cv, mats=('muzzle', 'pink', 'claw')):
    """다른 재질끼리 만나는 곳(주둥이·발·코 경계)에 아래쪽 한 단 어두운 선."""
    marks = []
    for y in range(1, H):
        for x in range(W):
            a, b = cv.mat[y][x], cv.mat[y - 1][x]
            if a and b and a[0] != b[0] and b[0] in mats and a[0] not in ('outline',) and cv.col[y][x] is None:
                marks.append((x, y, RAMP[b[0]][0]))
    for x, y, c in marks:
        cv.col[y][x] = c


def fur_strands(cv, seed):
    """등·머리 위 짧은 털 결(어두운 2~3px 사선)."""
    import random
    rnd = random.Random(seed)
    for _ in range(34):
        x = rnd.randint(14, 58)
        y = rnd.randint(22, 70)
        m = cv.mat[y][x]
        if not m or m[0] != 'fur' or cv.col[y][x] is not None:
            continue
        lvl = m[1]
        c = RAMP['fur'][max(0, lvl - 1)]
        for k in range(rnd.randint(2, 3)):
            yy, xx = y + k, x - k // 2
            mm = cv.mat[yy][xx] if 0 <= yy < H and 0 <= xx < W else None
            if mm and mm[0] == 'fur':
                cv.paint(xx, yy, c)
    # 하이라이트 털 몇 가닥(빛 쪽)
    for _ in range(14):
        x = rnd.randint(16, 44)
        y = rnd.randint(24, 46)
        m = cv.mat[y][x]
        if m and m[0] == 'fur' and m[1] >= 3:
            cv.paint(x, y, RAMP['fur'][4])
            if cv.mat[y + 1][x] and cv.mat[y + 1][x][0] == 'fur':
                cv.paint(x, y + 1, RAMP['fur'][4])


def tuft(x, y, ang):
    """머리 윗선 털 뭉치: 각도에 따라 반지름이 물결친다."""
    if ang < -0.3 and ang > -2.9:
        return 1.0 + 0.045 * math.sin(ang * 9.0) + 0.02 * math.sin(ang * 23.0)
    return 1.0


# ---- Naïl ------------------------------------------------------------------------------
def rim_light(cv, mat='fur'):
    """뒤쪽(오른쪽 아래) 가장자리 1px을 한 단 밝게: 덩어리가 배경에서 떠 보인다."""
    marks = []
    for y in range(H - 1):
        for x in range(W - 1):
            m = cv.mat[y][x]
            if not m or m[0] != mat or cv.col[y][x] is not None:
                continue
            r, d = cv.mat[y][x + 1], cv.mat[y + 1][x]
            if (r and r[0] == 'outline') or (d and d[0] == 'outline'):
                if m[1] == 0:
                    marks.append((x, y, RAMP[mat][1]))
    for x, y, c in marks:
        cv.col[y][x] = c


def nail(face):
    cv = Canvas()
    determined = face == 'determined'
    paw_y = 75 if determined else 79
    # 몸 겸 머리: 한 덩어리(두더지는 목이 없다). 위는 털 뭉치로 들쭉날쭉, 아래는 조금 넓게(배).
    def blob(x, y, ang):
        r = tuft(x, y, ang)
        if 0.3 < ang < 2.8:
            r *= 1.0 + 0.07 * math.sin(ang)
        return r
    ellipse(cv, 40, 51, 35, 31, 'fur', bias=0.08, rough=blob)
    # 배 쪽 밝은 털은 넣지 않는다: 발톱·발과 겹치면 "벌린 입과 이빨"로 읽힌다(스프라이트 v1의 교훈).
    # 짧은 주둥이 + 단추 코
    ellipse(cv, 64, 50, 10, 7.5, 'muzzle', bias=0.05)
    ellipse(cv, 73, 46, 6.5, 6, 'pink', bias=0.12)
    for (x, y) in ((70, 43), (71, 43), (70, 44)):
        cv.paint(x, y, RAMP['pink'][3])
    # 앞발 두 개(분홍 손바닥 + 크림 발톱 셋). 얼굴과 떨어진 가슴 아래.
    for (px, py, s_) in ((30, paw_y + 2, 0.85), (54, paw_y, 1.0)):
        ellipse(cv, px, py, 8.5 * s_, 7.5 * s_, 'pink', bias=-0.05)
        for k in range(3):
            cx = px - 4.6 * s_ + k * 4.6 * s_
            ellipse(cv, cx, py - 7.0 * s_, 1.6, 2.6 * s_, 'claw', bias=0.1)
    outline(cv)
    inner_edges(cv)
    rim_light(cv)
    fur_strands(cv, 7)
    # 코 아래 그늘
    for (x, y) in ((74, 51), (75, 51), (76, 50)):
        cv.paint(x, y, RAMP['pink'][0])

    # 볼 홍조(눈 아래)
    dither_ellipse(cv, 51, 49, 5.0, 2.8, BLUSH, 0.5)
    # 눈(가까운 눈 크게, 먼 눈 작게: 3/4 시점)
    if face == 'happy':
        happy_eye(cv, 52, 40, 4.6)
        happy_eye(cv, 66, 37, 3.4)
    else:
        ry = 3.6 if determined else 5.6
        cy = 41 if determined else 40
        eye(cv, 52, cy, 4.4, ry)
        eye(cv, 66, cy - 3, 3.2, ry * 0.8, look=(0.3, 0.0))
        if determined:  # 위 눈꺼풀을 반듯하게 눌러 결의
            line(cv, [(47, 37), (57, 38)], RAMP['fur'][1], 2)
            line(cv, [(63, 34), (70, 35)], RAMP['fur'][1], 2)
    if face == 'worried':
        line(cv, [(46, 31), (55, 29)], RAMP['fur'][0], 2)    # 안쪽이 올라간 걱정 눈썹
        line(cv, [(63, 28), (69, 30)], RAMP['fur'][0], 2)
        for (x, y) in ((21, 21), (21, 22), (20, 23), (21, 23), (22, 23), (20, 24), (21, 24), (22, 24), (21, 25)):
            cv.paint(x, y, (170, 220, 255) if (x, y) != (21, 21) else (230, 248, 255))
    if determined:
        line(cv, [(46, 30), (57, 33)], RAMP['fur'][0], 2)    # 안쪽이 내려간 결의 눈썹
        line(cv, [(63, 32), (70, 30)], RAMP['fur'][0], 2)
    # 입: 코 아래 작게(ω). 표정마다.
    if face == 'happy':
        for (x, y) in ((65, 54), (66, 55), (67, 55), (68, 55), (69, 54)):
            cv.paint(x, y, MOUTH)
        for (x, y, l) in ((66, 62, 2), (67, 62, 2), (68, 62, 1)):
            cv.paint(x, y, RAMP['pink'][l])
    else:
        mouth = {
            'normal': [(64, 54), (66, 55), (67, 54), (68, 55), (70, 54)],
            'worried': [(64, 55), (66, 54), (68, 55), (70, 54)],
            'determined': [(65, 55), (70, 55)],
        }[face]
        line(cv, mouth, MOUTH, 1)
    return cv


# ---- 변해 버린 이웃(???) -------------------------------------------------------------
def neighbor():
    cv = Canvas()

    def wobble(x, y, ang):
        return 1.0 + 0.06 * math.sin(ang * 5.0 + 1.0) + 0.03 * math.sin(ang * 13.0)

    ellipse(cv, 46, 56, 32, 30, 'shade', rough=wobble)
    ellipse(cv, 26, 30, 8, 11, 'shade', bias=-0.1)     # 귀 같은 것
    ellipse(cv, 64, 28, 7, 10, 'shade', bias=-0.1)
    # 몸을 감은 뿌리
    for (pts, w) in (
        ([(8, 70), (20, 62), (34, 64), (46, 58), (60, 62), (76, 54), (84, 44)], 3),
        ([(14, 84), (24, 76), (40, 78), (52, 72)], 3),
        ([(60, 20), (56, 28), (60, 36), (54, 44)], 2),
    ):
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            n = int(max(abs(x1 - x0), abs(y1 - y0))) * 2 + 1
            for i in range(n + 1):
                t = i / n
                x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
                for oy in range(-w // 2, w // 2 + 1):
                    for ox in range(-w // 2, w // 2 + 1):
                        if ox * ox + oy * oy <= (w / 2) ** 2 + 0.5:
                            lvl = 2 if oy < 0 else 1
                            cv.put(int(x + ox), int(y + oy), 'root', lvl)
    outline(cv)
    # 빛나는 눈 두 개(번짐 + 심)
    for (ex, ey) in ((36, 50), (58, 47)):
        for y in range(ey - 8, ey + 9):
            for x in range(ex - 10, ex + 11):
                d = ((x - ex) / 10) ** 2 + ((y - ey) / 8) ** 2
                if d <= 1.0 and cv.mat[y][x] and cv.mat[y][x][0] == 'shade':
                    if BAYER2[y % 2][x % 2] < (1.0 - d) * 0.9:
                        cv.paint(x, y, RAMP['shade'][3] if d > 0.35 else RAMP['glow'][0])
        flat_pts = [(ex + dx, ey + dy) for dy in range(-3, 4) for dx in range(-4, 5) if (dx / 4.4) ** 2 + (dy / 3.0) ** 2 <= 1.0]
        for (x, y) in flat_pts:
            cv.paint(x, y, RAMP['glow'][1])
        for (dx, dy) in ((-2, -1), (-1, -1), (-1, -2), (0, -1)):
            cv.paint(ex + dx, ey + dy, RAMP['glow'][2])
    # 뿌리 마디 하이라이트
    for (x, y) in ((34, 63), (35, 63), (60, 61), (61, 61), (40, 77), (57, 29)):
        cv.paint(x, y, RAMP['root'][3])
    return cv


# ---- 라쿤 포포 ------------------------------------------------------------------------
def raccoon(face):
    cv = Canvas()

    def cheeks(x, y, ang):
        # 옆으로 뻗은 볼털(양옆 아래), 머리 윗선 털 뭉치
        r = tuft(x, y, ang)
        a = abs(math.cos(ang))
        if 0.0 < ang < 1.2 or 1.9 < ang < 3.2:
            r *= 1.0 + 0.10 * a * (0.6 + 0.4 * math.sin(ang * 7.0))
        return r
    # 귀(뒤): 회색 + 흰 테두리 + 검은 안쪽
    for (ex, ey) in ((20, 22), (60, 16)):
        ellipse(cv, ex, ey, 10, 11, 'grey', bias=0.0)
        flat(cv, ex + 1, ey + 2, 6, 7, 'mask', 1)
    # 머리
    ellipse(cv, 42, 50, 34, 28, 'grey', bias=0.05, rough=cheeks)
    # 흰 이마·눈썹 무늬, 흰 주둥이
    ellipse(cv, 30, 32, 9, 4.5, 'white', bias=0.1)
    ellipse(cv, 55, 29, 8, 4.0, 'white', bias=0.1)
    ellipse(cv, 55, 60, 17, 12, 'white', bias=0.08)
    # 검은 가면: 두 눈 둘레 + 콧등으로 이어짐
    ellipse(cv, 32, 45, 13, 9, 'mask', bias=0.0)
    ellipse(cv, 57, 42, 11, 8, 'mask', bias=0.0)
    flat(cv, 45, 46, 6, 3.5, 'mask', 1)
    # 코
    ellipse(cv, 66, 54, 6, 4.2, 'nose', bias=0.1)
    cv.paint(64, 52, RAMP['nose'][2])
    cv.paint(65, 52, RAMP['nose'][2])
    outline(cv)
    rim_light(cv, 'grey')
    # 눈: 가면 속이라 테두리를 한 단 밝게(눈이 묻히지 않게)
    if face == 'happy':
        # 가면이 검어서 감은 눈은 밝은 선으로.
        for (cx, cy, rx) in ((34, 44, 4.6), (57, 41, 3.8)):
            pts = [(cx - rx, cy + 1.5), (cx - rx * 0.5, cy - 1.5), (cx, cy - 2.5), (cx + rx * 0.5, cy - 1.5), (cx + rx, cy + 1.5)]
            line(cv, pts, RAMP['white'][2], 2)
    else:
        for (ex, ey, rx, ry) in ((34, 44, 5.0, 5.8), (57, 41, 4.0, 4.8)):
            dither_ellipse(cv, ex, ey, rx + 1.6, ry + 1.4, RAMP['mask'][3], 0.6)
        eye(cv, 34, 44, 4.6, 5.4)
        eye(cv, 57, 41, 3.6, 4.4, look=(0.3, 0.0))
    # 볼 홍조, 입
    dither_ellipse(cv, 46, 61, 5.0, 2.6, BLUSH, 0.5)
    if face == 'happy':
        for (x, y) in ((60, 63), (61, 64), (62, 64), (63, 64), (64, 63)):
            cv.paint(x, y, MOUTH)
        for (x, y) in ((61, 65), (62, 66), (63, 65)):
            cv.paint(x, y, RAMP['pink'][2])
    else:
        line(cv, [(59, 63), (61, 64), (63, 63), (65, 64)], MOUTH, 1)
    # 털 결(볼·이마)
    for (x, y) in ((14, 56), (15, 57), (12, 60), (70, 66), (71, 67), (24, 70), (25, 71)):
        m = cv.mat[y][x]
        if m and m[0] == 'grey':
            cv.paint(x, y, RAMP['grey'][1])
    return cv


# ---- 모모(물든 굴토끼) -------------------------------------------------------------------
def momo_tainted(face):
    """물든 모모: 긴 귀 둘(먼 귀는 뒤로 눕고 가까운 귀는 서서 뿌리가 감음), 동그란 얼굴과 볼털, 작은 주둥이,
    초록으로 빛나는 큰 눈(졸개의 붉은 눈과 다르다), 눈 밑 그늘, 가까운 귀 밑동에 시든 꽃 장식(돌아오면 핀다).
    face: 'cold'(차갑게 노려봄) / 'pained'(괴로움, 눈물)."""
    cv = Canvas()

    def fluff(x, y, ang):
        # 볼털이 양옆 아래로 삐죽, 머리 위는 털 뭉치
        r = tuft(x, y, ang)
        if 0.2 < ang < 1.3 or 1.85 < ang < 2.95:
            r *= 1.0 + 0.09 * (0.5 + 0.5 * math.sin(ang * 9.0))
        return r
    # 먼 귀: 뒤로 비스듬히 눕는다
    ellipse(cv, 26, 22, 7, 17, 'mfur', bias=-0.08, clip=lambda x, y: y < 40)
    flat(cv, 27, 22, 3.5, 12, 'mear', 1)
    # 가까운 귀: 똑바로 서고 끝이 살짝 접힌다
    ellipse(cv, 50, 18, 8, 18, 'mfur', bias=0.04, clip=lambda x, y: y < 40)
    flat(cv, 51, 19, 4.0, 13, 'mear', 2)
    for (x, y) in ((49, 6), (50, 5), (51, 5), (52, 6)):
        cv.put(x, y, 'mfur', 1)
    # 머리 + 볼털
    ellipse(cv, 42, 54, 29, 24, 'mfur', bias=0.05, rough=fluff)
    # 어깨(아래로 잘림)
    ellipse(cv, 40, 92, 22, 10, 'mfur', bias=-0.15)
    # 작은 주둥이·턱(밝은 털), 작은 코
    ellipse(cv, 61, 62, 5.5, 4.5, 'mlight', bias=-0.05)
    ellipse(cv, 66, 58, 2.4, 1.9, 'pink', bias=-0.2)
    # 몸을 감은 뿌리: 목을 한 바퀴, 가까운 귀를 타고 오른다
    for (pts, w) in (
        ([(10, 76), (22, 72), (36, 75), (50, 72), (64, 74), (76, 70)], 3),
        ([(44, 38), (47, 30), (53, 24), (49, 16), (54, 9)], 2),
        ([(18, 40), (24, 34), (22, 26)], 2),
    ):
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            n = int(max(abs(x1 - x0), abs(y1 - y0))) * 2 + 1
            for i in range(n + 1):
                t = i / n
                x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
                for oy in range(-(w // 2), w // 2 + 1):
                    for ox in range(-(w // 2), w // 2 + 1):
                        if ox * ox + oy * oy <= (w / 2) ** 2 + 0.5:
                            cv.put(int(x + ox), int(y + oy), 'vroot', 2 if oy < 0 else 1)
    outline(cv)
    rim_light(cv, 'mfur')
    # 뿌리 가시·마디
    for (x, y) in ((22, 71), (36, 74), (50, 71), (64, 73), (47, 29), (52, 23)):
        cv.paint(x, y, RAMP['vroot'][3])
    for (x, y) in ((28, 69), (57, 70), (55, 18)):
        cv.paint(x, y, RAMP['vroot'][0])
    # 시든 꽃(가까운 귀 밑동): 고개 숙인 꽃 한 송이. 꽃잎 다섯이 아래로 처지고 색이 바랬다(돌아오면 분홍으로 핀다).
    fx, fy = 52, 24
    for (cx, cy) in ((-4, 1), (4, 1), (-3, 5), (3, 5), (0, 7)):
        flat(cv, fx + cx, fy + cy, 2.6, 2.2, 'mflower', 2)
        cv.paint(fx + cx - 1, fy + cy - 1, RAMP['mflower'][3])
        cv.paint(fx + cx + 1, fy + cy + 1, RAMP['mflower'][1])
    for (ox, oy) in ((-1, 2), (0, 2), (1, 2), (-1, 3), (0, 3), (1, 3), (0, 4)):
        cv.paint(fx + ox, fy + oy, (150, 138, 82))       # 꽃술(빛 바랜 노랑)
    for (x, y) in ((fx - 6, fy + 2), (fx + 6, fy + 2), (fx - 4, fy + 7), (fx + 4, fy + 7), (fx, fy + 9)):
        cv.paint(x, y, RAMP['mflower'][0])               # 처진 꽃잎 끝
    for (x, y) in ((fx, fy - 2), (fx + 1, fy - 3), (fx + 1, fy - 4)):
        cv.paint(x, y, (96, 110, 66))                    # 마른 줄기
    # 눈 밑 그늘(지친 기색)
    dither_ellipse(cv, 39, 59, 6, 2.0, RAMP['mfur'][1], 0.45)
    dither_ellipse(cv, 58, 56, 4.5, 1.6, RAMP['mfur'][1], 0.45)
    # 눈: 크고 둥근 눈. 물든 표시는 초록 빛 홍채로만(동공은 둥글게, 반사는 크게 — 귀여움은 남긴다).
    for (ex, ey, rx, ry, outer) in ((39, 48, 6.0, 7.2, -1), (58, 45, 4.6, 6.0, 1)):
        for y in range(int(ey - ry - 1), int(ey + ry + 2)):
            for x in range(int(ex - rx - 1), int(ex + rx + 2)):
                dx, dy = (x + 0.5 - ex) / rx, (y + 0.5 - ey) / ry
                d = dx * dx + dy * dy
                if d <= 1.0:
                    if d > 0.78:
                        c = GLOW_EYE[0]
                    elif d < 0.2:
                        c = RAMP['eye'][0]                       # 둥근 동공
                    else:
                        c = GLOW_EYE[1] if dy < 0.15 else GLOW_EYE[2]
                    cv.paint(x, y, c)
        # 반사: 큰 것 2×2 + 작은 것
        hx, hy = int(ex - rx * 0.38), int(ey - ry * 0.42)
        for (ox, oy) in ((0, 0), (1, 0), (0, 1), (1, 1)):
            cv.paint(hx + ox, hy + oy, WHITE)
        cv.paint(int(ex + rx * 0.35), int(ey + ry * 0.4), GLOW_EYE[3])
        # 속눈썹: 바깥 눈꼬리 위로 두 가닥
        bx = int(ex + outer * rx * 0.9)
        by = int(ey - ry * 0.6)
        cv.paint(bx, by, RAMP['eye'][0])
        cv.paint(bx + outer, by - 1, RAMP['eye'][0])
        cv.paint(bx + outer, by + 1, RAMP['eye'][0])
        cv.paint(bx + 2 * outer, by, RAMP['eye'][0])
        # 윗눈꺼풀 선
        for x in range(int(ex - rx * 0.8), int(ex + rx * 0.8) + 1):
            cv.paint(x, int(ey - ry) , RAMP['mfur'][0])
    # 눈썹과 입: 차가움 = 평평한 눈썹·꾹 다문 작은 입, 괴로움 = 안쪽이 올라간 눈썹·떨리는 입·눈물
    if face == 'cold':
        line(cv, [(33, 37), (44, 38)], RAMP['mfur'][0], 2)
        line(cv, [(54, 35), (62, 35)], RAMP['mfur'][0], 2)
        line(cv, [(60, 67), (63, 68), (66, 67)], MOUTH, 1)
    else:
        line(cv, [(33, 38), (44, 35)], RAMP['mfur'][0], 2)
        line(cv, [(54, 34), (62, 36)], RAMP['mfur'][0], 2)
        line(cv, [(59, 68), (61, 67), (63, 68), (65, 67)], MOUTH, 1)
        for (x, y) in ((44, 55), (44, 56), (45, 57), (45, 58), (45, 59), (46, 60)):
            cv.paint(x, y, (170, 220, 255) if y > 55 else (230, 248, 255))
    # 볼 홍조(옅게, 물들어도 남은 원래 모습의 흔적)
    dither_ellipse(cv, 49, 61, 4.5, 2.2, (150, 92, 118), 0.35)
    # 초록 빛 번짐(눈가)
    for (ex, ey) in ((39, 48), (58, 45)):
        for y in range(ey - 10, ey + 11):
            for x in range(ex - 11, ex + 12):
                dd = ((x - ex) / 11) ** 2 + ((y - ey) / 10) ** 2
                if 0.5 < dd <= 1.0 and cv.mat[y][x] and cv.mat[y][x][0] == 'mfur' and cv.col[y][x] is None:
                    if BAYER2[y % 2][x % 2] < (1.0 - dd) * 0.45:
                        cv.paint(x, y, RAMP['mfur'][3])
    return cv


# ---- 들쥐 대장 / 보리 ---------------------------------------------------------------------
def bori(face):
    """들쥐 대장(보리): 크고 둥근 귀(먼 귀는 찢겨 이가 빠짐), 길쭉한 주둥이와 앞니, 가까운 눈을 지나는 흉터.
    face: 'tainted'(물듦: 붉은 눈, 머리 위 뿌리 왕관, 찌푸림) / 'normal' / 'happy'(돌아온 보리)."""
    cv = Canvas()
    tainted = face == 'tainted'
    fur, muz = ('tfur', 'tmuz') if tainted else ('rfur', 'rmuz')
    # 먼 귀(뒤, 왼쪽 위): 가장자리가 찢겨 V자로 빠졌다.
    ellipse(cv, 22, 22, 12, 13, fur, bias=-0.1)
    flat(cv, 23, 23, 7, 8, 'pink', 1)
    for y in range(8, 16):
        for x in range(17 - (y - 8) // 2, 19 + (y - 8) // 2):
            if y < 13 or abs(x - 18) < 15 - y:
                cv.mat[y][x] = None
    # 가까운 귀(오른쪽 위): 크고 둥글다.
    ellipse(cv, 60, 18, 14, 15, fur, bias=0.06)
    flat(cv, 61, 20, 9, 10, 'pink', 2)
    # 머리: 볼털이 아래로 삐죽.

    def cheeks(x, y, ang):
        r = tuft(x, y, ang)
        if 0.3 < ang < 1.4 or 1.8 < ang < 2.9:
            r *= 1.0 + 0.08 * (0.5 + 0.5 * math.sin(ang * 8.0))
        return r
    ellipse(cv, 38, 54, 31, 26, fur, bias=0.05, rough=cheeks)
    # 어깨(아래로 잘림)
    ellipse(cv, 34, 94, 26, 12, fur, bias=-0.15)
    # 길쭉한 주둥이와 코
    ellipse(cv, 62, 60, 17, 11, muz, bias=0.05)
    ellipse(cv, 77, 55, 4.2, 3.4, 'pink', bias=-0.1)
    if tainted:
        # 머리 위 뿌리 왕관: 가지 셋이 위로 솟는다.
        for (pts, w) in (
            ([(30, 32), (28, 22), (24, 12), (26, 5)], 3),
            ([(38, 30), (40, 20), (38, 10)], 2),
            ([(46, 32), (50, 24), (48, 16)], 2),
            ([(24, 12), (19, 9)], 2),
            ([(10, 60), (18, 66), (16, 76)], 2),
        ):
            for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
                n = int(max(abs(x1 - x0), abs(y1 - y0))) * 2 + 1
                for i in range(n + 1):
                    t = i / n
                    x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
                    for oy in range(-(w // 2), w // 2 + 1):
                        for ox in range(-(w // 2), w // 2 + 1):
                            if ox * ox + oy * oy <= (w / 2) ** 2 + 0.5:
                                cv.put(int(x + ox), int(y + oy), 'vroot', 2 if ox < 0 else 1)
    outline(cv)
    rim_light(cv, fur)
    if tainted:
        for (x, y) in ((26, 6), (38, 11), (48, 17), (28, 20), (16, 70)):
            cv.paint(x, y, RAMP['vroot'][3])
    # 앞니(쥐): 코 아래 흰 두 칸.
    for (x, y) in ((70, 66), (71, 66), (72, 66), (70, 67), (71, 67), (72, 67), (70, 68), (72, 68)):
        cv.paint(x, y, RAMP['claw'][2])
    cv.paint(71, 68, RAMP['claw'][0])
    # 수염
    for pts in (((72, 58), (84, 55)), ((73, 61), (86, 61)), ((72, 64), (84, 67))):
        line(cv, list(pts), RAMP[muz][3], 1)
    # 눈
    eyes = ((40, 44, 5.4, 6.4), (58, 41, 4.2, 5.2))
    if tainted:
        for (ex, ey, rx, ry) in eyes:
            for y in range(int(ey - ry - 1), int(ey + ry + 2)):
                for x in range(int(ex - rx - 1), int(ex + rx + 2)):
                    dx, dy = (x + 0.5 - ex) / rx, (y + 0.5 - ey) / ry
                    d = dx * dx + dy * dy
                    if d <= 1.0:
                        cv.paint(x, y, RED_EYE[0] if d > 0.7 else (RED_EYE[1] if dy > 0.1 else RED_EYE[2]))
            cv.paint(int(ex - rx * 0.4), int(ey - ry * 0.4), RED_EYE[3])
            cv.paint(int(ex - rx * 0.4) + 1, int(ey - ry * 0.4), RED_EYE[3])
        # 찌푸린 눈썹(안쪽이 내려감)
        line(cv, [(32, 33), (46, 38)], RAMP[fur][0], 2)
        line(cv, [(53, 35), (63, 32)], RAMP[fur][0], 2)
        # 이를 드러낸 입
        line(cv, [(60, 70), (64, 71), (69, 70)], MOUTH, 1)
    elif face == 'happy':
        happy_eye(cv, 40, 45, 5.0)
        happy_eye(cv, 58, 42, 3.8)
        dither_ellipse(cv, 48, 60, 5.0, 2.6, BLUSH, 0.5)
        for (x, y) in ((60, 70), (61, 71), (62, 72), (63, 72), (64, 72), (65, 71), (66, 70)):
            cv.paint(x, y, MOUTH)
        for (x, y) in ((62, 73), (63, 73), (64, 73)):
            cv.paint(x, y, RAMP['pink'][2])
    else:
        eye(cv, 40, 44, 5.0, 6.0)
        eye(cv, 58, 41, 3.8, 4.8, look=(0.3, 0.0))
        line(cv, [(33, 35), (46, 35)], RAMP[fur][1], 2)
        line(cv, [(53, 33), (62, 33)], RAMP[fur][1], 2)
        dither_ellipse(cv, 48, 60, 4.5, 2.2, BLUSH, 0.35)
        line(cv, [(61, 71), (64, 72), (67, 71)], MOUTH, 1)
    # 흉터: 가까운 눈 위에서 아래로 비스듬히(눈은 비켜 간다).
    for (x, y) in ((33, 30), (34, 32), (35, 34), (36, 36), (44, 52), (45, 54), (46, 56)):
        cv.paint(x, y, RAMP['scar'][2])
        cv.paint(x + 1, y + 1, RAMP['scar'][0])
    return cv


def to_image(cv):
    img = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    px = img.load()
    for y in range(H):
        for x in range(W):
            c = cv.col[y][x]
            if c is None and cv.mat[y][x] is not None:
                mat, lvl = cv.mat[y][x]
                c = RAMP[mat][lvl] if mat in RAMP else OUTLINE_BASE
            if c is not None:
                px[x, y] = c + (255,)
    return img


def sheet():
    frames = [nail(f) for f in ('normal', 'happy', 'worried', 'determined')] + [neighbor()]
    frames += [raccoon('normal'), raccoon('happy')]
    frames += [momo_tainted('cold'), momo_tainted('pained')]
    frames += [bori('tainted'), bori('normal'), bori('happy')]
    out = Image.new('RGBA', (W * len(frames), H), (0, 0, 0, 0))
    for i, cv in enumerate(frames):
        out.paste(to_image(cv), (i * W, 0))
    return out


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/portraits.png')
