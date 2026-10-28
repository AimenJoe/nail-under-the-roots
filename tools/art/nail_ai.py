"""Naïl 스프라이트 시트(24×24 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)

옮기는 법(색표·다수결 줄이기·효과 지우기·손발 색·흔들기·숨쉬기)은 공용 변환기 `sprite_ai.py`에 있고, 여기는 Naïl의
설정(`SPEC`)과 시트 배치뿐이다. 왜 이 길인가(0203 → 0206): 16칸 손 그림은 "두더지 같지 않다"(사장) → 실루엣 시안 1 몸 +
둥근 시트의 눈·볼(`nail_s1_v2`), 동작은 시트 B(`nail_s1_v2_b`). 몸 16px(타일·들쥐와 같은 크기, 28은 4배 넓이라 컸다).

칸 배치(게임 코드는 칸 크기 `art::NAIL_CELL`만 안다):
  줄 0: 서기 2, 걷기 4 / 줄 1: 치기 3, 맞음, 쓰러짐, 들기 3, 거두기 / 줄 2: 흙더미 2, 그림자(16칸 그림을 칸 아래 가운데에)
  줄 3~5: 줄 0~2의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import os
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import nail as legacy  # noqa: E402  흙더미·그림자 16칸 그림과 털빛 표(FURS)를 같이 쓴다
from sprite_ai import Limbs, Source, Spec, convert, is_fur, seat, seat_on_fur  # noqa: E402

SPRITES = HERE / "sources" / "ai" / "sprites"
CELL = 24          # 몸 16px + 뻗은 발톱이 들어가는 칸(0206: 32칸·몸 28은 월드에 비해 컸다)
FOOT = 22          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(16칸 시절 그대로).

SPEC = Spec(
    sources=[
        # 시트 A: 시안 1 몸 + 둥근 시트의 눈·볼(✦는 지움). 원본은 서기 칸이 걷기 칸보다 크게 그려져 배율은 무리마다.
        # 발밑 보라 줄(윗줄 y 386~391, 아랫줄 661~666)은 칸 밖으로 뺀다. 둘째 서기 칸은 쓰지 않는다(`breathe`).
        Source(
            SPRITES / "nail_s1_v2.png",
            {
                "idle_a": (39, 7, 463, 385),
                "walk_a": (6, 419, 247, 660),
                "walk_b": (265, 419, 499, 660),
                "walk_c": (523, 419, 754, 660),
                "walk_d": (777, 419, 1013, 660),
            },
            groups={"idle_a": "idle_a", "walk_a": "walk_a", "walk_b": "walk_a", "walk_c": "walk_a", "walk_d": "walk_a"},
        ),
        # 시트 B(동작, 3×3): 눈 크기가 아홉 칸 모두 같아(같은 배율로 그렸다) 배율 하나 — 똑바로 선 들기 A의 몸 키로.
        # 그려 온 휘두름 호는 그 네모 안의 살구 갈색만 지운다.
        Source(
            SPRITES / "nail_s1_v2_b.png",
            {
                "windup0": (9, 5, 331, 295),
                "attack0": (350, 5, 669, 295),
                "windup1": (692, 5, 1022, 295),
                "attack1": (9, 317, 335, 604),
                "windup2": (378, 314, 658, 607),
                "attack2": (722, 320, 1022, 608),
                "settle": (9, 626, 335, 913),
                "hurt": (353, 621, 668, 913),
                "dead": (698, 678, 1022, 913),
            },
            groups=dict.fromkeys(
                ("windup0", "attack0", "windup1", "attack1", "windup2", "attack2", "settle", "hurt", "dead"), "windup0"
            ),
            erase={"attack0": [(455, 170, 669, 295)], "attack1": [(255, 317, 335, 604)]},
            tidy=True,
        ),
    ],
    # 색표는 기준 칸 하나에서(다섯 칸을 합쳐 고르면 넓은 갈색에 밀려 목도리 주황이 빠졌다).
    palette_ref=(0, "idle_a"),
    body_h=16,
    # 손발: 원본은 검게 왔지만 초상화는 분홍 손 + 크림 발톱이다(사장 "검은 다리가 어색"). 손은 코(231, 137, 153)보다
    # 한 단 짙게(코 바로 아래 앞발과 섞이지 않게), 발은 더 짙게. 원본 검은 손발은 색표에서 (32, 24, 35)를 받는다
    # (눈 (20, 9, 25)과는 RGB 차 합 37).
    limbs=Limbs(
        limb=(32, 24, 35), paw=(198, 102, 120), sole=(150, 80, 96), arm=(108, 77, 59), shade=(75, 44, 50),
        claw=(232, 217, 174),
    ),
    # 걷기 앞발: 앞으로 갈 때 1px 들어 올리고(위아래로 흔들어야 걷는 것 같다, 사장) 뒤로 1px — 오가는 폭 2px.
    # 옆으로 2px를 그대로 밀면 앞발이 몸에서 떨어져 공중에 뜬 막대가 됐다(로딩 화면 4배에서 보였다).
    swing={"walk_a": (1, -1), "walk_c": (-1, 0)},
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


def build():
    spr = convert(SPEC)
    rows = [
        [seat(spr[k], CELL, FOOT) for k in ("idle_a", "idle_b", "walk_a", "walk_b", "walk_c", "walk_d")],
        # 동작 칸은 몸통 털의 가운데를 서기 칸에 맞춘다(발톱을 뻗어도 몸이 옆으로 뛰지 않게).
        [seat_on_fur(spr[k], spr["idle_a"], CELL, FOOT)
         for k in ("attack0", "attack1", "attack2", "hurt", "dead", "windup0", "windup1", "windup2", "settle")],
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
