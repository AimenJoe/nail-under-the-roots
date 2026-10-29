"""그림 변환기 재현 검사(0230 R17): 나노바나나 원본 → 게임 시트 변환기를 모두 다시 돌려, 커밋된 PNG와 점 하나까지 같은지 본다.

    python3 tools/art/check_rebuild.py          # 다르면 시트·점 수를 적고 1로 끝난다
    python3 tools/art/check_rebuild.py --diff   # 다른 점의 자리(시트·칸·줄)까지

왜: 공용 변환기(sprite_ai.py)를 고치면 다른 캐릭터 시트가 조용히 바뀔 수 있다(도토 8점, 0230). 변환기를 고친 뒤,
원본을 바꾼 뒤, 시트를 커밋하기 전에 돌린다. 원본이 없는 시트(팩·옛 코드 그림)는 보지 않는다.
"""
import importlib
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ART = ROOT / "assets" / "art"
sys.path.insert(0, str(HERE))

# (모듈, build 인자, 시트 파일, 칸 크기). 한 원본에서 여러 장이면 줄마다.
SINGLE = [
    ("nail_ai", None, "nail.png", 24),
    ("momo_ai", None, "momo.png", 32),
    ("doto_ai", None, "doto.png", 40),
    ("kongi_ai", None, "kongi.png", 32),
    ("clover_ai", None, "clover.png", 40),
    ("bori_ai", None, "chief.png", 40),
    ("halmae_ai", None, "halmae.png", 40),
    ("props_ai", None, "props.png", 24),
    ("hub_ai", None, "hub.png", 48),
]


def targets():
    yield from SINGLE
    for mod, table in (("neighbors_ai", "SHEETS"), ("mobs_ai", "MOBS")):
        m = importlib.import_module(mod)
        for name in getattr(m, table):
            if (HERE / "sources" / "ai" / "sprites" / f"{name}_v1.png").exists():
                yield mod, name, f"{name}.png", None


def main() -> int:
    show = "--diff" in sys.argv
    bad = 0
    for mod, arg, file, cell in targets():
        m = importlib.import_module(mod)
        built = np.asarray(m.build() if arg is None else m.build(arg)).astype(int)
        sheets = [(file, built)]
        if mod == "nail_ai":
            # 털빛 셋(nail_fur*.png)은 같은 시트에서 색만 바꾼 것.
            sheets += [(f"nail_fur{n}.png", np.asarray(m.fur_variant(built.astype(np.uint8), n)).astype(int))
                       for n in m.legacy.FURS]
        for name, img in sheets:
            have = np.asarray(Image.open(ART / name).convert("RGBA")).astype(int)
            if have.shape != img.shape:
                print(f"다름 {name}: 크기 {have.shape[:2]} → {img.shape[:2]}")
                bad += 1
                continue
            diff = np.argwhere(np.abs(have - img).sum(-1) > 0)
            if len(diff):
                bad += 1
                print(f"다름 {name}: {len(diff)}점")
                if show:
                    c = cell or img.shape[0] // 4
                    for y, x in diff[:20]:
                        print(f"  ({x},{y}) 칸 {x // c} 줄 {y // c}: {have[y, x]} → {img[y, x]}")
            else:
                print(f"같음 {name}")
    print("재현됨" if not bad else f"{bad}장이 다르다")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
