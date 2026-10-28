"""소품 새 그림(C5, 0222): 부술 수 있는 것 넷 + 보물 상자(닫힘·열림)를 받으려고 나노바나나에 붙일 크기·생김새 기준 한 장.

실행: python3 tools/art/props_ref.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_props.png: 게임 크기 Naïl(24칸) 옆에 여섯 소품을 목표 크기로 칠한 색 안내(지금 코드 그림 `render/props.rs`의
    `draw_breakable`·`draw_chest`와 같은 모양·색을 키운 것), 마젠타 바탕 8배.

왜: 부술 수 있는 것은 거의 모든 굴 방에 나오는데 지금은 폭 10px쯤의 밋밋한 덩어리라 24칸 잡몹(0221) 곁에서 가장 작다.
목표: 바닥 칸 하나(16px) 위에 폭 14~16·키 11~14px. 넷이 나눠 쓰는 표시(0205: 어두운 외곽선 = 바닥 장식이 아니라 물건,
밝은 금 = 깨진다)는 그림에 넣고, 2.5초마다의 반짝은 게임이 그린다.
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL, FOOT = 24, 22
Z = 8


def ell(d, cx, cy, rx, ry, c):
    d.ellipse([(cx - rx) * Z, (cy - ry) * Z, (cx + rx) * Z, (cy + ry) * Z], fill=c)


def crack(d, cx, top):
    for i, dx in enumerate((1, 0, 1, 2, 1)):
        d.rectangle([(cx + dx) * Z, (top + i) * Z, (cx + dx + 1) * Z - 1, (top + i + 1) * Z - 1], fill=(255, 242, 210))


OUTLINE = (43, 29, 42)


def mound(d, cx):
    ell(d, cx, FOOT - 4, 8, 5.5, OUTLINE)
    ell(d, cx, FOOT - 4, 7, 4.5, (107, 77, 51))
    ell(d, cx - 0.6, FOOT - 5.5, 5.5, 3.2, (158, 120, 82))
    crack(d, cx, FOOT - 9)


def gall(d, cx):
    ell(d, cx, FOOT - 6, 7, 7, OUTLINE)
    ell(d, cx, FOOT - 6, 6, 6, (102, 56, 46))
    ell(d, cx - 1, FOOT - 7.5, 4.4, 3.8, (148, 84, 66))
    d.line([((cx - 6) * Z, (FOOT - 3) * Z), ((cx + 5) * Z, (FOOT - 8) * Z)], fill=(71, 41, 31), width=Z)
    crack(d, cx, FOOT - 11)


def rocks(d, cx):
    for x, y, rx, ry in ((cx - 4, FOOT - 3, 4.5, 3.2), (cx + 4, FOOT - 3, 4.5, 3.2), (cx, FOOT - 7.5, 4.5, 3.8)):
        ell(d, x, y, rx + 1, ry + 1, OUTLINE)
    for x, y, rx, ry, c in ((cx - 4, FOOT - 3, 4.5, 3.2, (92, 84, 92)), (cx + 4, FOOT - 3, 4.5, 3.2, (92, 84, 92)),
                            (cx, FOOT - 7.5, 4.5, 3.8, (148, 140, 148))):
        ell(d, x, y, rx, ry, c)
    crack(d, cx, FOOT - 10)


def silk(d, cx):
    ell(d, cx, FOOT - 6, 7, 6.5, OUTLINE)
    ell(d, cx, FOOT - 6, 6, 5.5, (184, 174, 204))
    ell(d, cx - 1, FOOT - 7.2, 4.4, 3.8, (235, 230, 250))
    for a, b in (((cx - 6, FOOT - 6), (cx + 6, FOOT - 5)), ((cx - 3, FOOT - 11), (cx + 2, FOOT - 1))):
        d.line([(a[0] * Z, a[1] * Z), (b[0] * Z, b[1] * Z)], fill=(153, 128, 191), width=Z)
    crack(d, cx, FOOT - 10)


def chest(d, cx, opened):
    x0, x1 = cx - 8, cx + 8
    d.rectangle([x0 * Z, (FOOT - 12) * Z, x1 * Z, FOOT * Z], fill=OUTLINE)
    d.rectangle([(x0 + 1) * Z, (FOOT - 7) * Z, (x1 - 1) * Z, (FOOT - 1) * Z], fill=(140, 92, 56))
    d.rectangle([(x0 + 1) * Z, (FOOT - 3) * Z, (x1 - 1) * Z, (FOOT - 1) * Z], fill=(92, 56, 36))
    if opened:
        d.rectangle([x0 * Z, (FOOT - 17) * Z, x1 * Z, (FOOT - 11) * Z], fill=OUTLINE)
        d.rectangle([(x0 + 1) * Z, (FOOT - 16) * Z, (x1 - 1) * Z, (FOOT - 12) * Z], fill=(92, 56, 36))
        d.rectangle([(x0 + 2) * Z, (FOOT - 11) * Z, (x1 - 2) * Z, (FOOT - 8) * Z], fill=(14, 8, 14))
    else:
        d.rectangle([(x0 + 1) * Z, (FOOT - 11) * Z, (x1 - 1) * Z, (FOOT - 7) * Z], fill=(140, 92, 56))
        d.rectangle([(cx - 1) * Z, (FOOT - 9) * Z, (cx + 1) * Z, (FOOT - 5) * Z], fill=(242, 204, 89))


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    items = [mound, gall, rocks, silk, lambda d, cx: chest(d, cx, False), lambda d, cx: chest(d, cx, True)]
    w = 4 + 24 + 6 + len(items) * (CELL + 6) + 4
    board = Image.new("RGBA", (w * Z, (CELL + 8) * Z), (255, 0, 255, 255))
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    board.alpha_composite(nail.resize((24 * Z, 24 * Z), Image.NEAREST), (4 * Z, 4 * Z))
    d = ImageDraw.Draw(board)
    for i, fn in enumerate(items):
        ox = 4 + 24 + 6 + i * (CELL + 6)
        sub = Image.new("RGBA", (CELL * Z, CELL * Z), (0, 0, 0, 0))
        fn(ImageDraw.Draw(sub), CELL / 2)
        board.alpha_composite(sub, (ox * Z, 4 * Z))
    board.save(OUT / "ref_props.png")
    del d


if __name__ == "__main__":
    main()
