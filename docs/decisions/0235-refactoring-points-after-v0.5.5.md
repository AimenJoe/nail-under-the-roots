# 0235. v0.5.5 뒤 리팩터링 거리(R19~R25) — 제안

- 날짜: 2026-10-29
- 상태: **제안 → R19·R20·R24 구현**(사장 "위 패치 적용할 수 있는 것 해 주고", 2026-10-29). R21·R22는 크레딧(라이선스) 화면과 함께, R23은 레포 나누기 때, R25는 미룸.
- 잰 것(49d89b8, 시험 모듈 빼고): `src/` 55,186줄(0230 때 53,998), 시험 217. 200줄 넘는 함수 6(`draw_enemy` 366, `draw_descent` 349, `scenario::build` 300, `dungeon::generate` 251, `draw_player_fx` 218, `draw_journal` 204), 10줄 넘는 같은 블록 2곳, 가장 큰 파일 `world.rs` 3,829줄, 가장 큰 구조체 `App` 78필드·`Player` 45필드. 지난 한 주 커밋/시험: `app.rs` 92/0, `world.rs` 81/11.
- 판정: 급한 구조 문제는 없다. 보스 2단계·줍기 표시·터치 아이콘은 있던 틀 안에 들어갔다. 약한 곳은 **손으로 맞추는 나란한 표**와 **v0.6 일감(크레딧·PC·데모·레포 나누기)이 지나갈 자리**.

## 거리
| 번호 | 무엇 | 왜(실제 아픔) | 크기 | 언제(추천) |
|---|---|---|---|---|
| R19 | 소리 표의 손 번호: `sfx.rs` `FILES`·`file_of` 숫자 0~14·`synth` None 목록, `music.rs` `Track` ↔ `TRACKS`·타이틀 곡 `matches!` | 가운데 한 줄을 넣거나 빼면 소리가 조용히 엇갈리거나 `files[f]` 범위 밖 panic. `_ => None`이라 새 `Id`를 컴파일러가 묻지 않는다. `TRACKS`에서 빠진 곡은 그 장소가 조용히 무음 | 30~45분 | **지금**(다음 패치) |
| R20 | 플랫폼별 에셋 싣기 | **APK v0.5.5 51.9MB 중 mp3 23.9MB는 Android가 쓰지 못한다**(웹 전용. 직접 내려받아 확인). iOS·앞으로의 PC·Steam 데포도 같다. CI 스테이징에서 플랫폼 확장자만 싣고, `Art::files()`·`Sfx::files()`·글꼴·`TRACKS`를 목록 하나로 → 데모·PC 패키징이 같이 쓴다. 목록 시험으로 "빼서 무음" 막기 | CI 30분~1시간(목록은 +1시간) | **지금 CI 부분**(APK 거의 절반), 목록은 PC 빌드 전 |
| R21 | 장면 분류 한 곳 + 곡 고르기 순수 함수 | 타이틀 쪽 곡(`app.rs` `music_track`)·`in_world`·`title_bg` 세 `matches!` 목록 — 새 `Scene`(크레딧 화면)을 빠뜨려도 컴파일된다. `music_track`에 오늘 규칙 다섯(런 번갈아·마지막 쉼터·2단계 곡·hush·환경음)이 시험 0. `Scene::layer()` exhaustive + `music::choose(&Listen) -> Choice`(0230 R15 `flow.rs` 방식) + 시험 | 1~2시간 | 크레딧 화면 전 |
| R22 | 크레딧 원본 한 곳 | 같은 출처가 `make_audio.py` 표·`LICENSES.md`·`tools/licenses.py`의 손 줄·앞으로의 크레딧 화면 넷. 표 파일 하나(분류·작가·작품·URL·라이선스)를 `licenses.py`와 게임(`include_str!`)이 같이 읽고, `make_audio` slug가 표에 있는지 시험 | 2시간 | 크레딧 화면과 함께(그 앞) |
| R23 | 레포 나누기 전에 끊을 연결 | `site/site.js` `REPO`·APK 링크가 게임 레포 Releases를 본다(비공개가 되면 404·죽은 링크). `tools/site/make_images.py`는 출력이 `site/`로 고정. vendor checkout은 `persist-credentials` 기본값 — 운영 웹 `/assets/vendor/.git/config` 404, **APK에 `vendor/.git` 없음**(직접 확인), iOS만 명시적으로 지운다 → 시크릿 옮길 때 `persist-credentials: false` | 1시간 | 레포 나누기 때 |
| R24 | 패널 아래 안내 띠 | 같은 6줄(fit → 폭 → 띠 → 가운데 글)이 `app/pick.rs`·`app/dream.rs`·`app/stage.rs` 셋, 크레딧 화면이 넷째 → `ui.hint_strip` | 15분 | 지금 |
| R25 | `world.rs`를 폴더로 | 제품 코드 3,374줄, 2~3천 줄 문턱을 넘는 유일한 파일. 덩어리(가장 깊은 곳 ≈280, 실 굴 ≈175, `Story` ≈130, 연출 도우미 ≈250)는 분명하지만 함수 길이는 괜찮다 — 얻는 건 머지 충돌 줄이기뿐 | 1~2시간(기계적) | 미룸(그 부분을 다시 고칠 때) |

## 보기 싫지만 두는 것
- 보스 2단계 `roused`: 읽는 곳 넷 + `windup_mul`. 깃발 하나·`BOSS_P2_*`·시험이 있다. 보스마다 다르게 할 때 `bosses.rs` 표의 열로.
- `draw_pickup_mark`(이미 하나를 여섯 곳이), `icon_mask`(데이터, 12×12 시험 있음), `scenario::build`(가지끼리 독립 — 다만 보스 길 번호를 손으로 적어 `bosses.rs`와 겹친다 → 새 보스 때 `Scenario::Keeper(kind)`), `draw_enemy`·`App` 78필드(0170·0230 판단 그대로).
- 데모: 미리 할 리팩터링 없음(갈림길이 `flow.rs`·`after.rs`·`bosses.rs`에 이미 있다). 깃발은 `#[cfg]` 블록보다 `const DEMO: bool = cfg!(feature = "demo")`로 — 두 갈래가 늘 컴파일·clippy된다(0193의 죽은 코드 경고 문제를 피한다). flow 시험에 데모 경우.
- PC: 저장 위치(`profile.rs`)는 있다. 끝내기·전체 화면은 기능 일. CI에 Windows 타깃 check가 없다(PC 빌드 때 더한다).

## 교차 확인(Claude, 2026-10-29)
- 조사는 하위 에이전트가 했고, 아픔이 큰 둘을 직접 다시 봤다: v0.5.5 APK를 내려받아 열어 mp3 23.9MB·ogg 24.6MB(142파일), `vendor/.git` 없음. `tools/licenses.py`에 Kenney 줄이 둘(옛 줄 "Interface·Impact·RPG Audio"는 아직 쓰는 파일이 있어 틀리진 않지만 둘로 갈렸다).
- 함께 찾은 작은 것: `deep_last_knot_pulls_twice` 위에 옛 시험의 doc 주석이 붙어 있던 것(0233에서 시험을 끼우며 생김) — 바로 고침.

## 구현(2026-10-29)
- **R19**: `sfx::file_of`가 숫자 대신 이름을 돌려주고 `_` 없이 모든 `Id`를 적는다(자리는 `file_index`가 이름으로 찾는다). `music::Track::file()`도 `_` 없는 match — 예전 `TRACKS` 표를 없앴다. 시험 `file_names_match_the_table`(돌려주는 이름은 모두 표에, 표의 줄은 모두 쓰인다).
- **R20**: CI만 고쳤다 — Android·iOS는 `assets/music/*.mp3`를 빼고, 웹 배포는 `*.ogg`를 뺀다(레포·로컬 빌드는 그대로). 공용 파일 목록(데모·PC 패키징)은 PC 빌드 때.
- **R24**: `Ui::hint_strip(hint, panel, max_w)` 하나를 고르기 창·실의 꿈·큰 나무 창이 같이 쓴다(세 창 모두 화면 가운데라 자리는 그대로).
- 덤: `dungeon.rs`의 겹친 `#[test]`(같은 시험이 두 번 돌았다) 지움.

## 남은 것(2026-10-29, v0.5.6 뒤 다시 봄)
- **R21**: 라이선스 화면(크레딧 대신)이 R21보다 먼저 들어갔고, 예상한 대로 `Scene::Licenses`를 타이틀 곡·`in_world`·`title_bg` 세 `matches!`에 손으로 넣었다(빠뜨려도 컴파일된다). `music_track` 73줄, 시험 0. → 다음 장면(데모 끝 화면·PC 끝내기) 전에 한다.
- **R22**: `tools/licenses.py`에 Kenney 줄이 아직 둘, 출처 원본이 넷(`make_audio.py`·`LICENSES.md`·`licenses.py`·고지 파일). 라이선스 화면은 생성된 `THIRD_PARTY_LICENSES.txt`를 그대로 읽으니 원본 하나로 모으면 화면도 따라온다. → Steam 출시 전(고지 정확도).
- **R23**: 그대로(`persist-credentials` 지정 0곳, `site.js` `REPO`). → 레포 나누기 때(0237 순서 2~6과 같이).
- **R20 목록 부분**: 공용 에셋 목록 → PC 빌드 때.
- **R25**: 미룸(`world.rs` 3,829줄).
- 0169 R8의 하지 않은 것: `CLAUDE.md`의 "어디에 뭐가 있나"를 `docs/architecture.md`로 — 글 작업, 미룸.
