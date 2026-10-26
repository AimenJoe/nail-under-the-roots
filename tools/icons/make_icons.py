"""앱 아이콘 만들기(iOS 0086, Android 0087). 산출물(ios/icons/*.png, android/res/mipmap-*)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/icons/make_icons.py
- 그림: 대화창 초상화 시트(assets/art/portraits.png, 88×88 칸)의 Naïl 기본 얼굴(줄 0, 칸 0)을 정수배로 키우고
  (도트가 고르게, 칸보다 크면 칸 크기로), 타이틀과 같은 어두운 흙빛 바탕 + 아래쪽 따뜻한 빛 위에 둔다.
- iOS: 투명하면 안 된다(검게 채운다). 60pt @2x·@3x(120·180), iPad 76pt @2x(152)·83.5pt @2x(167), 1024.
  Info.plist의 CFBundleIcons가 이 이름(AppIcon60x60 등)을 가리킨다(에셋 카탈로그 없이).
- Android: 옛 런처용 ic_launcher(48·72·96·144·192, mdpi~xxxhdpi)와 Android 8+ 적응형 아이콘
  (mipmap-anydpi-v26/ic_launcher.xml = 배경 + 앞 그림, 108dp 칸, 가운데 66dp만 늘 보인다 → 얼굴을 그 안에).
  Cargo.toml [package.metadata.android]의 res = "android/res", icon = "@mipmap/ic_launcher".
"""
import os

from PIL import Image

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


def backdrop(size):
    """어두운 바탕 + 아래에서 번지는 따뜻한 빛(계단 넷, 타이틀 굴 방 빛과 같은 색 계열)."""
    bg = Image.new('RGB', (size, size), (24, 16, 22))
    px = bg.load()
    cx, cy = size * 0.5, size * 0.78
    for y in range(size):
        for x in range(size):
            d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5 / size
            k = 0 if d > 0.62 else 1 if d > 0.46 else 2 if d > 0.32 else 3
            px[x, y] = [(24, 16, 22), (46, 28, 26), (78, 46, 32), (112, 68, 40)][k]
    return bg


def scaled_face(face, box):
    """정수배로 키운 얼굴(box 안에 들어가는 가장 큰 정수배, 안 들어가면 box 크기로)."""
    scale = max(1, int(box) // 88)
    if scale == 1 and box > 88 * 1.5:
        scale = 2
    if 88 * scale > box:
        return face.resize((int(box), int(box)), Image.NEAREST)
    return face.resize((88 * scale, 88 * scale), Image.NEAREST)


def ios(face):
    out = os.path.join(ROOT, 'ios', 'icons')
    os.makedirs(out, exist_ok=True)
    for name, size in IOS.items():
        img = backdrop(size)
        scale = max(1, -(-int(size * 0.9) // 88))       # 정수배, 칸보다 크면 칸 크기로
        n = 88 * scale if 88 * scale <= size else size
        f = face.resize((n, n), Image.NEAREST)
        img.paste(f, ((size - f.width) // 2, size - f.height), f)
        img.save(os.path.join(out, name), optimize=True)
        print('ios', name, size)


def android(face):
    res = os.path.join(ROOT, 'android', 'res')
    for dens, legacy, cell in ANDROID:
        d = os.path.join(res, f'mipmap-{dens}')
        os.makedirs(d, exist_ok=True)
        # 옛 아이콘: 칸을 꽉 채운 네모(런처가 모양을 깎는다)
        img = backdrop(legacy)
        f = face.resize((legacy, legacy), Image.NEAREST) if legacy < 88 * 2 else scaled_face(face, legacy)
        img.paste(f, ((legacy - f.width) // 2, legacy - f.height), f)
        img.save(os.path.join(d, 'ic_launcher.png'), optimize=True)
        # 적응형: 배경 한 장 + 얼굴(가운데 66dp 안전 구역에)
        backdrop(cell).save(os.path.join(d, 'ic_launcher_background.png'), optimize=True)
        fg = Image.new('RGBA', (cell, cell), (0, 0, 0, 0))
        safe = cell * 66 / 108
        f = scaled_face(face, safe)
        fg.paste(f, ((cell - f.width) // 2, int((cell + safe) / 2) - f.height), f)
        fg.save(os.path.join(d, 'ic_launcher_foreground.png'), optimize=True)
        print('android', dens, legacy, cell)
    d = os.path.join(res, 'mipmap-anydpi-v26')
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, 'ic_launcher.xml'), 'w') as f:
        f.write(ADAPTIVE_XML)


def main():
    sheet = Image.open(os.path.join(ROOT, 'assets', 'art', 'portraits.png')).convert('RGBA')
    face = sheet.crop((0, 0, 88, 88))
    ios(face)
    android(face)


if __name__ == '__main__':
    main()
