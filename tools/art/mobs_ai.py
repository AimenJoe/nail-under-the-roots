"""잡몹 새 그림(0221): 24×24 칸, 발바닥 22. 원본 `sources/ai/sprites/<이름>_v1.png`(✦ 지움) → `assets/art/<이름>.png`.

실행: python3 tools/art/mobs_ai.py [이름 ...]   (기본: 원본이 있는 잡몹 모두)

참고 그림·프롬프트는 `mobs_ref.py`(0221). 받은 시트는 3×2 여섯 칸(서기·걷기·예고·공격·맞음·쓰러짐)이 기본이지만 잡몹마다
다르게 와서(뒷모습 칸 등) `frames`(원본 칸 상자)와 `order`(게임 칸 여덟)로 고른다. "~이름"은 그 칸을 1px 내쉰 칸.
게임 칸(critters 그 잡몹 줄과 같은 뜻 + 큰 시트의 7): 0 서기, 1 날숨, 2·3 걷기, 4 예고, 5 공격, 6 맞음, 7 쓰러짐.
시트: 줄 0 잡몹, 줄 1은 줄 0과 같다(잡몹은 물든 모습 하나 — 큰 시트 줄 순서를 맞추려고), 줄 2·3 흰 실루엣(피격 번쩍).

물든 표지(붉은 눈·보라 뿌리)는 원본에서 드문 색이라 색표(NCOL)에 못 들거나 다수결에 진다 → 원본에서 옮겨 칠한다
(`neighbors_ai.transfer`, 포포 연필과 같은 길).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from neighbors_ai import redraw_eyes, transfer  # noqa: E402
from sprite_ai import Source, Spec, breathe, convert, seat  # noqa: E402

CELL, FOOT = 24, 22


def lum(r):
    return (r * [0.299, 0.587, 0.114]).sum(-1)


# 붉은 눈(240, 50, 100): 귓속·꼬리의 분홍(185, 100, 115)은 빼려고 G < 80, R > 210.
RED_EYE = (lambda r: (r[..., 0] > 210) & (r[..., 1] < 80) & (r[..., 2] < 140), (232, 44, 64), 2)
PURPLE = (lambda r: (r[..., 2] > 120) & (r[..., 2] - r[..., 1] > 50) & (r[..., 0] < 190) & (r[..., 0] > 80),
          (128, 70, 178), 3)
TAINT = [PURPLE, RED_EYE]

MOBS = {
    # 들쥐: 서기·걷기 두 칸이 뒷모습으로 왔다 — 옆모습 넷(웅크림·달리기·맞음·쓰러짐)만 쓴다. 서기는 웅크림(조금 낮은 서기로
    # 읽힌다), 걷기는 웅크림 ↔ 달리기, 예고 = 웅크림(게임이 떨고 붉게 한다).
    "field_mouse": dict(
        frames={"crouch": (548, 67, 760, 213), "dash": (29, 292, 306, 455), "hurt": (377, 271, 611, 462),
                "down": (712, 356, 995, 455)},
        order=["crouch", "~crouch", "dash", "crouch", "crouch", "dash", "hurt", "down"],
        body_h=12,
        strip=8,
        # 눈을 검게 그려 왔다(물든 잡몹은 붉은 눈) — 원본의 검은 눈 자리에 붉은 눈을 다시 찍는다(뜬 눈 칸만).
        red_eyes=("crouch", "dash"),
    ),
    "house_mouse": dict(
        frames={"stand": (62, 50, 300, 226), "walk": (370, 35, 642, 226), "crouch": (709, 57, 982, 226),
                "dash": (11, 286, 315, 469), "hurt": (394, 286, 638, 469), "down": (700, 343, 994, 469)},
        order=["stand", "~stand", "walk", "stand", "crouch", "dash", "hurt", "down"],
        body_h=10,
        strip=12,
    ),
}


def spec(name):
    m = MOBS[name]
    ref = m["order"][0].lstrip("~")
    return Spec(
        sources=[Source(HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png", m["frames"],
                        groups=dict.fromkeys(m["frames"], ref), tidy=0.15, strip=m["strip"])],
        palette_ref=(0, ref),
        body_h=m["body_h"],
    )


def build(name):
    m = MOBS[name]
    spr = convert(spec(name))
    src = np.asarray(Image.open(HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png").convert("RGB")).astype(int)
    sheet = np.zeros((CELL * 4, CELL * len(m["order"]), 4), np.uint8)
    for c, src_name in enumerate(m["order"]):
        base = src_name.lstrip("~")
        cell = seat(breathe(spr[base]) if src_name.startswith("~") else spr[base], CELL, FOOT)
        for pred, ink, hits in m.get("marks", TAINT):
            cell = transfer(cell, src, m["frames"][base], pred, ink, hits)
        if base in m.get("red_eyes", ()):
            cell = redraw_eyes(cell, src, m["frames"][base], False, ink=RED_EYE[1], band=(0.15, 0.7))
        sil = np.zeros_like(cell)
        sil[cell[..., 3] > 0] = [255, 255, 255, 255]
        for r, im in enumerate((cell, cell, sil, sil)):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = im
    return sheet


if __name__ == "__main__":
    for name in sys.argv[1:] or [n for n in MOBS if (HERE / "sources" / "ai" / "sprites" / f"{n}_v1.png").exists()]:
        Image.fromarray(build(name), "RGBA").save(f"assets/art/{name}.png")
        print(name)
