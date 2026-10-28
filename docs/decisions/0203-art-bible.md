# 0203. 그림 규칙(art bible): 칸·비율·팔레트·외곽선·빛·값 위계·동작 목록·생성 파이프라인

- 날짜: 2026-10-28
- 상태: 채택(2026-10-28, 사장 "응 다 진행해도 되겠음" — 아래 "사람이 정할 것" 네 가지 모두 제안대로). 0191 C1. C2(도구)는 아래 "C2 구현", 다음은 C3(Naïl 기준 시트).
- 사장: "0.5의 b 진행하고" → B 끝, C1 차례.

## 왜 숫자로 정하나
- 여러 번에 걸쳐 만든 그림이 한 게임처럼 보이려면 칸·팔레트·빛을 **숫자로** 고정해야 한다. 이미지 모델은 글만 주면 흔한 그림으로 채우고(KB `image-model-for-game-illustrations`), AI "도트"는 격자가 어긋나고 중간색·부드러운 가장자리가 섞여 그대로는 게임에 못 쓴다(2차: [How to tame your AI pixel art](https://dev.to/jenissimo/how-to-tame-your-ai-pixel-art-3pk5), [Sprite Fusion Pixel Snapper](https://www.gigazine.net/gsc_news/en/20260516-spritefusion-pixel-snapper)).
- 도트 스타일 가이드가 보통 담는 것: 해상도·팔레트·외곽선·명암·비율·동작 박자(2차: [2D pixel art style guide](https://www.sprite-ai.art/blog/2d-pixel-art-style-guide), 협업 도트 명세 예 [LPC Style Guide](https://lpc.opengameart.org/static/LPC-Style-Guide/build/styleguide.html)). 아래는 그 틀에 이 게임의 지금 값을 채운 것.

## 1. 칸과 비율
| 무엇 | 칸 | 몸 | 지금 |
|---|---|---|---|
| 플레이어(Naïl·모모·도토)·이웃·잡몹 | **16×16** (→ 새 그림: Naïl 24·이웃 32·잡몹 24, 0221) | 키 12~14px, 발바닥 줄 y = 14(그림자 타원은 발 + 6) | 같다(코드 그림) |
| 이름 있는 보스(모모·도토·콩이·토끼풀) | **24×24** (채택) | 키 18~20px | 지금 16×16 — 잡몹과 크기가 같아 "주인"으로 덜 읽힌다. 새 그림부터 24칸(발바닥 줄 y = 21) |
| 큰 보스(대장) | 32×32 | 키 26~28px | 같다(`chief.py`) |
| 타일·소품 | 16×16 | — | Sunnyside 팩(바꾸지 않음, C5는 선택) |

- **비율은 2등신**(머리 ≈ 몸 키의 절반): 16칸에서 표정이 남는 유일한 크기. 큰 눈 2×2 + 반사 1px, 동그란 코, 볼 홍조 1px, **입은 그리지 않는다**(KB `pixel-art-without-an-artist` "16×16 얼굴").
- 모두 **오른쪽을 본** 그림 하나(왼쪽은 뒤집기). 3/4 탑다운(0006): 등과 머리 윗면이 조금 보인다.

## 2. 팔레트
- 재질마다 **3단 + 반사 1**(밝음·바탕·그림자 + 가장 밝은 점). 그림자 단은 보라 쪽으로, 밝음 단은 노랑 쪽으로 색조를 옮긴다(hue shift — 같은 색을 검게만 하면 탁해진다).
- 캐릭터 하나에 **12색 안**(외곽선 포함). 지금 공통 색(`critters.py COMMON`: 분홍 코·발, 눈, 뿌리 보라, 흰 줄무늬)은 모든 시트가 같은 값을 쓴다.
- 팩 그림은 AI 참고로 쓰지 않는다(0064) — 팔레트는 우리 코드 그림·초상화에서 뽑는다.

## 3. 외곽선
- **어두운 외곽선 1px을 실루엣 둘레에**(안쪽 선은 그리지 않거나 재질의 가장 어두운 단). 지금 코드 그림이 이 방식이고 무늬가 잔 Sunnyside 바닥에서 몸이 읽힌다. 선택 외곽선(sel-out)은 더 부드럽지만 바닥에 녹을 수 있다(이 게임에서 시험은 안 함).
- 색: 플레이어·이웃 = 따뜻한 자두 `(43, 29, 42)`, 물든 적 = 더 어둡고 차가운 `(26, 16, 38)`(지금 규칙 그대로).

## 4. 빛
- **왼쪽 위 하나**(초상화 코드 그림의 빛과 같다, KB "대화창 초상화"). 땅에 드리우는 그림자 방향(해·모닥불, `shade`)과는 따로다 — 몸의 명암만. 빛 받는 쪽 실루엣에 가장 어두운 단이 세로로 서지 않게(`pxl.py check`).
- 16칸에서는 **디더 없음**(단 경계를 바둑판으로 섞으면 잡음이 된다). 24·32칸만 경계 한 줄 2×2 디더 허용.

## 5. 값(밝기) 위계 — 무리 속에서 내가 먼저
1. 플레이어: 가장 따뜻하고 밝다.
2. 이웃: 중간.
3. 물든 적: 종 색을 보라 쪽 28% + 밝기 82%(`TAINT_MIX`·`TAINT_VALUE`), 보라 뿌리 무늬, 붉은 눈.
- 확인: 게임 화면을 흑백으로(`tools/a11y/cvd.py`) 돌려 플레이어가 가장 밝은 덩어리로 남는지(KB "무리 속에서 내 캐릭터가 보이게").

## 6. 동작 목록(시트의 줄 = 동작, 칸 = 프레임)
| 줄 | 동작 | 칸 | 박자 |
|---|---|---|---|
| 0 | 서기(숨쉬기) | 2 | 0.5초씩 |
| 1 | 걷기 | 4 | 0.1초씩 |
| 2 | 평타 셋: 들기·치기 ×3(마지막 타는 두 발·크게) + 거두기 | 7 | 무브셋 시간표 |
| 3 | 강타: 모으기(웅크림)·휘두르기·거두기 | 3 | 모으기는 멈춘 칸 |
| 4 | 회피(Naïl 파고들기·모모 깡충·도토 뒤로 뛰기): 시작·공중(또는 땅속)·착지 | 3 | |
| 5 | 특기(드릴·이어 뛰기·묻기) | 2~3 | |
| 6 | 맞음·쓰러짐 | 2 | |
| 7 | 먹기 | 2 | |
- 흰 실루엣 줄(맞음 번쩍·다 모인 몸빛 0202)은 손으로 그리지 않고 **도구가 만든다**(실루엣을 흰색으로).
- 잡몹은 서기 2·걷기 4·예고(웅크림) 1·공격 1·맞음 1·쓰러짐 1(지금 칸 수 그대로), 보스는 + 수마다 예고 1.

## 7. 생성 파이프라인(C2에서 도구로 만든다)
1. **기준 시트를 먼저 하나**: Naïl의 서기·걷기 시트를 사장이 나노바나나로 그린다 → 확정. 그다음 시트마다 이 확정본을 참고 그림으로 붙인다(같은 캐릭터 반복은 앞선 그림을 다시 넣는 것이 공식 권장, KB).
2. **프롬프트 틀**: "16×16 픽셀 아트 스프라이트 시트, 한 칸 = 128×128 정사각(픽셀 하나 = 8×8 블록), N열 × M줄 격자, 단색 배경(#FF00FF), 격자선 없음, 그림끼리 떨어뜨림, 오른쪽을 봄, 왼쪽 위 빛, 어두운 자두색 1px 외곽선, 2등신, 입 없음, 그림자 없음" + 캐릭터 생김새 한두 문장(틀리기 쉬운 것: 털빛·귀·꼬리·옷).
3. **격자 맞추기**(`pxl.py snap`, 아래 "C2 구현"): 그림 덩어리를 찾고, 블록 크기·어긋남을 재서 16칸으로 줄이고(블록 가운데 색), 팔레트(위 2)로 묶는다. 외톨이 정리는 하지 않는다(16칸의 눈은 한 점이다) — 손질에서.
4. **손질**(`.pxl` 원본, 브라우저 편집기): 얼굴은 손으로(자동으로는 뭉개진다), `pxl.py check`(계단·외톨이·빛 방향).
5. **한 줄 검사**(C2 시험): 칸마다 몸 키·발바닥 줄이 같은지, 팔레트 밖 색이 없는지.
6. 게임에서 확인: 원본 1배·8배·게임 화면 셋, 흑백·색약 화면.
- 참고 그림은 **우리가 만든 것만**(코드 그림·나노바나나 초상화). 게임 화면 캡처에는 팩 타일이 섞이니 넣지 않는다(0064).

## 순서(0191 C)
C1 확인 → C2 도구(`pxl.py snap` + 한 줄 검사) → C3 Naïl 기준 시트 → 모모·도토 → C4 잡몹 열·보스 다섯·이웃.

## 사람이 정할 것 → 정함(2026-10-28, 모두 제안대로)
1. 이름 있는 보스 **24×24**(16×16 아님).
2. **어두운 외곽선 1px**(선택 외곽선 아님).
3. **2등신**(2.5등신 아님).
4. **Naïl 서기·걷기 한 장을 먼저** 시험으로 — 프롬프트는 사장에게 드린 것(아래 "C3 첫 시트").

## C2 구현(2026-10-28): `pxl.py snap`
나노바나나 "도트"는 격자가 어긋나고(블록이 7.8px처럼 정수가 아니거나 그림마다 밀림), 가장자리가 번지고, 중간색이 섞인다. 그대로 줄이면 한 점씩 밀리거나 흐린 색이 남는다(2차: [How to tame your AI pixel art](https://dev.to/jenissimo/how-to-tame-your-ai-pixel-art-3pk5) — 격자 찾기 → 블록 가운데 → 팔레트 묶기, [Sprite Fusion Pixel Snapper](https://www.gigazine.net/gsc_news/en/20260516-spritefusion-pixel-snapper) 같은 방식). 그래서:

```sh
python3 tools/art/strip_logo.py tools/art/sources/ai/sprites/nail_idle_walk.png     # ✦ 로고를 먼저(0133) — 안 지우면 그림 하나로 잡힌다
python3 tools/pixel/pxl.py snap tools/art/sources/ai/sprites/nail_idle_walk.png 새.pxl --find --palette assets/art/nail.png --preview 미리.png
python3 tools/pixel/test_snap.py                                                    # 도구를 고치면 왕복 시험
```
- **`--find`(기본으로 쓸 것)**: 배경(네 귀퉁이 색 또는 `--bg`)이 아닌 점이 이어진 덩어리를 그림 하나로 찾고, 덩어리마다 가장자리 자리의 "원 위 평균"이 가장 잘 모이는 블록 크기를 잰다(s/2도 모이고 2s부터 흩어지므로 잘 모이는 것 중 가장 큰 s, 덩어리마다 어긋남이 달라도 된다, `--block`으로 줄 수도). 덩어리마다 위상(어긋남)을 재고 블록 가운데 반쪽의 밝기 가운데 값을 찍는다. 그림은 칸(`--cell`, 기본 16)에 **발바닥 줄 `--foot`(기본 14)·가운데로** 앉힌다. 줄은 위아래가 겹치는 덩어리끼리, 왼쪽부터.
- **`--grid 4x2`**: 시트를 똑같이 나눌 때(블록 = 칸 폭 / 16). 그림이 칸 끝에 닿으면 밀린 블록이 번짐에 진다 — 칸 가장자리에서 1px 띄울 것.
- 색: `--palette 그림.png`면 그 색 가운데 가까운 것으로(지금 그림과 색을 맞출 때), 아니면 시트 전체를 `--colors`(기본 12)색으로 합친다(redmean).
- **한 줄 검사**(위 5): 줄(동작)마다 칸끼리 발바닥 줄(`--foot-tol`, 걷기 들썩임 1)·몸 키(`--tall-tol` 1)가 다르면 "← 칸마다 다르다", `--strict`면 1로 끝난다. 칸을 넘는 그림은 "잘린다"고 알린다(블록을 잘못 쟀거나 그림이 너무 크다).
- 시험(`test_snap.py`, 직접 실행): `nail.png` 여덟 칸을 블록 확대(8px·7.8·8.3·8.8, `--find`는 7.7·8.0·9.3px)·어긋남 0~6px·흐림 1.3·잡음 ±12·마젠타 배경·잡티로 망가뜨려 다시 맞춘다. 2048점 중 틀린 점 2~12, 남는 것은 거의 다 **1px 눈**(번짐에 섞여 아주 가까운 외곽선 색으로 간다) — 얼굴은 위 4대로 손으로.
- 한계: 그림 둘이 블록 한두 개 거리로 붙어 있으면 한 덩어리로 잡힌다(프롬프트에 "그림끼리 떨어뜨림"). 디더·반투명 그림자는 가운데 값으로 뭉개진다(16칸은 디더 없음, 위 4). 실제 나노바나나 시트로는 아직 안 돌려 봤다 — C3 첫 시트에서 확인.

## C3 첫 시트(사장에게 드린 프롬프트)
첨부: 지금 Naïl 코드 그림 첫 줄(서기 2 + 걷기 4)을 틈을 두고 마젠타 바탕에 8배로 키운 것(Naïl은 칸 16열을 다 써서 틈이 없으면 `--find`가 한 덩어리로 잡는다, 우리 그림이라 0064에 걸리지 않는다, 만들기는 아래) + 초상화 `tools/art/sources/ai/nail_normal.png`(생김새). 비율 16:9. 받은 원본은 `tools/art/sources/ai/sprites/nail_idle_walk.png`로.
```sh
python3 -c "
from PIL import Image
a = Image.open('assets/art/nail.png').convert('RGBA'); b = Image.new('RGBA', (6 * 24, 24), (255, 0, 255, 255))
for i in range(6): b.alpha_composite(a.crop((i * 16, 0, i * 16 + 16, 16)), (4 + i * 24, 4))   # 칸 사이 8px 틈
b.resize((b.width * 8, b.height * 8), Image.NEAREST).save('nail_ref_x8.png')"
```
> Pixel art sprite sheet of the same mole character as the attached images (the small sprite strip shows pose and proportions, the portrait shows the look). 16×16-pixel sprites drawn big: one game pixel is a hard square 8×8 block — no anti-aliasing, no gradients, no dithering, no soft edges. Flat solid magenta background (#FF00FF), no grid lines, leave clear empty space around every sprite. Top row: idle breathing, 2 frames. Bottom row: walk cycle, 4 frames. Facing right, 3/4 top-down view (top of head and back slightly visible). Light from the top left. Dark plum 1-pixel outline around the silhouette (#2B1D2A). Chibi 2-head proportions, big 2×2 eyes with one white highlight pixel, round pink nose, one blush pixel, no mouth. Feet on the same line in every frame, same body size in every frame. No ground shadow, no text. At most 12 colors.

## C3 첫 시트 결과(2026-10-28, `tools/art/sources/ai/sprites/nail_idle_walk_v1.png`, ✦는 `strip_logo.py`로 지움 n = 1.08)
- **생김새는 맞다**: 둥근 털뭉치 몸·주황 뜨개 목도리와 매듭·분홍 큰 앞발과 크림 발톱·분홍 코·보라 눈 — 초상화(`nail_normal.png`)와 같은 캐릭터다. 지금 게임 스프라이트(길쭉한 옆모습 두더지)는 초상화와 처음부터 달랐다.
- **크기는 안 지켰다**: "16×16, 픽셀 하나 = 8×8 블록"이라 했는데 그림 하나가 약 55~60 게임 픽셀 높이(블록 약 3.7px)로 그렸다. 받은 판(1024×432)은 줄여 저장된 것이라 격자도 남지 않았다(가장자리 원 위 평균이 모든 크기에서 0.06 아래 — 블록 크기를 잴 수 없다). `snap --find`가 36.9px을 블록으로 잡았다.
- **16칸으로 억지로 줄이면 못 쓴다**: 블록 15px로 `snap`(블록 가운데) / 면적 평균 + 12색 둘 다 외곽선이 없어지고 얼굴이 사라지고 마젠타 번짐이 테두리에 남았다(직접 비교).
- 그 밖에: 첫 칸(위·아래)은 정면에 입을 그렸다(나머지는 3/4 오른쪽), 위 줄은 2칸이 아니라 3칸, 위아래 줄 그림이 맞닿아(틈 0) 한 덩어리로 잡혔다.
- **손으로 16칸에 옮기면 읽힌다**: 이 시트를 보고 서기 한 칸을 `.pxl`로 찍은 시안(16색, 발바닥 y14)은 게임 크기(1배)에서 "둥근 털뭉치 + 주황 목도리 + 분홍 발"로 읽혔다. 얼굴은 2×2 눈 + 먼 눈 1×2 + 볼 + 코 2칸.
- 시안 원본: `tools/pixel/drafts/nail_idle_new.pxl`(게임에는 아직 안 넣었다).
- 그래서 제안(사장 결정 전): 나노바나나 시트는 **생김새·자세 참고**로 쓰고 16칸 프레임은 손으로(`.pxl`) 찍는다. `snap`은 진짜 격자로 그린 그림(가끔 나오는 작은 판, 다른 도구)에만.

## C3 Naïl 새 옷(2026-10-28, 사장 "응 손으로 찍어봐")
- 새 생김새(둥근 털뭉치 + 주황 목도리)로 갈아입히고 16칸 프레임은 손으로. 서기·걷기만 바꾸면 치기 프레임이 옛 옆모습으로 남아 따로 놀아서 **시트 전체(줄 0·1, 15칸)**를 새로 찍었다.
- 원본은 그대로 `tools/art/nail.py`(채움 글자 + 자동 외곽선, 시트 배치·털빛 셋·흰 실루엣 줄 그대로 — 게임 코드는 안 바뀐다). 새 `BASE` 하나 + 프레임마다 손으로 고친 자리(`put`). 옛 길쭉한 옆모습은 `tools/art/legacy/nail_v1.py`.
- 팔레트: 털 넷(털빛 저널 보상이 바꾸는 d·b·h·l)·분홍 둘·발톱·눈 둘·볼 + 목도리 둘(`f`·`g`, 털빛과 상관없이 그대로) + 외곽선.
- 자리: 털 끝 1줄, 몸 2~12줄, 발 13줄(외곽선 14 = 발바닥 줄). 3/4라 먼 눈(7열)이 보인다. 가까운 눈 2×2 + 빛, 볼, 코 2칸, 입 없음. 목도리 8~9줄 + 매듭. 앞발 13~14열(치기·들기·거두기가 이 발을 옮긴다), 뒷발 1~2열.
- 게임에서 확인(헤드리스, 꽃 핀 입구·적 다섯 종): 1배에서 둥근 몸·주황 목도리로 바로 읽힌다. 치기 프레임은 앞발이 짧아 덜 보인다(앞으로 나가는 맛은 전진 lunge가 낸다) — 사장 판정 뒤 다듬기.
- 눈 고침(사장 "눈이 좀 별로"): 첫 판은 가까운 눈(11~12열)이 코(13열)에 붙어 눈·코가 한 덩어리로 번졌고, 먼 눈은 1×2 세로 막대라 찢어진 눈으로 읽혔다. 두 눈 다 2×2 + 왼쪽 위 빛(6~7·10~11열), 코는 한 줄 내려 눈과 한 칸 띄웠다. 보라 아랫단(2×3)은 눈 밑 그늘로 읽혀 뺐다. 질끈·> 눈·× 눈 자리도 같이 옮겼다.
- 아직: 땅 단면 전환의 32px 파 내려가기 그림(`dig.py` 줄 0~3)은 옛 옆모습이다. 사이트 걷는 띠·파비콘은 `make_images.py`를 다음 배포 때 돌리면 따라온다.

## C3 모모 시트(2026-10-28, 사장 "다음 캐릭터 프롬프트")
나일 첫 시트에서 배운 것(위 "C3 첫 시트 결과"): 이미지 모델은 16칸 크기를 안 지키고, 정면 칸·입을 섞고, 그림을 붙여 그린다. 시트는 이제 **생김새·자세 참고**라(16칸은 손으로) 크기는 맞추라 하지 않고 "같은 크기·3/4 오른쪽만·띄워서·입 없음"을 세게 적는다. 칸은 게임의 모모 칸 순서 그대로(`critters.py` 줄 8 돌아온 모모: 서기 2·깡충 2·웅크림·뛰어 차기·맞음).
첨부 셋: 캐릭터 시트 `tools/art/sources/ai/sheets/momo.png`(앞·3/4), 초상화 `tools/art/sources/ai/momo_normal.png`, 지금 모모 칸 일곱을 틈을 두고 8배로 키운 띠(자세 순서, 우리 그림).
> Pixel art sprite sheet of the same rabbit character as the attached images (the character sheet and portrait show the look; the small sprite strip shows the seven poses in order). Clean retro pixel art with hard square pixels, no anti-aliasing, no gradients, no dithering. Flat solid magenta background (#FF00FF), no grid lines, no text, no shadows on the ground, and a wide empty gap between every sprite so no two sprites touch. Every frame is the SAME character at the SAME size, in a 3/4 view facing right — never a front view. Chibi 2-head proportions: big round head, long upright ears, a bloomed pink flower at the base of one ear, warm grey-brown fur, cream belly and muzzle, white cotton tail, big glossy black eyes with one white highlight, small pink nose, pink blush, no mouth. Dark plum outline around the silhouette (#2B1D2A), light from the top left. Top row, 4 frames: idle breathing (2 frames, ears slightly lowered in the second), hop (2 frames: crouch-off and mid-air). Bottom row, 3 frames: crouch wind-up (body low, ears back), flying kick with both hind legs stretched forward, hurt (flinching, eyes squeezed shut). Feet on the same baseline in every grounded frame.
- 받으면 `tools/art/sources/ai/sprites/momo_v1.png`(✦는 `strip_logo.py`) → 나일처럼 `critters.py` 돌아온 모모 줄을 손으로(16칸, 0203 1~6).
