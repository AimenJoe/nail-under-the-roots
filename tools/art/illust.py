"""컷씬 일러스트·타이틀 바탕 만들기(docs/decisions/0039·0083·0125). PNG는 산출물이다.

실행: python3 tools/art/illust.py [출력 폴더] [장면 ...]  (기본 assets/art/illust)
장면 열둘과 타이틀 바탕은 모두 나노바나나 원본(tools/art/sources/ai/illust/<이름>.png)을 ai_illust.py가 게임 칸으로 만든다(0125~0137).
코드로 그리는 것은 타이틀 위에 흐르는 안개 `title_fog`뿐이다. 예전 코드 그림(v1·v2·v4, 파스텔·수채 바탕, 0039·0072·0073)은
tools/art/legacy/illust.py에 그대로 있다(0169 R8) — 원본이 없는 장면을 코드로 그리려면 그쪽을 돌린다.
사람이 그린 그림으로 바꿀 때는 같은 이름의 PNG를 넣고 AI_SCENES와 원본에서 빼야 다시 돌려도 덮이지 않는다(docs/art/cutscene-illustrations.md).

크기: 화면에 보이는 창 640×360(게임 320×180의 2배 밀도). 패닝하는 그림은 더 크다(src/illust.rs의 pan과 맞출 것, 표는 ai_illust.SIZE).
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ai_illust  # noqa: E402  (나노바나나 원본, 0125)


def vnoise(h, w, cell, seed, wrap_x=True):
    """값 잡음(부드러운 0..1). wrap_x면 가로로 이어진다(굴려도 이음매가 없다)."""
    r = np.random.default_rng(seed)
    gw = max(1, w // cell)
    gh = h // cell + 2
    g = r.random((gh, gw + (0 if wrap_x else 1)))
    y = np.arange(h) / cell
    x = np.arange(w) / cell
    y0 = np.floor(y).astype(int)
    x0 = np.floor(x).astype(int)
    fy = y - y0
    fx = x - x0
    fy = fy * fy * (3 - 2 * fy)
    fx = fx * fx * (3 - 2 * fx)
    xi0 = x0 % gw if wrap_x else x0
    xi1 = (x0 + 1) % gw if wrap_x else x0 + 1
    a = g[y0][:, xi0]
    b = g[y0][:, xi1]
    c = g[y0 + 1][:, xi0]
    d = g[y0 + 1][:, xi1]
    top = a + (b - a) * fx
    bot = c + (d - c) * fx
    return top + (bot - top) * fy[:, None]


def fbm(h, w, seed, cells=(120, 60, 30, 14), wrap_x=True):
    out = np.zeros((h, w), np.float32)
    amp, tot = 1.0, 0.0
    for k, c in enumerate(cells):
        out += vnoise(h, w, c, seed + k * 101, wrap_x) * amp
        tot += amp
        amp *= 0.5
    return out / tot


class _Rgba:
    def __init__(self, img):
        self.img = img

    def image(self):
        return self.img


def title_fog():
    """타이틀 위를 천천히 흐르는 안개(0083), 640×360 RGBA. 가로로 이어진다(게임이 옆으로 굴린다).
    땅 줄 둘레와 아래쪽에 짙고, 메뉴 한가운데는 옅다. 알파는 게임 픽셀(2×2) 단위로 계단을 둔다."""
    H, W = 360, 640
    yy = np.mgrid[0:H, 0:W][0].astype(np.float32)
    n = fbm(H, W, 41, cells=(160, 80, 40, 20))
    band = np.exp(-((yy - 112) / 40) ** 2) * 0.8 + np.clip((yy - 230) / 130, 0, 1) * 0.9
    a = np.clip((n - 0.5) * 2.6, 0, 1) * band
    a = (np.round(a * 6) / 6)                       # 알파 계단(부드러운 그라디언트 대신 도트 질감)
    a = a.reshape(H // 2, 2, W // 2, 2).mean((1, 3)).repeat(2, 0).repeat(2, 1)
    rgba = np.zeros((H, W, 4), np.uint8)
    rgba[..., 0], rgba[..., 1], rgba[..., 2] = 150, 126, 170
    rgba[..., 3] = np.clip(a * 80, 0, 255).astype(np.uint8)
    return _Rgba(Image.fromarray(rgba, 'RGBA'))


# 나노바나나 원본에서 만드는 장면(ai_illust.SIZE에 크기가 있어야 한다).
AI_SCENES = [
    'great_tree',
    'mother_burrow',
    'bori_freed',
    'momo_bloom',
    'halmae_deep',
    'momo_deep',
    'last_bloom',
    'sprout',
    'molehill',
    'doto_memory',
    'kongi_door',
    'clover_cocoon',
    'title',
]
# 코드로 그리는 것.
CODE_SCENES = {
    'title_fog': title_fog,
}


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else 'assets/art/illust'
    only = sys.argv[2:]
    os.makedirs(out, exist_ok=True)
    for name in AI_SCENES + list(CODE_SCENES):
        if only and name not in only:
            continue
        if name in CODE_SCENES:
            im = CODE_SCENES[name]().image()
        else:
            im = ai_illust.scene(name)
            if im is None:
                sys.exit(f'{name}: 나노바나나 원본이 없다(tools/art/sources/ai/illust/{name}.png). 코드 그림은 tools/art/legacy/illust.py')
        path = os.path.join(out, name + '.png')
        im.save(path, optimize=True)
        print(f'{path}: {im.size[0]}×{im.size[1]}, {os.path.getsize(path) // 1024}KB')


if __name__ == '__main__':
    main()
