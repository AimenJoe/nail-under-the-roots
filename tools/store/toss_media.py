"""앱인토스 콘솔 그림(0299·0300). 한국 손님이 보는 곳이라 한국어 화면(공식 페이지 스크린숏 site/img/s_*.png)과 한국어 글.
규칙은 Epic·Play 그림과 같다 — 게임 도트는 정수배로만 키우고, 글자는 게임 글꼴 Galmuri11을 원래 크기로 찍어 정수배.

    python3 tools/store/toss_media.py      # docs/toss/media/*

| 파일 | 콘솔 칸 | 규격 |
|---|---|---|
| logo-600.png | 앱 로고(다크모드 로고도 같은 것) | 600×600 — 앱 아이콘(tools/icons) |
| thumbnail-1932x828.png | 썸네일(게임 앱) | 1932×828 — 타이틀 그림 ×4를 잘라 한국어 이름 |
| landscape-*.png | 미리보기·스크린샷 가로형(최소 1장) | 1504×741 — 게임 화면 ×5, 아래 버튼 줄은 잘라 낸다(토스는 터치) |
| portrait-*.png | 미리보기·스크린샷 세로형(최소 3장) | 636×1048 — 위에 한 줄 글, 가운데 게임 화면 일부 ×3, 아래에 캐릭터 |
"""
import importlib.util
from pathlib import Path

from PIL import Image

from epic_media import ILL, place, text_layer, up

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "toss" / "media"
SHOTS = ROOT / "site" / "img"

CREAM = (236, 229, 218, 255)
GLOW = (242, 205, 92, 255)
EARTH = (38, 29, 24, 255)
EARTH_LIGHT = (62, 46, 36, 255)


def game(name: str) -> Image.Image:
    """공식 페이지 스크린숏(960×540 = 게임 320×180의 3배)을 게임 해상도로 되돌린다."""
    return Image.open(SHOTS / f"{name}.png").convert("RGBA").resize((320, 180), Image.NEAREST)


def icon(size: int) -> Image.Image:
    spec = importlib.util.spec_from_file_location("make_icons", ROOT / "tools" / "icons" / "make_icons.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m.render(m.art(), size)


# Galmuri11은 12px 격자(src/ui.rs) — 한글은 12의 배수 크기로, 굵게(1칸 겹치기)는 받침이 뭉개져 쓰지 않는다.
def wordmark_ko() -> Image.Image:
    big = text_layer("나일", 24, CREAM, bold=False)
    small = text_layer("나무뿌리 밑에서", 12, GLOW, bold=False)
    w = max(big.width, small.width)
    im = Image.new("RGBA", (w, big.height + small.height - 1), (0, 0, 0, 0))
    im.alpha_composite(big, ((w - big.width) // 2, 0))
    im.alpha_composite(small, ((w - small.width) // 2, big.height - 1))
    return im


# 세로형: (스크린숏, 위 글, 아래 글, 게임 화면에서 자를 왼쪽 x(게임 칸), 아래에 세울 캐릭터 띠)
PORTRAIT = [
    ("s_hall", "박자에 맞춰", "치고 받아치기", 70, "sp_nail"),
    ("s_meadow", "길 끝에 기다리는", "길의 주인", 54, "sp_bori"),
    ("s_village", "구한 이웃이 모이는", "큰 나무 아래 마을", 108, "sp_momo"),
]
LANDSCAPE = ["s_hall", "s_meadow", "s_village", "s_boss", "s_doto"]


def portrait(shot: str, top: str, bottom: str, x0: int, sprite: str) -> Image.Image:
    card = Image.new("RGBA", (636, 1048), EARTH)
    # 위: 두 줄 글(11px 글꼴 ×4)
    place(card, up(text_layer(top, 12, CREAM, bold=False), 4), 318, 115)
    place(card, up(text_layer(bottom, 12, GLOW, bold=False), 4), 318, 200)
    # 가운데: 게임 화면 212×164 칸 ×3 = 636×492(맨 아래 16칸 — 키보드 안내 줄 — 은 뺀다, 토스는 터치)
    crop = game(shot).crop((x0, 0, x0 + 212, 164))
    card.alpha_composite(up(crop, 3), (0, 290))
    # 아래: 걷는 띠의 첫 칸 ×6, 흙 띠 위에
    card.paste(EARTH_LIGHT, (0, 810, 636, 1048))
    strip = Image.open(SHOTS / f"{sprite}.png").convert("RGBA")
    cell = strip.height
    fig = up(strip.crop((0, 0, cell, cell)), 6)
    card.alpha_composite(fig, (318 - fig.width // 2, 1000 - fig.height))
    return card


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    icon(600).save(OUT / "logo-600.png", optimize=True)

    # 썸네일: 타이틀 그림(640×360) ×4 = 2560×1440에서 1932×828 — 굴 속 잠든 Naïl이 다 들어오게 아래쪽, 이름은 오른쪽 위 흙.
    big = up(Image.open(ILL / "title.png").convert("RGBA"), 4)
    thumb = big.crop((0, 1440 - 828, 1932, 1440))
    place(thumb, up(wordmark_ko(), 5), 1330, 230)
    thumb.convert("RGB").save(OUT / "thumbnail-1932x828.png", optimize=True)

    for i, name in enumerate(LANDSCAPE, 1):
        land = up(game(name), 5).crop((48, 0, 48 + 1504, 741))
        land.convert("RGB").save(OUT / f"landscape-{i}-{name[2:]}.png", optimize=True)

    for i, row in enumerate(PORTRAIT, 1):
        portrait(*row).convert("RGB").save(OUT / f"portrait-{i}-{row[0][2:]}.png", optimize=True)


if __name__ == "__main__":
    main()
