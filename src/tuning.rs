//! 손맛 수치. 전부 여기에. (docs/phase-1-combat-prototype.md 초안 → 플레이하며 조정)
//!
//! 단위: 거리 px(내부 해상도 320×180 기준), 시간 초, 속도 px/s.

// ---- 시간 --------------------------------------------------------------
/// 로직 고정 타임스텝(60Hz).
pub const TICK_DT: f32 = 1.0 / 60.0;
/// 한 렌더 프레임이 이보다 길면 잘라서 "죽음의 나선"을 막는다.
pub const MAX_FRAME_DT: f32 = 0.25;

// ---- 방 ----------------------------------------------------------------
pub const VIEW_W: f32 = 320.0;
pub const VIEW_H: f32 = 180.0;
pub const WALL: f32 = 8.0;
/// 위쪽 벽 두께. 3/4 시점이라 벼랑 앞면(16px)이 보인다(docs/decisions/0006).
pub const WALL_TOP: f32 = 24.0;

// ---- 플레이어 ------------------------------------------------------------
pub const PLAYER_RADIUS: f32 = 6.0;
pub const PLAYER_SPEED: f32 = 90.0;
/// 최대 체력. 3이면 오래 못 버틴다(사장 피드백 2026-10-25) → 5 + 산딸기 회복(docs/decisions/0016).
pub const PLAYER_HP: i32 = 5;
/// 퀵 슬롯이 저절로 바뀌었을 때 깜빡이는 시간(0091).
pub const QUICK_FLASH: f32 = 1.2;
/// 피격 후 무적 시간.
pub const PLAYER_HURT_INVULN: f32 = 0.8;
pub const PLAYER_HURT_KNOCKBACK: f32 = 170.0;
pub const PLAYER_HURT_HITSTOP: f32 = 0.12;
pub const PLAYER_HURT_SHAKE: f32 = 0.55;
/// 입력 버퍼. 이 시간 안에 들어온 공격/굴파기 입력은 가능해지는 순간 실행된다.
pub const INPUT_BUFFER: f32 = 0.10;
/// 스틱 데드존(반지름). Android 빌드에는 gamepads 크레이트가 없어 쓰이지 않는다.
#[cfg_attr(target_os = "android", allow(dead_code))]
pub const STICK_DEADZONE: f32 = 0.2;
/// 메뉴 탐색(스틱): 큰 축이 이만큼 기울면 한 칸, 이 밑으로 돌아와야 다음 칸(히스테리시스).
/// 데드존을 뺀 뒤 0..1 값 기준. 0.5/0.3은 흔한 UI 내비게이션 기본값 근처(KB gamepad-and-android-input).
pub const NAV_PRESS: f32 = 0.5;
pub const NAV_RELEASE: f32 = 0.3;
/// 누르고 있을 때 반복: 처음 지연, 그 뒤 간격(초).
pub const NAV_REPEAT_DELAY: f32 = 0.35;
pub const NAV_REPEAT_EVERY: f32 = 0.1;

// ---- 발톱 3연타 ------------------------------------------------------------
/// 타별 스윙 길이. 1·2타 빠름, 3타 느림.
pub const CLAW_SWING_TIME: [f32; 3] = [0.12, 0.12, 0.20];
/// 타별 히트박스 활성 구간 (시작, 끝).
pub const CLAW_ACTIVE: [(f32, f32); 3] = [(0.02, 0.08), (0.02, 0.08), (0.05, 0.14)];
/// 3타 뒤 추가 경직.
pub const CLAW_RECOVERY: [f32; 3] = [0.0, 0.0, 0.14];
/// 스윙이 끝난 뒤 다음 타로 이어지는 입력 유효 시간.
pub const CLAW_COMBO_WINDOW: f32 = 0.30;
/// 공격 중 이동 속도 배율.
pub const CLAW_MOVE_FACTOR: f32 = 0.4;
/// 히트박스: 전방 반원 반지름.
pub const CLAW_RANGE: f32 = 14.0;
/// 스윙 시작 시 전방으로 살짝 전진.
pub const CLAW_LUNGE: f32 = 45.0;
pub const CLAW_DAMAGE: [i32; 3] = [1, 1, 2];
/// 3타(마무리)는 "그저 그렇다"(첫 바깥 판정, 0090): 넉백 230 → 330(마찰 9로 약 25px → 37px 밀림), 멈춤·흔들림·경직도 올림.
/// 마무리 타의 충격 이펙트·소리는 world(`finisher_impact`).
pub const CLAW_KNOCKBACK: [f32; 3] = [90.0, 90.0, 330.0];
pub const CLAW_STUN: [f32; 3] = [0.15, 0.15, 0.50];
pub const CLAW_HITSTOP: [f32; 3] = [0.05, 0.05, 0.13];
pub const CLAW_SHAKE: [f32; 3] = [0.15, 0.15, 0.55];

// ---- 굴파기 ------------------------------------------------------------
/// 땅속 최대 시간. 지나면 자동으로 솟구친다.
pub const BURROW_MAX_TIME: f32 = 0.35;
pub const BURROW_SPEED_FACTOR: f32 = 1.3;
/// 쿨다운(솟구친 순간부터).
pub const BURROW_COOLDOWN: f32 = 0.6;
/// 모모의 회피 깡충(0056): 체공 시간·거리·높이(그림). 공중에서는 안 맞고 적·탄·묻힌 도토리를 넘는다. 대기는 굴파기 대기와 같은 값을 쓴다.
pub const HOP_TIME: f32 = 0.34;
pub const HOP_DIST: f32 = 56.0;
pub const HOP_HEIGHT: f32 = 10.0;
/// 모모의 특기 이어 뛰기(0056): 기 게이지를 다 써서 세 번 뛴다. 뛸 때마다 입력 쪽으로 다시 겨누고, 착지마다 둘레에 피해.
pub const CHAIN_HOPS: u32 = 3;
pub const CHAIN_HOP_TIME: f32 = 0.30;
pub const CHAIN_HOP_DIST: f32 = 60.0;
pub const CHAIN_HOP_HEIGHT: f32 = 16.0;
pub const CHAIN_RADIUS: f32 = 28.0;
pub const CHAIN_DAMAGE: i32 = 2;
pub const CHAIN_KNOCKBACK: f32 = 180.0;
pub const CHAIN_STUN: f32 = 0.5;
pub const CHAIN_HITSTOP: f32 = 0.06;
pub const CHAIN_SHAKE: f32 = 0.35;
pub const CHAIN_RECOVER: f32 = 0.25;
/// 솟구침 범위 공격 반지름. 16이면 적 바로 밑에서만 맞아 "숨는 용도"로만 썼다(첫 바깥 판정, 0090) → 22.
pub const EMERGE_RADIUS: f32 = 22.0;
/// 솟구침 동작 시간(이동 불가).
pub const EMERGE_TIME: f32 = 0.12;
/// 솟구친 직후 공격 불가 시간(리스크). 솟구친 뒤 바로 발톱으로 잇게 0.10 → 0.06(0090).
pub const EMERGE_NO_ATTACK: f32 = 0.06;
/// 기습 보상: 평타 한 대(1)보다 세게(0090).
pub const EMERGE_DAMAGE: i32 = 2;
pub const EMERGE_KNOCKBACK: f32 = 340.0;
pub const EMERGE_STUN: f32 = 0.6;
pub const EMERGE_HITSTOP: f32 = 0.11;
pub const EMERGE_SHAKE: f32 = 0.5;
/// 밀쳐내기(0090): 맞는 반지름 밖, 이 반지름 안의 잡몹은 피해 없이 바깥으로 밀려나고 잠깐 멈춘다(보스는 안 밀린다).
/// 가까울수록 세게(바깥 끝은 `EMERGE_PUSH_EDGE`배).
pub const EMERGE_PUSH_RADIUS: f32 = 44.0;
pub const EMERGE_PUSH_KNOCKBACK: f32 = 260.0;
pub const EMERGE_PUSH_EDGE: f32 = 0.55;
pub const EMERGE_PUSH_STUN: f32 = 0.25;
/// 솟구칠 때(맞든 안 맞든) 흔들림. 땅이 터지는 느낌(예전 0.15).
pub const EMERGE_BURST_SHAKE: f32 = 0.25;

/// 이웃 NPC(포포)가 걷는 속도.
pub const NPC_SPEED: f32 = 70.0;

// ---- 적 ----------------------------------------------------------
// 종류별 체력·속도·예고 시간 등은 src/species.rs 표(docs/decisions/0021). 여기는 모든 종에 같은 것.
/// 뛰기형(청설모): 체공 높이(그리기 px), 내려앉기 직전 피해가 있는 시간.
pub const LEAP_HEIGHT: f32 = 14.0;
pub const LEAP_LAND_WINDOW: f32 = 0.06;
/// 던지는 도토리.
pub const PROJECTILE_RADIUS: f32 = 3.0;
pub const PROJECTILE_DAMAGE: i32 = 1;

// ---- 획득 ------------------------------------------------------------
/// 바닥 무기에 이만큼 다가가면 줍는다.
pub const PICKUP_RADIUS: f32 = 10.0;

// ---- 떨어지는 것: 반딧불(화폐)·산딸기(회복) (src/loot.rs, docs/decisions/0016) -------------
/// 튀어나오는 수평 속도(최대)와 위로 튀는 속도, 가짜 중력.
pub const LOOT_POP_SPEED: f32 = 70.0;
pub const LOOT_POP_UP: f32 = 90.0;
pub const LOOT_GRAVITY: f32 = 420.0;
/// 바닥에서 미끄러짐이 멎는 정도(초당).
pub const LOOT_FRICTION: f32 = 5.0;
/// 나온 뒤 이 시간 동안은 줍지 않는다(튀어나오는 걸 보게).
pub const LOOT_PICKUP_DELAY: f32 = 0.35;
/// 이 거리 안이면 줍는다.
pub const LOOT_COLLECT_RADIUS: f32 = 8.0;
/// 반딧불 자석: 이 거리 안이면 끌려온다(넉넉하게 — "스쳤는데 못 주웠다"가 없게).
pub const GLOWBUG_MAGNET_RADIUS: f32 = 44.0;
pub const GLOWBUG_MAGNET_ACCEL: f32 = 900.0;
pub const GLOWBUG_MAGNET_MAX_SPEED: f32 = 230.0;
/// 산딸기가 떨어질 확률 = 기본 + 깎인 체력 1칸마다. 체력이 가득이면 0.
pub const BERRY_BASE_CHANCE: f32 = 0.06;
pub const BERRY_CHANCE_PER_MISSING: f32 = 0.08;
/// 체력이 깎인 채로 이만큼 처치해도 안 나왔으면 다음엔 반드시(불운 보정).
pub const BERRY_PITY_KILLS: u32 = 6;
/// 산딸기 하나가 채우는 체력.
pub const BERRY_HEAL: i32 = 1;
/// 쥬시 지렁이: 처치마다 이 확률로 떨어진다(체력과 무관, 드묾).
pub const WORM_CHANCE: f32 = 0.035;

// ---- 가방(퀵 아이템, src/items.rs, docs/decisions/0017) ----------------------------------
/// 들고 다닐 수 있는 수. 가득이면 바닥에 남는다.
pub const BERRY_MAX: u8 = 5;
/// 쥬시 지렁이 = 목숨. 들고 있다가 쓰러지면 저절로 먹고 일어난다. 최대 목숨 = 1 + 이것.
pub const WORM_MAX: u8 = 2;
/// 먹는 데 걸리는 시간(docs/decisions/0038). 이 동안 걸음이 느려지고 공격은 못 한다. 다 먹어야 회복된다.
/// 맞으면 끊기고 먹지 않은 것으로(아이템은 그대로). 굴파기로는 끊을 수 있다(먹지 않음).
pub const BERRY_EAT_TIME: f32 = 0.35;
pub const WORM_EAT_TIME: f32 = 0.6;
pub const EAT_MOVE_FACTOR: f32 = 0.5;
/// 먹기 시작부터 다음 먹기까지(끊겨도, 지렁이로 일어나도 같다). 같은 행동이 바로 다시 나가지 않게.
pub const ITEM_COOLDOWN: f32 = 1.5;
/// 쓰러졌다 일어날 때 체력(최대의 비율, 올림)과 무적 시간.
pub const REVIVE_HP_FRAC: f32 = 0.6;
pub const REVIVE_INVULN: f32 = 2.0;
/// 일어날 때 주변을 밀어내는 충격.
pub const REVIVE_PUSH_RADIUS: f32 = 40.0;
pub const REVIVE_PUSH_KNOCKBACK: f32 = 260.0;
/// 상점 화폐 씨앗: 보물 상자 한 줌(최소, 최대), 보스.
pub const CHEST_SEEDS: (u32, u32) = (3, 5);
pub const BOSS_SEEDS: u32 = 6;
/// 끝방 출구는 마지막으로 쓰러진 자리·플레이어에서 이만큼(px) 넘게 떨어진 칸에 생긴다(0060). 보상을 줍기 전에 빠지지 않게.
pub const EXIT_MIN_DIST: f32 = 80.0;
/// 잠긴 문(물든 뿌리 + 보라 실, 0094)이 자라 막는 시간 / 풀려 걷히는 시간.
pub const DOOR_CLOSE_TIME: f32 = 0.35;
pub const DOOR_OPEN_TIME: f32 = 0.5;
/// 뿌리 심장으로 늘릴 수 있는 최대 체력 상한(기본 PLAYER_HP 위로).
pub const ROOT_HEART_MAX: i32 = 5;

// ---- 패링(공격 타이밍, docs/decisions/0020) -----------------------------------------
/// 스윙을 시작하고 이 시간 안에 앞쪽에서 공격이 닿으면 튕겨낸다(실제 시간, 공격 속도와 무관).
pub const PARRY_WINDOW: f32 = 0.12;
/// 앞쪽 판정: 공격 방향과 위협 방향의 내적이 이보다 크면.
pub const PARRY_FRONT: f32 = 0.2;
/// 패링 창이 열린 순간부터 다음 창이 열릴 수 있기까지(docs/decisions/0038). 연타로 창을 이어 붙이지 못하게.
/// 성공하면 바로 다시 열 수 있다(여럿을 연달아 튕겨내는 건 보상).
pub const PARRY_COOLDOWN: f32 = 0.45;
pub const PARRY_SP: f32 = 4.0;
pub const PARRY_HITSTOP: f32 = 0.12;
pub const PARRY_SHAKE: f32 = 0.35;
/// 튕겨낸 돌격형: 긴 경직 + 뒤로 밀림.
pub const PARRY_STUN: f32 = 1.1;
pub const PARRY_KNOCKBACK: f32 = 220.0;
/// 되돌린 투사체: 빨라지고 적을 때린다.
pub const PARRY_REFLECT_SPEED: f32 = 1.4;
pub const PARRY_REFLECT_DAMAGE: i32 = 2;

// ---- 모험(필드·던전, src/adventure.rs, docs/decisions/0019) ------------------------------
/// 실뜨기 매듭(src/knots.rs, docs/decisions/0043): 적 체력 배율, 예고 배율, 산딸기 확률 배율, 가게 값 배율, 먹기 대기 배율.
pub const KNOT_ENEMY_HP: f32 = 1.25;
pub const KNOT_WINDUP: f32 = 0.8;
pub const KNOT_BERRY: f32 = 0.6;
pub const KNOT_PRICE: f32 = 1.5;
pub const KNOT_EAT_CD: f32 = 1.6;
/// 실뜨기 6~10(0053): 적 걸음 ×1.15, 길의 주인 체력 ×1.25(1단계에 더해), 패링 창 ×0.75, 굴파기 대기 ×1.25, 지렁이 안 떨어짐.
pub const KNOT_SPEED: f32 = 1.15;
pub const KNOT_BOSS_HP: f32 = 1.25;
pub const KNOT_PARRY: f32 = 0.75;
pub const KNOT_BURROW_CD: f32 = 1.25;
/// 마을 씨앗(docs/decisions/0045): 가장 깊은 곳에 닿으면 남은 씨앗 + 이것, 새 실뜨기 단계 + 이것, 오늘의 굴 완주 + 이것.
pub const VILLAGE_SEEDS_CLEAR: u32 = 5;
pub const VILLAGE_SEEDS_KNOT: u32 = 10;
pub const VILLAGE_SEEDS_DAILY: u32 = 5;
/// 한 런 = 필드 + 길 몇 개. 이 번째 길의 주인을 이기면 가장 깊은 곳(엔딩, docs/decisions/0042).
pub const FINAL_STAGE: u32 = 3;
/// 장소를 바꿀 때 어두워지는(그리고 밝아지는) 시간.
pub const FADE_HALF: f32 = 0.3;
/// 카메라가 목표를 따라가는 빠르기(초당, 지수 감쇠). 던전 방을 넘을 때 이 속도로 미끄러진다.
pub const CAM_FOLLOW: f32 = 9.0;
/// 필드의 자는 적이 깨는 거리.
pub const ENEMY_AGGRO_RADIUS: f32 = 110.0;
/// 던전 전투방에 들어선 뒤 적이 움직이기 시작하기까지.
pub const ROOM_WAKE_DELAY: f32 = 0.7;

// ---- 필살기: 드릴 돌진 (player.rs State::Drill, docs/decisions/0018, 0024) ---------------------
/// SP 게이지 최대. 발톱이 맞으면 1, 마지막 타는 2 찬다(장신구로 배율).
pub const SP_MAX: f32 = 12.0;
pub const SP_PER_HIT: f32 = 1.0;
pub const SP_FINISHER_BONUS: f32 = 1.0;
/// 시동(부아아…): 제자리에서 돌며 힘을 모은다. 이동 없음·무적, 적·투사체는 멈춘다(필살기 정지 연출).
/// 이 동안은 방향을 마음대로 고른다.
pub const DRILL_REV: f32 = 0.3;
/// 돌진(…아앙): 시동 뒤 이 시간. 처음 DRILL_ACCEL 동안 START → SPEED로 붙는다. 거리 ≈ 화면 한 폭(320px).
pub const DRILL_TIME: f32 = 0.9;
pub const DRILL_SPEED_START: f32 = 230.0;
pub const DRILL_SPEED: f32 = 380.0;
pub const DRILL_ACCEL: f32 = 0.2;
/// 직진성: 초당 이만큼(라디안)만 방향을 틀 수 있다.
pub const DRILL_TURN: f32 = 2.2;
/// 연타 간격과 한 번의 피해. 넉백이 돌진보다 세서 적이 앞으로 튕겨 나갔다가 다시 잡힌다(벽까지 끌고 간다).
pub const DRILL_HIT_EVERY: f32 = 0.06;
pub const DRILL_RADIUS: f32 = 12.0;
pub const DRILL_DAMAGE: i32 = 1;
pub const DRILL_KNOCKBACK: f32 = 430.0;
pub const DRILL_STUN: f32 = 0.25;
pub const DRILL_HITSTOP: f32 = 0.025;
pub const DRILL_HIT_SHAKE: f32 = 0.14;
/// 돌진하는 동안 계속 걸리는 흔들림(틱당 trauma).
pub const DRILL_RIDE_SHAKE: f32 = 0.012;
/// 벽 쿵: 부딪힌 자리 충격파.
pub const DRILL_WALL_RADIUS: f32 = 36.0;
pub const DRILL_WALL_DAMAGE: i32 = 4;
pub const DRILL_WALL_KNOCKBACK: f32 = 260.0;
pub const DRILL_WALL_STUN: f32 = 0.9;
pub const DRILL_WALL_HITSTOP: f32 = 0.2;
pub const DRILL_WALL_SHAKE: f32 = 1.0;
/// 벽 쿵 순간 화면이 하얗게 번쩍(초).
pub const DRILL_WALL_FLASH: f32 = 0.12;
/// 드릴이 끝나거나 벽에 부딪힌 뒤 경직.
pub const DRILL_RECOVER: f32 = 0.22;
pub const DRILL_WALL_RECOVER: f32 = 0.4;
/// 획득 창: 최소 표시 시간(연타로 바로 닫히지 않게), 자동으로 닫히는 시간.
pub const GIFT_MIN: f32 = 0.5;
pub const GIFT_AUTO_CLOSE: f32 = 3.5;

// ---- 독 ------------------------------------------------------------
/// 독은 이 간격마다 체력 1을 깎는다. 경직·넉백은 없다.
pub const POISON_TICK: f32 = 0.8;
/// 독 발톱 1·2타가 거는 독(초), 마지막 타가 거는 독(초).
pub const VENOM_POISON: f32 = 0.9;
pub const VENOM_POISON_FINISHER: f32 = 2.5;

// ---- 공통 전투 ------------------------------------------------------------
/// 넉백 속도 감쇠(1/s). v *= exp(-k·dt).
pub const KNOCKBACK_FRICTION: f32 = 9.0;
/// 피격 플래시(흰색) 시간.
pub const HIT_FLASH: f32 = 0.08;
/// 적끼리 겹치지 않게 밀어내는 힘.
pub const ENEMY_SEPARATION_SPEED: f32 = 40.0;
pub const ENEMY_DIE_SHAKE: f32 = 0.2;

// ---- 화면 흔들림 ------------------------------------------------------------
/// 최대 오프셋 px. 실제 오프셋 = max × trauma².
pub const SHAKE_MAX_OFFSET: f32 = 4.0;
/// trauma 감쇠(1/s).
pub const SHAKE_DECAY: f32 = 2.2;

// ---- 사망 연출 ------------------------------------------------------------
/// 쓰러진 뒤 화면이 어두워지는 시간.
pub const DEATH_FADE: f32 = 0.8;
/// 쓰러진 뒤 입력을 버리는 시간. 연타 중이던 손이 멈출 여유. 이 뒤에 안내가 뜨고 새 눌림만 받는다.
pub const DEATH_INPUT_LOCK: f32 = 1.2;

// ---- 컷씬 ------------------------------------------------------------
/// 레터박스 띠 높이(논리 px)와 내려오는/걷히는 시간.
pub const LETTERBOX_BAR: f32 = 18.0;
pub const LETTERBOX_TIME: f32 = 0.4;
/// 컷씬 중 화면 가장자리 비네트 세기(0..1).
pub const CUTSCENE_VIGNETTE: f32 = 0.75;

// ---- 웨이브 ------------------------------------------------------------
pub const WAVE_RESPAWN_DELAY: f32 = 1.2;
/// 스폰 지점이 플레이어와 이보다 가까우면 피한다.
pub const SPAWN_MIN_DIST: f32 = 70.0;

// ---- 사운드 ------------------------------------------------------------
pub const MASTER_VOLUME: f32 = 0.5;

// ---- 보스(들쥐 대장, docs/decisions/0032) ------------------------------------------
/// 울부짖기 예고 = 예고 시간 × 이 배율.
pub const CHIEF_HOWL_MUL: f32 = 1.6;
/// 부하를 부르는 최대 횟수(한 번에 생쥐 떼 하나 = 셋).
pub const CHIEF_SUMMONS: u32 = 2;
/// 공격이 아닐 때 맞으면 멈칫하는 시간 배율(보통 적의 경직 × 이것).
pub const CHIEF_STUN_MUL: f32 = 0.35;

// ---- 큰 나무의 재주(src/talents.rs, docs/decisions/0047) -------------------------------
/// 굴파기 A 깊은 굴: 땅속 시간·솟구침 반지름·대기 배율. B 얕은 굴.
pub const TAL_DEEP_TIME: f32 = 0.6;
pub const TAL_DEEP_RADIUS: f32 = 30.0;
pub const TAL_DEEP_CD: f32 = 1.4;
pub const TAL_SHALLOW_TIME: f32 = 0.2;
pub const TAL_SHALLOW_RADIUS: f32 = 16.0;
pub const TAL_SHALLOW_CD: f32 = 0.55;
/// 패링 A 되받아치기: 튕겨낸 돌격에 주는 피해(게이지는 0). B 빛 모으기: 패링 한 번에 차는 게이지.
pub const TAL_PARRY_COUNTER: i32 = 2;
pub const TAL_PARRY_SP: f32 = 8.0;
/// 드릴 A 긴 드릴: 돌진 시간 배율·게이지 배율. B 짧은 드릴.
pub const TAL_LONG_DRILL: f32 = 1.5;
pub const TAL_LONG_DRILL_SP: f32 = 0.75;
pub const TAL_SHORT_DRILL: f32 = 0.55;
pub const TAL_SHORT_DRILL_SP: f32 = 1.6;
/// 먹기 A 꼭꼭 씹기: 먹는 시간 배율·산딸기 회복. B 한입에: 먹는 시간 배율.
pub const TAL_CHEW_TIME: f32 = 2.0;
pub const TAL_CHEW_HEAL: i32 = 2;
pub const TAL_GULP_TIME: f32 = 0.4;

// ---- 모모(첫 길의 보스, 세 번, docs/decisions/0046) -----------------------------------
/// 뒷발 차기(짧은 돌진) 시간. 높이 뛰기 체공은 종 표의 dash_time.
pub const MOMO_KICK_TIME: f32 = 0.2;
/// 굴로 숨기: 흙 둔덕이 플레이어를 따라오는 시간과 속도(플레이어 걸음보다 느리다). 멈춘 뒤 예고 시간만큼 떨다가 튀어나온다.
pub const MOMO_DIG_TRACK: f32 = 1.1;
pub const MOMO_DIG_SPEED: f32 = 64.0;
/// 튀어나오는 동작 시간과 그 첫머리의 둘레 피해 시간.
pub const MOMO_POP_TIME: f32 = 0.35;
pub const MOMO_POP_WINDOW: f32 = 0.1;
/// 세 번째 싸움: 높이 뛰기를 이어서 몇 번. 이어 뛰는 웅크림은 예고의 이 비율에서 시작한다(짧다).
pub const MOMO_HOP_CHAIN: u32 = 3;
pub const MOMO_CHAIN_SKIP: f32 = 0.45;
/// 몇 번 이기면 돌아오나(0037 3-A).
pub const MOMO_FIGHTS: u32 = 3;

// ---- 도토(둘째 길의 보스, docs/decisions/0052) -----------------------------------------
/// 부채꼴 던지기: 도토리 수와 이웃 사이 각(라디안). 체력이 반 아래면 둘 더.
pub const DOTO_FAN: usize = 3;
pub const DOTO_FAN_HURT: usize = 5;
pub const DOTO_FAN_SPREAD: f32 = 0.32;
/// 도토리 묻기: 땅을 파는 예고(종 예고 × 이 배율), 묻는 수(하나는 플레이어 발밑, 나머지는 둘레), 둘레 거리.
pub const DOTO_BURY_MUL: f32 = 1.2;
pub const DOTO_MINES: usize = 3;
pub const DOTO_MINES_HURT: usize = 4;
pub const DOTO_MINE_SPREAD: f32 = 34.0;
/// 묻힌 도토리: 터지기까지(흙이 부풀며 고리가 좁혀 온다), 터지는 둘레. 발톱으로 미리 파내면 안 터진다.
pub const DOTO_MINE_FUSE: f32 = 1.3;
pub const DOTO_MINE_RADIUS: f32 = 14.0;
// ---- 콩이(셋째 길의 보스, docs/decisions/0054) -----------------------------------------
/// 구르기: 벽에 몇 번 튕기나(체력이 반 아래면 더). 튕길 때 플레이어 쪽으로 이만큼 꺾는다.
pub const KONGI_BOUNCES: u32 = 2;
pub const KONGI_BOUNCES_HURT: u32 = 4;
pub const KONGI_BOUNCE_AIM: f32 = 0.3;
/// 웅크려 버티기 시간. 그동안 발톱은 등딱지에 튕긴다. 치면 바로 굴러 온다(예고의 이 비율에서 시작).
pub const KONGI_GUARD_TIME: f32 = 1.4;
/// 버티기는 발톱이 닿을 만큼 다가온 뒤에(미끼: 치면 튕기고 바로 굴러 온다).
pub const KONGI_GUARD_NEAR: f32 = 40.0;
pub const KONGI_COUNTER_SKIP: f32 = 0.5;
/// 등딱지에 튕긴 Naïl: 밀려나는 세기, 그동안 다시 못 휘두르는 시간.
pub const KONGI_RECOIL: f32 = 170.0;
pub const KONGI_RECOIL_LOCK: f32 = 0.35;
/// 솟구치기(아래 배)·드릴로 등딱지를 열면 이만큼 어질어질.
pub const KONGI_BREAK_STUN: f32 = 1.6;

/// 깨운 독 발톱(큰 나무, 0052)의 값(반딧불).
pub const VENOM_AWAKE_COST: u32 = 120;

// ---- 쉼터·상점(docs/decisions/0030) ---------------------------------------------------
/// 상점 값(씨앗). 산딸기 · 쥬시 지렁이 · 뿌리 심장(최대 체력 +1, 쉼터마다 하나) · 장신구.
pub const PRICE_BERRY: u32 = 3;
pub const PRICE_WORM: u32 = 10;
pub const PRICE_HEART: u32 = 14;
pub const PRICE_CHARM: u32 = 12;
/// 길 지도의 끝 무리(굴·둥지)를 잡으면 나오는 씨앗.
pub const PACK_SEEDS: u32 = 3;
/// 이웃 곁에서 말 걸기 버튼이 먹히는 거리(px).
pub const INTERACT_RADIUS: f32 = 22.0;
