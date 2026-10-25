"""대화창 초상화 생성기(docs/decisions/0015). 이 파일이 원본이고 PNG는 산출물이다(직접 그린 그림, 프로젝트 소유).

실행: python3 tools/art/portraits.py [출력 경로]  (기본 assets/art/portraits.png)
칸 88×88(한 줄 12칸, 칸 번호는 왼쪽 위부터), 게임 픽셀의 2배 밀도(대화창의 44×44 칸에 찍는다 → 화면 배율 4면 원본 1px = 화면 2px).
  칸 0~3: Naïl 보통·기쁨·걱정·결의
  칸 4  : 변해 버린 이웃(아직 정체를 모름: 그림자 + 빛나는 눈 + 뿌리)
  칸 5~6: 라쿤 포포(튜토리얼 안내) 보통·기쁨
  칸 7~8: 모모(굴토끼, 히로인) 물든 모습 — 차가움·괴로움(docs/decisions/0029)
  칸 9  : 들쥐 대장(튜토리얼 보스) 물든 모습 — 노려봄(docs/decisions/0032)
  칸 10~11: 돌아온 보리(같은 들쥐) 보통·기쁨. 흉터는 남는다
  칸 12~14: 돌아온 모모 보통·기쁨·걱정 — 따뜻한 회갈색, 검은 눈, 핀 꽃, 뿌리 없음(docs/decisions/0049)
  칸 15~16: 누룩 할매 보통·기쁨 — 흰 실 덩어리 몸, 머리에 버섯 갓 셋, 감은 눈
  칸 17  : 엄마 두더지 — Naïl보다 크고 털이 짙다, 다정하게 감은 눈
  칸 18~29: Naïl 털빛 1~3(잿빛·밤빛·모래빛) × 표정 4(저널 보상, docs/decisions/0048) — 털 경사만 바꾼다
  칸 30  : 도토(물든 청설모, 둘째 길 보스) — 도토리 갓 모자, 호박색 빛 눈, 꼬리를 감은 뿌리(docs/decisions/0052)
  칸 31~32: 돌아온 도토 보통·기쁨 — 붉은 밤색 털, 검은 눈, 모자는 그대로
  칸 33~34: 땅강아지 수수 보통·기쁨 — 벨벳 투구 같은 가슴, 이마에 올린 보안경, 큰 삽발(docs/decisions/0053)
  칸 35  : 콩이(물든 세띠아르마딜로, 셋째 길 보스) — 띠 무늬 등딱지, 머리 방패, 옅은 하늘빛 눈, 뿌리(docs/decisions/0054)
  칸 36~37: 돌아온 콩이 보통·기쁨

그리는 법(손 도트를 흉내 낸 절차):
  1. 부위를 타원으로 놓고 칸마다 재질을 정한다(뒤에 놓인 것부터).
  2. 재질마다 4~5단 색 경사(ramp). 타원을 구로 보고 왼쪽 위 빛으로 단을 고른다. 단 경계만 2×2 바둑판 디더.
  3. 털 결: 머리 윗선을 들쭉날쭉하게(털 뭉치), 등 쪽에 짧은 어두운 결.
  4. 선택 외곽선(sel-out): 바깥선은 이웃 재질의 가장 어두운 색, 빛 받는 쪽은 한 단 밝게. 검은 선을 피한다.
  5. 눈: 큰 흰 반사 + 작은 반사 + 아래쪽 푸른 반사광. 볼 홍조는 디더.
비교 시험: NAIL_ART_V2=1(색조 이동·셀 음영, 캐릭터 해석), NAIL_ART_V4=1(v2 + 털 뭉치·삐침·작은 하이라이트, docs/decisions/0071).
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
RED_EYE = [(80, 14, 34), (190, 34, 72), (255, 84, 116), (255, 222, 230)]
GLOW_EYE = [(40, 70, 36), (110, 190, 70), (190, 245, 120), (240, 255, 210)]
SKY_EYE = [(30, 60, 96), (90, 170, 230), (170, 220, 255), (235, 250, 255)]
AMBER_EYE = [(96, 52, 12), (224, 132, 32), (255, 192, 84), (255, 240, 200)]
OUTLINE_BASE = (43, 29, 42)
# Naïl 털빛(nail.py FURS와 같은 색 방향, 초상화 5단 경사).
FUR_VARIANTS = {
    1: [(48, 46, 56), (76, 74, 86), (108, 106, 118), (140, 138, 150), (176, 174, 184)],   # 잿빛
    2: [(46, 28, 30), (72, 46, 42), (102, 66, 54), (132, 94, 74), (162, 124, 98)],       # 밤빛
    3: [(110, 78, 56), (150, 112, 76), (188, 150, 102), (216, 182, 132), (240, 214, 170)],  # 모래빛
}
BLUSH = (236, 120, 128)
MOUTH = (96, 52, 60)
WHITE = (255, 255, 255)
BAYER2 = [[0.0, 0.5], [0.75, 0.25]]


# ---- v2(비교 시험, 기본 꺼짐): NAIL_ART_V2=1 -------------------------------------------
# 참고한 기법(KB guides/pixel-art-without-an-artist.md "코드 그림 v2"):
#   색조 이동(그늘은 차갑게 보라·파랑 쪽, 빛은 따뜻하게 노랑 쪽), 디더 없는 단(셀 음영) + 외톨이 픽셀 정리(덩어리),
#   홍조는 디더 대신 작은 덩어리, 털 결은 무작위 줄을 줄여 덩어리 쪽 몇 가닥만.
V2 = os.environ.get('NAIL_ART_V2') == '1'
# v4(비교 시험, 기본 꺼짐): NAIL_ART_V4=1. v2 위에 점 다듬기에서 배운 것을 단계로 얹는다(docs/decisions/0071).
V4 = os.environ.get('NAIL_ART_V4') == '1'
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
                if V2:
                    # 디더 대신 작은 덩어리: 가운데만 칠한다(밀도가 낮으면 더 작게).
                    if r2 < 0.25 + 0.5 * density:
                        cv.paint(x, y, rgb)
                    continue
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


def silhouette_tufts(cv, spacing=(6, 9, 7, 8), min_run=12):
    """털 덩어리 윗선·왼쪽 선에 끝이 털 흐름(코에서 뒤로 = 왼쪽 아래)을 보는 작은 삐침. 간격은 일부러 고르지 않게.
    귀처럼 좁은 곳(가로 덩어리가 min_run보다 짧은 줄)은 건너뛴다."""
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
    i, nxt = 0, 4
    for x in range(2, W - 2):                     # 윗선: 끝이 왼쪽 위
        top = next((y for y in range(1, H - 30) if fur(x, y)), None)
        if top is None or not all(empty(x, yy) for yy in range(0, top)):
            continue
        if x < nxt or run(x, top + 1) < min_run:
            continue
        m = cv.mat[top][x]
        adds += [(x, top - 1, m), (x - 1, top - 1, m), (x - 2, top - 2, m)]
        nxt = x + spacing[i % len(spacing)]
        i += 1
    i, nxt = 0, 0
    for y in range(4, H - 12):                    # 왼쪽 선: 끝이 왼쪽 아래
        left = next((x for x in range(1, W) if cv.mat[y][x] is not None), None)
        if left is None or not fur(left, y) or y < nxt or run(left, y) < min_run:
            continue
        m = cv.mat[y][left]
        adds += [(left - 1, y, m), (left - 1, y + 1, m), (left - 2, y + 1, m), (left - 2, y + 2, m)]
        nxt = y + spacing[(i + 1) % len(spacing)]
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
    for _ in range(10 if V2 else 34):
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
    for _ in range(5 if V2 else 14):
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


# ---- 돌아온 모모 -------------------------------------------------------------------------
def momo_back(face):
    """돌아온 모모(0049): 물든 모습(momo_tainted)과 같은 실루엣. 털은 따뜻하게, 귀 안은 분홍, 눈은 검고 맑게,
    가까운 귀 밑동의 꽃이 분홍으로 피었다. 뿌리는 없다. face: 'normal' / 'happy' / 'worried'."""
    cv = Canvas()

    def fluff(x, y, ang):
        r = tuft(x, y, ang)
        if 0.2 < ang < 1.3 or 1.85 < ang < 2.95:
            r *= 1.0 + 0.09 * (0.5 + 0.5 * math.sin(ang * 9.0))
        return r
    ellipse(cv, 26, 22, 7, 17, 'bfur', bias=-0.08, clip=lambda x, y: y < 40)
    flat(cv, 27, 22, 3.5, 12, 'pink', 0)
    ellipse(cv, 50, 18, 8, 18, 'bfur', bias=0.04, clip=lambda x, y: y < 40)
    flat(cv, 51, 19, 4.0, 13, 'pink', 1)
    flat(cv, 51, 16, 2.0, 8, 'pink', 2)
    for (x, y) in ((49, 6), (50, 5), (51, 5), (52, 6)):
        cv.put(x, y, 'bfur', 2)
    ellipse(cv, 42, 54, 29, 24, 'bfur', bias=0.05, rough=fluff)
    ellipse(cv, 40, 92, 22, 10, 'bfur', bias=-0.15)
    ellipse(cv, 40, 88, 12, 6, 'blight', bias=-0.1)                 # 가슴 털
    ellipse(cv, 61, 62, 5.5, 4.5, 'blight', bias=-0.05)
    ellipse(cv, 66, 58, 2.4, 1.9, 'pink', bias=-0.1)
    outline(cv)
    rim_light(cv, 'bfur')
    # 털 결(머리 위·볼)
    for (x, y) in ((30, 36), (31, 37), (36, 33), (37, 34), (22, 52), (23, 53), (62, 40), (63, 41)):
        cv.paint(x, y, RAMP['bfur'][1])
    for (x, y) in ((28, 40), (34, 36), (24, 48)):
        cv.paint(x, y, RAMP['bfur'][4])
    # 핀 꽃(가까운 귀 밑동): 꽃잎 다섯이 위로 펴지고 분홍, 가운데 노랑, 잎 하나.
    fx, fy = 53, 23
    for (cx, cy) in ((0, -4), (-4, -1), (4, -1), (-3, 4), (3, 4)):
        flat(cv, fx + cx, fy + cy, 2.8, 2.5, 'bloom', 2)
        cv.paint(fx + cx - 1, fy + cy - 1, RAMP['bloom'][3])
        cv.paint(fx + cx + 1, fy + cy + 1, RAMP['bloom'][1])
    for (ox, oy) in ((-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1), (0, -1)):
        cv.paint(fx + ox, fy + oy, (255, 222, 120))
    cv.paint(fx, fy, (236, 180, 70))
    for (x, y) in ((fx + 6, fy + 3), (fx + 7, fy + 4), (fx + 7, fy + 5), (fx + 6, fy + 5)):
        cv.paint(x, y, (110, 170, 90))
    # 눈: 크고 검은 눈 + 큰 반사(Naïl과 같은 눈). 기쁨은 감은 눈.
    if face == 'happy':
        happy_eye(cv, 39, 48, 5.6)
        happy_eye(cv, 58, 45, 4.2)
    else:
        eye(cv, 39, 48, 5.6, 6.6)
        eye(cv, 58, 45, 4.2, 5.4, look=(0.3, 0.0))
        for (ex, ey, rx, ry, outer) in ((39, 48, 5.6, 6.6, -1), (58, 45, 4.2, 5.4, 1)):
            bx, by = int(ex + outer * rx * 0.9), int(ey - ry * 0.6)
            for (ox, oy) in ((0, 0), (outer, -1), (outer, 1), (2 * outer, 0)):
                cv.paint(bx + ox, by + oy, RAMP['eye'][0])
    if face == 'worried':
        line(cv, [(33, 38), (44, 35)], RAMP['bfur'][0], 2)
        line(cv, [(54, 34), (62, 36)], RAMP['bfur'][0], 2)
        line(cv, [(60, 68), (63, 67), (66, 68)], MOUTH, 1)
    elif face == 'happy':
        for (x, y) in ((59, 67), (60, 68), (61, 69), (62, 69), (63, 69), (64, 68), (65, 67)):
            cv.paint(x, y, MOUTH)
        for (x, y) in ((61, 70), (62, 70), (63, 70)):
            cv.paint(x, y, RAMP['pink'][2])
    else:
        line(cv, [(34, 39), (43, 38)], RAMP['bfur'][1], 2)
        line(cv, [(54, 36), (61, 36)], RAMP['bfur'][1], 2)
        line(cv, [(60, 67), (62, 68), (64, 68), (66, 67)], MOUTH, 1)
    dither_ellipse(cv, 48, 60, 5.0, 2.6, BLUSH, 0.5)
    dither_ellipse(cv, 30, 58, 3.5, 2.0, BLUSH, 0.4)
    return cv


# ---- 도토(청설모) ------------------------------------------------------------------------
def doto(face):
    """도토(0052): 등 뒤로 솟은 큰 꼬리, 끝에 털 술이 난 뾰족한 귀, 머리에 도토리 갓 모자(가장 아끼는 것).
    face: 'tainted'(물든 모습 — 호박색 빛 눈, 찌푸린 눈썹, 앞니, 꼬리·목을 감은 뿌리) / 'normal' / 'happy'."""
    cv = Canvas()
    bad = face == 'tainted'
    fur, light = ('dtfur', 'dtlight') if bad else ('dfur', 'dlight')

    def bushy(x, y, ang):
        return 1.0 + 0.07 * math.sin(ang * 11.0) + 0.03 * math.sin(ang * 27.0)
    # 꼬리(뒤): 왼쪽 아래에서 솟아 머리 위로 말린다
    ellipse(cv, 15, 64, 13, 22, fur, bias=-0.15, rough=bushy)
    ellipse(cv, 12, 40, 13, 19, fur, bias=-0.08, rough=bushy)
    ellipse(cv, 20, 20, 15, 12, fur, bias=-0.02, rough=bushy)
    ellipse(cv, 30, 16, 7, 6, fur, bias=0.05, rough=bushy)
    # 귀: 뾰족하게, 끝에 털 술(짙은 털 몇 가닥이 위로 삐죽)
    ellipse(cv, 37, 23, 5, 11, fur, bias=0.0, clip=lambda x, y: y < 36)
    flat(cv, 37, 25, 2.0, 6, 'pink', 0)
    ellipse(cv, 57, 20, 6, 12, fur, bias=0.05, clip=lambda x, y: y < 36)
    flat(cv, 57, 23, 2.5, 7, 'pink', 0 if bad else 1)
    for (x, y) in ((36, 11), (37, 10), (37, 9), (38, 8), (36, 9), (35, 8),
                   (56, 8), (57, 7), (57, 6), (58, 5), (56, 5), (55, 6), (59, 7)):
        cv.put(x, y, fur, 0)
    # 머리 + 볼털
    def fluff(x, y, ang):
        r = tuft(x, y, ang)
        if 0.2 < ang < 1.3 or 1.85 < ang < 2.95:
            r *= 1.0 + 0.08 * (0.5 + 0.5 * math.sin(ang * 9.0))
        return r
    ellipse(cv, 47, 55, 24, 21, fur, bias=0.05, rough=fluff)
    ellipse(cv, 46, 92, 20, 10, fur, bias=-0.15)
    ellipse(cv, 46, 88, 10, 6, light, bias=-0.1)                    # 가슴 털
    # 주둥이·볼(밝은 털), 코
    ellipse(cv, 62, 63, 5.5, 4.5, light, bias=-0.05)
    ellipse(cv, 67, 59, 2.2, 1.8, 'pink', bias=-0.2 if bad else -0.1)
    # 도토리 갓 모자: 머리 위 반원 + 짧은 꼭지
    ellipse(cv, 45, 34, 14, 8, 'acorn', bias=0.1, clip=lambda x, y: y < 36)
    flat(cv, 45, 35, 14, 1.6, 'acorn', 0)
    for (x, y) in ((45, 25), (46, 24), (46, 23), (47, 22)):
        cv.put(x, y, 'acorn', 1)
    # 뿌리(물든 모습만): 목을 한 바퀴, 꼬리를 타고 오른다
    if bad:
        for (pts, w) in (
            ([(14, 78), (28, 74), (42, 77), (56, 73), (70, 76)], 3),
            ([(10, 70), (14, 56), (20, 42), (16, 28), (24, 16)], 2),
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
    rim_light(cv, fur)
    # 모자 비늘 무늬(엇갈린 점), 챙 밝은 선
    for y in range(27, 35):
        for x in range(32, 59):
            m = cv.mat[y][x]
            if m and m[0] == 'acorn' and (x + (y // 2) * 2) % 4 == 0 and y % 2 == 0:
                cv.paint(x, y, RAMP['acorn'][max(0, m[1] - 1)])
    for x in range(33, 58):
        if cv.mat[34][x] and cv.mat[34][x][0] == 'acorn':
            cv.paint(x, 34, RAMP['acorn'][3])
    # 꼬리 털 결
    for (x0, y0) in ((8, 36), (14, 26), (6, 52), (12, 64), (20, 12), (26, 18), (10, 44)):
        for k in range(3):
            x, y = x0 + k, y0 + k
            if cv.mat[y][x] and cv.mat[y][x][0] == fur and cv.col[y][x] is None:
                cv.paint(x, y, RAMP[fur][max(0, cv.mat[y][x][1] - 1)])
    if bad:
        for (x, y) in ((28, 73), (42, 76), (56, 72), (14, 55), (19, 41)):
            cv.paint(x, y, RAMP['vroot'][3])
        # 눈 밑 그늘
        dither_ellipse(cv, 42, 58, 5.5, 1.8, RAMP[fur][1], 0.45)
        dither_ellipse(cv, 59, 55, 4.0, 1.5, RAMP[fur][1], 0.45)
        # 호박색 빛 눈(둥근 동공, 큰 반사 — 모모의 초록과 같은 짜임, 색만 다르다)
        for (ex, ey, rx, ry) in ((42, 49, 5.4, 6.4), (60, 46, 4.0, 5.2)):
            for y in range(int(ey - ry - 1), int(ey + ry + 2)):
                for x in range(int(ex - rx - 1), int(ex + rx + 2)):
                    dx, dy = (x + 0.5 - ex) / rx, (y + 0.5 - ey) / ry
                    d = dx * dx + dy * dy
                    if d <= 1.0:
                        if d > 0.78:
                            c = AMBER_EYE[0]
                        elif d < 0.2:
                            c = RAMP['eye'][0]
                        else:
                            c = AMBER_EYE[1] if dy < 0.15 else AMBER_EYE[2]
                        cv.paint(x, y, c)
            hx, hy = int(ex - rx * 0.38), int(ey - ry * 0.42)
            for (ox, oy) in ((0, 0), (1, 0), (0, 1), (1, 1)):
                cv.paint(hx + ox, hy + oy, WHITE)
            cv.paint(int(ex + rx * 0.35), int(ey + ry * 0.4), AMBER_EYE[3])
        # 찌푸린 눈썹(안쪽이 내려감), 앙다문 입 + 앞니 둘
        line(cv, [(36, 39), (47, 42)], RAMP[fur][0], 2)
        line(cv, [(56, 40), (63, 37)], RAMP[fur][0], 2)
        line(cv, [(62, 68), (65, 69), (68, 68)], MOUTH, 1)
        for (x, y) in ((64, 70), (65, 70), (64, 71), (65, 71)):
            cv.paint(x, y, (250, 246, 236))
        dither_ellipse(cv, 52, 63, 4.0, 2.0, (150, 96, 104), 0.35)
    else:
        if face == 'happy':
            happy_eye(cv, 42, 49, 5.2)
            happy_eye(cv, 60, 46, 4.0)
            for (x, y) in ((61, 68), (62, 69), (63, 70), (64, 70), (65, 70), (66, 69), (67, 68)):
                cv.paint(x, y, MOUTH)
            for (x, y) in ((63, 71), (64, 71), (65, 71)):
                cv.paint(x, y, RAMP['pink'][2])
        else:
            eye(cv, 42, 49, 5.2, 6.2)
            eye(cv, 60, 46, 3.9, 5.0, look=(0.3, 0.0))
            line(cv, [(36, 40), (46, 39)], RAMP[fur][1], 2)
            line(cv, [(56, 38), (63, 38)], RAMP[fur][1], 2)
            line(cv, [(62, 68), (64, 69), (66, 69), (68, 68)], MOUTH, 1)
        for (x, y) in ((64, 70), (65, 70)):
            cv.paint(x, y, (250, 246, 236))
        dither_ellipse(cv, 52, 62, 5.0, 2.6, BLUSH, 0.5)
        dither_ellipse(cv, 33, 60, 3.0, 1.8, BLUSH, 0.4)
    return cv


# ---- 수수(땅강아지) ----------------------------------------------------------------------
def susu(face):
    """수수(0053): 땅강아지 장인. 뒤로 마디진 배, 둥근 투구 같은 가슴(벨벳 윤기), 작은 머리와 구슬 눈, 짧은 더듬이,
    앞에 든 큰 삽발(이가 난 짙은 앞다리 — 두더지 발과 같은 "파는 손"), 이마에 올려 쓴 보안경. face: 'normal' / 'happy'."""
    cv = Canvas()
    # 배(뒤): 마디 줄
    ellipse(cv, 20, 70, 19, 14, 'sfur', bias=-0.12)
    # 가슴(투구)
    ellipse(cv, 42, 44, 25, 21, 'sfur', bias=0.12)
    # 머리
    ellipse(cv, 60, 60, 15, 12, 'sfur', bias=0.05)
    # 삽발 둘(먼 것 → 가까운 것)
    ellipse(cv, 50, 80, 9, 6, 'shovel', bias=-0.05)
    ellipse(cv, 70, 78, 11, 7, 'shovel', bias=0.05)
    outline(cv)
    rim_light(cv, 'sfur')
    # 배 마디
    for x0 in (8, 14, 20, 26, 32):
        for y in range(60, 82):
            m = cv.mat[y][x0 + (y - 60) // 8]
            if m and m[0] == 'sfur' and cv.col[y][x0 + (y - 60) // 8] is None:
                cv.paint(x0 + (y - 60) // 8, y, RAMP['sfur'][max(0, m[1] - 1)])
    # 가슴 윤기(벨벳): 위쪽에 밝은 띠
    for x in range(28, 52):
        y = 30 + int(((x - 40) / 12.0) ** 2 * 3)
        if cv.mat[y][x] and cv.mat[y][x][0] == 'sfur':
            cv.paint(x, y, RAMP['sfur'][4])
    # 삽발의 이: 앞 끝에 크림색 발톱이 넷·셋 삐죽(Naïl 발톱과 같은 색 — 같은 "파는 손")
    for (x, y, n) in ((81, 73, 3), (82, 76, 3), (82, 79, 3), (80, 82, 2), (59, 77, 2), (60, 80, 2), (58, 83, 2)):
        for k in range(n):
            cv.paint(x + k, y, RAMP['claw'][2 if k < n - 1 else 3])
            cv.paint(x + k, y + 1, RAMP['claw'][0] if k < n - 1 else OUTLINE_BASE)
    # 더듬이
    line(cv, [(64, 50), (72, 38), (82, 30)], RAMP['sfur'][0], 1)
    line(cv, [(68, 52), (78, 42), (86, 38)], RAMP['sfur'][0], 1)
    # 보안경(이마 위로 올렸다): 테 + 알 + 끈
    line(cv, [(20, 34), (30, 26)], RAMP['brass'][0], 2)
    for (gx, gy, r) in ((38, 24, 6.0), (51, 22, 5.5)):
        flat(cv, gx, gy, r, r, 'brass', 1)
        flat(cv, gx, gy, r - 2.0, r - 2.0, 'lens', 1)
        cv.paint(int(gx - 1), int(gy - 2), RAMP['lens'][2])
        cv.paint(int(gx - 2), int(gy - 1), RAMP['lens'][2])
        for k in range(8):
            a = k / 8 * 6.283
            cv.paint(int(gx + math.cos(a) * r), int(gy + math.sin(a) * r), RAMP['brass'][0])
    line(cv, [(44, 22), (46, 22)], RAMP['brass'][2], 1)
    # 눈(작은 구슬 눈 둘, 3/4)
    if face == 'happy':
        happy_eye(cv, 60, 56, 3.4)
        happy_eye(cv, 70, 54, 2.6)
        for (x, y) in ((63, 66), (64, 67), (65, 67), (66, 67), (67, 66)):
            cv.paint(x, y, MOUTH)
    else:
        eye(cv, 60, 56, 3.2, 3.8, look=(0.2, 0.0))
        eye(cv, 70, 54, 2.4, 3.0, look=(0.1, 0.0))
        line(cv, [(64, 66), (66, 67), (68, 66)], MOUTH, 1)
    dither_ellipse(cv, 58, 63, 3.5, 1.8, BLUSH, 0.45)
    return cv


# ---- 콩이(세띠아르마딜로) -----------------------------------------------------------------
def kongi(face):
    """콩이(0054): 깊은 곳 입구의 문지기. 뒤로 둥근 띠 무늬 등딱지, 위로 선 두 귀, 머리 방패, 긴 주둥이, 작은 눈.
    face: 'tainted'(물든 모습 — 옅은 하늘빛 눈, 앙다문 입, 등딱지를 감은 뿌리) / 'normal' / 'happy'."""
    cv = Canvas()
    bad = face == 'tainted'
    shell, skin = ('ktshell', 'ktskin') if bad else ('kshell', 'kskin')
    # 등딱지(뒤)와 어깨 띠
    ellipse(cv, 22, 62, 27, 28, shell, bias=-0.05)
    ellipse(cv, 40, 88, 26, 10, shell, bias=-0.15)
    # 귀(뒤 → 앞), 머리, 주둥이
    ellipse(cv, 42, 26, 4.5, 11, skin, bias=-0.1, clip=lambda x, y: y < 38)
    ellipse(cv, 52, 23, 5, 12, skin, bias=0.0, clip=lambda x, y: y < 38)
    flat(cv, 52, 25, 2, 7, 'pink', 0 if bad else 1)
    ellipse(cv, 52, 54, 19, 15, skin, bias=0.05)
    ellipse(cv, 68, 60, 10, 7, skin, bias=0.0)
    ellipse(cv, 77, 60, 7, 4.5, skin, bias=-0.05)
    # 머리 방패: 정수리에서 주둥이 위까지 길게 덮는 판(비니처럼 보이지 않게 앞으로 가늘어진다)
    ellipse(cv, 54, 44, 18, 7, shell, bias=0.15, clip=lambda x, y: y < 48)
    ellipse(cv, 70, 54, 11, 3.5, shell, bias=0.1, clip=lambda x, y: y < 56)
    if bad:
        for (pts, w) in (
            ([(4, 52), (14, 44), (26, 46), (38, 40)], 3),
            ([(10, 80), (22, 74), (34, 78), (46, 74)], 2),
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
    rim_light(cv, shell)
    # 등딱지 띠: 세로 홈(어두운 선) + 띠마다 밝은 윗가장자리
    for bx in (8, 15, 22, 29, 36):
        for y in range(34, 88):
            for x in (bx + (y - 60) // 14,):
                m = cv.mat[y][x] if 0 <= x < 88 else None
                if m and m[0] == shell and cv.col[y][x] is None:
                    cv.paint(x, y, RAMP[shell][max(0, m[1] - 2)])
                    if x + 1 < 88 and cv.mat[y][x + 1] and cv.mat[y][x + 1][0] == shell:
                        cv.paint(x + 1, y, RAMP[shell][min(4, m[1] + 1)])
    # 머리 방패 비늘 점
    for (x, y) in ((42, 40), (48, 38), (54, 38), (60, 40), (45, 44), (51, 43), (57, 43)):
        if cv.mat[y][x] and cv.mat[y][x][0] == shell:
            cv.paint(x, y, RAMP[shell][1])
    # 코끝
    for (x, y) in ((83, 59), (84, 60), (83, 61), (84, 59)):
        cv.paint(x, y, RAMP['nose'][0])
    if bad:
        for (x, y) in ((14, 43), (26, 45), (22, 73), (34, 77)):
            cv.paint(x, y, RAMP['vroot'][3])
        for (ex, ey, rx, ry) in ((58, 52, 3.4, 3.8),):
            for y in range(int(ey - ry - 1), int(ey + ry + 2)):
                for x in range(int(ex - rx - 1), int(ex + rx + 2)):
                    dx, dy = (x + 0.5 - ex) / rx, (y + 0.5 - ey) / ry
                    d = dx * dx + dy * dy
                    if d <= 1.0:
                        c = SKY_EYE[0] if d > 0.7 else (RAMP['eye'][0] if d < 0.18 else SKY_EYE[1 if dy < 0.1 else 2])
                        cv.paint(x, y, c)
            cv.paint(int(ex - 1), int(ey - 2), WHITE)
        line(cv, [(52, 45), (62, 48)], RAMP[shell][0], 2)                # 찌푸린 눈썹
        line(cv, [(66, 66), (70, 67), (74, 66)], MOUTH, 1)
        dither_ellipse(cv, 58, 61, 3.5, 1.8, (150, 96, 120), 0.35)
    else:
        if face == 'happy':
            happy_eye(cv, 58, 53, 3.4)
            for (x, y) in ((66, 66), (67, 67), (68, 68), (69, 68), (70, 68), (71, 67), (72, 66)):
                cv.paint(x, y, MOUTH)
        else:
            eye(cv, 58, 52, 3.2, 3.8, look=(0.2, 0.0))
            line(cv, [(66, 66), (69, 67), (72, 66)], MOUTH, 1)
        dither_ellipse(cv, 57, 61, 4.0, 2.0, BLUSH, 0.5)
    return cv


# ---- 누룩 할매 ---------------------------------------------------------------------------
def halmae(face):
    """누룩 할매(0045·0049): 흰 실이 뭉친 둥근 몸(뽕나무버섯의 균사, 0037), 머리에 버섯 갓 셋, 늘 감은 눈.
    얼굴은 실 사이로 동그랗게 드러난다. face: 'normal'(조용히 감은 눈) / 'happy'(웃는 눈 + 홍조)."""
    cv = Canvas()

    def wool(x, y, ang):
        return 1.0 + 0.05 * math.sin(ang * 11.0 + 0.4) + 0.03 * math.sin(ang * 23.0)
    # 버섯 갓(뒤 → 앞): 대 먼저, 갓은 반원
    for (sx, sy, sh) in ((24, 22, 12), (46, 14, 14), (66, 24, 11)):
        flat(cv, sx, sy + sh / 2, 2.5, sh / 2, 'stem', 1)
    ellipse(cv, 44, 60, 36, 32, 'thread', bias=0.08, rough=wool)
    for (cx, cy, rx, ry) in ((24, 20, 11, 7), (46, 11, 13, 8), (66, 22, 10, 6.5)):
        ellipse(cv, cx, cy, rx, ry, 'cap', bias=0.05, clip=lambda x, y, cy=cy: y <= cy + 1)
    # 얼굴: 실 사이로 드러난 부드러운 면(한 단 밝게)
    flat(cv, 50, 56, 17, 13, 'thread', 4)
    flat(cv, 50, 57, 15, 11, 'thread', 3)
    outline(cv)
    rim_light(cv, 'thread')
    # 갓 반점
    for (x, y) in ((20, 16), (28, 18), (42, 6), (50, 9), (63, 19), (70, 21)):
        cv.paint(x, y, RAMP['cap'][3])
        cv.paint(x + 1, y, RAMP['cap'][3])
    # 실 결: 몸을 감는 가는 흰 선 몇 가닥 + 바닥으로 풀린 실
    for pts in (
        [(12, 52), (22, 46), (34, 44)],
        [(66, 42), (76, 48), (82, 58)],
        [(10, 70), (20, 76), (30, 74)],
        [(70, 74), (78, 70), (84, 76)],
        [(20, 86), (28, 80), (38, 84)],
    ):
        line(cv, pts, RAMP['thread'][4], 1)
    for pts in ([(14, 60), (22, 58)], [(74, 62), (80, 66)], [(56, 84), (66, 80)]):
        line(cv, pts, RAMP['thread'][1], 1)
    # 눈: 늘 감고 있다. 보통 = 아래로 휜 조용한 선(︶), 기쁨 = ^ + 눈가 주름
    if face == 'happy':
        happy_eye(cv, 42, 54, 4.4)
        happy_eye(cv, 59, 53, 3.6)
        dither_ellipse(cv, 41, 61, 4.5, 2.2, BLUSH, 0.5)
        dither_ellipse(cv, 61, 60, 3.6, 2.0, BLUSH, 0.45)
        for (x, y) in ((61, 64), (62, 65), (63, 65), (64, 65), (65, 64)):
            cv.paint(x, y, MOUTH)
    else:
        line(cv, [(38, 53), (40, 55), (43, 55), (46, 53)], RAMP['eye'][1], 2)
        line(cv, [(56, 52), (58, 54), (60, 54), (62, 52)], RAMP['eye'][1], 2)
        dither_ellipse(cv, 41, 61, 4.0, 2.0, BLUSH, 0.3)
        dither_ellipse(cv, 61, 60, 3.2, 1.8, BLUSH, 0.3)
        line(cv, [(60, 65), (62, 66), (64, 65)], MOUTH, 1)
    # 눈가 주름(나이)
    for (x, y) in ((35, 51), (34, 52), (64, 50), (65, 51)):
        cv.paint(x, y, RAMP['thread'][1])
    return cv


# ---- 엄마 두더지 -------------------------------------------------------------------------
def mom():
    """엄마 두더지(오프닝, 0037 5-A): Naïl보다 크고(칸을 거의 채운다) 털이 짙다. 다정하게 감은 눈, 긴 주둥이."""
    cv = Canvas()
    ellipse(cv, 38, 54, 40, 36, 'mfurd', bias=0.1, rough=tuft)
    ellipse(cv, 66, 52, 13, 9, 'muzzle', bias=0.0)
    ellipse(cv, 78, 47, 7, 6.5, 'pink', bias=0.1)
    for (x, y) in ((75, 44), (76, 44), (75, 45)):
        cv.paint(x, y, RAMP['pink'][3])
    outline(cv)
    inner_edges(cv)
    rim_light(cv, 'mfurd')
    import random
    rnd = random.Random(11)
    for _ in range(40):
        x, y = rnd.randint(8, 56), rnd.randint(22, 82)
        m = cv.mat[y][x]
        if m and m[0] == 'mfurd' and cv.col[y][x] is None:
            c = RAMP['mfurd'][max(0, m[1] - 1)]
            cv.paint(x, y, c)
            if cv.mat[y + 1][x - 1] and cv.mat[y + 1][x - 1][0] == 'mfurd':
                cv.paint(x - 1, y + 1, c)
    # 감은 눈(아래로 휜 부드러운 선, ︶): 반쯤 뜬 눈은 의심하는 눈처럼 읽혔다. 눈썹은 살짝 올라가 다정하게.
    line(cv, [(49, 40), (51, 42), (55, 42), (58, 40)], RAMP['eye'][1], 2)
    line(cv, [(63, 37), (65, 39), (68, 39), (70, 37)], RAMP['eye'][1], 2)
    line(cv, [(48, 33), (57, 32)], RAMP['mfurd'][1], 2)
    dither_ellipse(cv, 53, 50, 5.0, 2.6, BLUSH, 0.4)
    line(cv, [(66, 58), (69, 59), (72, 58)], MOUTH, 1)
    return cv


# ---- v2 캐릭터 해석(NAIL_ART_V2=1에서만 시트에 쓴다) --------------------------------------------
# 아기 도식(Lorenz): 큰 머리·높은 이마·낮게 놓인 큰 눈·둥근 볼. 둥근 모양 = 부드럽고 다가가기 쉬움(모양 언어).
# 눈은 표정의 대부분이라 새로 짓는다: 두꺼운 윗눈꺼풀 선 + 바깥 끝 살짝 올림, 홍채 3단(자두 → 아래 푸른 반사), 동공,
# 큰 반사 하나 + 반대쪽 작은 반사. 털 가장자리는 물결(풍선) 대신 뾰족한 털 뭉치(톱니).
EYE2 = [(30, 18, 40), (60, 40, 78), (104, 92, 150)]
PUPIL = (12, 8, 18)
LID = (24, 14, 26)


def saw(ang, lo, hi, freq, amp, sharp=1.6):
    """lo < ang < hi 구간에서 뾰족한 털 뭉치(삼각파). 바깥 반지름 배율."""
    if not (lo < ang < hi):
        return 1.0
    t = ((ang - lo) * freq / (2 * math.pi)) % 1.0
    tri = 1.0 - abs(2.0 * t - 1.0)
    return 1.0 + amp * tri ** sharp


def eye2(cv, cx, cy, rx, ry, look=(0.5, 0.0), outer=1, lash=False, iris=None):
    iris = iris or EYE2
    for y in range(int(cy - ry - 2), int(cy + ry + 3)):
        for x in range(int(cx - rx - 2), int(cx + rx + 3)):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            if dx * dx + dy * dy > 1.0:
                continue
            c = iris[0] if dy < -0.15 else (iris[1] if dy < 0.5 else iris[2])
            px, py = (x + 0.5 - cx - look[0]) / (rx * 0.5), (y + 0.5 - cy - ry * 0.08) / (ry * 0.55)
            if px * px + py * py <= 1.0:
                c = PUPIL
            cv.paint(x, y, c)
    # 윗눈꺼풀: 위쪽 가장자리 2px(가운데 70%), 바깥 끝은 위로 한 칸 튄다.
    for x in range(int(cx - rx * 0.85), int(cx + rx * 0.85) + 1):
        dx = (x + 0.5 - cx) / rx
        top = cy - ry * math.sqrt(max(0.0, 1 - dx * dx))
        cv.paint(x, int(top), LID)
        cv.paint(x, int(top) + 1, LID)
    ox = int(cx + outer * rx * 0.95)
    cv.paint(ox, int(cy - ry * 0.55), LID)
    cv.paint(ox + outer, int(cy - ry * 0.75), LID)
    if lash:
        cv.paint(ox + outer * 2, int(cy - ry * 0.85), LID)
        cv.paint(ox + outer, int(cy - ry * 0.35), LID)
    # 큰 반사(둥근 3×3, 작은 눈은 2×3)와 반대쪽 작은 반사
    hx, hy = int(cx - rx * 0.42 + look[0]), int(cy - ry * 0.42)
    big = [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 1), (0, 2)] if rx >= 4.2 else [(0, 0), (1, 0), (0, 1), (1, 1)]
    for ox_, oy_ in big:
        cv.paint(hx + ox_, hy + oy_, WHITE)
    cv.paint(int(cx + rx * 0.38 + look[0]), int(cy + ry * 0.38), WHITE)


def happy_eye2(cv, cx, cy, rx):
    """∩ 모양 감은 눈(두껍게, 끝이 살짝 아래로)."""
    pts = [(cx - rx, cy + 1.8), (cx - rx * 0.6, cy - 1.2), (cx, cy - 2.4), (cx + rx * 0.6, cy - 1.2), (cx + rx, cy + 1.8)]
    line(cv, pts, LID, 2)


def nail2(face):
    """Naïl v2: 한 덩어리 몸(목이 없다) + 뾰족한 털 뭉치 윗선, 엄마가 떠 준 빨간 목도리, 큰 삽 모양 앞발."""
    cv = Canvas()
    determined = face == 'determined'

    def blob(x, y, ang):
        r = saw(ang, -3.05, -0.25, 11, 0.07)
        if 0.3 < ang < 2.8:
            r *= 1.0 + 0.06 * math.sin(ang)
        return r
    ellipse(cv, 41, 50, 35, 31, 'fur', bias=0.08, rough=blob)
    # 목도리: 가슴을 가로지르는 굵은 띠(살짝 웃는 곡선) + 앞가슴 매듭 + 짧은 꼬리 둘. 세로 뜨개 골.
    def yc(x):
        return 62 + 0.0016 * (x - 42) ** 2
    for y in range(52, 80):
        for x in range(6, 78):
            if cv.mat[y][x] is None:
                continue
            d = y + 0.5 - yc(x)
            if -5.0 <= d <= 5.0:
                if V4:
                    # 몸을 감은 원통: 위 3분의 1이 가장 밝고 빛에서 먼 오른쪽이 어둡다. 골은 4칸마다 한 줄(2칸 줄무늬는 소음).
                    ny_ = max(-0.99, min(0.99, ((d + 5.0) / 10.0 - 0.42) * 1.6))
                    nx_ = (x - 40) / 40 * 0.8
                    i_ = nx_ * LIGHT[0] + ny_ * LIGHT[1] + math.sqrt(max(0.0, 1 - nx_ * nx_ - ny_ * ny_)) * LIGHT[2]
                    lvl = 0 if i_ < 0.05 else 1 if i_ < 0.3 else 2 if i_ < 0.62 else 3
                    if (x + 1) % 4 == 0 and lvl > 0:
                        lvl -= 1
                    cv.put(x, y, 'scarf', lvl)
                    continue
                lvl = 3 if d < -2.6 else (2 if d < 2.2 else 1)
                if (x // 2) % 2 == 0:                       # 뜨개 골
                    lvl -= 1
                cv.put(x, y, 'scarf', lvl)
    for (tx, sx) in ((40, -0.35), (47, 0.3)):               # 꼬리 둘(매듭 아래로 벌어진다)
        for y in range(68, 82):
            cx = tx + (y - 68) * sx
            for x in range(int(cx - 2.6), int(cx + 2.6) + 1):
                lvl = 0 if y >= 80 and x % 2 == 0 else (2 if x < cx else 1)
                if y >= 80 and x % 2 == 1:
                    continue
                cv.put(x, y, 'scarf', lvl)
    ellipse(cv, 44, 66, 5.2, 4.4, 'scarf', bias=0.05)        # 매듭
    # 주둥이 + 단추 코(크게)
    ellipse(cv, 63, 50, 11, 8, 'muzzle', bias=0.05)
    ellipse(cv, 73, 46, 7, 6.5, 'pink', bias=0.12)
    # 앞발: 크고 넓은 삽 모양(두더지의 상징) + 발톱 넷
    paw_y = 76 if determined else 80
    for (px, py, s_) in ((31, paw_y + 2, 0.9), (55, paw_y, 1.05)):
        ellipse(cv, px, py, 10 * s_, 7 * s_, 'pink', bias=-0.05)
        for k in range(4):
            cx = px - 6.0 * s_ + k * 4.0 * s_
            if V4:
                # 끝이 평평한 2×3 발톱, 오른쪽 그늘. 바깥 발톱은 한 줄 낮게(부채꼴). 뾰족한 크림색 줄은 이빨로 읽힌다.
                ty = int(py - 7.6 * s_) + (1 if k in (0, 3) else 0)
                for dy, row in enumerate(((2, 1), (3, 0), (2, 0))):
                    for dx, lv in enumerate(row):
                        cv.put(int(cx) + dx, ty + dy, 'claw', lv)
                continue
            ellipse(cv, cx, py - 6.2 * s_, 1.5, 2.6 * s_, 'claw', bias=0.12)
    if V4:
        # 정수리 한 가닥: 얼굴 쪽으로 기운 잎 모양(1px 선은 과일 꼭지로 읽혔다). 바깥선은 outline()이 두른다.
        top = next(y for y in range(H) if cv.mat[y][40] is not None)
        for dy, xs in ((-6, (43,)), (-5, (41, 42, 43)), (-4, (40, 41, 42)), (-3, (39, 40, 41, 42)),
                       (-2, (38, 39, 40, 41)), (-1, (36, 37, 38, 39, 40, 41, 42))):
            for x in xs:
                cv.put(x, top + dy, 'fur', 3 if x <= xs[0] + 1 else 2)
        for y in range(top, top + 2):
            for x in range(36, 43):
                if cv.mat[y][x] is None:
                    cv.put(x, y, 'fur', 2)
    outline(cv)
    inner_edges(cv, mats=('muzzle', 'pink', 'claw', 'scarf'))
    rim_light(cv)
    fur_strands(cv, 7)
    for (x, y) in ((70, 42), (71, 42), (70, 43), (71, 43)):    # 코 반사
        cv.paint(x, y, RAMP['pink'][3])
    for (x, y) in ((74, 51), (75, 51), (76, 50), (73, 52)):    # 코 아래 그늘
        cv.paint(x, y, RAMP['pink'][0])
    dither_ellipse(cv, 50, 51, 4.6, 2.4, BLUSH, 0.45)
    # 눈: 낮고 크게(아기 도식). 가까운 눈이 크다(3/4 시점).
    if face == 'happy':
        happy_eye2(cv, 51, 43, 5.0)
        happy_eye2(cv, 65, 40, 3.6)
    else:
        ry = 4.2 if determined else 6.4
        eye2(cv, 51, 43, 5.0, ry, outer=-1)
        eye2(cv, 65, 40, 3.6, ry * 0.82, look=(0.3, 0.0), outer=1)
        if determined:   # 윗눈꺼풀을 평평하게 눌러 결의
            line(cv, [(45, 39), (57, 40)], LID, 2)
            line(cv, [(61, 37), (69, 37)], LID, 2)
    if face == 'worried':
        line(cv, [(45, 33), (54, 31)], RAMP['fur'][0], 2)
        line(cv, [(62, 30), (68, 32)], RAMP['fur'][0], 2)
        for (x, y) in ((22, 24), (22, 25), (21, 26), (22, 26), (23, 26), (21, 27), (22, 27), (23, 27), (22, 28)):
            cv.paint(x, y, (170, 220, 255) if (x, y) != (22, 24) else (230, 248, 255))
    elif determined:
        line(cv, [(45, 33), (56, 36)], RAMP['fur'][0], 2)
        line(cv, [(61, 35), (69, 32)], RAMP['fur'][0], 2)
    else:
        line(cv, [(46, 34), (54, 33)], RAMP['fur'][1], 2)
        line(cv, [(62, 32), (68, 32)], RAMP['fur'][1], 2)
    if face == 'happy':
        for (x, y) in ((64, 55), (65, 56), (66, 57), (67, 57), (68, 57), (69, 56), (70, 55)):
            cv.paint(x, y, MOUTH)
        for (x, y) in ((66, 58), (67, 58), (68, 58)):
            cv.paint(x, y, RAMP['pink'][1])
    else:
        mouth = {'normal': [(64, 55), (66, 56), (67, 55), (68, 56), (70, 55)],
                 'worried': [(64, 56), (66, 55), (68, 56), (70, 55)],
                 'determined': [(65, 56), (70, 56)]}[face]
        line(cv, mouth, MOUTH, 1)
    return cv


def raccoon2(face):
    """포포 v2: 뾰족한 볼털, 바깥이 처진 가면, 흰 테두리 귀, 귀 뒤에 꽂은 잔가지 연필(가게 주인)."""
    cv = Canvas()

    def cheeks(x, y, ang):
        r = saw(ang, -3.0, -0.2, 10, 0.06)
        if -0.2 < ang < 1.25:
            r *= saw(ang, -0.2, 1.25, 7, 0.16, 1.2)
        if 1.9 < ang < 3.3:
            r *= saw(ang, 1.9, 3.3, 7, 0.16, 1.2)
        return r
    # 연필(노란 몸, 분홍 지우개, 깎은 나무 끝, 검은 심): 귀보다 먼저 그려 귀가 가운데를 가린다 = 귀 뒤에 꽂았다.
    dx_, dy_ = 0.92, 0.39
    for k in range(34):
        x0, y0 = 46 + k * dx_, 4 + k * dy_
        for w in range(4):
            x, y = int(x0 - dy_ * (w - 1.5)), int(y0 + dx_ * (w - 1.5))
            if k < 4:
                cv.put(x, y, 'pink', 2 if w < 2 else 1)          # 지우개
            elif k < 5:
                cv.put(x, y, 'claw', 1)                          # 쇠 띠
            elif k < 28:
                cv.put(x, y, 'pencil', 3 if w == 0 else (2 if w < 3 else 1))
            elif k < 32 and abs(w - 1.5) <= (32 - k) * 0.5:
                cv.put(x, y, 'twig', 3 if w < 2 else 2)          # 깎은 나무
            elif k >= 32 and abs(w - 1.5) <= 0.6:
                cv.put(x, y, 'mask', 1)                          # 심
    for (ex, ey) in ((20, 22), (60, 16)):
        ellipse(cv, ex, ey, 10, 11, 'white', bias=0.0)
        ellipse(cv, ex, ey + 1, 8.6, 9.6, 'grey', bias=0.0)
        flat(cv, ex + 1, ey + 2, 5.5, 6.5, 'mask', 1)
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


def momo_back2(face):
    """모모 v2: 먼 귀 끝이 접혔다(비대칭 = 성격), 뾰족한 볼털, 폭신한 가슴 털, 핀 꽃은 그대로, 속눈썹 있는 큰 눈."""
    cv = Canvas()

    def fluff(x, y, ang):
        r = saw(ang, -3.0, -0.3, 10, 0.05)
        if 0.15 < ang < 1.3:
            r *= saw(ang, 0.15, 1.3, 6, 0.12, 1.3)
        if 1.85 < ang < 3.0:
            r *= saw(ang, 1.85, 3.0, 6, 0.12, 1.3)
        return r
    # 먼 귀: 끝 1/3이 바깥(왼쪽)으로 접혀 내려온다
    ellipse(cv, 26, 25, 7, 14, 'bfur', bias=-0.08, clip=lambda x, y: y < 40)
    flat(cv, 27, 27, 3.4, 8, 'pink', 0)
    # 접힌 끝: 귀 꼭대기에서 바깥(왼쪽)으로 꺾여 내려온다. 바깥 털이 보이니 한 단 어둡게, 귀와 붙여 그린다.
    for y in range(9, 21):
        for x in range(12, 30):
            t = (x - 12) / 17.0
            top = 13 - 6 * math.sin(t * math.pi * 0.9)
            if top <= y <= top + 5.5 and not (x > 26 and y > 15):
                cv.put(x, y, 'bfur', 1 if y > top + 2 else 2)
    ellipse(cv, 50, 18, 8, 18, 'bfur', bias=0.04, clip=lambda x, y: y < 40)
    flat(cv, 51, 19, 4.0, 13, 'pink', 1)
    flat(cv, 51, 16, 2.0, 8, 'pink', 2)
    ellipse(cv, 42, 54, 29, 24, 'bfur', bias=0.05, rough=fluff)
    def ruff(x, y, ang):
        return saw(ang, -3.1, -0.05, 9, 0.18, 1.2)
    ellipse(cv, 40, 92, 22, 10, 'bfur', bias=-0.15)
    ellipse(cv, 40, 89, 14, 7, 'blight', bias=-0.1, rough=ruff)
    ellipse(cv, 61, 62, 5.5, 4.5, 'blight', bias=-0.05)
    ellipse(cv, 66, 58, 2.4, 1.9, 'pink', bias=-0.1)
    outline(cv)
    rim_light(cv, 'bfur')
    fx, fy = 53, 23
    for (cx, cy) in ((0, -4), (-4, -1), (4, -1), (-3, 4), (3, 4)):
        flat(cv, fx + cx, fy + cy, 2.8, 2.5, 'bloom', 2)
        cv.paint(fx + cx - 1, fy + cy - 1, RAMP['bloom'][3])
        cv.paint(fx + cx + 1, fy + cy + 1, RAMP['bloom'][1])
    for (ox, oy) in ((-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1), (0, -1)):
        cv.paint(fx + ox, fy + oy, (255, 222, 120))
    cv.paint(fx, fy, (236, 180, 70))
    for (x, y) in ((fx + 6, fy + 3), (fx + 7, fy + 4), (fx + 7, fy + 5), (fx + 6, fy + 5)):
        cv.paint(x, y, (110, 170, 90))
    if face == 'happy':
        happy_eye2(cv, 39, 49, 5.6)
        happy_eye2(cv, 58, 46, 4.2)
    else:
        eye2(cv, 39, 49, 5.6, 6.8, outer=-1, lash=True)
        eye2(cv, 58, 46, 4.2, 5.6, look=(0.3, 0.0), outer=1, lash=True)
    if face == 'worried':
        line(cv, [(33, 39), (44, 36)], RAMP['bfur'][0], 2)
        line(cv, [(54, 35), (62, 37)], RAMP['bfur'][0], 2)
        line(cv, [(60, 69), (63, 68), (66, 69)], MOUTH, 1)
    elif face == 'happy':
        for (x, y) in ((59, 67), (60, 68), (61, 69), (62, 69), (63, 69), (64, 68), (65, 67)):
            cv.paint(x, y, MOUTH)
        for (x, y) in ((61, 70), (62, 70), (63, 70)):
            cv.paint(x, y, RAMP['pink'][2])
    else:
        line(cv, [(34, 39), (43, 38)], RAMP['bfur'][1], 2)
        line(cv, [(54, 37), (61, 37)], RAMP['bfur'][1], 2)
        line(cv, [(60, 68), (62, 69), (64, 69), (66, 68)], MOUTH, 1)
    dither_ellipse(cv, 48, 61, 4.6, 2.4, BLUSH, 0.45)
    dither_ellipse(cv, 30, 59, 3.2, 1.8, BLUSH, 0.4)
    return cv


def nail_fur(face, fur):
    """털빛을 바꾼 Naïl: 'fur' 경사만 잠깐 바꿔 그리고, 색으로 굳혀 둔다(시트에 넣은 뒤 경사를 되돌려도 남게)."""
    saved = RAMP['fur']
    RAMP['fur'] = FUR_VARIANTS[fur]
    cv = (nail2 if V2 else nail)(face)
    for y in range(H):
        for x in range(W):
            if cv.col[y][x] is None and cv.mat[y][x] is not None:
                mat, lvl = cv.mat[y][x]
                cv.col[y][x] = RAMP[mat][lvl] if mat in RAMP else OUTLINE_BASE
    RAMP['fur'] = saved
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
    nail_, raccoon_, momo_back_ = (nail2, raccoon2, momo_back2) if V2 else (nail, raccoon, momo_back)
    frames = [nail_(f) for f in ('normal', 'happy', 'worried', 'determined')] + [neighbor()]
    frames += [raccoon_('normal'), raccoon_('happy')]
    frames += [momo_tainted('cold'), momo_tainted('pained')]
    frames += [bori('tainted'), bori('normal'), bori('happy')]
    frames += [momo_back_('normal'), momo_back_('happy'), momo_back_('worried')]
    frames += [halmae('normal'), halmae('happy'), mom()]
    for fur in (1, 2, 3):
        frames += [nail_fur(f, fur) for f in ('normal', 'happy', 'worried', 'determined')]
    frames += [doto('tainted'), doto('normal'), doto('happy')]
    frames += [susu('normal'), susu('happy')]
    frames += [kongi('tainted'), kongi('normal'), kongi('happy')]
    # 한 줄 12칸(src/art.rs PORTRAIT_COLS): 한 줄로 늘어놓으면 2,048px를 넘어 오래된 폰에서 텍스처가 안 올라간다.
    cols = 12
    rows = (len(frames) + cols - 1) // cols
    out = Image.new('RGBA', (W * cols, H * rows), (0, 0, 0, 0))
    for i, cv in enumerate(frames):
        out.paste(to_image(cv), ((i % cols) * W, (i // cols) * H))
    return out


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/portraits.png')
