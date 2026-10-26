"""앱 아이콘 만들기(iOS 0086, Android 0087, 그림 0123). 산출물(ios/icons/*.png, android/res/mipmap-*)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/icons/make_icons.py
- 그림(0123): 큰 나무와 보라 실이 얽힌 뿌리의 추상 실루엣. 64×64 격자에 코드로 찍는다(`art`).
  위는 금빛 원 앞의 나무 실루엣, 땅 줄 아래는 옅은 보랏빛 앞의 뿌리 실루엣, 실 고리 둘이 뿌리 다발을 묶는다
  (앞쪽 반은 뿌리 위로, 뒤쪽 반은 뿌리에 가려진다). 가장자리는 바탕색이라 런처가 어떤 모양으로 깎아도 이음매가 없다.
  예전에는 Naïl 초상화 얼굴을 썼다(0087).
- 크기마다: 64칸을 정수배(크기 이상)로 키운 뒤 매끈하게 줄인다(큰 크기는 도트가 그대로, 작은 크기는 뭉개지지 않게).
- iOS: 투명하면 안 된다. 60pt @2x·@3x(120·180), iPad 76pt @2x(152)·83.5pt @2x(167), 1024.
  Info.plist의 CFBundleIcons가 이 이름(AppIcon60x60 등)을 가리킨다(에셋 카탈로그 없이).
- Android: 옛 런처용 ic_launcher(48·72·96·144·192, mdpi~xxxhdpi)와 Android 8+ 적응형 아이콘
  (mipmap-anydpi-v26/ic_launcher.xml = 배경 + 앞 그림, 108dp 칸, 가운데 66dp만 늘 보인다).
  배경은 바탕색 한 장, 앞 그림은 그림 전체를 108dp 칸의 72dp에 둔다(원으로 깎아도 나무 꼭대기·뿌리 끝이 66dp 원 안).
  Cargo.toml [package.metadata.android]의 res = "android/res", icon = "@mipmap/ic_launcher".
"""
import math
import os

from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
IOS = {
    'AppIcon60x60@2x.png': 120,
    'AppIcon60x60@3x.png': 180,
    'AppIcon76x76@2x~ipad.png': 152,
    'AppIcon83.5x83.5@2x~ipad.png': 167,
    'AppIcon1024.png': 1024,
}
# 밀도: (폴더, 옛 아이콘 px, 적응형 칸 px = 108dp)
ANDROID = [('mdpi', 48, 108), ('hdpi', 72, 162), ('xhdpi', 96, 216), ('xxhdpi', 144, 324), ('xxxhdpi', 192, 432)]
ADAPTIVE_XML = """<?xml version="1.0" encoding="utf-8"?>
<!-- 적응형 아이콘(Android 8+, docs/decisions/0087). tools/icons/make_icons.py가 만든다. -->
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
"""


G = 64
BASE = (22, 14, 22)
GROUND = 37
# 앞 그림이 108dp 칸에서 차지할 크기(dp).
ADAPTIVE_ART_DP = 72


def art():
    img = Image.new('RGB', (G, G), BASE)
    px = img.load()
    for y in range(G):
        for x in range(G):
            if y < GROUND:
                d = math.hypot(x - 32, y - 22) / 26
                c = [(232, 196, 120), (206, 150, 82), (150, 92, 58), (88, 50, 44), BASE]
                k = 0 if d < 0.42 else 1 if d < 0.62 else 2 if d < 0.82 else 3 if d < 1.02 else 4
                px[x, y] = c[k]
            else:
                # 땅속: 뿌리 뒤로 아주 옅은 보랏빛(실의 빛이 흙에 번진 것). 뿌리가 실루엣으로 서게.
                d = math.hypot((x - 32) / 1.35, y - 46) / 20
                c = [(58, 36, 66), (44, 27, 50), (32, 20, 36), BASE]
                k = 0 if d < 0.45 else 1 if d < 0.75 else 2 if d < 1.0 else 3
                px[x, y] = c[k]
    d = ImageDraw.Draw(img)
    S = (14, 8, 16)
    d.rectangle((6, GROUND, 57, GROUND + 1), fill=S)
    d.polygon([(28, GROUND + 1), (36, GROUND + 1), (34, 24), (30, 24)], fill=S)
    for (cx, cy, r) in [(32, 14, 10), (23, 19, 7), (41, 19, 7), (27, 11, 6), (38, 11, 6), (32, 20, 7)]:
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=S)
    for (x, y) in [(16, 20), (48, 20), (19, 14), (45, 14), (32, 3)]:
        d.rectangle((x - 1, y - 1, x, y), fill=S)
    R = (12, 7, 14)
    roots = [
        [(32, 38), (32, 44), (31, 50), (32, 55), (31, 59)],
        [(30, 38), (25, 42), (19, 45), (13, 49), (9, 53)],
        [(34, 38), (39, 42), (45, 45), (51, 49), (55, 53)],
        [(31, 40), (27, 47), (23, 52), (20, 57)],
        [(33, 40), (37, 47), (41, 52), (44, 57)],
    ]
    widths = [3, 2, 2, 1]
    for pts in roots:
        for j in range(len(pts) - 1):
            d.line([pts[j], pts[j + 1]], fill=R, width=widths[min(j, 3)])
    # 보라 실: 뿌리 다발을 묶는 고리 둘. 앞쪽 반(아래 호)은 뿌리 위로 지나가고, 뒤쪽 반(위 호)은 뿌리에 가려진다.
    core, dim = (176, 124, 228), (104, 70, 148)
    for (cx, cy, rx, ry, tilt, ph) in ((32, 44, 9.5, 2.6, -0.10, 0.4), (32, 51, 15.5, 3.2, 0.08, 2.1)):
        seen = set()
        for i in range(720):
            t = i / 720 * 2 * math.pi
            wob = 1 + 0.07 * math.sin(3 * t + ph)   # 손으로 감은 듯 반지름이 조금 흔들린다
            x = cx + rx * wob * math.cos(t)
            y = cy + ry * wob * math.sin(t) + tilt * rx * math.cos(t) + 0.6 * math.sin(2 * t + ph)
            p = (round(x), round(y))
            if p in seen or not (0 <= p[0] < G and GROUND + 1 < p[1] < G):
                continue
            seen.add(p)
            front = math.sin(t) > 0
            if front:
                px[p] = core
            elif px[p] != R:
                px[p] = dim
    return img


def render(src, size):
    """64칸 그림을 size로: 크기 이상의 정수배로 키운 뒤(도트 그대로) 매끈하게 줄인다."""
    k = -(-size // src.width)
    big = src.resize((src.width * k, src.height * k), Image.NEAREST)
    return big if big.width == size else big.resize((size, size), Image.LANCZOS)


def ios(src):
    out = os.path.join(ROOT, 'ios', 'icons')
    os.makedirs(out, exist_ok=True)
    for name, size in IOS.items():
        render(src, size).save(os.path.join(out, name), optimize=True)
        print('ios', name, size)


def android(src):
    res = os.path.join(ROOT, 'android', 'res')
    for dens, legacy, cell in ANDROID:
        d = os.path.join(res, f'mipmap-{dens}')
        os.makedirs(d, exist_ok=True)
        # 옛 아이콘: 칸을 꽉 채운 네모(런처가 모양을 깎는다)
        render(src, legacy).save(os.path.join(d, 'ic_launcher.png'), optimize=True)
        # 적응형: 바탕색 한 장 + 그림(가운데 72dp)
        Image.new('RGB', (cell, cell), BASE).save(os.path.join(d, 'ic_launcher_background.png'), optimize=True)
        fg = Image.new('RGBA', (cell, cell), (0, 0, 0, 0))
        n = round(cell * ADAPTIVE_ART_DP / 108)
        fg.paste(render(src, n), ((cell - n) // 2, (cell - n) // 2))
        fg.save(os.path.join(d, 'ic_launcher_foreground.png'), optimize=True)
        print('android', dens, legacy, cell)
    d = os.path.join(res, 'mipmap-anydpi-v26')
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, 'ic_launcher.xml'), 'w') as f:
        f.write(ADAPTIVE_XML)


def main():
    src = art()
    ios(src)
    android(src)


if __name__ == '__main__':
    main()
