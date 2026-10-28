"""이웃 새 그림(0206 C4, 0220): 포포(라쿤)·수수(땅강아지) 32×32 칸, 발바닥 30.

실행: python3 tools/art/neighbors_ai.py [이름 ...]   (기본: 원본이 있는 이웃 모두 → assets/art/<이름>.png)

원본: `sources/ai/sprites/<이름>_v1.png`(✦ 지움). 크기 기준 그림(`neighbors_silhouette.py`의 `ref_neighbors_scale`)을 붙여 받았다.
칸(critters 그 이웃 줄과 같은 뜻 — `art::BIG_SHEETS`가 critters 줄을 여기로 돌린다):
  칸 0·1 서기(들숨·날숨), 2·3 걷기, 4 말하기(포포)·파기(수수), 5 기쁨
  줄 0 이웃, 줄 1은 줄 0과 같다(물든 모습이 없다 — 큰 시트 줄 순서를 맞추려고), 줄 2·3 흰 실루엣.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import OUTLINE, Source, Spec, convert, seat  # noqa: E402

CELL, FOOT = 32, 30
NAMES = ["idle_a", "idle_b", "walk_a", "walk_b", "act", "happy"]
SHEETS = {
    # 포포: 4×2 여덟 칸으로 왔다(부탁은 3×2, 칸 사이 세로 경계선). 1줄 = 서기·숨·걷기(웃음)·걷기(같은 걸음, 기쁜 눈),
    # 2줄 = 걷기(다른 걸음)·턱 괴고 말하기·기쁨 둘. 걷기는 걸음이 다른 1줄 셋째·2줄 첫째.
    "popo": dict(
        boxes=[(33, 12, 196, 184), (292, 21, 459, 184), (547, 11, 717, 185),
               (36, 200, 204, 375), (292, 200, 466, 375), (549, 200, 716, 375)],
        body_h=24,      # 외곽선 안 키 → 외곽선까지 26
        strip=6,
        # 귀 뒤 연필: 노랑은 원본에서 드물어 색표(NCOL)에 못 들고, 가는 빗금이라 다수결에도 진다 → 원본에서 옮겨 칠한다.
        marks=[(lambda r: (r[..., 0] > 190) & (r[..., 1] > 140) & (r[..., 2] < 110), (236, 190, 64), 3)],
        # 가면 속 눈: 검은 눈이 짙은 회색 가면에 묻혀 빛점을 찍는다(기쁨 칸은 감은 눈이라 뺀다).
        sparkle=(56, 61, 64),
    ),
}


def spec(name):
    s = SHEETS[name]
    src = HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png"
    return Spec(
        sources=[Source(src, dict(zip(NAMES, s["boxes"])), groups=dict.fromkeys(NAMES, "idle_a"), tidy=0.15,
                        strip=s["strip"], pink=s.get("pink", False))],
        palette_ref=(0, "idle_a"),
        body_h=s["body_h"],
    )


def transfer(cell, src, box, pred, ink, hits):
    """원본 상자 `box`에서 `pred`에 맞는 점을 몸 겉 네모끼리 맞춰 칸으로 옮기고, 칸 점 하나에 `hits`개 넘게 모이면 `ink`로
    칠한다(외곽선 안쪽만). 작은 표지(연필·고글)가 색표·다수결에서 사라지지 않게."""
    x0, y0, x1, y1 = box
    r = src[y0:y1, x0:x1]
    body = (np.minimum(r[..., 0], r[..., 2]) - r[..., 1]) < 60
    rows = np.nonzero(body.sum(1) >= 6)[0]
    cols = np.nonzero(body.sum(0) >= 6)[0]
    on = cell[..., 3] > 0
    cys, cxs = np.nonzero(on)
    sy = (cys.max() - cys.min() + 1) / (rows.max() - rows.min() + 1)
    sx = (cxs.max() - cxs.min() + 1) / (cols.max() - cols.min() + 1)
    hit = np.zeros(on.shape, int)
    for y, x in zip(*np.nonzero(pred(r) & body)):
        ty, tx = int(cys.min() + (y - rows.min()) * sy), int(cxs.min() + (x - cols.min()) * sx)
        if 0 <= ty < hit.shape[0] and 0 <= tx < hit.shape[1]:
            hit[ty, tx] += 1
    out = cell.copy()
    for y, x in zip(*np.nonzero(hit >= hits)):
        if on[y, x] and tuple(int(v) for v in cell[y, x, :3]) != OUTLINE:
            out[y, x, :3] = ink
    return out


def sparkle(cell, mask):
    """가면(`mask` 색) 속 눈 빛점: 가면 색 점을 둘 넘게 곁에 둔 아주 어두운 점(외곽선 색 아님) 가운데 가장 왼쪽 위를 희게.
    원본의 흰 빛점은 32칸에서 사라졌고, 검은 눈이 짙은 회색 가면에 묻혀 얼굴이 가면 덩어리로만 읽혔다."""
    on = cell[..., 3] > 0
    rgb = cell[..., :3].astype(int)
    is_mask = on & np.all(rgb == mask, -1)
    best = None
    for y, x in zip(*np.nonzero(on)):
        key = tuple(int(v) for v in rgb[y, x])
        if key == OUTLINE or (rgb[y, x] * [0.299, 0.587, 0.114]).sum() > 40:
            continue
        near = sum(is_mask[y + dy, x + dx] for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1))
                   if 0 <= y + dy < cell.shape[0] and 0 <= x + dx < cell.shape[1])
        if near >= 2 and (best is None or (y, x) < best):
            best = (y, x)
    out = cell.copy()
    if best:
        out[best[0], best[1], :3] = (255, 255, 255)
    return out


def build(name):
    spr = convert(spec(name))
    src = np.asarray(Image.open(HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png").convert("RGB")).astype(int)
    sheet = np.zeros((CELL * 4, CELL * len(NAMES), 4), np.uint8)
    for c, k in enumerate(NAMES):
        cell = seat(spr[k], CELL, FOOT)
        for pred, ink, hits in SHEETS[name].get("marks", []):
            cell = transfer(cell, src, SHEETS[name]["boxes"][c], pred, ink, hits)
        if "sparkle" in SHEETS[name] and k != "happy":
            cell = sparkle(cell, SHEETS[name]["sparkle"])
        sil = np.zeros_like(cell)
        sil[cell[..., 3] > 0] = [255, 255, 255, 255]
        for r, im in enumerate((cell, cell, sil, sil)):
            sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = im
    return sheet


if __name__ == "__main__":
    for name in sys.argv[1:] or [n for n in SHEETS if (HERE / "sources" / "ai" / "sprites" / f"{n}_v1.png").exists()]:
        Image.fromarray(build(name), "RGBA").save(f"assets/art/{name}.png")
        print(name)
