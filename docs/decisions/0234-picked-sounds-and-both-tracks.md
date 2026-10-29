# 0234. 사장이 고른 소리 넣기 — 옛 곡과 새 곡을 둘 다, 환경음 층

- 날짜: 2026-10-29
- 상태: **구현**(v0.5.x 패치 후보). 판정 10 "음악 및 효과음들은 더 필요할듯" → "여러 풀에서 내가 들어보고 선택할 수 있었으면" → 듣기 페이지(CC0 후보 142개, 26칸)에서 사장이 26칸을 다 고름 → "음악은 기존 것도 좋으니 둘 다 쓰는 방법을 생각해 줘"

## 무엇을 골랐나(듣기 페이지 기록 그대로)
| 칸 | 고른 것 | 들어가는 곳 |
|---|---|---|
| 타이틀 | First Light Particles(Yoiyami) | `title_light` |
| 마을 | Hot Springs Town(Kistol) | `village_springs` |
| 필드 | RPG Overworld(KarateStudios) | `field_overworld` |
| 쉼터 | Calm1 - A Place I Call Home(Juhani Junkala) | `camp_calm` |
| 뿌리 굴 | Cave explorer(mutantleg) | `roots_explorer` |
| 심층 굴 | The Seeing Pool(Umplix) | `deep_pool` |
| 실 지대(새 칸) | The Relic Under the Ice(Umplix) | `silk` |
| 보스 | Action1 - Encounter With The Witches(Juhani Junkala) | `boss_rise` |
| 가장 깊은 곳(새 칸) | Lament for a Warrior's Soul(RandomMind) | `final` |
| 엔딩(새 칸) | Goodbye to Friends(Écrivain) | `ending` |
| 환경음 들판·굴·깊은 곳(새) | 새소리(syncopika) · 물방울 고리(qubodup) · 땅 울림(cinameng) | `amb_meadow`·`amb_cave`·`amb_deep` |
| 효과음 | 발소리(Kenney 흙길 넷, 새) · 휘두르기(artisticdude) · 적 맞음(Junkala 펀치 둘) · 줍기(Bobjt 보석) · 회피(artisticdude 큰 휘익, 새) · 파고들기(삽) · 솟구치기(폴짝 착지) · 상자(Kenney 나무 문) · 메뉴 이동·확인(Kenney) | `sfx/*` |
| 그대로 | 적 쓰러짐·내가 맞음·메뉴 뒤로 | — |

## 둘 다 쓰는 규칙(`App::music_track` 한 곳)
나눌 까닭이 있는 곳은 **상황으로 가르고**, 같은 곳은 **런마다 번갈아**(지난 런 수가 짝수면 새 곡, 홀수면 옛 곡 — 첫 판은 새 곡). 무작위가 아니라 규칙이라 "왜 이 곡?"에 답할 수 있다.
| 곳 | 새 곡 | 옛 곡 | 어떻게 |
|---|---|---|---|
| 타이틀 | First Light | Prologue Theme | 런마다 번갈아 |
| 마을 | Hot Springs Town | Lovely Town | 런마다 번갈아 |
| 필드 | RPG Overworld = 보리 공터로 가는 트인 길 | 칩튠 Bittersweet Story = 꽃 핀 입구 굴 | 상황으로 |
| 쉼터 | Calm = 첫·둘째 쉼터 | Snowfall = 마지막 쉼터(가장 깊은 곳 앞) | 상황으로 |
| 뿌리 굴 | Cave explorer | 칩튠 Dungeon | 런마다 번갈아 |
| 심층 굴 | The Seeing Pool | 칩튠 Shrine of Mysteries | 런마다 번갈아 |
| 보스 | Encounter With The Witches = **2단계**(0233, 깨어나면 넘어간다) | 칩튠 Battle 2 = 1단계 | 상황으로(싸움 한가운데 교차) |
| 실 지대·가장 깊은 곳·엔딩 | 새 곡만(예전엔 심층·보스 곡을 같이 썼다) | — | — |

## 환경음 층
- 음악과 따로 된 층(`Music::ambience()` — 같은 교차·덕킹 장치, 크기는 설정의 **효과음**, `AMB_MASTER` 0.35).
- 들판(필드·마을·꽃 핀 굴) = 새소리, 굴(뿌리·실 지대·쉼터) = 물방울, 깊은 곳(심층·가장 깊은 곳) = 땅 울림. 아레나는 없음.
- 컷씬이 음악을 내려도(`Step::Hush`) 환경음은 남아 정적을 채운다.
- 새소리 원본은 3초뿐이라 그대로 돌리면 되풀이가 들린다 → `make_audio.py`가 24초 바탕에 크기를 바꿔 다섯 번 흩어 둔다(`SCATTER`).

## 효과음 연결
- 휘두르기 = 평타 1·2타(`Claw(0|1)`), 마무리 타는 무거운 합성음 그대로. 발소리 = 걷는 동안 두 걸음(24px)에 한 번(`World::juice.steps`, 발걸음 먼지와 같은 자리). 회피 휘익 = 모모·도토의 깡충·뒤로 뛰기·멀리뛰기(`player::Event::Hop`, 예전엔 돌진 합성음). 줍기 = 가방 아이템·무기(반딧불·씨앗은 오르는 음 그대로). 파고들기·솟구치기는 합성음 → 파일.
- 파일이 없으면 예전처럼 합성음으로 난다.

## 만들기
`python3 tools/audio/make_audio.py oga <OpenGameArt 폴더>` — 새 단계(`OGA_SFX`·`OGA_MUSIC`·`OGA_AMB` 표). 단계마다 자기 표의 파일만 다시 쓴다(예전엔 한 번에 `assets/sfx`를 비웠다). 원본은 커밋하지 않는다. 라이선스는 모두 페이지 라이선스 칸이 CC0 하나뿐인 것(여러 라이선스를 건 페이지는 후보에서 뺐다), 무작위 여섯 페이지를 다시 열어 확인. 출처·작가는 `assets/LICENSES.md`·`THIRD_PARTY_LICENSES.txt`.

## 무게
- 새 곡 열 + 환경음 셋 = ogg 약 17MB + mp3 약 17MB(웹은 들을 때 받는다, APK는 ogg만 든다).
- 네이티브는 곡을 통째로 f32 스테레오로 푼다(1분 ≈ 21MB). 가장 긴 실 지대 곡 175초 ≈ 61MB. 보스 곡 둘을 쥐는 동안(`KEEP`) 약 80MB. 핸드헬드에서 멈칫·메모리를 사장이 확인한다.

## 확인
- `cargo test`: `every_track_has_both_files`(표의 곡마다 mp3·ogg), `every_file_sound_exists`(효과음 표의 변형 수 = 파일 수).
- 헤드리스 Chromium: 타이틀 → `title_light`, 필드 → `field_overworld` + `amb_meadow`, 쉼터 → `camp_calm` + `amb_cave`, 2단계 보스(개발 "보스 2단계") → `boss_rise`가 모두 200.
- 사람 귀: 크기 균형(음악 대 환경음 대 효과음), 발소리가 거슬리는지, 고리 이음매("고리 ?" 곡들), 2단계 곡 넘김.
