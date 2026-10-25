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
pub const PLAYER_HP: i32 = 3;
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
pub const CLAW_KNOCKBACK: [f32; 3] = [90.0, 90.0, 230.0];
pub const CLAW_STUN: [f32; 3] = [0.15, 0.15, 0.40];
pub const CLAW_HITSTOP: [f32; 3] = [0.05, 0.05, 0.10];
pub const CLAW_SHAKE: [f32; 3] = [0.15, 0.15, 0.40];

// ---- 굴파기 ------------------------------------------------------------
/// 땅속 최대 시간. 지나면 자동으로 솟구친다.
pub const BURROW_MAX_TIME: f32 = 0.35;
pub const BURROW_SPEED_FACTOR: f32 = 1.3;
/// 쿨다운(솟구친 순간부터).
pub const BURROW_COOLDOWN: f32 = 0.6;
/// 솟구침 범위 공격 반지름.
pub const EMERGE_RADIUS: f32 = 16.0;
/// 솟구침 동작 시간(이동 불가).
pub const EMERGE_TIME: f32 = 0.12;
/// 솟구친 직후 공격 불가 시간(리스크).
pub const EMERGE_NO_ATTACK: f32 = 0.10;
pub const EMERGE_DAMAGE: i32 = 1;
pub const EMERGE_KNOCKBACK: f32 = 190.0;
pub const EMERGE_STUN: f32 = 0.45;
pub const EMERGE_HITSTOP: f32 = 0.08;
pub const EMERGE_SHAKE: f32 = 0.35;

// ---- 적: 돌격형 ----------------------------------------------------------
pub const CHARGER_RADIUS: f32 = 7.0;
pub const CHARGER_HP: i32 = 3;
pub const CHARGER_WALK_SPEED: f32 = 35.0;
/// 이 거리 안이면 돌진 준비.
pub const CHARGER_TRIGGER_RANGE: f32 = 75.0;
/// 돌진 전 예고(멈춰서 떨림).
pub const CHARGER_WINDUP: f32 = 0.45;
pub const CHARGER_CHARGE_SPEED: f32 = 230.0;
pub const CHARGER_CHARGE_TIME: f32 = 0.35;
/// 돌진 후 멈춤.
pub const CHARGER_REST: f32 = 0.7;
/// 벽에 박았을 때 멈춤(더 김 = 반격 기회).
pub const CHARGER_WALL_REST: f32 = 1.1;
pub const CHARGER_DAMAGE: i32 = 1;

// ---- 적: 원거리형 ----------------------------------------------------------
pub const SHOOTER_RADIUS: f32 = 6.0;
pub const SHOOTER_HP: i32 = 2;
pub const SHOOTER_SPEED: f32 = 40.0;
/// 이보다 가까우면 물러남.
pub const SHOOTER_KEEP_MIN: f32 = 60.0;
/// 이보다 멀면 다가옴.
pub const SHOOTER_KEEP_MAX: f32 = 110.0;
/// 발사 전 조준 예고.
pub const SHOOTER_AIM_TIME: f32 = 0.35;
pub const SHOOTER_FIRE_INTERVAL: f32 = 1.6;
pub const PROJECTILE_SPEED: f32 = 65.0;
pub const PROJECTILE_RADIUS: f32 = 3.0;
pub const PROJECTILE_DAMAGE: i32 = 1;

// ---- 튜토리얼 허수아비 ------------------------------------------------------------
pub const DUMMY_HP: i32 = 6;

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

// ---- 웨이브 ------------------------------------------------------------
pub const WAVE_RESPAWN_DELAY: f32 = 1.2;
/// 스폰 지점이 플레이어와 이보다 가까우면 피한다.
pub const SPAWN_MIN_DIST: f32 = 70.0;

// ---- 사운드 ------------------------------------------------------------
pub const MASTER_VOLUME: f32 = 0.5;
