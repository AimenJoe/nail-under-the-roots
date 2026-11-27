"""마을·쉼터 새 그림(0224 A): 공용 물건 → `assets/art/hub.png`(48×48 칸, 발바닥 46, 한 줄).

실행: python3 tools/art/hub_ai.py [출력 경로]

원본: `sources/ai/sprites/hub_a1_v1.png`(좌판·화덕·입구)·`hub_a2_v1.png`(그루터기·바위·통나무), ✦ 지움. 그림체 판
(`ref_hub_style.png`)만 붙여 받은 둘째 시도(첫 시도는 도형 판을 베껴 버렸다, 0224 덧붙임). 물건마다 원본 크기가 달라 하나씩
따로 줄인다(`H` = 외곽선 안 키 — 목표 폭에서 셈). 칸 순서는 `art::HUB_*`: 0 좌판, 1 화덕(불 없이), 2 그루터기, 3 바위, 4 통나무 자리,
5 뿌리 길 입구(A1 셋째는 흙 바탕으로 와서 버리고 A3으로 다시 받았다 — 흰 바탕·분홍 테를 마젠타로 바꿔 저장).
6~8은 시트 E1(`hub_e1_v1.png`): 이웃 등불·샘 그릇·갈대 피리, 9~11은 시트 E2(`hub_e2_v1.png`): 큰 깍정이·풀 방석·버섯 등,
12~14는 시트 E3(`hub_e3_v1.png`): 솜털 잠자리·모모의 꽃밭·바람종, 15~17은 시트 E5(`hub_e5_v1.png`): 텃밭 빈 칸·갓 심은 칸·산딸기 덤불,
18~20은 시트 E6(`hub_e6_v1.png`): 물웅덩이·다시 솟는 샘·길의 샘(세로로 눌러 넣는다)
(`docs/art/hub-sprite-prompts.md` 시트 E).
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
    # 뿌리 길 입구: 다시 받은 `hub_a3_v1.png`(흰 바탕 → 마젠타로 바꿔 둠, 시안 다섯 가운데 팻말 + 첫 구멍). 폭 40.
    ("gate", "hub_a3_v1.png", (4, 14, 294, 176), 21),
    # 시트 E1(사장 2026-11-27 — 코드로 찍던 그림을 바꾼다): 이웃 등불(키 22, 유리 속 빛은 코드가 얹는다)·샘 그릇(폭 16)·
    # 통나무에 기댄 갈대 피리(폭 24). 칸 6·7·8 = `art::HUB_LAMP`·`HUB_BOWL`·`HUB_PIPE`.
    ("lamp", "hub_e1_v1.png", (156, 0, 268, 147), 22),
    ("bowl", "hub_e1_v1.png", (436, 9, 625, 144), 11),
    ("pipe", "hub_e1_v1.png", (763, 4, 972, 147), 16),
    # 시트 E2: 큰 깍정이(폭 14)·풀 방석(폭 17)·버섯 등(키 13, 빛은 코드가 얹는다). 칸 9·10·11 = `HUB_CAP`·`HUB_CUSHION`·`HUB_SHROOM`.
    ("cap", "hub_e2_v1.png", (24, 175, 309, 410), 11),
    ("cushion", "hub_e2_v1.png", (322, 149, 736, 410), 10),
    ("shroom", "hub_e2_v1.png", (774, 167, 992, 401), 13),
    # 시트 E3: 솜털 잠자리(폭 18)·모모의 꽃밭(폭 20)·바람종(키 30 — 가는 기둥이 살아남는 크기). 칸 12·13·14 = `HUB_NEST`·`HUB_FLOWERS`·`HUB_CHIME`.
    ("nest", "hub_e3_v1.png", (41, 201, 343, 401), 11),
    ("flowers", "hub_e3_v1.png", (382, 201, 778, 401), 10),
    ("chime", "hub_e3_v1.png", (834, 73, 983, 469), 30),
    # 시트 E5(텃밭): 빈 흙 칸(폭 16)·갓 심은 칸·다 익은 산딸기 덤불(폭 16). 칸 15·16·17 = `HUB_PLOT`·`HUB_PLOT_SOWN`·`HUB_BERRY_BUSH`.
    ("plot", "hub_e5_v1.png", (25, 33, 354, 244), 10),
    ("plot_sown", "hub_e5_v1.png", (375, 12, 702, 244), 11),
    ("berry_bush", "hub_e5_v1.png", (736, 28, 988, 244), 14),
    # 시트 E6(물): 물웅덩이(폭 22)·다시 솟는 샘(폭 25)·길의 샘(폭 44). 칸 18·19·20 = `HUB_PUDDLE`·`HUB_SPRING`·`HUB_POOL`.
    # 받은 그림은 너무 위에서 본 각도라(사장 2026-11-27) 줄이기 전에 세로로 누른다(`SQUASH`).
    ("puddle", "hub_e6_v1.png", (88, 246, 366, 426), 10),
    ("spring", "hub_e6_v1.png", (435, 177, 803, 485), 13),
    ("pool", "hub_e6_v1.png", (871, 107, 1506, 565), 24),
]


def soil_grain(cell, sown):
    """텃밭 흙 칸의 흙 면에 이랑과 알갱이를 찍는다(사장 2026-11-27 "중간에 노이즈가 좀 있어야").

    원본에는 갈라진 금과 씨앗 둔덕이 있지만 폭 16으로 줄이면 흙 면이 한 색으로 뭉개진다. 흙 면(가장 많은 색)을 찾아
    그 안에만 찍는다: 어두운 이랑 두 줄(군데군데 끊김), 밝고 어두운 알갱이, 갓 심은 칸은 밝은 씨앗 둔덕 셋. 자리는 칸 좌표의
    해시라 다시 만들어도 같다.
    """
    op = cell[..., 3] > 0
    cols, counts = np.unique(cell[op][:, :3], axis=0, return_counts=True)
    soil = cols[counts.argmax()]
    mask = op & (cell[..., :3] == soil).all(-1)
    ys, xs = np.where(mask)
    y0, y1, x0, x1 = ys.min(), ys.max(), xs.min(), xs.max()
    dark = (soil * 0.80).astype(np.uint8)
    deep = (soil * 0.66).astype(np.uint8)
    light = np.minimum(soil.astype(int) + 24, 255).astype(np.uint8)
    put = lambda y, x, c: mask[y, x] and cell.__setitem__((y, x, slice(0, 3)), c)
    h = y1 - y0
    for fy in (y0 + round(h * 0.30), y0 + round(h * 0.68)):          # 이랑 두 줄
        for x in range(x0, x1 + 1):
            if (x * 7 + fy * 3) % 5:
                put(fy, x, dark)
    for y in range(y0, y1 + 1):                                      # 알갱이
        for x in range(x0, x1 + 1):
            k = (x * 31 + y * 17 + x * y * 7) % 23
            if k == 0:
                put(y, x, light)
            elif k == 1:
                put(y, x, deep)
    if sown:                                                         # 씨앗 둔덕 셋: 밝은 꼭대기 + 어두운 밑
        w = x1 - x0
        for mx, my in ((x0 + round(w * 0.22), y0 + round(h * 0.50)), (x0 + round(w * 0.55), y0 + round(h * 0.22)),
                       (x0 + round(w * 0.78), y0 + round(h * 0.62))):
            put(my, mx, light)
            for dx in (-1, 0, 1):
                put(my + 1, mx + dx, light if dx == 0 else dark)
            for dx in (-1, 0, 1):
                put(my + 2, mx + dx, deep)


# 바닥에 누운 물: 받은 그림의 세로를 이 비율로 눌러 게임의 비스듬한 시점(폭이 높이의 두 배쯤)에 맞춘다.
SQUASH = {"puddle": 0.72, "spring": 0.64, "pool": 0.76}


def squash(k):
    """`Source.prep`: 칸 하나짜리 원본의 그 칸을 세로로 `k`배(가장 가까운 점). 외곽선도 같이 눌려 가늘어지지만 벗기고 다시 두른다."""
    def prep(img, frames):
        (name, (x0, y0, x1, y1)), = frames.items()
        crop = Image.fromarray(img[y0:y1, x0:x1].astype(np.uint8), "RGB")
        crop = crop.resize((crop.width, max(1, round(crop.height * k))), Image.NEAREST)
        return np.asarray(crop).astype(int), {name: (0, 0, crop.width, crop.height)}
    return prep


# 가는 물건: 외곽선을 6px 벗기면 기둥(속 6px)이 통째로 사라지고, 떨어진 조각 버리기가 매달린 관을 버린다.
THIN = {"chime": {"tidy": 0.0, "strip": 4}}


def build():
    sheet = np.zeros((CELL, CELL * len(ITEMS), 4), np.uint8)
    for c, (name, src, box, h) in enumerate(ITEMS):
        opt = {"tidy": 0.1, "strip": 6, **THIN.get(name, {})}
        if name in SQUASH:
            opt["prep"] = squash(SQUASH[name])
        spr = convert(Spec(sources=[Source(SRC / src, {name: box}, groups={name: name}, **opt)],
                           palette_ref=(0, name), body_h=h))
        cell = seat(spr[name], CELL, FOOT)
        if name in ("plot", "plot_sown"):
            soil_grain(cell, sown=name == "plot_sown")
        sheet[:, c * CELL:(c + 1) * CELL] = cell
    return sheet


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/hub.png"
    Image.fromarray(build(), "RGBA").save(out)
