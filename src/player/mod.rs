//! 플레이어. Naïl: 이동, 발톱 3연타, 굴파기, 필살기 드릴. 모모(0056): 발차기 3연타, 깡충(회피), 이어 뛰기(특기).
//! `Actions`와 `Room`만 본다. 캐릭터는 `kit`로 고른다(회피·특기만 다르고 나머지 틀은 같다, 0003).
//!
//! 상태 기계: Free ⇄ Attack / Burrow → Emerge → Free(모모는 Hop → Free), Free·Attack → Drill → Recover → Free
//! (모모는 Chain × 3 → Recover), Free → Eat → Free. 피격·사망은 어느 상태에서나.
//! 모든 행동은 바로 다시 나가지 않는다(docs/decisions/0038): 발톱 = 스윙·후딜레이, 굴파기 = 대기, 필살기 = 게이지, 먹기 = 먹는 시간 + 대기, 패링 창 = 대기.
//! 수치는 `stats`(장비·장신구가 정한 것, equipment::Stats)를 거친다.
//! 명세 초안: docs/phase-1-combat-prototype.md. 수치: tuning.rs.

use macroquad::prelude::*;

use crate::combat::{self, Shape, Strike, StrikeKind};
use crate::equipment::Stats;
use crate::input::Actions;
use crate::items::Consumable;
use crate::moveset::{self, Moveset};
use crate::room::Room;
use crate::tuning as t;

// 0170 R9: 상태 함수를 셋으로 나눴다 — 발톱(attack), 굴파기·드릴(burrow), 모모의 뛰기(momo). 본문은 그대로.
mod attack;
mod burrow;
mod doto;
mod momo;
mod run;

pub use doto::Spawn;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Free,
    Attack {
        idx: usize,
        time: f32,
        dir: Vec2,
    },
    /// 강공격 모으기(0161·0167): 어느 타든 활성 구간이 지났는데 공격을 계속 누르고 있으면. `HEAVY_CHARGE` 뒤 떼면 `Heavy`,
    /// 그 전에 떼면 그냥 선다. `after`는 강공격 뒤 이어질 콤보 번호(1타 뒤면 2타) — 콤보 사이 어디에든 끼운다.
    /// 평타는 누르는 순간 나가므로(0167) 모으기는 늘 평타 뒤다.
    Charging {
        time: f32,
        dir: Vec2,
        after: usize,
    },
    /// 강공격 스윙(0161): `Moveset::heavy`. 첫머리 창은 둘레 공격(진홍)도 받아친다. 끝나면 `after`번 타로 콤보가 이어진다.
    /// `perfect` = 다 모인 뒤 `HEAVY_PERFECT` 안에 뗐다(0168): 피해 +1, 더 긴 멈춤.
    Heavy {
        time: f32,
        dir: Vec2,
        after: usize,
        perfect: bool,
    },
    Burrow {
        time: f32,
    },
    /// `charge`: 땅속에서 모은 힘 0~1(0158). 솟구침 크기·이펙트가 따른다.
    Emerge {
        time: f32,
        charge: f32,
    },
    /// 필살기: 시동(`time < DRILL_REV`, 제자리) 뒤 직진하며 연타. `dir`은 조금씩만 틀어진다. `next_hit`까지 남은 시간.
    /// `aim`은 시동 중 입력이 가리킨 쪽, `dir`은 거기서 가까운 적 쪽으로 맞춘 실제 방향(`drill_assist`, 0139).
    Drill {
        time: f32,
        dir: Vec2,
        aim: Vec2,
        next_hit: f32,
    },
    /// 드릴 뒤 경직.
    Recover {
        time: f32,
    },
    /// 모모의 깡충(회피): `dir`로 HOP_DIST만큼 뛴다. 공중에서는 안 맞는다.
    Hop {
        time: f32,
        dir: Vec2,
    },
    /// 도토의 뒤로 뛰기(회피, 0195): `dir`로 `leap_dist`만큼. 공중에서는 안 맞는다. 겨눈 쪽은 그대로.
    Leap {
        time: f32,
        dir: Vec2,
    },
    /// 달리기(0209, 모모·도토): 깡충·뒤로 뛰기가 내려앉았는데 회피를 아직 쥐고 있으면. `time` = 달린 시간(도움닫기).
    /// 떼면 `Jump`(짧게 달렸으면 그냥 선다), 공격을 누르면 그 자리에서 평타. 공중이 아니라 맞는다.
    Run {
        time: f32,
    },
    /// 멀리뛰기(0209): 달리다 떼면. `dist`는 도움닫기만큼 멀다. 공중에서는 안 맞는다.
    /// 모모는 내려앉으며 둘레를 밟고, 도토는 공중에서 한 번 던질 수 있다(`thrown` = 이미 던졌다).
    Jump {
        time: f32,
        dir: Vec2,
        dist: f32,
        thrown: bool,
    },
    /// 모모의 이어 뛰기(특기): 남은 뛰기 수, 이번 뛰기 시간·방향. 착지마다 둘레에 피해.
    Chain {
        left: u32,
        time: f32,
        dir: Vec2,
    },
    /// 퀵 아이템을 먹는 중. 다 먹으면 `Event::Ate`(가방에서 꺼내 회복하는 건 월드). 맞으면 끊긴다.
    Eat {
        item: Consumable,
        time: f32,
    },
    /// 쓰러졌는데 쥬시 지렁이가 있었다(0243): `REVIVE_DOWN_TIME` 동안 누워 지렁이를 먹고, 끝나면 일어나며 `Event::Rose`.
    /// 이 동안은 안 맞고 움직이지 못한다. 체력은 일어날 때 채운다(그 전엔 0 — HUD도 쓰러진 걸 보인다).
    Revive {
        time: f32,
    },
    Dead,
}

/// 콤보 중에 눌러 둔 다음 타(0174): 박자 창 안이면 `Beat`(그 자리에서 나가고 덤), 창 전이면 `Early`(박자까지 기다렸다 나간다).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queued {
    Early,
    Beat,
}

/// 박자 고리를 그릴 때 보는 것(0174).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatCue {
    /// 스윙 시작 0 → 박자 1(그 뒤로도 자란다).
    pub k: f32,
    /// 지금 박자 창 안인가.
    pub window: bool,
    /// 창 전에 눌러 박자를 기다리는 중인가.
    pub waiting: bool,
}

/// 이번 틱에 일어난 일(사운드·이펙트용). 로직은 이걸 다시 읽지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Swing(usize),
    /// 박자에 맞춰 이어진 타(0167·0174): 퍼지는 고리·연속 수만큼 오르는 음. 값은 연속 수(1부터).
    Beat(usize),
    /// 박자가 왔다(0180): 콤보 중 박자 시계가 박자를 지나는 틱. 그 틱에 타가 나가지 않았을 때만 — 메트로놈 '똑' + 과녁 번쩍.
    BeatTick,
    /// 강공격이 나갔다 / 다 모여 발톱이 빛난다(0161). `HeavyPerfect`는 다 모인 직후에 뗐다(0168).
    Heavy,
    HeavyReady,
    HeavyPerfect,
    /// 모으기 시작(0168): 올라가는 소리.
    ChargeStart,
    Burrow,
    Emerge,
    /// 정한 시간을 지나 누르고 있다: 힘을 모으기 시작(땅이 운다, 0158).
    Rumble,
    Hurt,
    Die,
    /// 필살기 시동(소리 시작).
    DrillRev,
    /// 시동이 끝나고 튀어 나감.
    Drill,
    DrillWall,
    Parry,
    /// 다 먹었다(월드가 가방에서 꺼내 회복).
    Ate(Consumable),
    /// 모모가 뛰어오름 / 내려앉음(이어 뛰기 착지는 true).
    Hop,
    Land(bool),
    /// 모모 깡충 착지 밟기(재주 발 구르기, 0142).
    Stamp,
    /// 회피의 밀치기(0214): 모모 깡충 착지·도토 뒤로 뛰기 도약.
    Shove,
    /// 멀리뛰기 착지(0209). `stomp` = 모모의 밟기(둘레 `jump_stomp_radius`).
    JumpLand {
        stomp: bool,
    },
    /// 도토가 도토리를 묻었다(0195).
    Bury,
    /// 지렁이를 먹고 일어났다(0243): 월드가 둘레를 밀어내고 연출.
    Rose,
}

pub struct Player {
    pub pos: Vec2,
    /// 직전 틱 위치(렌더 보간용).
    pub prev_pos: Vec2,
    /// 넉백·돌진 속도. 입력 이동과 별도로 감쇠한다.
    pub vel: Vec2,
    pub facing: Vec2,
    pub hp: i32,
    /// 장비가 정한 수치(앱이 장착을 바꾸면 갱신).
    pub stats: Stats,
    /// 필살기 게이지(0..=SP_MAX). 발톱이 맞으면 찬다.
    pub sp: f32,
    pub invuln: f32,
    pub flash: f32,
    pub state: State,
    pub burrow_cd: f32,
    /// 다음 먹기까지 남은 시간(먹기 시작 순간부터 ITEM_COOLDOWN).
    pub item_cd: f32,
    /// 먹기 대기 배율(실뜨기 5, docs/decisions/0043).
    pub eat_cd_mul: f32,
    /// 실뜨기 8·9(0053): 패링 창 배율, 굴파기 대기 배율.
    pub parry_mul: f32,
    pub burrow_knot: f32,
    /// 큰 나무의 재주(docs/decisions/0047). 앱이 기록에서 정한다.
    pub talents: crate::talents::Talents,
    /// 받는 피해를 줄이는 비율(0..1). 도움 모드 "포포의 응원"(docs/decisions/0041). 앱이 기록에서 정한다.
    pub resist: f32,
    /// 줄인 피해의 소수 부분을 모아 둔다(체력은 정수라 1이 차면 그때 깎는다).
    dmg_acc: f32,
    pub events: Vec<Event>,
    /// 이번 틱에 월드에 내놓는 탄·묻은 도토리(도토, 0195). 월드가 비운다.
    pub spawns: Vec<Spawn>,
    /// 드릴 겨누기용 적 자리(월드가 틱마다 채운다, 맞을 수 있는 적만). 시동 중 방향을 맞추는 데만 쓴다(0139).
    pub targets: Vec<Vec2>,
    /// 흰 실 그물(0165)에 걸려 있다: 월드가 틱마다 정한다. 걷는 속도 `WEB_SLOW`배.
    pub slow: bool,
    /// 받아칠 위협이 가까이 있다(0189): 월드가 틱마다 정한다. 그러면 창 전에 눌러 둔 타도 박자를 기다리지 않는다
    /// (박자 스냅이 받아치기를 늦추면 안 된다).
    pub threat: bool,
    /// 시동 중 겨눈 적(그리기용 표시). 없으면 입력 방향 그대로.
    pub drill_lock: Option<Vec2>,
    /// 콤보 표. 무기·캐릭터에 따라 바뀐다(moveset.rs).
    pub moveset: &'static Moveset,
    /// 스프라이트 좌우(마지막으로 본 가로 방향).
    pub face_left: bool,
    /// 어떤 캐릭터의 몸인가(회피·특기·그림, 0056).
    pub kit: crate::equipment::Character,
    /// 무기를 들었나. 첫 시작에는 맨손이라 공격이 안 나간다(docs/decisions/0012).
    pub armed: bool,
    attack_buf: f32,
    dodge_buf: f32,
    item_buf: f32,
    /// 다음 패링 창이 열릴 수 있기까지. 지금 스윙에 창이 있나는 `parry_open`.
    parry_cd: f32,
    parry_open: bool,
    combo_next: usize,
    combo_timer: f32,
    no_attack: f32,
    /// 박자 콤보(0167 → 0174): 콤보 중(스윙·강공격·콤보 창)에 눌러 둔 다음 타. 먼저 누른 것이 남는다.
    queued: Option<Queued>,
    /// 지금 스윙(평타·강타)을 시작하고 흐른 시간(공격 속도 배율로 센다). 다음 타의 박자는 `beat_at`.
    beat_clock: f32,
    beat_at: f32,
    /// 지금 스윙이 박자에 맞아 나갔나(그림·소리, 맞히면 멈춤·킥 덤 0168).
    pub on_beat: bool,
    /// 이번 콤보에서 박자에 맞춘 이음 수(0168): 모두 맞추면 마무리 타 넉백 `FULL_BEAT_KNOCK`배.
    combo_beats: usize,
    /// 연달아 박자에 맞춘 타 수(0174, 머리 위 음표·오름음). 박자를 놓치면 0.
    beat_streak: usize,
    /// 음표가 남는 시간.
    beat_show: f32,
    swing_id: u32,
    /// 지금 스윙을 시작하고 흐른 실제 시간(패링 창).
    swing_age: f32,
    /// 공격 버튼을 이어서 쥔 시간(0255). 떼면 0. `swing_age`보다 길면 지금 스윙을 낸 누름(또는 그 전 누름)을 아직 쥐고 있다.
    hold: f32,
}

/// 한 틱의 바깥 상황(0170 R9): 상태 함수들이 같은 것을 본다.
pub(super) struct Ctx<'a> {
    pub a: &'a Actions,
    pub dt: f32,
    pub move_dir: Vec2,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
            facing: vec2(1.0, 0.0),
            hp: t::PLAYER_HP,
            stats: Stats::BASE,
            sp: 0.0,
            invuln: 0.0,
            flash: 0.0,
            state: State::Free,
            burrow_cd: 0.0,
            item_cd: 0.0,
            eat_cd_mul: 1.0,
            parry_mul: 1.0,
            burrow_knot: 1.0,
            talents: crate::talents::Talents::default(),
            resist: 0.0,
            dmg_acc: 0.0,
            events: Vec::new(),
            spawns: Vec::new(),
            targets: Vec::new(),
            slow: false,
            threat: false,
            drill_lock: None,
            moveset: &moveset::CLAWS,
            face_left: false,
            kit: crate::equipment::Character::Mole,
            armed: true,
            attack_buf: 0.0,
            dodge_buf: 0.0,
            item_buf: 0.0,
            parry_cd: 0.0,
            parry_open: false,
            combo_next: 0,
            combo_timer: 0.0,
            no_attack: 0.0,
            queued: None,
            beat_clock: 0.0,
            beat_at: t::BEAT_PERIOD,
            on_beat: false,
            combo_beats: 0,
            beat_streak: 0,
            beat_show: 0.0,
            swing_id: 0,
            swing_age: f32::MAX,
            hold: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        t::PLAYER_RADIUS
    }

    /// 땅 위를 걷고 있다(발걸음 먼지, 0081): 굴 속·공중·드릴 돌진이 아니다.
    /// 걷는 속도: 흰 실 그물에 걸렸으면 느리다(0165).
    /// 큰 나무 "가벼운 발"(0246)이 곱해진다.
    fn speed(&self) -> f32 {
        let base = t::PLAYER_SPEED * self.stats.speed;
        if self.slow {
            base * t::WEB_SLOW
        } else {
            base
        }
    }

    pub fn on_foot(&self) -> bool {
        matches!(
            self.state,
            State::Free
                | State::Attack { .. }
                | State::Charging { .. }
                | State::Heavy { .. }
                | State::Run { .. }
        )
    }

    pub fn is_alive(&self) -> bool {
        self.state != State::Dead
    }

    pub fn is_burrowed(&self) -> bool {
        matches!(self.state, State::Burrow { .. })
    }

    /// 땅속에서 모은 힘 0~1(0158): 정한 시간을 지나 머문 만큼 BURROW_HOLD_EXTRA에 걸쳐 찬다.
    /// 솟구치는 동안은 모았던 값. 그 밖에는 0.
    pub fn burrow_charge(&self) -> f32 {
        match self.state {
            State::Burrow { time } => Self::charge_of(time - self.talents.burrow_time()),
            State::Emerge { charge, .. } => charge,
            _ => 0.0,
        }
    }

    /// 정한 시간을 넘긴 첫 틱(톡 눌렀을 때 자동으로 솟구치는 틱)은 세지 않는다: 톡은 힘 0.
    fn charge_of(held: f32) -> f32 {
        ((held - t::TICK_DT) / (t::BURROW_HOLD_EXTRA - t::TICK_DT)).clamp(0.0, 1.0)
    }

    /// 솟구침 반지름: 재주 값 × 모은 힘만큼(0158). 이펙트·밀쳐내기도 같은 값.
    pub fn emerge_radius_for(&self, charge: f32) -> f32 {
        self.talents.emerge_radius()
            * self.stats.emerge
            * (1.0 + (t::EMERGE_CHARGE_RADIUS - 1.0) * charge)
    }

    pub fn is_invulnerable(&self) -> bool {
        cfg!(feature = "clip")
            || self.invuln > 0.0
            || self.is_burrowed()
            || matches!(
                self.state,
                State::Drill { .. }
                    | State::Revive { .. }
                    | State::Hop { .. }
                    | State::Leap { .. }
                    | State::Chain { .. }
                    | State::Jump { .. }
            )
            || !self.is_alive()
    }

    /// 뛰는 높이(그리기용 px, 바닥 = 0).
    pub fn height(&self) -> f32 {
        let (time, dur, h) = match self.state {
            State::Hop { time, .. } => (time, t::HOP_TIME, t::HOP_HEIGHT),
            State::Leap { time, .. } => (time, t::BACKLEAP_TIME, t::BACKLEAP_HEIGHT),
            State::Chain { time, .. } => (time, t::CHAIN_HOP_TIME, t::CHAIN_HOP_HEIGHT),
            State::Jump { time, .. } => (time, t::JUMP_TIME, t::JUMP_HEIGHT),
            _ => return 0.0,
        };
        let k = (time / dur).clamp(0.0, 1.0);
        4.0 * h * k * (1.0 - k)
    }

    /// 회피 시작(캐릭터마다): 굴파기 / 깡충 / 뒤로 뛰기.
    fn start_dodge(&mut self, move_dir: Vec2) {
        match self.kit.def().dodge {
            crate::characters::Dodge::Hop => self.start_hop(move_dir),
            crate::characters::Dodge::Burrow => self.start_burrow(),
            crate::characters::Dodge::Leap => self.start_leap(move_dir),
        }
    }

    /// 특기 시작(캐릭터마다): 드릴 / 이어 뛰기 / 도토리 묻기.
    fn start_special(&mut self, move_dir: Vec2) {
        match self.kit.def().special {
            crate::characters::Special::Chain => self.start_chain(move_dir),
            crate::characters::Special::Drill => self.start_drill(move_dir),
            crate::characters::Special::Bury => self.start_bury(move_dir),
        }
    }

    /// 콩이의 등딱지에 발톱이 튕겼다(0054): 휘두르기가 끊기고 뒤로 밀려나며, 잠깐 다시 못 휘두른다. 피해는 없다.
    pub fn recoil(&mut self, from: Vec2, push: f32, lock: f32) {
        if matches!(self.state, State::Attack { .. }) {
            self.state = State::Free;
        }
        self.combo_timer = 0.0;
        self.attack_buf = 0.0;
        self.no_attack = self.no_attack.max(lock);
        let away = (self.pos - from).normalize_or(-self.facing);
        self.vel = away * push;
    }

    /// 굴파기 대기 한 번의 길이(장신구 × 재주 × 실뜨기). 상태 탭·HUD 막대도 이 값을 쓴다.
    pub fn burrow_cd_full(&self) -> f32 {
        t::BURROW_COOLDOWN * self.stats.burrow_cd * self.talents.burrow_cd_mul() * self.burrow_knot
    }

    /// 패링 창이 열려 있으면 공격 방향. 스윙 시작 직후 PARRY_WINDOW 동안(실뜨기 8단계면 짧다).
    /// 테스트용: 이 방향으로 받아치기 창을 연다(월드 테스트가 받아치기 길을 밟아 본다).
    #[cfg(test)]
    pub fn open_parry_for_test(&mut self, dir: Vec2) {
        self.state = State::Attack {
            idx: 0,
            time: 0.0,
            dir,
        };
        self.facing = dir;
        self.parry_open = true;
        self.swing_age = 0.0;
    }

    pub fn parry_dir(&self) -> Option<Vec2> {
        match self.state {
            State::Attack { dir, .. } | State::Heavy { dir, .. }
                if self.parry_open && self.swing_age < t::PARRY_WINDOW * self.parry_mul =>
            {
                Some(dir)
            }
            _ => None,
        }
    }

    /// 강공격의 받아치기 창(0161): 이 동안은 둘레 공격(내려앉기·튀어나오기)도 받아친다.
    pub fn parry_heavy(&self) -> bool {
        matches!(self.state, State::Heavy { .. }) && self.parry_dir().is_some()
    }

    /// 패링 성공: SP를 크게 채운다. 다음 창은 바로 열 수 있다.
    pub fn on_parry(&mut self) {
        self.sp = (self.sp + self.talents.parry_sp()).min(t::SP_MAX);
        self.parry_cd = 0.0;
        self.events.push(Event::Parry);
    }

    pub fn sp_full(&self) -> bool {
        self.sp >= t::SP_MAX
    }

    /// 공격 버튼의 대기(터치, 0..1, 1 = 바로 칠 수 있음): 휘두르는 동안(스윙 + 후딜레이)·강공격·후딜레이에 차오른다.
    pub fn attack_ready(&self) -> f32 {
        let k = match self.state {
            State::Attack { idx, time, .. } => {
                let h = self.moveset.hits[idx];
                time / (h.swing + h.recovery).max(0.01)
            }
            State::Heavy { time, .. } => {
                let h = self.moveset.heavy();
                time / (h.swing + h.recovery).max(0.01)
            }
            State::Recover { .. } => 0.0,
            _ => 1.0,
        };
        k.clamp(0.0, 1.0)
    }

    /// 박자 창(±초). 무기마다 배율(꽃잎 발목끈은 넓다), 큰 나무 "박자 귀"(0246).
    pub fn beat_window(&self) -> f32 {
        t::BEAT_WINDOW * self.moveset.beat_window * self.stats.beat
    }

    /// 강공격을 다 모으는 시간(장신구·큰 나무로 짧아진다, 0246).
    pub fn charge_time(&self) -> f32 {
        t::HEAVY_CHARGE * self.stats.charge
    }

    /// 모으는 자세로 들어갈 때인가(0255): 지금 스윙을 낸 누름을 `HEAVY_HOLD_START` 넘게 쥐고 있다. 들어설 때의 모으기 시계
    /// (스윙 시작부터 `HEAVY_CHARGE_LEAD` 뒤부터 센다 — 다 모이는 때가 스윙 길이와 상관없다)를 돌려준다.
    /// 스윙·후딜레이가 끝난 뒤에만 부른다 — 모으다 떼면 바로 다음 타가 나가니, 다음 타를 늦추는 것은 무기의 공격 속도뿐이다.
    pub(super) fn charge_from_hold(&self, a: &Actions) -> Option<f32> {
        (a.attack_held && self.hold >= self.swing_age && self.swing_age >= t::HEAVY_HOLD_START)
            .then(|| {
                // 스윙이 길어 이미 다 모였으면 다음 틱에 다 모인다(번쩍·`HeavyReady`를 거른다).
                (self.swing_age - t::HEAVY_CHARGE_LEAD).min(self.charge_time() - 1e-3)
            })
    }

    /// 발톱이 적을 맞혔다: SP를 채운다(드릴 자체는 채우지 않는다). 박자에 맞춘 타는 무기의 `beat_sp`배(0246).
    pub fn gain_sp(&mut self, finisher: bool) {
        let mut g = t::SP_PER_HIT;
        if finisher {
            g += t::SP_FINISHER_BONUS;
        }
        if self.on_beat {
            g *= self.moveset.beat_sp * self.stats.beat_sp;
        }
        self.sp = (self.sp + g * self.stats.sp_gain * self.talents.sp_gain_mul()).min(t::SP_MAX);
    }

    /// 장비가 바뀌었다. 최대 체력이 줄면 체력도 따라 줄인다.
    pub fn set_stats(&mut self, s: Stats) {
        self.stats = s;
        self.hp = self.hp.min(s.max_hp);
    }

    pub fn heal(&mut self, n: i32) {
        self.hp = (self.hp + n).min(self.stats.max_hp);
    }

    /// 쓰러졌는데 쥬시 지렁이가 있다(0243): 누워서 먹기 시작. 일어나는 건 `tick_revive`.
    pub fn start_revive(&mut self) {
        self.state = State::Revive { time: 0.0 };
        self.vel *= 0.5;
        self.combo_timer = 0.0;
        self.queued = None;
    }

    fn tick_revive(&mut self, dt: f32, time: f32) {
        let time = time + dt;
        self.vel *= 0.85;
        if time >= t::REVIVE_DOWN_TIME {
            self.revive();
            self.events.push(Event::Rose);
        } else {
            self.state = State::Revive { time };
        }
    }

    /// 누워 있던 비율 0..1(그림: 지렁이가 입으로 들어가고 빛이 모인다). 일어나기 전이 아니면 None.
    pub fn revive_progress(&self) -> Option<f32> {
        match self.state {
            State::Revive { time } => Some((time / t::REVIVE_DOWN_TIME).clamp(0.0, 1.0)),
            _ => None,
        }
    }

    /// 쓰러진 자리에서 일어난다(쥬시 지렁이로 누웠다 일어날 때, 쉼터에서 깰 때).
    pub fn revive(&mut self) {
        let hp = (self.stats.max_hp as f32 * t::REVIVE_HP_FRAC).ceil() as i32;
        self.hp = hp.clamp(1, self.stats.max_hp);
        self.state = State::Free;
        self.invuln = t::REVIVE_INVULN;
        // 지렁이를 먹고 일어났다 = 먹기. 바로 또 먹지 못한다.
        self.item_cd = t::ITEM_COOLDOWN;
        self.vel = Vec2::ZERO;
        self.combo_timer = 0.0;
    }

    /// 콤보가 이어지는 중인가(0174): 스윙·강공격 중이거나, 서서 콤보 창이 열려 있거나(눌러 둔 타가 있으면 창이 닫혀도).
    fn combo_pending(&self) -> bool {
        match self.state {
            State::Attack { .. } | State::Heavy { .. } => true,
            State::Free => self.combo_timer > 0.0 || self.queued.is_some(),
            _ => false,
        }
    }

    /// 박자 고리(0174): 콤보 중일 때만. 박자가 지나고 한 창만큼 더 보이고 사라진다.
    pub fn beat_cue(&self) -> Option<BeatCue> {
        if !self.armed || !self.combo_pending() {
            return None;
        }
        let d = self.beat_clock - self.beat_at;
        if d > self.beat_window() * 2.0 {
            return None;
        }
        Some(BeatCue {
            k: self.beat_clock / self.beat_at,
            window: d.abs() <= self.beat_window(),
            waiting: self.queued == Some(Queued::Early),
        })
    }

    /// 지금 콤보의 박자 간격(초, 공격 속도 배율을 넣은 시계 기준).
    pub fn beat_period(&self) -> f32 {
        self.beat_at
    }

    /// 연달아 맞춘 박자 수와 그 표시가 남은 시간(0174, 머리 위 음표).
    pub fn beat_pips(&self) -> (usize, f32) {
        (self.beat_streak, self.beat_show)
    }

    /// 히트스톱 중에도 호출된다: 눌림을 버퍼에 넣기만 한다.
    pub fn buffer(&mut self, a: &Actions) {
        if a.attack {
            // 박자 콤보(0174): 콤보 중의 눌림은 버퍼가 아니라 `queued`에 — 박자 창 전이면 `Early`(박자까지 기다린다),
            // 창 안이면 `Beat`. 창 뒤의 눌림은 보통 버퍼(그 자리에서 나간다, 덤 없음).
            if self.combo_pending() {
                if self.queued.is_none() {
                    let d = self.beat_clock - self.beat_at;
                    if d < -self.beat_window() {
                        self.queued = Some(Queued::Early);
                    } else if d <= self.beat_window() {
                        self.queued = Some(Queued::Beat);
                    } else {
                        self.attack_buf = t::INPUT_BUFFER;
                    }
                }
            } else {
                self.attack_buf = t::INPUT_BUFFER;
            }
        }
        if a.dodge {
            self.dodge_buf = t::INPUT_BUFFER;
        }
        if a.use_item {
            self.item_buf = t::INPUT_BUFFER;
        }
    }

    /// 먹기를 눌렀고(버퍼) 지금 먹기 시작할 수 있나. 무엇을 먹을지·먹을 필요가 있는지는 월드가 본다.
    pub fn wants_to_eat(&self) -> bool {
        self.item_buf > 0.0 && self.item_cd <= 0.0 && self.state == State::Free
    }

    /// 먹기 대기 중(먹은 지 얼마 안 됨).
    pub fn item_cooling(&self) -> bool {
        self.item_cd > 0.0
    }

    /// 필살기를 눌렀는데 낼 수 없다(게이지가 덜 찼거나 무기가 없다, 0149). 서 있거나 휘두르는 중일 때만 본다
    /// (땅속·드릴·먹기 중에 누른 것은 원래 받지 않는 입력이다).
    pub fn special_denied(&self, a: &Actions) -> bool {
        a.special
            && matches!(self.state, State::Free | State::Attack { .. })
            && !(self.armed && self.sp_full())
    }

    /// 먹기 버튼을 눌렀지만 먹지 않았다(체력이 가득 등): 버퍼를 비워 알림이 프레임마다 다시 뜨지 않게.
    pub fn drop_item_press(&mut self) {
        self.item_buf = 0.0;
    }

    /// 먹기 시작. 대기는 지금부터 센다(끊겨도 그대로).
    pub fn start_eat(&mut self, item: Consumable) {
        self.item_buf = 0.0;
        self.item_cd = t::ITEM_COOLDOWN * self.eat_cd_mul;
        self.state = State::Eat { item, time: 0.0 };
    }

    /// 먹는 중이면 얼마나 먹었나(0..1).
    pub fn eat_progress(&self) -> Option<(Consumable, f32)> {
        match self.state {
            State::Eat { item, time } => Some((
                item,
                (time / (item.eat_time() * self.talents.eat_time_mul())).min(1.0),
            )),
            _ => None,
        }
    }

    /// 한 로직 틱. 이번 틱에 활성인 공격 히트박스를 돌려준다.
    /// 자유: 특기·회피·평타·먹기 시작, 이동.
    pub(super) fn tick_free(&mut self, c: &Ctx<'_>) -> Option<Strike> {
        let a = c.a;
        let dt = c.dt;
        let move_dir = c.move_dir;
        if a.special && self.armed && self.sp_full() {
            self.start_special(move_dir);
        } else if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
            self.start_dodge(move_dir);
        } else if self.armed && self.no_attack <= 0.0 && (self.attack_buf > 0.0 || self.queue_due())
        {
            // 누르는 순간 평타(0167). 콤보 중에 눌러 둔 타(0174)는 박자 창 안이면 바로(덤), 창 전이면 박자에 나간다.
            // 누르고 있으면 활성 구간 뒤에 모으기로 간다.
            let beat = self.queued == Some(Queued::Beat);
            let idx = if self.combo_timer > 0.0 || self.queued.is_some() {
                self.combo_next
            } else {
                0
            };
            self.start_swing(idx, a.aim, beat);
        } else if let Some(time) = self
            .charge_from_hold(a)
            .filter(|_| self.armed && self.no_attack <= 0.0 && self.queued.is_none())
        {
            // 스윙이 끝난 뒤에도 그 누름을 쥐고 있다(짧은 스윙): 모으기로(0255). 다음 자리는 콤보가 이어지면 그 타.
            let after = if self.combo_timer > 0.0 {
                self.combo_next
            } else {
                0
            };
            self.start_charging(time, self.facing, after);
        } else {
            self.pos += move_dir * self.speed() * dt;
        }
        None
    }

    /// 눌러 둔 다음 타가 지금 나갈 때인가(0174): 박자 창 안에서 누른 것은 바로, 창 전에 누른 것은 박자에.
    /// 받아칠 위협이 가까우면 창 전에 누른 것도 바로(0189, 덤은 없다).
    pub(super) fn queue_due(&self) -> bool {
        match self.queued {
            Some(Queued::Beat) => true,
            Some(Queued::Early) => self.threat || self.beat_clock >= self.beat_at,
            None => false,
        }
    }

    /// 먹기: 시간이 차면 효과, 맞으면 끊긴다.
    pub(super) fn tick_eat(&mut self, c: &Ctx<'_>, item: Consumable, time: f32) -> Option<Strike> {
        let dt = c.dt;
        let move_dir = c.move_dir;
        let time = time + dt;
        self.pos += move_dir * self.speed() * t::EAT_MOVE_FACTOR * dt;
        if self.dodge_buf > 0.0 && self.burrow_cd <= 0.0 {
            // 굴파기(깡충)로 끊는다: 먹지 않은 것으로. 대기는 이미 흐르고 있다.
            self.start_dodge(move_dir);
        } else if time >= item.eat_time() * self.talents.eat_time_mul() {
            self.events.push(Event::Ate(item));
            self.state = State::Free;
        } else {
            self.state = State::Eat { item, time };
        }
        None
    }

    pub fn update(&mut self, a: &Actions, room: &Room, dt: f32) -> Option<Strike> {
        self.events.clear();
        self.spawns.clear();
        self.prev_pos = self.pos;
        self.buffer(a);

        self.invuln = (self.invuln - dt).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        self.combo_timer = (self.combo_timer - dt).max(0.0);
        self.burrow_cd = (self.burrow_cd - dt).max(0.0);
        self.item_cd = (self.item_cd - dt).max(0.0);
        self.parry_cd = (self.parry_cd - dt).max(0.0);
        self.no_attack = (self.no_attack - dt).max(0.0);
        self.swing_age += dt;
        self.hold = if a.attack_held { self.hold + dt } else { 0.0 };
        // 박자를 지나는 틱인가(0180): 콤보 중일 때만. 타가 나가 시계가 되감기면 그 소리가 박자다.
        let before = self.beat_clock;
        self.beat_clock = (self.beat_clock + dt * self.stats.attack_speed).min(60.0);
        let beat_now = self.armed
            && self.combo_pending()
            && before < self.beat_at
            && self.beat_clock >= self.beat_at;
        self.beat_show = (self.beat_show - dt).max(0.0);
        if !matches!(
            self.state,
            State::Attack { .. } | State::Heavy { .. } | State::Free
        ) {
            self.queued = None;
        }

        let move_dir = a.move_dir;
        if move_dir != Vec2::ZERO
            && !matches!(self.state, State::Attack { .. } | State::Heavy { .. })
        {
            self.facing = move_dir.normalize();
        }

        let mut strike = None;
        let c = Ctx { a, dt, move_dir };
        match self.state {
            State::Dead => {}
            State::Revive { time } => self.tick_revive(dt, time),
            State::Free => strike = self.tick_free(&c),
            State::Attack { idx, time, dir } => strike = self.tick_attack(&c, idx, time, dir),
            State::Charging { time, dir, after } => {
                strike = self.tick_charging(&c, time, dir, after)
            }
            State::Heavy {
                time,
                dir,
                after,
                perfect,
            } => strike = self.tick_heavy(&c, time, dir, after, perfect),
            State::Burrow { time: was } => strike = self.tick_burrow(&c, was),
            State::Hop { time, dir } => strike = self.tick_hop(&c, time, dir),
            State::Leap { time, dir } => strike = self.tick_leap(&c, time, dir),
            State::Chain { left, time, dir } => strike = self.tick_chain(&c, left, time, dir),
            State::Run { time } => strike = self.tick_run(&c, time),
            State::Jump {
                time,
                dir,
                dist,
                thrown,
            } => strike = self.tick_jump(&c, time, dir, dist, thrown),
            State::Emerge { time, charge } => strike = self.tick_emerge(&c, time, charge),
            State::Drill {
                time,
                dir,
                aim,
                next_hit,
            } => strike = self.tick_drill(&c, time, dir, aim, next_hit),
            State::Recover { time } => strike = self.tick_recover(&c, time),
            State::Eat { item, time } => strike = self.tick_eat(&c, item, time),
        }

        if beat_now && !self.events.iter().any(|e| matches!(e, Event::Swing(_))) {
            self.events.push(Event::BeatTick);
        }

        self.attack_buf = (self.attack_buf - dt).max(0.0);
        self.dodge_buf = (self.dodge_buf - dt).max(0.0);
        self.item_buf = (self.item_buf - dt).max(0.0);

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        let bumped = room.resolve_circle(&mut self.pos, r);
        // 드릴이 벽(기둥)에 부딪히면 쿵: 충격파 + 경직. 시작하자마자 벽에 붙어 있던 건 치지 않는다.
        if let State::Drill { time, dir, .. } = self.state {
            if bumped && time > t::DRILL_REV + 0.06 {
                self.swing_id += 1;
                self.vel = -dir * 80.0;
                self.state = State::Recover {
                    time: t::DRILL_WALL_RECOVER,
                };
                self.events.push(Event::DrillWall);
                strike = Some(Strike {
                    id: self.swing_id,
                    kind: StrikeKind::DrillWall,
                    shape: Shape::Circle {
                        center: self.pos + dir * 4.0,
                        r: t::DRILL_WALL_RADIUS,
                    },
                    damage: t::DRILL_WALL_DAMAGE + self.stats.damage,
                    knockback: t::DRILL_WALL_KNOCKBACK,
                    stun: t::DRILL_WALL_STUN,
                    hitstop: t::DRILL_WALL_HITSTOP,
                    shake: t::DRILL_WALL_SHAKE,
                    finisher: false,
                    poison: 0.0,
                    crush: false,
                });
            }
        }
        if self.facing.x.abs() > 0.2 {
            self.face_left = self.facing.x < 0.0;
        }
        strike
    }

    /// 피해를 입힌다. 무적이면 false.
    pub fn take_damage(&mut self, from: Vec2, dmg: i32) -> bool {
        if self.is_invulnerable() {
            return false;
        }
        // 모으다 맞으면 끊긴다(0161).
        if matches!(self.state, State::Charging { .. }) {
            self.state = State::Free;
        }
        // 도움 모드: 피해를 비율만큼 줄인다. 소수는 모아 두었다가 1이 되면 깎는다(맞은 반응은 그대로).
        let dmg = if self.resist > 0.0 {
            self.dmg_acc += dmg as f32 * (1.0 - self.resist.min(0.95));
            let d = self.dmg_acc.floor();
            self.dmg_acc -= d;
            d as i32
        } else {
            dmg
        };
        self.hp -= dmg;
        self.invuln = t::PLAYER_HURT_INVULN;
        self.flash = t::HIT_FLASH;
        let away = (self.pos - from).normalize_or_zero();
        let away = if away == Vec2::ZERO {
            -self.facing
        } else {
            away
        };
        self.vel = away * t::PLAYER_HURT_KNOCKBACK;
        if self.hp <= 0 {
            self.hp = 0;
            self.state = State::Dead;
            self.events.push(Event::Die);
        } else {
            // 공격·솟구침·먹기 중이었으면 끊긴다(먹던 것은 먹지 않은 것으로).
            self.state = State::Free;
            self.combo_timer = 0.0;
            self.events.push(Event::Hurt);
        }
        true
    }
}

/// 돌진 속도: 시동 뒤 `dash_t`초. 처음엔 붙고(DRILL_ACCEL), 그 뒤 최고 속도.
/// 드릴 겨누기(0139): `aim`에서 DRILL_ASSIST_CONE 안, DRILL_ASSIST_RANGE 안의 적 가운데 방향이 가장 가까운 적.
/// 입력이 8방향(키보드·패드·터치)뿐이라 그 사이 각도의 적을 겨눌 수 없었고, 돌진 중에는 반지름 ~170px로만 틀 수 있어
/// 빗나간 걸 알아도 고치지 못했다. 고르는 것은 시동 동안뿐이다(돌진 중 방향을 바꿔 주면 예측이 안 된다).
pub fn drill_assist(pos: Vec2, aim: Vec2, targets: &[Vec2]) -> Option<Vec2> {
    let mut best: Option<(f32, Vec2)> = None;
    for &q in targets {
        let d = q - pos;
        let dist = d.length();
        if dist < 1.0 || dist > t::DRILL_ASSIST_RANGE {
            continue;
        }
        let ang = aim.angle_between(d).abs();
        if ang <= t::DRILL_ASSIST_CONE && best.is_none_or(|(b, _)| ang < b) {
            best = Some((ang, q));
        }
    }
    best.map(|(_, q)| q)
}

pub fn drill_speed(dash_t: f32) -> f32 {
    let k = (dash_t / t::DRILL_ACCEL).clamp(0.0, 1.0);
    t::DRILL_SPEED_START + (t::DRILL_SPEED - t::DRILL_SPEED_START) * k
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(p: &mut Player, room: &Room, a: &Actions) -> (Option<Strike>, Vec<Event>) {
        let s = p.update(a, room, t::TICK_DT);
        (s, p.events.clone())
    }

    /// 모모(0056): 회피 버튼 = 깡충(공중 무적, HOP_DIST만큼), 특기 = 이어 뛰기(기 게이지, 착지 셋 = 쿵 셋).
    #[test]
    fn momo_hops_and_chain_hops() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.kit = crate::equipment::Character::Momo;
        p.moveset = &moveset::KICKS;
        let x0 = p.pos.x;
        let hop = Actions {
            move_dir: vec2(1.0, 0.0),
            dodge: true,
            ..Default::default()
        };
        let none = Actions::default();
        tick(&mut p, &room, &hop);
        assert!(matches!(p.state, State::Hop { .. }) && p.is_invulnerable());
        assert!(!p.is_burrowed(), "땅속이 아니다(적이 계속 노린다)");
        let mut landed = false;
        for _ in 0..40 {
            let (_, ev) = tick(&mut p, &room, &none);
            landed |= ev.contains(&Event::Land(false));
        }
        assert!(landed && p.state == State::Free);
        assert!((p.pos.x - x0 - t::HOP_DIST).abs() < 3.0, "{}", p.pos.x - x0);
        assert!(p.burrow_cd > 0.0, "깡충도 대기가 있다");
        // 이어 뛰기: 게이지가 차야 나가고, 착지마다 쿵.
        p.sp = t::SP_MAX;
        let go = Actions {
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        assert!(matches!(p.state, State::Chain { .. }) && p.sp == 0.0);
        let mut stomps = 0;
        for _ in 0..120 {
            if let (Some(s), _) = tick(&mut p, &room, &none) {
                assert_eq!(s.kind, StrikeKind::Stomp);
                stomps += 1;
            }
        }
        assert_eq!(stomps, t::CHAIN_HOPS as usize);
        assert_eq!(p.state, State::Free);
    }

    /// 모모 재주(0142): 발 구르기는 깡충 착지에 작은 밟기, 네 번 뛰기는 밟기 넷, 높이 뛰기는 둘이고 넓다.
    #[test]
    fn momo_talents_change_hop_and_chain() {
        let room = Room::new();
        let none = Actions::default();
        let momo = |pick: [u8; 4]| {
            let mut p = Player::new(room.center());
            p.kit = crate::equipment::Character::Momo;
            p.moveset = &moveset::KICKS;
            p.talents = crate::talents::Talents {
                pick,
                who: crate::equipment::Character::Momo,
                ..Default::default()
            };
            p
        };
        // 발 구르기: 착지 한 번에 밟기 하나(반지름 TAL_STAMP_RADIUS).
        let mut p = momo([2, 0, 0, 0]);
        let hop = Actions {
            dodge: true,
            ..Default::default()
        };
        tick(&mut p, &room, &hop);
        let mut stamps = Vec::new();
        for _ in 0..40 {
            if let (Some(s), _) = tick(&mut p, &room, &none) {
                stamps.push(s);
            }
        }
        assert_eq!(stamps.len(), 1);
        assert!(matches!(stamps[0].shape, Shape::Circle { r, .. } if r == t::TAL_STAMP_RADIUS));
        // 이어 뛰기: 네 번 뛰기 = 넷, 높이 뛰기 = 둘(넓게).
        for (side, hops, radius) in [
            (1, t::TAL_FOUR_HOPS, t::CHAIN_RADIUS),
            (2, t::TAL_HIGH_HOPS, t::TAL_HIGH_RADIUS),
        ] {
            let mut p = momo([0, 0, side, 0]);
            p.sp = t::SP_MAX;
            let go = Actions {
                special: true,
                ..Default::default()
            };
            tick(&mut p, &room, &go);
            let mut n = 0;
            for _ in 0..160 {
                if let (Some(s), _) = tick(&mut p, &room, &none) {
                    assert!(matches!(s.shape, Shape::Circle { r, .. } if r == radius));
                    n += 1;
                }
            }
            assert_eq!(n, hops as usize, "갈래 {side}");
        }
    }

    /// 드릴 겨누기(0139): 입력 둘레 콘 안·거리 안의 적 가운데 방향이 가장 가까운 적. 콘 밖·멀리·없으면 입력 그대로.
    #[test]
    fn drill_assist_picks_the_enemy_closest_to_the_aim() {
        let o = vec2(100.0, 100.0);
        let right = vec2(1.0, 0.0);
        // 오른쪽 30°(8방향 사이) 적은 잡고, 50°는 콘 밖.
        let at = |deg: f32, d: f32| o + Vec2::from_angle(deg.to_radians()) * d;
        assert_eq!(
            drill_assist(o, right, &[at(30.0, 100.0)]),
            Some(at(30.0, 100.0))
        );
        assert_eq!(drill_assist(o, right, &[at(50.0, 100.0)]), None);
        // 둘이면 방향이 더 가까운 쪽(거리는 보지 않는다).
        assert_eq!(
            drill_assist(o, right, &[at(25.0, 60.0), at(-10.0, 200.0)]),
            Some(at(-10.0, 200.0))
        );
        // 너무 멀면 안 잡는다.
        assert_eq!(
            drill_assist(o, right, &[at(0.0, t::DRILL_ASSIST_RANGE + 10.0)]),
            None
        );
        assert_eq!(drill_assist(o, right, &[]), None);
    }

    /// 겨누기가 실제 돌진 방향이 된다: 오른쪽만 누르고 있어도 오른쪽 위 30°의 적 쪽으로 나간다.
    #[test]
    fn drill_launches_toward_the_assisted_enemy() {
        let room = Room::new();
        let start = room.center() + vec2(-60.0, 40.0);
        let mut p = Player::new(start);
        let foe = start + Vec2::from_angle((-30f32).to_radians()) * 110.0;
        p.targets = vec![foe];
        p.sp = t::SP_MAX;
        let go = Actions {
            move_dir: vec2(1.0, 0.0),
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        for _ in 0..((t::DRILL_REV / t::TICK_DT) as usize - 1) {
            tick(&mut p, &room, &hold);
        }
        assert_eq!(p.drill_lock, Some(foe), "시동 중 적을 겨눈다");
        let State::Drill { dir, aim, .. } = p.state else {
            panic!("드릴 중이 아니다")
        };
        assert!((aim - vec2(1.0, 0.0)).length() < 1e-4, "입력은 오른쪽");
        assert!(
            dir.angle_between(foe - start).abs() < 0.01,
            "실제 방향은 적 쪽"
        );
        // 돌진에 들어가면 겨눈 표시는 사라진다.
        for _ in 0..3 {
            tick(&mut p, &room, &hold);
        }
        assert_eq!(p.drill_lock, None);
    }

    /// 필살기: SP가 차야 나가고, 벽에 부딪히면 충격파(DrillWall) + 경직.
    #[test]
    fn drill_needs_full_sp_and_bonks_walls() {
        let room = Room::new();
        let mut p = Player::new(vec2(room.inner.right() - 120.0, room.center().y));
        let go = Actions {
            move_dir: vec2(1.0, 0.0),
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        assert!(
            !matches!(p.state, State::Drill { .. }),
            "SP가 없으면 안 나간다"
        );
        p.sp = t::SP_MAX;
        tick(&mut p, &room, &go);
        assert!(matches!(p.state, State::Drill { .. }));
        assert_eq!(p.sp, 0.0);
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        let mut bonk = false;
        let mut hits = 0;
        let x0 = p.pos.x;
        // 시동 동안은 제자리.
        for _ in 0..((t::DRILL_REV / t::TICK_DT) as usize - 1) {
            tick(&mut p, &room, &hold);
            assert!(p.drill_revving());
        }
        assert!((p.pos.x - x0).abs() < 0.01, "시동 중엔 움직이지 않는다");
        for _ in 0..120 {
            let (s, ev) = tick(&mut p, &room, &hold);
            if let Some(s) = s {
                match s.kind {
                    StrikeKind::Drill => hits += 1,
                    StrikeKind::DrillWall => {
                        assert!(s.damage >= t::DRILL_WALL_DAMAGE);
                        bonk = true;
                    }
                    _ => {}
                }
            }
            if ev.contains(&Event::DrillWall) {
                assert!(matches!(p.state, State::Recover { .. }));
            }
        }
        assert!(bonk, "벽 쿵이 나야 한다");
        assert!(hits >= 3, "벽까지 가는 동안 연타");
    }

    /// 직진성: 반대쪽을 눌러도 한 틱에 조금만 돈다.
    #[test]
    fn drill_steers_only_a_little() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.sp = t::SP_MAX;
        let go = Actions {
            move_dir: vec2(1.0, 0.0),
            special: true,
            ..Default::default()
        };
        tick(&mut p, &room, &go);
        let hold = Actions {
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        while p.drill_revving() {
            tick(&mut p, &room, &hold);
        }
        let back = Actions {
            move_dir: vec2(0.0, 1.0),
            ..Default::default()
        };
        let before = match p.state {
            State::Drill { dir, .. } => dir,
            _ => panic!("드릴 중이어야 한다"),
        };
        assert!(before.angle_between(vec2(1.0, 0.0)).abs() < 1e-3);
        tick(&mut p, &room, &back);
        if let State::Drill { dir, .. } = p.state {
            let turned = vec2(1.0, 0.0).angle_between(dir).abs();
            assert!(turned > 0.0 && turned <= t::DRILL_TURN * t::TICK_DT + 1e-4);
        } else {
            panic!("드릴 중이어야 한다");
        }
    }

    /// 돌진 거리는 화면 한 폭 남짓(320px ± 10%).
    #[test]
    fn drill_travels_about_a_screen() {
        let n = (t::DRILL_TIME / t::TICK_DT).round() as usize;
        let d: f32 = (0..n)
            .map(|i| drill_speed(i as f32 * t::TICK_DT) * t::TICK_DT)
            .sum();
        assert!((290.0..=355.0).contains(&d), "{d}");
    }

    /// 패링 창: 스윙 시작 직후 PARRY_WINDOW 동안만, 공격 방향으로.
    /// 도움 모드: 피해가 비율만큼 줄고, 소수는 모였다가 깎인다.
    #[test]
    fn assist_resist_spreads_damage_over_hits() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        p.resist = 0.5;
        let start = p.hp;
        for _ in 0..4 {
            p.invuln = 0.0;
            assert!(
                p.take_damage(p.pos + vec2(10.0, 0.0), 1),
                "맞은 반응(넉백·무적)은 그대로"
            );
        }
        assert_eq!(start - p.hp, 2, "1 피해 네 번 × 0.5 = 2");
    }

    /// 연타로 패링 창을 이어 붙이지 못한다: 창은 PARRY_COOLDOWN마다 한 번. 성공하면 바로 다시(0038).
    #[test]
    fn mashing_attack_does_not_chain_parry_windows() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        let atk = Actions {
            attack: true,
            ..Default::default()
        };
        // 1초 동안 매 틱 공격을 누른다: 창이 열려 있던 시간의 비율.
        let ticks = (1.0 / t::TICK_DT) as usize;
        let mut open = 0;
        for _ in 0..ticks {
            tick(&mut p, &room, &atk);
            if p.parry_dir().is_some() {
                open += 1;
            }
        }
        let frac = open as f32 / ticks as f32;
        let most = t::PARRY_WINDOW / t::PARRY_COOLDOWN + 0.05;
        assert!(frac <= most, "연타 창 비율 {frac} > {most}");
        assert!(open > 0, "연타해도 가끔은 걸린다");

        // 성공하면 다음 스윙에 바로 창이 열린다.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &atk);
        assert!(p.parry_dir().is_some());
        p.on_parry();
        // 다음 타의 박자(0174)에 누른다: 그 자리에서 나가고 창이 열린다.
        for _ in 0..((t::BEAT_PERIOD / t::TICK_DT) as usize) {
            tick(&mut p, &room, &Actions::default());
        }
        tick(&mut p, &room, &atk);
        assert!(p.parry_dir().is_some(), "튕겨낸 뒤엔 대기 없이");
    }

    /// 먹기: 먹는 시간이 지나야 Ate, 맞으면 끊기고, 대기 동안은 다시 못 먹는다(0038).
    #[test]
    fn eating_takes_time_breaks_on_hit_and_has_a_cooldown() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        let eat = Actions {
            use_item: true,
            ..Default::default()
        };
        p.buffer(&eat);
        assert!(p.wants_to_eat());
        p.start_eat(Consumable::Berry);
        let need = (Consumable::Berry.eat_time() / t::TICK_DT).ceil() as usize;
        for _ in 0..need - 1 {
            let (_, ev) = tick(&mut p, &room, &Actions::default());
            assert!(
                !ev.contains(&Event::Ate(Consumable::Berry)),
                "먹는 중엔 아직"
            );
        }
        let (_, ev) = tick(&mut p, &room, &Actions::default());
        assert!(ev.contains(&Event::Ate(Consumable::Berry)));
        assert_eq!(p.state, State::Free);
        // 다 먹었어도 대기가 남아 있다.
        p.buffer(&eat);
        assert!(!p.wants_to_eat(), "같은 행동이 바로 다시 나가면 안 된다");
        let wait = (t::ITEM_COOLDOWN / t::TICK_DT).ceil() as usize;
        for _ in 0..wait {
            tick(&mut p, &room, &Actions::default());
        }
        p.buffer(&eat);
        assert!(p.wants_to_eat());

        // 먹다가 맞으면 끊긴다(Ate 없음).
        p.start_eat(Consumable::Worm);
        tick(&mut p, &room, &Actions::default());
        assert!(p.take_damage(p.pos + vec2(10.0, 0.0), 1));
        assert_eq!(p.state, State::Free);
        for _ in 0..((t::WORM_EAT_TIME / t::TICK_DT) as usize + 2) {
            let (_, ev) = tick(&mut p, &room, &Actions::default());
            assert!(!ev.contains(&Event::Ate(Consumable::Worm)));
        }

        // 굴파기로 끊을 수 있다.
        let mut p = Player::new(room.center());
        p.start_eat(Consumable::Berry);
        let dig = Actions {
            dodge: true,
            ..Default::default()
        };
        tick(&mut p, &room, &dig);
        assert!(p.is_burrowed());
    }

    #[test]
    fn parry_window_opens_only_at_swing_start() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        assert!(p.parry_dir().is_none());
        let atk = Actions {
            attack: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        tick(&mut p, &room, &atk);
        assert_eq!(p.parry_dir(), Some(vec2(1.0, 0.0)));
        let ticks = (t::PARRY_WINDOW / t::TICK_DT).ceil() as usize + 1;
        for _ in 0..ticks {
            tick(&mut p, &room, &Actions::default());
        }
        assert!(p.parry_dir().is_none(), "창이 닫혀야 한다");
        let sp = p.sp;
        p.on_parry();
        assert!(p.sp > sp);
    }

    #[test]
    fn attack_speed_shortens_the_swing() {
        let room = Room::new();
        let swing_ticks = |speed: f32| {
            let mut p = Player::new(room.center());
            p.stats.attack_speed = speed;
            let atk = Actions {
                attack: true,
                ..Default::default()
            };
            tick(&mut p, &room, &atk);
            let mut n = 0;
            while matches!(p.state, State::Attack { .. }) && n < 200 {
                tick(&mut p, &room, &Actions::default());
                n += 1;
            }
            n
        };
        assert!(swing_ticks(1.5) < swing_ticks(1.0));
    }

    /// 굴파기(0101): 톡 누르면 정한 시간, 누르고 있으면 더 머물고 떼면 바로 솟구친다. 더 머문 만큼 대기가 는다.
    #[test]
    fn burrow_holds_while_pressed_and_pops_on_release() {
        let room = Room::new();
        let secs = |p: &mut Player, hold_ticks: usize| -> f32 {
            let dig = Actions {
                dodge: true,
                dodge_held: true,
                ..Default::default()
            };
            tick(p, &room, &dig);
            let mut n = 1;
            while matches!(p.state, State::Burrow { .. }) {
                let a = Actions {
                    move_dir: vec2(1.0, 0.0),
                    dodge_held: n < hold_ticks,
                    ..Default::default()
                };
                tick(p, &room, &a);
                n += 1;
            }
            n as f32 * t::TICK_DT
        };
        let tap = {
            let mut p = Player::new(room.center());
            let s = secs(&mut p, 1);
            assert!(
                (p.burrow_cd - p.burrow_cd_full()).abs() < 1e-4,
                "톡: 대기 그대로"
            );
            s
        };
        assert!(
            (tap - t::BURROW_MAX_TIME).abs() < 0.04,
            "톡 = 정한 시간 {tap}"
        );
        let mut p = Player::new(room.center());
        let full = secs(&mut p, usize::MAX);
        assert!(
            (full - t::BURROW_MAX_TIME - t::BURROW_HOLD_EXTRA).abs() < 0.04,
            "끝까지 누르면 더 머문다 {full}"
        );
        assert!(
            p.burrow_cd > p.burrow_cd_full() + 0.3,
            "더 머문 만큼 대기가 는다"
        );
        let mut p = Player::new(room.center());
        let half = secs(&mut p, 36); // 0.6초에 뗀다
        assert!((half - 0.6).abs() < 0.04, "떼면 바로 솟구친다 {half}");
    }

    /// 강공격(0161·0167·0255): 누르면 평타가 바로, 누르고 있으면 스윙 뒤 모으기 → 누른 지 0.55초 뒤 떼면 피해 2 이상·넓은 한 방.
    /// 강공격 뒤 콤보는 다음 타로 이어지고, 후딜레이는 끊지 못하지만 잠금은 없다(0161 덧붙임 철회).
    #[test]
    fn press_swings_at_once_and_holding_charges_a_heavy_strike() {
        let room = Room::new();
        let press = Actions {
            attack: true,
            attack_held: true,
            ..Default::default()
        };
        let hold = Actions {
            attack_held: true,
            ..Default::default()
        };
        let release = Actions::default();
        // 누르는 순간 평타.
        let mut p = Player::new(room.center());
        let (_, ev) = tick(&mut p, &room, &press);
        assert!(
            matches!(p.state, State::Attack { idx: 0, .. }),
            "누르면 바로 평타"
        );
        assert!(ev.contains(&Event::Swing(0)));
        // 평타 뒤에도 누르고 있으면 모은다(스윙이 끝나고 쥔 지 `HEAVY_HOLD_START` 뒤, 0255), 다음 자리는 2타.
        let mut n = 0;
        while !matches!(p.state, State::Charging { .. }) && n < 100 {
            tick(&mut p, &room, &hold);
            n += 1;
        }
        assert!(
            matches!(p.state, State::Charging { after: 1, .. }),
            "스윙 뒤에도 누르고 있으면 모은다 {:?}",
            p.state
        );
        let mut ready = false;
        for _ in 0..((t::HEAVY_CHARGE / t::TICK_DT) as usize + 2) {
            let (s, ev) = tick(&mut p, &room, &hold);
            assert!(s.is_none(), "모으는 동안 판정은 없다");
            ready |= ev.contains(&Event::HeavyReady);
            if ready {
                break;
            }
        }
        assert!(ready, "다 모이면 발톱이 빛난다");
        let (_, ev) = tick(&mut p, &room, &release);
        assert!(ev.contains(&Event::Heavy));
        assert!(matches!(
            p.state,
            State::Heavy {
                after: 1,
                perfect: true,
                ..
            }
        ));
        assert!(p.parry_heavy(), "강공격 첫머리는 둘레 공격도 받아친다");
        let mut strike = None;
        let mut after_release = 0;
        for _ in 0..60 {
            let (s, _) = tick(&mut p, &room, &release);
            after_release += 1;
            if s.is_some() {
                strike = s;
                break;
            }
        }
        let s = strike.expect("강공격 판정");
        assert!(
            after_release <= 1,
            "떼고 바로 나간다(0189): {after_release}틱 뒤"
        );
        assert_eq!(s.kind, StrikeKind::Heavy);
        assert!(s.finisher);
        let heavy = p.moveset.heavy();
        assert_eq!(
            s.damage,
            heavy.damage + t::HEAVY_PERFECT_DAMAGE,
            "다 모인 직후에 뗐다: 딱 맞춘 강타는 +1(0168)"
        );
        assert!(
            heavy.damage >= 3,
            "강공격은 마무리 타보다 세다(0168) {}",
            heavy.damage
        );
        assert!((s.hitstop - heavy.hitstop - t::HEAVY_PERFECT_HITSTOP).abs() < 1e-5);
        let Shape::Sector { r, .. } = s.shape else {
            panic!("부채꼴");
        };
        let last = p.moveset.hits[p.moveset.last()].range;
        assert!((r - last * 1.3).abs() < 1e-3, "반지름 1.3배 {r}");
        // 강공격 뒤 잠금은 없다: 후딜레이가 끝나면 선다, 콤보는 2타로.
        let mut n = 0;
        while matches!(p.state, State::Heavy { .. }) && n < 100 {
            tick(&mut p, &room, &release);
            n += 1;
        }
        assert!(
            matches!(p.state, State::Free),
            "강공격이 끝나면 선다 {:?}",
            p.state
        );
        tick(&mut p, &room, &press);
        assert!(
            matches!(p.state, State::Attack { idx: 1, .. }),
            "강공격 뒤 콤보가 2타로 이어진다 {:?}",
            p.state
        );
        // 짧게 쥐고 떼면(보통 누름) 모으기에 들어서지 않는다 — 평타 하나만(0255).
        let mut q = Player::new(room.center());
        tick(&mut q, &room, &press);
        let mut swings = 1;
        for k in 0..60 {
            let a = if k < 8 { &hold } else { &release };
            let (_, ev) = tick(&mut q, &room, a);
            assert!(
                !matches!(q.state, State::Charging { .. }),
                "짧게 쥔 손은 모으지 않는다"
            );
            swings += ev.iter().filter(|e| matches!(e, Event::Swing(_))).count();
        }
        assert_eq!(swings, 1, "보통 누름은 평타 하나");
    }

    /// 딱 맞춘 강타(0168): 다 모인 뒤 `HEAVY_PERFECT`를 넘겨 떼면 보통 강공격(피해 그대로, `perfect` 아님).
    #[test]
    fn late_release_is_an_ordinary_heavy_strike() {
        let room = Room::new();
        let press = Actions {
            attack: true,
            attack_held: true,
            ..Default::default()
        };
        let hold = Actions {
            attack_held: true,
            ..Default::default()
        };
        let release = Actions::default();
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &press);
        let mut n = 0;
        while !matches!(p.state, State::Charging { .. }) && n < 100 {
            tick(&mut p, &room, &hold);
            n += 1;
        }
        for _ in 0..(((t::HEAVY_CHARGE + t::HEAVY_PERFECT + 0.06) / t::TICK_DT) as usize) {
            tick(&mut p, &room, &hold);
        }
        let (_, ev) = tick(&mut p, &room, &release);
        assert!(ev.contains(&Event::Heavy) && !ev.contains(&Event::HeavyPerfect));
        assert!(matches!(p.state, State::Heavy { perfect: false, .. }));
        let mut strike = None;
        for _ in 0..60 {
            let (s, _) = tick(&mut p, &room, &release);
            if s.is_some() {
                strike = s;
                break;
            }
        }
        let s = strike.expect("강공격 판정");
        assert_eq!(s.damage, p.moveset.heavy().damage);
    }

    /// 박자에 맞춘 타는 멈춤이 조금 더 길고, 이음을 모두 맞춘 콤보의 마무리 타는 더 멀리 민다(0168, 사장 아이디어).
    #[test]
    fn on_beat_hits_stop_longer_and_a_full_beat_combo_knocks_farther() {
        let room = Room::new();
        let tap = Actions {
            attack: true,
            ..Default::default()
        };
        let none = Actions::default();
        let ms = &crate::moveset::CLAWS;
        // 박자 콤보: 박자 창이 열리면 한 번씩 누른다(0174).
        let mut p = Player::new(room.center());
        let mut strikes = vec![];
        let mut pressed = 0;
        tick(&mut p, &room, &tap);
        for _ in 0..200 {
            let a = if strikes.len() <= ms.last()
                && pressed + 1 == strikes.len()
                && p.beat_cue().is_some_and(|c| c.window)
            {
                pressed += 1;
                &tap
            } else {
                &none
            };
            let (s, _) = tick(&mut p, &room, a);
            // 활성 구간은 여러 틱이라 같은 스윙은 한 번만.
            if let Some(s) = s.filter(|s| strikes.last().map(|l: &Strike| l.id) != Some(s.id)) {
                strikes.push(s);
            }
            if strikes.len() == ms.hits.len() {
                break;
            }
        }
        assert_eq!(strikes.len(), 3, "{strikes:?}");
        let beat_1 = strikes[1];
        let beat_fin = strikes[2];
        // 연타 콤보: 매 틱 누른다.
        let mut q = Player::new(room.center());
        let mut mashed = vec![];
        for _ in 0..200 {
            let (s, _) = tick(&mut q, &room, &tap);
            if let Some(s) = s.filter(|s| mashed.last().map(|l: &Strike| l.id) != Some(s.id)) {
                mashed.push(s);
            }
            if mashed.len() == ms.hits.len() {
                break;
            }
        }
        assert_eq!(mashed.len(), 3, "{mashed:?}");
        assert!(
            (beat_1.hitstop - mashed[1].hitstop - t::BEAT_HITSTOP_BONUS).abs() < 1e-5,
            "박자 타의 멈춤이 더 길다"
        );
        assert!(
            (beat_fin.knockback - mashed[2].knockback * t::FULL_BEAT_KNOCK).abs() < 1e-3,
            "이음을 다 맞춘 마무리 타가 더 멀리 민다 {} vs {}",
            beat_fin.knockback,
            mashed[2].knockback
        );
        assert_eq!(
            beat_fin.damage,
            mashed[2].damage + t::FULL_BEAT_DAMAGE,
            "이음을 다 맞춘 마무리 타는 피해 +1(0189)"
        );
    }

    #[test]
    fn a_nearby_threat_lets_an_early_press_out_at_once() {
        // 받아칠 위협이 가까우면(0189) 창 전에 누른 타도 박자를 기다리지 않는다 — 받아치기가 늦어지면 안 된다.
        let room = Room::new();
        let tap = Actions {
            attack: true,
            ..Default::default()
        };
        let none = Actions::default();
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        // 스윙이 끝나 설 때까지.
        let mut n = 0;
        while matches!(p.state, State::Attack { .. }) && n < 100 {
            tick(&mut p, &room, &none);
            n += 1;
        }
        p.parry_cd = 0.0;
        // 창 전: 위협이 없으면 기다린다.
        let (_, ev) = tick(&mut p, &room, &tap);
        assert!(
            !ev.contains(&Event::Swing(1)),
            "위협이 없으면 박자에: {ev:?}"
        );
        assert!(p.beat_cue().is_some_and(|c| c.waiting));
        // 위협이 오면 바로 나가고 받아치기 창이 열린다. 덤은 없다.
        p.threat = true;
        let (_, ev) = tick(&mut p, &room, &none);
        assert!(ev.contains(&Event::Swing(1)), "위협이면 바로: {ev:?}");
        assert!(!ev.iter().any(|e| matches!(e, Event::Beat(_))));
        assert!(p.parry_dir().is_some(), "받아치기 창이 열린다");
    }

    /// 박자 콤보(0174): 다음 타의 박자는 스윙 시작 `BEAT_PERIOD` 뒤. 창(±`BEAT_WINDOW`) 안에서 누르면 그 자리에서
    /// 다음 타(`Beat`), 창 전에 누르면(연타) 박자까지 기다렸다 나간다(`Beat` 없음, 버려지지 않음), 창 뒤에 누르면 그 자리에서
    /// `Beat` 없이. 강공격 뒤도 같고, 마지막 타 뒤는 1타부터.
    #[test]
    fn combo_on_the_beat_chains_at_the_press_and_early_presses_wait_for_the_beat() {
        let room = Room::new();
        let tap = Actions {
            attack: true,
            ..Default::default()
        };
        let none = Actions::default();
        let hit0 = crate::moveset::CLAWS.hits[0];
        let beat_at = t::BEAT_PERIOD.max(hit0.swing + hit0.recovery);
        // 박자: 창이 열리자마자 한 번 누른다 → 그 틱에 2타 + Beat(1).
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        let mut ticks = 0;
        let mut pressed = false;
        for n in 1..60 {
            // 눌림은 틱 머리에서 보니, 그때의 박자 시계는 (n-1)틱.
            let clock = (n - 1) as f32 * t::TICK_DT;
            let a = if clock >= beat_at - t::BEAT_WINDOW && !pressed {
                pressed = true;
                &tap
            } else {
                &none
            };
            let (_, ev) = tick(&mut p, &room, a);
            if ev.contains(&Event::Swing(1)) {
                assert!(ev.contains(&Event::Beat(1)), "박자에 맞췄다: {ev:?}");
                ticks = n;
                break;
            }
        }
        assert!(ticks > 0, "2타가 나간다");
        let pressed_at = ((beat_at - t::BEAT_WINDOW) / t::TICK_DT).ceil() as usize + 1;
        assert_eq!(ticks, pressed_at, "누른 틱에 바로 이어진다");
        assert!(p.beat_cue().is_some(), "콤보 중엔 박자 고리가 보인다");
        // 연타: 매 틱 누른다 → 첫 눌림이 창 전이라 박자에야 2타, Beat 없음.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        let mut mash_ticks = 0;
        for n in 1..60 {
            let (_, ev) = tick(&mut p, &room, &tap);
            if ev.contains(&Event::Swing(1)) {
                assert!(
                    !ev.iter().any(|e| matches!(e, Event::Beat(_))),
                    "연타는 박자가 아니다"
                );
                mash_ticks = n;
                break;
            }
        }
        assert!(mash_ticks > ticks, "연타 {mash_ticks} > 박자 {ticks}");
        assert!(
            (mash_ticks as f32 * t::TICK_DT - beat_at).abs() <= t::TICK_DT + 1e-4,
            "박자에 나간다: {mash_ticks}틱"
        );
        // 이른 눌림 한 번도 버려지지 않는다: 그 뒤 아무것도 안 눌러도 박자에 나간다. 기다리는 동안 고리는 '기다림'.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        tick(&mut p, &room, &tap);
        assert!(
            p.beat_cue().is_some_and(|c| c.waiting),
            "{:?}",
            p.beat_cue()
        );
        let mut out = false;
        for _ in 0..60 {
            let (_, ev) = tick(&mut p, &room, &none);
            if ev.contains(&Event::Swing(1)) {
                out = true;
                break;
            }
        }
        assert!(out, "이른 눌림도 나간다");
        // 늦은 눌림(창 뒤, 콤보 창 안): 그 자리에서 2타, Beat 없음.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        let late = ((beat_at + t::BEAT_WINDOW) / t::TICK_DT).ceil() as usize + 2;
        for _ in 0..late {
            tick(&mut p, &room, &none);
        }
        let (_, ev) = tick(&mut p, &room, &tap);
        assert!(ev.contains(&Event::Swing(1)), "늦어도 바로 나간다: {ev:?}");
        assert!(
            !ev.iter().any(|e| matches!(e, Event::Beat(_))),
            "늦은 눌림은 박자가 아니다"
        );
        // 마지막 타 뒤의 연타는 1타부터, 타마다 박자(마무리 타는 스윙·후딜레이 끝)에.
        let mut p = Player::new(room.center());
        let mut swings = vec![];
        for n in 0..200 {
            let (_, ev) = tick(&mut p, &room, &tap);
            for e in ev {
                if let Event::Swing(i) = e {
                    swings.push((n, i));
                }
            }
            if swings.len() >= 4 {
                break;
            }
        }
        assert_eq!(
            swings.iter().map(|s| s.1).collect::<Vec<_>>(),
            vec![0, 1, 2, 0],
            "{swings:?}"
        );
        let fin = crate::moveset::CLAWS.hits[2];
        let gaps: Vec<f32> = swings
            .windows(2)
            .map(|w| (w[1].0 - w[0].0) as f32 * t::TICK_DT)
            .collect();
        let want = [
            beat_at,
            beat_at,
            t::BEAT_PERIOD.max(fin.swing + fin.recovery),
        ];
        for (g, w) in gaps.iter().zip(want) {
            assert!(
                (g - w).abs() <= t::TICK_DT + 1e-4,
                "연타 간격 {gaps:?} vs {want:?}"
            );
        }
    }

    /// 덜 모은 강타(0253 → 0255, 사장 "중간 취소로 공격이 나가지 않는 경우는 없어야 함. 무기의 공격 속도만이 다음 공격을
    /// 유예시킨다"): 모든 무기에서 첫 타를 누른 채 몇 틱을 쥐든 — 모으는 자세에 들어섰으면 떼는 틱에 강타나 다음 평타가
    /// 나간다. 짧게 쥔 손(`HEAVY_HOLD_START` 아래)은 모으기에 들어서지 않는다. 뗀 타는 첫 타의 스윙·후딜레이가 끝나기 전에는
    /// 나가지 않는다.
    #[test]
    fn every_release_of_a_charge_attacks_and_only_the_swing_delays_it() {
        let room = Room::new();
        for w in crate::equipment::WEAPONS.iter() {
            let ms = w.moveset;
            let first = ms.hits[0].swing + ms.hits[0].recovery;
            for held in 1..=70usize {
                let mut p = Player::new(room.center());
                p.moveset = ms;
                let mut charged = false;
                let mut age = 0.0;
                for k in 0..held {
                    let a = Actions {
                        attack: k == 0,
                        attack_held: true,
                        ..Default::default()
                    };
                    tick(&mut p, &room, &a);
                    age += t::TICK_DT;
                    if matches!(p.state, State::Charging { .. }) {
                        charged = true;
                        assert!(
                            age + 1e-4 >= t::HEAVY_HOLD_START,
                            "{held}틱: 짧게 쥔 손은 모으지 않는다"
                        );
                    }
                }
                let was = p.state;
                let (_, ev) = tick(&mut p, &room, &Actions::default());
                age += t::TICK_DT;
                let attacked = ev
                    .iter()
                    .any(|e| matches!(e, Event::Swing(_) | Event::Heavy));
                if matches!(was, State::Charging { .. }) {
                    assert!(
                        attacked,
                        "{held}틱 쥐고 뗐는데 아무것도 안 나갔다 {:?}",
                        p.state
                    );
                    assert!(age + 1e-4 >= first, "{held}틱: 첫 타가 끝나기 전에 다음 타");
                    if ev.contains(&Event::Swing(1)) {
                        assert!(matches!(p.state, State::Attack { idx: 1, .. }));
                    }
                } else {
                    assert!(!charged || attacked, "{held}틱: 모으다 말았다");
                }
            }
        }
    }

    /// 무기마다 박자 콤보(사장 "무기마다 리듬 타격 검증, 2타가 제대로 안 나간다"): 박자 한가운데에 누르면 모든 이음이
    /// 다음 타 + Beat. 사람 손가락은 한 틱이 아니라 0.05~0.2초 누르고 있으니 그 길이마다 본다 — 활성 구간 뒤에도
    /// 누르고 있으면 모으기로 갔다가 덜 모으고 떼는데, 그때 콤보 창이 박자보다 먼저 닫히면 1타로 돌아갔다.
    #[test]
    fn every_weapon_chains_every_hit_on_the_beat_whatever_the_tap_length() {
        let room = Room::new();
        for (name, ms) in [
            ("claws", &moveset::CLAWS),
            ("sprout", &moveset::SPROUT),
            ("kicks", &moveset::KICKS),
            ("stone", &moveset::STONE),
            ("venom", &moveset::VENOM),
            ("acorns", &moveset::ACORNS),
        ] {
            // 박자 창 안 어디서 누르든(이른 끝·한가운데·늦은 끝).
            let edge = t::BEAT_WINDOW - t::TICK_DT * 1.5;
            for (hold, off) in [1usize, 3, 5, 6, 7, 8, 10, 12, 15]
                .into_iter()
                .flat_map(|h| [(h, -edge), (h, 0.0), (h, edge)])
            {
                let mut p = Player::new(room.center());
                p.moveset = ms;
                let n = ms.hits.len();
                let mut swings: Vec<(usize, bool)> = vec![];
                // 첫 타를 누르고, 그 뒤로는 박자 시계가 박자에 닿는 틱에 눌러 `hold`틱 동안 쥔다.
                let mut held = 0usize;
                let mut press_next = true;
                for _ in 0..600 {
                    // 사람은 박자 고리에 맞춰 누른다(콤보가 살아 있든 아니든).
                    let due = !press_next && p.beat_clock + t::TICK_DT * 0.5 >= p.beat_at + off;
                    let start = press_next || (due && held == 0);
                    if start {
                        press_next = false;
                        held = hold;
                    }
                    let a = Actions {
                        attack: start,
                        attack_held: held > 0,
                        ..Default::default()
                    };
                    held = held.saturating_sub(1);
                    let (_, ev) = tick(&mut p, &room, &a);
                    for e in &ev {
                        if let Event::Swing(i) = e {
                            swings.push((*i, ev.iter().any(|e| matches!(e, Event::Beat(_)))));
                        }
                    }
                    if swings.len() >= n {
                        break;
                    }
                }
                let idx: Vec<usize> = swings.iter().map(|s| s.0).collect();
                assert_eq!(
                    idx,
                    (0..n).collect::<Vec<_>>(),
                    "{name} 쥔 {hold}틱 {off:+.3}초: 타 순서 {swings:?}"
                );
                assert!(
                    swings[1..].iter().all(|s| s.1),
                    "{name} 쥔 {hold}틱 {off:+.3}초: 박자에 누른 이음은 모두 Beat {swings:?}"
                );
                assert!(p.full_beat(n - 1), "{name} 쥔 {hold}틱: 다 맞춘 마무리");
            }
        }
    }

    /// 메트로놈(0180): 한 타 뒤 아무것도 안 누르면 박자에 `BeatTick` 한 번. 이른 눌림이 박자에 나가면
    /// 그 틱은 타 소리가 박자라 `BeatTick`이 없다. 콤보가 아닐 때(서 있기만)는 울리지 않는다.
    #[test]
    fn metronome_ticks_on_the_beat_unless_a_swing_lands_there() {
        let room = Room::new();
        let tap = Actions {
            attack: true,
            ..Default::default()
        };
        let none = Actions::default();
        let hit0 = crate::moveset::CLAWS.hits[0];
        let beat_at = t::BEAT_PERIOD.max(hit0.swing + hit0.recovery);
        let ticks_of = |p: &mut Player, presses: &[usize]| {
            let mut at = vec![];
            for n in 1..60 {
                let a = if presses.contains(&n) { &tap } else { &none };
                let (_, ev) = tick(p, &room, a);
                if ev.contains(&Event::BeatTick) {
                    at.push(n);
                }
            }
            at
        };
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        let at = ticks_of(&mut p, &[]);
        assert_eq!(at.len(), 1, "박자에 한 번: {at:?}");
        assert!(
            (at[0] as f32 * t::TICK_DT - beat_at).abs() <= t::TICK_DT + 1e-4,
            "박자에: {at:?}"
        );
        // 이른 눌림 → 박자에 2타가 나가고 그 틱엔 똑이 없다.
        let mut p = Player::new(room.center());
        tick(&mut p, &room, &tap);
        let at = ticks_of(&mut p, &[2]);
        let first = ((beat_at / t::TICK_DT).round() as usize).max(1);
        assert!(
            !at.iter().any(|n| n.abs_diff(first) <= 1),
            "타가 박자: {at:?}"
        );
        // 콤보가 아니면 울리지 않는다.
        let mut p = Player::new(room.center());
        assert!(ticks_of(&mut p, &[]).is_empty());
    }

    /// 땅속 대시(0160): 정한 시간까지는 빠르게, 힘을 모으는 동안은 느리게 간다.
    #[test]
    fn burrow_dashes_fast_then_slows_while_charging() {
        let room = Room::new();
        let mut p = Player::new(room.center());
        let dig = Actions {
            dodge: true,
            dodge_held: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        tick(&mut p, &room, &dig);
        let go = Actions {
            dodge_held: true,
            move_dir: vec2(1.0, 0.0),
            ..Default::default()
        };
        let x0 = p.pos.x;
        let base_ticks = (t::BURROW_MAX_TIME / t::TICK_DT) as usize;
        for _ in 0..base_ticks {
            tick(&mut p, &room, &go);
        }
        let dash = p.pos.x - x0;
        let x1 = p.pos.x;
        for _ in 0..base_ticks {
            tick(&mut p, &room, &go);
        }
        let crawl = p.pos.x - x1;
        assert!(matches!(p.state, State::Burrow { .. }));
        assert!(
            dash > crawl * 3.0,
            "대시 {dash} 는 모으는 동안 {crawl} 보다 훨씬 멀리"
        );
        let plain = t::PLAYER_SPEED * t::BURROW_MAX_TIME;
        assert!(dash > plain * 1.8, "걷기보다 훨씬 빠르다 {dash} vs {plain}");
    }

    /// 힘 모으기(0158): 톡은 보통 솟구침, 정한 시간을 넘겨 누르고 있으면 힘이 차고(땅이 운다) 끝까지 모으면
    /// 반지름·피해·넉백이 큰 솟구침. 반지름은 재주 값 × EMERGE_CHARGE_RADIUS까지.
    #[test]
    fn holding_charges_a_bigger_emerge() {
        let room = Room::new();
        let run = |hold_ticks: usize| -> (Strike, bool, f32) {
            let mut p = Player::new(room.center());
            let dig = Actions {
                dodge: true,
                dodge_held: true,
                ..Default::default()
            };
            tick(&mut p, &room, &dig);
            let mut n = 1;
            let mut rumbled = false;
            let mut peak = 0.0f32;
            loop {
                let a = Actions {
                    dodge_held: n < hold_ticks,
                    ..Default::default()
                };
                let (strike, events) = tick(&mut p, &room, &a);
                rumbled |= events.contains(&Event::Rumble);
                peak = peak.max(p.burrow_charge());
                if let Some(s) = strike {
                    return (s, rumbled, peak);
                }
                n += 1;
                assert!(n < 200, "솟구치지 않는다");
            }
        };
        let base = t::EMERGE_RADIUS;
        let (tap, rumbled, peak) = run(1);
        assert!(!rumbled && peak == 0.0, "톡: 힘을 모으지 않는다");
        assert_eq!(tap.damage, t::EMERGE_DAMAGE);
        let Shape::Circle { r, .. } = tap.shape else {
            panic!("원");
        };
        assert!((r - base).abs() < 1e-3, "톡 = 보통 반지름 {r}");

        let (full, rumbled, peak) = run(usize::MAX);
        assert!(rumbled, "정한 시간을 넘기면 땅이 운다");
        assert!((peak - 1.0).abs() < 1e-3, "끝까지 누르면 다 찬다 {peak}");
        let Shape::Circle { r, .. } = full.shape else {
            panic!("원");
        };
        assert!(
            (r - base * t::EMERGE_CHARGE_RADIUS).abs() < 1e-3,
            "다 모으면 큰 반지름 {r}"
        );
        assert_eq!(full.damage, t::EMERGE_DAMAGE + t::EMERGE_CHARGE_DAMAGE);
        assert!(full.knockback > tap.knockback && full.stun > tap.stun);

        // 반쯤(정한 시간 + 더 머무는 시간의 절반) 떼면 그 사이.
        let mid = ((t::BURROW_MAX_TIME + t::BURROW_HOLD_EXTRA * 0.5) / t::TICK_DT) as usize;
        let (half, _, peak) = run(mid);
        let Shape::Circle { r, .. } = half.shape else {
            panic!("원");
        };
        assert!(
            r > base + 1.0 && r < base * t::EMERGE_CHARGE_RADIUS - 1.0,
            "반쯤 {r}"
        );
        assert!(peak > 0.3 && peak < 0.7, "반쯤 찬다 {peak}");
    }
}
