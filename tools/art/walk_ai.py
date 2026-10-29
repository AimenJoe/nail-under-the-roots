"""걷기 네 칸(0229): 따로 받은 나노바나나 걷기 시트 → 그 이웃 시트 끝에 붙일 네 칸. 각 변환기가 부른다.

원본: `sources/ai/sprites/<이름>_walk_v1.png`(✦ 지움, 마젠타 바탕 한 줄 네 칸). 참고 그림은 `walk_ref.py`, 프롬프트는
`docs/art/walk-cycle-prompts.md`.

- 색표는 그 이웃 원본 시트의 서기 칸에서(다른 칸과 같은 색) — 원본 서기 칸을 이 Spec의 원본 0으로 함께 싣는다.
- 배율은 네 칸이 한 무리(기준 칸 `ref`)라 걸음의 들썩임이 남는다. `h`는 기준 칸의 외곽선 안 키 — 서기 칸과 나란히
  놓아 머리·몸통 크기가 같아지게 맞춘 값.
"""
from pathlib import Path

from sprite_ai import Source, Spec, convert, seat

HERE = Path(__file__).resolve().parent
NAMES = ["walk4_a", "walk4_b", "walk4_c", "walk4_d"]
# 이름: 칸 상자(디딤·지나감·디딤·지나감 순), 기준 칸, 기준 칸 키, 원본 외곽선 두께(px), 진분홍 여부.
WALK = {
    "popo": dict(boxes=[(24, 67, 250, 303), (268, 40, 496, 302), (519, 66, 746, 302), (763, 40, 997, 302)],
                 ref=1, h=24, strip=9),
    "bori": dict(boxes=[(32, 41, 230, 306), (288, 41, 486, 306), (553, 33, 746, 306), (793, 24, 1011, 306)],
                 ref=0, h=25, strip=9),
    "susu": dict(boxes=[(33, 99, 257, 384), (272, 113, 499, 384), (520, 99, 744, 384), (758, 94, 1004, 384)],
                 ref=0, h=18, strip=12),
    # 콩이는 0238부터 걷기 넷이 제 시트 안(kongi_ai.py) — 이 줄은 옛 원본 기록.
    "kongi": dict(boxes=[(41, 124, 278, 302), (284, 106, 520, 293), (529, 115, 766, 302), (779, 89, 1016, 293)],
                  ref=0, h=19, strip=8),
    "clover": dict(boxes=[(25, 111, 256, 309), (273, 68, 512, 309), (532, 46, 775, 237), (782, 89, 1019, 308)],
                   ref=3, h=26, strip=6, pink=True),
    # 모모 깡충: 원본의 ✦가 바탕(마젠타)에 있어 그 자리만 마젠타로 다시 칠해 저장했다(메우기는 발 둘레를 번지게 했다).
    "momo": dict(boxes=[(34, 142, 241, 293), (293, 61, 490, 293), (528, 43, 739, 213), (779, 106, 990, 293)],
                 ref=3, h=21, strip=9),
}


def source(name):
    return HERE / "sources" / "ai" / "sprites" / f"{name}_walk_v1.png"


def spec(name, orig, stand="idle_a"):
    """`orig`: 그 이웃 원본 시트의 Source(서기 칸 `stand`의 색표를 쓴다)."""
    w = WALK[name]
    walk = Source(source(name), dict(zip(NAMES, w["boxes"])),
                  groups=dict.fromkeys(NAMES, NAMES[w["ref"]]), tidy=0.15, strip=w["strip"], pink=w.get("pink", False))
    base = Source(orig.path, {stand: orig.frames[stand]}, groups={stand: stand}, tidy=orig.tidy, strip=orig.strip,
                  pink=orig.pink)
    return Spec(sources=[base, walk], palette_ref=(0, stand), body_h=w["h"])


def cells(name, orig, cell, foot, stand="idle_a"):
    """이름 → 칸에 앉힌 걷기 네 칸(외곽선 포함)."""
    spr = convert(spec(name, orig, stand))
    return {k: seat(spr[k], cell, foot) for k in NAMES}
