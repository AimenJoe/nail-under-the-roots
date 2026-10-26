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
- 입력: `dodge` 누름 → 즉시 땅속(무적, 이동 가능, 속도 1.3배, 그림자만 표시) → 0.35초 후 또는 `dodge` 다시 누름 → 솟구침(반경 ~16px 범위 공격, 적 넉백·경직).
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

## 구현 상태 (2026-10-24)
코드는 전부 들어갔다. **판정: 통과**(2026-10-25 사장 "판정 및 손맛은 좋은 편이야, 지금 이대로도"). 수치는 지금 값을 기준으로 둔다. 본인 외 한 명 더는 아직이다.

| 항목 | 위치 | 비고 |
|---|---|---|
| 고정 타임스텝 60Hz + 가변 렌더(보간) | `src/main.rs`, `world.rs` | 눌림은 `input.rs` 래치에 모아 틱이 소비. 히트스톱·일시정지 중엔 보간 prev를 현재로 맞춰 미끄러짐 방지 |
| 모듈 분리 | `input` `player` `enemy` `combat` `room` `render` `world` `tuning` `sfx` | `world`는 틱 순서를 엮는 곳, `sfx`는 임시 합성음(명세 밖 추가) |
| 발톱 3연타 | `player.rs` | 활성 구간이 끝나면 버퍼된 입력으로 다음 타가 바로 이어짐. 3타 뒤 추가 경직 0.14초. 스윙 시작 시 전방 전진(lunge) |
| 굴파기 | `player.rs` | 명세대로. 솟구침은 투사체도 지운다(추가). 쿨다운 바를 HUD 오른쪽 위에 표시 |
| 적(물든 이웃) | `enemy.rs`, `species.rs`, `tools/art/critters.py` | 그림 v2: 종마다 기본 자세를 손으로 찍고 동작은 옮기기·몇 점 바꾸기(v1 타원 쌓기는 얼룩져 안 읽혔다). 다섯 종: 들쥐·생쥐 떼·시궁쥐(돌격 셋), 다람쥐(도토리), 청설모(웅크렸다 뛰기, 착지 고리). 종류 표 = 수치·도감 글. 쥐가 절반 넘게 뽑힘. 개발 시나리오 "적 다섯 종"(docs/decisions/0021) |
| 내 캐릭터 구별 | `tools/art/critters.py`(taint), `render.rs`(draw_player_halo) | 물든 적은 털색을 보라 쪽 + 어둡게, 외곽선 차갑게. Naïl은 따뜻한 색 그대로 + 발밑 빛 고리(천천히 숨 쉼). 흑백으로 봐도 Naïl이 더 밝다 |
| 떨어진 것 | `loot.rs`(bounds), `adventure.rs`(loot_bounds, push_loot) | 던전에서는 떨어진 방 안쪽에서만 구른다(문 칸으로 굴러가 주우려다 옆 방으로 넘어가던 문제) |
| 공격 예고 | `render.rs`(draw_telegraphs), `room.rs`(ray_len) | 돌진 길 띠·조준 점선·착지 고리를 바닥에 반투명 붉게(채움 3→16%, 테두리 15→50%), 예고가 끝날수록 진하게 + 마지막 15% 깜빡임(docs/decisions/0025, 0026) |
| 웹 로딩 화면 | `web/index.html`, `src/loading.rs`, `main.rs` | 글 없는 걷는 Naïl + 막대. wasm 받은 바이트(0~60%) + 준비 5단계(60~100%), 첫 프레임 뒤 걷음(docs/decisions/0028). 느린 회선 흉내로 확인 |
| 웹 GL 컨텍스트 | `web/index.html` | 안티에일리어싱·투명 배경 끔, 고성능 GPU 요청. 아이폰급(dpr 3) 캔버스 2532×1170에서 매 프레임 4배 샘플을 없앰. 헤드리스 같은 3초 동안 그린 프레임 62 → 144(docs/decisions/0026) |
| 도감·상태 | `app.rs`(draw_bestiary, draw_status) | 도감: 만난 이웃, 능력은 말로. 상태: 플레이어 능력치 숫자(체력·공격력·공속·굴파기 대기·분노 충전·무기·장신구·반딧불·씨앗) |
| 조작 설정 | `input.rs`(Bindings, prompt, capture), `prefs.rs`, `app.rs`(draw_controls), `web/index.html`(nail_prefs) | 오른쪽 넷 = 공격·굴파기·필살기·아이템, 어깨 = 아이템 넘기기, Select = 지도. 다시 붙이기(겹치면 맞바꿈), 웹 저장(docs/decisions/0022) |
| 피격·사망·재시작 | `player.rs`, `world.rs` | 체력 3, 피격 무적 0.8s. R / Start로 즉시 재시작 |
| 전투감 | `combat.rs`(Feel), `world.rs`, `render.rs` | 히트스톱(로직 시계만), trauma² 흔들림(정수 px), 흰색 플래시, 넉백(감쇠), 합성음 |
| 게임패드 | `input.rs`, `web/macroquad-gamepads-0.1.js` | 메뉴는 D패드·왼쪽 스틱 둘 다(스틱은 큰 축 하나, 0.5에서 잡고 0.3에서 풀림, 누르고 있으면 반복). 데스크톱(gilrs)·웹(플러그인). 웹은 비표준 매핑 패드도 받도록 플러그인 패치. Android 13 폰 브라우저 + 실기 패드로 인식 확인(2026-10-25). 데스크톱 실기는 미확인 |
| 터치(보조) | `input.rs` TouchState·`layout_for`, `render.rs` 오버레이, `web/index.html` | 왼쪽 반 떠다니는 스틱 + 버튼. 공격이 가장 크고(지름 ≈ 짧은 변의 27%, 가로 폰 약 105pt), 굴파기·필살기·아이템은 엄지 호를 따라(2026-10-25, 겹침·화면 안 테스트). 모바일 브라우저용. 터치 좌표는 DPI로 나눠야 함(macroquad touches()는 물리 픽셀). Android 13 폰 브라우저 실기 확인(2026-10-25) |
| 디버그 | F1 오버레이(fps, 상태), F2 히트박스 | 기본 꺼짐. 설정에서도 켬/끔 |
| 그림 | `art.rs`, `biome.rs`, `render.rs`, `tools/art/` | Naïl 스프라이트 v2(직접 그림. 얼굴: 동그란 코·볼 홍조, 입 없음, 발톱은 발 줄에만 — v1은 이빨처럼 보였다), 대체 타일, 3/4 벽·y 정렬·그림자, 환경 3종(색조·장식·어둠), 발톱 자국 이펙트, 컷씬 레터박스. Sunnyside 팩은 레포 밖(docs/decisions/0010) |
| 장비 | `equipment.rs`, `moveset.rs` | 캐릭터별 무기 종류. Naïl은 발톱 4종(새싹·두더지·바위·독), 가방에서 장착, 독 지속 피해. 규칙은 `cargo test`(docs/decisions/0011) |
| 무브셋 | `moveset.rs` | 콤보를 데이터로(발톱 3타, 실험용 무거운 2타). 타별 회피 캔슬 창 `dodge_cancel_after`. 개발 시나리오 "무거운 무기"로 비교(docs/decisions/0009) |
| 튜토리얼 | `tutorial.rs`, `cutscene.rs`(popo_talk), `world.rs`(npc) | 라쿤 포포의 9단계: 이동·발톱·3연타·굴파기·솟구치기·패링(30초면 넘김)·필살기·아이템·메뉴 안내. 단계마다 대화 + "[n/9]" 힌트(지금 버튼 이름), 연습 중 체력 그대로(docs/decisions/0023). 헤드리스 Chromium으로 오프닝 → 포포 → 1~3단계 확인 |
| 개발 시나리오 | `scenario.rs` | 타이틀 "개발"(dev feature). 빈 방, 돌격형 1/3, 원거리형 1, 1+1, 웨이브 3부터, 오프닝 컷씬. 판정 2(적 3마리)는 "돌격형 3마리"로 바로 들어간다 |
| 컷씬 | `cutscene.rs` | JRPG식 대화창·배우 이동·페이드·건너뛰기. 오프닝: 물든 모모(이름표 "???", 초상화 차가움·괴로움, 스프라이트)가 길을 막는다(docs/decisions/0008, 0029) |
| 게임 틀 | `app.rs`, `ui.rs` | 인트로 → 타이틀(시작·설정) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도). 커서 없는 탐색 + 탭/클릭 보조. 한글 픽셀 폰트. 가방·지도는 틀만, 저널은 튜토리얼 퀘스트 4개(docs/decisions/0007) |
| 글자 성능 | `ui.rs`(glyph_plan, warm), `render.rs`(present) | 픽셀 폰트를 원래 12 물리 px(정수배)로 굽고 정수배로 키움 + 언어를 정할 때 그 언어 글자를 한 번에 미리 굽기. 대화 중 텍스처 올리기 160MB → 0(헤드리스 측정). 화면은 수정 전과 픽셀 단위로 같음. Safari 버벅임 대응(실기 확인은 사람) |
| 다국어 | `i18n.rs`, `ui.rs`, `tools/fonts/`, `web/index.html` | 한국어·영어·일본어·간체·브라질 포르투갈어(0033) 표, 언어별 폰트(일·중은 Ark Pixel 서브셋), 기기 선호 목록으로 시작, 설정에서 전환, 가짜 언어 Pseudo. CJK 줄바꿈(금칙)·넘침 대비(fit·탭 여백). 폰트 글자·자리표시자 테스트(docs/decisions/0013, 0014). 번역은 초안 |
| 초상화 | `tools/art/portraits.py`, `art.rs`, `render.rs`(draw_portrait), `cutscene.rs`(Face) | 88×88, 게임 픽셀의 2배 밀도. Naïl 보통·기쁨·걱정·결의, 이웃은 그림자(docs/decisions/0015) |
| 반딧불·산딸기 | `loot.rs`, `world.rs`(drop_loot), `render.rs`(draw_loot, HUD) | 화폐(자석·HUD·상태 탭), 회복(불운 보정), 최대 체력 5, 줍는 소리(음 올라감). 개발 시나리오 "떨어지는 것 확인"(docs/decisions/0016) |
| 능력치·장신구 | `equipment.rs`(Stats, CHARMS), `app.rs`(가방 끼기/빼기, 상태 탭) | 장신구 5종 2칸, 생명의 씨앗, 공격 속도 = 스윙 시계 배율(docs/decisions/0017) |
| 퀵 아이템 | `items.rs`, `player.rs`(State::Eat), `world.rs`(start_eating, finish_eating, revive), `render.rs`(퀵 슬롯·draw_eating) | 산딸기 1칸 회복, 쥬시 지렁이 가득 회복 + 목숨(최대 3), I·Y 먹기, Q·L1 넘기기. 먹는 시간 0.35초(맞으면 끊김, 굴파기로 끊기) + 먹기 대기 1.5초, 슬롯 덮개·터치 테두리(0038) |
| 필살기 | `player.rs`(State::Drill/Recover), `world.rs`, `render.rs`(trail·super dim), `sfx.rs`(engine) | 분노 게이지. 시동 0.3초(적 정지·화면 어둡게) → 화면 한 폭 돌진(≈320px), 연타로 벽까지 끌고 가 쾅(번쩍임). 엔진음 하나. 단위 테스트(docs/decisions/0018, 0024) |
| 모험(필드·던전) | `adventure.rs`, `map.rs`, `field.rs`, `dungeon.rs`, `room.rs`(격자), `render.rs`(draw_grid_art) | 연속 스크롤 필드, 자는 적, 저장 지점, 굴 입구 → 생성 던전(문 잠금·보물·휴식·보스·출구), 밀기 전환, 지도 탭, 저장 지점 부활. 단위 테스트(docs/decisions/0019) |
| 패링 | `player.rs`(parry_dir), `world.rs`(parry_feedback) | 스윙 첫 0.12초 앞쪽: 돌진 경직, 투사체 반사, SP +4(docs/decisions/0020). 창이 열린 뒤 0.45초는 새 창 없음(연타 방지), 성공하면 바로(0038) |
| 스테이지 고리 | `adventure.rs`, `camp.rs`, `route.rs`, `dungeon.rs`(Params), `app/stage.rs` | 필드 → 공터 보스 → 쉼터 → 길 지도(층 3 + 보스, 엇갈리지 않는 갈래, 다음 층만 보임) → 굴(굴·둥지·저장고·샘·보스) → 다음 쉼터. 쉼터: 체력 가득·저장 고리·포포 가게(씨앗)·보리·빈 자리, 공격 버튼 = 말 걸기. 쓰러지면 쉼터, 길은 새로. 단위 테스트(길 규칙 3, 고리 2)(docs/decisions/0030, 0031) |
| 마을 | `village.rs`, `adventure.rs`(World::village, Place::Village), `render.rs`(큰 나무·꾸미기·할매), `app/stage.rs`(꾸미기 가게·할매·뿌리 길) | 튜토리얼 뒤 시작·엔딩 뒤 돌아오는 허브. 큰 나무 잎 = 깬 횟수, 포포 꾸미기 가게(마을 씨앗 6가지), 보리·할매(실뜨기)·빈 자리(0045) |
| 모모 보스(세 번) | `enemy.rs`(Momo·Under·Pop), `species.rs`(MOMO), `dungeon.rs`(Params::boss), `adventure.rs`(MomoIntro·MomoBeaten), `app/stage.rs`(momo_beaten), `cutscene.rs`(momo_meet·momo_flee·momo_returns), `tools/art/critters.py`(MOMO_BLOOM) | 첫 길 끝방. 만날수록 한 수씩(뛰기·차기 → 굴 숨기 → 이어 뛰기), 쓰러지면 그 런은 보통 무리. 이길 때마다 빈 자리 꽃잎, 세 번째에 돌아와 마을 그루터기에 앉음(참 엔딩 조건 `momo`). 보스 막대를 화면 위로(0046) |
| 도토 보스(한 번) | `enemy.rs`(Doto·doto_next·Bury), `species.rs`(DOTO), `world.rs`(Mine·mines), `render.rs`(draw_mine), `adventure.rs`(DotoIntro·DotoBeaten), `app/stage.rs`(doto_beaten·큰 나무 독 발톱 줄), `cutscene.rs`(doto_meet·doto_returns), `tools/art/critters.py`(DOTO·DOTO_BACK), `portraits.py`(칸 30~32) | 둘째 길 끝방. 도토리 부채꼴(패링으로 되받기) → 높이 뛰기 → 도토리 묻기(발밑 + 둘레, 고리 둘, 파내기·굴로 넘김), 반 아래면 수가 늚. 이기면 씨앗 자리를 기억(`doto_memory`, 참 엔딩 조건 둘째), 마을 그루터기 밑. 큰 나무에서 독 발톱 깨우기(120, 런마다 가방에), 저널 목표 둘(0052) |
| 실의 꿈(0055) | `dream.rs`(보스 넷·열림·기록 키), `app/dream.rs`(고르기 창·꿈 싸움·깨어나기·HUD), `village.rs`(X), `adventure.rs`(Interact::Dream), `render.rs`(실 고리), `world.rs`(dream: 떨어뜨리지 않음) | 마을 할매 곁 실 고리에서 이긴 길의 주인과 빈 방에서 다시 싸움(모모는 세 번째 모습), 가장 빠른 기록·★(안 맞고 이김), 넷 다 이기면 이어서 꾸기, 얻는 것 없음, 저널 둘 |
| 콩이 보스(한 번) | `enemy.rs`(Kongi·Guard·kongi_next·shell_hit·break_guard·on_parried), `species.rs`(KONGI), `world.rs`(등딱지 튕김·열기), `player.rs`(recoil), `adventure.rs`(KongiIntro·KongiBeaten), `app/stage.rs`(kongi_beaten), `cutscene.rs`(kongi_meet·kongi_returns), `tools/art/critters.py`(KONGI)·`portraits.py`(칸 35~37) | 셋째 길 끝방. 구르기(벽에 튕기고 어질어질) ↔ 웅크려 버티기(발톱은 튕기고 바로 굴러 옴 — 패링 거울), 솟구치기·드릴로 등딱지 열기, 패링하면 구르기 멈춤. 이기면 바위 발톱(런마다 가방에), 마을 바위 곁, 저널 "깊은 곳의 문", 도감 목록 넘기기(0054) |
| 지도 그림(0062·0063) | `terrain.rs`(굽기·장식 자리·흙길), `render.rs`(구운 지도 + 바뀌는 칸), `dungeon.rs`(shape_room·KEEP) | 장소가 바뀔 때 지도를 한 장으로 굽는다: 잡음으로 물결치는 둥근 벽 윤곽, 3/4 벼랑(지층·뿌리·늘어진 잎)과 윗면(덤불·뿌리·돌), 바닥 큰 띠·반대 재질 얼룩·던전 흙길·그늘, 무리 지은 장식. 던전 방은 모서리·가장자리 혹·바위 무리로 굴처럼(문 앞 길·적 자리 보호, 이어짐 검사). 마을 큰 나무는 깨울 재주가 있으면 빛난다(0061). 굽기는 `terrain::Baker`로 프레임마다 12ms씩 나눠 하고 그동안 화면 검정·월드 틱 멈춤, 최근 장소 세 장은 남긴다(0068, 브라우저 가장 긴 프레임 133~150ms → 33ms) |
| 모모 플레이어블(0056) | `equipment.rs`(Character::Momo·WeaponKind::Kick·뒷발), `moveset.rs`(KICKS), `player.rs`(kit·Hop·Chain·stomp), `tuning.rs`(HOP_*·CHAIN_*), `app/pick.rs`(누구와 내려갈까), `app/stage.rs`(뿌리 길·reach_deepest), `cutscene.rs`(ending_momo·플레이어 배우), `illust.py`(momo_deep), `render.rs`(draw_momo_player) | 모모가 돌아온 뒤 마을 뿌리 길에서 Naïl/모모를 고름. 같은 버튼에 뒷발 3타·깡충(무적)·이어 뛰기(세 번 밟기), 발톱은 줍지 않음. 깬 수·실뜨기는 캐릭터마다, 모모 런은 모모 보스 없음, 가장 깊은 곳 → 모모 장면(`momo_deep`, 엔딩 깃발과 따로), 포포·할매 한마디, 저널 "꽃잎을 따라", 앨범 10장 |
| 작은 것들(0053) | `camp.rs`(z), `cutscene.rs`(susu_meet·Portrait::SUSU), `app/stage.rs`(수수 말 걸기), `illust.rs`·`tools/art/illust.py`(doto_memory), `knots.rs`(6~10), `story.rs`(TalkDoto·진행 줄), `tools/art/critters.py`(SUSU)·`portraits.py`(칸 33~34) | 첫 쉼터의 땅강아지 수수가 새싹 발톱을 만들어 줌(런마다 가방에, 저널 "파는 손"), 도토 기억 일러스트(앨범 9장 5×2), 실뜨기 10단계(걸음·보스 체력·패링 창·굴파기 대기·지렁이), 진행을 따르는 이웃 말 7줄 |
| 반딧불·큰 나무 재주 | `talents.rs`, `app/stage.rs`(bank_glowbugs·update_tree·draw_tree), `player.rs`·`world.rs`(재주 배율), `adventure.rs`(Interact::Tree), `sfx.rs`(드릴 소리 셋) | 쉼터·런 끝에 반딧불을 마을로. 마을 큰 나무에서 자리 넷 × 갈래 둘(굴파기·패링·드릴·먹기), 깨우기는 반딧불, 바꿔 끼기는 공짜(0047) |
| 임시 그림 다듬기 | `tools/art/portraits.py`(칸 12~17), `tools/art/illust.py`(bands, momo_bloom), `cutscene.rs`(Portrait::MOMO·HALMAE·MOM) | 돌아온 모모·누룩 할매·엄마 초상화, 일러스트 띠 경계 2×2 섞기, 모모 일러스트 몸·눈(0049) |
| 작은 정리(0051) | `app.rs`(상태 탭·도감 쪽), `story.rs`(TalkBori·TalkMomo), `portraits.py`(털빛 초상화, 12칸 줄), `illust.py`(큰 나무 가지) | 상태 탭에 재주, 대화창 Naïl 털빛, 도감 이야기·공격·요령 쪽(확인으로 넘김), 진행을 따르는 보리·모모 말, 큰 나무 가지 |
| 초상화 v4(0070·0071) | `tools/pixel`(점 도구·편집기), `tools/art/portraits.py`(fur_clumps·silhouette_tufts, 기본 v4) | 털 인물(Naïl·엄마·보리·모모·도토·포포)에 털 뭉치 경계·크기 섞인 실루엣 삐침·작은 하이라이트·빛 쪽 외곽선 한 단, Naïl 원통 목도리·평평한 발톱·정수리 한 가닥. 대화창에서 헤드리스 확인 |
| 일러스트 바탕(0072·0073) | `tools/art/illust.py`(set_style·SCENE_STYLE·pastel_bg·ambient_tint), `tools/art/sources/`, `tools/pixel/pxl.py`(pixelate --cel) | 일러스트 열 장 모두 v4: 여섯 장(흙더미·큰 나무·마지막 꽃·엄마 굴·보리·모모)은 파스텔·수채·유화(CC0·퍼블릭 도메인) 바탕을 도트화, 땅속·꿈 네 장은 v4 규칙만 |
| 저널 | `journal.rs`, `app.rs`(Tally·draw_journal), `app/stage.rs`(sync_journal·claim_journal·물웅덩이), `village.rs`(자리 7~9·W), `equipment.rs`(장신구 8), `tools/art/nail.py`(털빛 3) | 런을 넘어 남는 목표 31개(이야기·싸움·탐험·이웃·마을), 앞 목표 뒤에 보임, 받기로 반딧불·씨앗·저널 전용 꾸미기 3·털빛 3·새 장신구 3(0048). 장신구는 받으면 가방에·런마다 들고 시작, 받은 뒤 보상이 어디 있는지 표시(0050) |
| 오늘의 굴 | `daily.rs`, `adventure.rs`(camp_start_seeded), `app.rs`(TitleItem) | 날짜(UTC) 시드, 첫 쉼터에서, 가장 먼 쉼터·완주 기록, 타이틀 줄에 날짜·기록, 한 달 넘은 기록 지움(0044) |
| 실뜨기 | `knots.rs`, `enemy.rs`(max_hp·windup·apply_knots), `world.rs`(knots), `app.rs`(begin_run·KnotPick) | 첫 클리어로 1단계, 가장 높은 단계로 닿으면 다음 단계. 시작할 때 고르기. 매듭: 적 체력 ×1.25 · 예고 ×0.8 · 산딸기 ×0.6 · 가게 ×1.5 · 먹기 대기 ×1.6(0043) |
| 런의 끝 | `tuning.rs`(FINAL_STAGE), `route.rs`(is_final), `adventure.rs`(AdvEvent::Deepest), `app/stage.rs`(reach_deepest) | 필드 + 길 3개, 셋째 길 보스 칸 = 가장 깊은 곳 → 반쪽/참 엔딩(조건 깃발) → 타이틀, 클리어 수 기록, 반쪽 엔딩 뒤 포포 한마디(0042) |
| 기록·다회차 | `profile.rs`, `story.rs`, `app.rs`(start·on_death·album·설정), `web/index.html`(nail_save), `player.rs`(resist) | 깃발 + 수 기록(웹 localStorage·데스크톱 파일·Android 앱 폴더), 튜토리얼 뒤 새 런은 바로 필드·보리 뒤엔 대장 없음, 런 사이 한마디(가장 구체적인 줄·한 번), 도감 유지, 앨범(타이틀), 포포의 응원(20%+2%/쓰러짐, 최대 80%), 기록 지우기(0041) |
| 컷씬 일러스트 | `illust.rs`, `cutscene.rs`(Step::Illust·Narrate, cut_to, 이야기 장면), `render.rs`(draw_illust), `tools/art/illust.py` | 640×360 2배 밀도 한 장 그림 + 느린 패닝 + 입자, 내레이션 띠. 8장(오프닝 2·보리·모모·할매·마지막 꽃·새싹·둔덕) 임시 그림, 대사 다섯 언어. 오프닝·보리는 게임에, 나머지는 개발 "이야기 일러스트"(0037, 0039) |
| 튜토리얼 보스 | `enemy.rs`(Chief), `species.rs`(CHIEF), `field.rs`(공터), `tools/art/chief.py`, `render.rs`(보스 막대) | 들쥐 대장 보리: 돌진 ↔ 뛰어 찍기, 반 아래면 생쥐 떼, 공격 중 슈퍼아머, 벽에 박으면 2.2초 틈. 32×32 그림, 초상화 3, 처치 컷씬 → 쉼터의 이웃. 헤드리스로 공터 잠금·막대·처치 컷씬·쉼터·가게·길 지도 확인(docs/decisions/0032) |
| 웨이브 | `world.rs` | 적을 다 잡으면 1.2초 뒤 다음 웨이브(1C+1S → 2C+1S → 2C+2S → 3C+2S). 명세엔 없지만 5분 플레이 판정을 위해 넣음 |

확인한 것: 데스크톱(Xvfb) 실행 캡처로 렌더 방향·공격 반원·굴파기·히트박스 표시, 헤드리스 Chromium으로 웹 빌드 로드·입력·콘솔(mismatch 없음). 확인 못 한 것: 실제 손맛(판정 1~4), 실기 게임패드, 소리의 실제 느낌(컨테이너에 오디오 장치 없음).

## 사람이 할 일 (다음 세션 전)
2026-10-25에 정리했다. 끝난 것(Netlify 시크릿, 플레이 판정, Sunnyside 받기·라이선스·비공개 에셋 레포, 화면 이동·스토리·다회차 결정, 저장 파일, 기술 해금·가게 화면)은 지웠다. 기록은 각 결정(docs/decisions)에 있다.

### 먼저
- [ ] **본인 외 한 사람에게 건네기**(v0.1.0부터 남은 1단계 완료 조건): 안내 페이지 https://claude.ai/artifact/6NSJVxv4dSvx7e46EwjZ9j (원본 `docs/playtest/playtest.html`, 공유는 페이지의 Share 메뉴에서)를 건넨다. 설명 없이 바로 시작 → 10~15분 → 판정 기준 4개 + 기기·좋았던 것·답답했던 것을 "답 복사"로 받는다. 받은 답은 다음 세션에 붙여 넣는다
- [ ] **v0.1.5 확인**(0058~0074를 한 번에): 웹·APK에서
  - **APK가 설치되는지**(0074: v0.1.4까지는 Android 12+에서 "package appears to be invalid"로 거부됐다). 기기 모델·Android 버전을 함께 적는다
  - Sunnyside 그림(큰 나무·들판 나무·꽃·모닥불·버섯)이 나오는지, 장소를 바꿀 때 멈칫 없이 밝아지는지(검정이 0.2초쯤 길어진 게 거슬리지 않는지)
  - APK를 지우지 않고 덮어 설치했을 때 기록이 남는지, Android Chrome 글자 색, 타이틀 "v0.1.5 · <커밋>"
  - 모모 런: 포포 배웅과 쉼터 빈 자리 말(나일은 마을에서 기다린다)
  - 새 그림(0071~0073): 대화 초상화의 털·목도리, 컷씬 일러스트 열 장(특히 패닝하는 큰 나무·마지막 꽃이 끝까지 자연스러운지, 대화창에 얼굴이 가리지 않는지)
- [ ] GitHub에서 Secret `SECRETS_TOKEN`과 그 fine-grained 토큰을 지운다(0067, 이미 했으면 체크)
- [x] 공식 페이지 켜기(0069, 2026-10-25 사장이 Source = GitHub Actions, `pages` 4번째 실행 성공)
- [ ] **공식 페이지 보기**: https://aimenjoe.github.io/nail-under-the-roots/ 를 폰·PC로 보고 글·그림이 어색한 곳을 알려 준다

### 판정 (한 사람의 의견을 들은 뒤, 몰아서)
- [ ] **싸움 손맛**: 패링 창 0.12초·분노 +4, 행동 대기(먹기 0.35초·대기 1.5초, 0038), 공격 예고가 읽히는지(0025), 드릴 시동·거리·벽 쿵(0024), 장신구·퀵 아이템
- [ ] **보스**: 튜토리얼 보스 체력 36·슈퍼아머, 모모(0046) 셋째 이어 뛰기 세기, 도토(0052) 부채꼴·묻은 도토리, 콩이(0054) 튕김·버티기, 실의 꿈(0055) 보랏빛·체력 2 회복
- [ ] **모모 플레이어블**(0056): 깡충 56px·대기, 이어 뛰기 세 번 × 2, 뒷발 사거리, 작아 보이는지, 고르기 창, 모모 장면·`momo_deep` 그림
- [ ] **런 구조**: 필드 + 길 3개가 10~15분인지(0042), 필드 크기·적 밀도·던전 방 수·카메라 속도·부활 규칙, 반딧불·씨앗·산딸기 개수와 자석 반경, 포포의 응원 비율, 길 지도 갈래(다음 층만 보임), 쉼터 가게 값, 마을(0045) 꾸미기 값·잎 자람, 큰 나무 재주 값(0047)·빛남(0061), 저널 목표 숫자(0048), 실뜨기 1~10 어려움(0043), 오늘의 굴(0044)
- [ ] **그림**: 지도(0062·0063: 둥근 벽·바닥 띠가 어지럽지 않은지, 던전 바위가 싸움을 막는지), 적 다섯 종 구별, Naïl 스프라이트·초상화(표정 4, 털빛), 모모 물든 모습, 할매·엄마 초상화(0049), 컷씬 일러스트 길이·패닝(0039)
- [ ] **이야기·글**: 0037 뼈대에서 어색한 곳, 튜토리얼 9단계 길이·포포 말투(0023), 수수·이웃 말투
- [ ] **기기**: iPhone Safari(안티에일리어싱 끔, 0026·대화창 버벅임, 저전력 모드 끄고), 폰 터치 공격 버튼 크기·가게/길 지도 닫기(✕·바깥 탭), 패드 실기(다시 붙이기, 장치별 안내 0036, 닌텐도 배치), 화면 떨림(0034), 타이틀 글자 테두리(0035), 가게 사기 전 확인 창, 세로↔가로 초상화 크기, 껐다 켜도 기록이 남는지(0041)
- [ ] **번역**: 브라질 포르투갈어 검수(가능하면 원어민, 0033)

### 정할 것
- [ ] 필드 = 넓은 쉼터로 합칠지(0030 "제안")
- [ ] 굴토끼 가족(할매 실 속) 갈래를 이야기에 넣을지, 모모 전용 재주·장신구(0056)
- [ ] 컷씬 일러스트를 사람이 그릴지(본인/외주). 정하면 `docs/art/cutscene-illustrations.md`를 넘긴다(0039)
- [ ] 다음 캐릭터(0040 순서 7)
