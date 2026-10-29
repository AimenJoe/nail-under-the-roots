"""Naïl 스프라이트 시트(28×28 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206 → 0238).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)

옮기는 법(색표·다수결 줄이기·효과 지우기·손발 색·흔들기·숨쉬기)은 공용 변환기 `sprite_ai.py`에 있고, 여기는 Naïl의
설정(`SPEC`)과 시트 배치뿐이다. 왜 이 길인가(0203 → 0206 → 0238): 16칸 손 그림은 "두더지 같지 않다"(사장) → 실루엣 시안 1
(`nail_s1_v2`, 몸 16px) → 사장 "처음부터 다시"(0238): 큰 그림 다섯 안에서 고른 1번(서서 삽 손을 벌린 몸 + 주황 목도리)을
시트 둘(`nail_v3_a`·`nail_v3_b`)로 받아 키 약 22px로(모모 23·도토 25와 나란히). 0242에 머리를 키워(`chibi`) 키 약 24px. 원본 손발이 이미 분홍·크림이라 `Limbs`·`swing`은 없다.

칸 배치(게임 코드는 칸 크기 `art::NAIL_CELL`만 안다):
  줄 0: 서기 2, 걷기 4 / 줄 1: 치기 3, 맞음, 쓰러짐, 들기 3, 거두기 / 줄 2: 흙더미 2, 그림자(16칸 그림을 칸 아래 가운데에)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import os
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import nail as legacy  # noqa: E402  흙더미·그림자 16칸 그림과 털빛 표(FURS)를 같이 쓴다
from sprite_ai import Source, Spec, convert, is_fur, magenta, seat, seat_on_fur, step_cycle  # noqa: E402

SPRITES = HERE / "sources" / "ai" / "sprites"
CELL = 28          # 키 약 24px(머리 키운 뒤, 0242) + 뻗은 발톱(0238: 18px는 모모 23·도토 25보다 작아 주인공이 가장 작았다)
FOOT = 26          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(`art::nail_origin`).

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


def chibi(img, frames):
    """칸마다 머리를 키운 그림을 새 판에 칸씩 세로로 쌓는다(넓힌 상자가 옆 칸을 덮지 않게). → (새 판, 새 칸 상자)."""
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
        cand = fg & ~orange & ~cream & (rows < yn)
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
        if shave.any():
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
SPEC = Spec(
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
INK = {
    "line": (43, 29, 42), "dark": (61, 27, 51), "fur": (84, 53, 58), "fur2": (121, 75, 77),
    "eye": (20, 12, 16), "scarf": (211, 105, 53), "scarf2": (169, 66, 49), "shade": (63, 29, 53),
    "snout": (236, 160, 137),
}
# 뒤통수의 붉은 점(목도리 매듭이 머리 외곽선 띠 칠하기에 섞인 색) → 그늘 털, 눈은 세로 2px.
TOUCH = {
    "idle_a": {(9, 7): "shade", (9, 8): "shade", (8, 9): "shade", (7, 10): "shade", (8, 10): "shade",
               (9, 11): "shade", (10, 10): "fur", (14, 7): "eye"},
    "idle_b": {(9, 8): "shade", (9, 9): "shade", (8, 10): "shade", (7, 11): "shade", (8, 11): "shade",
               (9, 12): "shade", (10, 11): "fur", (14, 8): "eye"},
    # 맞음: 얼굴을 가린 앞발 사이 검은 점이 몰려 있다 → 그늘로 풀어 준다.
    "hurt": {(9, 8): "shade", (9, 9): "shade", (8, 10): "shade", (9, 10): "shade", (8, 11): "shade",
             (9, 11): "shade", (14, 7): "fur", (17, 13): "shade", (16, 15): "shade", (18, 12): "shade",
             (12, 12): "shade"},
    "dead": {(9, 16): "shade", (16, 17): "shade", (17, 17): "fur", (18, 17): "shade", (17, 18): "shade",
             (18, 18): "shade"},
    # 들기 2: 머리 위 앞발과 머리 사이 검은 쐐기.
    "windup2": {(19, 4): "fur", (20, 4): "line", (14, 7): "eye"},
    "settle": {(9, 7): "shade", (9, 8): "shade", (7, 9): "shade", (8, 9): "shade", (8, 10): "shade",
               (8, 11): "shade", (9, 10): "shade", (9, 11): "shade"},
    "windup1": {(9, 8): "shade", (8, 10): "shade", (10, 11): "shade", (15, 8): "eye"},
    "attack0": {(8, 8): "shade", (8, 9): "shade", (8, 10): "shade", (7, 11): "shade", (8, 11): "shade",
                (9, 9): "shade", (9, 10): "shade", (11, 12): "fur"},
    "attack1": {(7, 6): "shade", (7, 7): "shade", (8, 7): "shade", (7, 8): "shade", (8, 8): "shade",
                (8, 9): "shade", (6, 10): "shade"},
    "attack2": {(9, 9): "shade", (10, 9): "shade", (9, 10): "shade", (10, 10): "shade", (11, 10): "shade",
                (11, 9): "fur"},
    # 걷기: 눈이 주둥이 윗선에 붙어 검은 덩어리 → 주둥이 뒤로 옮긴다.
    "walk_a": {(18, 6): "fur", (19, 6): "line", (19, 7): "snout", (16, 6): "eye", (16, 7): "eye",
               (8, 7): "shade", (8, 9): "shade", (8, 10): "shade", (8, 11): "shade", (9, 10): "fur"},
}


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
    for (x, y), ink in TOUCH.get(name, {}).items():
        out[y, x] = (0, 0, 0, 0) if ink is None else INK[ink] + (255,)
    return out


def build():
    spr = convert(SPEC)
    sat = lambda k: touch(seat_on_fur(spr[k], spr["idle_a"], CELL, FOOT), k)
    idle = touch(seat(spr["idle_a"], CELL, FOOT), "idle_a")
    rows = [
        [idle, touch(seat(spr["idle_b"], CELL, FOOT), "idle_b")]
        + step_cycle(sat("walk_a"), FOOT - 1),
        # 동작 칸은 몸통 털의 가운데를 서기 칸에 맞춘다(발톱을 뻗어도 몸이 옆으로 뛰지 않게).
        [sat(k) for k in ("attack0", "attack1", "attack2", "hurt", "dead", "windup0", "windup1", "windup2", "settle")],
        [small(legacy.MOUND_A), small(legacy.MOUND_B), small(legacy.SHADOW)],
    ]
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


def fur_variant(sheet, fur):
    """털빛(0048): 털 갈색만 그 털빛의 4단(d·b·h·l)에 밝기 순으로 옮긴다. 목도리·분홍·눈·흙은 그대로."""
    if fur == 0:
        return sheet
    target = [legacy.FURS[fur][k] for k in "dbhl"]
    base = [legacy.P[k] for k in "dbhl"]
    lum = lambda c: 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
    out = sheet.copy()
    flat = out[: CELL * 3].reshape(-1, 4)
    cache = {}
    for i, px in enumerate(flat):
        if px[3] == 0:
            continue
        key = tuple(int(v) for v in px[:3])
        if key not in cache:
            if not is_fur(key):
                cache[key] = None
            else:
                # 기본 털 4단 밝기 사이 어디쯤인가 → 같은 자리의 털빛 색
                L = lum(key)
                bl = [lum(c) for c in base]
                t = np.interp(L, bl, range(4))
                i0, f = int(min(t, 2.999)), t - int(min(t, 2.999))
                c0, c1 = target[i0], target[i0 + 1]
                cache[key] = tuple(int(c0[j] * (1 - f) + c1[j] * f) for j in range(3))
        if cache[key] is not None:
            flat[i, :3] = cache[key]
    return out


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/nail.png"
    sheet = build()
    Image.fromarray(sheet, "RGBA").save(out)
    for n in legacy.FURS:
        Image.fromarray(fur_variant(sheet, n), "RGBA").save(os.path.join(os.path.dirname(out) or ".", f"nail_fur{n}.png"))
