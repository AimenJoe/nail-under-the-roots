"""대화창 초상화 생성기(docs/decisions/0015). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/portraits.py [출력 경로]  (기본 assets/art/portraits.png)
칸 88×88, 게임 픽셀의 2배 밀도(대화창의 44×44 칸에 찍는다 → 화면 배율 4면 원본 1px = 화면 2px).
  칸 0~3: Naïl 보통·기쁨·걱정·결의
  칸 4  : 변해 버린 이웃(아직 정체를 모름: 그림자 + 빛나는 눈 + 뿌리)

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
}
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
    out = Image.new('RGBA', (W * len(frames), H), (0, 0, 0, 0))
    for i, cv in enumerate(frames):
        out.paste(to_image(cv), (i * W, 0))
    return out


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/portraits.png')
