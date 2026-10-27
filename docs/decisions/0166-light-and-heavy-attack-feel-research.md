# 0166. 평타·모으기 강타의 손맛 — 리서치와 제안(사장 확인 전)

- 날짜: 2026-10-27
- 상태: **제안**. 사장(v0.3.7 판정 중): "손맛이 좀 애매해. 땅파기·필살기는 괜찮아. 근데 평타랑 모으기 강타가 좀 맛이 없어. 리서치 좀 해볼래?"
- 바탕: KB `concepts/fixed-timestep-and-hitstop.md`·`special-move-impact.md`·`game-feel-particles-and-light.md`, 지금 코드의 수치(`src/tuning.rs`·`src/moveset.rs`·`src/player.rs`·`src/world.rs`), 바깥 자료(아래 출처, 모두 2차). 손맛은 사람이 해 봐야 아니 아래 숫자는 전부 가설이다.

## 1. 지금 평타·강타가 하는 일(코드에서 잰 것)

| | 1타 | 2타 | 3타(마무리) | 강공격(`heavy_of`) |
|---|---|---|---|---|
| 입력 → 스윙 | **떼는 순간**(0161) | 활성 뒤 선입력 | 활성 뒤 선입력 | 0.45초 모으고 떼면 |
| 들기(예비) | 0.02초(1f) | 0.02 | 0.05(3f) | 0.07(4f) |
| 활성 | 0.02~0.08 | 같음 | 0.05~0.14 | 0.07~0.19 |
| 스윙+후딜 | 0.12+0 | 0.12+0 | 0.20+0.14 | 0.26+0.22 (+잠금 0.3) |
| 피해 | 1 | 1 | 2 | **2** |
| 넉백 → 밀리는 거리(÷마찰 9) | 90 → ≈10px | 10px | 330 → ≈37px | 380 → ≈42px |
| 히트스톱 | 0.05(3f) | 3f | 0.13(8f) | 0.16(10f) |
| 흔들림 trauma → 실제 px(4·t²) | 0.15 → 0.09 → **0px** | 0px | 0.55 → 1.2 → 1px | 0.65 → 1.7 → 2px |
| 맞힌 쪽 표시 | 흰 고리 + 불똥 4 + 흰 번쩍 0.08초 + impactPunch | 같음 | + 충격 고리·쿵 | 같음(마무리 취급) |
| 공격자 쪽 반응 | 없음(전진 45는 맞기 전) | 없음 | 없음 | 없음 |
| 피격자 몸짓 | 흰 실루엣만(자세 없음) | 같음 | 같음 | 같음 |

콤보 3타 = 피해 4를 0.58초에. 강공격 = 피해 2를 모으기 0.45 + 스윙·후딜 0.48 + 잠금 0.3 = **1.2초**에.

## 2. 왜 "맛이 없나" — 원인 다섯

### 평타
1. **누르고 떼야 나간다(0161의 비용).** 강공격을 "1타부터" 되게 하려고 평타를 떼는 순간으로 옮겼다. 사람의 탭은 보통 0.05~0.1초라 모든 1타가 그만큼 늦게 나간다. 바깥 사례가 똑같다: INVERSUS는 모으기 쏘기를 넣자 기본 쏘기가 떼는 순간에 묶여 둔해졌고 그걸 줄이려 따로 글을 썼다(2차). Project Ghost Knight는 결국 "공격은 누르는 순간, 모으기는 평타 뒤"로 바꿨다(2차). 모으기 입력을 다룬 글들의 공통 결론: **기본 공격은 누르는 순간 나가야 한다.**
2. **1·2타의 피드백이 거의 0이다.** 히트스톱 3프레임, 10px 밀림, 흔들림 **0px**(trauma²라 작은 값은 반올림에서 사라진다 — 설계상 1·2타는 절대 흔들리지 않는다), 흰 번쩍 5프레임. 스매시브라더스의 히트랙 공식(2차, SmashWiki)은 가장 약한 공격도 **6~7프레임**(Ultimate: 0.65×피해% + 6)을 준다. 사쿠라이(2차, Famitsu 칼럼·YouTube 요약): 멈춤 자체보다 **멈추는 동안 맞은 쪽을 흔드는 것**, 공격자는 살짝 계속 움직이는 것이 핵심인데 우리는 둘 다 없다. 맞은 적은 흰색으로만 바뀌고 자세·찌그러짐·진동이 없다.
3. **맞혔다는 확인이 공격자에게 없다.** 때린 쪽은 아무 일도 없다. Hollow Knight는 칼이 닿으면 기사가 뒤로 튕기고(Steady Body 참이 이걸 끄는 참이다, 2차), Nuclear Throne은 쏠 때 카메라를 조준 쪽으로 6px 차고 되돌린다(Nijman 글, 2차). 맞힘 = 빗나감의 차이가 피격자 쪽 표시에만 있으면 손에는 안 온다.

### 모으기 강타
4. **보상이 숫자로 없다.** 강공격 피해 2 = 마무리 타와 같다. 같은 1.2초면 콤보로 피해 4가 나간다. 강공격의 이득은 범위 ×1.3, 넉백 ×1.15, 진홍 고리 받아치기뿐. KB `special-move-impact.md`의 함정 그대로 "조금 센 평타"다. 기준(2차, 기억·위키): Hollow Knight Great Slash = 칼 피해 ×2.5, 스매시의 모으기 스매시 = 최대 ×1.4 + 긴 경직, 소울류 모으기 강공격 = 포이즈 깨기(자세 무너뜨림). 모으기는 **시간을 내준 만큼 상대의 상태를 바꿔야**(넘어뜨림·자세 깨기·벽 쿵) 쓴다.
5. **모으는 0.45초가 비어 있다.** 발톱 끝 빛이 2→5px로 커지고 다 차면 떨림·부채꼴이 뜨지만, 그 전까지 몸·소리·입자가 없다. "힘이 모인다"는 몸(웅크림)·소리(올라가는 음)·입자(빨려 드는 먼지)로 읽힌다. 그리고 떼서 맞혔을 때의 임팩트(10f·2px)가 마무리 타(8f·1px)와 거의 같다 — 세 손잡이(시간·공간·감각)가 다 비슷하니 같은 것으로 느껴진다.

## 3. 제안(수치는 `tuning.rs`·`moveset.rs`, 전부 가설)

### A. 평타는 누르는 순간 — 작음
- `State::Free`에서 공격을 누르면 **바로** `start_swing`(0161 전으로). 누르고 있으면 활성 구간 뒤에 모으기(`Charging{pending:false}`)로 — 이 흐름은 이미 있다.
- 잃는 것: "1타 자리의 강공격"(처음부터 누르고 있으면 1타가 먼저 나간다). 얻는 것: 모든 1타의 0.05~0.1초 지연이 사라진다. 바깥 사례 둘 다 이 맞바꿈을 택했다.
- 대안(1타 강공격을 꼭 지키려면): 누르는 순간 **들기 자세·전진(lunge)** 을 먼저 시작해 두고 떼면 치기 — 눈이 먼저 반응하니 체감 지연이 준다(INVERSUS식). 코드가 더 복잡하고 "눌렀는데 안 나감"이 남는다. 추천하지 않는다.

### B. 작은 타격의 피드백 층 — 중간
1. **히트스톱**: 1·2타 0.05 → 0.08(5f), 3타 0.13 → 0.15(9f), 강공격 0.16 → 0.22(13f). 피해·세기에 비례하는 사다리(스매시 공식의 뜻).
2. **피격자 진동·찌그러짐**(사쿠라이 1번): 히트스톱 동안 맞은 적의 그림을 맞은 축으로 ±1px 번갈아(렌더에서 `feel.hitstop > 0 && e.flash > 0`이면 `((frame/2)%2)*2-1`), 첫 2프레임은 맞은 방향으로 x 0.85·y 1.15 찌그림. 상태 없음, 렌더만.
3. **공격자 확인**: 맞히는 순간 플레이어 전진 속도를 0으로(`vel = 0`) + **카메라 킥**: 맞은 방향으로 1·2타 2px, 3타 3px, 강공격 4px를 차고 0.1초에 되돌린다(`Feel::kick: Vec2`, 흔들림 오프셋에 더함). trauma²가 못 담는 작은 타격을 킥이 맡는다.
4. **피격 자세 한 칸**: 흰 실루엣 1프레임 → 맞은 자세(옆으로 기운 칸) 2프레임 → 평소(Nuclear Throne 규칙, 2차). 잡몹 스프라이트에 "맞음" 칸 하나(`critters.py`).
5. **소리**: 맞힘 소리를 두 층(impactPunch + 짧은 높은 '틱')으로, 피치 ±6%, 맞히면 스윙 소리는 덕킹. 들어 봐야 안다.

### C. 강타의 보상과 모으는 느낌 — 중간
1. **피해 2 → 3**(마무리 타 ×1.5), 경직 0.6 → 1.0, 넉백 ×1.15 → ×1.7(≈62px). 보스도 `CHIEF_STUN_MUL` 밖에서 한 번은 휘청(짧은 Stunned).
2. **벽 쿵**: 강공격 넉백으로 벽에 박히면 +1 피해 + 돌 파편 + 긴 히트스톱(드릴의 `DRILL_WALL_*` 재사용). 넉백을 키운 이유가 생긴다.
3. **콩이 등딱지**: 강공격은 튕기지 않고 자세를 깬다(`break_guard`) — 소울류 "모으기 = 자세 깨기".
4. **모으는 동안**: 시작부터 올라가는 저음(지금 `Charge`는 완료 때만) → 완료에 '칭', 몸 1px 웅크림, 발 쪽으로 빨려 드는 먼지 2~3개, 완료 순간 흰 번쩍 0.03초.
5. **떼서 맞혔을 때**: 히트스톱 13f + 피격자 진동 2px + 카메라 킥 4px + 흰 번쩍 0.04초(`feel.flash`, 번쩍 설정 따름).

### 순서 추천
A → B(1·2·3 먼저, 4·5는 그 뒤) → C. A·B가 평타, C가 강타. 전부 상수와 렌더라 되돌리기 쉽다. 하나씩 넣고 사장이 판정한다 — 한꺼번에 넣으면 무엇이 효과였는지 모른다.

## 4. 확인 못 한 것
- 바깥 자료의 원문(Famitsu 칼럼 번역·INVERSUS 글·slynyrd 글)은 이 세션의 프록시에서 열리지 않아 검색 요약으로만 읽었다(2차). Hollow Knight Great Slash ×2.5는 기억이다.
- "탭 0.05~0.1초"는 0161에 적은 추정이지 잰 값이 아니다. 개발 빌드에 "누름→뗌" 시간을 콘솔에 찍으면 잴 수 있다.
- 어느 것이 실제로 "맛"을 바꾸는지는 A·B·C를 따로 넣어 사장이 해 봐야 안다.

## 출처(2차)
- SmashWiki, Hitlag: https://ssbwiki.com/Hitlag
- NintendoWire, This Week in Sakurai (hit stop 여덟 가지): https://nintendowire.com/news/2022/12/12/this-week-in-sakurai-12-5-12-11-fine-tuning-hit-stop-and-cheating-the-system/
- Source Gaming, Sakurai Famitsu 칼럼 "Thinking About Hitstop": https://sourcegaming.info/2015/11/11/thoughts-on-hitstop-sakurais-famitsu-column-vol-490-1/
- Game Developer, Reducing Perceived Attack Delay in INVERSUS: https://www.gamedeveloper.com/design/reducing-perceived-attack-delay-in-inversus
- Project Ghost Knight, New attack input system: https://www.moddb.com/games/project-ghost-knight/news/new-attack-input-system
- Kalia devlog, charged attack input design: https://gdn001.itch.io/kalia-and-the-fire-staff/devlog/525674/kalia-and-the-charged-attack-and-why-input-design-is-important
- DK Liao, The Art of Screenshake 재현(효과 목록): https://dkliao.itch.io/the-art-of-screenshake-recreation/devlog/451576/quick-breakdown-of-all-the-effects
- Hollow Knight wiki, Steady Body(공격자 반동)·Heavy Blow: https://hollowknight.wiki/w/Heavy_Blow
- slynyrd, Pixelblog 9 Melee Attacks(예비·스미어·따라가기): https://www.slynyrd.com/blog/2018/9/8/pixelblog-9-melee-attacks
- wayline, Level up indie game melee combat: https://www.wayline.io/blog/level-up-indie-game-melee-combat
