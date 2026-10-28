"""소품 새 그림(C5, 0222): 부술 수 있는 것 넷 + 보물 상자(닫힘·열림) → `assets/art/props.png`(24×24 칸 여섯, 발바닥 22, 한 줄).

실행: python3 tools/art/props_ai.py [출력 경로]

원본: `sources/ai/sprites/props_v1.png`(✦ 지움). 시안 둘 가운데 한 줄 여섯 개짜리를 썼다 — 다른 하나는 금을 보랏빛 번개로
그려(마젠타 바탕과 함께 지워져 구멍이 난다) 세밀해 16px에서 뭉개진다. 이쪽은 0205의 연노랑 금 그대로다.
물건마다 원본 폭이 제멋대로라(흙더미가 참고 그림의 두 배) 하나씩 따로 줄인다(`H` = 외곽선 안 키). 열린 상자는 닫힌
상자와 같은 배율(뚜껑만큼 키가 크다). 칸 순서는 `art::PROP_*`: 0 흙더미, 1 뿌리혹, 2 돌무더기, 3 실뭉치, 4 상자, 5 열린 상자.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from neighbors_ai import transfer  # noqa: E402
from sprite_ai import Source, Spec, convert, seat  # noqa: E402

CELL, FOOT = 24, 22
SRC = HERE / "sources" / "ai" / "sprites" / "props_v1.png"
# 이름, 원본 상자, 외곽선 안 키(흙더미는 폭 18이 되게, 열린 상자는 닫힌 상자와 같은 배율)
PROPS = [
    ("mound", (7, 22, 217, 137), 9),
    ("gall", (238, 3, 373, 150), 13),
    ("rocks", (391, 8, 538, 145), 12),
    ("silk", (560, 22, 687, 148), 12),
    ("chest", (718, 37, 852, 148), 12),
    ("chest_open", (887, 3, 1020, 150), 16),
]


# 금(0205 "깨진다" 표시, 연노랑 (230, 220, 175)): 가는 갈지자라 줄이면 흙·돌 색에 묻힌다 → 원본에서 옮겨 칠한다
# (`neighbors_ai.transfer`, 포포 연필과 같은 길). 실뭉치의 흰 털·상자 금속은 노랑이 아니라 안 걸린다.
CRACK = (lambda r: (r[..., 0] > 200) & (r[..., 1] > 185) & (r[..., 2] > 130) & (r[..., 2] < 200)
         & (r[..., 0] - r[..., 2] > 35), (250, 238, 190), 4)
BREAKABLE = 4   # 앞의 넷만 금이 있다


def build():
    sheet = np.zeros((CELL, CELL * len(PROPS), 4), np.uint8)
    for c, (name, box, h) in enumerate(PROPS):
        spr = convert(Spec(sources=[Source(SRC, {name: box}, groups={name: name}, tidy=0.1, strip=8)],
                           palette_ref=(0, name), body_h=h))
        cell = seat(spr[name], CELL, FOOT)
        if c < BREAKABLE:
            pred, ink, hits = CRACK
            cell = transfer(cell, np.asarray(Image.open(SRC).convert("RGB")).astype(int), box, pred, ink, hits)
        sheet[:, c * CELL:(c + 1) * CELL] = cell
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/props.png"
    Image.fromarray(build(), "RGBA").save(out)
