"""Microsoft Store 상점 그림(0301). 규칙은 Epic 그림과 같다 — 게임 도트는 정수배로만, 글자는 게임 글꼴 Galmuri11을 원래 크기로 찍어 정수배.
규격은 Microsoft 문서(2026-11-23): 게임 가이드(Game Publishing Guide "Configure Store listings for a game") — 상자 1:1 PNG 1080/2160,
슈퍼 히어로 16:9 글자 없이 1920×1080/3840×2160(3840 권장), 제목 넣은 히어로 1920×1080 PNG. 포스터는 Windows 앱 문서 "Attract customers"의
2:3(720×1080/1440×2160) — 게임 가이드는 9:16이라 적어 둘이 다르다(제출 화면이 묻는 비율로 고른다, 9:16도 만든다).

    python3 tools/store/msstore_media.py      # docs/msstore/media/*

| 파일 | 칸 |
|---|---|
| box-art-2160x2160.png | 상자 그림(1:1) — 제목은 위 2/3 안 |
| poster-1440x2160.png | 포스터 2:3 |
| poster-1440x2560.png | 포스터 9:16(게임 가이드) |
| hero-3840x2160.png | 슈퍼 히어로(글자 없음) |
| hero-titled-1920x1080.png | 제목 넣은 히어로 |
| screenshot-en-*.png | 스크린숏(영어 화면, Epic 캐러셀과 같은 것) |
| screenshot-ko-*.png | 스크린숏(한국어 화면, 공식 페이지 스크린숏 ×2) |
"""
import shutil
from pathlib import Path

from PIL import Image

from epic_media import ILL, OUT as EPIC_OUT, SHOTS, place, up, wordmark

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "msstore" / "media"
KO_SHOTS = ["s_meadow", "s_hall", "s_boss", "s_village", "s_doto"]


def fit(mark: Image.Image, w: int, h: int) -> Image.Image:
    """로고를 w×h 안에 드는 가장 큰 정수배로."""
    return up(mark, max(1, min(w // mark.width, h // mark.height)))


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    wm = wordmark()
    title = Image.open(ILL / "title.png").convert("RGBA")  # 640×360
    tree = Image.open(ILL / "great_tree.png").convert("RGBA")
    nail = Image.open(SHOTS / "sp_nail.png").convert("RGBA")
    nail = nail.crop((0, 0, nail.height, nail.height))   # 걷는 띠의 첫 칸(칸 = 띠 높이)

    # 상자 1:1: 타이틀 왼쪽 360×360(굴 속에 잠든 나일) ×6, 제목은 위 2/3 안(가운데 위).
    box = up(title.crop((0, 0, 360, 360)), 6)
    place(box, fit(wm, 1700, 520), 1080, 520)
    box.convert("RGB").save(OUT / "box-art-2160x2160.png", optimize=True)

    # 포스터: 큰 나무 그림(640×560) 가운데를 잘라 정수배, 나일은 왼쪽 풀밭, 제목은 위.
    # 2:3 = 240×360 ×6, 9:16 = 315×560 → 1440×2560이 정수배로 안 떨어져 288×512 ×5(1440×2560).
    for (w, h, k) in [(240, 360, 6), (288, 512, 5)]:
        x0 = (tree.width - w) // 2
        post = up(tree.crop((x0, 0, x0 + w, h)), k)
        post.alpha_composite(up(nail, 9), (200, post.height - 300 - nail.height * 9))
        place(post, fit(wm, 1300, 420), 720, 360)
        post.convert("RGB").save(OUT / f"poster-{w * k}x{h * k}.png", optimize=True)

    # 히어로: 타이틀 그림 그대로(글자 없음) ×6, 제목 넣은 것은 ×3 + 로고.
    up(title, 6).convert("RGB").save(OUT / "hero-3840x2160.png", optimize=True)
    hero = up(title, 3)
    place(hero, fit(wm, 900, 300), 960, 315)
    hero.convert("RGB").save(OUT / "hero-titled-1920x1080.png", optimize=True)

    # 스크린숏: 영어는 Epic 캐러셀 그대로(1920×1080), 한국어는 공식 페이지 스크린숏(960×540) ×2.
    for p in sorted(EPIC_OUT.glob("carousel-*.png")):
        shutil.copyfile(p, OUT / p.name.replace("carousel-", "screenshot-en-"))
    for i, name in enumerate(KO_SHOTS, 1):
        up(Image.open(SHOTS / f"{name}.png").convert("RGB"), 2).save(OUT / f"screenshot-ko-{i}-{name[2:]}.png", optimize=True)


if __name__ == "__main__":
    main()
