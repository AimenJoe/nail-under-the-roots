"""도토(청설모) 스프라이트 시트(40×40 칸, 발바닥 38, 0206 C3). 몸 크기는 Naïl·모모와 같고 칸만 넓다 — 달리기·쓰러짐은
꼬리를 뒤로 길게 뻗어 외곽선까지 37px라 32칸에서도 잘렸다.

실행: python3 tools/art/doto_ai.py [출력 경로]   (기본 assets/art/doto.png)

원본: `sources/ai/sprites/doto_v1.png`(✦ 지움). 모모 6배 그림을 점 크기 기준으로 붙여 받았는데 한 단 크게(블록 9~10px) 왔다
→ 공용 변환기(`sprite_ai`)로 원본 외곽선을 걷고(`strip`) 서기 칸의 키(꼬리 끝~발, 외곽선 안)를 23px로 줄인다
(모모 서기 칸과 나란히 놓아 같은 몸 크기로 맞춘 값). 서기 두 칸은 정면을 보고 왔다(그대로 쓴다). 달리기는 한 칸만 와서
둘째 칸은 같은 그림을 1px 띄운다(걷기 통통). 달리기 네 칸은 따로 받은 `doto_run_v1.png`(바운드: 밀기·공중·착지·모으기, 0226).

칸 배치(critters 도토 줄과 같은 뜻, `art::Art::critter`가 도토 두 줄을 여기로 돌린다):
  칸 0·1 서기, 2·3 걷기(달리기 그림 한 칸), 4 겨누기(던지기 들기), 5 던지기, 6 맞음, 7 쓰러짐, 8~11 달리기 바운드(있으면 달릴 때 돈다)
  줄 0 돌아온 도토 · 줄 1 물든 도토(보스: 회보라 털·호박색으로 빛나는 눈 — 줄 0의 색을 바꿔 만든다, 모자·도토리는 그대로)
  줄 2·3: 줄 0·1의 흰 실루엣(피격 번쩍·다 모인 몸빛)
"""
import colorsys
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sprite_ai import OUTLINE, Source, Spec, convert, lum, ramp, seat, sheet4  # noqa: E402

CELL, FOOT = 40, 38
NAMES = ["idle_a", "idle_b", "run", "aim", "throw", "hurt", "down"]
SPEC = Spec(
    sources=[
        Source(
            HERE / "sources" / "ai" / "sprites" / "doto_v1.png",
            dict(zip(NAMES, [(28, 68, 268, 344), (292, 68, 532, 344), (576, 96, 988, 324), (124, 408, 400, 688),
                             (560, 408, 888, 688), (108, 716, 396, 980), (548, 820, 952, 976)])),
            groups=dict.fromkeys(NAMES, "idle_a"),
            tidy=0.15,
            strip=10,
        )
    ],
    palette_ref=(0, "idle_a"),
    body_h=23,
)
# 달리기 네 칸(0226): `doto_run_v1.png`(✦ 지움) — 바운드(밀기·공중·착지·모으기). 원본 크기가 달라 따로 줄이고, 첫 칸(밀기)의
# 키를 예전 달리기 칸과 같게(외곽선 안 18) 맞춘다. 칸 8~11 — 게임은 이 칸이 있으면 달릴 때 돈다(`Art::critter_frames`).
RUN_NAMES = ["run_a", "run_b4", "run_c", "run_d"]
RUN_SPEC = Spec(
    sources=[
        Source(
            HERE / "sources" / "ai" / "sprites" / "doto_run_v1.png",
            dict(zip(RUN_NAMES, [(9, 259, 299, 416), (260, 165, 537, 339), (553, 196, 783, 416), (799, 204, 1008, 416)])),
            groups=dict.fromkeys(RUN_NAMES, "run_a"),
            tidy=0.15,
            strip=8,
        )
    ],
    palette_ref=(0, "run_a"),
    body_h=18,
)
ORDER = ["idle_a", "idle_b", "run", "run_b", "aim", "throw", "hurt", "down"] + RUN_NAMES

# 물든 도토(0052): 초상화 doto_tainted_glare의 회보라 털, 호박색으로 빛나는 눈. 모자·도토리는 그대로.
TAINT_FUR = [(58, 48, 62), (86, 74, 90), (114, 100, 116), (146, 132, 146), (196, 186, 194)]
TAINT_OUTLINE = (26, 16, 38)
EYE_AMBER = [(130, 64, 12), (232, 142, 32), (255, 214, 96)]




def taint(cell):
    """돌아온 도토 칸 → 물든 도토 칸. 눈 = 빛점(흰 점) 둘레 2칸 안의 어두운 점."""
    out = cell.copy()
    rgb = cell[..., :3].astype(int)
    on = cell[..., 3] > 0
    white = on & (rgb.sum(-1) > 700)
    near = white.copy()
    for _ in range(2):
        g = near.copy()
        g[1:] |= near[:-1]
        g[:-1] |= near[1:]
        g[:, 1:] |= near[:, :-1]
        g[:, :-1] |= near[:, 1:]
        near = g
    for y, x in zip(*np.nonzero(on)):
        key = tuple(int(v) for v in rgb[y, x])
        L = lum(key) / 255
        if key == OUTLINE:
            c = TAINT_OUTLINE
        elif white[y, x]:
            c = key
        elif near[y, x] and L < 0.3:                                     # 눈
            c = ramp(EYE_AMBER, L * 3)
        else:
            h, l, s = colorsys.rgb_to_hls(*(v / 255 for v in key))
            if 0.075 <= h <= 0.16 and s < 0.62 and L < 0.75:            # 깍정이 모자·도토리(올리브 갈색)
                c = key
            else:                                                        # 털·배·귓속·볼
                c = ramp(TAINT_FUR, L * 1.15)
        out[y, x, :3] = c
    return out


def build():
    spr = convert(SPEC)
    cells = {k: seat(spr[k], CELL, FOOT) for k in NAMES}
    cells["run_b"] = seat(spr["run"], CELL, FOOT, dy=-1)     # 통통: 같은 그림을 1px 띄운다(걷기에 쓴다)
    run = convert(RUN_SPEC)
    for k in RUN_NAMES:
        # 공중 칸은 발 줄에서 3px 띄운다(뛰는 높이).
        cells[k] = seat(run[k], CELL, FOOT, dy=-3 if k == "run_b4" else 0)
    return sheet4([(cells[k], taint(cells[k])) for k in ORDER])


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/art/doto.png"
    Image.fromarray(build(), "RGBA").save(out)
