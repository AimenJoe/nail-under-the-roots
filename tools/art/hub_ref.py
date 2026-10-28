"""마을·쉼터 새 그림(0224): 나노바나나에 붙일 크기·생김새 기준 그림.

실행: python3 tools/art/hub_ref.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_hub_a.png: 쉼터·마을 공용 물건 여섯(가게 좌판·모닥불 화덕·그루터기·바위·통나무 자리·뿌리 길 입구)
  - ref_hub_b.png: 마을 꾸미기 여섯(꽃밭·반딧불 등·빛나는 버섯·통나무 의자·바람종·깃발 줄)
  - ref_hub_c.png: 마을 꾸미기 다섯(반딧불 병·들꽃 화분·실 그네·물웅덩이·실 고리)
  게임 크기 Naïl(24칸) 옆에 목표 크기로 칠한 색 안내, 마젠타 바탕 8배.

왜 도형으로: 지금 이 물건들 대부분이 Sunnyside 팩 그림이라 팩 그림은 생성 참고로 쓸 수 없다(0064). 크기·색·자리만 주고
생김새는 프롬프트 글로 준다(`docs/art/hub-sprite-prompts.md`). 소품(0222)과 같은 길.
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
Z = 8
OUTLINE = (43, 29, 42)
FOOT = 46          # 물건 칸(48) 안 발 줄


class Pad:
    """물건 하나를 그릴 판: 논리 좌표(게임 점)로 그리면 8배로 찍는다."""

    def __init__(self, w, h):
        self.im = Image.new("RGBA", (w * Z, h * Z), (0, 0, 0, 0))
        self.d = ImageDraw.Draw(self.im)

    def ell(self, cx, cy, rx, ry, c, line=True):
        if line:
            self.d.ellipse([(cx - rx - 1) * Z, (cy - ry - 1) * Z, (cx + rx + 1) * Z, (cy + ry + 1) * Z], fill=OUTLINE)
        self.d.ellipse([(cx - rx) * Z, (cy - ry) * Z, (cx + rx) * Z, (cy + ry) * Z], fill=c)

    def box(self, x0, y0, x1, y1, c, line=True):
        if line:
            self.d.rectangle([(x0 - 1) * Z, (y0 - 1) * Z, (x1 + 1) * Z - 1, (y1 + 1) * Z - 1], fill=OUTLINE)
        self.d.rectangle([x0 * Z, y0 * Z, x1 * Z - 1, y1 * Z - 1], fill=c)

    def line(self, a, b, c, w=1):
        self.d.line([(a[0] * Z, a[1] * Z), (b[0] * Z, b[1] * Z)], fill=c, width=w * Z)

    def poly(self, pts, c):
        self.d.polygon([(x * Z, y * Z) for x, y in pts], fill=c)


WOOD, WOOD_D, WOOD_L = (140, 92, 56), (92, 56, 36), (184, 132, 84)
STONE, STONE_D, STONE_L = (120, 112, 124), (84, 78, 92), (170, 164, 172)
LEAF, LEAF_D = (92, 160, 72), (56, 112, 56)
RED, CREAM = (196, 64, 64), (240, 222, 186)
SILK = (236, 232, 248)


# --- A: 공용 물건 ---------------------------------------------------------------------------------------------------
def stall(p, cx):
    """포포 가게 좌판: 줄무늬 차양 + 나무 진열대(산딸기 바구니·씨앗 자루). 폭 30 키 36."""
    x0, x1 = cx - 15, cx + 15
    for x in (x0 + 1, x1 - 3):
        p.box(x, FOOT - 34, x + 2, FOOT, WOOD_D)
    for i in range(6):
        p.box(x0 + i * 5, FOOT - 36, x0 + i * 5 + 5, FOOT - 29, RED if i % 2 == 0 else CREAM, line=False)
    p.d.rectangle([x0 * Z, (FOOT - 37) * Z, x1 * Z, (FOOT - 36) * Z], fill=OUTLINE)
    p.box(x0, FOOT - 14, x1, FOOT - 1, WOOD)
    p.box(x0, FOOT - 14, x1, FOOT - 11, WOOD_L)
    p.ell(cx - 7, FOOT - 16, 4, 2, (176, 52, 72))
    p.ell(cx + 6, FOOT - 17, 3, 3, (206, 180, 120))


def firepit(p, cx):
    """모닥불 화덕: 돌 아홉 고리 + 가운데 장작 둘(불꽃은 게임이 그린다). 폭 24 키 14."""
    import math
    for k in range(9):
        a = k / 9 * math.tau
        p.ell(cx + math.cos(a) * 10, FOOT - 7 + math.sin(a) * 5, 2.5, 2, STONE_L if math.sin(a) < 0 else STONE)
    p.line((cx - 5, FOOT - 6), (cx + 5, FOOT - 8), WOOD_D, 2)
    p.line((cx - 5, FOOT - 8), (cx + 5, FOOT - 6), WOOD, 2)


def stump(p, cx):
    """그루터기: 나이테 보이는 윗면 + 뿌리 셋. 폭 16 키 14."""
    p.box(cx - 7, FOOT - 10, cx + 7, FOOT - 1, WOOD)
    p.ell(cx, FOOT - 10, 7, 3, WOOD_L)
    p.ell(cx, FOOT - 10, 3, 1.4, WOOD, line=False)
    for dx in (-8, 8):
        p.line((cx + dx * 0.8, FOOT - 3), (cx + dx, FOOT - 1), WOOD_D, 2)


def boulder(p, cx):
    """바위: 둥근 큰 돌 + 이끼 조금. 폭 22 키 18."""
    p.ell(cx, FOOT - 9, 11, 8, STONE)
    p.ell(cx - 3, FOOT - 12, 6, 4, STONE_L, line=False)
    p.ell(cx + 5, FOOT - 15, 3, 1.5, LEAF, line=False)


def log_seat(p, cx):
    """통나무 자리(빈 자리): 가로로 누운 짧은 통나무. 폭 20 키 12."""
    p.box(cx - 9, FOOT - 10, cx + 7, FOOT - 1, WOOD)
    p.box(cx - 9, FOOT - 10, cx + 7, FOOT - 8, WOOD_L)
    p.ell(cx + 8, FOOT - 5.5, 3, 4.5, WOOD_L)
    p.ell(cx + 8, FOOT - 5.5, 1.3, 2, WOOD, line=False)


def gate(p, cx):
    """뿌리 길 입구: 땅에 난 어두운 구멍 + 둘레 뿌리 넷 + 작은 나무 팻말. 폭 40 키 26."""
    p.ell(cx, FOOT - 8, 15, 7, (60, 40, 30))
    p.ell(cx, FOOT - 7, 11, 4.5, (14, 8, 14), line=False)
    for a, b in (((cx - 17, FOOT - 12), (cx - 9, FOOT - 9)), ((cx + 17, FOOT - 13), (cx + 9, FOOT - 10)),
                 ((cx - 14, FOOT - 1), (cx - 7, FOOT - 4)), ((cx + 13, FOOT), (cx + 6, FOOT - 3))):
        p.line(a, b, WOOD_D, 2)
    p.box(cx - 19, FOOT - 24, cx - 17, FOOT - 12, WOOD_D)
    p.box(cx - 24, FOOT - 26, cx - 12, FOOT - 20, WOOD_L)


# --- B·C: 마을 꾸미기 -----------------------------------------------------------------------------------------------
def flowerbed(p, cx):
    """꽃밭: 흙 둔덕에 덤불 둘 + 큰 꽃 셋. 폭 40 키 20."""
    p.ell(cx, FOOT - 4, 19, 4, (120, 84, 56))
    p.ell(cx - 9, FOOT - 10, 8, 7, LEAF)
    p.ell(cx + 9, FOOT - 10, 8, 7, LEAF)
    for x, y, c in ((cx - 11, FOOT - 13, (220, 70, 80)), (cx + 2, FOOT - 15, (250, 210, 80)), (cx + 11, FOOT - 12, (240, 240, 250))):
        p.ell(x, y, 2.5, 2.5, c)


def lantern(p, cx):
    """반딧불 등: 나무 기둥 끝에 매단 유리 등(안에 반딧불). 폭 8 키 22."""
    p.box(cx - 1, FOOT - 20, cx + 1, FOOT, WOOD_D)
    p.box(cx - 4, FOOT - 22, cx + 4, FOOT - 14, (250, 236, 150))


def glow_mush(p, cx):
    """빛나는 버섯: 파란 갓 버섯 셋(키 다르게). 폭 24 키 16."""
    for dx, h, r in ((-7, 10, 4), (1, 14, 5), (8, 8, 3.5)):
        p.box(cx + dx - 1, FOOT - h + 2, cx + dx + 1, FOOT, CREAM)
        p.ell(cx + dx, FOOT - h + 1, r, r * 0.6, (80, 170, 240))


def bench(p, cx):
    """통나무 의자: 반 가른 통나무 의자, 다리 둘. 폭 26 키 14."""
    p.box(cx - 12, FOOT - 12, cx + 12, FOOT - 7, WOOD_L)
    for dx in (-9, 7):
        p.box(cx + dx, FOOT - 7, cx + dx + 3, FOOT, WOOD_D)


def chime(p, cx):
    """바람종: 기둥 + 가로대 + 매달린 대롱 셋. 폭 14 키 20."""
    p.box(cx - 1, FOOT - 20, cx + 1, FOOT, WOOD_D)
    p.box(cx - 6, FOOT - 20, cx + 6, FOOT - 18, WOOD)
    for dx in (-5, 0, 5):
        p.box(cx + dx - 1, FOOT - 17, cx + dx + 1, FOOT - 11, (200, 210, 220))


def bunting(p, cx):
    """깃발 줄: 기둥 둘 사이 줄에 작은 세모 깃발 여섯(빨강·노랑·파랑). 폭 36 키 18."""
    for dx in (-17, 15):
        p.box(cx + dx, FOOT - 18, cx + dx + 2, FOOT, WOOD_D)
    p.line((cx - 16, FOOT - 17), (cx + 16, FOOT - 15), OUTLINE)
    for i, c in enumerate([(220, 70, 80), (250, 210, 80), (90, 150, 230)] * 2):
        x = cx - 13 + i * 5
        p.poly([(x - 2, FOOT - 16), (x + 2, FOOT - 16), (x, FOOT - 11)], c)


def jar(p, cx):
    """반딧불 병: 코르크 마개 유리병 안에 빛 넷. 폭 12 키 16."""
    p.box(cx - 5, FOOT - 13, cx + 5, FOOT, (200, 230, 230))
    p.box(cx - 3, FOOT - 16, cx + 3, FOOT - 13, WOOD)
    for x, y in ((-2, 9), (2, 6), (0, 3), (3, 10)):
        p.ell(cx + x, FOOT - y, 1, 1, (250, 240, 120), line=False)


def pot(p, cx):
    """들꽃 화분: 흙 화분에 들꽃 셋. 폭 14 키 18."""
    p.box(cx - 5, FOOT - 8, cx + 5, FOOT, (180, 100, 70))
    for dx, c in ((-3, (220, 70, 80)), (0, (250, 210, 80)), (3, (180, 120, 230))):
        p.line((cx + dx, FOOT - 8), (cx + dx, FOOT - 14), LEAF_D)
        p.ell(cx + dx, FOOT - 15, 1.6, 1.6, c)


def swing(p, cx):
    """실 그네: 나무 틀 둘 + 흰 실 두 가닥 + 나무 발판. 폭 20 키 24."""
    p.box(cx - 10, FOOT - 24, cx + 10, FOOT - 22, WOOD)
    for dx in (-10, 8):
        p.box(cx + dx, FOOT - 22, cx + dx + 2, FOOT, WOOD_D)
    for dx in (-4, 4):
        p.line((cx + dx, FOOT - 22), (cx + dx, FOOT - 8), SILK)
    p.box(cx - 6, FOOT - 8, cx + 6, FOOT - 6, WOOD_L)


def puddle(p, cx):
    """물웅덩이: 납작한 파란 물(하늘이 비친 밝은 줄). 폭 24 키 10."""
    p.ell(cx, FOOT - 5, 12, 4.5, (96, 150, 200))
    p.ell(cx - 3, FOOT - 6, 5, 1.2, (190, 220, 240), line=False)


def loop(p, cx):
    """실 고리: 땅에 꽂힌 막대에 감긴 흰 실 고리(꿈으로 가는 표지). 폭 16 키 20."""
    p.box(cx - 1, FOOT - 14, cx + 1, FOOT, WOOD_D)
    p.d.ellipse([(cx - 7) * Z, (FOOT - 20) * Z, (cx + 7) * Z, (FOOT - 8) * Z], outline=SILK, width=2 * Z)


SHEETS = {
    "a": [stall, firepit, stump, boulder, log_seat, gate],
    "b": [flowerbed, lantern, glow_mush, bench, chime, bunting],
    "c": [jar, pot, swing, puddle, loop],
}
CELL = 48


def board(items):
    w = 4 + 24 + 8 + len(items) * (CELL + 6) + 4
    out = Image.new("RGBA", (w * Z, (CELL + 8) * Z), (255, 0, 255, 255))
    nail = Image.open(HERE.parents[1] / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    out.alpha_composite(nail.resize((24 * Z, 24 * Z), Image.NEAREST), (4 * Z, (4 + CELL - 24) * Z))
    for i, fn in enumerate(items):
        p = Pad(CELL, CELL)
        fn(p, CELL / 2)
        out.alpha_composite(p.im, ((4 + 24 + 8 + i * (CELL + 6)) * Z, 4 * Z))
    return out


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for k, items in SHEETS.items():
        board(items).save(OUT / f"ref_hub_{k}.png")
    print(", ".join(f"ref_hub_{k}.png" for k in SHEETS))


if __name__ == "__main__":
    main()
