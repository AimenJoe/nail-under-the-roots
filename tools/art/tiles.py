"""대체 타일셋 생성기(Sunnyside World가 들어오기 전 임시, 프로젝트 소유 그림).

실행: python3 tools/art/tiles.py [출력 경로]  (기본 assets/art/tiles.png)
16×16 칸. 줄 0: 풀 3종, 흙, 꽃(흰·노랑·분홍), 풀잎
        줄 1: 벽 윗면, 벽 앞면, 벽 앞면(뿌리), 돌멩이, 버섯(빛남), 뿌리 덩굴, 이끼, 빈칸
        줄 2~3: 그루터기 기둥(16×32, 0열), 바위 기둥(16×32, 1열), 흙 얼룩 2종(줄 2, 2·3열), 풀 얼룩 2종(줄 3, 2·3열)
같은 시드면 같은 그림이 나온다(재현 가능).
"""
import random
import sys
from PIL import Image

T = 16
rng = random.Random(20261025)

GRASS = [(96, 156, 80), (110, 172, 88), (82, 136, 72), (132, 190, 98)]
DIRT = [(150, 110, 78), (128, 92, 64), (172, 132, 94)]
WALL_TOP = [(86, 140, 74), (100, 156, 82), (70, 116, 64)]
EARTH = [(118, 84, 62), (98, 70, 52), (142, 104, 74), (78, 56, 44)]
OUT = (43, 29, 42)


def px(img, x, y, c):
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), c + (255,) if len(c) == 3 else c)


def fill(img, ox, oy, w, h, c):
    for y in range(h):
        for x in range(w):
            px(img, ox + x, oy + y, c)


def grass(img, ox, oy, variant):
    r = random.Random(variant * 97 + 3)
    fill(img, ox, oy, T, T, GRASS[0])
    for _ in range(18):
        x, y = r.randrange(T), r.randrange(T)
        px(img, ox + x, oy + y, GRASS[2])
        if y > 0:
            px(img, ox + x, oy + y - 1, GRASS[1])  # 풀잎: 아래 어둡고 위 밝게
    for _ in range(4 + variant * 2):
        x, y = r.randrange(T), r.randrange(T)
        px(img, ox + x, oy + y, GRASS[3])


def dirt(img, ox, oy):
    r = random.Random(11)
    fill(img, ox, oy, T, T, DIRT[0])
    for _ in range(22):
        px(img, ox + r.randrange(T), oy + r.randrange(T), DIRT[1])
    for _ in range(10):
        px(img, ox + r.randrange(T), oy + r.randrange(T), DIRT[2])


def flower(img, ox, oy, petal, center, seed):
    r = random.Random(seed)
    spots = [(4, 5), (11, 9), (6, 12)] if seed % 2 else [(10, 4), (4, 10), (12, 13)]
    for (x, y) in spots[: 2 + seed % 2]:
        x += r.randrange(-1, 2)
        y += r.randrange(-1, 2)
        px(img, ox + x, oy + y + 2, GRASS[2])          # 줄기
        px(img, ox + x, oy + y + 1, GRASS[1])
        for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1)):
            px(img, ox + x + dx, oy + y + dy, petal)
        px(img, ox + x, oy + y, center)


def tuft(img, ox, oy):
    for (x, y) in [(5, 10), (6, 9), (7, 10), (9, 6), (10, 5), (11, 6)]:
        px(img, ox + x, oy + y, GRASS[3])
    for (x, y) in [(5, 11), (7, 11), (9, 7), (11, 7)]:
        px(img, ox + x, oy + y, GRASS[2])


def wall_top(img, ox, oy):
    r = random.Random(5)
    fill(img, ox, oy, T, T, WALL_TOP[0])
    for _ in range(16):
        px(img, ox + r.randrange(T), oy + r.randrange(T - 3), WALL_TOP[1])
    for x in range(T):  # 아래 가장자리: 풀이 벼랑 위로 늘어짐
        px(img, ox + x, oy + T - 2, WALL_TOP[2])
        if r.random() < 0.5:
            px(img, ox + x, oy + T - 1, WALL_TOP[2])


def wall_front(img, ox, oy, roots):
    r = random.Random(7 + roots)
    fill(img, ox, oy, T, T, EARTH[0])
    for y in range(0, T, 5):  # 흙 지층
        for x in range(T):
            if r.random() < 0.8:
                px(img, ox + x, oy + y + r.randrange(2), EARTH[1])
    for _ in range(8):  # 돌 박힘
        x, y = r.randrange(1, T - 2), r.randrange(1, T - 2)
        px(img, ox + x, oy + y, EARTH[2])
        px(img, ox + x + 1, oy + y, EARTH[2])
        px(img, ox + x, oy + y + 1, EARTH[3])
    for x in range(T):  # 아래 가장자리 그늘
        px(img, ox + x, oy + T - 1, EARTH[3])
    if roots:  # 뿌리가 벽을 타고 내려온다
        x = 5
        for y in range(T):
            px(img, ox + x, oy + y, (170, 130, 92))
            px(img, ox + x + 1, oy + y, (120, 86, 60))
            if r.random() < 0.3:
                x += r.choice((-1, 1))
        for (x, y) in [(10, 3), (11, 4), (11, 5), (12, 6)]:
            px(img, ox + x, oy + y, (170, 130, 92))


def pebble(img, ox, oy):
    for (x, y, c) in [(6, 9, (150, 146, 140)), (7, 9, (176, 172, 164)), (6, 10, (120, 116, 112)),
                      (7, 10, (150, 146, 140)), (10, 12, (150, 146, 140)), (11, 12, (120, 116, 112))]:
        px(img, ox + x, oy + y, c)


def mushroom(img, ox, oy):
    # 깊은 곳의 빛나는 버섯 두 개
    for (bx, by, s) in [(5, 9, 1), (10, 12, 0)]:
        cap = (120, 220, 210) if s else (170, 140, 230)
        glow = (200, 255, 245) if s else (220, 200, 255)
        stem = (220, 210, 190)
        px(img, ox + bx, oy + by + 1, stem)
        px(img, ox + bx, oy + by + 2, stem)
        for dx in (-1, 0, 1):
            px(img, ox + bx + dx, oy + by, cap)
        px(img, ox + bx, oy + by - 1, cap)
        px(img, ox + bx - 1, oy + by, glow)


def root_vine(img, ox, oy):
    r = random.Random(3)
    y = 4
    for x in range(T):
        px(img, ox + x, oy + y, (110, 78, 56))
        px(img, ox + x, oy + y + 1, (78, 54, 40))
        if r.random() < 0.35:
            y = max(2, min(13, y + r.choice((-1, 1))))
    for (x, yy) in [(4, 7), (4, 8), (5, 9), (11, 8), (11, 9), (12, 10)]:
        px(img, ox + x, oy + yy, (110, 78, 56))


def moss(img, ox, oy):
    r = random.Random(9)
    for _ in range(14):
        x, y = r.randrange(2, 14), r.randrange(6, 14)
        px(img, ox + x, oy + y, (70, 120, 90))
        px(img, ox + x + 1, oy + y, (90, 140, 100))


def stump(img, ox, oy):
    # 16×32. 아래 16이 충돌 발판(기둥 자리), 위 16은 몸통이 솟은 부분.
    bark, bark_d, bark_l = (122, 84, 58), (88, 60, 44), (152, 110, 76)
    ring, ring_d = (214, 176, 126), (180, 140, 96)
    # 몸통(앞면)
    for y in range(12, 30):
        for x in range(2, 14):
            c = bark
            if x in (2, 3):
                c = bark_d
            elif x in (11, 12, 13):
                c = bark_d if x == 13 else bark
            if (x + y) % 5 == 0 and 3 < x < 12:
                c = bark_l
            px(img, ox + x, oy + y, c)
    # 뿌리가 바닥으로 퍼짐
    for (x, y) in [(0, 29), (1, 28), (1, 29), (14, 28), (14, 29), (15, 29), (5, 30), (6, 30), (10, 30)]:
        px(img, ox + x, oy + y, bark_d)
    # 윗면(나이테) 3/4 시점: 타원
    for y in range(8, 14):
        for x in range(2, 14):
            dx, dy = (x - 7.5) / 6.0, (y - 11.0) / 3.2
            if dx * dx + dy * dy <= 1.0:
                c = ring if (int((dx * dx + dy * dy) * 6) % 2 == 0) else ring_d
                px(img, ox + x, oy + y, c)
    # 외곽선
    outline_region(img, ox, oy, 16, 32)


def rock(img, ox, oy):
    base, dark, light = (138, 132, 128), (100, 96, 96), (176, 172, 166)
    for y in range(10, 31):
        for x in range(1, 15):
            dx, dy = (x - 7.5) / 7.0, (y - 21.0) / 10.5
            if dx * dx + dy * dy <= 1.0:
                c = base
                if dx < -0.3 and dy < 0.2:
                    c = light
                if dx > 0.4 or dy > 0.6:
                    c = dark
                px(img, ox + x, oy + y, c)
    for (x, y) in [(6, 14), (7, 15), (10, 20), (11, 21), (5, 24)]:
        px(img, ox + x, oy + y, dark)
    # 이끼 모자
    for x in range(4, 11):
        px(img, ox + x, oy + 11 + (x % 2), (90, 150, 84))
    outline_region(img, ox, oy, 16, 32)


def outline_region(img, ox, oy, w, h):
    pts = []
    for y in range(h):
        for x in range(w):
            if img.getpixel((ox + x, oy + y))[3] != 0:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and img.getpixel((ox + nx, oy + ny))[3] != 0:
                    pts.append((x, y))
                    break
    for (x, y) in pts:
        px(img, ox + x, oy + y, OUT)


def patch(img, ox, oy, seed, cols):
    """가장자리가 불규칙한 얼룩(투명 배경). 바닥 위에 덮어 네모 칸 경계를 숨긴다."""
    r = random.Random(seed)
    blobs = [(r.uniform(4, 12), r.uniform(4, 12), r.uniform(3.0, 5.5)) for _ in range(4)]
    for y in range(T):
        for x in range(T):
            inside = any((x - bx) ** 2 + (y - by) ** 2 <= rad * rad for bx, by, rad in blobs)
            if not inside:
                continue
            c = cols[0]
            v = r.random()
            if v < 0.18:
                c = cols[1]
            elif v < 0.26:
                c = cols[2]
            px(img, ox + x, oy + y, c)


def sheet():
    img = Image.new('RGBA', (T * 8, T * 4), (0, 0, 0, 0))
    for i in range(3):
        grass(img, i * T, 0, i)
    dirt(img, 3 * T, 0)
    flower(img, 4 * T, 0, (250, 248, 240), (250, 210, 90), 1)
    flower(img, 5 * T, 0, (252, 214, 96), (230, 140, 60), 2)
    flower(img, 6 * T, 0, (244, 160, 196), (252, 240, 200), 3)
    tuft(img, 7 * T, 0)
    wall_top(img, 0, T)
    wall_front(img, T, T, 0)
    wall_front(img, 2 * T, T, 1)
    pebble(img, 3 * T, T)
    mushroom(img, 4 * T, T)
    root_vine(img, 5 * T, T)
    moss(img, 6 * T, T)
    stump(img, 0, 2 * T)
    rock(img, T, 2 * T)
    # 얼룩: 흙 2종(2·3열, 줄 2), 풀 2종(2·3열, 줄 3)
    patch(img, 2 * T, 2 * T, 41, DIRT)
    patch(img, 3 * T, 2 * T, 42, DIRT)
    patch(img, 2 * T, 3 * T, 43, [GRASS[0], GRASS[2], GRASS[3]])
    patch(img, 3 * T, 3 * T, 44, [GRASS[0], GRASS[2], GRASS[1]])
    return img


if __name__ == '__main__':
    sheet().save(sys.argv[1] if len(sys.argv) > 1 else 'assets/art/tiles.png')
