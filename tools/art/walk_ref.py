"""걷기 네 칸(0229): 나노바나나에 붙일 그림 — 지금 시트(우리 그림)의 서기 칸 + 걷기 두 칸을 6배로, 이웃마다 한 장.

실행: python3 tools/art/walk_ref.py [이름 ...]   (기본: 아래 표 모두 → tools/art/sources/ai/sprites/ref/ref_walk_<이름>.png)

왜: 포포·수수·보리·콩이·토끼풀·모모는 원본 시트에 걷기가 두 칸뿐이라 두 자세를 오간다(사장 "포포 걷는 애니메이션 좀 성의 없군").
도토 달리기(`doto_run_ref.py`, 0226)처럼 네 칸을 따로 받아 시트 끝에 붙인다. 프롬프트는 `docs/art/walk-cycle-prompts.md`.
"""
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ART = HERE.parents[1] / "assets" / "art"
OUT = HERE / "sources" / "ai" / "sprites" / "ref"
# 이름: (시트, 칸 크기, 서기 칸, 걷기 칸들) — 줄 0(돌아온 모습).
SHEETS = {
    "popo": ("popo.png", 32, 0, (2, 3)),
    "bori": ("chief.png", 40, 0, (2, 3)),
    "susu": ("susu.png", 32, 0, (2,)),
    "kongi": ("kongi.png", 32, 0, (2, 3)),
    "clover": ("clover.png", 40, 0, (2, 3)),
    "momo": ("momo.png", 32, 0, (2, 3)),
}


def board(name: str) -> Path:
    file, cell, stand, walk = SHEETS[name]
    sheet = Image.open(ART / file).convert("RGBA")
    cells = [sheet.crop((c * cell, 0, (c + 1) * cell, cell)) for c in (stand, *walk)]
    b = Image.new("RGBA", (len(cells) * (cell + 4) + 8, cell + 4), (255, 0, 255, 255))
    for i, c in enumerate(cells):
        # 서기와 걷기 사이는 한 칸 더 띄운다(왼쪽 = 서기, 오른쪽 = 지금 걷기).
        b.alpha_composite(c, (4 + i * (cell + 4) + (4 if i else 0), 2))
    b = b.resize((b.width * 6, b.height * 6), Image.NEAREST)
    out = OUT / f"ref_walk_{name}.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    b.save(out)
    return out


if __name__ == "__main__":
    for n in sys.argv[1:] or SHEETS:
        print(board(n))
