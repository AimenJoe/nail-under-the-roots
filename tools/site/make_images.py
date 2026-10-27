"""공식 페이지(site/) 그림·글꼴 만들기(docs/decisions/0069).

게임 그림(assets/art, 초상화·일러스트는 나노바나나)에서 초상화를 잘라 내고 일러스트를 복사한다. 게임 PNG는 고치지 않는다.
글꼴은 게임의 Galmuri11을 페이지(site/**/*.html)에 나오는 글자만 남겨 woff2로 줄인다(OFL, 예약 이름 없음).
페이지 글을 고치면 다시 돌린다. 스크린숏은 따로 찍는다(tools/site/shots.js).

    python3 tools/site/make_images.py
"""

import re
import shutil
from html import unescape
from pathlib import Path

from fontTools import subset
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
ART = ROOT / "assets" / "art"
OUT = ROOT / "site" / "img"

# 초상화 표(src/cutscene.rs의 Portrait::first): 88×88, 한 줄에 12칸. 평소 얼굴(첫 칸)만.
CELL, PER_ROW = 88, 12
PORTRAITS = {
    "nail": 0,
    "popo": 5,
    "bori": 10,
    "momo": 12,
    "halmae": 15,
    "doto": 31,
    "susu": 33,
    "kongi": 36,
}
# 스포일러가 없는 일러스트만(오프닝·첫 굴).
ILLUSTS = ["great_tree", "molehill"]


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    sheet = Image.open(ART / "portraits.png").convert("RGBA")
    for name, i in PORTRAITS.items():
        x, y = (i % PER_ROW) * CELL, (i // PER_ROW) * CELL
        sheet.crop((x, y, x + CELL, y + CELL)).save(OUT / f"p_{name}.png", optimize=True)
    for name in ILLUSTS:
        shutil.copyfile(ART / "illust" / f"{name}.png", OUT / f"i_{name}.png")
    # 파비콘: 나일 스프라이트 첫 칸(16×16).
    Image.open(ART / "nail.png").convert("RGBA").crop((0, 0, 16, 16)).save(OUT / "favicon.png")
    subset_font()


def subset_font() -> None:
    site = ROOT / "site"
    text = ""
    for page in site.rglob("*.html"):
        raw = re.sub(r"<[^>]+>", " ", page.read_text(encoding="utf-8"))
        text += unescape(raw)
    # 스크립트가 채우는 글(버전·날짜·릴리스 부제)에 쓰일 글자도 넣는다.
    text += "0123456789.v-—·()ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
    fonts = site / "fonts"
    fonts.mkdir(exist_ok=True)
    opts = subset.Options()
    opts.flavor = "woff2"
    opts.layout_features = ["*"]
    font = subset.load_font(str(ROOT / "assets" / "fonts" / "Galmuri11-Condensed.ttf"), opts)
    sub = subset.Subsetter(opts)
    sub.populate(text=text)
    sub.subset(font)
    subset.save_font(font, str(fonts / "galmuri11-sub.woff2"), opts)
    shutil.copyfile(ROOT / "assets" / "fonts" / "OFL.md", fonts / "OFL.md")


if __name__ == "__main__":
    main()
