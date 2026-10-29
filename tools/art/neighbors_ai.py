"""이웃 새 그림(0206 C4, 0220): 포포(라쿤)·수수(땅강아지) 32×32 칸, 발바닥 30.

실행: python3 tools/art/neighbors_ai.py [이름 ...]   (기본: 원본이 있는 이웃 모두 → assets/art/<이름>.png)

원본: `sources/ai/sprites/<이름>_v1.png`(✦ 지움). 크기 기준 그림(`neighbors_silhouette.py`의 `ref_neighbors_scale`)을 붙여 받았다.
칸(critters 그 이웃 줄과 같은 뜻 — `art::BIG_SHEETS`가 critters 줄을 여기로 돌린다):
  칸 0·1 서기(들숨·날숨), 2·3 걷기, 4 말하기(포포)·파기(수수), 5 기쁨
  이웃마다 원본 칸(`frames`)이 다르게 와서 `order`로 게임 칸 여섯을 고른다("~이름"은 그 칸을 1px 내쉰 칸, `breathe`).
  줄 0 이웃, 줄 1은 줄 0과 같다(물든 모습이 없다 — 큰 시트 줄 순서를 맞추려고), 줄 2·3 흰 실루엣.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import walk_ai  # noqa: E402
from sprite_ai import OUTLINE, Source, Spec, breathe, convert, seat  # noqa: E402

CELL, FOOT = 32, 30
NAMES = ["idle_a", "idle_b", "walk_a", "walk_b", "act", "happy"]
SHEETS = {
    # 포포: 4×2 여덟 칸으로 왔다(부탁은 3×2, 칸 사이 세로 경계선). 1줄 = 서기·숨·걷기(웃음)·걷기(같은 걸음, 기쁜 눈),
    # 2줄 = 걷기(다른 걸음)·턱 괴고 말하기·기쁨 둘. 걷기는 걸음이 다른 1줄 셋째·2줄 첫째.
    "popo": dict(
        frames=dict(zip(NAMES, [(33, 12, 196, 184), (292, 21, 459, 184), (547, 11, 717, 185),
                                (36, 200, 204, 375), (292, 200, 466, 375), (549, 200, 716, 375)])),
        order=NAMES,
        body_h=24,      # 외곽선 안 키 → 외곽선까지 26
        strip=6,
        # 귀 뒤 연필: 노랑은 원본에서 드물어 색표(NCOL)에 못 들고, 가는 빗금이라 다수결에도 진다 → 원본에서 옮겨 칠한다.
        marks=[(lambda r: (r[..., 0] > 190) & (r[..., 1] > 140) & (r[..., 2] < 110), (236, 190, 64), 3)],
        # 가면 속 눈: 검은 눈이 짙은 회색 가면에 묻혀 빛점을 찍는다(기쁨 칸은 감은 눈이라 뺀다).
        sparkle=(56, 61, 64),
    ),
    # 수수: 첫 시도(3×2 여섯 칸, 참고 그림 다섯 장)는 나오지 않았다 — 새 대화에서 캐릭터 시트 + 크기 기준 두 장만 붙이고
    # 2×2 네 칸(서기·걷기·파기·기쁨)으로 줄여 받았다. 날숨은 서기를 1px 내쉬고, 걷기는 걷는 칸 ↔ 선 칸.
    # 파기 칸의 흙은 보랏빛이라 마젠타 바탕과 함께 지워진다.
    "susu": dict(
        frames={"stand": (54, 86, 447, 469), "walk": (534, 86, 981, 469), "dig": (43, 587, 468, 970),
                "happy": (577, 587, 938, 970)},
        order=["stand", "~stand", "walk", "stand", "dig", "happy"],
        body_h=17,      # 외곽선 안 키 → 외곽선까지 19
        strip=12,
        # 더듬이: 가는 짙은 갈색 줄이라 줄이면 사라진다 — 몸 밖으로 뻗은 부분만 원본에서 옮겨 그린다.
        marks=[(lambda r: ((r * [0.299, 0.587, 0.114]).sum(-1) > 30) & ((r * [0.299, 0.587, 0.114]).sum(-1) < 80)
                & (r[..., 0] - r[..., 2] > 15), (70, 44, 30), 60, True)],
        eyes=True,
    ),
}


def spec(name):
    s = SHEETS[name]
    src = HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png"
    ref = s["order"][0]
    return Spec(
        sources=[Source(src, s["frames"], groups=dict.fromkeys(s["frames"], ref), tidy=0.15,
                        strip=s["strip"], pink=s.get("pink", False))],
        palette_ref=(0, ref),
        body_h=s["body_h"],
    )


def transfer(cell, src, box, pred, ink, hits, outside=False):
    """원본 상자 `box`에서 `pred`에 맞는 점을 몸 겉 네모끼리 맞춰 칸으로 옮기고, 칸 점 하나에 `hits`개 넘게 모이면 `ink`로
    칠한다(외곽선 안쪽만). 작은 표지(연필·고글)가 색표·다수결에서 사라지지 않게. `outside`면 거꾸로 몸 밖(투명한 점)에만
    칠한다 — 몸 밖으로 뻗는 가는 것(더듬이)."""
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
        if outside and not on[y, x]:
            out[y, x] = list(ink) + [255]
        elif not outside and on[y, x] and tuple(int(v) for v in cell[y, x, :3]) != OUTLINE:
            out[y, x, :3] = ink
    return out


def body_map(cell, src, box):
    """원본 상자 → 칸 좌표로 옮기는 함수(몸 겉 네모끼리)."""
    x0, y0, x1, y1 = box
    r = src[y0:y1, x0:x1]
    body = (np.minimum(r[..., 0], r[..., 2]) - r[..., 1]) < 60
    rows = np.nonzero(body.sum(1) >= 6)[0]
    cols = np.nonzero(body.sum(0) >= 6)[0]
    cys, cxs = np.nonzero(cell[..., 3] > 0)
    sy = (cys.max() - cys.min() + 1) / (rows.max() - rows.min() + 1)
    sx = (cxs.max() - cxs.min() + 1) / (cols.max() - cols.min() + 1)
    return r, body, lambda y, x: (cys.min() + (y - rows.min()) * sy, cxs.min() + (x - cols.min()) * sx)


def redraw_eyes(cell, src, box, happy, ink=OUTLINE, band=(0.2, 0.55)):
    """눈을 원본에서 다시 찍는다: 원본의 검은 눈(밝기 45 미만, 바탕에 닿지 않는 덩어리, 몸 위쪽 반, 꽉 찬 둥근 꼴)을 찾아
    (몸 키의 20~55% 줄 — 고글·앞발 테두리를 빼려고) 칸 자리로 옮기고, 그 둘레의 줄이며 남은 검은 점을 얼굴색으로 덮은 뒤 2×2 눈 + 오른쪽 위 흰 빛점(기쁨은 ^)을 찍는다.
    32칸으로 줄이면 원본의 둥근 눈 + 빛점이 십자 모양 검은 덩어리가 됐다. `ink`는 눈 색(잡몹은 물든 붉은 눈, 0221)."""
    r, body, to = body_map(cell, src, box)
    dark = body & ((r * [0.299, 0.587, 0.114]).sum(-1) < 45)
    h, w = dark.shape
    seen = np.zeros_like(dark)
    found = []
    for sy, sx in zip(*np.nonzero(dark)):
        if seen[sy, sx]:
            continue
        stack, comp, edge = [(sy, sx)], [], False
        seen[sy, sx] = True
        while stack:
            y, x = stack.pop()
            comp.append((y, x))
            for ny, nx in ((y + dy, x + dx) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dy or dx):   # 8방향: ^는 대각선 블록
                if not (0 <= ny < h and 0 <= nx < w) or not body[ny, nx]:
                    edge = True
                elif dark[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    stack.append((ny, nx))
        ys, xs = [c[0] for c in comp], [c[1] for c in comp]
        bh, bw = max(ys) - min(ys) + 1, max(xs) - min(xs) + 1
        fill = len(comp) / (bh * bw)
        rows = np.nonzero(body.any(1))[0]
        at = (np.mean(ys) - rows.min()) / (rows.max() - rows.min())
        face_band = band[0] < at < band[1]                # 고글(맨 위)·앞발(아래)의 검은 테두리는 빼고
        eye_like = 300 < len(comp) < 4000 and fill > 0.5 and 0.4 < bh / bw < 2.5
        arc_like = 200 < len(comp) < 2000 and bh >= 25 and 0.3 < bh / bw <= 1.05   # 기쁨 ^: 속이 빈 꺾쇠(입·이마 줄은 낮다)
        if not edge and face_band and (arc_like if happy else eye_like):
            found.append((np.mean(ys), np.mean(xs), comp))
    out = cell.copy()
    on = cell[..., 3] > 0
    rgb = cell[..., :3].astype(int)
    lum = (rgb * [0.299, 0.587, 0.114]).sum(-1)
    for my, mx, comp in found:
        ty, tx = (int(round(v)) for v in to(my, mx))
        box_px = [(y, x) for y in range(ty - 2, ty + 3) for x in range(tx - 2, tx + 3)
                  if 0 <= y < 32 and 0 <= x < 32 and on[y, x]]
        face = [tuple(int(v) for v in rgb[y, x]) for y, x in box_px if lum[y, x] > 80 and not (rgb[y, x, 0] > 200)]
        base = max(face, key=face.count) if face else None
        for y, x in box_px:
            interior = all(0 <= y + dy < 32 and 0 <= x + dx < 32 and on[y + dy, x + dx]
                           for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)))
            if base and interior and lum[y, x] < 60:
                out[y, x, :3] = base
        glyph = [(0, -1), (-1, 0), (0, 1)] if happy else [(-1, -1), (-1, 0), (0, -1), (0, 0)]
        for dy, dx in glyph:
            out[ty + dy, tx + dx, :3] = ink
        if not happy:
            out[ty - 1, tx, :3] = (255, 255, 255)
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
    """칸 0~5(`NAMES`) + 걷기 네 칸 6~9(따로 받은 걷기 시트가 있으면, `walk_ai`, 0229)."""
    sp = spec(name)
    spr = convert(sp)
    src = np.asarray(Image.open(HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png").convert("RGB")).astype(int)
    # (칸 이름, 줄인 그림, 표지·눈을 옮겨 올 원본, 그 원본의 상자)
    plan = []
    for k, src_name in zip(NAMES, SHEETS[name]["order"]):
        base = src_name.lstrip("~")
        plan.append((k, seat(breathe(spr[base]) if src_name.startswith("~") else spr[base], CELL, FOOT), src,
                     SHEETS[name]["frames"][base]))
    if name in walk_ai.WALK and walk_ai.source(name).exists():
        wsrc = np.asarray(Image.open(walk_ai.source(name)).convert("RGB")).astype(int)
        walk = walk_ai.cells(name, sp.sources[0], CELL, FOOT, sp.palette_ref[1])
        plan += [(k, walk[k], wsrc, box) for k, box in zip(walk_ai.NAMES, walk_ai.WALK[name]["boxes"])]
    sheet = np.zeros((CELL * 4, CELL * len(plan), 4), np.uint8)
    for c, (k, cell, src, box) in enumerate(plan):
        for pred, ink, hits, *outside in SHEETS[name].get("marks", []):
            cell = transfer(cell, src, box, pred, ink, hits, *outside)
        if SHEETS[name].get("eyes"):
            cell = redraw_eyes(cell, src, box, k == "happy")
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
