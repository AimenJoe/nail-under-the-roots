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
]

PROMPT = """Redraw the creature in the attached image (the right figure) as a pixel art sprite sheet for a cute top-down
action game: {look}. It is corrupted by purple threads: glowing red eyes and a few small purple root-threads on its
back. No mouth line.

Use the attached image for size: draw it as big as the right figure next to the small mole, same pixel block size.
Keep every frame the same body size.

3 columns x 2 rows, 6 frames, all facing right (3/4 view), solid magenta background (#FF00FF), figures well separated,
no text, no grid lines, no shadows, no effects.
1. standing   2. walking (one leg forward)   3. {windup}
4. {attack}   5. hurt (flinching back, eyes squeezed shut)   6. defeated (lying on its side, eyes closed)"""


def board(row, target):
    """Naïl 서기 옆에 critters `row`줄 서기 칸을 목표 키로 키운 것(가장 가까운 점), 발 줄을 맞춰 8배."""
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
        board(row, target).save(OUT / f"ref_mob_{name}.png")
        lines += [f"## {name} (키 {target}px, critters 줄 {row})", "", "```",
                  PROMPT.format(look=look, windup=windup, attack=attack), "```", ""]
    (ROOT / "docs" / "art" / "mob-sprite-prompts.md").write_text("\n".join(lines))
    print(len(MOBS), "mobs")


if __name__ == "__main__":
    main()
