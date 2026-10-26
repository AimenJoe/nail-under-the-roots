"""나노바나나 일러스트용 참고 그림 묶음 만들기(docs/art/nano-banana-prompts.md 3, docs/decisions/0124).

실행: python3 tools/art/ref_pack.py <출력 폴더>
- 캐릭터 시트(tools/art/sources/ai/sheets/<이름>.png, 사장이 0-1 단계로 그린 것)를 장면 첨부용으로 다듬는다:
  글자 띠(제목·"FRONT VIEW")를 잘라 내고(첨부 그림에 글자가 있으면 장면에도 글자가 들어가기 쉽다), 오른쪽 아래 ✦(Gemini 로고)를 지운다.
- 시트가 없는 모습(물든 넷·엄마)은 초상화 원본(tools/art/sources/ai/*.png)을 쓴다.
- overview.png: 한 장에 모두 모아 이름을 붙인 확인용 그림(첨부용이 아니다).
원본은 고치지 않는다. 같은 원본 = 같은 묶음.
"""
import os
import sys

from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.abspath(__file__))
AI = os.path.join(ROOT, 'sources', 'ai')
SHEETS = os.path.join(AI, 'sheets')

# 시트: 이름 → 남길 줄 범위(위, 아래). None이면 전부. 글자 띠는 줄마다 어두운 점 수로 찾았다.
SHEET_ROWS = {
    'nail': (40, 256),    # 위 "FRONT VIEW / THREE-QUARTER VIEW", 아래 "CHARACTER REFERENCE SHEET: NAÏL"
    'popo': (40, 256),
    'momo': (0, 534),     # 아래 "FRONT VIEW / THREE-QUARTER VIEW"
    'bori': None,
    'halmae': None,
    'doto': None,
    'susu': None,
    'kongi': None,
}
# 시트가 없는 모습: 묶음 이름 → 초상화 원본
PORTRAITS = {
    'mom': 'mom_gentle',
    'momo_tainted': 'momo_tainted_normal',
    'bori_tainted': 'chief_glare',
    'doto_tainted': 'doto_tainted_glare',
    'kongi_tainted': 'kongi_tainted_glare',
}


# ✦가 캐릭터 위에 겹쳐 배경색으로 못 찾는 시트: 이름 → (가운데 x, y, 반지름), 원본 px. 네 꼭짓점 별 모양 자리를 둘레 색으로 메운다.
STAR_AT = {'susu': (935, 483, 18)}


def erase_star_at(img, cx, cy, r):
    px = img.load()
    hole = set()
    for y in range(int(cy - r) - 1, int(cy + r) + 2):
        for x in range(int(cx - r) - 1, int(cx + r) + 2):
            dx, dy = abs(x - cx) / r, abs(y - cy) / r
            if dx ** (2 / 3) + dy ** (2 / 3) <= 1.15:
                hole.add((x, y))
    fill_hole(img, hole)


def fill_hole(img, hole):
    """구멍을 둘레에서 가장 흔한 색으로 바깥부터 안쪽으로 메운다."""
    w, h = img.size
    px = img.load()
    while hole:
        filled = []
        for (x, y) in hole:
            near = [px[x + i, y + j] for i in (-1, 0, 1) for j in (-1, 0, 1)
                    if (i or j) and 0 <= x + i < w and 0 <= y + j < h and (x + i, y + j) not in hole]
            if len(near) >= 3:
                filled.append(((x, y), max(set(near), key=near.count)))
        if not filled:
            break
        for (p, c) in filled:
            px[p] = c
            hole.discard(p)


def erase_star(img):
    """오른쪽 아래의 ✦: 그 자리 배경색(가장 흔한 색)보다 밝은 점 덩어리를 찾아 둘레에서 가장 흔한 색으로 안쪽부터 메운다.
    ✦는 반투명이라 배경보다 조금만 밝다(배경 (230,223,204)에 별 (239,234,221)). "아주 밝은 흰색"으로 찾으면 놓쳤다."""
    w, h = img.size
    px = img.load()
    box = (int(w * 0.80), int(h * 0.70), w, h)
    region = [px[x, y] for y in range(box[1], box[3]) for x in range(box[0], box[2])]
    bg = max(set(region), key=region.count)
    pts = [(x, y) for y in range(box[1], box[3]) for x in range(box[0], box[2])
           if sum(px[x, y]) > sum(bg) + 12 and all(px[x, y][i] >= bg[i] - 2 for i in range(3))]
    if not pts:
        return
    # 가장 큰 덩어리만(배경의 다른 밝은 점이 섞이지 않게)
    left, comps = set(pts), []
    while left:
        stack, comp = [left.pop()], set()
        while stack:
            p = stack.pop()
            comp.add(p)
            for q in ((p[0] + 1, p[1]), (p[0] - 1, p[1]), (p[0], p[1] + 1), (p[0], p[1] - 1)):
                if q in left:
                    left.discard(q)
                    stack.append(q)
        comps.append(comp)
    star = max(comps, key=len)
    hole = {(x + i, y + j) for (x, y) in star for i in (-2, -1, 0, 1, 2) for j in (-2, -1, 0, 1, 2)
            if 0 <= x + i < w and 0 <= y + j < h}
    fill_hole(img, hole)


def pack(out):
    os.makedirs(out, exist_ok=True)
    made = []
    for name, rows in SHEET_ROWS.items():
        img = Image.open(os.path.join(SHEETS, f'{name}.png')).convert('RGB')
        if name in STAR_AT:
            erase_star_at(img, *STAR_AT[name])
        else:
            erase_star(img)
        if rows:
            img = img.crop((0, rows[0], img.width, rows[1]))
        path = os.path.join(out, f'sheet_{name}.png')
        img.save(path, optimize=True)
        made.append((f'sheet_{name}', img))
    for name, src in PORTRAITS.items():
        img = Image.open(os.path.join(AI, f'{src}.png')).convert('RGB')
        erase_star(img)
        path = os.path.join(out, f'ref_{name}.png')
        img.save(path, optimize=True)
        made.append((f'ref_{name}', img))
    # 확인용 한 장: 칸마다 그림 + 이름
    cw, ch = 400, 240
    cols = 4
    rows_n = -(-len(made) // cols)
    sheet = Image.new('RGB', (cw * cols, (ch + 22) * rows_n), (250, 248, 242))
    d = ImageDraw.Draw(sheet)
    for i, (label, img) in enumerate(made):
        t = img.copy()
        t.thumbnail((cw - 8, ch - 8))
        x, y = (i % cols) * cw, (i // cols) * (ch + 22)
        sheet.paste(t, (x + (cw - t.width) // 2, y + (ch - t.height) // 2))
        d.text((x + 8, y + ch + 4), label + '.png', fill=(40, 30, 40))
    sheet.save(os.path.join(out, 'overview.png'), optimize=True)
    return [m[0] for m in made]


if __name__ == '__main__':
    print(pack(sys.argv[1] if len(sys.argv) > 1 else 'ref_pack'))
