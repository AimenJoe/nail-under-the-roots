"""Steam 업적 아이콘(0276 S2): 업적마다 64×64 두 장 — 열림(색)·잠김(회색). 게임에 이미 있는 그림만 쓴다
(컷씬 일러스트·초상화·스프라이트 — 모두 나노바나나 원본에서 만든 우리 그림, 타일 팩 그림은 쓰지 않는다).

실행: python3 tools/steam/achievement_icons.py        → docs/steam/icons/<API>.png·<API>_locked.png + 한눈에 보기 sheet.png
업적 표는 `src/achievements.rs` — 줄이 바뀌면 여기 `ICONS`도 바꾼다(`cargo test`가 아이콘이 다 있는지 본다).

- 그림: 일러스트·초상화는 게임 점의 2배로 저장돼 있어(초상화 88 = 44점, 일러스트 640 = 320점) 128×128을 잘라 반으로 줄이면
  점이 그대로 64×64에 맞는다(초상화는 64×64를 잘라 그대로 = 32점 × 2). 스프라이트는 정수 배로 키워 가운데에.
- 테두리: 바깥 1px 자두빛 선 + 2px 갈래 색(발자국 `journal::Cat::color`와 같은 색 — 이야기·싸움·탐험·이웃·마을).
- 잠김: 그림을 흑백으로 어둡게, 테두리도 회색 — Steam이 이루기 전에 보여 주는 칸.
"""
import os

from PIL import Image, ImageOps

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ART = os.path.join(ROOT, 'assets', 'art')
OUT = os.path.join(ROOT, 'docs', 'steam', 'icons')
S = 64
PLUM = (43, 29, 42)
# journal::Cat::color(0..1) × 255.
CAT = {
    'story': (230, 181, 120),
    'fight': (217, 115, 102),
    'explore': (140, 191, 115),
    'neighbors': (128, 179, 217),
    'village': (217, 140, 191),
}

# (API 이름, 갈래, 그림): ('illust', 파일, 가운데 x, y) — 파일 점 기준, 128×128을 잘라 반으로
#                       ('portrait', 칸 번호, 잘라 낼 왼쪽 위 x, y) — 88칸 안에서 64×64
#                       ('sprite', 파일, 칸 크기, 열, 줄, 배율)
ICONS = [
    ('TUTORIAL', 'story', ('illust', 'molehill.png', 320, 190)),
    ('BORI_BACK', 'story', ('illust', 'bori_freed.png', 300, 196)),
    ('MOMO_FIRST', 'story', ('portrait', 7, 12, 16)),
    ('MOMO_BACK', 'story', ('portrait', 13, 12, 10)),
    ('DOTO_MEMORY', 'story', ('illust', 'doto_memory.png', 330, 196)),
    ('SUSU_SPROUT', 'story', ('portrait', 34, 12, 12)),
    ('KONGI_BACK', 'story', ('illust', 'kongi_door.png', 236, 232)),
    ('DEEPEST', 'story', ('illust', 'halmae_deep.png', 400, 176)),
    ('CLOVER_FAMILY', 'story', ('illust', 'clover_cocoon.png', 350, 150)),
    ('ALL_BACK', 'story', ('illust', 'last_bloom.png', 300, 236)),
    ('TRUE_ENDING', 'story', ('illust', 'last_bloom.png', 250, 110)),
    ('LAST_MEMORY', 'story', ('illust', 'mem_dyed_thread.png', 326, 196)),
    ('SIDE_PATH', 'story', ('portrait', 41, 14, 12)),
    ('MUMU_FREED', 'story', ('portrait', 42, 14, 12)),
    ('MOMO_DEEP', 'story', ('illust', 'momo_deep.png', 590, 230)),
    ('DOTO_DEEP', 'story', ('portrait', 32, 12, 12)),
    ('KILLS', 'fight', ('portrait', 3, 12, 12)),
    ('AMBUSH', 'fight', ('sprite', 'dig.png', 32, 6, 0, 2)),
    ('PARRY', 'fight', ('sprite', 'nail.png', 28, 2, 0, 2)),
    ('HEAVY', 'fight', ('sprite', 'nail.png', 28, 7, 1, 2)),
    ('FINISHER', 'fight', ('portrait', 9, 12, 12)),
    ('BOWLING', 'fight', ('sprite', 'momo.png', 32, 5, 0, 2)),
    ('CHESTS', 'explore', ('sprite', 'props.png', 24, 5, 0, 2)),
    ('BREAKER', 'explore', ('sprite', 'props.png', 24, 1, 0, 2)),
    ('FALLS', 'explore', ('portrait', 2, 12, 12)),
    ('BESTIARY', 'neighbors', ('sprite', 'chipmunk.png', 24, 0, 0, 2)),
    ('TALENTS', 'neighbors', ('sprite', 'tree.png', 128, 2, 0, 1)),
    ('DREAM_ALL', 'village', ('portrait', 30, 12, 12)),
    ('DECOR', 'village', ('portrait', 6, 12, 12)),
    ('KNOTS', 'village', ('portrait', 16, 12, 12)),
]


def _bg(color):
    """갈래 색을 자두빛에 섞은 어두운 바탕."""
    return tuple(int(p * 0.62 + c * 0.38) for p, c in zip(PLUM, color)) + (255,)


def picture(src, color):
    kind = src[0]
    base = Image.new('RGBA', (S, S), _bg(color))
    if kind == 'illust':
        _, f, cx, cy = src
        im = Image.open(os.path.join(ART, 'illust', f)).convert('RGBA')
        x0 = max(0, min(im.width - 128, cx - 64))
        y0 = max(0, min(im.height - 128, cy - 64))
        crop = im.crop((x0, y0, x0 + 128, y0 + 128)).resize((S, S), Image.NEAREST)
        base.alpha_composite(crop)
    elif kind == 'portrait':
        _, cell, ox, oy = src
        sheet = Image.open(os.path.join(ART, 'portraits.png')).convert('RGBA')
        px, py = (cell % 12) * 88, (cell // 12) * 88
        crop = sheet.crop((px + ox, py + oy, px + ox + S, py + oy + S))
        base.alpha_composite(crop)
    else:
        _, f, cell, col, row, k = src
        sheet = Image.open(os.path.join(ART, f)).convert('RGBA')
        crop = sheet.crop((col * cell, row * cell, (col + 1) * cell, (row + 1) * cell))
        bb = crop.getbbox()
        if bb:
            crop = crop.crop(bb)
        k = min(k, S // max(crop.width, 1), S // max(crop.height, 1)) or 1
        crop = crop.resize((crop.width * k, crop.height * k), Image.NEAREST)
        base.alpha_composite(crop, ((S - crop.width) // 2, (S - crop.height) // 2 + 2))
    return base


def frame(im, color):
    out = im.copy()
    px = out.load()
    for y in range(S):
        for x in range(S):
            d = min(x, y, S - 1 - x, S - 1 - y)
            if d == 0:
                px[x, y] = PLUM + (255,)
            elif d <= 2:
                px[x, y] = color + (255,)
    return out


def locked(im):
    g = ImageOps.grayscale(im.convert('RGB'))
    g = g.point(lambda v: int(v * 0.5 + 12))
    return frame(Image.merge('RGBA', (g, g, g, Image.new('L', (S, S), 255))), (96, 90, 100))


def main():
    os.makedirs(OUT, exist_ok=True)
    tiles = []
    for api, cat, src in ICONS:
        pic = picture(src, CAT[cat])
        on = frame(pic, CAT[cat])
        off = locked(pic)
        on.convert('RGB').save(os.path.join(OUT, f'{api}.png'))
        off.convert('RGB').save(os.path.join(OUT, f'{api}_locked.png'))
        tiles.append((on, off))
    # 한눈에 보기: 열 여섯 칸씩 열림 줄 + 잠김 줄.
    cols = 10
    rows = (len(tiles) + cols - 1) // cols
    sheet = Image.new('RGB', (cols * (S + 4), rows * 2 * (S + 4)), (24, 18, 26))
    for i, (on, off) in enumerate(tiles):
        x, y = (i % cols) * (S + 4), (i // cols) * 2 * (S + 4)
        sheet.paste(on.convert('RGB'), (x, y))
        sheet.paste(off.convert('RGB'), (x, y + S + 4))
    sheet.save(os.path.join(OUT, 'sheet.png'))
    print(len(tiles), 'icons')


if __name__ == '__main__':
    main()
