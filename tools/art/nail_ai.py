"""Naïl 스프라이트 시트(28×28 칸): 나노바나나 원본(tools/art/sources/ai/sprites/)을 게임 도트로 옮긴다(0206 → 0238).

실행: python3 tools/art/nail_ai.py [출력 경로]   (기본 assets/art/nail.png, 털빛 1~3은 같은 폴더의 nail_fur<n>.png)

옮기는 법(색표·다수결 줄이기·효과 지우기·손발 색·흔들기·숨쉬기)은 공용 변환기 `sprite_ai.py`에 있고, 여기는 Naïl의
설정(`SPEC`)과 시트 배치뿐이다. 왜 이 길인가(0203 → 0206 → 0238): 16칸 손 그림은 "두더지 같지 않다"(사장) → 실루엣 시안 1
(`nail_s1_v2`, 몸 16px) → 사장 "처음부터 다시"(0238): 큰 그림 다섯 안에서 고른 1번(서서 삽 손을 벌린 몸 + 주황 목도리)을
시트 둘(`nail_v3_a`·`nail_v3_b`)로 받아 키 약 22px로(모모 23·도토 25와 나란히). 원본 손발이 이미 분홍·크림이라 `Limbs`·`swing`은 없다.

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
from sprite_ai import Source, Spec, convert, is_fur, seat, seat_on_fur, step_cycle  # noqa: E402

SPRITES = HERE / "sources" / "ai" / "sprites"
CELL = 28          # 몸 키 약 22px + 뻗은 발톱(0238: 18px는 모모 23·도토 25보다 작아 주인공이 가장 작았다)
FOOT = 26          # 발바닥 줄(칸 위에서). 게임은 발을 몸 자리 + 6에 둔다(`art::nail_origin`).

# 원본(0238, 사장 고름 1단계 1번 → 2단계 시트 둘, ✦ 지움). 칸 상자는 redo_concepts.components로 찾은 값.
# 시트 A(3줄 × 4): 줄 1 서기·서기(숨)·맞음·쓰러짐, 줄 2·3 걷기(줄 2 첫 칸만 쓴다 — 네 칸이 거의 같고 앞발만 튀어서, 0241). 서기 두 칸은 벌린 발톱끼리 붙어 있어
# 서기 A만 잘라(오른쪽 발톱 끝까지) 옆 칸 발톱 조각은 `tidy`로 버린다. 서기 B는 `breathe`로 만든다(따로 그린 두 칸은 숨이 끊긴다).
# 시트 B(3줄 × 4): 줄 1 들기·치기·들기 2·(쓰지 않음), 줄 2 치기 2·강타 들기(머리 높이 — 더 높이 든 칸은 몸을 크게 그려 와 안 쓴다)·(쓰지 않음)·강타, 줄 3 거두기·(서기)·파고들기·솟구치기.
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
        ),
    ],
    palette_ref=(0, "idle_a"),
    body_h=21,
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
    idle = seat(spr["idle_a"], CELL, FOOT)
    rows = [
        [idle, seat(spr["idle_b"], CELL, FOOT)]
        + step_cycle(seat_on_fur(spr["walk_a"], spr["idle_a"], CELL, FOOT), FOOT - 1),
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
