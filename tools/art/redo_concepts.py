"""다시 그리기 1단계 안(0238)을 게임 크기로 줄여 보는 시험 판.

실행: python3 tools/art/redo_concepts.py [출력 폴더]   (기본: 이 파일 옆 sources/ai/sprites/ref)
  - redo_nail_test.png·redo_kongi_test.png: 안마다 [원본 | 키마다 색 ×4 | 같은 크기 한 색 실루엣 ×4],
    맨 왼쪽 열은 지금 게임 그림(Naïl·모모·도토·포포 서기 칸)을 같은 배율로 — 크기·읽힘을 나란히 본다.

왜(0206·0238): 큰 그림에서 두더지로 보여도 18~24px로 줄이면 갈색 덩어리가 될 수 있다. 고르기 전에 실제 크기의
한 색 실루엣에서 삽 손·코·몸(콩이는 등딱지 띠·귀·꼬리)이 읽히는지 본다.
원본은 마젠타 바탕(✦는 strip_logo.py로 지운 뒤). 콩이 판의 칸 사이 검은 세로줄은 바탕으로 친다.
"""
import sys
from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
SPRITES = HERE / "sources" / "ai" / "sprites"
ASSETS = HERE.parent.parent / "assets" / "art"
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else SPRITES / "ref"

ZOOM = 4
LEFT = 360           # 왼쪽 열(지금 게임 그림) 폭
FONT = ImageFont.truetype(str(HERE.parent.parent / "assets" / "fonts" / "Galmuri11-Condensed.ttf"), 12)
BG = (236, 228, 214)
INK = (58, 34, 52)          # 실루엣 색(그림 규칙의 짙은 자두 외곽선)
# 지금 게임 그림(파일, 칸 크기) — 서기 첫 칸.
NOW = [("Naïl", "nail.png", 24), ("모모", "momo.png", 32), ("도토", "doto.png", 40), ("포포", "popo.png", 32)]
NOW_KONGI = [("콩이", "kongi.png", 32), ("도토", "doto.png", 40), ("포포", "popo.png", 32)]


def foreground(rgb):
    r, g, b = (rgb[..., i].astype(int) for i in range(3))
    magenta = (r > 170) & (b > 170) & (g < 110)
    fg = ~magenta
    # 칸 사이 검은 세로줄(콩이 판): 거의 전부 어두운 열.
    dark = (r + g + b) < 120
    cols = dark.mean(axis=0) > 0.8
    fg[:, cols] = False
    return fg


def components(fg, min_px=400):
    """4-이웃 덩어리. 작은 것(반딧불·티끌)은 버린다. (넓이, 상자, 마스크) 목록."""
    h, w = fg.shape
    seen = np.zeros_like(fg)
    out = []
    for y0, x0 in zip(*np.nonzero(fg)):
        if seen[y0, x0]:
            continue
        q = deque([(y0, x0)])
        seen[y0, x0] = True
        pts = []
        while q:
            y, x = q.popleft()
            pts.append((y, x))
            for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ny, nx = y + dy, x + dx
                if 0 <= ny < h and 0 <= nx < w and fg[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    q.append((ny, nx))
        if len(pts) < min_px:
            continue
        ys, xs = np.array(pts).T
        m = np.zeros_like(fg)
        m[ys, xs] = True
        out.append((len(pts), (xs.min(), ys.min(), xs.max() + 1, ys.max() + 1), m))
    return out


def shrink(rgb, mask, box, height):
    """상자를 키 `height`로 줄인다(넓이 평균, 알파 반 넘으면 칠함). RGBA 배열."""
    x0, y0, x1, y1 = box
    a = mask[y0:y1, x0:x1].astype(np.float32)
    c = rgb[y0:y1, x0:x1].astype(np.float32) * a[..., None]
    w = max(1, round((x1 - x0) * height / (y1 - y0)))
    size = (w, height)
    small_a = np.asarray(Image.fromarray((a * 255).astype(np.uint8)).resize(size, Image.BOX)).astype(np.float32) / 255
    chans = [np.asarray(Image.fromarray(c[..., i].astype(np.uint8)).resize(size, Image.BOX)).astype(np.float32) / 255
             for i in range(3)]
    out = np.zeros((height, w, 4), np.uint8)
    on = small_a >= 0.45
    for i in range(3):
        out[..., i] = np.where(on, np.clip(chans[i] * 255 / np.maximum(small_a, 1e-3), 0, 255), 0)
    out[..., 3] = on * 255
    return out


def silhouette(cell):
    s = np.zeros_like(cell)
    s[..., :3] = INK
    s[..., 3] = cell[..., 3]
    return s


def game_idle(name, cell):
    im = np.asarray(Image.open(ASSETS / name).convert("RGBA"))[:cell, :cell]
    ys, xs = np.nonzero(im[..., 3])
    return im[ys.min():ys.max() + 1, xs.min():xs.max() + 1]


def paste(sheet, arr, x, y_bottom):
    im = Image.fromarray(arr, "RGBA").resize((arr.shape[1] * ZOOM, arr.shape[0] * ZOOM), Image.NEAREST)
    sheet.alpha_composite(im, (x, y_bottom - im.height))
    return im.width


def figures(path, keep):
    """원본에서 큰 덩어리 `keep`개(왼쪽부터). 콩이는 칸마다 선 그림 + 공이라 큰 것만 고르고 공은 따로."""
    rgb = np.asarray(Image.open(path).convert("RGB"))
    comps = sorted(components(foreground(rgb)), key=lambda c: -c[0])[:keep]
    return rgb, sorted(comps, key=lambda c: c[1][0])


def build(path, heights, now, keep, out_name, title):
    rgb, comps = figures(path, keep)
    # 콩이 판: 큰 덩어리 열 개 = 선 그림 다섯 + 공 다섯. 칸마다 넓이 큰 쪽이 선 그림.
    if keep == 10:
        stand = sorted(sorted(comps, key=lambda c: -c[0])[:5], key=lambda c: c[1][0])
        balls = sorted(sorted(comps, key=lambda c: -c[0])[5:], key=lambda c: c[1][0])
    else:
        stand, balls = comps, []
    col_w = 200
    row_h = max(heights) * ZOOM + 24
    W = LEFT + col_w * len(stand)
    H = 150 + row_h * len(heights) * 2 + 40
    sheet = Image.new("RGBA", (W, H), BG + (255,))
    d = ImageDraw.Draw(sheet)
    d.text((8, 6), title, fill=INK, font=FONT)
    # 원본 줄
    for i, (_, box, m) in enumerate(stand):
        x0, y0, x1, y1 = box
        crop = np.zeros((y1 - y0, x1 - x0, 4), np.uint8)
        crop[..., :3] = rgb[y0:y1, x0:x1]
        crop[..., 3] = m[y0:y1, x0:x1] * 255
        im = Image.fromarray(crop, "RGBA")
        im.thumbnail((col_w - 10, 120))
        sheet.alpha_composite(im, (LEFT + i * col_w, 140 - im.height))
        d.text((LEFT + i * col_w, 142), f"{i + 1}", fill=INK, font=FONT)
    y = 150 + 10
    for hgt in heights:
        for kind in ("색", "실루엣"):
            yb = y + row_h - 14
            d.text((8, y + 2), f"키 {hgt}px {kind}", fill=INK, font=FONT)
            # 지금 게임 그림(같은 배율, 크기 비교용) — 맨 왼쪽 칸 안에 작게.
            x = 8
            for name, f, cell in now[:3]:
                g = game_idle(f, cell)
                x += paste(sheet, g if kind == "색" else silhouette(g), x, yb) + 6
            for i, (_, box, m) in enumerate(stand):
                s = shrink(rgb, m, box, hgt)
                paste(sheet, s if kind == "색" else silhouette(s), LEFT + i * col_w, yb)
                if balls and kind == "색":
                    _, bb, bm = balls[i]
                    ball = shrink(rgb, bm, bb, max(10, hgt * 2 // 3))
                    paste(sheet, ball, LEFT + i * col_w + s.shape[1] * ZOOM + 4, yb)
            y += row_h
    d.text((8, H - 30), "왼쪽: 지금 게임 그림(" + "·".join(n for n, _, _ in now[:3]) + f") 같은 ×{ZOOM}", fill=INK, font=FONT)
    OUT.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(OUT / out_name)
    print(OUT / out_name)


if __name__ == "__main__":
    build(SPRITES / "nail_redo_concepts_v1.png", [18, 22, 24], NOW, 5, "redo_nail_test.png", "Naïl 다시 그리기 1단계 — 실제 크기 시험")
    build(SPRITES / "kongi_redo_concepts_v1.png", [20, 24], NOW_KONGI, 10, "redo_kongi_test.png", "콩이 다시 그리기 1단계 — 실제 크기 시험")


def tree_test(path, heights, out_name):
    """큰 나무 두 장(왼쪽 시든 · 오른쪽 다 핀)을 마을 크기로. 반딧불 같은 작은 덩어리는 버린다."""
    rgb = np.asarray(Image.open(path).convert("RGB"))
    half = rgb.shape[1] // 2
    trees = []
    for x0 in (0, half):
        part = rgb[:, x0:x0 + half]
        comps = sorted(components(foreground(part), min_px=2000), key=lambda c: -c[0])
        big = comps[0]
        mask = np.zeros(part.shape[:2], bool)
        for _, _, m in comps[:3]:
            mask |= m
        ys, xs = np.nonzero(mask)
        trees.append((part, mask, (xs.min(), ys.min(), xs.max() + 1, ys.max() + 1)))
        del big
    W = 40 + sum(max(1, round((b[2] - b[0]) * max(heights) / (b[3] - b[1]))) * ZOOM + 40 for _, _, b in trees) * len(heights)
    H = max(heights) * ZOOM + 60
    sheet = Image.new("RGBA", (W, H), BG + (255,))
    d = ImageDraw.Draw(sheet)
    x = 20
    for hgt in heights:
        d.text((x, 6), f"키 {hgt}px ×{ZOOM}", fill=INK, font=FONT)
        for part, mask, box in trees:
            s = shrink(part, mask, box, hgt)
            x += paste(sheet, s, x, H - 10) + 30
        x += 20
    OUT.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(OUT / out_name)
    print(OUT / out_name)


if __name__ == "__main__":
    tree_test(SPRITES / "tree_v1.png", [90, 110], "redo_tree_test.png")
