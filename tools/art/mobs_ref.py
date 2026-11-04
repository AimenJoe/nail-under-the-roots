"""잡몹 열(0221): 24칸 새 그림을 받으려고 나노바나나에 붙일 크기·생김새 기준 그림과 프롬프트를 만든다.

실행: python3 tools/art/mobs_ref.py [출력 폴더]   (기본 tools/art/sources/ai/sprites/ref)
  - ref_mob_<이름>.png: 게임 크기 Naïl(24칸, 몸 16px) 옆에 지금 16칸 그림(서기)을 목표 키로 키운 것, 마젠타 바탕 8배.
    잡몹은 캐릭터 시트가 없어 이 한 장이 크기 기준이자 생김새 참고다(우리 코드 그림이라 0064에 걸리지 않는다).
  - docs/art/mob-sprite-prompts.md: 잡몹마다 프롬프트(이 표에서 만든다 — 표를 고치면 다시 돌린다).

왜 24칸: 플레이어·이웃은 24~32칸, 보스는 32~96칸으로 커져 16칸 잡몹만 작고 거칠었다(사장 확인 2026-10-28). 키는 종마다
달라 크고 작은 맛을 남긴다(생쥐 12 … 청설모 18). 받는 법은 수수 때 통한 것: 새 대화, 그림 한두 장, 짧은 프롬프트, 3×2 여섯 칸.
"""
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "sources" / "ai" / "sprites" / "ref"
CELL, FOOT = 24, 22

# 이름, critters 줄, 목표 키(외곽선까지), 생김새, 3칸(예고), 4칸(공격)
MOBS = [
    ("field_mouse", 0, 14, "a small brown field mouse with a pink nose, round ears and a thin pink tail",
     "crouching low, tense, about to dash", "dashing forward, body stretched"),
    ("house_mouse", 1, 12, "a tiny grey house mouse with big round pink ears and a thin pink tail",
     "crouching low, tense, about to dash", "dashing forward, body stretched"),
    ("sewer_rat", 2, 16, "a big chunky dark grey sewer rat with a long pink tail and a bristly back",
     "crouching low, tense, about to charge", "charging forward, body stretched, heavy"),
    ("chipmunk", 3, 17, "a brown chipmunk with a cream stripe along its side and a fluffy upright tail",
     "holding an acorn up high, about to throw", "throwing the acorn forward"),
    ("red_squirrel", 4, 18, "a dark red-brown squirrel with tufted ears and a big bushy curled tail",
     "crouching low, about to leap", "leaping through the air, body stretched"),
    ("shrew", 14, 10, "a tiny dark grey shrew with a long pointed pink snout and a thin tail",
     "low and tense, about to dart", "darting forward, very stretched"),
    ("hedgehog", 15, 14, "a round brown hedgehog with dark spines on its back and a pale pointed face",
     "curled up into a spiky ball (face hidden)", "the same spiky ball, rotated a little (rolling)"),
    ("toad", 16, 13, "a squat olive-green toad with warty skin and a pale belly",
     "puffed up, about to strike", "mouth wide open (its tongue is drawn by the game, do not draw a tongue)"),
    ("bat", 17, 13, "a small dark purple-grey bat with big wings and small ears (always flying)",
     "hovering with wings spread wide", "diving forward, wings folded back"),
    ("spore", 18, 17, "a walking mushroom: a dark red cap with pale spots on a short pale stem with tiny feet",
     "cap squashed down, about to burst", "cap puffed up, releasing a puff (no spore cloud, the game draws it)"),
    # 물길 굴(0275 P4): 가재는 줄이며 가는 팔이 끊겼다 → 소금쟁이는 다리를 굵게 그리라고 적는다. critters 줄이 없어 게임의 코드 임시 그림(`render/actors.rs draw_code_mob`)을 옮긴 `code_ref`가 기준.
    ("crayfish", None, 11, "a small reddish-brown crayfish with two big pincers held forward, a segmented tail and thin feelers",
     "pincers raised and open, about to snap", "lunging forward, snapping both pincers shut"),
    ("strider", None, 9, "a dark brown water strider (pond skater) with a small chunky body held up on long bent legs. Draw the legs THICK and bold (at least 3 pixel blocks wide, with an outline), because the game shrinks the sheet a lot and thin lines disappear",
     "legs bent low, tapping the surface", "legs stamping down hard (no ripple, the game draws it)"),
    # 넷째 길의 주인 무무(0275 P5): 보스라 크게(키 16·길이 약 36 — 칸 40 꼴). 동굴도롱뇽(olm)처럼 눈이 없고 살빛이 옅은 분홍.
    ("mumu", None, 16, "a big old blind cave salamander (an olm): a long pale pink body, small stubby legs, a long flat tail, "
     "feathery pink gills on both sides of the head and NO eyes at all (smooth skin where eyes would be)",
     "head raised and mouth open, about to spit water (no water, the game draws it)",
     "sliding forward low on its belly, body stretched, tail swinging"),
]


def code_ref(name):
    """critters 줄이 없는 종: 게임의 코드 임시 그림(draw_code_mob)과 같은 점들을 16칸 가운데(8, 8)에 오른쪽을 보게."""
    im = Image.new("RGBA", (24, 24), (0, 0, 0, 0))
    px = im.load()

    def rect(x, y, w, h, c):
        for yy in range(h):
            for xx in range(w):
                px[12 + x + xx, 12 + y + yy] = c

    line = (34, 24, 30, 255)
    if name == "mumu":
        # 옆모습 실루엣(40칸 보스 꼴에 맞춰 길이 약 36): 둥근 긴 몸통 + 가늘어지는 꼬리 + 짧은 다리 넷 + 머리 양옆 깃털 아가미, 눈 없음.
        from PIL import ImageDraw
        big = Image.new("RGBA", (48, 24), (0, 0, 0, 0))
        d = ImageDraw.Draw(big)
        skin, belly, gill = (226, 186, 190, 255), (240, 210, 212, 255), (232, 120, 140, 255)
        for (x0, y0, x1, y1) in [(15, 14, 18, 20), (21, 14, 24, 20), (29, 14, 32, 20), (34, 14, 37, 20)]:
            d.rectangle((x0, y0, x1, y1), fill=line)
            d.rectangle((x0 + 1, y0, x1 - 1, y1 - 1), fill=skin)
        d.polygon([(3, 14), (16, 10), (16, 16)], fill=line)            # 꼬리
        d.polygon([(5, 14), (16, 11), (16, 15)], fill=skin)
        d.ellipse((12, 8, 40, 18), fill=line)                          # 몸통
        d.ellipse((13, 9, 39, 17), fill=skin)
        d.line((16, 15, 36, 15), fill=belly)
        d.ellipse((35, 7, 46, 16), fill=line)                          # 머리
        d.ellipse((36, 8, 45, 15), fill=skin)
        for gx, gy in [(36, 6), (38, 5), (37, 16)]:                    # 아가미
            d.line((gx, gy, gx + 1, gy + 2), fill=gill)
        return big
    if name == "crayfish":
        shell, dark, lite = (184, 82, 61, 255), (115, 43, 38, 255), (235, 143, 107, 255)
        for x, y, w, h, c in [(-9, -1, 3, 4, line), (-8, 0, 2, 2, dark), (-7, -2, 3, 5, line), (-6, -1, 2, 3, shell),
                              (-5, -2, 3, 5, line), (-4, -1, 2, 3, shell), (-3, -3, 8, 7, line), (-2, -2, 6, 5, shell),
                              (-1, -2, 3, 1, lite), (0, 0, 3, 1, (158, 97, 204, 255)), (4, -3, 1, 1, (0, 0, 0, 255)),
                              (4, 2, 1, 1, (0, 0, 0, 255))]:
            rect(x, y, w, h, c)
        for side in (-1, 1):
            y = side * 3
            rect(4, y, 3, 1, dark)
            rect(7, y - 1, 4, 3, line)
            rect(8, y, 2, 1, shell)
    else:
        # 옆모습(게임 코드 그림은 위에서 본 모양이라 크기만 같다): 가는 몸을 긴 다리 셋이 물 위로 들어 올린다.
        body, leg = (77, 69, 61, 255), (51, 46, 43, 255)
        from PIL import ImageDraw
        d = ImageDraw.Draw(im)
        for knee, foot in [((4, -5), (8, 4)), ((0, -6), (-3, 4)), ((-4, -5), (-10, 4))]:
            d.line((12 - 1 + knee[0] // 3, 12 - 1, 12 + knee[0], 12 + knee[1]), fill=leg)
            d.line((12 + knee[0], 12 + knee[1], 12 + foot[0], 12 + foot[1]), fill=leg)
        for x, y, w, h, c in [(-5, -2, 10, 3, line), (-4, -1, 8, 1, body), (5, -2, 2, 2, line), (6, -2, 1, 1, (217, 64, 77, 255))]:
            rect(x, y, w, h, c)
    return im

# 들쥐·생쥐(첫 둘)에서 배운 것: 들쥐는 서기·걷기를 뒷모습으로 그렸고 눈을 검게 그렸다 → "모든 칸 오른쪽 옆모습, 뒷모습 금지",
# "눈은 빛나는 빨강"을 세게 적는다.
PROMPT = """Redraw the creature in the attached image (the right figure) as a pixel art sprite sheet for a cute top-down
action game: {look}. {taint} No mouth line.

Use the attached image for size: draw it as big as the right figure next to the small mole, same pixel block size.
Keep every frame the same body size.

3 columns x 2 rows, 6 frames. EVERY frame is a side view facing RIGHT (never a back view or front view).
Solid magenta background (#FF00FF), figures well separated, no text, no grid lines, no shadows, no effects.
1. standing   2. walking (one leg forward)   3. {windup}
4. {attack}   5. {hurt}   6. {down}"""

TAINT = ("It is corrupted by purple threads: its eyes are GLOWING RED (red, not black), and a few small\n"
         "purple root-threads grow on its back.")
# 눈이 없는 것(무무)은 표지·맞음·쓰러짐 글을 바꾼다 — 틀 글대로면 "빛나는 붉은 눈"을 그려 온다.
OVERRIDE = {
    "mumu": dict(
        taint="It is corrupted by purple threads: a faint red glow shows under the skin where eyes would be, and a few "
              "small purple root-threads are wrapped around its body.",
        hurt="hurt (flinching back, gills pressed flat)",
        down="defeated (lying on its side, limp)",
    ),
}


def board(row, target):
    """Naïl 서기 옆에 critters `row`줄 서기 칸을 목표 키로 키운 것(가장 가까운 점), 발 줄을 맞춰 8배."""
    if isinstance(row, str):
        crit = code_ref(row)
    else:
        crit = Image.open(ROOT / "assets" / "art" / "critters.png").convert("RGBA").crop((0, row * 16, 16, row * 16 + 16))
    box = crit.getbbox()
    crit = crit.crop(box)
    k = target / crit.height
    crit = crit.resize((max(1, round(crit.width * k)), target), Image.NEAREST)
    nail = Image.open(ROOT / "assets" / "art" / "nail.png").convert("RGBA").crop((0, 0, 24, 24))
    w = 4 + 24 + 8 + max(CELL, crit.width) + 4
    b = Image.new("RGBA", (w, CELL + 8), (255, 0, 255, 255))
    b.alpha_composite(nail, (4, 4))
    b.alpha_composite(crit, (4 + 24 + 8, 4 + FOOT + 1 - crit.height))
    return b.resize((b.width * 8, b.height * 8), Image.NEAREST)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    lines = ["# 잡몹 새 그림 프롬프트(0221)", "",
             "`tools/art/mobs_ref.py`가 만든다(표를 고치면 다시 돌린다). 잡몹마다 **새 대화**에서 그 잡몹의 "
             "`tools/art/sources/ai/sprites/ref/ref_mob_<이름>.png` 한 장만 붙이고 아래 프롬프트를 넣는다. "
             "받은 원본은 `tools/art/sources/ai/sprites/<이름>_v1.png`로.", ""]
    for name, row, target, look, windup, attack in MOBS:
        board(name if row is None else row, target).save(OUT / f"ref_mob_{name}.png")
        where = "코드 임시 그림" if row is None else f"critters 줄 {row}"
        lines += [f"## {name} (키 {target}px, {where})", "", "```",
                  PROMPT.format(look=look, windup=windup, attack=attack,
                                **{**dict(taint=TAINT, hurt="hurt (flinching back, eyes squeezed shut)",
                                          down="defeated (lying on its side, eyes closed)"), **OVERRIDE.get(name, {})}),
                  "```", ""]
    (ROOT / "docs" / "art" / "mob-sprite-prompts.md").write_text("\n".join(lines))
    print(len(MOBS), "mobs")


if __name__ == "__main__":
    main()
