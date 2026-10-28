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
    "clover": 39,
}
# 스포일러가 없는 일러스트만(타이틀 바탕·오프닝의 엄마 굴).
ILLUSTS = ["title", "mother_burrow"]
# 페이지에서 걷는 스프라이트(0155): (시트, 줄, 첫 칸, 칸 수). 정사각 칸을 가로 띠로 잘라 CSS steps()로 돌린다.
# v0.5.0 새 그림(0206~0221): 시트마다 칸 크기가 다르다(`CELLS`, art::BIG_SHEETS와 같은 값). 줄 0, 칸 2·3 = 걷기
# (나일은 nail.png 줄 0 = 서 있기 2 + 걷기 4, 박쥐는 0~3 날갯짓). 페이지는 칸 크기의 2배(--s)로 그린다.
SPRITES = {
    "nail": ("nail.png", 0, 2, 4),
    "momo": ("momo.png", 0, 2, 2),
    "doto": ("doto.png", 0, 8, 4),  # 달리기 바운드 네 칸(0226)
    "popo": ("popo.png", 0, 2, 2),
    "rat": ("field_mouse.png", 0, 2, 2),
    "squirrel": ("chipmunk.png", 0, 2, 2),
    "shrew": ("shrew.png", 0, 2, 2),
    "hedgehog": ("hedgehog.png", 0, 2, 2),
    "toad": ("toad.png", 0, 2, 2),
    "bat": ("bat.png", 0, 0, 4),
    "sporecap": ("spore.png", 0, 2, 2),
}
CELLS = {"nail.png": 24, "momo.png": 32, "doto.png": 40, "popo.png": 32, "bat.png": 32}


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    sheet = Image.open(ART / "portraits.png").convert("RGBA")
    for name, i in PORTRAITS.items():
        x, y = (i % PER_ROW) * CELL, (i // PER_ROW) * CELL
        sheet.crop((x, y, x + CELL, y + CELL)).save(OUT / f"p_{name}.png", optimize=True)
    for name in ILLUSTS:
        shutil.copyfile(ART / "illust" / f"{name}.png", OUT / f"i_{name}.png")
    for name, (sheet_name, row, first, n) in SPRITES.items():
        sh = Image.open(ART / sheet_name).convert("RGBA")
        c = CELLS.get(sheet_name, 24)
        sh.crop((first * c, row * c, (first + n) * c, row * c + c)).save(OUT / f"sp_{name}.png", optimize=True)
    # 파비콘: 나일 스프라이트 첫 칸.
    c = CELLS["nail.png"]
    Image.open(ART / "nail.png").convert("RGBA").crop((0, 0, c, c)).save(OUT / "favicon.png")
    subset_font()
    bust_cache()


def bust_cache() -> None:
    """style.css·site.js 주소에 내용 해시를 붙인다(0155). Pages는 파일을 몇 분 캐시해서, 새 HTML이 옛 CSS와
    만나면 깊이 눈금이 맨 위 번호 목록으로, 숨긴 "본문으로" 링크가 보였다(사장 피드백 2026-10-27)."""
    import hashlib

    site = ROOT / "site"
    # 글꼴(페이지 글자 서브셋)도 글을 바꾸면 바뀐다: CSS 안의 주소에 먼저 붙이고, 그 CSS로 해시를 잰다.
    css = site / "style.css"
    font = hashlib.sha1((site / "fonts" / "galmuri11-sub.woff2").read_bytes()).hexdigest()[:8]
    css.write_text(
        re.sub(r'(fonts/galmuri11-sub\.woff2)(\?v=[0-9a-f]+)?"', rf'\1?v={font}"', css.read_text(encoding="utf-8")),
        encoding="utf-8",
    )
    tags = {}
    for name in ("style.css", "site.js"):
        tags[name] = hashlib.sha1((site / name).read_bytes()).hexdigest()[:8]
    for page in site.rglob("*.html"):
        text = page.read_text(encoding="utf-8")
        for name, h in tags.items():
            text = re.sub(rf'((?:\.\./)?{re.escape(name)})(\?v=[0-9a-f]+)?"', rf'\1?v={h}"', text)
        page.write_text(text, encoding="utf-8")


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
