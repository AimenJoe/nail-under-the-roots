#!/usr/bin/env python3
"""CC0 식물 그림 → 게임 시트 `assets/art/plants.png`(0306).

원본(레포 안, CC0 — 출처는 `tools/art/sources/cc0/README.md`·`assets/LICENSES.md`):
- `sources/cc0/idylwild_foliage_pack.png` — Idylwild's Foliage Pack(OpenGameArt node 171914), 16×16 칸.
  → 0줄: 마을 화단 꽃 여섯(텃밭에서 길러 마을에 옮겨 심는 꽃, `garden::Bloom`). 칸 16×16.
- `sources/cc0/spiderdave_plants.png` — SpiderDave "Flowers"(OpenGameArt), 12×24 칸 78개, 바탕색 (34,32,52).
  → 1줄: 들판·마을 바탕에 굽는 작은 들꽃(`terrain` 굽기). 칸 12×24, 바탕은 투명.

  python3 tools/art/plants_cc0.py
"""
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "tools/art/sources/cc0"
OUT = ROOT / "assets/art/plants.png"

# 화단 꽃(Idylwild 칸 (열, 줄)) — 순서가 `garden::Bloom`의 Blue..Yellow(0306 시안 1~6).
BLOOMS = [(14, 15), (14, 16), (13, 17), (14, 17), (13, 18), (14, 18)]
BLOOM = 16
# 바탕 들꽃(SpiderDave 번호) — 가는 들꽃·작은 꽃 무리만(버섯·덩굴·부들은 뺀다). 순서는 `terrain::BG_PLANTS` 수와 맞는다.
WILD = [15, 16, 17, 18, 19, 20, 24, 33, 36, 37, 38, 41, 44, 47, 50, 55]
WILD_W, WILD_H = 12, 24


def main():
    idy = Image.open(SRC / "idylwild_foliage_pack.png").convert("RGBA")
    sd = Image.open(SRC / "spiderdave_plants.png").convert("RGBA")
    bg = sd.getpixel((0, 0))[:3]
    w = max(len(BLOOMS) * BLOOM, len(WILD) * WILD_W)
    out = Image.new("RGBA", (w, BLOOM + WILD_H), (0, 0, 0, 0))
    for i, (cx, cy) in enumerate(BLOOMS):
        out.paste(idy.crop((cx * BLOOM, cy * BLOOM, cx * BLOOM + BLOOM, cy * BLOOM + BLOOM)), (i * BLOOM, 0))
    for i, n in enumerate(WILD):
        cell = sd.crop((n * WILD_W, 0, n * WILD_W + WILD_W, WILD_H))
        px = cell.load()
        for y in range(WILD_H):
            for x in range(WILD_W):
                if px[x, y][:3] == bg:
                    px[x, y] = (0, 0, 0, 0)
        out.paste(cell, (i * WILD_W, BLOOM))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    out.save(OUT, optimize=True)
    print(f"{OUT.relative_to(ROOT)} {out.size}")


if __name__ == "__main__":
    main()
