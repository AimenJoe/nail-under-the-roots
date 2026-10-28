"""`pxl.py snap` 왕복 시험: 우리 그림(nail.png 앞 4칸 × 2줄)을 이미지 모델이 그린 것처럼 망가뜨려
(블록 확대·어긋남·흐림·잡음·마젠타 배경) 다시 격자로 맞추고, 원본과 다른 점을 센다.
--grid는 칸을 똑같이 나눈 시트, --find는 그림마다 자리·간격이 제멋대로이고 블록이 소수(7.7px)인 시트.
남는 틀린 점은 거의 다 1px 눈(번짐에 섞여 아주 가까운 외곽선 색으로 간다) — 얼굴은 손질 단계에서 손으로(0203).

    python3 tools/pixel/test_snap.py
"""
import random
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageFilter

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import pxl  # noqa: E402

SRC = HERE.parent.parent / "assets" / "art" / "nail.png"
# (칸 px, 어긋남 px, 허용 틀린 점 / 2048). 칸 128 = 블록 8, 117·125·141 = 블록이 정수가 아닌 경우.
CASES = [(128, 0, 8), (128, 3, 8), (128, 5, 12), (125, 0, 24), (125, 3, 24), (133, 6, 16), (141, 3, 12)]


def ruin(ref, cell, shift, seed):
    bg = Image.new("RGBA", ref.size, (255, 0, 255, 255))
    bg.alpha_composite(ref)
    big = bg.convert("RGB").resize((4 * cell, 2 * cell), Image.NEAREST)
    canvas = Image.new("RGB", big.size, (255, 0, 255))
    canvas.paste(big, (shift, shift // 2))
    canvas = canvas.filter(ImageFilter.GaussianBlur(1.3))
    rnd = random.Random(seed)
    px = canvas.load()
    for y in range(canvas.height):
        for x in range(canvas.width):
            n = rnd.randint(-12, 12)
            px[x, y] = tuple(max(0, min(255, c + n)) for c in px[x, y])
    return canvas


def ruin_loose(ref, b, seed):
    """그림 여덟을 제멋대로 자리에(칸 격자 없음), 블록 b px(소수), 잡티 한 점."""
    rnd = random.Random(seed)
    W, H = 1100, 520
    canvas = Image.new("RGB", (W, H), (255, 0, 255))
    for i in range(8):
        c, r = i % 4, i // 4
        cell = ref.crop((c * 16, r * 16, c * 16 + 16, r * 16 + 16))
        tile = Image.new("RGBA", cell.size, (255, 0, 255, 255))
        tile.alpha_composite(cell)
        size = round(16 * b)
        big = tile.convert("RGB").resize((size, size), Image.NEAREST)
        canvas.paste(big, (40 + c * 260 + rnd.randint(-25, 25), 30 + r * 250 + rnd.randint(-20, 20)))
    canvas.paste((60, 40, 50), (W - 30, H - 30, W - 27, H - 27))      # 블록보다 작은 잡티
    canvas = canvas.filter(ImageFilter.GaussianBlur(1.3))
    px = canvas.load()
    for y in range(H):
        for x in range(W):
            n = rnd.randint(-12, 12)
            px[x, y] = tuple(max(0, min(255, v + n)) for v in px[x, y])
    return canvas


def crops(img, cols, rows):
    """칸마다 그림이 있는 사각형만 잘라 낸다(--find는 칸 안 자리를 다시 잡으니 모양만 견준다)."""
    out = []
    for r in range(rows):
        for c in range(cols):
            cell = img.crop((c * 16, r * 16, c * 16 + 16, r * 16 + 16))
            out.append(cell.crop(cell.getbbox()))
    return out


def wrong(a, b):
    pa, pb, bad = a.load(), b.load(), 0
    for y in range(a.height):
        for x in range(a.width):
            A, B = pa[x, y], pb[x, y]
            if (A[3] == 0) != (B[3] == 0) or (A[3] and sum(abs(A[i] - B[i]) for i in range(3)) > 30):
                bad += 1
    return bad


def main():
    ref = Image.open(SRC).convert("RGBA").crop((0, 0, 64, 32))
    fails = 0
    with tempfile.TemporaryDirectory() as d:
        d = Path(d)
        ref.save(d / "ref.png")
        for i, (cell, shift, limit) in enumerate(CASES):
            ruin(ref, cell, shift, i).save(d / "ai.png")
            pxl.main(["snap", str(d / "ai.png"), str(d / "out.pxl"), "--grid", "4x2", "--palette", str(d / "ref.png")])
            bad = wrong(ref, pxl.Pxl.load(d / "out.pxl").image())
            ok = bad <= limit
            fails += 0 if ok else 1
            print(f"칸 {cell} 어긋남 {shift}: 틀린 점 {bad} / 2048 (허용 {limit}) {'ok' if ok else '실패'}")
        for i, (b, limit) in enumerate([(7.7, 12), (8.0, 12), (9.3, 12)]):
            ruin_loose(ref, b, 10 + i).save(d / "loose.png")
            pxl.main(["snap", str(d / "loose.png"), str(d / "out.pxl"), "--find", "--palette", str(d / "ref.png")])
            got = pxl.Pxl.load(d / "out.pxl").image()
            bad = 0
            for want, have in zip(crops(ref, 4, 2), crops(got, 4, 2)):
                bad += wrong(want, have) if want.size == have.size else want.width * want.height
            ok = bad <= limit and got.size == ref.size
            fails += 0 if ok else 1
            print(f"찾기 블록 {b}: 틀린 점 {bad} (허용 {limit}) {'ok' if ok else '실패'}")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
