"""파 내려가기 다시 그리기(0238 덧붙임)의 자세 참고 그림: 지금 dig.png(손 그림, 옛 Naïl)의 Naïl 줄을 새 시트 배치대로.

실행: python3 tools/art/dig_ref.py [출력 경로]   (기본 tools/art/sources/ai/sprites/ref/ref_dig_pose.png)

줄 1: 긁기 0~3 · 줄 2: 우리 보기, 윙크, 긁기 4·5 · 줄 3: 한 바퀴 돌기(등 → 오른쪽 옆 → 배 → 왼쪽 옆, 지금 8칸 중 0·2·4·6).
카메라(0156): 머리가 아래(땅 쪽), 등이 위(우리 쪽) — 이 자세·카메라만 따라 그리게 하고 몸은 새 Naïl 시트를 붙인다.
dig.png는 우리가 코드로 그린 그림(팩 그림 아님)이라 참고로 붙여도 된다(0064는 팩 그림만 막는다).
"""
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref" / "ref_dig_pose.png"
S, Z, GAP = 32, 8, 48
LAYOUT = [[0, 1, 2, 3], [6, 7, 4, 5], [8, 10, 12, 14]]


def main():
    dig = Image.open(ROOT / "assets" / "art" / "dig.png").convert("RGBA")
    cols, rows = len(LAYOUT[0]), len(LAYOUT)
    sheet = Image.new("RGBA", (cols * (S * Z + GAP) + GAP, rows * (S * Z + GAP) + GAP), (255, 0, 255, 255))
    for r, row in enumerate(LAYOUT):
        for c, i in enumerate(row):
            cell = dig.crop((i * S, 0, (i + 1) * S, S)).resize((S * Z, S * Z), Image.NEAREST)
            sheet.alpha_composite(cell, (GAP + c * (S * Z + GAP), GAP + r * (S * Z + GAP)))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(OUT)
    print(OUT)


if __name__ == "__main__":
    main()
