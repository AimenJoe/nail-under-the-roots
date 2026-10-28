"""pxl — 점으로 찍는 도트 그림 도구. 도형 대신 픽셀 하나하나를 글자로 적는다.

.pxl 파일(텍스트, 레포에 두는 원본):
    # 아무 설명(무시)
    size 88 88
    palette
    . transparent
    o #2b1d2a  외곽선
    b #806050  털
    ...
    grid
    ....oooo....      ← 한 줄 = 한 픽셀 줄, 한 글자 = 한 픽셀(팔레트 글자)
    ...

명령:
    python3 tools/pixel/pxl.py import 그림.png 새.pxl [--colors 48]   PNG → .pxl (색이 많으면 줄인다)
    python3 tools/pixel/pxl.py render 그림.pxl 출력.png [--zoom 8 --grid]   PNG로(zoom이면 확대 미리보기 + 격자·좌표)
    python3 tools/pixel/pxl.py show 그림.pxl x0 y0 x1 y1               한 구역을 좌표 눈금과 함께 출력(고칠 곳 찾기)
    python3 tools/pixel/pxl.py check 그림.pxl                          줄 길이·모르는 글자·외톨이 픽셀·윤곽 계단·색 수
    python3 tools/pixel/pxl.py sheet 출력.png 열 a.pxl b.pxl ...       칸을 이어 시트로
    python3 tools/pixel/pxl.py put 그림.pxl x y 글자열                 (x, y)부터 오른쪽으로 덮어쓴다('_'는 건너뜀)
    python3 tools/pixel/pxl.py rekey 그림.pxl d=O:외곽선 u=4:털4 ...      글자를 한꺼번에 바꾸고 이름을 붙인다(가져온 뒤 읽기 쉽게)
    python3 tools/pixel/pxl.py pixelate 사진.jpg 새.pxl --size 320x180 [--colors 24 | --palette 그림.png] [--preview 미리보기.png]
        그림·사진을 도트로: 자르기 → 크기에 맞춘 중앙값 흐림(재질 무늬를 덩어리로) → 칸 평균으로 줄이기 → 색 줄이기(디더 없음)
        → 외톨이 정리. --palette면 그 그림의 색만 쓴다(게임 그림과 색을 맞출 때). 그 뒤는 손으로(show·put·editor.html).
        --cel: 4배에서 Kuwahara 단순화 → 면 나누기 → 다수결로 줄이기 → 큰 면 경계에만 1px 외곽선(--edge 밝기 차).
    python3 tools/pixel/pxl.py snap 시트.png 새.pxl --grid 4x2 [--cell 16 --colors 12 --bg ff00ff --palette 그림.png --preview 미리.png]
        이미지 모델이 그린 "도트" 시트(한 게임 픽셀 = 큰 블록, 단색 배경, 칸 격자)를 진짜 격자로(0203 C2):
        칸마다 블록 크기·어긋남(위상)을 재고 → 블록 가운데 색(가장자리 번짐·중간색을 피한다) → 배경은 투명 →
        시트 전체를 같은 팔레트로 묶는다(외톨이 정리는 하지 않는다: 16칸의 눈은 한 점이다). 끝에 한 줄 검사:
        칸마다 발바닥 줄·몸 키·팔레트 밖 색 — 칸끼리 다르면 알린다(같은 몸 크기·발 줄).
"""
import math
import argparse
import sys
from pathlib import Path

from PIL import Image, ImageDraw

KEYS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@$%&*+=?<>~^:;,'\"|/\\()[]{}-"


class Pxl:
    def __init__(self, w, h, palette, rows, notes=None):
        self.w, self.h = w, h
        self.palette = palette          # 글자 → (r, g, b, a), 이름
        self.rows = rows                # 문자열 목록
        self.notes = notes or []

    @staticmethod
    def load(path):
        lines = Path(path).read_text(encoding="utf-8").splitlines()
        w = h = None
        palette, rows, notes = {}, [], []
        mode = None
        for ln in lines:
            if mode != "grid" and (not ln.strip() or ln.lstrip().startswith("#")):
                if ln.startswith("#"):
                    notes.append(ln)
                continue
            if mode != "grid" and ln.startswith("size"):
                _, w, h = ln.split()
                w, h = int(w), int(h)
            elif mode != "grid" and ln.strip() == "palette":
                mode = "palette"
            elif mode != "grid" and ln.strip() == "grid":
                mode = "grid"
            elif mode == "palette":
                parts = ln.split(None, 2)
                key, val = parts[0], parts[1]
                name = parts[2] if len(parts) > 2 else ""
                if val == "transparent":
                    palette[key] = ((0, 0, 0, 0), name)
                else:
                    v = val.lstrip("#")
                    a = int(v[6:8], 16) if len(v) == 8 else 255
                    palette[key] = ((int(v[0:2], 16), int(v[2:4], 16), int(v[4:6], 16), a), name)
            elif mode == "grid":
                rows.append(ln.rstrip("\n"))
        if w is None:
            w, h = max(len(r) for r in rows), len(rows)
        return Pxl(w, h, palette, rows, notes)

    def save(self, path):
        out = list(self.notes) + [f"size {self.w} {self.h}", "palette"]
        for k, (c, name) in self.palette.items():
            if c[3] == 0:
                out.append(f"{k} transparent {name}".rstrip())
            else:
                hx = "#%02x%02x%02x" % c[:3] + ("%02x" % c[3] if c[3] != 255 else "")
                out.append(f"{k} {hx} {name}".rstrip())
        out.append("grid")
        out += self.rows
        Path(path).write_text("\n".join(out) + "\n", encoding="utf-8")

    def image(self):
        im = Image.new("RGBA", (self.w, self.h), (0, 0, 0, 0))
        px = im.load()
        for y, row in enumerate(self.rows[: self.h]):
            for x, k in enumerate(row[: self.w]):
                if k in self.palette:
                    px[x, y] = self.palette[k][0]
        return im


def cmd_import(a):
    im = Image.open(a.png).convert("RGBA")
    w, h = im.size
    colors = {c for c in (im.getpixel((x, y)) for y in range(h) for x in range(w)) if c[3] > 0}
    if len(colors) > a.colors:
        rgb = Image.new("RGB", im.size, (0, 0, 0))
        rgb.paste(im, mask=im.split()[3])
        q = rgb.quantize(colors=a.colors, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGB")
        alpha = im.split()[3]
        im = Image.merge("RGBA", (*q.split(), alpha))
    px = im.load()
    keymap, palette = {}, {".": ((0, 0, 0, 0), "투명")}
    # 어두운 색부터 글자를 준다(읽을 때 a = 가장 어두움).
    uniq = sorted({px[x, y] for y in range(h) for x in range(w) if px[x, y][3] > 0}, key=lambda c: sum(c[:3]))
    for i, c in enumerate(uniq):
        k = KEYS[i]
        keymap[c] = k
        palette[k] = (c if c[3] == 255 else c[:3] + (c[3],), "")
    rows = ["".join(keymap.get(px[x, y], ".") if px[x, y][3] > 0 else "." for x in range(w)) for y in range(h)]
    Pxl(w, h, palette, rows, [f"# {Path(a.png).name}에서 가져옴"]).save(a.pxl)
    print(f"{a.pxl}: {w}×{h}, 색 {len(uniq)}")


def cmd_render(a):
    p = Pxl.load(a.pxl)
    im = p.image()
    if a.zoom > 1:
        z = a.zoom
        bg = Image.new("RGBA", (p.w * z + 24, p.h * z + 24), (58, 50, 40, 255))
        big = im.resize((p.w * z, p.h * z), Image.NEAREST)
        bg.alpha_composite(big, (24, 24))
        d = ImageDraw.Draw(bg)
        if a.grid:
            for i in range(p.w + 1):
                col = (255, 255, 255, 70) if i % 8 == 0 else (0, 0, 0, 40)
                d.line([(24 + i * z, 24), (24 + i * z, 24 + p.h * z)], fill=col)
            for j in range(p.h + 1):
                col = (255, 255, 255, 70) if j % 8 == 0 else (0, 0, 0, 40)
                d.line([(24, 24 + j * z), (24 + p.w * z, 24 + j * z)], fill=col)
            for i in range(0, p.w, 8):
                d.text((24 + i * z + 1, 6), str(i), fill=(240, 230, 210, 255))
            for j in range(0, p.h, 8):
                d.text((2, 24 + j * z + 1), str(j), fill=(240, 230, 210, 255))
        im = bg
    im.save(a.png)
    print(a.png, im.size)


def cmd_show(a):
    p = Pxl.load(a.pxl)
    x0, y0, x1, y1 = a.x0, a.y0, min(a.x1, p.w - 1), min(a.y1, p.h - 1)
    tens = "".join(str((x // 10) % 10) for x in range(x0, x1 + 1))
    ones = "".join(str(x % 10) for x in range(x0, x1 + 1))
    print("     " + tens)
    print("     " + ones)
    for y in range(y0, y1 + 1):
        print(f"{y:>3}  " + p.rows[y][x0:x1 + 1])
    used = sorted({k for y in range(y0, y1 + 1) for k in p.rows[y][x0:x1 + 1]})
    for k in used:
        c, name = p.palette.get(k, ((0, 0, 0, 0), "?"))
        print(f"  {k} {'#%02x%02x%02x' % c[:3] if c[3] else 'transparent'} {name}")


def cmd_check(a):
    p = Pxl.load(a.pxl)
    bad = 0
    if len(p.rows) != p.h:
        print(f"줄 수 {len(p.rows)} ≠ 높이 {p.h}")
        bad += 1
    for y, r in enumerate(p.rows):
        if len(r) != p.w:
            print(f"{y}줄 길이 {len(r)} ≠ 폭 {p.w}")
            bad += 1
        for x, k in enumerate(r):
            if k not in p.palette:
                print(f"모르는 글자 {k!r} at ({x},{y})")
                bad += 1
    # 외톨이: 둘레 여덟 칸 어디에도 같은 글자가 없는 불투명 픽셀(의도한 반사·점이 아니면 소음)
    orphans = []
    for y in range(1, p.h - 1):
        for x in range(1, p.w - 1):
            k = p.rows[y][x]
            if k == "." or p.palette.get(k, ((0, 0, 0, 0),))[0][3] == 0:
                continue
            if all(p.rows[y + dy][x + dx] != k for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy):
                orphans.append((x, y, k))
    jag = jaggies(p)
    used = {k for r in p.rows for k in r}
    if jag:
        print(f"윤곽 계단이 들쭉날쭉한 줄(길이가 늘다 줄다 다시 는다): {jag[:16]}")
    print(f"{p.w}×{p.h}, 팔레트 {len(p.palette)}색 중 {len(used)}색 사용, 외톨이 {len(orphans)}개" + (f" (앞 12: {orphans[:12]})" if orphans else ""))
    return 1 if bad else 0


def jaggies(p):
    """실루엣 왼쪽·오른쪽 끝의 줄마다 이동량. 곡선은 이동량이 한 방향으로 점점 늘거나 줄어야 매끈하다.
    같은 방향으로 가면서 1, 3, 1이나 4, 3, 6처럼 튀는 줄을 돌려준다(볼록·오목 꼭대기는 제외)."""
    out = []
    for side in ("왼", "오른"):
        edge = []
        for y, r in enumerate(p.rows):
            xs = [x for x, k in enumerate(r) if k in p.palette and p.palette[k][0][3] > 0]
            edge.append((y, (xs[0] if side == "왼" else xs[-1]) if xs else None))
        d = [(edge[i + 1][0], edge[i + 1][1] - edge[i][1]) for i in range(len(edge) - 1)
             if edge[i][1] is not None and edge[i + 1][1] is not None]
        for i in range(1, len(d) - 1):
            a, b, c = d[i - 1][1], d[i][1], d[i + 1][1]
            if a * b > 0 and b * c > 0:
                a, b, c = abs(a), abs(b), abs(c)
                if (b > a and b > c and b - max(a, c) >= 1) or (b < a and b < c and min(a, c) - b >= 1):
                    out.append((side, d[i][0]))
    return out


def cmd_sheet(a):
    ims = [Pxl.load(f).image() for f in a.files]
    w, h = ims[0].size
    rows = (len(ims) + a.cols - 1) // a.cols
    out = Image.new("RGBA", (w * a.cols, h * rows), (0, 0, 0, 0))
    for i, im in enumerate(ims):
        out.paste(im, ((i % a.cols) * w, (i // a.cols) * h))
    out.save(a.png)
    print(a.png, out.size)


def cmd_put(a):
    p = Pxl.load(a.pxl)
    row = list(p.rows[a.y])
    for i, k in enumerate(a.text):
        if k != "_" and 0 <= a.x + i < p.w:
            row[a.x + i] = k
    p.rows[a.y] = "".join(row)
    p.save(a.pxl)


def cmd_rekey(a):
    p = Pxl.load(a.pxl)
    m, names = {}, {}
    for spec in a.specs:
        pair, _, name = spec.partition(":")
        old, new = pair.split("=")
        m[old] = new
        names[new] = name
    news = list(m.values())
    if len(set(news)) != len(news):
        sys.exit("새 글자가 겹친다")
    keep = {k for k in p.palette if k not in m}
    if keep & set(news):
        sys.exit(f"이미 쓰는 글자와 겹친다: {sorted(keep & set(news))}")
    pal = {}
    for k, (c, name) in p.palette.items():
        nk = m.get(k, k)
        pal[nk] = (c, names.get(nk) or name)
    p.palette = pal
    p.rows = ["".join(m.get(k, k) for k in r) for r in p.rows]
    p.save(a.pxl)


def kuwahara(img, r):
    """Kuwahara 필터(numpy): 네 사분면 중 밝기 분산이 가장 작은 쪽의 평균. 경계는 살리고 면 안은 평평해진다(붓 그림 흉내)."""
    import numpy as np
    h, w, _ = img.shape
    pad = np.pad(img, ((r, r), (r, r), (0, 0)), mode="edge")
    lum = pad @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    S = np.zeros((h + 2 * r + 1, w + 2 * r + 1, 3), np.float64); S[1:, 1:] = pad.cumsum(0).cumsum(1)
    L = np.zeros((h + 2 * r + 1, w + 2 * r + 1), np.float64); L[1:, 1:] = lum.cumsum(0).cumsum(1)
    L2 = np.zeros_like(L); L2[1:, 1:] = (lum * lum).cumsum(0).cumsum(1)
    n = (r + 1) ** 2
    best_var = np.full((h, w), np.inf); out = np.zeros((h, w, 3), np.float32)

    def box(T, y0, x0):        # 출력 (y,x)에 대해 pad 좌표 [y+y0, y+y0+r] × [x+x0, x+x0+r] 합
        ys, xs = np.arange(h)[:, None] + y0, np.arange(w)[None, :] + x0
        return T[ys + r + 1, xs + r + 1] - T[ys, xs + r + 1] - T[ys + r + 1, xs] + T[ys, xs]
    for y0, x0 in ((0, 0), (0, r), (r, 0), (r, r)):
        m = box(L, y0, x0) / n
        var = box(L2, y0, x0) / n - m * m
        mean = box(S, y0, x0) / n
        pick = var < best_var
        best_var = np.where(pick, var, best_var)
        out[pick] = mean[pick]
    return out


def cel_pixelate(src, tw, th, k, edge):
    """단순화 → 면 나누기 → 다수결로 줄이기 → 목표 해상도에서 1px 외곽선. (RGB 이미지, 외곽선 마스크)를 돌려준다."""
    import numpy as np
    f = 4
    big = np.asarray(src.resize((tw * f, th * f), Image.LANCZOS), dtype=np.float32)
    flat = kuwahara(kuwahara(big, 3), 2)
    q = Image.fromarray(np.clip(flat, 0, 255).astype(np.uint8)).quantize(colors=k, method=Image.Quantize.MEDIANCUT,
                                                                           dither=Image.Dither.NONE)
    lab = np.asarray(q)
    pal = np.array(q.getpalette()[: 3 * 256], dtype=np.float32).reshape(-1, 3)
    # 칸마다 가장 많은 면(평균이 아니라 다수결: 경계가 섞인 색을 만들지 않는다)
    blocks = lab.reshape(th, f, tw, f).transpose(0, 2, 1, 3).reshape(th, tw, f * f)
    counts = np.stack([(blocks == i).sum(-1) for i in range(k)], -1)
    small = counts.argmax(-1)
    # 작은 섬 합치기: 3×3 다수결 두 번(제 면이 셋 미만이면 둘레에 많은 면으로)
    for _ in range(2):
        pad = np.pad(small, 1, mode="edge")
        win = np.stack([pad[dy:dy + th, dx:dx + tw] for dy in range(3) for dx in range(3)], -1)
        cnt = np.stack([(win == i).sum(-1) for i in range(k)], -1)
        own = np.take_along_axis(cnt, small[..., None], -1)[..., 0]
        small = np.where(own >= 3, small, cnt.argmax(-1))
    # 덩어리 크기(4방향 이어진 같은 면): 큰 덩어리끼리의 경계에만 선을 긋는다(잔무늬 경계는 선 부스러기가 된다)
    comp = -np.ones((th, tw), np.int32)
    size = []
    for y in range(th):
        for x in range(tw):
            if comp[y, x] >= 0:
                continue
            cid, lv, stack, n = len(size), small[y, x], [(y, x)], 0
            comp[y, x] = cid
            while stack:
                cy, cx = stack.pop()
                n += 1
                for ny, nx in ((cy + 1, cx), (cy - 1, cx), (cy, cx + 1), (cy, cx - 1)):
                    if 0 <= ny < th and 0 <= nx < tw and comp[ny, nx] < 0 and small[ny, nx] == lv:
                        comp[ny, nx] = cid
                        stack.append((ny, nx))
            size.append(n)
    big_part = np.array(size)[comp] >= max(12, tw * th // 1500)
    rgb = pal[small]
    lum = rgb @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    # 외곽선: 오른쪽·아래 이웃과 면이 다르고 밝기 차가 edge 이상이면, 더 어두운 쪽 칸에 1px.
    line = np.zeros((th, tw), bool)
    for dy, dx in ((0, 1), (1, 0)):
        a_l, b_l = small[: th - dy, : tw - dx], small[dy:, dx:]
        a_y, b_y = lum[: th - dy, : tw - dx], lum[dy:, dx:]
        diff = (a_l != b_l) & (np.abs(a_y - b_y) >= edge) & big_part[: th - dy, : tw - dx] & big_part[dy:, dx:]
        dark_a = diff & (a_y <= b_y)
        dark_b = diff & (a_y > b_y)
        line[: th - dy, : tw - dx] |= dark_a
        line[dy:, dx:] |= dark_b
    # 홀로 떨어진 선 점 지우기(둘레 여덟 칸에 선이 없으면)
    lp = np.pad(line, 1)
    nb = sum(lp[1 + dy:1 + dy + th, 1 + dx:1 + dx + tw] for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dy or dx)
    line &= nb > 0
    # 선 색: 그 면 색을 어둡게 + 보라 쪽으로(검은 선 대신 선택 외곽선)
    shade = rgb * 0.5 + np.array([18, 8, 26], dtype=np.float32)
    rgb = np.where(line[..., None], shade, rgb)
    return Image.fromarray(np.clip(rgb + 0.5, 0, 255).astype(np.uint8)), line


def prepare(src, tw, th, margin=0.0, contrast=1.1, saturation=1.1):
    """도트화 앞 단계: 테두리 걷기 → 목표 비율로 가운데 자르기 → 대비·채도 → 한 칸 크기에 맞춘 중앙값 흐림."""
    from PIL import ImageEnhance, ImageFilter
    if margin > 0:      # 그림 테두리(액자·종이 가장자리) 걷어 내기
        mw, mh = int(src.width * margin), int(src.height * margin)
        src = src.crop((mw, mh, src.width - mw, src.height - mh))
    # 목표 비율로 가운데 자르기(cover)
    sw, sh = src.size
    k = max(tw / sw, th / sh)
    cw, ch = int(round(tw / k)), int(round(th / k))
    x0, y0 = (sw - cw) // 2, (sh - ch) // 2
    src = src.crop((x0, y0, x0 + cw, y0 + ch))
    if contrast != 1.0:
        src = ImageEnhance.Contrast(src).enhance(contrast)
    if saturation != 1.0:
        src = ImageEnhance.Color(src).enhance(saturation)
    # 한 칸이 원본 몇 px인지에 맞춘 중앙값 흐림: 잎·풀·털 같은 잔무늬가 덩어리로 뭉친다(바로 줄이면 칸마다 색이 튄다).
    cell = max(1, cw // tw)
    if cell >= 3:
        src = src.filter(ImageFilter.MedianFilter(min(9, cell | 1)))
    return src


def cmd_pixelate(a):
    tw, th = (int(v) for v in a.size.lower().split("x"))
    src = prepare(Image.open(a.src).convert("RGB"), tw, th, a.margin, a.contrast, a.saturation)
    small = src.resize((tw, th), Image.BOX)
    if a.cel:
        q, _ = cel_pixelate(src, tw, th, a.colors, a.edge)
    elif a.palette:
        pal_im = Image.open(a.palette).convert("RGB")
        cols = sorted({c for c in (pal_im.getpixel((x, y)) for y in range(pal_im.height) for x in range(pal_im.width))})
        if len(cols) > 256:
            cols = sorted({c for c in pal_im.quantize(256, method=Image.Quantize.MEDIANCUT).convert("RGB").getdata()})
        pim = Image.new("P", (1, 1))
        flat = [v for c in cols for v in c][:768]
        pim.putpalette(flat + [0] * (768 - len(flat)))
        q = small.quantize(palette=pim, dither=Image.Dither.NONE).convert("RGB")
    else:
        # 색을 넉넉히(4배) 뽑은 뒤 가까운 색끼리 합친다. 바로 N색으로 자르면 넓은 면(하늘·풀)이 색을 다 가져가서
        # 좁지만 뚜렷한 색(나무 기둥 갈색, 눈)이 사라진다.
        many = small.quantize(colors=min(256, a.colors * 4), method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
        counts = {i: n for n, i in many.getcolors(256)}
        pal = many.getpalette()[: 3 * 256]
        groups = {i: ([pal[3 * i], pal[3 * i + 1], pal[3 * i + 2]], counts[i]) for i in counts}

        def dist(c1, c2):          # redmean: 사람 눈에 가까운 RGB 거리
            r = (c1[0] + c2[0]) / 2
            dr, dg, db = c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2]
            return (2 + r / 256) * dr * dr + 4 * dg * dg + (2 + (255 - r) / 256) * db * db
        owner = {i: i for i in groups}
        while len(groups) > a.colors:
            ks = list(groups)
            best = min(((dist(groups[i][0], groups[j][0]), i, j) for n, i in enumerate(ks) for j in ks[n + 1:]))
            _, i, j = best
            (ci, ni), (cj, nj) = groups[i], groups[j]
            groups[i] = ([(ci[k] * ni + cj[k] * nj) / (ni + nj) for k in range(3)], ni + nj)
            del groups[j]
            for k, v in owner.items():
                if v == j:
                    owner[k] = i
        final = {i: tuple(int(round(v)) for v in groups[owner[i]][0]) for i in owner}
        q = Image.new("RGB", small.size)
        mp = many.load()
        qp = q.load()
        for y in range(th):
            for x in range(tw):
                qp[x, y] = final[mp[x, y]]
    px = q.load()
    # 외톨이 정리: 둘레 여덟 칸에 같은 색이 없으면 둘레에 가장 많은 색으로(두 번)
    for _ in range(0 if a.cel else 2):          # --cel은 면 나누기가 이미 덩어리라, 정리하면 1px 선 끝이 먹힌다
        changes = []
        for y in range(1, th - 1):
            for x in range(1, tw - 1):
                c = px[x, y]
                nb = [px[x + dx, y + dy] for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy]
                if c not in nb:
                    changes.append((x, y, max(set(nb), key=nb.count)))
        for x, y, c in changes:
            px[x, y] = c
    uniq = sorted({px[x, y] for y in range(th) for x in range(tw)}, key=sum)
    if len(uniq) > len(KEYS):
        sys.exit(f"색이 {len(uniq)}개라 글자가 모자란다. --colors를 줄일 것")
    keymap = {c: KEYS[i] for i, c in enumerate(uniq)}
    palette = {".": ((0, 0, 0, 0), "투명")}
    palette.update({keymap[c]: (c + (255,), "") for c in uniq})
    rows = ["".join(keymap[px[x, y]] for x in range(tw)) for y in range(th)]
    Pxl(tw, th, palette, rows, [f"# {Path(a.src).name}에서 도트로({tw}×{th}, 색 {len(uniq)})"]).save(a.pxl)
    if a.preview:
        z = max(1, 960 // tw)
        q.resize((tw * z, th * z), Image.NEAREST).save(a.preview)
    print(f"{a.pxl}: {tw}×{th}, 색 {len(uniq)}")


def merge_colors(cols_counts, n):
    """(색, 수) 목록을 가까운 것끼리 합쳐 n색으로(redmean 거리, 수로 가중 평균). → 원래 색 → 합친 색."""
    def dist(c1, c2):
        r = (c1[0] + c2[0]) / 2
        dr, dg, db = c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2]
        return (2 + r / 256) * dr * dr + 4 * dg * dg + (2 + (255 - r) / 256) * db * db
    groups = {i: (list(c), k) for i, (c, k) in enumerate(cols_counts)}
    owner = {i: i for i in groups}
    while len(groups) > n:
        ks = list(groups)
        _, i, j = min((dist(groups[i][0], groups[j][0]), i, j) for m, i in enumerate(ks) for j in ks[m + 1:])
        (ci, ni), (cj, nj) = groups[i], groups[j]
        groups[i] = ([(ci[k] * ni + cj[k] * nj) / (ni + nj) for k in range(3)], ni + nj)
        del groups[j]
        for k, v in owner.items():
            if v == j:
                owner[k] = i
    return {tuple(cols_counts[i][0]): tuple(int(round(v)) for v in groups[owner[i]][0]) for i in owner}


def snap_cell(im, box, nx, ny, bw, bh, bg, tol):
    """구역 하나(box)를 nx×ny 게임 픽셀로(None = 투명). 블록 = bw×bh(소수 가능), 위상은 `phase`.
    --grid면 구역 = 칸, 블록 = 칸 폭 / 16. 그림은 칸 가장자리에서 1px 띄울 것 — 칸 밖으로 밀린 블록은
    남은 조각만 보니 번짐에 진다."""
    x0, y0, x1, y1 = box
    px = im.load()

    def diff(a, b):
        return abs(a[0] - b[0]) + abs(a[1] - b[1]) + abs(a[2] - b[2])

    def phase(along_x):
        """블록 첫 점이 칸 앞에서 얼마나 밀렸나(0..size, 소수). 가장자리마다 (자리 mod size)를 원 위의 각으로
        보고 밝기 차이로 무게를 준 평균 — 번진 가장자리가 k-1·k·k+1로 나뉘어도 가운데가 잡히고,
        블록이 7.8px처럼 정수가 아니어도 된다."""
        size = bw if along_x else bh
        sx = sy = 0.0
        # 재는 쪽으로는 한 점도 건너뛰지 않는다(건너뛰면 짝수 자리 가장자리만 빠져 위상이 쏠린다).
        xstep, ystep = (1, 2) if along_x else (2, 1)
        for yy in range(y0 + 1, y1 - 1, ystep):
            for xx in range(x0 + 1, x1 - 1, xstep):
                a, b = px[xx, yy], (px[xx - 1, yy] if along_x else px[xx, yy - 1])
                d = diff(a, b)
                if d > 60:
                    t = 2 * math.pi * (((xx - x0) if along_x else (yy - y0)) % size) / size
                    sx += d * math.cos(t)
                    sy += d * math.sin(t)
        if sx == 0 and sy == 0:
            return 0.0
        off = (math.atan2(sy, sx) / (2 * math.pi) * size) % size
        # 끝에서 반 px 안쪽(7.8 / 8)이면 번져서 0이 넘어간 것 = 조금 당긴 것(−0.2). 아니면 몸이 한 블록 통째로 밀린다.
        return off - size if off > size - 0.5 else off

    ox, oy = phase(True), phase(False)

    def inside(start, length, lo, hi):
        return [v for v in range(int(start), max(int(start) + 1, int(start + length))) if lo <= v < hi]

    out = []
    for gy in range(ny):
        row = []
        for gx in range(nx):
            # 블록 가운데 반쪽만 본다(가장자리 번짐 피함). 위상만큼 민다. 칸 밖은 옆 칸 것이라 보지 않는다 —
            # 가운데 반쪽이 다 칸 밖이면 블록에서 칸 안에 남은 쪽을 본다.
            bx, by = x0 + ox + gx * bw, y0 + oy + gy * bh
            xs = inside(bx + bw * 0.25, bw * 0.5, x0, x1) or inside(bx, bw, x0, x1)
            ys = inside(by + bh * 0.25, bh * 0.5, y0, y1) or inside(by, bh, y0, y1)
            samp = [px[xx, yy] for yy in ys for xx in xs]
            fg = [c for c in samp if diff(c, bg) > tol]
            if not samp or len(fg) * 2 <= len(samp):
                row.append(None)
            else:
                fg.sort(key=sum)
                row.append(fg[len(fg) // 2])          # 밝기 가운데 값: 튀는 점 하나에 끌리지 않게
        out.append(row)
    return out


def block_size(im, boxes, lo=3.0, hi=40.0):
    """블록 크기(게임 픽셀 하나가 몇 px인가)를 잰다. 그림 덩어리마다 가장자리 자리를 원 위의 각(자리 / s)으로 보고
    얼마나 잘 모이나(0..1)를 덩어리끼리 더한다 — 덩어리마다 어긋남(위상)이 달라도 된다.
    s/2·s/3도 똑같이 모이고 2s부터는 흩어지므로 "잘 모이는 것 중 가장 큰 s"."""
    import numpy as np
    arr = np.asarray(im, dtype=np.int32)
    cands = np.arange(lo, hi, 0.02)
    total = np.zeros(len(cands))
    for (x0, y0, x1, y1) in boxes:
        sub = arr[y0:y1, x0:x1]
        pos, w = [], []
        for axis in (1, 0):
            d = np.abs(np.diff(sub, axis=axis)).sum(-1)
            ys, xs = np.nonzero(d > 60)
            pos.append((xs if axis == 1 else ys) + 1.0)
            w.append(d[ys, xs].astype(np.float64))
        pos, w = np.concatenate(pos), np.concatenate(w)
        if len(pos) < 8:
            continue
        step = max(1, len(pos) // 4000)
        pos, w = pos[::step], w[::step]
        for i in range(0, len(cands), 128):
            t = 2 * np.pi * pos[None, :] / cands[i:i + 128, None]
            total[i:i + 128] += np.hypot((w * np.cos(t)).sum(1), (w * np.sin(t)).sum(1)) / w.sum()
    if not total.any():
        sys.exit("가장자리가 없다 — 배경뿐인 그림?")
    k = np.nonzero(total >= 0.8 * total.max())[0].max()
    while k + 1 < len(total) and total[k + 1] > total[k]:    # 그 봉우리의 꼭대기로
        k += 1
    while k - 1 >= 0 and total[k - 1] > total[k]:
        k -= 1
    return float(cands[k])


def find_sprites(im, bg, tol, k=4):
    """배경이 아닌 점이 이어진 덩어리를 찾는다(k px로 줄인 지도에서 이웃 둘까지 — 2k px 틈은 잇는다).
    → [(x0, y0, x1, y1), …]. 블록을 재기 전이라 잡티는 아주 작은 것(4k² 미만)만 버린다."""
    import numpy as np
    arr = np.asarray(im, dtype=np.int32)
    fg = np.abs(arr - np.array(bg)).sum(-1) > tol
    H, W = fg.shape[0] // k, fg.shape[1] // k
    small = fg[:H * k, :W * k].reshape(H, k, W, k).any(axis=(1, 3))
    seen = np.zeros_like(small)
    boxes = []
    for sy, sx in zip(*np.nonzero(small)):
        if seen[sy, sx]:
            continue
        stack, cells = [(sy, sx)], []
        seen[sy, sx] = True
        while stack:
            y, x = stack.pop()
            cells.append((y, x))
            for dy in (-2, -1, 0, 1, 2):
                for dx in (-2, -1, 0, 1, 2):
                    yy, xx = y + dy, x + dx
                    if 0 <= yy < H and 0 <= xx < W and small[yy, xx] and not seen[yy, xx]:
                        seen[yy, xx] = True
                        stack.append((yy, xx))
        if len(cells) < 4:
            continue
        ys, xs = [c[0] for c in cells], [c[1] for c in cells]
        boxes.append((min(xs) * k, min(ys) * k, (max(xs) + 1) * k, (max(ys) + 1) * k))
    return boxes


def sprite_rows(boxes):
    """위아래가 겹치는 것끼리 한 줄, 줄마다 왼쪽부터."""
    rows = []
    for bx in sorted(boxes, key=lambda q: q[1]):
        for row in rows:
            top, bot = min(q[1] for q in row), max(q[3] for q in row)
            if bx[1] < bot and bx[3] > top:
                row.append(bx)
                break
        else:
            rows.append([bx])
    return [sorted(row, key=lambda q: q[0]) for row in rows]


def snap_found(im, bg, a):
    """--find: 블록 크기를 재고(또는 --block), 덩어리마다 격자를 맞춰 n×n 칸에 발바닥 줄(--foot, 기본 14)·가운데로 앉힌다."""
    n = a.cell
    boxes = find_sprites(im, bg, a.tol)
    b = a.block or block_size(im, boxes)
    foot = 14 if a.foot is None else a.foot
    # 블록 둘(가로·세로)이 안 되는 덩어리는 잡티 — 블록을 잰 다음에야 고를 수 있다.
    rows = sprite_rows([q for q in boxes if (q[2] - q[0]) >= 2 * b and (q[3] - q[1]) >= 2 * b])
    print(f"블록 {b:.2f}px, 그림 {sum(len(r) for r in rows)}개, 줄 {[len(r) for r in rows]}")
    out = []
    for row in rows:
        cells = []
        for (x0, y0, x1, y1) in row:
            ex0, ey0 = max(0, int(x0 - b)), max(0, int(y0 - b))
            ex1, ey1 = min(im.width, int(x1 + b)), min(im.height, int(y1 + b))
            g = snap_cell(im, (ex0, ey0, ex1, ey1), int((ex1 - ex0) / b) + 1, int((ey1 - ey0) / b) + 1, b, b, bg, a.tol)
            cells.append(seat(g, n, foot, (int(x0), int(y0))))
        out.append(cells)
    return out


def seat(g, n, foot, at):
    """그림이 있는 사각형만 떼어 n×n 칸에 발바닥 줄 foot·가운데로 앉힌다."""
    cell = [[None] * n for _ in range(n)]
    ys = [y for y, r in enumerate(g) if any(v is not None for v in r)]
    xs = [x for x in range(len(g[0])) if any(r[x] is not None for r in g)]
    if not ys:
        return cell
    h, w = ys[-1] - ys[0] + 1, xs[-1] - xs[0] + 1
    if h > foot + 1 or w > n:
        print(f"  {at} 그림이 {w}×{h}라 칸({n}, 발 {foot})을 넘는다 — 잘린다. 그림끼리 붙었거나(덩어리 하나) --block을 확인")
    top, left = foot - h + 1, (n - w) // 2
    for y in range(h):
        for x in range(w):
            if 0 <= top + y < n and 0 <= left + x < n:
                cell[top + y][left + x] = g[ys[0] + y][xs[0] + x]
    return cell


def cmd_snap(a):
    im = Image.open(a.src).convert("RGB")
    n = a.cell
    if a.bg:
        v = a.bg.lstrip("#")
        bg = (int(v[0:2], 16), int(v[2:4], 16), int(v[4:6], 16))
    else:
        corners = [im.getpixel((0, 0)), im.getpixel((im.width - 1, 0)), im.getpixel((0, im.height - 1)), im.getpixel((im.width - 1, im.height - 1))]
        bg = max(set(corners), key=corners.count)
    if a.find:
        found = snap_found(im, bg, a)
        cols, rows = max(len(r) for r in found), len(found)
        empty = [[None] * n for _ in range(n)]
        cells = [r[c] if c < len(r) else empty for r in found for c in range(cols)]
    else:
        if not a.grid:
            sys.exit("--grid 열x줄 이나 --find 가 필요하다")
        cols, rows = (int(v) for v in a.grid.lower().split("x"))
        cw, ch = im.width / cols, im.height / rows
        cells = []
        for r in range(rows):
            for c in range(cols):
                box = (int(c * cw), int(r * ch), int((c + 1) * cw), int((r + 1) * ch))
                cells.append(snap_cell(im, box, n, n, (box[2] - box[0]) / n, (box[3] - box[1]) / n, bg, a.tol))
    # 팔레트: --palette면 그 그림의 색에 가장 가까운 것으로, 아니면 시트 전체에서 --colors색으로 합친다.
    counts = {}
    for cell in cells:
        for row in cell:
            for col in row:
                if col is not None:
                    counts[col] = counts.get(col, 0) + 1
    if a.palette:
        pal_im = Image.open(a.palette).convert("RGBA")
        pal = sorted({pal_im.getpixel((x, y))[:3] for y in range(pal_im.height) for x in range(pal_im.width) if pal_im.getpixel((x, y))[3] > 0})

        def near(cc):
            return min(pal, key=lambda q: (q[0] - cc[0]) ** 2 + (q[1] - cc[1]) ** 2 + (q[2] - cc[2]) ** 2)
        mapping = {cc: near(cc) for cc in counts}
    else:
        mapping = merge_colors(list(counts.items()), a.colors) if counts else {}
    W, H = cols * n, rows * n
    grid = [[None] * W for _ in range(H)]
    for i, cell in enumerate(cells):
        r, c = divmod(i, cols)
        for gy in range(n):
            for gx in range(n):
                v = cell[gy][gx]
                grid[r * n + gy][c * n + gx] = None if v is None else mapping[v]
    uniq = sorted({v for row in grid for v in row if v is not None}, key=sum)
    if len(uniq) > len(KEYS):
        sys.exit(f"색이 {len(uniq)}개라 글자가 모자란다. --colors를 줄일 것")
    keymap = {cc: KEYS[i] for i, cc in enumerate(uniq)}
    palette = {".": ((0, 0, 0, 0), "투명")}
    palette.update({keymap[cc]: (cc + (255,), "") for cc in uniq})
    out_rows = ["".join("." if v is None else keymap[v] for v in row) for row in grid]
    notes = [f"# {Path(a.src).name}에서 격자 맞추기({cols}×{rows}칸, 칸 {n}, 색 {len(uniq)})"]
    Pxl(W, H, palette, out_rows, notes).save(a.pxl)
    if a.preview:
        z = max(1, 960 // W)
        Pxl.load(a.pxl).image().resize((W * z, H * z), Image.NEAREST).save(a.preview)
    # 한 줄 검사(0203): 칸마다 발바닥 줄(가장 아래 칠한 줄)·몸 키. 줄(동작)마다 칸끼리 같아야 한다.
    print(f"{a.pxl}: {W}×{H}, 색 {len(uniq)}, 배경 #{bg[0]:02x}{bg[1]:02x}{bg[2]:02x}")
    bad = 0
    for r in range(rows):
        feet, tall = [], []
        for c in range(cols):
            ys = [gy for gy in range(n) if any(grid[r * n + gy][c * n + gx] is not None for gx in range(n))]
            feet.append(max(ys) if ys else None)
            tall.append((max(ys) - min(ys) + 1) if ys else 0)
        fs = [f for f in feet if f is not None]
        ok_feet = (max(fs) - min(fs) <= a.foot_tol) if fs else True
        ok_tall = (max(tall) - min(t for t in tall if t) <= a.tall_tol) if any(tall) else True
        flag = "" if ok_feet and ok_tall else "  ← 칸마다 다르다"
        bad += 0 if flag == "" else 1
        print(f"  줄 {r}: 발바닥 {feet}  몸 키 {tall}{flag}")
    if a.foot is not None:
        off = [(i, f) for i, f in enumerate(
            max((gy for gy in range(n) if any(cell[gy][gx] is not None for gx in range(n))), default=None)
            for cell in cells) if f is not None and f != a.foot]
        if off:
            bad += 1
            print(f"  발바닥 줄이 {a.foot}가 아닌 칸: {off}")
    return 1 if (bad and a.strict) else 0


def main(argv=None):
    import signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)  # | head 로 잘라 읽을 때 조용히
    ap = argparse.ArgumentParser(description="점으로 찍는 도트 그림 도구")
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("import"); s.add_argument("png"); s.add_argument("pxl"); s.add_argument("--colors", type=int, default=48)
    s = sub.add_parser("render"); s.add_argument("pxl"); s.add_argument("png"); s.add_argument("--zoom", type=int, default=1); s.add_argument("--grid", action="store_true")
    s = sub.add_parser("show"); s.add_argument("pxl"); [s.add_argument(n, type=int) for n in ("x0", "y0", "x1", "y1")]
    s = sub.add_parser("check"); s.add_argument("pxl")
    s = sub.add_parser("sheet"); s.add_argument("png"); s.add_argument("cols", type=int); s.add_argument("files", nargs="+")
    s = sub.add_parser("put"); s.add_argument("pxl"); s.add_argument("x", type=int); s.add_argument("y", type=int); s.add_argument("text")
    s = sub.add_parser("rekey"); s.add_argument("pxl"); s.add_argument("specs", nargs="+")
    s = sub.add_parser("pixelate"); s.add_argument("src"); s.add_argument("pxl"); s.add_argument("--size", default="320x180")
    s.add_argument("--colors", type=int, default=24); s.add_argument("--palette"); s.add_argument("--preview")
    s.add_argument("--contrast", type=float, default=1.1); s.add_argument("--saturation", type=float, default=1.1)
    s.add_argument("--margin", type=float, default=0.0, help="가장자리를 이 비율만큼 먼저 잘라 낸다(예: 0.03)")
    s.add_argument("--cel", action="store_true", help="단순화(Kuwahara) → 면 나누기 → 다수결 축소 → 1px 외곽선")
    s.add_argument("--edge", type=float, default=30, help="--cel: 외곽선을 그을 면 사이 밝기 차(0~255)")
    s = sub.add_parser("snap"); s.add_argument("src"); s.add_argument("pxl"); s.add_argument("--grid", help="열x줄, 예: 4x2(시트를 똑같이 나눈다)")
    s.add_argument("--find", action="store_true", help="칸을 나누지 않고 그림 덩어리를 찾는다(격자가 고르지 않을 때)")
    s.add_argument("--block", type=float, help="--find: 블록 크기 px(없으면 잰다)")
    s.add_argument("--cell", type=int, default=16, help="칸 하나의 게임 픽셀 수")
    s.add_argument("--colors", type=int, default=12); s.add_argument("--palette"); s.add_argument("--preview")
    s.add_argument("--bg", help="배경색(예: ff00ff). 없으면 네 귀퉁이에서 고른다")
    s.add_argument("--tol", type=int, default=90, help="배경과 이만큼(RGB 합) 넘게 다르면 그림")
    s.add_argument("--foot", type=int, help="발바닥 줄이 이 줄이어야 한다(0203: 14). --find면 여기에 앉힌다(기본 14)")
    s.add_argument("--tall-tol", type=int, default=1, dest="tall_tol", help="한 줄 안 칸끼리 몸 키 차이 허용")
    s.add_argument("--foot-tol", type=int, default=1, dest="foot_tol", help="한 줄 안 칸끼리 발바닥 줄 차이 허용(걷기 들썩임)")
    s.add_argument("--strict", action="store_true", help="검사에 걸리면 1로 끝난다")
    a = ap.parse_args(argv)
    return {"import": cmd_import, "render": cmd_render, "show": cmd_show, "check": cmd_check,
            "sheet": cmd_sheet, "put": cmd_put, "rekey": cmd_rekey,
            "pixelate": cmd_pixelate, "snap": cmd_snap}[a.cmd](a) or 0


if __name__ == "__main__":
    sys.exit(main())
