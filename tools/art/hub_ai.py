"""마을·쉼터 새 그림(0224 A): 공용 물건 → `assets/art/hub.png`(48×48 칸, 발바닥 46, 한 줄).

실행: python3 tools/art/hub_ai.py [출력 경로]

원본: `sources/ai/sprites/hub_a1_v1.png`(좌판·화덕·입구)·`hub_a2_v1.png`(그루터기·바위·통나무), ✦ 지움. 그림체 판
(`ref_hub_style.png`)만 붙여 받은 둘째 시도(첫 시도는 도형 판을 베껴 버렸다, 0224 덧붙임). 물건마다 원본 크기가 달라 하나씩
따로 줄인다(`H` = 외곽선 안 키 — 목표 폭에서 셈). 칸 순서는 `art::HUB_*`: 0 좌판, 1 화덕(불 없이), 2 그루터기, 3 바위, 4 통나무 자리.
뿌리 길 입구(A1 셋째)는 마젠타가 아닌 흙 바탕으로 와서(구멍 테두리도 흙색이라 못 가른다) 아직 쓰지 않는다 — 다시 받는다.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import Source, Spec, convert, seat  # noqa: E402

CELL, FOOT = 48, 46
SRC = HERE / "sources" / "ai" / "sprites"
# 이름, 원본, 상자, 외곽선 안 키(목표 폭: 좌판 40·화덕 28·그루터기 20·바위 22·통나무 22)
ITEMS = [
    ("stall", "hub_a1_v1.png", (57, 3, 323, 173), 24),
    ("firepit", "hub_a1_v1.png", (405, 15, 670, 173), 16),
    ("stump", "hub_a2_v1.png", (47, 9, 324, 177), 12),
    ("boulder", "hub_a2_v1.png", (404, 14, 623, 172), 15),
    ("log", "hub_a2_v1.png", (715, 33, 1000, 172), 10),
]


def build():
    sheet = np.zeros((CELL, CELL * len(ITEMS), 4), np.uint8)
    for c, (name, src, box, h) in enumerate(ITEMS):
        spr = convert(Spec(sources=[Source(SRC / src, {name: box}, groups={name: name}, tidy=0.1, strip=6)],
                           palette_ref=(0, name), body_h=h))
        sheet[:, c * CELL:(c + 1) * CELL] = seat(spr[name], CELL, FOOT)
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/hub.png"
    Image.fromarray(build(), "RGBA").save(out)
