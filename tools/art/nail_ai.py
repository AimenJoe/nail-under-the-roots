"""Naïl 스프라이트 시트(32×32 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206 → 0238 → 0308).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png. 털빛 시트 nail_fur*.png는 0272에서 뺐다)

옮기는 법(색표·다수결 줄이기·효과 지우기·손발 색·흔들기·숨쉬기)은 공용 변환기 `sprite_ai.py`에 있고, 여기는 Naïl의
설정(`SPEC`)과 시트 배치뿐이다. 왜 이 길인가(0203 → 0206 → 0238): 16칸 손 그림은 "두더지 같지 않다"(사장) → 실루엣 시안 1
(`nail_s1_v2`, 몸 16px) → 사장 "처음부터 다시"(0238): 큰 그림 다섯 안에서 고른 1번(서서 삽 손을 벌린 몸 + 주황 목도리)을
시트 둘(`nail_v3_a`·`nail_v3_b`)로 받아 키 약 22px로(모모 23·도토 25와 나란히). 0242: 머리를 키운 시험(`chibi`, 사장 "좋은데?") →
그 비율로 처음부터 그린 v4 시트 둘(`nail_v4_a`·`nail_v4_b`, 2.5등신·큰 눈, `SPEC` — 지금은 `build_v4`로만). 원본 손발이 이미 분홍·크림이라 `Limbs`·`swing`은 없다.
0308: 초상화 얼굴에서 다시 그린 v5(`nail_v5_a`·`_b` + 걷기 영상 띠 `nail_v5_walk`)를 지금 쓴다(`build` = `build_v5`): 키 24px, 눈은 원본의
반짝 자리를 재서 같은 도장으로, 외곽선은 곁의 털색으로 물들이고 안쪽 선을 원본에서 옮긴다. 머리는 `HEAD_V5`배(이웃과 나란히 보아 고른 값).

칸 배치(게임 코드는 칸 크기 `art::NAIL_CELL`만 안다):
  줄 0: 서기 2, 걷기 4, 파고들기 2(숙임·머리부터), 솟구치기 / 줄 1: 치기 3, 맞음, 쓰러짐, 들기 3, 거두기 / 줄 2: 흙더미 2, 그림자(16칸 그림을 칸 아래 가운데에)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import nail as legacy  # noqa: E402  흙더미·그림자 16칸 그림을 같이 쓴다
from sprite_ai import Source, Spec, convert, magenta, seat, seat_on_fur, step_cycle  # noqa: E402

SPRITES = HERE / "sources" / "ai" / "sprites"
CELL = 32          # 키 24px + 뻗은 발톱·날리는 목도리(0308: v5의 치기 칸이 28칸 가장자리에 닿았다. 0238의 28칸은 v4까지)
FOOT = 30          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(`art::nail_origin`).

# 원본(0238, 사장 고름 1단계 1번 → 2단계 시트 둘, ✦ 지움). 칸 상자는 redo_concepts.components로 찾은 값.
# 시트 A(3줄 × 4): 줄 1 서기·서기(숨)·맞음·쓰러짐, 줄 2·3 걷기(줄 2 첫 칸만 쓴다 — 네 칸이 거의 같고 앞발만 튀어서, 0241). 서기 두 칸은 벌린 발톱끼리 붙어 있어
# 서기 A만 잘라(오른쪽 발톱 끝까지) 옆 칸 발톱 조각은 `tidy`로 버린다. 서기 B는 `breathe`로 만든다(따로 그린 두 칸은 숨이 끊긴다).
# 시트 B(3줄 × 4): 줄 1 들기·치기·들기 2·(쓰지 않음), 줄 2 치기 2·강타 들기(머리 높이 — 더 높이 든 칸은 몸을 크게 그려 와 안 쓴다)·(쓰지 않음)·강타, 줄 3 거두기·(서기)·파고들기·솟구치기.
# 머리 키우기(0242, 사장 "머리 비율을 늘려 귀엽게"): 22px에서 원본 비율(머리 ≈ 키의 1/4)은 얼굴이 1px 눈으로 줄었다.
# 줄이기 전에 원본(큰 그림)에서 머리만 HEAD배 키우고 눈을 크게(흰 반짝 포함) 다시 그린 뒤 같은 변환기로 줄인다 —
# 줄인 도트를 손보면 계단이 생기지만 큰 그림에서 키우면 다수결 줄이기가 선을 다시 고른다. 키는 BODY_H로 2~3px 커진다.
# 머리 = 목도리(주황) 윗쪽 털·주둥이 덩어리 중 맨 위 털에 이어진 것(발톱 크림은 뺀다), 기준점 = 목도리 윗선의 가운데.
# 누운 칸(쓰러짐)은 머리가 옆이라 그대로.
HEAD = 1.35
EYE = (9, 11)       # 원본 px 반지름(가로, 세로)
NO_HEAD = {"dead"}
THIN = 5            # 머리 안 선 깎기(MinFilter 크기): 양쪽 2px씩 — 1.35배로 굵어진 만큼
OUTLINE_BAND = 13    # 키운 머리 가장자리에서 이 폭(원본 px, 홀수) 안의 어두운 점 = 외곽선(원본 외곽선 ≈ 6px × 1.35)


def _flood(mask, seed):
    out = np.zeros_like(mask)
    stack = [seed]
    H, W = mask.shape
    while stack:
        y, x = stack.pop()
        if y < 0 or x < 0 or y >= H or x >= W or out[y, x] or not mask[y, x]:
            continue
        out[y, x] = True
        stack += [(y + 1, x), (y - 1, x), (y, x + 1), (y, x - 1)]
    return out


def chibi(img, frames, ref=False):
    """칸마다 머리를 키운 그림을 새 판에 칸씩 세로로 쌓는다(넓힌 상자가 옆 칸을 덮지 않게). → (새 판, 새 칸 상자).
    ref: 참고 그림용(`chibi_ref.py`) — 원래 머리를 지우거나 외곽선을 털로 칠하지 않고 큰 그림 그대로 위에 얹는다(줄이기용 손질은
    큰 그림에서 보면 선이 끊겨 보여, 나노바나나가 그 흠까지 따라 그린다)."""
    slots, out = [], {}
    for k, (x0, y0, x1, y1) in frames.items():
        a = img[y0:y1, x0:x1]
        if k in NO_HEAD:
            slots.append((k, Image.fromarray(a.astype(np.uint8), "RGB")))
            continue
        r, g, b = a[..., 0], a[..., 1], a[..., 2]
        fg = magenta(a) < 60
        orange = fg & (r > 150) & (g > 60) & (g < 165) & (b < 100) & (r - b > 90)
        cream = (r > 190) & (g > 175) & (b < 175) & (r - b > 40)
        oy, ox = np.nonzero(orange)
        cx = int(np.median(ox))
        yn = int(oy[np.abs(ox - cx) <= 12].min())     # 목도리 윗선(가운데)
        rows = np.arange(a.shape[0])[:, None]
        cand = fg & ~cream & (rows < yn) & (~orange if not ref else True)   # 참고 그림은 뒤통수 목도리 매듭도 머리와 같이 키운다
        ty, tx = np.nonzero(cand)
        top = int(ty.min())
        # 머리 폭: 머리 윗부분(맨 위 ~ 목도리까지의 위 40%) 털의 좌우 끝에서 조금 더. 벌린 앞발(어깨 높이)은 이 밖이다.
        upper = cand & (rows < top + (yn - top) * 0.4)
        uy, ux = np.nonzero(upper)
        pad = (ux.max() - ux.min()) * 0.25
        cols = np.arange(a.shape[1])[None, :]
        cand &= (cols >= ux.min() - pad) & (cols <= ux.max() + pad)
        head = _flood(cand, (int(ty[ty.argmin()]), int(tx[ty.argmin()])))
        hy, hx = np.nonzero(head)
        ax, ay = float(hx[hy >= hy.max() - 6].mean()), float(yn)
        # 눈: 머리 속(외곽선에서 5px 안)의 어두운 점 덩어리 중 가장 큰 것. 감은 눈(가는 선)은 작아서 그리지 않는다.
        inner = np.asarray(Image.fromarray((fg * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(11))) > 0
        # 오른쪽을 보는 그림 — 눈은 머리 앞쪽 절반, 위쪽 2/3에 있다(뒤통수의 귀·목도리 매듭 그늘을 눈으로 잡지 않게).
        front = (cols >= hx.mean()) & (rows < hy.min() + (hy.max() - hy.min()) * 0.7)
        dark = head & inner & front & (a.sum(-1) < 170)
        eye = None
        if dark.sum() >= 12:
            dy, dx = np.nonzero(dark)
            best = None
            for y, x in zip(dy, dx):
                n = ((np.abs(dy - y) <= 5) & (np.abs(dx - x) <= 4)).sum()
                if best is None or n > best[0]:
                    best = (n, y, x)
            n, by, bx = best
            if n >= 12:
                close = (np.abs(dy - by) <= 6) & (np.abs(dx - bx) <= 5)
                eye = (dx[close].mean(), dy[close].mean())
        layer = np.zeros(a.shape[:2] + (4,), np.uint8)
        layer[..., :3] = a
        layer[..., 3] = head * 255
        L = Image.fromarray(layer, "RGBA")
        L = L.resize((round(L.width * HEAD), round(L.height * HEAD)), Image.NEAREST)
        px, py = round(ax - ax * HEAD), round(ay - ay * HEAD)
        up, side = max(0, -py) + 4, max(0, -px, px + L.width - a.shape[1]) + 4
        # 원래 머리는 지우고 붙인다 — 남겨 두면 키운 머리 밑으로 옛 턱·주둥이 외곽선이 비져 나와 검은 쐐기가 됐다(사장).
        body = a.copy()
        if not ref:
            body[head] = (255, 0, 255)
        canvas = Image.new("RGB", (a.shape[1] + 2 * side, a.shape[0] + up), (255, 0, 255))
        canvas.paste(Image.fromarray(body.astype(np.uint8), "RGB"), (side, up))
        la = np.array(L)
        m = la[..., 3] > 0
        band = m & ~(np.asarray(Image.fromarray((m * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(OUTLINE_BAND))) > 0)
        dark = la[..., :3].astype(int).sum(-1) < 170
        # 키운 외곽선(≈8px)은 변환기가 걷는 폭(`strip` 6)보다 굵어, 남은 선 + 변환기 외곽선이 두 겹이 됐다. 빼서 투명으로 두면
        # 목에 틈이 생겨 변환기가 틈 양쪽에 선을 둘러 더 검어졌다 → 외곽선 띠는 둘레 털 색으로 칠하고(실루엣은 그대로),
        # 변환기가 바탕 쪽에만 1px 외곽선을 새로 두르게 한다.
        # 머리 안의 선(주둥이와 얼굴 사이·귀)도 1.35배 굵어져 → 한 겹 깎아 원래 굵기로.
        inner_dark = dark & m & ~band
        core = np.asarray(Image.fromarray((inner_dark * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(THIN))) > 0
        shave = (inner_dark & ~core) | (band & dark)
        if shave.any() and not ref:
            light = (m & ~dark).astype(float)
            col = la[..., :3].astype(float) * light[..., None]
            win = 15     # 외곽선 띠(≈8px) 한가운데서도 털 점이 잡히게
            pad = win // 2
            cs = np.pad(col, ((pad, pad), (pad, pad), (0, 0)))
            ws = np.pad(light, pad)
            acc = np.zeros_like(col)
            wacc = np.zeros_like(light)
            for dy in range(win):
                for dx in range(win):
                    acc += cs[dy:dy + col.shape[0], dx:dx + col.shape[1]]
                    wacc += ws[dy:dy + col.shape[0], dx:dx + col.shape[1]]
            fill = shave & (wacc > 0)
            la[fill, :3] = (acc[fill] / wacc[fill][:, None]).astype(np.uint8)
            la[shave & ~fill, 3] = 0
        L = Image.fromarray(la, "RGBA")
        canvas.paste(L, (px + side, py + up), L)
        if eye:
            ex, ey = (eye[0] - ax) * HEAD + ax + side, (eye[1] - ay) * HEAD + ay + up
            d = ImageDraw.Draw(canvas)
            rx, ry = EYE
            d.ellipse((ex - rx, ey - ry, ex + rx, ey + ry), fill=(20, 12, 16))
            # 반짝은 눈 안쪽 위(뒤통수 쪽) — 앞쪽에 두면 줄였을 때 머리 외곽선 밖 흰 점이 됐다.
            d.ellipse((ex - rx + 2, ey - ry + 3, ex - rx + 2 + rx * 0.8, ey - ry + 3 + rx * 0.8), fill=(250, 250, 250))
        slots.append((k, canvas))
    W = max(c.width for _, c in slots)
    sheet = Image.new("RGB", (W, sum(c.height + 8 for _, c in slots)), (255, 0, 255))
    y = 0
    for k, c in slots:
        sheet.paste(c, (0, y))
        out[k] = (0, y, c.width, y + c.height)
        y += c.height + 8
    return np.asarray(sheet).astype(int), out


SHEET_A = SPRITES / "nail_v3_a.png"
SHEET_B = SPRITES / "nail_v3_b.png"
# v3 + 머리 키우기(0242 첫판) — 지금 게임은 v4(아래 `SPEC`). 참고 그림 `chibi_ref.py`가 이 설정을 쓴다.
SPEC_V3 = Spec(
    sources=[
        Source(
            SHEET_A,
            {
                "idle_a": (7, 52, 276, 330),
                "hurt": (522, 65, 739, 326),
                "dead": (738, 166, 1001, 331),
                "walk_a": (20, 413, 238, 686),   # 걷기는 이 한 칸에서 `sprite_ai.step_cycle`로 네 칸(0241)
            },
            # 배율은 서 있는 키(머리~발)로: 서기·맞음·쓰러짐은 서기 A, 걷기는 걷기 A(줄마다 그려 온 크기가 조금씩 다르다).
            groups={"idle_a": "idle_a", "hurt": "idle_a", "dead": "idle_a",
                    "walk_a": "walk_a"},
            tidy=0.15,
            strip=6,
            prep=chibi,
        ),
        # 시트 B는 발을 머리 위로 든 칸이 많아 겉 네모 키가 칸마다 다르다 → 똑바로 선 거두기 칸의 키로 한 배율.
        Source(
            SHEET_B,
            {
                "windup0": (30, 51, 242, 325),
                "attack0": (253, 67, 556, 325),
                "windup1": (531, 60, 751, 325),
                "attack1": (20, 413, 303, 686),
                "windup2": (271, 413, 498, 686),
                "attack2": (751, 447, 985, 686),
                "settle": (20, 746, 242, 1009),
            },
            groups=dict.fromkeys(
                ("windup0", "attack0", "windup1", "attack1", "windup2", "attack2", "settle"), "settle"
            ),
            tidy=0.15,
            strip=6,
            prep=chibi,
        ),
    ],
    palette_ref=(0, "idle_a"),
    body_h=24,          # 머리를 키운 키(0242). 몸은 예전 21일 때와 거의 같은 크기
    breathe=("idle_a", "idle_b"),
)


# v4(0242, 사장 고름): 처음부터 2.5등신·큰 눈으로 받은 시트 둘. 머리 키우기(`chibi`) 없이 그대로 줄인다.
# 시트 A(2줄 × 4): 줄 1 들기 1·치기 1·들기 2·치기 2, 줄 2 서기·서기 2(안 씀)·파고들기(안 씀)·강타 들기.
# 시트 B(2줄 × 4): 줄 1 맞음·쓰러짐·강타·거두기, 줄 2 걷기 넷. 칸 상자는 redo_concepts.components로 찾은 값.
SHEET_V4_A = SPRITES / "nail_v4_a.png"
SHEET_V4_B = SPRITES / "nail_v4_b.png"
SPEC = Spec(
    sources=[
        Source(
            SHEET_V4_A,
            {
                "windup0": (38, 34, 239, 265),
                "attack0": (281, 34, 517, 265),
                "windup1": (542, 34, 735, 265),
                "attack1": (768, 34, 986, 265),
                "idle_a": (29, 294, 252, 530),
                "windup2": (789, 290, 995, 538),
            },
            groups=dict.fromkeys(("windup0", "attack0", "windup1", "attack1", "idle_a", "windup2"), "idle_a"),
            tidy=0.15,
            strip=6,
        ),
        # 시트 B는 따로 받아 그려 온 크기가 조금 다르다 → 똑바로 선 거두기 칸의 키로 한 배율.
        Source(
            SHEET_V4_B,
            {   # 줄 2 걷기 넷 중 첫 칸만 쓴다: (289, 294, 487, 530) · (542, 294, 751, 530) · (789, 289, 986, 538)는 안 씀
                "hurt": (46, 34, 239, 265),
                "dead": (272, 93, 517, 265),
                "attack2": (546, 38, 764, 265),
                "settle": (793, 34, 986, 265),
                "walk_1": (29, 294, 252, 530),
            },
            groups=dict.fromkeys(("hurt", "dead", "attack2", "settle", "walk_1"), "settle"),
            tidy=0.15,
            strip=6,
        ),
    ],
    palette_ref=(0, "idle_a"),
    body_h=19,      # 외곽선까지 키 약 21px — 모모 23·도토 25·포포 26보다 작게(사장 "아기라는 느낌이 아니라 좀 줄여야", 0242)
    breathe=("idle_a", "idle_b"),
)


def small(grid_text):
    """16칸 그림(흙더미·그림자)을 칸 아래 가운데에."""
    rows = legacy.outline(legacy.grid(grid_text))
    cell = np.zeros((CELL, CELL, 4), np.uint8)
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == ".":
                continue
            c = legacy.P[ch]
            c = c if len(c) == 4 else c + (255,)
            cell[CELL - 16 + y, (CELL - 16) // 2 + x] = c
    return cell


# 점 다듬기(0242, 사장 "스프라이트 하나씩 보면서 다듬어"): 변환 뒤 칸마다 손볼 점. (x, y) → 색 이름(`INK`) 또는 None(지움).
# 칸 좌표는 28칸 그대로(왼쪽 위 0,0). PNG를 손으로 고치지 않고 여기 적어 두면 다시 만들어도 남는다.
INK = {   # v4 색표의 점
    "line": (43, 29, 42), "fur": (82, 56, 57), "eye": (0, 0, 0), "shine": (249, 225, 190),
}


def eye(x, y, clear=()):
    """눈 도장: 가로 2 × 세로 3 검은 점 + 오른쪽 위 반짝 한 점(왼쪽 위가 (x, y)). clear는 털로 덮을 옛 눈 점.
    v4를 줄이면 칸마다 눈이 "+"·고리·세모로 달라져(반짝이 가운데·옆에 남아) 얼굴이 칸마다 바뀌어 보였다."""
    t = {p: "fur" for p in clear}
    t.update({(x, y): "eye", (x + 1, y): "shine", (x, y + 1): "eye", (x + 1, y + 1): "eye",
              (x, y + 2): "eye", (x + 1, y + 2): "eye"})
    return t


# 눈 도장을 찍을 칸(맞음·쓰러짐은 눈을 감았다). 자리는 줄인 칸의 검은 점 덩어리 가운데에서 찾는다(`auto_eye`) —
# 키(`body_h`)를 바꾸면 눈 자리도 바뀌어, 손으로 적은 좌표는 매번 틀어졌다(0242 크기 줄이기).
EYE_FRAMES = ("idle_a", "idle_b", "walk_1", "attack0", "attack1", "attack2", "windup0", "windup1", "windup2", "settle")
TOUCH = {}


def auto_eye(cell):
    """머리(위 절반)의 새까만 점 덩어리 가운데에 `eye` 도장. 덩어리의 다른 검은 점은 털로."""
    a = cell[: CELL // 2]
    blk = (a[..., 3] > 0) & (a[..., :3].astype(int).sum(-1) == 0)
    ys, xs = np.nonzero(blk)
    if len(ys) == 0:
        return {}
    cx, cy = xs.mean(), ys.mean()
    near = (np.abs(xs - cx) <= 2.5) & (np.abs(ys - cy) <= 2.5)
    x, y = int(round(xs[near].mean() - 0.5)), int(round(ys[near].mean() - 1.0))
    return eye(x, y, [(int(px), int(py)) for px, py in zip(xs[near], ys[near])])


def fill_holes(cell):
    """실루엣 안의 1~2px 구멍(위아래 또는 좌우가 막힌 투명 점)을 외곽선 색으로 메운다 — 머리를 붙인 목·뒤통수에 남던 틈."""
    out = cell.copy()
    for _ in range(2):
        on = out[..., 3] > 0
        H, W = on.shape
        for y in range(1, H - 1):
            for x in range(1, W - 1):
                if on[y, x]:
                    continue
                n = on[y - 1, x] + on[y + 1, x] + on[y, x - 1] + on[y, x + 1]
                if n >= 3 or (on[y, x - 1] and on[y, x + 1] and (on[y - 1, x] or on[y + 1, x])):
                    out[y, x] = INK["line"] + (255,)
    return out


def touch(cell, name):
    out = fill_holes(cell)
    marks = dict(auto_eye(out)) if name in EYE_FRAMES else {}
    marks.update(TOUCH.get(name, {}))
    for (x, y), ink in marks.items():
        out[y, x] = (0, 0, 0, 0) if ink is None else INK[ink] + (255,)
    return out


# v5(사장 2026-11-27 "아직 예쁘지 않거든, 주인공인데도"): **초상화를 붙여** 받은 여섯 안 가운데 6번(뾰족 털 머리·밝은 주둥이·큰 눈)으로
# 받은 시트 둘. v4는 머리 키운 옛 스프라이트를 붙여 받아 거의 검은 갈색·옆얼굴·점 하나 눈이 따라왔다. 아기 느낌은 키가 아니라
# 귀여움(큰 머리·큰 눈·둥근 몸)이라 키를 줄이지 않는다(`BODY_H_V5`).
# 시트 A(2줄 × 4): 줄 1 서기·서기 숨(안 씀 — `breathe`)·맞음·쓰러짐, 줄 2 걷기 넷. 시트 B(3줄 × 3): 들기 1·치기 1·들기 2 /
# 치기 2·강타 들기·강타 / 거두기·파고들기(안 씀 — 영상에서 뽑은 두 칸을 쓴다)·솟구치기. 상자는 그림 둘레 + 8px.
SHEET_V5_A = SPRITES / "nail_v5_a.png"
SHEET_V5_B = SPRITES / "nail_v5_b.png"
BODY_H_V5 = 24
_V5_A = {
    "idle_a": (34, 5, 335, 380), "hurt": (718, 12, 1007, 379), "dead": (1035, 104, 1362, 379),
    "walk_1": (42, 382, 322, 752), "walk_2": (388, 380, 668, 752), "walk_3": (723, 380, 1003, 756),
    "walk_4": (1063, 381, 1344, 751),
}
_V5_B = {
    "windup0": (56, 5, 290, 297), "attack0": (484, 5, 752, 297), "windup1": (894, 5, 1123, 297),
    "attack1": (58, 300, 327, 597), "windup2": (493, 304, 734, 597), "attack2": (882, 316, 1148, 605),
    "settle": (62, 608, 286, 888), "burst": (910, 593, 1148, 896),
}
# 걷기 영상(사장이 받은 GIF 30칸·10fps, 한 바퀴 ≈ 11.5칸)에서 네 칸(0·3·6·9)을 뽑아 바탕을 순수 마젠타로 정리한 띠.
# 영상은 같은 몸이 이어져 움직여 칸마다 얼굴·몸이 달라지지 않는다. 목도리 끝이 뒤로 날린다(서 있을 때는 늘어져 있다).
SHEET_V5_WALK = SPRITES / "nail_v5_walk.png"
_V5_W = {
    "walkg_1": (13, 3, 249, 272), "walkg_2": (282, 11, 520, 273), "walkg_3": (551, 5, 793, 272),
    "walkg_4": (822, 11, 1059, 273),
}
# 파고들기 영상(사장이 받은 GIF 30칸, 640×360)에서 뽑은 칸: 23(다시 선 모습 — 배율의 기준으로만), 5(앞으로 숙임), 8(머리부터 내려감).
# ✦를 지우고 바탕을 순수 마젠타로 정리한 띠. 솟구치는 칸(11~19)은 쓰지 않는다 — 몸이 정면으로 돌고 길게 늘어나 다른 동물이 됐다(0308).
SHEET_V5_DIVE = SPRITES / "nail_v5_dive.png"
_V5_D = {"dive_ref": (120, 21, 440, 360), "dive_a": (801, 51, 1144, 360), "dive_b": (1439, 13, 1754, 357)}
# 맞음 → 쓰러짐 영상(GIF 30칸, 16:9 첫 칸으로 받아 카메라가 안 움직였다)에서: 0(서기 — 배율 기준), 12(뒤로 밀려나며 눈을 꼭 감음), 25(등을 대고 누움).
# 영상이 그려 넣은 흰 충격 효과가 있는 칸(5~7)은 피했다.
SHEET_V5_FALL = SPRITES / "nail_v5_fall.png"
_V5_F = {"fall_ref": (186, 73, 378, 313), "hurtg": (793, 68, 1013, 315), "deadg": (1391, 168, 1666, 334)}
# 머리가 위에 있지 않은 칸은 `shrink_head`가 머리 띠를 못 찾는다. 파고들기는 기준 칸(dive_ref)도 줄이지 않는다 — 줄이면 배율이
# 커져 줄지 않은 머리가 더 커진다(머리 +18%). 그대로 두면 온몸 크기가 맞고 머리 +7%·몸 −9%쯤(0.1초 스치는 칸이라 받아들인다).
NO_SHRINK = {"dead", "deadg", "dive_ref", "dive_a", "dive_b"}
# 목도리가 가려 머리 띠를 못 찾는 칸은 목 자리를 적어 준다: 칸 상자 안의 (목 줄 y, 목 가운데 x), 원본 px.
# 맞음 칸(hurtg)은 든 두 발이 목도리를 가린다 — 발도 머리 띠 안이라 같이 줄어든다.
NECK = {"hurtg": (150, 125)}
SPEC_V5 = Spec(
    sources=[
        Source(SHEET_V5_A, _V5_A, groups=dict.fromkeys(_V5_A, "idle_a"), tidy=0.15, strip=6),
        # 시트 B는 따로 받아 그려 온 크기가 조금 다르다 → 똑바로 선 거두기 칸의 키로 한 배율.
        Source(SHEET_V5_B, _V5_B, groups=dict.fromkeys(_V5_B, "settle"), tidy=0.15, strip=6),
        # 영상은 그린 크기가 또 다르다(키 약 250px) → 첫 칸의 키로. 외곽선이 가늘어(≈4px) 덜 벗긴다.
        Source(SHEET_V5_WALK, _V5_W, groups=dict.fromkeys(_V5_W, "walkg_1"), tidy=0.15, strip=4),
        Source(SHEET_V5_DIVE, _V5_D, groups=dict.fromkeys(_V5_D, "dive_ref"), tidy=0.15, strip=3),
        Source(SHEET_V5_FALL, _V5_F, groups=dict.fromkeys(_V5_F, "fall_ref"), tidy=0.15, strip=3),
    ],
    palette_ref=(0, "idle_a"),
    body_h=BODY_H_V5,
    breathe=("idle_a", "idle_b"),
)
INK_V5 = {"eye": (30, 22, 48), "shine": (255, 255, 255), "blush": (232, 140, 150)}


def _blobs(mask):
    """붙어 있는 점 덩어리들(큰 것부터)."""
    seen = np.zeros_like(mask)
    H, W = mask.shape
    out = []
    for y, x in zip(*np.nonzero(mask)):
        if seen[y, x]:
            continue
        stack, pts = [(y, x)], []
        while stack:
            cy, cx = stack.pop()
            if cy < 0 or cx < 0 or cy >= H or cx >= W or seen[cy, cx] or not mask[cy, cx]:
                continue
            seen[cy, cx] = True
            pts.append((cy, cx))
            stack += [(cy + 1, cx), (cy - 1, cx), (cy, cx + 1), (cy, cx - 1)]
        out.append(pts)
    return sorted(out, key=len, reverse=True)


def eye_shines(img, box):
    """원본 칸에서 눈의 흰 반짝을 찾는다 → [(x, y)] 큰 것(가까운 눈)부터, 그림 둘레 상자 기준 0..1.

    큰 눈은 24px로 줄이면 칸마다 다른 얼룩이 되고 색표(서기 칸 기준)에 눈 색이 없어 분홍·갈색으로 바뀐다. 원본에서 눈 자리를 재서
    줄인 칸의 같은 자리에 같은 눈을 찍는다. 흰색(세 값 다 205 넘음)은 머리에서 눈 반짝뿐이다(발톱은 크림). 감은 눈(맞음·쓰러짐)은 없다."""
    x0, y0, x1, y1 = box
    a = img[y0:y1, x0:x1]
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    fg = ~((r > 170) & (g < 110) & (b > 170))
    ys, xs = np.nonzero(fg)
    fx0, fx1, fy0, fy1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    white = (np.minimum(np.minimum(r, g), b) > 205) & fg
    white[int(fy0 + (fy1 - fy0) * 0.55):] = False
    out = []
    for pts in _blobs(white)[:2]:
        if len(pts) >= 10:
            py, px = np.mean([q[0] for q in pts]), np.mean([q[1] for q in pts])
            out.append(((px - fx0) / (fx1 - fx0), (py - fy0) / (fy1 - fy0)))
    return out


def stamp_face(spr, shines):
    """줄인 그림(앉히기 전)에 눈과 볼 홍조를 찍는다. 가까운 눈 = 가로 3 × 세로 3 짙은 남색 + 왼쪽 위 흰 반짝(반짝 자리가 기준),
    먼 눈 = 세로 2 + 위 반짝, 홍조 = 가까운 눈 왼쪽 아래 분홍 두 점."""
    out = spr.copy()
    ys, xs = np.nonzero(out[..., 3] > 0)
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1

    def put(x, y, ink):
        if 0 <= y < out.shape[0] and 0 <= x < out.shape[1] and out[y, x, 3] > 0:
            out[y, x] = INK_V5[ink] + (255,)

    for i, (nx, ny) in enumerate(shines):
        ex, ey = int(x0 + nx * (x1 - x0)), int(y0 + ny * (y1 - y0))
        if i == 0:
            for dy in range(3):
                for dx in range(3):
                    put(ex + dx, ey + dy, "eye")
            put(ex, ey, "shine")
            put(ex - 2, ey + 3, "blush")
            put(ex - 1, ey + 3, "blush")
        else:
            put(ex, ey, "shine")
            put(ex, ey + 1, "eye")
            put(ex + 1, ey, "eye")
            put(ex + 1, ey + 1, "eye")
    return out


# 감은 눈(사장 2026-11-27 "눈 감는 게 좀 어색하다"): 맞음·쓰러짐 칸은 원본의 가는 감은 눈 선이 줄이면 사라져 얼굴이 민둥해졌다.
# 눈 자리(원본 그림 둘레 상자 기준 0..1, 원본을 보고 적은 값)에 읽히는 모양을 찍는다: 맞음 = 질끈 감은 ">", 쓰러짐 = "×".
CLOSED_EYES = {"hurt": ((0.34, 0.30), ">"), "dead": ((0.24, 0.34), "x"), "hurtg": ((0.36, 0.30), ">"), "deadg": ((0.22, 0.38), "x")}
_SHUT = {">": ((0, 0), (1, 0), (2, 1), (0, 2), (1, 2)), "x": ((0, 0), (2, 0), (1, 1), (0, 2), (2, 2))}


def stamp_shut(spr, at, kind):
    """줄인 그림(앉히기 전)에 감은 눈과 볼 홍조를 찍는다."""
    out = spr.copy()
    ys, xs = np.nonzero(out[..., 3] > 0)
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    ex, ey = int(x0 + at[0] * (x1 - x0)), int(y0 + at[1] * (y1 - y0))
    for dx, dy in _SHUT[kind]:
        out[ey + dy, ex + dx] = INK_V5["eye"] + (255,)
    for dx in (-2, -1):
        if out[ey + 3, ex + dx, 3] > 0:
            out[ey + 3, ex + dx] = INK_V5["blush"] + (255,)
    return out


LINE = (43, 29, 42)     # 변환기가 두르는 외곽선 색(sprite_ai.OUTLINE)


def ink_lines(spr, img, box, strength=0.72, cover=0.26):
    """원본에 그려진 **안쪽 선**(머리와 몸 사이·앞발 둘레·목도리 주름)을 줄인 그림에 그늘 줄로 되살린다.

    원본의 선은 한 블록(≈6px) 굵기라 키 24로 줄이면(원본 15px = 한 점) 다수결에서 늘 져서 사라지고, 몸이 한 덩어리가 된다.
    줄인 점마다 원본에서 그 점이 덮는 네모 안의 "어두운 선"(실루엣 가장자리에서 9px 안쪽) 비율이 `cover`를 넘으면 그 점을
    제 색의 어두운 음영(`strength`배)으로 — 검은 선이 아니라 그 자리 색의 그늘이라 눈에 거슬리지 않는다."""
    x0, y0, x1, y1 = box
    a = img[y0:y1, x0:x1]
    fg = ~((a[..., 0] > 170) & (a[..., 1] < 110) & (a[..., 2] > 170))
    ys, xs = np.nonzero(fg)
    fx0, fx1, fy0, fy1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    inner = np.asarray(Image.fromarray((fg * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(19))) > 0
    dark = (a.sum(-1) < 210) & inner
    out = spr.copy()
    oys, oxs = np.nonzero(out[..., 3] > 0)
    ox0, ox1, oy0, oy1 = oxs.min(), oxs.max() + 1, oys.min(), oys.max() + 1
    sx, sy = (fx1 - fx0) / (ox1 - ox0), (fy1 - fy0) / (oy1 - oy0)
    for y in range(oy0, oy1):
        for x in range(ox0, ox1):
            if out[y, x, 3] == 0 or tuple(out[y, x, :3]) == LINE:
                continue
            cell = dark[int(fy0 + (y - oy0) * sy):int(fy0 + (y - oy0 + 1) * sy), int(fx0 + (x - ox0) * sx):int(fx0 + (x - ox0 + 1) * sx)]
            if cell.size and cell.mean() >= cover:
                out[y, x, :3] = (out[y, x, :3].astype(float) * strength).astype(np.uint8)
    return out


def tinted_outline(cell, lit=0.40, shade=0.27, keep=0.25):
    """한 색 외곽선을 **곁의 색을 따르는 외곽선**으로(도트의 sel-out): 외곽선 점마다 맞닿은 안쪽 점들의 평균색을 어둡게 한 색.
    분홍 앞발 둘레는 짙은 장미색, 목도리는 짙은 녹빛, 털은 짙은 갈색이 된다. 빛이 왼쪽 위에서 오니 위·왼쪽 가장자리는 조금 밝게
    (`lit`), 아래·오른쪽은 더 어둡게(`shade`). 전체가 한 그림으로 묶이게 원래 선 색을 `keep`만큼 섞는다. 안쪽 이웃이 없는 점(가는 끝)은 그대로."""
    out = cell.copy()
    H, W = cell.shape[:2]
    line = (cell[..., 3] > 0) & np.all(cell[..., :3] == LINE, axis=-1)
    body = (cell[..., 3] > 0) & ~line
    for y, x in zip(*np.nonzero(line)):
        cols, dy, dx = [], 0.0, 0.0
        for ny in (y - 1, y, y + 1):
            for nx in (x - 1, x, x + 1):
                if 0 <= ny < H and 0 <= nx < W and body[ny, nx]:
                    cols.append(cell[ny, nx, :3].astype(float))
                    dy += ny - y
                    dx += nx - x
        if not cols:
            continue
        k = lit if (dy + dx) > 0 else shade          # 안쪽이 아래·오른쪽에 있으면 이 점은 위·왼쪽 가장자리
        c = np.mean(cols, axis=0) * k
        out[y, x, :3] = (c * (1 - keep) + np.array(LINE) * keep).astype(np.uint8)
    return out


def shrink_head(k):
    """`Source.prep`: 칸마다 머리(목도리 윗선 위)를 목을 기준으로 `k`배. 사장 2026-11-27 "다른 캐릭터에 비해 너무 이등신 느낌".

    머리 띠(그림 맨 위 ~ 목도리의 주황이 처음 넓게 나오는 줄)를 통째로 줄여 목 가운데에 다시 붙인다 — 머리가 몸 위에 얹힌 그림이라
    줄어든 자리는 바탕이 된다. 누운 칸(쓰러짐)은 머리가 옆이라 그대로. 키는 변환기가 서기 칸 전체 높이로 맞추니 머리가 줄면 몸이 그만큼 커진다."""
    def prep(img, frames):
        out = img.copy()
        for name, (x0, y0, x1, y1) in frames.items():
            if k == 1.0 or name in NO_SHRINK:
                continue
            a = out[y0:y1, x0:x1]
            r, g, b = a[..., 0], a[..., 1], a[..., 2]
            fg = ~((r > 170) & (g < 110) & (b > 170))
            ys, xs = np.nonzero(fg)
            fy0, fx0, fx1 = ys.min(), xs.min(), xs.max() + 1
            if name in NECK:
                neck, cx = NECK[name]
            else:
                scarf = fg & (r > 140) & (g > 50) & (g < 150) & (b < 80) & (r - g > 50)
                rows = np.nonzero(scarf.sum(1) >= 0.12 * (fx1 - fx0))[0]
                rows = rows[rows > fy0 + (ys.max() - fy0) * 0.3]          # 머리 위쪽의 주황 점은 목도리가 아니다
                if len(rows) == 0:
                    continue
                neck = int(rows[0]) + 2                                    # 목도리 윗줄을 조금 덮게
                cx = int(np.nonzero(scarf[rows[0]])[0].mean())
            head = Image.fromarray(a[fy0:neck].astype(np.uint8), "RGB")
            nw, nh = max(1, round(head.width * k)), max(1, round(head.height * k))
            small = np.asarray(head.resize((nw, nh), Image.NEAREST)).astype(int)
            a[fy0:neck] = (255, 0, 255)
            left = int(round(cx - cx * k))
            dst = a[neck - nh:neck, left:left + nw]
            keep = ~((small[..., 0] > 170) & (small[..., 1] < 110) & (small[..., 2] > 170))
            dst[keep] = small[keep]
        return out, frames
    return prep


# 모델이 혼자 크게 그려 온 칸: 줄이기 전에 그 칸만 발 가운데를 기준으로 줄인다. 2타 들기는 다른 칸보다 키가 3px(12%) 컸다 —
# 콤보 가운데에서 한 칸만 몸이 부풀었다.
SCALE_V5 = {"windup1": 0.89}


def scale_frames(scales):
    """`Source.prep`: 이름이 `scales`에 있는 칸을 그 배율로(칸 상자 안에서 아래 가운데 기준, 빈 자리는 바탕)."""
    def prep(img, frames):
        out = img.copy()
        for name, (x0, y0, x1, y1) in frames.items():
            k = scales.get(name)
            if not k:
                continue
            a = out[y0:y1, x0:x1]
            fg = ~((a[..., 0] > 170) & (a[..., 1] < 110) & (a[..., 2] > 170))
            ys, xs = np.nonzero(fg)
            bx0, bx1, by0, by1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
            fig = Image.fromarray(a[by0:by1, bx0:bx1].astype(np.uint8), "RGB")
            nw, nh = max(1, round(fig.width * k)), max(1, round(fig.height * k))
            small = np.asarray(fig.resize((nw, nh), Image.NEAREST)).astype(int)
            a[:] = (255, 0, 255)
            left = (bx0 + bx1) // 2 - nw // 2
            a[by1 - nh:by1, left:left + nw] = small
        return out, frames
    return prep


def prep_v5(head):
    """v5 원본의 손질: 혼자 큰 칸 줄이기 → 머리 줄이기."""
    def prep(img, frames):
        img, frames = scale_frames(SCALE_V5)(img, frames)
        return shrink_head(head)(img, frames)
    return prep


def spec_v5(head=1.0):
    """머리 배율을 준 v5 설정."""
    return Spec(
        sources=[Source(s.path, s.frames, groups=s.groups, tidy=s.tidy, strip=s.strip, prep=prep_v5(head)) for s in SPEC_V5.sources],
        palette_ref=SPEC_V5.palette_ref, body_h=SPEC_V5.body_h, breathe=SPEC_V5.breathe)


HEAD_V5 = 0.85      # 머리 배율(사장 2026-11-27 "너무 이등신 느낌 … 포포 정도면") — 머리 너비가 포포(15px)에 가깝다(받은 그대로 19 → 16px)


def build_v5(walk="gif", lines=True, head=HEAD_V5):
    """v5 시트. `walk`: "step" = 걷기 첫 칸에서 발만 딛는 네 칸(0241, 지금까지의 방식), "sheet" = 받은 걷기 네 칸 그대로,
    "gif" = 걷기 영상에서 뽑은 네 칸. `head` = 머리 배율(`shrink_head`)."""
    spr = convert(spec_v5(head))
    idle_img = None
    for sheet, frames in ((SHEET_V5_A, _V5_A), (SHEET_V5_B, _V5_B), (SHEET_V5_WALK, _V5_W), (SHEET_V5_DIVE, _V5_D),
                          (SHEET_V5_FALL, _V5_F)):
        img, _ = prep_v5(head)(np.asarray(Image.open(sheet).convert("RGB")).astype(int), frames)
        if sheet == SHEET_V5_A:
            idle_img = img
        for name, box in frames.items():
            if lines:
                spr[name] = ink_lines(spr[name], img, box)
            spr[name] = stamp_face(spr[name], eye_shines(img, box))
            if name in CLOSED_EYES:
                spr[name] = stamp_shut(spr[name], *CLOSED_EYES[name])
    if lines:
        spr["idle_b"] = ink_lines(spr["idle_b"], idle_img, _V5_A["idle_a"])
    spr["idle_b"] = stamp_face(spr["idle_b"], [(x, y + 1 / BODY_H_V5) for x, y in eye_shines(idle_img, _V5_A["idle_a"])])
    edge = tinted_outline if lines else (lambda c: c)
    sat = lambda k: edge(fill_holes(seat_on_fur(spr[k], spr["idle_a"], CELL, FOOT)))
    if walk == "step":
        walk4 = step_cycle(sat("walk_1"), FOOT - 1)
    else:
        walk4 = [sat(f"{'walkg' if walk == 'gif' else 'walk'}_{i}") for i in (1, 2, 3, 4)]
    rows = [
        [edge(fill_holes(seat(spr["idle_a"], CELL, FOOT))), edge(fill_holes(seat(spr["idle_b"], CELL, FOOT)))] + walk4
        + [sat("dive_a"), sat("dive_b"), sat("burst")],
        [sat(k) for k in ("attack0", "attack1", "attack2", "hurtg", "deadg", "windup0", "windup1", "windup2", "settle")],
        [small(legacy.MOUND_A), small(legacy.MOUND_B), small(legacy.SHADOW)],
    ]
    return rows


def sheet_of(rows):
    """줄 0~2(그림)와 줄 3~5(흰 실루엣)를 한 장으로."""
    sheet = np.zeros((CELL * 6, CELL * 9, 4), np.uint8)
    for r, row in enumerate(rows):
        for c, cell in enumerate(row):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = cell
            sil = cell.copy()
            on = sil[..., 3] > 0
            if r == 2:
                on &= ~np.all(sil[..., :3] == (24, 16, 28), axis=-1)   # 그림자는 실루엣 없음
            sil[on] = [255, 255, 255, 255]
            sil[~on] = 0
            sheet[(r + 3) * CELL:(r + 4) * CELL, c * CELL:(c + 1) * CELL] = sil
    return sheet


def build():
    """지금 게임 시트: v5(걷기는 영상 네 칸, 머리 `HEAD_V5`)."""
    return sheet_of(build_v5())


def build_v4():
    """옛 v4 시트(견줄 때만)."""
    spr = convert(SPEC)
    sat = lambda k: touch(seat_on_fur(spr[k], spr["idle_a"], CELL, FOOT), k)
    idle = touch(seat(spr["idle_a"], CELL, FOOT), "idle_a")
    rows = [
        [idle, touch(seat(spr["idle_b"], CELL, FOOT), "idle_b")]
        # v4도 걷기 넷을 그려 왔지만 넷째 칸만 몸이 좁고 키가 커 걸음마다 튀었다 → 첫 칸에서 발만 딛게(0241, 다른 캐릭터와 같이).
        + step_cycle(sat("walk_1"), FOOT - 1),
        # 동작 칸은 몸통 털의 가운데를 서기 칸에 맞춘다(발톱을 뻗어도 몸이 옆으로 뛰지 않게).
        [sat(k) for k in ("attack0", "attack1", "attack2", "hurt", "dead", "windup0", "windup1", "windup2", "settle")],
        [small(legacy.MOUND_A), small(legacy.MOUND_B), small(legacy.SHADOW)],
    ]
    return sheet_of(rows)


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/nail.png"
    sheet = build()
    Image.fromarray(sheet, "RGBA").save(out)
