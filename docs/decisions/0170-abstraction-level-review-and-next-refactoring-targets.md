# 0170. 추상화 수준 판정과 다음 리팩터링 목표(R9~R13)

- 날짜: 2026-10-27
- 상태: R9·R-소·R11·R10·R12·**R14(캐릭터 표, 사장 추가)** 구현(2026-10-27, 사장: "보스표 및 플레이어블 캐릭터 표까지 포함해서 다음 단계 진행해"). R13은 미룸(다섯째 장애물을 더할 때). 결과는 맨 아래 "구현" 절. 처음은 제안(사장: "좋은 추상화 수준을 가졌는지 판단해서, 다음 리팩터링 목표를 설정하자"). 0169 R1~R8 뒤의 코드를 잰 것.
- 잰 것: `src/` Rust 46,272줄, 함수 1,138개(100줄 넘는 50, 200줄 넘는 9), 테스트 158개. 수치는 전부 이 커밋에서 `wc`·`grep`·`awk`·짧은 파이썬으로 잰 것(1차).

## 1. 판정 기준
"좋은 추상화"를 셋으로 잰다.
1. **깊이**(Ousterhout, *A Philosophy of Software Design*): 인터페이스는 작고 안에 든 것은 많은가. 반대는 "인터페이스 = 필드 전부"인 얕은 모듈 — 밖에서 필드를 직접 바꾸면 그 모듈은 제 불변식을 못 지킨다.
2. **방향**: 아래층(로직)이 위층(앱·렌더)을 보지 않는가. 어떤 사실의 **진실은 한 곳**인가.
3. **셋의 규칙**(Sandi Metz, "중복이 잘못된 추상화보다 싸다"): 같은 모양이 셋 넘게 나란히 있으면 표·함수로 모으고, 둘뿐이면 두지 않는다. 플래그·모드 인자가 생기면 잘못된 추상화의 신호.

## 2. 한 줄 판정
**경계와 방향은 좋다. 모듈의 깊이가 고르지 않다.** 입력(`Actions`)·글(`Tr`)·소리(`sfx::Queue`)·이벤트(`Event`·`Cue`·`FxKind`)·표(무브셋·종·장비·재주·저널·시나리오)는 깊다 — 로직은 장치도 언어도 메뉴도 모른다. 반면 `World`와 `Enemy`는 얕다: `World`의 인터페이스는 사실상 필드 전부이고(앱이 77가지를 읽고 15가지를 **쓴다**), 적이 맞았을 때 무슨 일이 나는지는 `Enemy`가 아니라 `World::apply_strike`(220줄, 종 이름 16번)가 정한다. 보스는 표가 아니라 손으로 10~26개 파일에 배선돼 있다(다섯 마리가 나란히 — 셋의 규칙을 넘었다).

## 3. 잘된 것(지킬 것)
| 축 | 근거(잰 것) |
|---|---|
| 의존 방향 | 34개 모듈의 참조가 거의 DAG. 아래층이 위층을 보는 곳은 둘뿐: `terrain`·`sunny` → `render::{shadow_dir, sun_shear, SUN_SHADE}` |
| 로직 → 이벤트 → 연출 | `player::Event` 19·`enemy::Event` 13·`Cue` 9·`FxKind` 7. 월드는 소리·그림을 모르고 큐에 쌓는다(0169 R1) |
| 글 | `Tr = fn(&Strings) -> &str`. 월드·표는 언어를 모른다 |
| 표 | 새 적·무기·장신구·재주·저널·대사·시나리오·언어 = 표 한 줄 + 테스트 |
| 렌더가 월드를 읽는 것 | 89가지 경로를 읽지만 **읽기만** 한다. 0169 §4대로 뷰모델은 두지 않는다 |
| 적의 "깊은" 메서드 | `parryable`·`guarding`·`armored`·`is_boss` — 종 규칙을 적 안에 두는 길이 이미 반쯤 나 있다 |

## 4. 약한 것(고칠 것)
| # | 신호 | 잰 것 | 왜 문제 |
|---|---|---|---|
| A | `World`가 얕다 | 앱이 `world.*` 77가지를 읽고 15가지에 쓴다. `world.story.*`를 앱이 12곳에서 직접 대입(`app.rs` 8, `stage.rs` 4). 같은 사실이 `profile`(깃발)과 `World::story`(bool) 두 곳 — 앱이 손으로 맞춘다 | 이야기 깃발의 진실이 둘. 새 깃발 = 기록 키 + Story 필드 + 앱의 복사 두 곳. 월드는 제 불변식을 못 지킨다 |
| B | 종 규칙이 월드에 | `apply_strike` 220줄·if 44·종 이름 16, `hit_player` 163줄·if 46, `tick_player` 255줄. `world.rs`에 `Kind::` 종 이름 18번(FieldMouse·Dummy·Clover·HouseMouse·Doto·SporeCap·Hedgehog·Chief) | "콩이 등딱지에 튕긴다", "박쥐는 솟구치기에 안 맞는다", "토끼풀 고치는 못 때린다"가 적이 아니라 월드의 if에 있다. 새 종의 예외 = 월드 수정 |
| C | 보스가 표가 아니다 | 보스 하나가 닿는 파일: 모모 26, 도토·콩이·대장 13, 토끼풀 10. 나란한 코드: `adventure`의 `{Momo,Doto,Kongi,Clover}{Intro,Beaten}` 전환 변종, `stage.rs`의 `*_beaten` 넷, `cutscene`의 `*_meet`·`*_returns` 넷, `enemy/boss.rs`의 `*_next` 다섯, `dream.rs` 목록, 기록 키 | 다섯 마리가 같은 모양으로 나란하다(셋의 규칙을 넘음). 새 보스 = 8~10개 파일 손 배선 |
| D | `Player::update` 383줄 | 상태 12가지 `match`, R5 전의 `Enemy::update`와 같은 모양 | 기계적으로 쪼갤 수 있는 마지막 거대 함수 |
| E | 동적 장애물이 제각각 | `mines`·`clouds`·`rocks`·`cocoons`가 각자 `Vec` + 틱·그리기·빛·치우기·꿈 제외가 4~5개 파일에 흩어짐(`world`·`render/sorted`·`render/lights`·`adventure`·`app/dream`) | 넷이 나란하다. 공통은 **수명**(그릇·방 바뀔 때 치우기·꿈 제외·그리는 순서·빛)이고 충돌 규칙은 다 다르다 |
| F | 작은 것 | `Inventory::default`가 `journal::charm_pool(&Profile::default())`를 부른다(월드가 기록을 안다); `terrain`·`sunny` → `render` 상수; 기록 키 리터럴 36개 vs 상수 23개 | 방향이 거꾸로이거나 문자열이 흩어져 있다 |
| 지켜볼 것 | `Player::kit` 모드 분기 6곳 | 캐릭터 둘이면 괜찮다. 셋째가 오면 모드 인자가 "잘못된 추상화" 신호 → 그때 `Kit` 표·트레이트 |

## 5. 다음 목표(R9~R13 + 작은 것), 순서대로
각 단계는 **동작을 바꾸지 않는 커밋 하나**. 전후로 `cargo test` + 시나리오 통째 시험 + 연기 시험(`tools/test/smoke.js`).

| 단계 | 무엇 | 끝난 기준 | 비용 |
|---|---|---|---|
| **R9** | `Player::update` → `player/{mod,strike,dodge,special,…}.rs`, R5의 `Ctx` 방식 그대로 | `update` ≤ 60줄, 본문 그대로 | 1~2시간 |
| **R-소** | 해·그림자 모델(`shadow_dir`·`sun_shear`·`SUN_SHADE`)을 `sunny.rs`로 옮기고 `render`가 쓴다; 장신구 뽑기 표는 앱이 `World`에 넣어 준다; 기록 키 리터럴 → `profile::key` 상수 | 아래층 → 위층 참조 0, `world.rs`에 `profile::`·`journal::` 없음 | 1시간 |
| **R11** | 이야기의 진실 한 곳: `World::story`는 `World` 메서드로만 바뀐다(`beat_boss(kind)`·`mark(…)`), 기록 ↔ 월드 복사는 `App`의 두 함수(`story_from_profile`·`story_to_profile`)에만 | 앱의 `world.story.* =` 0곳(지금 12), `world.*` 쓰기 ≤ 3(toast·debug) | 반나절 |
| **R10** | `Enemy`를 깊게: "맞으면 무슨 일"을 적 메서드로(`on_struck(&Strike) -> Outcome`·`blocks_emerge()`·`touch_damage()` 등, `parryable`·`guarding`처럼). `apply_strike`·`hit_player`의 종 if를 그쪽으로 | `world.rs`의 종 이름 ≤ 4(뽑기·허수아비만), `apply_strike` ≤ 100줄, 옮긴 규칙마다 적 테스트 하나(등딱지 튕김·박쥐·고치·벽 쿵) | 1일 |
| **R12** | 보스 표 `bosses.rs`: {종, 길(몇째), 들어설 때·이기고 난 전환, 만남·귀환 컷씬, 기록 키, 마을 자리, 꿈 목록, 도감}. `adventure`의 `*Intro`·`*Beaten` → `BossIntro(kind)`·`BossBeaten(kind)`, `stage.rs`의 `*_beaten` → 표를 읽는 하나, `dream.rs` 목록 → 표 | 새 보스 = 표 한 줄 + `enemy/boss.rs`의 수 + 종 표 + 그림 + 글(파일 ≤ 5, 지금 8~10) | 1~2일 |
| **R13** | 동적 장애물 **수명만** 공통으로: `enum Hazard { Mine, Cloud, Rock, Cocoon }` + `Vec<Hazard>` 하나, 치우기·꿈 제외·그리는 순서·빛은 한 곳. 충돌 규칙은 변종별 `match` 가지에 그대로 | 장애물 하나가 닿는 파일 ≤ 3 | 반나절. **다섯째 장애물을 더할 때** 하거나 R12 뒤 |

하지 않을 것(0169 §4 그대로 + 이번 판정): 렌더에 뷰모델, ECS·트레이트 객체로 적 다시 짜기, `draw_sorted`(334줄)를 더 쪼개기 — 이미 `Drawable` 목록을 모아 정렬하는 올바른 모양이고 길이는 수집일 뿐, `draw_enemy`의 종 예외(그림의 예외는 그림에), i18n 구조.

## 6. 왜 이 순서인가
기계적인 것(R9·R-소)을 먼저 해 손이 가벼울 때 소유권(R11)을 정리하고, 그 위에서 적을 깊게 하고(R10), 마지막에 표(R12)를 짠다 — R12는 R11의 "진실 한 곳"이 있어야 기록 키를 표에 둘 수 있다. R13은 다섯째 장애물이 올 때까지 미룬다(둘 넷이 나란해도 충돌 규칙이 다 달라 지금 묶으면 모드 인자가 생긴다).

## 확인
- 수치는 1차(이 커밋에서 직접 잼). 기준의 출처는 2차: Ousterhout의 깊은 모듈(서평·요약), Sandi Metz의 "잘못된 추상화"(요약·비평). 비용은 추정.
- 사람이 정할 것: 어디까지 할지(R9·R-소·R11·R10 추천, R12는 보스를 하나 더 만들 계획이 있을 때 가장 값지다), 언제(마일스톤 판정 뒤).

## 구현(2026-10-27, 커밋 일곱: R9 → R-소 → R11 → R10 → R12 → R14, v0.3.9 배포 뒤)
| | 전 | 후 | 동작 변화 |
|---|---|---|---|
| R9 | `Player::update` 383줄(상태 12 가지) | 92줄 + `player/{attack,burrow,momo}.rs`의 `tick_*` 11개(`Ctx`: Actions·dt·move_dir), 시작 함수도 가족별 | 없음 |
| R-소 | `terrain`·`sunny` → `render::{shadow_dir,…}`, `Inventory::default`가 `journal::charm_pool(&Profile::default())` | `shade.rs`(해·모닥불 그림자 모델), 장신구 풀은 앱이 넣음. 아래층 → 위층 참조 0, `world.rs`에 `profile::`·`journal::` 0 | 없음(모든 시작이 풀을 덮어쓴다) |
| R11 | 앱이 `world.story.*`에 12곳 대입, 기록 ↔ 월드 복사가 흩어짐 | `Story` 필드 비공개. 기록 → `StorySeed` 한 번, 이긴 건 `Story::beat(kind) -> BossBeaten`, 읽기는 `fight(kind)` 등 게터. 앱의 대입 0 | 없음 |
| R10 | `apply_strike` 233줄·if 44·종 규칙 16(박쥐·등딱지·벽 쿵·독이 월드의 if) | `Enemy::receive(&Strike) -> Struck{Miss,Deflected,Hit}` 한 곳 + `strike_feel`·`strike_projectiles`·`dig_mines`. `apply_strike` 120줄(연출·통계·소리만). 혀는 `Enemy::tongue_reaches` | 없음. 적 테스트 셋(등딱지·벽 쿵, 한 스윙 한 번·밀 방향, 박쥐) |
| R12 | 보스마다 나란한 코드: `AdvEvent` 변종 8, `stage.rs` `*_beaten` 4, 장면 호출 4 | `bosses.rs` 표(종·몇째 길·칸·꽃잎·기록 키) + `KeeperIntro(kind)`·`KeeperBeaten(kind)` + `keeper_beaten` 하나 + `cutscene::keeper_meet/keeper_beaten`(주인마다 가지 하나). 새 보스 = 표 + 수 + 종 표 + 컷씬 함수·가지 둘 + 그림·글 | 없음. 테스트: 기록 키 그대로, 표의 주인마다 장면 있음 |
| R14 | `Character::Momo` 비교 분기 24곳(회피·특기·재주 키·고르기·큰 나무 탭·이어하기 번호·상태 탭·토스트 셋·그림 줄·클리어 합) | `characters.rs` 표(`CharacterDef` 18열) + `Character::def()`·`from_code()`·`available()`. 남은 비교 7: 모모에 관한 이야기 글 5, 그림 고르기 1, `Talents.momo` 1 | 없음 |

- 잰 것(뒤): 함수 1,150개 중 200줄 넘는 것 9 → 6(`draw_enemy`·`draw_sorted`·`draw_descent`·`scenario::build`·`dungeon::generate`·`tick_player`), `world.rs`의 종 이름은 `Story`의 보스 넷·소환·지뢰·죽은 포자버섯·토끼풀 실 정도(규칙은 적 안). 테스트 158 → 164.
- 하지 않은 것: R13(장애물 수명 공통)은 다섯째 장애물이 올 때. `draw_nail`의 모모 분기(그림의 예외는 그림에), 모모 이야기 글의 분기(글은 글), `Talents.momo`(모모 표의 자리 1·3 해석)는 정체성이라 남긴다.
- 바깥 확인: v0.3.9 main push에서 `smoke-web` 첫 실행이 팩 그림 404로 실패 → 잡음으로 거르고 팩을 받게 고쳐 작업 브랜치 수동 실행 성공.

