"""Epic Games Store 상점 그림을 게임 그림에서 만든다(규격: Storefront Media Guide, 2026-11-06 확인).

모든 그림은 게임 도트를 정수배로만 키운다(가장 가까운 칸 — 흐려지지 않게). 로고 글자는 게임 글꼴 Galmuri11.
그림 원본은 나노바나나 원본을 도트로 옮긴 것(assets/art/illust, AI 사용은 숨기지 않는다 — 상점 About 글에 적는다).

    python3 tools/store/epic_media.py      # docs/epic/media/*.png·jpg

| 파일 | 규격 | 쓰는 곳 |
|---|---|---|
| product-logo-960x540.png | 960×540 PNG ≤1MB, 투명 | Store Settings → Product details |
| offer-landscape-2560x1440.png | 2560×1440 | Offers → Images(일반·라이브러리) |
| offer-portrait-1200x1600.png | 1200×1600 | Offers → Images(일반·라이브러리) |
| carousel-*.png | 1920×1080 ≤4MB | Pages → Media carousel |
| social-1200x1200.jpg | 1200×1200 JPG ≤4MB | Pages → Social preview(선택) |
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
ILL = ROOT / "assets" / "art" / "illust"
SHOTS = ROOT / "site" / "img"
OUT = ROOT / "docs" / "epic" / "media"
FONT = ROOT / "assets" / "fonts" / "Galmuri11-Condensed.ttf"

CREAM = (236, 229, 218, 255)
GLOW = (242, 205, 92, 255)
INK = (23, 19, 15, 255)

# 상점 캐러셀에 넣을 스크린숏(공식 페이지의 960×540 = 게임 320×180의 3배 → 2배 하면 1920×1080)
CAROUSEL = ["s_meadow", "s_roots", "s_boss", "s_village", "s_silk", "s_hall"]


def text_layer(text: str, px: int, color, outline=INK, bold: bool = True) -> Image.Image:
    """도트 글꼴을 원래 크기(px)로 찍고(굵게 = 오른쪽으로 1칸 겹침) 외곽선 2칸을 두른 작은 그림.
    키우는 것은 부르는 쪽이 정수배로."""
    f = ImageFont.truetype(str(FONT), px)
    l, t, r, b = f.getbbox(text)
    pad = 3
    w, h = r - l + 2 * pad + 1, b - t + 2 * pad
    mask = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(mask)
    for bx in ((0, 1) if bold else (0,)):
        d.text((pad - l + bx, pad - t), text, font=f, fill=255)
    mask = mask.point(lambda v: 255 if v >= 128 else 0)  # 안티에일리어스를 끊어 도트로
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ol = Image.new("RGBA", (w, h), outline)
    for dx in range(-2, 3):
        for dy in range(-2, 3):
            if abs(dx) + abs(dy) <= 3:
                im.paste(ol, (dx, dy), mask)
    im.paste(Image.new("RGBA", (w, h), color), (0, 0), mask)
    return im


def wordmark() -> Image.Image:
    """'Naïl' 큰 글자 + 'UNDER THE ROOTS' 작은 글자(도트 크기에서 짠 뒤 한꺼번에 키운다)."""
    big = text_layer("Naïl", 24, CREAM)
    small = text_layer("UNDER THE ROOTS", 12, GLOW, bold=False)
    w = max(big.width, small.width)
    im = Image.new("RGBA", (w, big.height + small.height - 1), (0, 0, 0, 0))
    im.alpha_composite(big, ((w - big.width) // 2, 0))
    im.alpha_composite(small, ((w - small.width) // 2, big.height - 1))
    return im


def up(im: Image.Image, k: int) -> Image.Image:
    return im.resize((im.width * k, im.height * k), Image.NEAREST)


def place(base: Image.Image, mark: Image.Image, cx: int, cy: int) -> None:
    base.alpha_composite(mark, (cx - mark.width // 2, cy - mark.height // 2))


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    wm = wordmark()

    # 제품 로고: 960×540 투명, 글자를 들어가는 가장 큰 정수배로
    k = min(900 // wm.width, 500 // wm.height)
    logo = Image.new("RGBA", (960, 540), (0, 0, 0, 0))
    place(logo, up(wm, k), 480, 270)
    logo.save(OUT / "product-logo-960x540.png", optimize=True)

    # 가로 판: 타이틀 그림 ×4, 로고는 하늘(가운데 위)
    title = Image.open(ILL / "title.png").convert("RGBA")
    land = up(title, 4)
    place(land, up(wm, 11), 1280, 420)
    land.convert("RGB").save(OUT / "offer-landscape-2560x1440.png", optimize=True)

    # 세로 판: 큰 나무 그림에서 300×400을 잘라 ×4, 로고는 위쪽
    tree = Image.open(ILL / "great_tree.png").convert("RGBA")
    x0 = (tree.width - 300) // 2
    port = up(tree.crop((x0, 0, x0 + 300, 400)), 4)
    # 주인공: 걷는 띠의 첫 칸(28칸) ×8을 뿌리 앞에
    nail = Image.open(SHOTS / "sp_nail.png").convert("RGBA").crop((0, 0, 28, 28))
    port.alpha_composite(up(nail, 8), (330 - 112, 1180 - 28 * 8))  # 왼쪽 풀밭 위(뿌리 위는 어두워 묻힌다)
    place(port, up(wm, 7), 600, 300)
    port.convert("RGB").save(OUT / "offer-portrait-1200x1600.png", optimize=True)

    # 캐러셀: 스크린숏 ×2 = 1920×1080
    for i, name in enumerate(CAROUSEL, 1):
        up(Image.open(SHOTS / f"{name}.png").convert("RGB"), 2).save(OUT / f"carousel-{i}-{name[2:]}.png", optimize=True)

    # 소셜 미리보기: 타이틀 그림 왼쪽(굴 속 나일) 300×300 ×4 = 1200×1200, 작은 로고
    soc = up(title.crop((0, 60, 300, 360)), 4)
    place(soc, up(wm, 6), 600, 230)
    soc.convert("RGB").save(OUT / "social-1200x1200.jpg", quality=92)


if __name__ == "__main__":
    main()
