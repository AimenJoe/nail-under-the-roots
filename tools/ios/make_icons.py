"""iOS 앱 아이콘 만들기(docs/decisions/0086). 산출물(ios/icons/*.png)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/ios/make_icons.py
- 그림: 대화창 초상화 시트(assets/art/portraits.png, 88×88 칸)의 Naïl 기본 얼굴(줄 0, 칸 0)을 가장 가까운 점으로 키우고,
  타이틀과 같은 어두운 흙빛 바탕 + 아래쪽 따뜻한 빛 한 겹 위에 둔다. 아이콘은 투명하면 안 된다(iOS가 검게 채운다).
- 크기: iPhone 60pt @2x·@3x(120·180), iPad 76pt @2x(152)·83.5pt @2x(167), App Store용 1024.
  Info.plist의 CFBundleIcons가 이 파일 이름(AppIcon60x60 등)을 가리킨다(에셋 카탈로그 없이).
"""
import os

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, 'ios', 'icons')
SIZES = {
    'AppIcon60x60@2x.png': 120,
    'AppIcon60x60@3x.png': 180,
    'AppIcon76x76@2x~ipad.png': 152,
    'AppIcon83.5x83.5@2x~ipad.png': 167,
    'AppIcon1024.png': 1024,
}


def base(size):
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


def main():
    os.makedirs(OUT, exist_ok=True)
    sheet = Image.open(os.path.join(ROOT, 'assets', 'art', 'portraits.png')).convert('RGBA')
    face = sheet.crop((0, 0, 88, 88))
    for name, size in SIZES.items():
        img = base(size)
        scale = max(1, -(-int(size * 0.9) // 88))       # 정수배(도트가 고르게), 칸보다 크면 칸 크기로
        f = face.resize((88 * scale, 88 * scale), Image.NEAREST)
        if f.width > size:
            f = face.resize((size, size), Image.NEAREST)
        img.paste(f, ((size - f.width) // 2, size - f.height), f)
        img.save(os.path.join(OUT, name), optimize=True)
        print(name, size, f'×{scale}')


if __name__ == '__main__':
    main()
