"""대화창 초상화 시트 만들기(docs/decisions/0015·0108). PNG는 산출물이다.

실행: python3 tools/art/portraits.py [출력 경로]  (기본 assets/art/portraits.png)
칸 88×88(한 줄 12칸, 칸 번호는 왼쪽 위부터), 게임 픽셀의 2배 밀도(대화창의 44×44 칸에 찍는다). 칸 번호 ↔ 인물은 ai_portraits.py의 표.
칸 4(변해 버린 이웃: 그림자 + 빛나는 눈 + 뿌리)만 코드로 그리고, 그 밖의 칸은 모두 나노바나나 원본(tools/art/sources/ai/)을
ai_portraits.py가 바꿔 끼운다(0108~0122, 0165). 예전 코드 그림(Naïl·포포·모모·보리·할매·엄마·도토·수수·콩이, v1·v2·v4)은
tools/art/legacy/portraits.py에 그대로 있다(0169 R8).
비교 시험: NAIL_ART_V4=0 NAIL_ART_V2=1(v2), NAIL_ART_V4=0(v1) — 칸 4에만 영향.
"""
import colorsys
import math
import os
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
    # v2 캐릭터 해석: Naïl의 뜨개 목도리(엄마가 떠 준 것, 따뜻한 빨강), 포포의 노란 연필(깎은 나무 끝·심)
    'scarf':  [(104, 34, 44), (150, 52, 54), (196, 82, 66), (228, 122, 86), (246, 166, 120)],
    'twig':   [(70, 46, 34), (112, 78, 52), (152, 112, 74), (190, 150, 100)],
    'pencil': [(134, 92, 30), (196, 146, 50), (236, 192, 78), (252, 228, 140)],
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
    # 돌아온 모모: 원래 털(critters.py momo_bloom과 같은 계열), 밝은 주둥이, 분홍 귀 안, 핀 꽃
    'bfur':   [(66, 50, 46), (104, 84, 74), (146, 120, 102), (184, 158, 134), (218, 196, 172)],
    'blight': [(176, 156, 136), (204, 186, 166), (228, 214, 196), (246, 238, 224)],
    'bloom':  [(196, 86, 126), (236, 126, 166), (255, 168, 198), (255, 216, 230)],
    # 누룩 할매: 흰 실 덩어리(옅은 보랏빛 흰색), 버섯 갓, 버섯 대
    'thread': [(112, 104, 134), (150, 144, 174), (188, 184, 208), (218, 216, 234), (242, 240, 250)],
    'cap':    [(84, 56, 40), (126, 88, 56), (170, 126, 78), (206, 166, 108)],
    'stem':   [(170, 158, 140), (210, 200, 182), (236, 230, 216)],
    # 엄마 두더지: Naïl보다 한 단 짙은 털
    'mfurd':  [(46, 30, 34), (72, 50, 46), (102, 74, 62), (132, 98, 80), (162, 128, 104)],
    # 도토(청설모): 붉은 밤색 털, 크림빛 배·주둥이. 물들면 자두빛으로 기운다(모모·보리와 같은 방향).
    'dfur':   [(70, 36, 30), (112, 62, 42), (160, 96, 60), (200, 134, 86), (230, 170, 116)],
    'dtfur':  [(48, 28, 40), (80, 46, 54), (112, 66, 66), (146, 92, 82), (176, 120, 104)],
    'dlight': [(170, 140, 120), (206, 180, 156), (232, 212, 190), (248, 238, 222)],
    'dtlight': [(120, 96, 108), (150, 124, 132), (176, 152, 156), (200, 180, 180)],
    'acorn':  [(62, 40, 26), (98, 66, 40), (138, 98, 60), (176, 134, 84)],
    # 수수(땅강아지): 벨벳 갈색 몸, 더 짙은 삽발, 놋쇠 보안경 테와 하늘빛 알.
    'sfur':   [(48, 32, 28), (76, 54, 40), (108, 80, 58), (142, 108, 76), (176, 140, 100)],
    'shovel': [(40, 26, 22), (70, 46, 34), (104, 72, 50), (140, 104, 72)],
    'brass':  [(120, 84, 36), (176, 132, 60), (224, 184, 96)],
    'lens':   [(90, 140, 160), (150, 200, 214), (220, 244, 250)],
    # 콩이(아르마딜로): 등딱지·머리 방패(잿빛 황갈색), 맨살(분홍빛 회색). 물들면 자두빛으로.
    'kshell': [(70, 60, 56), (104, 92, 84), (140, 126, 112), (176, 160, 140), (206, 192, 170)],
    'kskin':  [(150, 110, 110), (186, 146, 140), (214, 180, 170), (236, 214, 204)],
    'ktshell': [(52, 44, 58), (80, 68, 84), (108, 94, 108), (138, 122, 134), (166, 152, 160)],
    'ktskin': [(116, 90, 110), (146, 118, 134), (176, 150, 160), (204, 184, 190)],
}


OUTLINE_BASE = (43, 29, 42)


# Naïl 털빛(nail.py FURS와 같은 색 방향, 초상화 5단 경사).
FUR_VARIANTS = {
    1: [(48, 46, 56), (76, 74, 86), (108, 106, 118), (140, 138, 150), (176, 174, 184)],   # 잿빛
    2: [(46, 28, 30), (72, 46, 42), (102, 66, 54), (132, 94, 74), (162, 124, 98)],       # 밤빛
    3: [(110, 78, 56), (150, 112, 76), (188, 150, 102), (216, 182, 132), (240, 214, 170)],  # 모래빛
}


BAYER2 = [[0.0, 0.5], [0.75, 0.25]]


# ---- v2(비교 시험, 기본 꺼짐): NAIL_ART_V2=1 -------------------------------------------
# 참고한 기법(KB guides/pixel-art-without-an-artist.md "코드 그림 v2"):
#   색조 이동(그늘은 차갑게 보라·파랑 쪽, 빛은 따뜻하게 노랑 쪽), 디더 없는 단(셀 음영) + 외톨이 픽셀 정리(덩어리),
#   홍조는 디더 대신 작은 덩어리, 털 결은 무작위 줄을 줄여 덩어리 쪽 몇 가닥만.
V2 = os.environ.get('NAIL_ART_V2') == '1'


# v4(기본, docs/decisions/0071): v2 위에 점 다듬기에서 배운 것을 단계로 얹는다. 옛 그림과 비교할 때만
# NAIL_ART_V4=0(그러면 NAIL_ART_V2=1이 v2, 둘 다 없으면 v1).
V4 = os.environ.get('NAIL_ART_V4', '1') != '0'


V2 = V2 or V4


def _toward(h, target, amt):
    d = (target - h + 0.5) % 1.0 - 0.5
    return (h + d * amt) % 1.0


def hue_shift(ramp, dark_to=0.73, light_to=0.12, dark_amt=0.2, light_amt=0.18):
    """어두운 단은 보라·파랑 쪽, 밝은 단은 노랑 쪽으로 색조를 민다. 어두운 단은 채도를 조금 올린다."""
    n = len(ramp)
    out = []
    for i, (r, g, b) in enumerate(ramp):
        t = i / max(1, n - 1)
        h, l, sat = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
        if t < 0.5:
            k = (0.5 - t) * 2
            h = _toward(h, dark_to, dark_amt * k)
            sat = min(1.0, sat * (1 + 0.12 * k) + 0.02 * k)
        else:
            k = (t - 0.5) * 2
            h = _toward(h, light_to, light_amt * k)
            l = min(0.97, l + 0.02 * k)
        rr, gg, bb = colorsys.hls_to_rgb(h, l, sat)
        out.append((round(rr * 255), round(gg * 255), round(bb * 255)))
    return out


if V2:
    for _k in list(RAMP):
        if _k not in ('eye', 'glow', 'lens'):
            RAMP[_k] = hue_shift(RAMP[_k])
    for _k in list(FUR_VARIANTS):
        FUR_VARIANTS[_k] = hue_shift(FUR_VARIANTS[_k])


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
    if V2:
        # 셀 음영: 빛 쪽 경계를 조금 더 밀어 가운데 단(바탕색)이 넓다. 디더 없음.
        v = (i ** 0.85) * (n_levels - 0.01)
        lvl = max(0, min(n_levels - 1, int(v)))
        if V4 and lvl == n_levels - 1 and n_levels > 3 and i < V4_TOP:
            lvl -= 1      # 가장 밝은 단은 빛 바로 밑 작은 덩어리만(넓으면 반들반들한 피부로 읽힌다)
        return lvl
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


# ---- v4 단계: 털이 대머리·피부로 읽히지 않게 -------------------------------------------------------
# 점 다듬기(tools/pixel, 0070)로 Naïl 초상화 하나를 고치며 찾은 것(KB pixel-art 가이드 "마지막 한 겹은 점으로").
#   털은 한 올씩 그리지 않는다. 단 경계를 한쪽으로 기운 털끝 톱니로, 실루엣에 고르지 않은 삐침 몇 개로 보여 준다.
FURRY = ('fur', 'rfur', 'tfur', 'mfur', 'bfur', 'dfur', 'dtfur', 'mfurd', 'grey')   # 수수(sfur)는 벨벳이라 매끈하게 둔다


V4_TOP = 0.9


_lu = math.hypot(LIGHT[0], LIGHT[1])


_U = (LIGHT[0] / _lu, LIGHT[1] / _lu)      # 화면에서 빛 쪽


_V = (-_U[1], _U[0])


def fur_clumps(cv, period=4.6, amp=3.4, slant=0.12):
    """털 재질의 단 경계를 털 뭉치로: 빛 방향으로 톱니(비대칭 삼각파)만큼 밀린 자리의 단을 가져온다.
    경계만 바뀌고 덩어리 안은 그대로다. 톱니 끝이 모두 같은 쪽을 봐서 털이 한 방향으로 자란 것처럼 읽힌다."""
    src = [row[:] for row in cv.mat]
    for y in range(H):
        for x in range(W):
            m = src[y][x]
            if not m or m[0] not in FURRY or cv.col[y][x] is not None:
                continue
            a = x * _V[0] + y * _V[1]
            b = x * _U[0] + y * _U[1]
            k = amp * (((a / period + b * slant) % 1.0) - 0.5)
            qx, qy = int(round(x + _U[0] * k)), int(round(y + _U[1] * k))
            if 0 <= qx < W and 0 <= qy < H:
                q = src[qy][qx]
                if q and q[0] == m[0] and cv.col[qy][qx] is None:
                    cv.mat[y][x] = q


# 삐침 도장(가장자리 칸 기준 상대 좌표). 크기를 섞어야 털로 읽힌다(같은 크기가 고르게 서면 톱니·가시가 된다).
TUFT_TOP = ([(0, -1), (-1, -1), (-2, -2)],                                          # 작은 것: 끝이 왼쪽 위
            [(0, -1), (-1, -1), (-2, -1), (-1, -2), (-2, -2), (-3, -3)],            # 큰 것
            [(0, -1), (-1, -1), (-2, -1), (-3, -1), (-2, -2), (-3, -2), (-4, -3), (-5, -4)])   # 긴 것


TUFT_LEFT = ([(-1, 0), (-1, 1), (-2, 1), (-2, 2)],                                  # 작은 것: 끝이 왼쪽 아래
             [(-1, 0), (-1, 1), (-2, 1), (-1, 2), (-2, 2), (-3, 2), (-3, 3)],
             [(-1, 0), (-1, 1), (-2, 1), (-1, 2), (-2, 2), (-3, 2), (-3, 3), (-4, 3), (-4, 4)])


TUFT_SIZES = (1, 0, 2, 0, 1, 1, 0, 2, 0, 1)


TUFT_GAPS = (4, 6, 5, 7, 4, 6, 8, 5)


def silhouette_tufts(cv, min_run=12):
    """털 덩어리 윗선·왼쪽 선에 끝이 털 흐름(코에서 뒤로 = 왼쪽 아래)을 보는 삐침. 간격·크기는 일부러 고르지 않게.
    귀처럼 좁은 곳(가로 덩어리가 min_run보다 짧은 줄)은 건너뛴다. 삐침 색은 붙은 자리의 단(빛 쪽 끝이 어두우면 점으로 보인다)."""
    def fur(x, y):
        m = cv.mat[y][x] if 0 <= x < W and 0 <= y < H else None
        return m if m and m[0] in FURRY and cv.col[y][x] is None else None

    def empty(x, y):
        return 0 <= x < W and 0 <= y < H and cv.mat[y][x] is None

    def run(x, y):
        n = 0
        while fur(x + n, y):
            n += 1
        m = 0
        while fur(x - m - 1, y):
            m += 1
        return n + m
    adds = []
    i, nxt = 0, 3
    for x in range(2, W - 2):                     # 윗선: 끝이 왼쪽 위
        top = next((y for y in range(1, H - 30) if fur(x, y)), None)
        if top is None or not all(empty(x, yy) for yy in range(0, top)):
            continue
        if x < nxt or run(x, top + 1) < min_run:
            continue
        m = cv.mat[top][x]
        adds += [(x + dx, top + dy, m) for dx, dy in TUFT_TOP[TUFT_SIZES[i % len(TUFT_SIZES)]]]
        nxt = x + TUFT_GAPS[i % len(TUFT_GAPS)]
        i += 1
    i, nxt = 0, 0
    for y in range(4, H - 12):                    # 왼쪽 선: 끝이 왼쪽 아래
        left = next((x for x in range(1, W) if cv.mat[y][x] is not None), None)
        if left is None or not fur(left, y) or y < nxt or run(left, y) < min_run:
            continue
        m = cv.mat[y][left]
        adds += [(left + dx, y + dy, m) for dx, dy in TUFT_LEFT[TUFT_SIZES[(i + 3) % len(TUFT_SIZES)]]]
        nxt = y + TUFT_GAPS[(i + 2) % len(TUFT_GAPS)]
        i += 1
    for x, y, m in adds:
        if empty(x, y):
            cv.mat[y][x] = m


def cleanup_clusters(cv, passes=2):
    """v2: 같은 재질 안에서 상하좌우 어느 이웃과도 단이 다른 외톨이 픽셀을 둘레에 가장 많은 단으로 바꾼다(덩어리)."""
    for _ in range(passes):
        changes = []
        for y in range(1, H - 1):
            for x in range(1, W - 1):
                m = cv.mat[y][x]
                if not m or cv.col[y][x] is not None:
                    continue
                mat, lvl = m
                same, count = 0, {}
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    n = cv.mat[y + dy][x + dx]
                    if n and n[0] == mat:
                        if n[1] == lvl:
                            same += 1
                        count[n[1]] = count.get(n[1], 0) + 1
                if same == 0 and count:
                    changes.append((x, y, mat, max(count, key=count.get)))
        for x, y, mat, lvl in changes:
            cv.mat[y][x] = (mat, lvl)


def outline(cv):
    """선택 외곽선: 빈 칸이 채운 칸과 닿으면 그 재질의 가장 어두운 색(빛 쪽은 한 단 밝게)."""
    if V4:
        fur_clumps(cv)
        silhouette_tufts(cv)
    if V2:
        cleanup_clusters(cv)
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
                    if V4 and lit and mat in FURRY:
                        c = ramp[max(0, cv.mat[ny][nx][1] - 1)]   # 빛 쪽 실루엣이 가장 어두우면 빛 방향과 어긋난다
                    if mat in ('fur', 'shade', 'root') and not lit:
                        c = OUTLINE_BASE if mat == 'fur' else ramp[0]
                    out.append((x, y, c))
                    break
    for x, y, c in out:
        cv.mat[y][x] = ('outline', 0)
        cv.col[y][x] = c


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


# 코드로 찍던 칸 수(줄 수 계산용: 38칸 → 네 줄). 칸 38~40은 나노바나나 전용(0165).
FRAMES = 38


def sheet():
    # 한 줄 12칸(src/art.rs PORTRAIT_COLS): 한 줄로 늘어놓으면 2,048px를 넘어 오래된 폰에서 텍스처가 안 올라간다.
    cols = 12
    rows = (FRAMES + cols - 1) // cols
    out = Image.new('RGBA', (W * cols, H * rows), (0, 0, 0, 0))
    out.paste(to_image(neighbor()), (4 * W, 0))
    # 나노바나나 원본이 있는 칸(ai_portraits.py, docs/decisions/0108).
    import ai_portraits
    for i, img in ai_portraits.cells(FUR_VARIANTS).items():
        box = ((i % cols) * W, (i // cols) * H)
        out.paste(Image.new('RGBA', (W, H), (0, 0, 0, 0)), box)
        out.paste(img, box)
    return out


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/portraits.png')
