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
    python3 tools/pixel/pxl.py check 그림.pxl                          줄 길이·모르는 글자·외톨이 픽셀·색 수
    python3 tools/pixel/pxl.py sheet 출력.png 열 a.pxl b.pxl ...       칸을 이어 시트로
    python3 tools/pixel/pxl.py put 그림.pxl x y 글자열                 (x, y)부터 오른쪽으로 덮어쓴다('_'는 건너뜀)
    python3 tools/pixel/pxl.py rekey 그림.pxl d=O:외곽선 u=4:털4 ...      글자를 한꺼번에 바꾸고 이름을 붙인다(가져온 뒤 읽기 쉽게)
"""
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
    a = ap.parse_args(argv)
    return {"import": cmd_import, "render": cmd_render, "show": cmd_show, "check": cmd_check,
            "sheet": cmd_sheet, "put": cmd_put, "rekey": cmd_rekey}[a.cmd](a) or 0


if __name__ == "__main__":
    sys.exit(main())
