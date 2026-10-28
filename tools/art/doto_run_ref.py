"""도토 달리기 칸(0226): 나노바나나에 붙일 그림 — 지금 도토 시트(우리 그림)의 서기·달리기 칸을 6배로.

실행: python3 tools/art/doto_run_ref.py [출력 경로]   (기본 tools/art/sources/ai/sprites/ref/ref_doto_run.png)

왜: 원본 시트(`doto_v1.png`)에 달리기가 한 칸뿐이라 둘째 칸은 같은 그림을 1px 띄웠다 — 달리면 위아래로만 흔들렸다
(사장 "성의 없다"). 청설모는 뛰어 달린다(앞발 짚고 뒷발이 앞발 앞으로 넘어오는 바운드) — 네 칸을 받아 칸 8~11에 붙인다.
"""
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref" / "ref_doto_run.png"
CELL = 40

sheet = Image.open(HERE.parents[1] / "assets" / "art" / "doto.png").convert("RGBA")
cells = [sheet.crop((c * CELL, 0, (c + 1) * CELL, CELL)) for c in (0, 2)]
board = Image.new("RGBA", (CELL * 2 + 12, CELL + 4), (255, 0, 255, 255))
for i, c in enumerate(cells):
    board.alpha_composite(c, (4 + i * (CELL + 4), 2))
board = board.resize((board.width * 6, board.height * 6), Image.NEAREST)
OUT.parent.mkdir(parents=True, exist_ok=True)
board.save(OUT)
print(OUT)
