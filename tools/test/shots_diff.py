#!/usr/bin/env python3
"""스크린숏 두 폴더를 파일 이름으로 맞춰 견준다(0307 0단계). 파일마다 다른 픽셀 수와 비율, 끝에 합계.

    python3 tools/test/shots_diff.py <앞 폴더> <뒤 폴더> [--save <차이 그림 폴더>]

같은 빌드를 두 번 찍은 폴더끼리 먼저 돌려 잡음 바닥을 본다. 다른 파일이 있으면 종료 코드 1.
"""
import sys
from pathlib import Path

from PIL import Image, ImageChops


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    save = Path(sys.argv[sys.argv.index("--save") + 1]) if "--save" in sys.argv else None
    if save:
        args = [a for a in args if Path(a) != save]
        save.mkdir(parents=True, exist_ok=True)
    a, b = Path(args[0]), Path(args[1])
    names = sorted({p.name for p in a.glob("*.png")} | {p.name for p in b.glob("*.png")})
    changed = 0
    for n in names:
        pa, pb = a / n, b / n
        if not pa.exists() or not pb.exists():
            print(f"{n}: 한쪽에만 있다")
            changed += 1
            continue
        ia, ib = Image.open(pa).convert("RGB"), Image.open(pb).convert("RGB")
        if ia.size != ib.size:
            print(f"{n}: 크기가 다르다 {ia.size} / {ib.size}")
            changed += 1
            continue
        diff = ImageChops.difference(ia, ib).convert("L").point(lambda v: 255 if v else 0)
        px = sum(1 for v in diff.getdata() if v)
        if px:
            changed += 1
            box = diff.getbbox()
            print(f"{n}: {px}px 다름({px / (ia.size[0] * ia.size[1]):.3%}) 자리 {box}")
            if save:
                diff.save(save / n)
        else:
            print(f"{n}: 같다")
    print(f"-- {len(names)}장 가운데 {changed}장이 다르다")
    return 1 if changed else 0


if __name__ == "__main__":
    sys.exit(main())
