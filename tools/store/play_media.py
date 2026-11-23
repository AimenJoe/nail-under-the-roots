"""Google Play 상점 그림(0295). Epic 그림(tools/store/epic_media.py)과 같은 원본·같은 규칙(정수배로만 키운다).

    python3 tools/store/epic_media.py && python3 tools/store/play_media.py      # docs/play/media/*

| 파일 | Play 규격 | 쓰는 곳 |
|---|---|---|
| icon-512.png | 512×512 PNG(32비트), ≤1MB | 기본 스토어 등록정보 → 앱 아이콘 — 앱 아이콘(tools/icons) 그대로 |
| feature-1024x500.png | 1024×500 PNG(24비트, 투명 없음) 또는 JPG, ≤15MB | 그래픽 이미지(대표 이미지) — 타이틀 그림 ×2를 잘라 로고 |
| screenshot-*.png | 16:9, 한 변 320~3840px, ≤8MB, 2~8장 | 휴대전화 스크린샷 — Epic 캐러셀과 같은 영어 화면 1920×1080 |
"""
import shutil
from pathlib import Path

from PIL import Image

from epic_media import ILL, OUT as EPIC, place, up, wordmark

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "play" / "media"


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    Image.open(EPIC / "product-icon-512.png").convert("RGBA").save(OUT / "icon-512.png", optimize=True)

    # 대표 이미지: 타이틀 그림(640×360) ×2 = 1280×720의 왼쪽 아래 1024×500 — 굴 속 잠든 Naïl과 뿌리가 다 들어오게.
    # 로고는 오른쪽 위 풀밭(가운데는 Play가 재생 단추를 얹을 수 있다).
    big = up(Image.open(ILL / "title.png").convert("RGBA"), 2)
    x0, y0 = 0, big.height - 500
    feat = big.crop((x0, y0, x0 + 1024, y0 + 500))
    place(feat, up(wordmark(), 4), 700, 105)
    feat.convert("RGB").save(OUT / "feature-1024x500.png", optimize=True)

    for f in sorted(EPIC.glob("carousel-*.png")):
        shutil.copyfile(f, OUT / f.name.replace("carousel-", "screenshot-"))


if __name__ == "__main__":
    main()
