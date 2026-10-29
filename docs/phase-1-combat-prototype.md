# 1단계: 전투 프로토타입 요구사항

> 목표는 하나다. **"굴파기로 튀어나와 때리는 게 기분 좋은가?"** 에 답한다. 에셋·콘텐츠·메타는 전부 범위 밖.

## 범위 (In)
- 방 1개(320×180 안에 들어오는 고정 방, 벽 충돌).
- 플레이어 동작 3개: 이동(8방향, 아날로그 스틱 대응), 발톱 3연타, 굴파기.
- 적 2종: 돌격형(플레이어 쪽으로 돌진 후 잠시 멈춤), 원거리형(거리 유지, 느린 투사체).
- 피격·사망·즉시 재시작(R 또는 Start).
- 전투감 레이어: 히트스톱(명중 시 수 프레임 정지), 화면 흔들림, 피격 플래시(흰색), 넉백, 임시 사운드(합성음이라도).
- 입력: 키보드 + 게임패드(`gamepads` 크레이트, 데스크톱·웹). 전부 `Actions`로 번역.
- 회색 박스 그래픽. 플레이어/적/투사체는 도형. 방향은 "코"(작은 점)로 표시.
- 디버그 오버레이: fps, 적 상태, 히트박스 토글.

## 범위 밖 (Out)
에셋, 보상 선택, 허브, 저장, 사운드 믹싱, 두 번째 캐릭터, Android 빌드.

## 굴파기 명세 (초안, 손맛으로 조정)
- 입력: `dodge` 누름 → 즉시 땅속(무적, 이동 가능, 속도 1.3배, 그림자만 표시) → 0.35초 후 또는 `dodge` 다시 누름 → 솟구침(반경 ~16px 범위 공격, 적 넉백·경직). `dodge`를 누르고 있으면 0.45초까지 더 머물고 떼면 솟구침, 더 머문 만큼 쿨다운이 는다(0101). 더 머무는 동안 힘이 차서 솟구침이 커진다(반지름 ×1.45·피해 1 → 2·넉백 ×1.5, 0158).
- 쿨다운 0.6초. 쿨다운 중 `dodge`는 버퍼(0.1초)로 받는다.
- 땅속에서 `attack` → 조기 솟구침.
- 굴파기 중 공격받지 않지만, 솟구친 직후 0.1초는 **공격 불가**(리스크).

## 발톱 3연타 명세 (초안)
- 1타·2타 빠름(각 0.12초), 3타 느리고 넉백 큼(0.2초). 입력 버퍼 0.1초.
- 공격 중 이동 속도 0.4배. 공격 방향은 마지막 이동 방향(스틱) 또는 오른쪽 스틱.
- 히트박스는 반원(전방 14px). 디버그에서 표시.

## 기술 요구
- 고정 타임스텝(60Hz) 로직 + 가변 렌더. 히트스톱은 로직 시계만 멈춘다.
- 내부 해상도 320×180, 정수 배율(이미 `main.rs`에 있음).
- 모듈 분리: `input`(Actions), `player`, `enemy`, `combat`(히트박스·히트스톱), `room`, `render`. 각 모듈이 `Actions`와 월드 상태만 본다.
- 숫자(속도, 시간, 반경)는 한 파일(`tuning.rs`)에 모아 손맛 조정이 한 곳에서 되게.

## 판정 기준 (이 중 하나라도 "아니오"면 메커닉을 바꾼다)
1. 아무 설명 없이 처음 잡은 사람이 30초 안에 굴파기 → 솟구침 → 때리기를 스스로 한다.
2. 적 3마리를 상대할 때 "어디서 튀어나올까"를 생각하게 된다(위치 선정이 전투의 일부).
3. 3타째가 들어갈 때 "묵직하다"고 느낀다(히트스톱·흔들림·사운드가 합쳐진 결과).
4. 5분 플레이 후 더 하고 싶다.

## 완료 정의
- 위 판정 4개를 두 명 이상(본인 포함)이 통과.
- 웹 빌드가 Netlify에 배포되어 링크로 열린다(CI는 이미 준비됨, Netlify 시크릿만 등록).
- 손맛 수치가 `tuning.rs`에 정리되고, 결정 사항이 `docs/decisions/`에 기록.

## 구현 상태 (2026-10-24, 표는 2026-10-27 v0.2.0 + 0139까지)
코드는 전부 들어갔다. **판정: 통과**(2026-10-25 사장 "판정 및 손맛은 좋은 편이야, 지금 이대로도"). 수치는 지금 값을 기준으로 둔다. 본인 외 한 사람도 판정했다(2026-10-26, 1·4 동의, 2·3 약함 → 0090). 2·3 다시 묻기가 남았다.

| 항목 | 위치 | 비고 |
|---|---|---|
| 고정 타임스텝 60Hz + 가변 렌더(보간) | `src/main.rs`, `world.rs` | 눌림은 `input.rs` 래치에 모아 틱이 소비. 히트스톱·일시정지 중엔 보간 prev를 현재로 맞춰 미끄러짐 방지 |
| 모듈 분리 | `input` `player` `enemy` `combat` `room` `render` `world` `tuning` `sfx` | `world`는 틱 순서를 엮는 곳, `sfx`는 임시 합성음(명세 밖 추가) |
| 발톱 3연타 | `player.rs` | 활성 구간이 끝나면 버퍼된 입력으로 다음 타가 바로 이어짐. 3타 뒤 추가 경직 0.14초. 스윙 시작 시 전방 전진(lunge) |
| 굴파기 | `player.rs`, `input.rs`(dodge_held) | 명세에서 바꿈: 솟구침 반지름 22·직후 공격 불가 0.06초·밀쳐내기 44(0090), 누르고 있기(0101), 누르고 있으면 힘 모으기 → 큰 솟구침(0158), 톡은 파고드는 그림 + 2.1배 대시 + 폴짝 솟구침(0160). 솟구침은 투사체도 지운다(추가). 쿨다운 바를 HUD 오른쪽 위에 표시 |
| 적(물든 이웃) | `enemy.rs`, `species.rs`, `tools/art/critters.py` | 그림 v2: 종마다 기본 자세를 손으로 찍고 동작은 옮기기·몇 점 바꾸기(v1 타원 쌓기는 얼룩져 안 읽혔다). 다섯 종: 들쥐·생쥐 떼·시궁쥐(돌격 셋), 다람쥐(도토리), 청설모(웅크렸다 뛰기, 착지 고리). 종류 표 = 수치·도감 글. 쥐가 절반 넘게 뽑힘. 개발 시나리오 "적 다섯 종"(docs/decisions/0021) |
| 내 캐릭터 구별 | `tools/art/critters.py`(taint), `render.rs`(draw_player_halo) | 물든 적은 털색을 보라 쪽 + 어둡게, 외곽선 차갑게. Naïl은 따뜻한 색 그대로 + 발밑 빛 고리(천천히 숨 쉼). 흑백으로 봐도 Naïl이 더 밝다 |
| 떨어진 것 | `loot.rs`(bounds), `adventure.rs`(loot_bounds, push_loot) | 던전에서는 떨어진 방 안쪽에서만 구른다(문 칸으로 굴러가 주우려다 옆 방으로 넘어가던 문제) |
| 공격 예고 | `render.rs`(draw_telegraphs), `room.rs`(ray_len) | 돌진 길 띠·조준 점선·착지 고리를 바닥에 반투명 붉게(채움 3→16%, 테두리 15→50%), 예고가 끝날수록 진하게 + 마지막 15% 깜빡임(docs/decisions/0025, 0026) |
| 웹 로딩 화면 | `web/index.html`, `src/loading.rs`, `main.rs` | 글 없는 걷는 Naïl + 막대. wasm 받은 바이트(0~60%) + 준비 5단계(60~100%), 첫 프레임 뒤 걷음(docs/decisions/0028). 느린 회선 흉내로 확인 |
| 웹 GL 컨텍스트 | `web/index.html` | 안티에일리어싱·투명 배경 끔, 고성능 GPU 요청. 아이폰급(dpr 3) 캔버스 2532×1170에서 매 프레임 4배 샘플을 없앰. 헤드리스 같은 3초 동안 그린 프레임 62 → 144(docs/decisions/0026) |
| 도감·상태 | `app.rs`(draw_bestiary, draw_status) | 도감: 만난 이웃, 능력은 말로. 상태: 플레이어 능력치 숫자(체력·공격력·공속·굴파기 대기·분노 충전·무기·장신구·반딧불·씨앗) |
| 조작 설정 | `input.rs`(Bindings, prompt, capture), `prefs.rs`, `app.rs`(draw_controls), `web/index.html`(nail_prefs) | 오른쪽 넷 = 공격·굴파기·필살기·아이템, 어깨 = 아이템 넘기기, Select = 지도. 다시 붙이기(겹치면 맞바꿈), 웹 저장(docs/decisions/0022) |
| 피격·사망·재시작 | `player.rs`, `world.rs`, `tuning.rs`(PLAYER_HP) | 체력 5(처음 3, 0016), 피격 무적 0.8s. 재시작은 메뉴 → 다시 시작(터치는 R 버튼) |
| 전투감 | `combat.rs`(Feel), `world.rs`, `render.rs` | 히트스톱(로직 시계만), trauma² 흔들림(정수 px), 흰색 플래시, 넉백(감쇠), 합성음 |
| 게임패드 | `input.rs`, `web/macroquad-gamepads-0.1.js` | 메뉴는 D패드·왼쪽 스틱 둘 다(스틱은 큰 축 하나, 0.5에서 잡고 0.3에서 풀림, 누르고 있으면 반복). 데스크톱(gilrs)·웹(플러그인). 웹은 비표준 매핑 패드도 받도록 플러그인 패치. Android 13 폰 브라우저 + 실기 패드로 인식 확인(2026-10-25). 데스크톱 실기는 미확인 |
| 터치(보조) | `input.rs` TouchState·`layout_for`, `render.rs` 오버레이, `web/index.html` | 왼쪽 반 떠다니는 스틱 + 버튼. 공격이 가장 크고(지름 ≈ 짧은 변의 27%, 가로 폰 약 105pt), 굴파기·필살기·아이템은 엄지 호를 따라(2026-10-25, 겹침·화면 안 테스트). R(다시 시작) 버튼은 뺐다(2026-10-27 사장: 쓰러진 뒤 ATK 탭으로 다시 시작하니 필요 없음). 모바일 브라우저용. 터치 좌표는 DPI로 나눠야 함(macroquad touches()는 물리 픽셀). Android 13 폰 브라우저 실기 확인(2026-10-25) |
| 디버그 | F1 오버레이(fps, 상태), F2 히트박스 | 기본 꺼짐. 설정에서도 켬/끔 |
| 그림 | `art.rs`, `biome.rs`, `render.rs`, `tools/art/` | Naïl 스프라이트 v2(직접 그림. 얼굴: 동그란 코·볼 홍조, 입 없음, 발톱은 발 줄에만 — v1은 이빨처럼 보였다), 대체 타일, 3/4 벽·y 정렬·그림자, 환경 3종(색조·장식·어둠), 발톱 자국 이펙트, 컷씬 레터박스. Sunnyside 팩은 레포 밖(docs/decisions/0010) |
| 장비 | `equipment.rs`, `moveset.rs` | 캐릭터별 무기 종류. Naïl은 발톱 4종(새싹·두더지·바위·독), 가방에서 장착, 독 지속 피해. 규칙은 `cargo test`(docs/decisions/0011) |
| 무브셋 | `moveset.rs` | 콤보를 데이터로(발톱 3타, 실험용 무거운 2타). 타별 회피 캔슬 창 `dodge_cancel_after`. 개발 시나리오 "무거운 무기"로 비교(docs/decisions/0009) |
| 튜토리얼 | `tutorial.rs`, `cutscene.rs`(popo_talk), `world.rs`(npc) | 라쿤 포포의 10단계: 이동·발톱·3연타·강공격(0161)·굴파기·솟구치기·패링(30초면 넘김)·필살기·아이템·메뉴 안내. 단계마다 대화 + "[n/10]" 힌트(지금 버튼 이름), 연습 중 체력 그대로(docs/decisions/0023). 헤드리스 Chromium으로 오프닝 → 포포 → 1~3단계 확인 |
| 개발 시나리오 | `scenario.rs` | 타이틀 "개발"(dev feature). 빈 방, 들쥐 1/3마리, 다람쥐 1마리, 1+1, 웨이브 3부터, 오프닝 컷씬 외 보스·장소·이야기 일러스트들. 판정 2(적 3마리)는 "들쥐 3마리"로 바로 들어간다 |
| 컷씬 | `cutscene.rs` | JRPG식 대화창·배우 이동·페이드·건너뛰기. 오프닝: 물든 모모(이름표 "???", 초상화 차가움·괴로움, 스프라이트)가 길을 막는다(docs/decisions/0008, 0029) — 음악 끊김·시든 꽃잎·대낮의 어둠·'???' 카드로 연출(0159 덧붙임). 장치(0159): 카메라 팬·어둠·음악 멈춤·이펙트 큐·이름 카드, 컷씬 중에도 입자는 흐른다 |
| 게임 틀 | `app.rs`, `ui.rs` | 인트로 → 타이틀(시작·이어하기·오늘의 굴·앨범·설정·개발) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도). 커서 없는 탐색 + 탭/클릭 보조. 한글 픽셀 폰트. 탭 여섯(메뉴·상태·가방·저널·도감·지도). 처음엔 가방·지도가 틀만, 저널이 튜토리얼 퀘스트 4개였다(0007) — 지금 내용은 아래 저널·장비·지도 줄 |
| 글자 성능 | `ui.rs`(glyph_plan, warm), `render.rs`(present) | 픽셀 폰트를 원래 12 물리 px(정수배)로 굽고 정수배로 키움 + 언어를 정할 때 그 언어 글자를 한 번에 미리 굽기. 대화 중 텍스처 올리기 160MB → 0(헤드리스 측정). 화면은 수정 전과 픽셀 단위로 같음. Safari 버벅임 대응(실기 확인은 사람) |
| 다국어 | `i18n.rs`, `ui.rs`, `tools/fonts/`, `web/index.html` | 처음 다섯(한국어·영어·일본어·간체·브라질 포르투갈어, 0033) → 지금 열하나(+프랑스어·스페인어 0084, 독일어·러시아어 0085, 번체·폴란드어 0144) 표, 언어별 폰트(일·중은 Ark Pixel 서브셋, 번체는 + 俐方體11號), 기기 선호 목록으로 시작, 설정에서 전환, 가짜 언어 Pseudo. CJK 줄바꿈(금칙)·넘침 대비(fit·탭 여백). 폰트 글자·자리표시자 테스트(docs/decisions/0013, 0014). 번역은 직접 쓴 초안(원어민 검수 전) |
| 초상화 | `tools/art/portraits.py`, `art.rs`, `render.rs`(draw_portrait), `cutscene.rs`(Face) | 88×88, 게임 픽셀의 2배 밀도. 아홉 캐릭터 열세 묶음(나노바나나, 0107~0122), 그림자는 칸 4만 남김(docs/decisions/0015). 홀수 배율에서도 칸을 꽉 채운다(0111) |
| 반딧불·산딸기 | `loot.rs`, `world.rs`(drop_loot), `render.rs`(draw_loot, HUD) | 화폐(자석·HUD·상태 탭), 회복(불운 보정), 최대 체력 5, 줍는 소리(음 올라감). 개발 시나리오 "떨어지는 것 확인"(docs/decisions/0016) |
| 능력치·장신구 | `equipment.rs`(Stats, CHARMS), `app.rs`(가방 끼기/빼기, 상태 탭) | 장신구 8종(기본 5 + 저널 3) 2칸, 생명의 씨앗, 공격 속도 = 스윙 시계 배율(docs/decisions/0017) |
| 퀵 아이템 | `items.rs`, `player.rs`(State::Eat), `world.rs`(start_eating, finish_eating, revive), `render.rs`(퀵 슬롯·draw_eating) | 산딸기 1칸 회복, 쥬시 지렁이 가득 회복 + 목숨(최대 3), I·Y 먹기, Q·E / LB·RB 넘기기. 먹는 시간 0.35초(맞으면 끊김, 굴파기로 끊기) + 먹기 대기 1.5초, 슬롯 덮개·터치 테두리(0038) |
| 필살기 | `player.rs`(State::Drill/Recover), `world.rs`, `render.rs`(trail·super dim), `sfx.rs`(engine) | 분노 게이지. 시동 0.3초(적 정지·화면 어둡게) → 화면 한 폭 돌진(≈320px), 연타로 벽까지 끌고 가 쾅(번쩍임). 엔진음 하나. 단위 테스트(docs/decisions/0018, 0024) |
| 모험(필드·던전) | `adventure.rs`, `map.rs`, `field.rs`, `dungeon.rs`, `room.rs`(격자), `render.rs`(draw_grid_art) | 연속 스크롤 필드, 자는 적, 저장 지점, 굴 입구 → 생성 던전(문 잠금·보물·휴식·보스·출구), 밀기 전환, 지도 탭, 저장 지점 부활. 단위 테스트(docs/decisions/0019) |
| Android 패드 버튼 | `third_party/miniquad`(패치, `[patch.crates-io]`), `input.rs`(Pad::key·read_gamepads) | 패드 버튼 키코드 → F13~F25, 동작은 Bindings, 메뉴 A/B(뒤로)/Start/LB·RB, 다시 붙이기. 스틱은 Android가 바꿔 준 D패드 키(디지털). 패치 표 = Pad::key 테스트(0075) |
| 저장 지점 빛 | `render.rs`(draw_save_ring·draw_save_lights·light_material) | 버섯 일곱 + 짙은 고리 자국, 더하기 섞기 금빛 후광·포자(어둠 위), 안 들른 고리는 빛기둥 높이(0076) |
| 팩으로 갈아 끼운 소품 | `sunny.rs`(STALL·BENCH·POT_WILDFLOWER·HEART·SPROUT), `render.rs`(draw_stall_sunny·draw_seat_sunny·draw_decor_sunny·draw_hud_bars), `ui.rs`(sprite) | 가게 가판, 모모 빈 자리 그루터기, 꾸미기 의자·화분, HUD 하트·새싹. 팩이 없으면 예전 그림(0077) |
| 저장 슬롯 | `profile.rs`(SLOTS·slot·set_slot·load_slot·save_slot), `app/slots.rs`, `app.rs`(TitleItem), `web/index.html`(nail_save 칸 번호) | 슬롯 셋, 예전 기록 = 슬롯 1, 타이틀 시작하기·이어하기(기록이 있을 때만) → 슬롯 고르기, 요약·두 번 눌러 덮어쓰기·지우기(0078) |
| 쉼터부터 이어하기 | `suspend.rs`(RunSave), `app/run.rs`(checkpoint_run·resume_run·clear_run), `adventure.rs`(resume_camp), `profile.rs`(칸 4~6) | 쉼터 도착·가게 닫기·쉼터에서 타이틀로 갈 때 적고, 이어하기 → 그 쉼터(같은 길 지도·재화·가방·장비·통계). 새 런·마을·엔딩·덮어쓰기에서 지움(0080) |
| 특수효과(0081) | `world.rs`(FxKind Puff·Clod·Glint·Thread, puff·clods·spray·unravel, 발걸음), `render.rs`(draw_fx, draw_ambient, draw_world_lights) | 먼지·흙덩이·맞은 쪽 불똥·쓰러진 이웃의 보라 실·반짝임, 환경 입자, 더하기 빛(반딧불·모닥불·빛 버섯) |
| 음악 바탕(0081) | `music.rs`, `app.rs`(music_track, 설정 음악 줄), `prefs.rs`(칸 18), `sfx.rs`(같은 소리 30ms) | 장소마다 곡·1초 교차·대사/창에서 낮춤 |
| 공격 동작(0082) | `tools/art/nail.py`(bob·windup·settle, 줄 1 칸 9개), `render.rs`(draw_nail 단계, draw_claw_marks sweep) | 들기 → 치기 → 거두기, 1·2타 가까운 발·먼 발 번갈아, 마지막 타 두 발 + 윗몸 내리찍기, 발톱 자국도 번갈아(마지막은 X) |
| 타이틀 바탕(0083) | `tools/art/illust.py`(title), `art.rs`(Art::title), `render.rs`(draw_title_art), `app.rs`(타이틀·슬롯·설정 창) | 지금은 나노바나나 해 질 녘 땅의 단면(0137) + 흐르는 안개 두 겹·반딧불·보라 알갱이. v2 코드 그림(어둠 속 단면·굴 방 비네트)은 원본이 없을 때만 |
| 프랑스어·스페인어(0084) | `i18n.rs`(FR·ES 표, `Lang::Fr`·`Es`, `fr-*`·`es-*`), `prefs.rs`(언어 값 6·7), `assets/fonts/Galmuri11-Condensed.ttf`(+œŒŸ) | 일곱 언어. 헤드리스로 여섯 화면 넘침 확인·수정 |
| 콩이의 문·독일어·러시아어(0085) | `illust.py`(kongi_door), `illust.rs`(KongiDoor), `cutscene.rs`(kongi_returns), `app.rs`(앨범 5칸·두 줄 굴리기), `i18n.rs`(DE·RU, kongi_door), `prefs.rs`(8·9) | 콩이가 지키던 문의 컷씬 그림(지금은 나노바나나, 0136)·내레이션, 앨범 스크롤, 아홉 언어 |
| 소리 파일(0081) | `tools/audio/make_audio.py`, `assets/sfx`·`assets/music`, `sfx.rs`(FILES·변형), `music.rs`(TRACKS·들을 때 읽기·KEEP), `app.rs`(menu_sound) | CC0 효과음(Kenney: 메뉴·사기·안 됨·맞음·쓰러짐·다침·상자)·음악 7곡(칩튠 Aureolus_Omicron 넷 + 쉬는 곳 피아노·바이올린 셋, 0154). 웹 헤드리스로 받기·재생·고리 확인, 네이티브 실제 소리는 미확인. 0234: 사장이 듣기 페이지에서 고른 새 곡 열(옛 곡과 둘 다 — `music::choose`)·환경음 셋(`Music::ambience`)·효과음 열(발소리·휘두르기·회피·줍기 …), `make_audio.py oga` 단계 |
| 강공격(0161) | `input.rs`(attack_held), `player.rs`(Charging·Heavy·heavy_strike·parry_heavy), `moveset.rs`(heavy_of), `combat.rs`(StrikeKind::Heavy), `world.rs`, `render.rs`, `tutorial.rs`(Stage::Heavy) | 누르는 순간 평타, 누르고 있으면 활성 구간 뒤 모으기 → 0.45초 뒤 떼면 피해 2 이상·반지름 1.3배 한 방(마지막 타에서 만든다), 콤보는 다음 타로(후딜레이는 못 끊지만 잠금은 없다, 0167). 첫머리 창은 진홍 고리 공격도 받아친다. 튜토리얼 열 단계 중 4번째 |
| 박자 콤보(0167 → 0174) | `player/`(Queued·beat_clock·beat_at·combo_pending·queue_due·beat_cue·beat_pips), `tuning.rs`(BEAT_PERIOD·BEAT_WINDOW·BEAT_RING_*·BEAT_PIP_SHOW), `world.rs`·`sfx.rs`(Beat(n) 도미솔도), `render/actors.rs`(draw_beat_cue) | 평타는 누르는 순간. 다음 타의 박자는 스윙 시작 0.25초 뒤(스윙·후딜레이가 길면 그 끝). 창 ±0.06초 안이면 그 자리에서 + 덤·퍼지는 고리·오르는 종소리·머리 위 음표, 창 전이면 박자까지 기다렸다(스냅, 덤 없음), 창 뒤면 그 자리에서(덤 없음). 닫히는 고리가 박자를 보여 준다. 강공격 뒤 잠금 없음 |
| 설정 파일(0175) | `prefs.rs`(store::read_from·write_to·load_all·save_all), `profile.rs`(data_dir) | 데스크톱·Android·iOS도 기록 폴더의 `prefs.txt`에 설정을 남긴다(전엔 켤 때마다 기본값) |
| 적 공격 예고의 "언제"(0176 → 0179) | `enemy/mod.rs`(telegraph), `render/fx.rs`(draw_telegraphs·draw_fuse_front·draw_dots·draw_dotted_fuse·draw_fill_disk) | 방향 안내 자체가 차오른다: 돌진 띠는 적 쪽부터 끝까지 + 흰 가로줄, 점선은 밝아지며 흰 점, 둘레는 가운데부터 원. 끝에 닿는 순간 공격. 적 발밑 고리는 뺌 |
| 부술 수 있는 것(0177) | `room.rs`(PropKind::Breakable·remove_prop), `dungeon.rs`(breakables), `world.rs`(prop_break·stats.broken), `render/props.rs`(draw_breakable), `sfx.rs`(Break), `tuning.rs`(BREAK_*) | 굴 방마다 1~3개(지대마다 흙더미·뿌리혹·돌무더기·실뭉치), 공격이 닿으면 깨져 씨앗 30%·산딸기 12%·반딧불 10%·지렁이 4% |
| 안내 화면(0178) | `help.rs`(TOPICS), `app/help.rs`(update_help·draw_help), `i18n/`(h_* 30키 × 11), `app/pause.rs`(PAUSE_MENU 5), `app/title.rs`(TitleItem::Help) | 일시정지·타이틀의 "안내": 주제 열셋, 쪽 넘기기. 산딸기는 송이 비트맵(`draw_berry`) |
| 안내 탭·힌트 그림·메트로놈(0180) | `app/pause.rs`(PAUSE_TABS 7·TAB_HELP), `app/help.rs`(help_nav·draw_help_body), `help.rs`(Pic), `render/help.rs`(draw_pic → `pic_*`, 0230), `player/`(BeatTick·beat_period), `sfx.rs`(BeatTick) | 안내는 일시정지 일곱째 탭(타이틀에서는 뺌, 0182). 주제마다 진짜 그림으로 한 장면을 되풀이. 콤보 중 박자에 '똑' + 과녁 번쩍(그 틱에 타가 나가면 없음) |
| 적 예고 면·느린 박자(0183) | `render/fx.rs`(tele_colors·draw_tele_lane·draw_tele_disk·TELE_EDGE_A·TELE_FILL_A0), `render/help.rs`, `tuning.rs`(BEAT_PERIOD 0.36·BEAT_WINDOW 0.08), `player/attack.rs`(combo_window_after_swing) | 예고는 면 + 진한 테두리, 안쪽이 진해져 테두리와 같은 색이 되는 순간 공격(황금 띠 = 받아침, 진홍 원판 + X = 피함). 평타 박자 0.25 → 0.36초, 창 ±0.08초 |
| 예고 채움이 자란다·박자 0.42초(0184) | `render/fx.rs`(fill_lane·draw_tele_lane·draw_tele_disk·TELE_BASE_A·TELE_GROW_A), `render/help.rs`(Pic::Combo 주기), `tuning.rs`(BEAT_PERIOD 0.42·BEAT_WINDOW 0.09) | 옅은 면 + 진한 테두리 안에서 진한 채움이 자라(띠는 적 쪽부터, 원은 가운데부터) 테두리 끝에 닿는 순간 공격. 평타 박자 0.36 → 0.42초 |
| 예고: 같은 투명도·띠는 폭이 넓어진다(0185) | `render/fx.rs`(TELE_A·fill_lane·draw_tele_lane·draw_tele_disk) | 테두리와 채움이 같은 색·같은 투명도(0.6) — 닿는 순간 한 덩어리가 되면 공격. 띠는 가운데 줄에서 양옆으로(진행 막대처럼 한쪽 끝부터 차지 않음), 원은 가운데부터 |
| 예고 외곽선 = 채움(0186) | `render/fx.rs`(TELE_RING·fill_ring·fill_poly·lane_outline·circle_pts, TELE_A 0.75) | 외곽선은 채움과 같은 색·같은 투명도의 2px 띠(어두운 밑띠 없음), 채움은 외곽선 안쪽까지만 자라 겹치지 않는다 — 다 차면 면 전체가 한 색 |
| 예고 색 하나(0187) | `render/fx.rs`(TELE_RGB), `i18n/`(h_parry_d·h_enemy_d) | 예고는 모두 붉은 계열. 받아칠 수 있는지는 모양으로 — 띠는 받아칠 수 있고, X가 있는 원은 피한다 |
| 예고가 다 찬 채로 0.1초(0188) | `tuning.rs`(TELE_FULL_HOLD), `enemy/mod.rs`(tele_progress), `render/fx.rs`(TELE_A 0.6·TELE_BASE_A 0.08) | 예고는 공격 0.1초 전에 다 차고 남는다(전엔 다 찬 그림이 한 프레임도 안 그려졌다). 공격 시각은 그대로 |
| 다 맞춘 마무리·즉시 강타·받아치기(0189) | `tuning.rs`(FULL_BEAT_DAMAGE·PARRY_COOLDOWN 0.3·PARRY_THREAT_*), `player/attack.rs`(full_beat), `moveset.rs`(heavy_of 활성 0초부터), `player/mod.rs`(threat·queue_due), `world.rs`(parry_threat) | 이음을 다 맞춘 마무리 타 피해 +1. 강타는 뗀 다음 틱에 맞는다. 받아칠 위협이 90px 안이면 창 전 눌림도 박자를 기다리지 않고, 받아치기 대기는 박자보다 짧게 |
| 길의 꽃잎·빛 건너가기·옅은 예고(0190) | `world.rs`(Story::petals·Cue::Trail·petal_tick), `field.rs`(petal_road), `cutscene.rs`(Step::Gloom 빛 보간), `render/fx.rs`(TELE_A 0.45) | 오프닝이 보리 공터 앞 길에 꽃잎을 하나 더 남긴다("이 길로 갔다"). 물든 것의 등장에서 빛이 Naïl에서 그쪽으로 0.9초에 건너간다. 예고 투명도 0.45 |
| 타격 피드백·강타 보상(0168) | `tuning.rs`(KICK_*·BEAT_*·HEAVY_PERFECT·WALL_SLAM_*·CHARGE_PULL_*), `combat.rs`(Feel::kick), `moveset.rs`(heavy_of), `player.rs`(on_beat·combo_beats·perfect), `enemy.rs`(slam), `world.rs`, `render.rs`(draw_enemy 진동·눌림, 킥), `sfx.rs`(ChargeUp·HitTick) | 히트스톱 5/9/13f, 맞은 적 진동·눌림, 맞히면 전진 멈춤 + 카메라 킥 2/3/4px, '틱'. 강타 피해 3·경직 +0.5·넉백 ×1.7 → 벽 쿵 +1, 콩이 등딱지 깨기, 모으는 동안 소리·웅크림·먼지. 박자 타 덤, 이음 다 맞춘 마무리 ×1.25, 다 모인 직후 뗀 강타 +1 |
| 코드 건강(0169 R1~R4) | `sfx.rs`(Queue·flush), `world.rs`(step 단계 메서드·enemy_down·Inventory·Story·RunStats·Hazards·Juice), `render/`(아홉 모듈), `app/`(pause·settings·album·title, start 넷으로), `scenario.rs`(통째 시험) | 로직은 오디오 장치를 모른다 → 시나리오 37개를 600틱씩 돌리는 시험. 동작 변화는 적 쓰러짐 손맛 통일뿐 |
| 코드 건강(0169 R5~R8) | `enemy/`(mod·melee·ranged·boss, `Ctx`), `i18n/`(mod + 언어 열하나), `tools/test/smoke.js`·`smoke-web.yml`, `equipment.rs`(발톱 상수), `profile.rs`·`prefs.rs`·`suspend.rs`(고정 샘플 시험), `tools/art/legacy/`, `build-android.yml`·`build-ios.yml`(main만), `render/actors.rs`(모모 웅크림) | 브라우저 연기 시험이 main push마다 돈다. 동작 변화는 모모 모으기 웅크림 1px뿐 |
| 추상화 깊이(0170 R9~R12·R14) | `player/`(attack·burrow·momo), `shade.rs`, `world.rs`(Story 비공개·StorySeed·beat, strike_feel), `enemy/`(receive·Struck·tongue_reaches), `bosses.rs`, `characters.rs`, `art.rs`(Portrait·Face) | 적이 "맞으면 무슨 일"을 정하고, 보스·캐릭터는 표 한 줄. 동작 변화 없음 |
| Android 키 반복(0171) | `third_party/miniquad`(MainActivity.java·QuadNative.java·android.rs), `input.rs` 테스트 | 패드 버튼을 누르고 있으면 반복 이벤트가 새 누름으로 들어와 땅파기·강타를 못 모았다 → repeat 플래그를 넘긴다. 실기 확인은 사장 |
| 터치 오버레이(0172) | `input.rs`(TouchKind·overlay·any_edge), `characters.rs`(Icon·touch), `render/hud.rs`(draw_touch_icon) | 장치가 터치일 때만 보이고 패드·키를 만지면 사라짐. 버튼은 픽토그램(캐릭터마다). 안내 글은 동작 이름 |
| 저널 새 단장(0173) | `journal.rs`(visible 순서·Cat::color·mobs3·heavy·heavy2·stat::HEAVY), `app/pause.rs`(draw_journal·draw_reward_icon), `app.rs`(Tally 10), `i18n/`(8키 × 11), `assets/fonts/NailCJK-*` | 받을 것부터, 갈래 띠·상태·진행 막대·보상 아이콘·받기 상자. 도감 15종에 맞춘 목표, 강공격 목표 둘 |
| 가장 깊은 곳(0162) | `adventure.rs`(Place::Deep·Transition::EnterDeep·enter_deep), `world.rs`(Deep·Knot·Pull·deep_tick·deep_hit), `tuning.rs`(DEEP_*), `render.rs`(draw_deep·draw_knot·draw_thread·draw_deep_bar), `cutscene.rs`(halmae_meet), `app/stage.rs`(DeepIntro·DeepDone·DeepSleep) | 콩이를 이기고 출구를 밟으면 내려가는 전환으로 지하 130 m 한 방: 할매(때릴 수 없음) + 매듭 셋(체력 6, 강공격 두 배) + 3.2초마다 당기기(황금 점선 예고 0.9초, 받아치면 끊김, 걸리면 1.3초 감기고 가까운 매듭이 다시 조여짐). 세 번 감기면 실에 잠겨 쉼터 모닥불에서 깸, 셋 다 끊으면 반쪽/참/모모 엔딩. 개발 → "가장 깊은 곳" |
| 얇은 장면 채우기(0163) | `cutscene.rs`(tainted_meet_steps·deep_sleep·momo_flee), `world.rs`(Cue::Vanish·Wrap, deep_release_fx), `render.rs`(draw_deep 놓아주기), `app.rs`(AfterCut::Respawn) | 대장·도토·콩이의 등장을 모모와 같은 문법으로(음악 끊김·어둠·팬·'???' 카드·울림), 모모는 꽃잎 속에 달아나 굴로 파고들고, 할매는 셋 다 끊기면 실이 풀리며 흰 꽃잎·빛·눈을 뜬다(3.4초), 세 번 감기면 어둠이 조여 오는 장면 뒤 쉼터에서 깬다 |
| 오프닝의 모모(0164) | `cutscene.rs`(opening, Cue::Eyes·Leave), `world.rs`(petal_at·petal_tick), `app.rs`(도감 '???'), `profile.rs`(key::PETAL) | 어둠 속 눈빛 → 걸어 나옴 → '???' 카드 → 덮쳐 한 방(피해 없음) → "…아니야." → Naïl을 엄마로 착각, "깊은 데로 가지 마. 실이… 들려." → 사라지며 시든 꽃잎을 남김 → 1초 침묵 → 포포가 굳는다. 꽃잎을 밟으면 토스트 + 도감 모모 칸이 '???'로 선다. 건너뛰기는 모모 앞에서 멈춘다(`Step::Checkpoint`, 한 번 더 누르면 끝까지) |
| 굴토끼 가족의 길(0165) | `biome.rs`(Silk), `terrain.rs`, `room.rs`(webs), `route.rs`(Warren·generate(seed, stage, warren)), `species.rs`(CLOVER), `enemy.rs`(Behavior::Clover·Thump·Gather), `world.rs`(cocoons·waves·thread_at·thump_at·Cue::Hatch), `cutscene.rs`(clover_meet·clover_returns), `village.rs`('j'), `journal.rs`·`equipment.rs`(SILK_KNOT), `story.rs`, `tools/art/critters.py` | 모모가 돌아온 뒤 셋째 길 보스 층이 둘(콩이의 문 / 굴토끼 옛 굴). 옛 굴은 흰 실 지대(흰 실 그물: 걸으면 느림, 발톱으로 끊김) 네 방, 끝에 토끼풀(발 구르기 충격파 + 실 그물 / 두 번 차기 / 내려앉기 / 반 아래 품기·실 한 가닥, 아기들의 고치는 못 때림). 이기면 고치가 풀려 가족이 마을로(모모 자리 곁), 저널 "가족" → 흰 실 매듭, 에필로그 한 줄. 개발 → "토끼풀"·"실타래 굴: 흰 실" |
| 패링 | `player.rs`(parry_dir), `world.rs`(parry_feedback), `enemy.rs`(parryable) | 스윙 첫 0.12초 앞쪽: 돌진 경직, 투사체 반사, SP +4(docs/decisions/0020). 창이 열린 뒤 0.45초는 새 창 없음(연타 방지), 성공하면 바로(0038). 곧게 오는 것(돌진·투사체·혀)만 받아치고 둘레 공격(내려앉기·튀어나오기·구름·돌)은 못 받아친다 — 예고가 황금 띠·점선 ↔ 진홍 고리 + ✕로 갈린다(0160) |
| 스테이지 고리 | `adventure.rs`, `camp.rs`, `route.rs`, `dungeon.rs`(Params), `app/stage.rs` | 필드 → 공터 보스 → 쉼터 → 길 지도(층 3 + 보스, 엇갈리지 않는 갈래, 다음 층만 보임) → 굴(굴·둥지·저장고·샘·보스) → 다음 쉼터. 쉼터: 체력 가득·저장 고리·포포 가게(씨앗)·보리·빈 자리, 공격 버튼 = 말 걸기. 쓰러지면 쉼터, 길은 새로. 단위 테스트(길 규칙 3, 고리 2)(docs/decisions/0030, 0031) |
| 마을 | `village.rs`, `adventure.rs`(World::village, Place::Village), `render.rs`(큰 나무·꾸미기·할매), `app/stage.rs`(꾸미기 가게·할매·뿌리 길) | 튜토리얼 뒤 시작·엔딩 뒤 돌아오는 허브. 큰 나무 잎 = 깬 횟수, 포포 꾸미기 가게(마을 씨앗 6가지), 보리·할매(실뜨기)·빈 자리(0045) |
| 모모 보스(세 번) | `enemy.rs`(Momo·Under·Pop), `species.rs`(MOMO), `dungeon.rs`(Params::boss), `adventure.rs`(MomoIntro·MomoBeaten), `app/stage.rs`(momo_beaten), `cutscene.rs`(momo_meet·momo_flee·momo_returns), `tools/art/critters.py`(MOMO_BLOOM) | 첫 길 끝방. 만날수록 한 수씩(뛰기·차기 → 굴 숨기 → 이어 뛰기), 쓰러지면 그 런은 보통 무리. 이길 때마다 빈 자리 꽃잎, 세 번째에 돌아와 마을 그루터기에 앉음(참 엔딩 조건 `momo`). 보스 막대를 화면 위로(0046) |
| 도토 보스(한 번) | `enemy.rs`(Doto·doto_next·Bury), `species.rs`(DOTO), `world.rs`(Mine·mines), `render.rs`(draw_mine), `adventure.rs`(DotoIntro·DotoBeaten), `app/stage.rs`(doto_beaten·큰 나무 독 발톱 줄), `cutscene.rs`(doto_meet·doto_returns), `tools/art/critters.py`(DOTO·DOTO_BACK), `portraits.py`(칸 30~32) | 둘째 길 끝방. 도토리 부채꼴(패링으로 되받기) → 높이 뛰기 → 도토리 묻기(발밑 + 둘레, 고리 둘, 파내기·굴로 넘김), 반 아래면 수가 늚. 이기면 씨앗 자리를 기억(`doto_memory`, 참 엔딩 조건 둘째), 마을 그루터기 밑. 큰 나무에서 독 발톱 깨우기(120, 런마다 가방에), 저널 목표 둘(0052) |
| 실의 꿈(0055) | `dream.rs`(보스 넷·열림·기록 키), `app/dream.rs`(고르기 창·꿈 싸움·깨어나기·HUD), `village.rs`(X), `adventure.rs`(Interact::Dream), `render.rs`(실 고리), `world.rs`(dream: 떨어뜨리지 않음) | 마을 할매 곁 실 고리에서 이긴 길의 주인과 빈 방에서 다시 싸움(모모는 세 번째 모습), 가장 빠른 기록·★(안 맞고 이김), 넷 다 이기면 이어서 꾸기, 얻는 것 없음, 저널 둘 |
| 콩이 보스(한 번) | `enemy.rs`(Kongi·Guard·kongi_next·shell_hit·break_guard·on_parried), `species.rs`(KONGI), `world.rs`(등딱지 튕김·열기), `player.rs`(recoil), `adventure.rs`(KongiIntro·KongiBeaten), `app/stage.rs`(kongi_beaten), `cutscene.rs`(kongi_meet·kongi_returns), `tools/art/critters.py`(KONGI)·`portraits.py`(칸 35~37) | 셋째 길 끝방. 구르기(벽에 튕기고 어질어질) ↔ 웅크려 버티기(발톱은 튕기고 바로 굴러 옴 — 패링 거울), 솟구치기·드릴로 등딱지 열기, 패링하면 구르기 멈춤. 이기면 바위 발톱(런마다 가방에), 마을 바위 곁, 저널 "깊은 곳의 문", 도감 목록 넘기기(0054) |
| 지도 그림(0062·0063) | `terrain.rs`(굽기·장식 자리·흙길), `render.rs`(구운 지도 + 바뀌는 칸), `dungeon.rs`(shape_room·KEEP) | 장소가 바뀔 때 지도를 한 장으로 굽는다: 잡음으로 물결치는 둥근 벽 윤곽, 3/4 벼랑(지층·뿌리·늘어진 잎)과 윗면(덤불·뿌리·돌), 바닥 큰 띠·반대 재질 얼룩·던전 흙길·그늘, 무리 지은 장식. 던전 방은 모서리·가장자리 혹·바위 무리로 굴처럼(문 앞 길·적 자리 보호, 이어짐 검사). 마을 큰 나무는 깨울 재주가 있으면 빛난다(0061). 굽기는 `terrain::Baker`로 프레임마다 12ms씩 나눠 하고 그동안 화면 검정·월드 틱 멈춤, 최근 장소 세 장은 남긴다(0068, 브라우저 가장 긴 프레임 133~150ms → 33ms) |
| 모모 플레이어블(0056) | `equipment.rs`(Character::Momo·WeaponKind::Kick·뒷발), `moveset.rs`(KICKS), `player.rs`(kit·Hop·Chain·stomp), `tuning.rs`(HOP_*·CHAIN_*), `app/pick.rs`(누구와 내려갈까), `app/stage.rs`(뿌리 길·reach_deepest), `cutscene.rs`(ending_momo·플레이어 배우), `illust.py`(momo_deep), `render.rs`(draw_momo_player) | 모모가 돌아온 뒤 마을 뿌리 길에서 Naïl/모모를 고름. 같은 버튼에 뒷발 3타·깡충(무적)·이어 뛰기(세 번 밟기), 발톱은 줍지 않음. 깬 수·실뜨기는 캐릭터마다, 모모 런은 모모 보스 없음, 가장 깊은 곳 → 모모 장면(`momo_deep`, 엔딩 깃발과 따로), 포포·할매 한마디, 저널 "꽃잎을 따라", 앨범 그때 10장(지금 11장) |
| 작은 것들(0053) | `camp.rs`(z), `cutscene.rs`(susu_meet·Portrait::SUSU), `app/stage.rs`(수수 말 걸기), `illust.rs`·`tools/art/illust.py`(doto_memory), `knots.rs`(6~10), `story.rs`(TalkDoto·진행 줄), `tools/art/critters.py`(SUSU)·`portraits.py`(칸 33~34) | 첫 쉼터의 땅강아지 수수가 새싹 발톱을 만들어 줌(런마다 가방에, 저널 "파는 손"), 도토 기억 일러스트(앨범 그때 9장, 지금 11장 5×2 굴리기), 실뜨기 10단계(걸음·보스 체력·패링 창·굴파기 대기·지렁이), 진행을 따르는 이웃 말 7줄 |
| 반딧불·큰 나무 재주 | `talents.rs`, `app/stage.rs`(bank_glowbugs·update_tree·draw_tree), `player.rs`·`world.rs`(재주 배율), `adventure.rs`(Interact::Tree), `sfx.rs`(드릴 소리 셋) | 쉼터·런 끝에 반딧불을 마을로. 마을 큰 나무에서 자리 넷 × 갈래 둘(굴파기·패링·드릴·먹기), 깨우기는 반딧불, 바꿔 끼기는 공짜(0047) |
| 임시 그림 다듬기 | `tools/art/portraits.py`(칸 12~17), `tools/art/illust.py`(bands, momo_bloom), `cutscene.rs`(Portrait::MOMO·HALMAE·MOM) | 돌아온 모모·누룩 할매·엄마 초상화, 일러스트 띠 경계 2×2 섞기, 모모 일러스트 몸·눈(0049). 지금은 나노바나나가 대신함(원본 없을 때만) |
| 작은 정리(0051) | `app.rs`(상태 탭·도감 쪽), `story.rs`(TalkBori·TalkMomo), `portraits.py`(털빛 초상화, 12칸 줄), `illust.py`(큰 나무 가지) | 상태 탭에 재주, 대화창 Naïl 털빛, 도감 이야기·공격·요령 쪽(확인으로 넘김), 진행을 따르는 보리·모모 말, 큰 나무 가지. 초상화·일러스트는 지금 나노바나나가 대신함 |
| 초상화 v4(0070·0071) | `tools/pixel`(점 도구·편집기), `tools/art/portraits.py`(fur_clumps·silhouette_tufts, 기본 v4) | 털 인물(Naïl·엄마·보리·모모·도토·포포)에 털 뭉치 경계·크기 섞인 실루엣 삐침·작은 하이라이트·빛 쪽 외곽선 한 단, Naïl 원통 목도리·평평한 발톱·정수리 한 가닥. 대화창에서 헤드리스 확인. 지금은 나노바나나가 대신함(칸 4 그림자 말고, 원본 없을 때만) |
| 일러스트 바탕(0072·0073) | `tools/art/illust.py`(set_style·SCENE_STYLE·pastel_bg·ambient_tint), `tools/art/sources/`, `tools/pixel/pxl.py`(pixelate --cel) | 그때 열 장 모두 v4(콩이의 문은 0085 실루엣 방식): 여섯 장(흙더미·큰 나무·마지막 꽃·엄마 굴·보리·모모)은 파스텔·수채·유화(CC0·퍼블릭 도메인) 바탕을 도트화, 땅속·꿈 네 장은 v4 규칙만. 지금은 나노바나나가 대신함(원본 없을 때만) |
| 저널 | `journal.rs`, `app.rs`(Tally·draw_journal), `app/stage.rs`(sync_journal·claim_journal·물웅덩이), `village.rs`(자리 7~9·W), `equipment.rs`(장신구 8), `tools/art/nail.py`(털빛 3) | 런을 넘어 남는 목표 38개(0048 때 31개, 보스·꿈·모모 등으로 늘어남)(이야기·싸움·탐험·이웃·마을), 앞 목표 뒤에 보임, 받기로 반딧불·씨앗·저널 전용 꾸미기 3·털빛 3·새 장신구 3(0048). 장신구는 받으면 가방에·런마다 들고 시작, 받은 뒤 보상이 어디 있는지 표시(0050) |
| 오늘의 굴 | `daily.rs`, `adventure.rs`(camp_start_seeded), `app.rs`(TitleItem) | 날짜(UTC) 시드, 첫 쉼터에서, 가장 먼 쉼터·완주 기록, 타이틀 줄에 날짜·기록, 한 달 넘은 기록 지움(0044) |
| 실뜨기 | `knots.rs`, `enemy.rs`(max_hp·windup·apply_knots), `world.rs`(knots), `app.rs`(begin_run·KnotPick) | 첫 클리어로 1단계, 가장 높은 단계로 닿으면 다음 단계. 시작할 때 고르기. 매듭: 적 체력 ×1.25 · 예고 ×0.8 · 산딸기 ×0.6 · 가게 ×1.5 · 먹기 대기 ×1.6(0043) |
| 런의 끝 | `tuning.rs`(FINAL_STAGE), `route.rs`(is_final), `adventure.rs`(AdvEvent::Deepest), `app/stage.rs`(reach_deepest) | 필드 + 길 3개, 셋째 길 보스 칸 = 가장 깊은 곳 → 반쪽/참 엔딩(조건 깃발) → 타이틀, 클리어 수 기록, 반쪽 엔딩 뒤 포포 한마디(0042) |
| 기록·다회차 | `profile.rs`, `story.rs`, `app.rs`(start·on_death·album·설정), `web/index.html`(nail_save), `player.rs`(resist) | 깃발 + 수 기록(웹 localStorage·데스크톱 파일·Android 앱 폴더), 튜토리얼 뒤 새 런은 바로 필드·보리 뒤엔 대장 없음, 런 사이 한마디(가장 구체적인 줄·한 번), 도감 유지, 앨범(타이틀), 포포의 응원(20%+2%/쓰러짐, 최대 80%), 기록 지우기(0041) |
| 컷씬 일러스트 | `illust.rs`, `cutscene.rs`(Step::Illust·Narrate, cut_to, 이야기 장면), `render.rs`(draw_illust), `tools/art/illust.py` | 2배 밀도 한 장 그림(기본 640×360, 패닝 장면은 더 큼) + 느린 패닝 + 입자, 내레이션 띠. 처음 8장(0039) → 지금 11장(+도토 기억 0053·모모 깊은 곳 0056·콩이의 문 0085), 모두 이야기 컷씬에서 나오고 앨범에 모인다. 개발 "이야기 일러스트"는 바로 보기용. 그림은 11장 모두 나노바나나(0125~0136), 내레이션 아홉 언어(0037, 0039) |
| 튜토리얼 보스 | `enemy.rs`(Chief), `species.rs`(CHIEF), `field.rs`(공터), `tools/art/chief.py`, `render.rs`(보스 막대) | 들쥐 대장 보리: 돌진 ↔ 뛰어 찍기, 반 아래면 생쥐 떼, 공격 중 슈퍼아머, 벽에 박으면 2.2초 틈. 32×32 그림, 초상화 3, 처치 컷씬 → 쉼터의 이웃. 헤드리스로 공터 잠금·막대·처치 컷씬·쉼터·가게·길 지도 확인(docs/decisions/0032) |
| iOS 빌드 1단계(0086) | `.github/workflows/build-ios.yml`, `ios/Info.plist`, `ios/icons/`, `profile.rs`(iOS 저장 자리) | macOS 러너에서 `aarch64-apple-ios` 빌드 → 손으로 짠 `.app` → 서명 안 한 `.ipa` 아티팩트. 실기 실행은 미확인(서명: Sideloadly·AltStore·TestFlight는 나중에). 패드는 iOS에서 아직 안 됨(gilrs) |
| 앱 아이콘(0087) | `tools/icons/make_icons.py`, `android/res/mipmap-*`, `Cargo.toml`(res·icon), `build-android.yml`(application-icon 검사) | iOS·Android 아이콘(그림은 0123에서 큰 나무·뿌리 실루엣으로 바꿈). Android는 옛 mipmap + 적응형(8+). CI가 APK에 아이콘이 없으면 실패 |
| 솟구치기·마무리 타(0090) | `tuning.rs`(EMERGE_*·CLAW_*), `moveset.rs`, `world.rs`(emerge_push·충격), `sfx.rs`(Impact) | 첫 바깥 판정 2·3("숨는 용도", "그저 그렇다") 뒤: 솟구침 반지름 22·피해 2(0158부터 톡 1·모으면 2)·넉백 340 + 반지름 44 밀쳐내기(잡몹만), 두 겹 고리·쿵. 마무리 타 넉백 ×1.35~1.45, 멈춤·흔들림↑, 충격 고리·쿵. 단위 테스트(밀쳐내기) |
| HUD·퀵 슬롯(0091) | `render.rs`(hud_plate·draw_quick_slots·draw_hud_bars), `items.rs`(Bag::take·flash) | 바탕판 셋·테두리 글자, 게이지 34px, 퀵 슬롯 22px·아이콘 2배·고른 칸 금빛 2px, 다 쓰면 가진 다른 것으로 저절로 + 1.2초 깜빡임(쓰러져 지렁이가 쓰여도). 가게 창 밑 HUD 숨김. 단위 테스트 |
| 화면 × 아홉 언어 점검(0089) | `ui.rs`(check, F3), 헤드리스 순회 | 넘침·겹침·잘림 검사기, 가게·꾸미기·실뜨기·조작·도감·저널·가방·상태·큰 나무·튜토리얼 힌트·말풍선 고침, 스페인어·러시아어 몇 줄 줄임 |
| 인트로·웹 로딩(0092) | `app.rs`(draw_intro), `loading.rs`(prefetch·bytes·texture·image·step), `art.rs`·`sfx.rs`(files), `main.rs` | 인트로 바탕 = 어둠 아래 타이틀 그림(줄마다 걷힘). 시작 파일 44개를 한꺼번에 받고 파일마다 막대: 지연 70ms·9Mbps 흉내에서 7.7초 → 4.4초 |
| 튜토리얼 아이템 단계(0093) | `tutorial.rs`, `world.rs`(start_eating, toast_eat_full) | 저장 지점에서 체력이 가득 차 막히던 것: 단계 동안 한 칸 비우고 산딸기를 둔다, 가득할 때 먹으면 알림 |
| 잠긴 문(0094·0096~0099) | `render.rs`(draw_root_gate·빛 패스), `sunny.rs`(FENCE_H·FENCE_V), `room.rs`(door_anim·tick_doors), `world.rs`(열릴 때 실 풀림) | 팩 나무 울타리 두 겹(가운데 칸만: 남북 39,2 · 동서 40,3, 가로대가 벽에서 벽까지, 팩이 없으면 물든 뿌리) + 가로대를 감는 보라 실, 0.35초에 솟아 닫히고 0.5초에 가라앉으며 실이 풀린다, 어둠 속에서도 실이 빛남 |
| 어둠 지도·야광버섯·그림자(0095·0100) | `render.rs`(dark_rt·cut_material·light_sources·draw_cast_shadows·draw_mushrooms), `art.rs`(light), `biome.rs` | 빛 자리마다 어둠을 빼는 렌더 타깃, 버섯 빛 64px·세기 0.28(은은하게 넓게), 심층 버섯 14%·뿌리 지대 4%, 빛 반대쪽으로 눕는 그림자 |
| 해·모닥불 그림자(0102~0106, 시험) | `render.rs`(draw_sun_shadows·shadow_dir·sun_shear·silhouette_material, draw_sorted의 sun), `sunny.rs`(stamp_sun_shadow), `terrain.rs`(Baker::fires), `tuning.rs`(SUN_SHADOW_*·FIRE_SHADOW_*·DECOR_LIFT_*) | 서 있는 것의 모양을 발 줄 축으로 눕혀 렌더 타깃에 모아 한 번에 얹는다. 밝은 곳은 해(오른쪽 아래), 모닥불 곁은 불 바깥쪽(150px, 멀수록 길게, 불이 아래면 더 길게 0104, 어두운 곳은 바깥으로 옅게). 모닥불 돌은 돌만 키운 모양(뒤쪽은 짧게, 0105). 그림자 패스는 몸만(빛·표시·반투명·바닥 것 제외, 밝은 곳은 타원 그림자 끔, 0106). 구운 꽃·조약돌·풀포기는 높이 비율만큼 짧게 |
| Naïl·포포·모모(돌아옴·물듦)·보리(돌아옴·물듦)·누룩 할매·엄마·도토(돌아옴·물듦)·수수·콩이(돌아옴·물듦) 초상화 나노바나나(0107~0122, 이웃 그림자 칸 4 말고 모두) | `tools/art/sources/ai/*.png`, `tools/art/ai_portraits.py`, `tools/art/portraits.py`(sheet), `docs/art/nano-banana-prompts.md` | 표정 원본(✦는 `strip_logo.py`가 원본에서 지움, 0133) → 배경 투명·88×88·한 팔레트(눈 8색 따로)·털빛 셋(주둥이·발톱은 자리로 지킴). 포포: 틀 맞추기(높이 92%)·채도 높은 색 따로. 모모: 검은 선 → 자두빛, 틀 1.22(귀 끝을 칸 맨 위에, 가슴을 칸 아래로), 가슴이 지워지지 않게 아래 끝 번지기 끔. 물든 모모: 같은 틀, 실은 굵게·위로, 눈은 선명한 초록으로 고쳐 그림, 흰빛은 보랏빛으로 물들임. 보리: 시트를 통통하게 고쳐 그림, 틀 그대로. 들쥐 대장: 보리를 고쳐 그림, 눈 팔레트 안 뗌(보라 기운 털), 빨간 눈 밝게. 할매: 시트의 둥근 얼굴 판으로 표정 짝, 둘러싸인 배경 틈 지움. 엄마: Naïl 초상화를 함께 넣고 털을 짙고 매끈하게, 앞발은 따로 고쳐 그림. Naïl 결의: 덜 표독스럽게 다시 그림, 팔레트를 얼려 다른 칸은 그대로. 도토: 시트에서 조끼·보라 꽃·리본 빼고, 몸이 잘린 줄까지만 써 배 구멍 막음. 물든 도토: 도토를 고쳐 그림, 호박색 눈 밝게. 수수: 생김새 글을 새로 쓰고 삽발을 칸 아래에. 콩이: 3/4 모습 기준, 토끼 귀가 아닌 뾰족한 귀. 물든 콩이: 하늘빛 눈 밝히기, 배경색 잡기를 튀는 점에 강하게. |
| 앱 아이콘: 큰 나무·실에 얽힌 뿌리 실루엣(0123) | `tools/icons/make_icons.py`(`art`), `ios/icons`, `android/res/mipmap-*` | 64칸 코드 그림, 금빛 원 앞 나무 + 보랏빛 앞 뿌리 + 뿌리 다발을 묶는 실 고리 둘. 일러스트에서 자른 것과 견주어 이것을 유지(48px에서 읽힘) |
| 나노바나나 일러스트(0125~) | `tools/art/sources/ai/illust/*.png`, `tools/art/ai_illust.py`, `tools/art/illust.py`(main) | `great_tree`·`mother_burrow`(실을 지운 자리에 맨 나무껍질을 그리게 해 집 장면에서 실 제거)·`bori_freed`(물든 모습 그림을 빼고 시트대로 다시 그림)·`momo_bloom`(구도 그림을 빼고 비율을 글로)·`halmae_deep`(Naïl을 대화창 위로)·`last_bloom`(이웃 넷을 패닝 끝 대화창 위로)·`sprout`·`molehill`·`doto_memory`·`momo_deep`(작은 모모의 색은 `FOCUS` 팔레트, 0135)·`kongi_door`(0136)·타이틀 바탕 `title`(0137, 작은 캐릭터는 `FOCUS` 자리에서 어두운 선도 지킨다). 컷씬 일러스트 열하나 + 타이틀 모두 나노바나나. ✦ 로고는 원본에서 한 번 지운다(`tools/art/strip_logo.py`, 흰색 얹기 되돌리기, 0133). 게임 칸마다 가장 흔한 색 + 채도 높은 색은 색상별 팔레트·칸 문턱 15%(작은 빨간 눈을 지킨다). |
| 웨이브 | `world.rs`, `scenario.rs` | 지금은 개발 시나리오(아레나·웨이브 3부터)에서만. 적을 다 잡으면 1.2초 뒤 다음 웨이브(1C+1S → 2C+1S → 2C+2S → 3C+2S). 명세엔 없지만 5분 플레이 판정을 위해 넣음 |
| 리팩터링 R15~R18(0230) | `app/after.rs`(After·ORDER·Gate·Do), `app/flow.rs`(character·veteran·first/First·fall/Fall·back_to_village), `world.rs`(Hazards::clear_fight·clear_all·place_cocoons, tick_player → player_rumble·player_events·drill_fx), `art.rs`(frame·BigSheet::run4·CHIEF·Art::chief·has_run·run), `render/help.rs`(PicCtx·pic_*), `tools/art/sprite_ai.py`(sheet4·silhouette·lum·ramp), `tools/art/check_rebuild.py` | 동작은 그대로. 고친 것 둘: 장소를 옮길 때 지뢰·충격파도 치운다, `doto.png` 재현(색표 접기를 다수결 뒤로). 앱 흐름 시험 11 |

확인한 것: 데스크톱(Xvfb) 실행 캡처로 렌더 방향·공격 반원·굴파기·히트박스 표시, 헤드리스 Chromium으로 웹 빌드 로드·입력·콘솔(mismatch 없음). 확인 못 한 것: 판정 2·3 다시(0090 뒤), 데스크톱 실기 패드, 소리의 실제 느낌(컨테이너에 오디오 장치 없음).

## 사람이 할 일 (다음 세션 전)
2026-10-25에 정리했다. 끝난 것(Netlify 시크릿, 플레이 판정, Sunnyside 받기·라이선스·비공개 에셋 레포, 화면 이동·스토리·다회차 결정, 저장 파일, 기술 해금·가게 화면)은 지웠다. 기록은 각 결정(docs/decisions)에 있다.

### 결정 대기 한눈에(2026-10-29 정리) → 사장 답 0232
- [x] 팔 것인가 → **판다(Steam)**. v0.6 → **팔 수 있는 판**(0232 범위 1~8)
- [x] 레포 → 홈페이지만 공개 레포로 분리, 게임은 비공개(에셋 레포처럼). 웹 무료판 → 데모로. 그림 → 바꾸지 않음(구체적 AI 공개). 그림 제품 → Gemini 앱(무료)
- [x] v0.5.4 배포(2026-10-29). 쉼터 갈래 → 안 함, "캠프 하나 더" = 넷째 길 → 다회차 깊이(v0.6 뒤)
- [x] v0.5.5 배포(2026-10-29): 판정에서 나온 할 일 넷(떨어진 것 표시·터치 아이콘·보스 2단계 0233·고른 소리 0234)
- [x] 보물 상자를 버튼으로 열기(사장 2026-10-29): 곁에서 공격 버튼(쉼터 말 걸기와 같은 "{버튼} 열기" 말풍선, 열한 언어), 닿기만 하면 안 열린다. 시험 `chest_opens_with_button_not_by_touch`
- [x] 리팩터링 R19·R20·R24(0235): 소리 표 이름으로·플랫폼마다 한쪽 소리만 싣기(APK 약 24MB 줄어듦)·안내 띠 하나로
- [x] 경제(0236, 사장 "씨앗도 반딧불도 너무 흔하다"): 떨어뜨리는 수를 줄임 — 한 런 반딧불 422 → 239, 씨앗 78 → 48(길마다 ≈15, 쉼터마다 심장 또는 장신구 하나). 사장 확인: 두세 런 해 보고 가게에서 고민이 생기는지
- [x] 설정 → 라이선스(크레딧 대신, 사장 2026-10-29): 제3자 고지를 앱 안에서 굴려 본다
- [x] 설정 창 정렬(0239, 사장 "아래 세 줄 들여쓰기 필요 없다"): 모든 줄 같은 왼쪽 선, 묶음 셋(값·다른 화면 →·기록 지우기 붉은 글) 사이 가는 선, 돌아가기 줄 빼고 안내 띠에 "{뒤로} 닫기", ▲▼는 제목 줄로
- [x] Naïl·콩이·큰 나무 다시 그리기(0238, 사장 고름 Naïl 1·콩이 3): Naïl 28칸·키 22px, 콩이 키 22px, 큰 나무 `tree.png` 잎 단계 다섯 칸(시든 줄기 + 잎 덩어리). 사장 확인: 걷기·물든 콩이 얼굴·나무 단계. 남음: 파 내려가기·초상화 속 Naïl
- [x] PC 빌드(0240): Windows·macOS(겸용 .app, 임시 서명)·Linux, 타이틀 끝내기·설정 전체 화면(F11). **v0.6.0 배포(2026-10-29, 사장 결정)** — 0232 나머지는 v0.6.x. 사장 확인: Windows PC·Mac에서 켜기(경고 넘기기)·패드·전체 화면
- [x] Naïl 걷기(0241): 원본 걷기 네 칸이 발은 그대로·앞발만 깜박여 미끄러지듯 → 한 칸에서 발을 번갈아 딛는 네 칸 + 1px 들썩임. 홈페이지 첫 화면 버튼 두 줄(브라우저·Android·iOS 자리 / Windows·macOS·Linux). **v0.6.1 배포(2026-10-29, 사장 결정)** → 뒷걸음처럼 보여(든 발이 뒤로) v0.6.2에서 고침. 콩이·보리 걷기도 같은 방식 + 적 걷기 칸은 걸은 거리로(v0.6.3, 몸이 칸마다 달라 꿈틀거림·느린 보스가 제자리 파닥임). 사장 확인: 걸음이 자연스러운지
- [x] Naïl 머리 키우기(0242): 시험(원본에서 머리 1.35배 + 큰 눈 `chibi`) → 그 비율로 새로 받은 v4 시트(2.5등신·큰 눈)로 바꿈, 키 약 24px, 눈은 `eye` 도장. v0.6.3에 함께. 사장 확인: 게임 속 크기에서 귀여운지·걸음
- [x] Naïl 크기 줄이기(0242 덧붙임): 외곽선까지 키 약 21px(모모 23·도토 25보다 작게, 사장 "아기 느낌"), 눈 도장 자리 자동(`auto_eye`). 지렁이로 일어나기(0243): 0.9초 눕고 지렁이 먹고 터지며 일어난다. 사장 확인: 크기·부활 연출
- [x] 넓은 굴(0244, 사장 "지도가 너무 적은 방들로"): 굴마다 방 6개쯤(보통·둥지·보스 4 → 6, 실타래 3 → 5, 저장고 3 → 4), 화면 두 칸짜리 넓은 방(카메라가 따라감, 두 칸 적이 함께), 방 지형 넷(굴 그대로·뿌리 기둥·골짜기·돌밭). 잡몹 반딧불·씨앗은 0.65배로 경제 유지. 안내 그림 반딧불·씨앗 속도 0.4배. 홈페이지 스크린숏·영상을 실제 굴에서 다시(20초). **v0.6.5 배포(2026-10-29, 사장 결정 — 처음엔 v0.6.6으로 냈다가 "건너뛰지 말라"에 지우고 v0.6.5로 다시)**. 사장 확인: 한 런 길이·넓은 방 싸움
- [x] 사장 시험 고침(0245): 도토·모모 런이 엔딩·올라오기·마을까지 그 캐릭터, 할매 방 투명벽·안 보이던 매듭(화살표·안내 한 줄), 굴파기 장신구는 Naïl만, 도토 파 내려가기 귀·꼬리, 도토 걷기 네 칸, 큰 나무 밑동. 사장 확인: 도토 엔딩·할매 방·걷기
- [x] 장신구 드롭(0246 1): 상자·엘리트·싸움방(5%)의 장신구가 바닥에 떨어져 주워야 얻는다. 모모·도토 무기·캐릭터 장신구·재주 늘리기는 0246 제안 — 사장이 고른다
- [ ] 귀·손으로 확인(사장, v0.5.5): 음악·환경음·효과음 크기 균형, 발소리가 거슬리는지, "고리 ?" 곡의 이음매, 보스 2단계 곡 넘김과 세기(`tuning::BOSS_P2_*`), 핸드헬드 메모리·멈칫(긴 곡 175초 ≈ 61MB)
- [x] Android → 정식 계정 $25. iOS → TestFlight까지. Steam 페이지·Next Fest 시기 → v0.6 뒤 결정
- [ ] 그림 받기(사장): 마을·쉼터 C 꾸미기 열하나 → D 아기 굴토끼(0224, `docs/art/hub-sprite-prompts.md`). B 큰 나무는 0238에서 끝(두 장 → 단계 다섯)
- [ ] v0.6 사람 몫(0232): Steamworks 가입·$100·W-8BEN, Android Developer Console $25, Apple $99/년, 새 공개 레포 이름·비공개 전환 시점, 출시 전 번역 검수 외주(원어민 없음)

### 판정표(2026-10-29 정리, 72칸 → 14) — 직접 해 보고 답할 것. Claude가 하나씩 묻는다
1. [x] **싸움 손맛**: 박자 콤보(0.42초 템포·창·스냅·고리·종소리·음표), 강공격(모으기·떼는 순간)·딱 맞춘 강타, 받아치기 반응, 타격 피드백(카메라 킥·진동·벽 쿵), 1·2타 발 번갈아·마지막 타, 솟구치기 밀쳐내기(0090·0161·0167·0168·0174·0183·0184·0189·0205) — **좋음(사장, 2026-10-29)**
2. [x] **굴파기**: 톡(파고들기 → 대시 → 폴짝)과 길게 누르기가 구분되는지, 힘 모으기 0.45초·솟구침 세기, 누르고 있기(0101·0158·0160) — **좋음(사장)**
3. [x] **드릴·스틱**: 드릴 겨누기가 엉뚱한 적에 끌리지 않는지·속도 270, 아날로그 스틱 이동 세기·메뉴 한 칸씩(0139·0143·0149) — **사장: 드릴 속도는 아직 조금 빠르지만 괜찮은 정도. 닫음**
4. [x] **위험 표시**: 적 예고(설정 "공격 예고" 기본값이 알맞은지), 덤불·떨어지는 돌·첫 찔림, 부술 것이 보이는지, 색약(0150·0198·0205) — **좋음(사장)**
5. [x] **보스 여섯**: 대장·모모·도토·콩이·토끼풀·할매(가장 깊은 곳 당기기·매듭)의 수와 세기, 실의 꿈 — **사장: 전체적으로 조금 약하다 → 할 일: 체력 반 아래에서 2단계, 경직을 조금 줄이기(아래 "판정에서 나온 할 일")**
6. [x] **모모·도토로 하기**: 깡충·달리기·멀리뛰기, 던지기·묻기, 재주·장신구, 고르기 창(0142·0195·0209) — **좋음(사장)**
7. [x] **런 구조·경제**: 한 런 길이, 실뿌리 셋(0210)이 가혹한지, 반딧불·씨앗 값, 부술 것 개수·빈손, 실뜨기·오늘의 굴 — **좋음(사장)**
8. [x] **그림**: 새 그림(보스·이웃·잡몹·소품·마을), 걷기 네 칸, 초상화·일러스트·타이틀·앱 아이콘, 특수효과, 해·모닥불 그림자(시험 — 빼자고 하면 끈다), 울타리·버섯 빛 — **좋음(사장). 해·모닥불 그림자 시험은 유지 확정**
9. [x] **연출·이야기**: 오프닝의 모모, 보스 등장·돌아오기, 굴토끼 가족, 단면·지대 카드, 엔딩 뒤 올라오기, 튜토리얼·이웃 말투 — **사장: 모모 꽃잎을 줍는 건지 잘 모르겠다 — 이펙트 필요. 산딸기·쥬시 지렁이 줍기도 → 할 일**
10. [x] **소리**: 새 곡(타이틀·마을·쉼터), 박자 '똑', 강타·피격 소리 — **사장: 음악·효과음이 더 필요. 여러 후보 풀에서 직접 들어 보고 고르고 싶다 → 할 일**
11. [x] **UI·안내**: 안내 탭·그림, 타이틀·키캡·단추감, HUD·퀵 슬롯, 앨범·목록 스크롤 — **사장: 터치스크린 아이콘이 아직 다듬어지지 않았다 → 할 일**
12. [x] **기기·저장**: Android(장소 바꿀 때 멈칫, 설정·기록이 남는지, 덮어 설치), 이어하기·슬롯·저장 지점, iPhone Safari·터치 — **좋음(사장)**
13. [x] **쉼터·마을 걸어 보기**: 길·이웃·숨은 자리 찾기, 굴 입구 곁 시작, 모모 런 배웅 — **좋음(사장)**
14. [x] **번역 원어민 검수**: 프랑스어·스페인어·독일어·러시아어·번체·폴란드어·브라질 포르투갈어(원어민이 있을 때) — **원어민 없음 → 출시 전 언어별 검수 외주(v0.6 사람 몫, 0232)**



### 판정에서 나온 할 일(2026-10-29) → 넷 다 v0.5.5로 배포(2026-10-29)
- [x] **떨어진 것 표시**(판정 9, 사장 "줍기 연출이 아니라 떨어진 아이템에 대한 표시"): 줍는 것 모두(꽃잎·산딸기·쥬시 지렁이·씨앗·뿌리 심장·무기)에 같은 "주울 수 있다" 표시 — 발밑 옅은 빛 자리 + 1.2초마다 퍼지는 고리 + 1.6초마다 네 갈래 별, 어두운 지대에선 별을 어둠 위에 한 번 더(`render/props.rs draw_pickup_mark`, `tuning::PICKUP_*`). 개발 "떨어지는 것 확인" 왼쪽 줄로 본다. 사장 확인
- [x] **터치 아이콘 다듬기**(판정 11): 선으로 그리던 픽토그램을 12×12 점 그림(`render/hud.rs icon_mask`)으로 — 흰 채움 + 어두운 1점 외곽선, 정수 배율로 찍어 흐림이 없다. 열두 아이콘(발톱·발·흙더미·뛰기·드릴·이음·도토리·멀리 뛰기·묻기·산딸기·바꾸기·메뉴), 시험 `touch_icons_are_12_by_12`. 사장 확인
- [x] **소리 후보 듣기**(판정 10, 0234): CC0 후보 142개(26칸) 듣기 페이지에서 사장이 다 고름 → 넣음. 음악은 옛 곡과 새 곡을 둘 다(상황으로 가르기·런마다 번갈아, 보스 2단계에서 곡 넘김), 새 칸(실 지대·가장 깊은 곳·엔딩), 환경음 층(들판·굴·깊은 곳), 효과음 열(발소리·회피는 새로). 사장 확인: 크기 균형·발소리·고리 이음매·핸드헬드 메모리
- [x] **보스 2단계**(판정 5, 0233): 다섯 보스는 체력 반 아래에서 한 번 깨어난다(예고 0.85배·쉬기 1.4배 빨리·걸음 1.15배·멈칫 0.6배, 붉은 발밑 고리·불씨, 막대 반 눈금), 경직은 1단계도 0.35 → 0.3. 모모는 2단계면 첫 만남에도 굴로 숨고 뛰기 두 번. 할매는 매듭 하나 남으면 당기기 두 번. 개발 "보스 2단계: 대장/모모". 사장 확인(수치는 `tuning::BOSS_P2_*`)

### v0.3.0 마일스톤(0081, 완성도) — 사람 몫
- [x] 세션 네트워크 허용 도메인에 `kenney.nl`·`opengameart.org` 추가
- [x] **v0.1.11 귀로 판정**(0081 진행 2): 곡이 장소에 맞는지(타이틀·마을·들판·쉼터·뿌리·심층·보스), 고리 이음매가 들리는지, 음악과 효과음 크기 균형, 메뉴 소리가 거슬리지 않는지, 맞음·쓰러짐·다침 변형이 반복으로 들리지 않는지, 폰 스피커·이어폰. 바꾸고 싶은 곡·소리는 "어디 → 무엇"으로 알려 주면 표만 고친다 — **정리(2026-10-29): 닫음: 사장 "귀 좋음"(2026-10-27, 아래 "먼저"). 새 곡은 판정표 10**
- [x] **v0.1.12 공격 동작**(0082): 1·2타가 발을 번갈아 치는 것으로 보이는지, 상체 들썩임이 너무 잔잔한지·과한지, 마지막 타(두 발 + 내리찍기 + X 자국)가 확실히 큰지 — **정리(2026-10-29): → 판정표 1**
- [x] **v0.1.14 프랑스어·스페인어**(0084): 원어민이 있으면 말투·용어 검수(특히 쥐 이름, 프랑스어 ! ? 앞 띄어쓰기, 스페인어 Mando/Control) — **정리(2026-10-29): → 판정표 14**
- [x] **v0.1.15**(0085): 앨범 스크롤, 독일어·러시아어(원어민 검수가 있으면 좋다) — **정리(2026-10-29): → 판정표 11(앨범)·14(독일어·러시아어)**
- [x] **번체·폴란드어**(0144): 원어민이 있으면 말투·용어 검수(번체: 보리에게 您, 輕觸·點擊 / 폴란드어: kopać 가려 쓰기, Bori 격변화, PŻ·HP). 대만·홍콩·폴란드어 기기로 켜면 그 언어로 시작하는지 — **정리(2026-10-29): → 판정표 14**
- [x] **iOS**(0086): 원하면 `nail-ios-unsigned` 아티팩트를 Sideloadly(Windows·Mac)로 내 Apple ID 서명해 설치 → 화면·터치·소리·저장(0086) — **정리(2026-10-29): 닫음: TestFlight로 가기로(0232)**
- [x] Android에서 장소를 바꿀 때 곡을 읽느라 멈칫하는지(검은 화면이 길어지는지), 보스가 깨어날 때 멈칫하지 않는지 — **정리(2026-10-29): → 판정표 12**
- [x] 새 특수효과(먼지·흙덩이·보라 실·꽃가루·모닥불 빛)가 지나치거나 모자라지 않은지 — **정리(2026-10-29): → 판정표 8**
- [x] **모모 재주·장신구**(0142): 큰 나무에서 모모 탭으로 바꿔 깨우기·끼기, 모모로 내려가 멀리뛰기(멀리 가는지)·발 구르기(착지에 꽃잎 고리·적이 맞는지)·네 번 뛰기·높이 뛰기(넓은 고리) 손맛, 모모 런 상자·가게에서 토끼풀 잎·솜꼬리가 나오는지(Naïl 런에서는 안 나오는지) — **정리(2026-10-29): → 판정표 6**
- [x] **Android 아날로그 스틱**(0143): 왼쪽 스틱을 살짝 기울이면 천천히 걷는지, 오른쪽 스틱으로 조준(필살기 포함)되는지, D패드가 그대로 듣는지, 메뉴에서 스틱 한 번 = 한 칸인지(두 칸 가지 않는지) — **정리(2026-10-29): → 판정표 3**

### v0.6.0 — 사장이 정할 오래 걸리는 큰 것(0227, 정해지기 전)
- [x] 무엇을 담을지(사장) — **정리(2026-10-29): 닫음: 팔 수 있는 판(0232)**
- [x] 그 앞의 리팩터링(0230): R15 앱 흐름(`app/after.rs`·`app/flow.rs`)·R13 위험물 수명·R16 칸 표(`art::frame`·`CHIEF`)·R17 그림 도구(`sheet4`·`check_rebuild.py`)·R18 긴 함수 둘 — 다 들어감(2026-10-29, 개발판)

### v0.5.1·v0.5.2 — 개발판 고침을 운영으로(0227, 2026-10-28·29)
- [x] 마을·쉼터 새 그림(0224, 사장 지시 2026-10-28): **A 다섯 들어감**(좌판·화덕·그루터기·바위·통나무 자리, `hub.png`). **뿌리 길 입구도 들어감**(A3, `HUB_GATE`). 남은 것: B 큰 나무 자람 다섯 → C 꾸미기 열하나 → D 아기 굴토끼. 그림체 판 `ref_hub_style.png`만 붙인다(`docs/art/hub-sprite-prompts.md`) — **정리(2026-10-29): 맨 위 "결정 대기"의 그림 받기로 옮김**
- [x] 0226(개발판): 마을 → 내려가며 길 지도 바로(첫 쉼터는 깨어나는 자리로만), 가게 장신구는 이 캐릭터 풀에서, 보리 할매 아래·의자 왼쪽(16,6). **도토 달리기 네 칸 들어감**(`doto.png` 칸 8~11)
- [x] 사장 피드백 한 묶음(0225, 개발판): 재주·독 발톱 값 ×3, 마을 뿌리 길 → 첫 쉼터 곧장(필드 안 거침, 도착 한마디는 단면 뒤), 실뿌리 셋째 쓰러짐에 런 끝 + 실뿌리가 감는 끝 연출(개발 "쉼터: 실뿌리 둘"), 맞음 피드백(카메라 킥·붉은 테두리·'아야'). 사장 확인: 값이 알맞은지, 전환·연출 길이, 맞음이 잘 느껴지는지
- [x] 도감 그림 크기·자리(큰 시트는 몸 자리에 맞춰 키운다), 메뉴 고른 칸의 글자 세로 가운데(`ui::mid_y`), 마을에 토끼풀 가족이 안 나오던 것(마커 `j`) — 개발판. 사장 확인: 도감 열다섯 종·메뉴 고른 칸·마을의 토끼풀 가족
- [x] 걷기 네 칸(0229, 개발판): 포포·보리·수수·콩이·토끼풀·모모(깡충) 들어감(게임·페이지). 사장 확인: 마을·쉼터·보스 걸음, 모모로 걷기·달리기 — **정리(2026-10-29): → 판정표 8**
- [x] 공식 페이지: 게임 클립(가짜 시계로 한 칸씩, 0228)·새 쉼터·마을 스크린숏·도토 달리기 띠·글(흐름·실뿌리·다음)

### 보류: "공개 데모"(0223 → 0227). A2~A4는 v0.5.x 패치 후보, A5·A6은 따로
- [x] A0. v0.5에서 넘긴 판정 칸(아래 v0.5 절의 빈 칸): 새 그림·쓰러짐의 대가·달리기·도토 런 한 바퀴 — **정리(2026-10-29): → 판정표 5~8**
- [x] A1. 데모 범위: ① 운영판 전체를 그대로(추천) / ② 첫 길만 따로 빌드 — **정리(2026-10-29): 닫음: 데모로 줄인다(튜토리얼 + 첫 길 + 마을, 0232)**
- [x] A2. 처음 15분 다듬기(새 슬롯 → 첫 쉼터까지 재고 막히는 곳 고치기) — Claude — **정리(2026-10-29): v0.6 데모 빌드 작업으로(Claude, 0232)**
- [x] A3. 처음 여는 사람의 기술 점검(첫 로딩·브라우저 넷·저사양·소리 시작·오류 알림) — Claude, 사장은 실기 한두 대 — **정리(2026-10-29): v0.6 데모 빌드 작업으로(Claude, 0232)**
- [x] A4. "의견 보내기"(버전·언어·장치·런 요약을 붙인 설문, 판정 넷) — 설문 서비스는 사장 확인 — **정리(2026-10-29): v0.6 데모 빌드 작업으로(0232)**
- [x] A5. itch.io 페이지 + CI zip 올리기(`butler`) — 사장: 계정·API 키 시크릿·공개 버튼 — **정리(2026-10-29): 데모를 공개할 때(Steam 페이지 시기와 함께, v0.6 뒤 결정, 0232)**
- [x] A6. 모르는 사람 다섯 이상이 해 보고 설문 — 사장이 알리기 — **정리(2026-10-29): 데모를 공개할 때(0232)**
- [x] A7. A6에서 나온 것 고치기 — **정리(2026-10-29): 데모를 공개할 때(0232)**

### v0.5.0 마일스톤(0191) — "도토와 새 옷" — 닫음(2026-10-28, v0.5.0). 아래 남은 칸은 보류한 0223의 A0(사장 판정)으로 넘김
- [x] B1. 도토 킷 설계(0195, 초안대로 진행): 던지기 3타·큰 도토리 강타·뒤로 뛰기·도토리 묻기·던지는 순간 받아치기
- [x] C1. 그림 규칙(0203) 채택(사장 "응 다 진행해도 되겠음"): 이름 있는 보스 24×24, 어두운 외곽선 1px, 2등신, Naïl 서기·걷기 시험 한 장 먼저
- [x] C2. 도구 `pxl.py snap`(0203 "C2 구현"): 나노바나나 시트 → 그림 찾기·블록 크기·어긋남 → 16칸 `.pxl` + 한 줄 검사, 왕복 시험 `tools/pixel/test_snap.py`
- [x] B4. 도토 이야기(0201): 씨앗을 찾지만 묻지는 않는 엔딩, 포포 배웅·빈 자리·마을 한마디 — 사장 "너의 각색으로". B5 안내 "도토로 싸우기"(그림)
- [x] B2·B3. 도토 손맛·성장(0195, 개발판): 개발 → "도토: 아레나"로 던지기·큰 도토리·뒤로 뛰기·묻기, 마을 뿌리 길 고르기 창 셋 칸, 큰 나무 도토 탭, 도토 장신구 셋(솔방울·깍정이 모자·단풍잎 목도리)의 맞바꿈. 도토 엔딩은 0201(제안) — **정리(2026-10-29): → 판정표 6**
- [x] 모모·도토 달리기·멀리뛰기(0209, 개발판, 사장 "손맛은 좋아졌어" — 거리만 72px로 줄임): 회피를 꾹 쥐면 내려앉은 뒤 달리고 떼면 멀리 뛴다(모모 착지 밟기, 도토 공중 던지기 한 번). 거리는 늘 72px(84에서 줄임, 사장 "지나치게 멀리"), 달리는 동안 앞 바닥에 내려앉을 자리(0211), 모모는 앞쪽 적에게 붙는다(0213). 첫 점프(깡충 착지·뒤로 뛰기 도약)는 피해 없이 밀치기만(0214). 달리며 모으기에도 땅울림(소리·흔들림·흙알·몸 떨림, 0215). 도토는 꼭대기에서 가장 가까운 적에게 저절로 부채꼴 셋(0212). 아레나 둘에서 빠르기·거리·표시가 잘 보이는지
- [x] 쓰러짐의 대가(0210, 개발판, 사장 "0.4.x에서 죽었을 때의 디메리트가 별로 없다는 의견"): 쉼터 뒤 실뿌리 셋(HUD 체력 아래) — 넷째로 쓰러지면 마을로, 길의 주인은 쓰러져도 남는다, 맡기지 않은 반딧불은 다 잃는다. 한 런을 돌아 보며 너무 가혹한지 — **정리(2026-10-29): → 판정표 7**
- [x] C3. **Naïl 새 옷**(0206, 사장 "좋아 이걸로 하자"): 실루엣 시안 1 몸 + 둥근 시트의 눈·볼(`nail_s1_v2`) + 동작 시트 B(`nail_s1_v2_b`), 몸 16px·칸 24, 분홍 손발, 걷기 앞발 흔들기(위아래 + 얼굴과 선), 숨쉬기 두 칸은 한 그림에서. 걷기 넷의 눈을 서기 눈 하나로(0212). 파 내려가기 그림(`dig.py`)도 목도리를 둘렀다
- [x] C3. 모모·도토 새 그림(개발판, 0206): 보스(물든 모모 보라·초록 눈·시든 꽃 / 물든 도토 회보라·호박색 눈)·플레이어·마을·컷씬에서 크기·꽃·모자·깡충·차기·던지기가 읽히는지. 개발 "모모: 아레나"로 모모를 바로 움직여 본다. 몸의 해 그림자는 위아래 1px 메움(0208, 도토 꼬리·머리 그림자가 갈라졌다) — **정리(2026-10-29): → 판정표 8**
- [x] C4. 잡몹 열·보스 다섯·이웃 — **보리(들쥐 대장) 새 그림 들어감**(0216, 개발판: 40칸, 물든 대장은 색 바꾸기 + 목의 보라 실), **콩이 새 그림 들어감**(0217, 개발판: 32칸, 공 두 칸은 따로 배율 + 공의 띠는 원본에서 옮기고 서기 칸의 띠는 "(" 곡선으로 새로 긋기, 물든 콩이는 색 바꾸기 + 빛나는 파란 눈 + 보라 실), **토끼풀 새 그림 들어감**(0218, 개발판: 40칸, 꽃을 살리려 변환기에 `pink` 바탕 판정, 물든 토끼풀은 색 바꾸기 + 초록 눈 + 보라 실). 보스 다섯 끝, **할매 새 그림 들어감**(0219, 개발판: 0191 목록에 빠져 있던 가장 깊은 곳의 주인 — 사장 "할매부터". 원본 한 장 → 가장 깊은 곳 96칸 + 마을 40칸, 마을 크기는 얼굴을 다시 찍는다, 마을의 보리는 모닥불 왼쪽으로(할매·큰 나무와 안 겹치게)), **이웃 포포·수수 새 그림 들어감**(0220, 32칸, 포포 연필은 원본에서 옮겨 칠하고 가면 속 눈에 빛점, 수수는 네 칸만 받아 날숨·걷기를 만들고 더듬이·눈을 원본에서 다시 찍는다). 다음은 잡몹 열 — **24칸으로 올린다**(0221, 사장 확인; 참고 그림·프롬프트 `mobs_ref.py` → `docs/art/mob-sprite-prompts.md`), **잡몹 열 종 모두 들어감**(`mobs_ai.py`, 박쥐만 32칸 — 날개, 연습 허수아비는 critters 16칸 그대로). C4의 그림(보스 여섯·이웃·잡몹)은 끝 — 사장 확인 남음 — **정리(2026-10-29): → 판정표 8**
- [x] C5. 소품 — 부술 수 있는 것 넷 + 보물 상자만(0222, 사장 "1, 2 하자"; 땅·벽·나무 등은 이미 Sunnyside 팩, 효과·장애물·줍는 것은 코드 그대로). **들어감**(`props_ai.py` → `props.png`, 금은 원본에서 옮겨 칠한다, 반짝은 코드). 다음은 D(사장 판정)
- [x] **박자 2타·부술 것 표시**(0205): 보통으로 톡 눌러 박자에 2타·3타가 이어지는지(무기마다), 굴에서 부술 것(흙더미·뿌리혹·돌무더기·실뭉치)이 외곽선·금·반짝으로 바로 보이는지 — **정리(2026-10-29): → 판정표 1·4**
- [x] **엔딩 뒤 마을로 올라오기**(0204, 사장 "할매 이후 지상 마을로 가는 게 안 보인다"): 개발 → "마을로 올라오기"로 130m → 0m 단면·뿌리 길 곁 튀어나오기·1초 뒤 포포 한마디의 길이와 손맛 — **정리(2026-10-29): → 판정표 9**
- [x] D. 사장 "0.5.0 배포해줘"(2026-10-28) — 판정 칸(위의 빈 칸들)은 보류한 0223의 A0에서 본다

### v0.4.0 마일스톤(0145) — "깊은 곳" — 닫음(2026-10-28, v0.4.0). 아래 남은 칸은 v0.4.x에서 계속 볼 것
- [x] A. 굴토끼 가족의 길(0165): 셋째 길 갈래 '굴토끼 옛 굴'(모모 귀환 뒤), 흰 실 지대 + 실 그물, 어미 토끼풀(발 구르기·두 번 차기·내려앉기·품기, 고치는 못 때림), 가족 귀환(마을·저널 "가족"·흰 실 매듭·에필로그). 초상 3칸(칸 38~40)·일러스트 `clover_cocoon`(귀환 장면, 앨범)까지 들어갔다
- [x] B. 가장 깊은 곳의 마지막 싸움(0162): 할매는 싸우지 않고 실을 당긴다, 매듭 셋을 끊으면 놓아준다. 세 번 감기면 쉼터 모닥불에서 깸. 손맛·길이·할매 그림은 사장(개발 → "가장 깊은 곳")
- [x] C. 새 잡몹 다섯(0146): 땃쥐(두 번 물기)·고슴도치(가시 공, 발톱 튕김)·두꺼비(혀로 끌기)·박쥐(공중, 솟구치기 안 닿음)·포자버섯(구름). 지대마다 다른 뽑기 표. 손맛·난이도·심층에서 잘 보이는지는 사장(개발 → "새 졸개 다섯")
- [x] D. 스테이지 다양화(0147): 뿌리 지대 가시뿌리 덤불(걸으면 찔림, 돌진하던 적은 박혀 멈칫), 심층 무너지는 천장(예고 뒤 돌), 길 지도 "실타래" 칸(엘리트 + 졸개 둘, 이기면 장신구). 세기·빈도는 사장(개발 → "실타래 굴: 뿌리/심층")
- [x] E. 열한 언어·도감·저널(0146·0147·0162·0165의 글은 모두 열한 언어. 앨범은 토끼풀 일러스트 원본이 오면)
- [x] F. 사장 판정(2026-10-28 "0.4.0으로 배포하자")
- [x] **안내 탭·그림·'똑'(0180)**: '똑'이 박자를 잡게 해 주는지·거슬리지 않는지, 그림 칸 크기, 그림이 바로 읽히는지 — **정리(2026-10-29): → 판정표 10·11**
- [x] **안내 화면(0178)**: 빠진 주제가 있는지, 설명이 너무 긴 언어(쪽 셋 넘음), 산딸기가 산딸기로 보이는지 — **정리(2026-10-29): → 판정표 11**
- [x] **설정 파일(0175)**: Android에서 언어·음량·버튼을 바꾸고 껐다 켜면 남는지, APK를 덮어 설치해도 남는지 — **정리(2026-10-29): → 판정표 12**
- [x] **적 예고 면(0179 → … → 0187 → 0188)**: 다 찬 뒤 0.1초가 "이제 온다"로 읽히는지, 투명도 0.6, 붉은 한 색에서 띠(받아침)와 X 원(피함)이 모양만으로 갈리는지, 붉은 지대에서 묻히지 않는지, 다 찬 순간 외곽선과 채움이 한 색으로 보이는지, 밑띠 없이 풀밭·어두운 지대에서 외곽선이 보이는지, 채움이 테두리에 닿아 "한 덩어리가 되는 순간"이 피할 만큼 읽히는지, 좁은 띠(던지기·혀)의 폭 변화, 풀밭 위 황금 띠의 대비, 여러 적이 겹칠 때 면이 쌓여 진해지지 않는지, 짧은 예고(땃쥐), 어두운 지대 — **정리(2026-10-29): 닫음: 사장 피드백으로 0192·0196·0197을 거쳐 진하기를 설정으로(0198). 남은 확인은 판정표 4**
- [x] **부술 수 있는 것(0177)**: 방마다 1~3개가 알맞은지, 빈손 44%가 "운"으로 느껴지는지, 들판·쉼터·마을에도 둘지 — **정리(2026-10-29): → 판정표 7**
- [x] **길의 꽃잎·빛 건너가기·옅은 예고(0190)**: 보리 공터 앞 꽃잎이 눈에 띄는지, 빛 건너가기 0.9초, 예고 0.45가 너무 옅지 않은지 — **정리(2026-10-29): 닫음: 꽃잎은 사장 피드백으로 0194, 예고 진하기는 설정(0198)**
- [x] **강타·받아치기 반응(0189)**: 강타가 떼는 순간 나가는지, 받아치기가 누른 때 바로인지, 위협 90px, 다 맞춘 마무리 +1이 센지 — **정리(2026-10-29): → 판정표 1**
- [x] **박자 콤보(0174 → 0183 → 0184)**: 0.42초 템포가 "타·타·탕"으로 읽히는지, 창 ±0.09초, 연타가 박자에 스냅되는 게 둔하지 않은지, 닫히는 고리·종소리·음표가 싸움을 가리거나 유치하지 않은지, 모모 발차기의 쉼 — **정리(2026-10-29): → 판정표 1**

### v0.3.0 마일스톤(0081·0140) — 닫음(2026-10-27, 사장 결정으로 v0.3.0 배포, 0148). 7·10~12는 아래 "먼저"로 넘김
- [x] 1~6. 특수효과·소리 바탕·메뉴 소리·효과음·음악(0081)
- [x] 7. 귀로 판정(사장) — **정리(2026-10-29): 닫음: 사장 "좋음"(2026-10-27)**
- [x] 8. 입자 효과 설정(0141): 설정 0~100%, 장식 효과·환경·일러스트·타이틀 입자를 줄이고 공격 범위 고리는 남김
- [x] 9. Android 아날로그 스틱·오른쪽 스틱 조준(0143): 왼쪽 스틱 세기, 오른쪽 스틱(Z·RZ) 조준, HAT D패드. 실기 확인은 사장
- [x] 10~12. 실기 스틱·드릴 겨누기·판정 2·3 다시(사장) — **정리(2026-10-29): → 판정표 1·3(아래 "먼저"와 같은 칸)**
- [x] 13. 모모 전용 재주·장신구(0142): 큰 나무 Naïl/모모 탭, 멀리뛰기·발 구르기·네 번 뛰기·높이 뛰기, 토끼풀 잎·솜꼬리(모모 런 상자·가게)·들꽃 화관(저널 "모모와 가장 깊은 곳")

### v0.2.0 마일스톤(0079) — 닫음(2026-10-26, 사장 결정으로 v0.2.0 배포)
- [x] 1. 본인 외 한 사람의 판정(2026-10-26 사장의 아들: 1·4 동의, 2·3 약함 → 0090)
- [x] 2. Android 실기 확인: 설치·패드 버튼·덮어 설치 뒤 저장 이어짐(2026-10-26 사장 확인)
- [x] 3. 1·2에서 나온 것 고치기(Claude): 솟구치기 밀쳐내기·마무리 타(0090) 들어감(v0.1.16~) → **같은 사람에게 판정 2·3을 다시** 물으면 끝. 이것만 남은 채 사장이 닫았다(아래 "먼저"의 판정 2·3 다시는 그대로 남긴다) — **정리(2026-10-29): → 판정표 1·2(판정 2·3 다시)**
- [x] 4. 쉼터부터 이어하기(Claude, 0080, v0.1.9)
- [x] 5. 문서를 실제와 맞추기(0079, concept.md, CLAUDE.md)

### 먼저
- [x] **v0.3.0에서 넘어온 판정**(0148·0149): ~~7 귀~~ 좋음, ~~10 오른쪽 스틱 조준~~ 됨(2026-10-27). 남은 것: 10의 이동 세기·메뉴 한 칸씩, 11 드릴 겨누기가 엉뚱한 적에 끌리지 않는지 — **정리(2026-10-29): → 판정표 3**
- [x] **0152·0153 보기**: 타이틀 메뉴 뒤 둥근 그늘·고른 줄(▶ + 밝은 글), 오프닝 내레이션(아래에서 옅어지는 어둠), 안내 글의 키캡(Enter·Esc·패드 버튼이 테두리 칸으로), 타이틀 안내 두 항목 — **정리(2026-10-29): → 판정표 11**
- [x] **0157 단추감**: 시작·이어하기 창의 단추가 눌리는 느낌이 나는지, 0.12초가 늦게 느껴지지 않는지(키·패드·터치) — **정리(2026-10-29): → 판정표 11**
- [x] **0165 굴토끼 가족의 길**: 흰 실 지대가 심층과 다르게 읽히는지, 실 그물이 귀찮기만 하지 않은지, 토끼풀 수 넷의 손맛(충격파·두 번 차기·품기), 고치를 때리면 안 되는 게 읽히는지, 귀환 장면·마을의 가족, 모모 런의 "엄마, 나야!", 흰 실 매듭, 콩이의 문/옛 굴을 매 런 고르는 맛. 초상 3칸(칸 38~40)이 대화창에 맞는지, 귀환 장면의 일러스트 `clover_cocoon`과 내레이션이 흐름을 끊지 않는지 — **정리(2026-10-29): → 판정표 5·9**
- [x] **0164 오프닝의 모모**: 덮침이 "공격당했다"로 읽히는지, 착각(엄마)·이상한 한마디가 궁금증을 만드는지, 포포의 한마디, 꽃잎을 밟게 되는지, 도감 '???' 칸 — **정리(2026-10-29): → 판정표 9**
- [x] **0163 얇은 장면**: 대장(대낮 공터)·도토·콩이 등장 연출이 어울리는지, 놓아주기 3.4초와 눈 뜬 할매, 잠드는 장면 뒤 되살아나기 토스트가 거슬리지 않는지. 남은 얇은 곳(쉼터 모닥불·발톱 받기·저장 고리·큰 나무·저널·꿈·마을 나무 성장)은 다음에 — **정리(2026-10-29): → 판정표 9**
- [x] **0162 가장 깊은 곳**: 당기기 3.2초·예고 0.9초가 읽히는지, 감기면 매듭이 다시 조여지는 벌이 무겁거나 가벼운지, 세 번에 잠드는 게 알맞은지, 할매 코드 그림(나노바나나 원본은 사장이 그린 뒤 바꿔 끼움), 끝났을 때 "놓아준다"로 읽히는지, 모모 런에서도 같은 싸움이 맞는지 — **정리(2026-10-29): → 판정표 5(할매 그림은 0219로 들어감)**
- [x] **0168 타격 피드백·강타 보상**: 카메라 킥 2/3/4px가 어지럽지 않은지, 피격자 진동·눌림이 읽히는지, 벽 쿵이 노림수가 되는지(넉백 ×1.7), 딱 맞춘 강타 창 0.15초, '틱'·올라가는 소리 크기, 강타 3·딱 맞춤 4가 잡몹에 너무 센지 — **정리(2026-10-29): → 판정표 1**
- [x] **0161·0167 강공격·박자 콤보**: 모으기 0.45초·이동 0.5배가 알맞은지, 강공격 뒤 후딜레이 0.22초가 알맞은지, 박자 창(활성 끝 0.03초 앞부터)이 빡빡하지 않은지·연타 벌칙 0.08초가 "조금 느리게"인지·'틱' 소리가 박자를 가르치는지, 평타 연타 중 실수로 모으기에 들어가지 않는지, 강공격으로 대장·모모 내려앉기를 받아치는 손맛, 터치에서 누르고 있기, 포포의 새 단계([4/10]) — **정리(2026-10-29): → 판정표 1**
- [x] **0160 톡 굴파기·받아치기 구분**: 톡이 "동작"으로 느껴지는지(파고들기 → 2.1배 대시 약 66px → 폴짝), 모으는 동안 0.55배가 답답하지 않은지, 황금 예고 = 받아칠 수 있음·진홍 고리 ✕ = 피하기가 읽히는지, 대장·모모 내려앉기를 못 받아치게 된 것이 싸움을 어렵게 하지 않는지 — **정리(2026-10-29): → 판정표 2**
- [x] **0159 모모 등장·돌아오기·샘·출구**: 오프닝의 모모(음악 끊김 → 꽃잎 → 대낮의 어둠 → 어둠에서 걸어 나옴 → 카드)가 보리의 "그 아이도…"까지 기억에 남는지. 굴 끝방 첫 만남(어둠 → 빛·카메라가 모모에게 → 꽃잎·땅 울림 → '???' 카드)이 궁금증을 만드는지, 약 5초가 길지 않은지, 굴의 시든 꽃잎이 눈에 띄는지. 보리·모모·도토·콩이가 돌아올 때 실이 풀리는 이펙트·소리. 샘에 발을 담그는 게 읽히는지(글 없이 빛·물결만). 끝방 출구가 "내려간다"로 읽히고, 화면 밖 표시가 거슬리지 않는지 — **정리(2026-10-29): → 판정표 9**
- [x] **0158 굴파기 힘 모으기**: 톡과 길게 누르기가 손에 구분되는지, 0.45초 모으기가 길지 않은지, 다 모은 솟구침(반지름 32·피해 2)이 세서 굴파기만 반복하게 되지 않는지, 우는 소리·큰 솟구침 소리, 터치에서도 모이는지 — **정리(2026-10-29): → 판정표 2**
- [x] **0156 보기**: 굴로·다음 쉼터로 갈 때 땅을 파 내려가는 단면(1.2초)이 길지 않은지, 깊이 숫자·지대 이름 카드("뿌리 지대 · 지하 12m")가 읽히는지, 비스듬한 길 지도(왼쪽 위 쉼터 → 오른쪽 아래 보스), 뿌리 지대의 빛줄기·떨어지는 흙, 심층의 물방울이 보이는지·거슬리지 않는지 — **정리(2026-10-29): → 판정표 9**
- [x] **0154 듣기**: 타이틀(바이올린·하프)·마을(피아노·바이올린)·쉼터(피아노 왈츠)가 장소에 맞는지, 칩튠 곡과 크기가 맞는지, 마을 곡 리미터가 들리는지, 타이틀 고리가 돌 때 첫 큰 화음이 거슬리는지(거슬리면 첫 3초를 고리에서 뺀다), 마을에 들어갈 때 멈칫하는지(네이티브 157초 곡). 아들에게 피아노·바이올린이 잘 들리는지도 물어보면 좋다 — **정리(2026-10-29): → 판정표 10**
- [x] **0151 걸어 보기**: 둘째(뿌리 동굴)·셋째(버섯 숲) 쉼터에서 길·이웃을 찾기 쉬운지, 숨은 자리(반딧불·씨앗)가 알맞게 숨었는지, 두 번째 런부터 굴 입구 곁에서 시작하는 게 어색하지 않은지 — **정리(2026-10-29): → 판정표 13**
- [x] **0149·0150 다시 보기**: 느려진 드릴(270px/s), 적 그림자(짧고 둥글게, 남북도 두껍게), 못 할 때 "안 됨" 소리·HUD 떨림·✕, 덤불(어두운 몸 + 흰 가시)·어둠 위 돌 예고, 밝아진 공격 예고 테두리, 첫 찔림 안내. 가능하면 색약인 사람 한 명에게도 — **정리(2026-10-29): → 판정표 3·4**
- [x] **본인 외 한 사람에게 건네기**(2026-10-26, 결과는 0090 배경)
- [x] **판정 2·3 다시**(0090, 2026-10-27 사장 "판정은 좋아", 0149): 솟구치기로 일부러 튀어나와 때리게 되는지(밀쳐내기·충격 고리·쿵), 3타가 묵직한지. 너무 세면(적이 흩어져 싸움이 늘어짐, 굴파기만 반복) 0090의 "다시 볼 조건"대로
- [x] **HUD·퀵 슬롯**(0091): 체력·게이지·퀵 슬롯이 폰 화면에서 바로 보이는지(너무 어둡거나 크지 않은지), 산딸기를 다 먹으면 지렁이로 바뀌고 깜빡이는 것이 알아보이는지, 그래서 지렁이를 실수로 먹게 되지 않는지 — **정리(2026-10-29): → 판정표 11**
- [x] **v0.1.18 확인**: — **정리(2026-10-29): → 판정표 2(누르고 있기)·8(울타리·버섯 빛·그림자)**
  - 굴파기 누르고 있기(0101): 0.45초 더 머무는 게 알맞은지, 더 머문 만큼 늘어나는 대기가 무거운지, 터치에서도 누르고 있기가 되는지
  - 잠긴 문 울타리(0094·0096~0099): 솟고 가라앉는 움직임, 보라 실 굵기
  - 어두운 굴의 버섯 빛(0095·0100): 충분히 보이는지, 심층이 너무 밝아지지 않았는지
  - 해·모닥불 그림자(0102~0106, 시험): 방향·길이·진하기가 맞는지, 팩에 구운 아래 그림자와 둘이 어색하지 않은지, 모닥불 곁 그림자가 퍼지는 모양, 빼자고 하면 끈다
- [x] **드릴 겨누기**(0139): 시동 동안 누른 쪽 둘레(±34°)의 적에게 모서리 표시가 뜨고 그쪽으로 나가는지, 원하지 않는 적에게 끌리지 않는지, 점선이 거슬리지 않는지 — **정리(2026-10-29): → 판정표 3**
- [x] **v0.2.0 확인**(나노바나나 그림, 0107~0138): — **정리(2026-10-29): → 판정표 8**
  - 대화 초상화 아홉 캐릭터(열세 묶음): 배율 3·5(홀수)에서도 칸을 꽉 채우는지(0111), 표정이 갈리는지, 물든 눈(빨강·초록·호박·하늘빛)이 보이는지
  - 컷씬 일러스트 열하나: 대화창·내레이션에 얼굴이 가리지 않는지(특히 패닝 끝: `last_bloom`·`momo_deep`), 작은 캐릭터(모모·콩이·잠든 Naïl)가 알아보이는지
  - 타이틀 바탕(0137): 해 질 녘 그림 위 메뉴 글이 읽히는지, 안개·반딧불이 어색하지 않은지, 저작권 줄이 굴 방에 겹치는 것이 거슬리는지
  - 앱 아이콘(0123): 폰 런처(원·둥근 네모)에서 뿌리 끝이 잘리는 게 거슬리는지
- [x] **v0.1.10 확인**(0058~0081을 한 번에): 웹·APK에서 — **정리(2026-10-29): → 판정표 12(저장·이어하기·슬롯·저장 지점·Chrome)·13(모모 런 배웅). 팩 소품(0077)은 나노바나나 마을 물건(0224)으로 바뀌어 닫음**
  - [x] **APK가 설치되는지**(사장 확인. 0074: v0.1.4까지는 Android 12+에서 "package appears to be invalid"로 거부됐다). 기기 모델·Android 버전을 함께 적는다
  - [x] **Android 패드 버튼**(0075, 2026-10-26 사장 "다 됨"): A·B·X·Y·LB·RB·LT·RT·Start·Select가 듣는지, 메뉴에서 A 확인·B 뒤로·기기 뒤로 키, 설정에서 버튼 다시 붙이기, 안내 글이 패드 이름으로 나오는지. 스틱이 디지털(8방향)인 게 답답한지(답답하면 아날로그·오른쪽 스틱을 넣는다)
  - **저장 지점**(0076): 필드·쉼터에서 멀리서도 보이는지, 너무 번쩍이지 않는지
  - **팩으로 바꾼 소품**(0077): 가게 가판·모모 빈 자리·마을 의자·화분, HUD 하트·새싹이 어색하지 않은지
  - **쉼터부터 이어하기**(0080): 쉼터에서 앱을 껐다 켜고 이어하기 → 같은 쉼터·같은 씨앗·가방인지, 굴 안에서 끄면 마지막 쉼터부터인지
  - **저장 슬롯**(0078): 덮어 설치한 뒤 예전 기록이 슬롯 1로 이어지는지, 타이틀 시작하기·이어하기 → 슬롯 고르기, 덮어쓰기·지우기(두 번)
  - Sunnyside 그림(큰 나무·들판 나무·꽃·모닥불·버섯)이 나오는지, 장소를 바꿀 때 멈칫 없이 밝아지는지(검정이 0.2초쯤 길어진 게 거슬리지 않는지)
  - Android Chrome 글자 색, 타이틀 "v<버전> · <커밋>"
  - 모모 런: 포포 배웅과 쉼터 빈 자리 말(나일은 마을에서 기다린다)
- [x] GitHub에서 Secret `SECRETS_TOKEN`과 그 fine-grained 토큰을 지운다(0067, 2026-10-27 사장 "Secret은 지웠어")
- [x] 공식 페이지 켜기(0069, 2026-10-25 사장이 Source = GitHub Actions, `pages` 4번째 실행 성공)
- [x] **공식 페이지 보기**(0155로 다시 짬): https://aimenjoe.github.io/nail-under-the-roots/ 를 폰·PC로 보고 글·그림이 어색한 곳을 알려 준다. 실제 기기로 찍은 짧은 플레이 영상이 있으면 첫 화면·싸움 칸에 넣는다 — **정리(2026-10-29): 닫음: 사장이 2026-10-27~29 여러 번 보고 고침을 요청해 반영(0228·0229)**

### 판정 (한 사람의 의견을 들은 뒤, 몰아서)
- [x] **싸움 손맛**: 패링 창 0.12초·분노 +4, 행동 대기(먹기 0.35초·대기 1.5초, 0038), 공격 예고가 읽히는지(0025), 드릴 시동·거리·벽 쿵(0024), 장신구·퀵 아이템 — **정리(2026-10-29): → 판정표 1**
- [x] **보스**: 튜토리얼 보스 체력 36·슈퍼아머, 모모(0046) 셋째 이어 뛰기 세기, 도토(0052) 부채꼴·묻은 도토리, 콩이(0054) 튕김·버티기, 실의 꿈(0055) 보랏빛·체력 2 회복 — **정리(2026-10-29): → 판정표 5**
- [x] **모모 플레이어블**(0056): 깡충 56px·대기, 이어 뛰기 세 번 × 2, 뒷발 사거리, 작아 보이는지, 고르기 창, 모모 장면·`momo_deep` 그림 — **정리(2026-10-29): → 판정표 6(깡충은 0209·0229로 바뀜)**
- [x] **런 구조**: 필드 + 길 3개가 10~15분인지(0042), 필드 크기·적 밀도·던전 방 수·카메라 속도·부활 규칙, 반딧불·씨앗·산딸기 개수와 자석 반경, 포포의 응원 비율, 길 지도 갈래(다음 층만 보임), 쉼터 가게 값, 마을(0045) 꾸미기 값·잎 자람, 큰 나무 재주 값(0047)·빛남(0061), 저널 목표 숫자(0048), 실뜨기 1~10 어려움(0043), 오늘의 굴(0044) — **정리(2026-10-29): → 판정표 7**
- [x] **그림**: 지도(0062·0063: 둥근 벽·바닥 띠가 어지럽지 않은지, 던전 바위가 싸움을 막는지), 적 다섯 종 구별, Naïl 스프라이트(털빛), 모모 물든 모습(스프라이트), 컷씬 일러스트 길이·패닝(0039). 초상화·일러스트 그림은 위 "v0.2.0 확인" — **정리(2026-10-29): → 판정표 8(적·모모 그림은 새 그림 C3·C4로 바뀜)**
- [x] **이야기·글**: 0037 뼈대에서 어색한 곳, 튜토리얼 9단계 길이·포포 말투(0023), 수수·이웃 말투 — **정리(2026-10-29): → 판정표 9**
- [x] **기기**: iPhone Safari(안티에일리어싱 끔, 0026·대화창 버벅임, 저전력 모드 끄고), 폰 터치 공격 버튼 크기·가게/길 지도 닫기(✕·바깥 탭), 패드 실기(다시 붙이기, 장치별 안내 0036, 닌텐도 배치), 화면 떨림(0034), 타이틀 글자 테두리(0035), 가게 사기 전 확인 창, 세로↔가로 초상화 크기, 껐다 켜도 기록이 남는지(0041) — **정리(2026-10-29): → 판정표 12**
- [x] **번역**: 브라질 포르투갈어 검수(가능하면 원어민, 0033) — **정리(2026-10-29): → 판정표 14**

### 정할 것
- [x] 필드 = 넓은 쉼터로 합칠지(0030 "제안") → 합치지 않고 쉼터 지도 셋·숨은 자리·필드 지름길(0151)
- [x] 굴토끼 가족(할매 실 속) 갈래를 이야기에 넣을지(2026-10-27 넣는다: v0.4.0 A, 셋째 길 갈래·모모의 엄마, 0145), 모모 전용 재주·장신구(0142로 넣음)
- [x] 컷씬 일러스트를 사람이 그릴지(본인/외주). 정하면 `docs/art/cutscene-illustrations.md`를 넘긴다(0039) — **정리(2026-10-29): 닫음: 그림은 바꾸지 않는다(0232)**
- [x] 다음 캐릭터(0040 순서 7) — **정리(2026-10-29): 다회차 깊이 묶음으로(v0.6 뒤, 0232)**
- [x] iOS 배포 길(0086): Apple Developer Program($99/년) + CI 서명 → TestFlight로 갈지, iOS 패드 다리(GCController)를 언제 넣을지 — **정리(2026-10-29): 닫음: TestFlight까지(0232)**
- [x] Android 개발자 인증(2026-09 일부 나라, 2027 전 세계): 사이드로드 APK도 인증된 개발자 서명이 필요해진다(ADB 설치는 예외). 무료 "제한 배포" 계정(기기 20대)으로 할지, 정식 등록할지, 릴리스 서명 키를 언제 만들지 — **정리(2026-10-29): 닫음: 정식 계정 $25(0232)**
