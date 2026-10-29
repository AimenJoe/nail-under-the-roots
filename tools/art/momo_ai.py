"""모모(굴토끼) 스프라이트 시트(32×32 칸, 발바닥 30, 0206 C3). 몸 크기는 Naïl과 같고 칸만 넓다 — 차기·쓰러짐은
몸을 옆으로 길게 뻗어 외곽선까지 28px라 24칸에서는 얼굴·뒷발이 잘렸다(사장).

실행: python3 tools/art/momo_ai.py [출력 경로]   (기본 assets/art/momo.png)

원본:
  - 시트 A(서기·깡충): `tools/art/sources/pxl/momo_a.pxl` — 나노바나나가 게임 크기로 다시 그린 꽃 달린 모모를
    `pxl.py snap --find`로 옮기고 손본 것(눈·마젠타 찌꺼기·외곽선 색).
  - 시트 B(차기·맞음·쓰러짐): `sources/ai/sprites/momo_b_v1.png`(✦ 지움). 시트 A 그림을 붙여 받았는데 한 단 크게(블록 7.3px)
    그려 왔다 → 공용 변환기(`sprite_ai`)로 원본 외곽선을 걷고(`strip`) "맞음" 칸(서 있는 자세)을 시트 A 서기 키에 맞춰 줄인다.

칸 배치(critters 모모 줄과 같은 뜻, `art::Art::critter`가 모모 두 줄을 여기로 돌린다):
  칸 0·1 서기, 2·3 깡충(웅크림·뜀), 4 웅크림(차기 들기), 5 차기, 6 맞음, 7 쓰러짐
  줄 0 돌아온 모모(꽃) · 줄 1 물든 모모(보스: 보라 털·초록 눈·시든 꽃 — 줄 0의 색을 바꿔 만든다)
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import colorsys
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "pixel"))
from pxl import Pxl  # noqa: E402
import walk_ai  # noqa: E402
from sprite_ai import OUTLINE, Source, Spec, convert, lum, ramp, seat, sheet4  # noqa: E402

CELL, FOOT = 32, 30
A_CELL, A_FOOT = 24, 22     # 시트 A(.pxl)의 칸
SHEET_A = HERE / "sources" / "pxl" / "momo_a.pxl"
A_COLS = {"idle_a": 0, "idle_b": 1, "crouch": 2, "jump": 4}      # .pxl 안의 칸 번호(3 = 귀 젖힘은 안 쓴다)
SPEC_B = Spec(
    sources=[
        Source(
            HERE / "sources" / "ai" / "sprites" / "momo_b_v1.png",
            {"windup": (7, 29, 232, 205), "kick": (261, 22, 512, 205), "hurt": (538, 7, 737, 205),
             "down": (763, 66, 1017, 205)},
            groups=dict.fromkeys(("windup", "kick", "hurt", "down"), "hurt"),
            tidy=0.15,
            strip=9,
        )
    ],
    palette_ref=(0, "hurt"),
    body_h=21,     # 시트 A 서기 칸의 외곽선 안 키
)
ORDER = ["idle_a", "idle_b", "crouch", "jump", "windup", "kick", "hurt", "down"] + walk_ai.NAMES   # 8~11 깡충 넷(0229)

# 물든 모모(0029): critters의 물든 털(보라 4단) + 밝은 털 한 단, 초록으로 빛나는 눈, 시든 꽃.
TAINT_FUR = [(58, 42, 66), (88, 64, 90), (118, 90, 116), (150, 124, 146), (196, 178, 196)]
TAINT_OUTLINE = (26, 16, 38)
EYE_GREEN = [(24, 84, 52), (60, 170, 96), (140, 240, 150)]
EYE_NAVY = (60, 37, 85)        # 돌아온 모모 눈(시트 A의 남빛)
WITHER = [(92, 62, 70), (130, 92, 96), (164, 124, 120)]




def taint(cell):
    """돌아온 모모 칸 → 물든 모모 칸."""
    out = cell.copy()
    cache = {}
    for y, x in zip(*np.nonzero(cell[..., 3])):
        key = tuple(int(v) for v in cell[y, x, :3])
        if key not in cache:
            r, g, b = (v / 255 for v in key)
            h, l, s = colorsys.rgb_to_hls(r, g, b)
            L = lum(key) / 255
            if key == OUTLINE or L < 0.12:
                c = TAINT_OUTLINE if key == OUTLINE else key
            elif key[2] > key[0] + 18 and key[2] > key[1] + 18:          # 눈(남빛)
                c = ramp(EYE_GREEN, L * 2)
            elif L > 0.95:                                                # 눈 빛점
                c = key
            elif s > 0.45 and (h > 0.85 or h < 0.06 or (0.09 < h < 0.17 and L < 0.8)):  # 분홍 꽃·귓속·코·노란 꽃술(크림 가슴 털은 더 밝다)
                c = ramp(WITHER, L)
            else:                                                         # 털·가슴 털
                c = ramp(TAINT_FUR, L)
            cache[key] = c
        out[y, x, :3] = cache[key]
    return out


def navy_eyes(cell):
    """깡충 칸(0229)의 눈: 원본 눈이 외곽선처럼 새까매서 줄이면 거의 검정이 된다 — 그러면 물든 줄의 초록 눈(`taint`는
    남빛을 초록으로)이 깡충 칸에서만 꺼진다. 머리 자리(겉 네모 키의 20~60%·앞쪽, 귀 끝은 빼고)의 외곽선 아닌 안쪽 아주 어두운 점을 남빛으로."""
    out = cell.copy()
    on = np.pad(cell[..., 3] > 0, 1)[1:, 1:]       # 오른쪽·아래 가장자리 너머를 비어 있는 것으로
    ys, xs = np.nonzero(cell[..., 3] > 0)
    top, h, left, w = ys.min(), ys.max() - ys.min() + 1, xs.min(), xs.max() - xs.min() + 1
    for y, x in zip(ys, xs):
        key = tuple(int(v) for v in cell[y, x, :3])
        inner = all(on[y + dy, x + dx] for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)))   # 귀 가장자리는 뺀다
        if key != OUTLINE and lum(key) < 40 and inner and top + h * 0.2 < y < top + h * 0.6 and x > left + w * 0.55:
            out[y, x, :3] = EYE_NAVY
        elif key != OUTLINE and lum(key) < 40 and y < top + h * 0.25:
            # 웅크린 칸의 누운 귀: 원본이 뒷귀 외곽선을 앞귀 외곽선 바로 밑에 그려 줄이면 머리 위에 검은 띠가 남았다
            # → 둘레의 밝은 털색으로 메운다.
            near = [tuple(int(v) for v in cell[y + dy, x + dx, :3]) for dy, dx in ((1, 0), (0, -1), (0, 1), (-1, 0))
                    if 0 <= y + dy < CELL and 0 <= x + dx < CELL and cell[y + dy, x + dx, 3]]
            fur = [c for c in near if c != OUTLINE and lum(c) >= 40]
            if fur:
                out[y, x, :3] = max(fur, key=fur.count)
    return out


def build():
    a = np.asarray(Pxl.load(SHEET_A).image())
    cells = {}
    for k, c in A_COLS.items():
        cell = np.zeros((CELL, CELL, 4), np.uint8)
        x0, y0 = (CELL - A_CELL) // 2, FOOT - A_FOOT
        cell[y0:y0 + A_CELL, x0:x0 + A_CELL] = a[:, c * A_CELL:(c + 1) * A_CELL]
        cells[k] = cell
    b = convert(SPEC_B)
    for k in ("windup", "kick", "hurt", "down"):
        cells[k] = seat(b[k], CELL, FOOT)
    # 색표는 시트 B의 맞음 칸(시트 A는 .pxl이라 원본이 없다).
    for k, v in walk_ai.cells("momo", SPEC_B.sources[0], CELL, FOOT, "hurt").items():
        cells[k] = navy_eyes(v)
    return sheet4([(cells[k], taint(cells[k])) for k in ORDER])


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/momo.png"
    Image.fromarray(build(), "RGBA").save(out)
