//! 적: 뿌리에 물든 땅속 이웃. 종류(`Kind`)마다 행동 하나와 수치가 `species.rs` 표에 있다.
//! 행동: 돌격(쥐들), 던지기(다람쥐), 뛰어 덮치기(청설모), 보스 넷(대장·모모·도토·콩이). 플레이어 위치와 `Room`만 본다.

use macroquad::prelude::*;

use crate::combat::{self, Strike, StrikeKind};
use crate::room::Room;
use crate::species::{self, Behavior, Species};
use crate::tuning as t;

// 0169 R5: 상태 함수를 셋으로 나눴다 — 잡몹의 돌격(melee), 거리 두는 적(ranged), 보스의 수(boss). 본문은 그대로.
mod boss;
mod melee;
mod ranged;
pub use ranged::tongue_out;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// 들쥐: 기본 돌격.
    FieldMouse,
    /// 생쥐: 작고 빠르고 약하다. 떼로.
    HouseMouse,
    /// 시궁쥐: 크고 단단하고 세게 돌진.
    SewerRat,
    /// 다람쥐: 도토리를 던진다.
    Chipmunk,
    /// 청설모: 뛰어 덮친다.
    RedSquirrel,
    /// 튜토리얼 허수아비. 움직이지도 때리지도 않는다.
    Dummy,
    /// 튜토리얼 보스: 들쥐 대장(보리). 돌진 · 뛰어 찍기 · 부하 부르기(docs/decisions/0032).
    Chief,
    /// 첫 길의 보스: 물든 굴토끼 모모. 세 번 만난다(docs/decisions/0046).
    Momo,
    /// 둘째 길의 보스: 물든 청설모 도토. 던지고, 뛰고, 묻는다(docs/decisions/0052).
    Doto,
    /// 셋째 길의 보스: 물든 세띠아르마딜로 콩이. 굴러 튕기고, 웅크려 버틴다(docs/decisions/0054).
    Kongi,
    /// 굴토끼 옛 굴의 보스: 물든 어미 굴토끼 토끼풀. 발을 구르고, 두 번 차고, 뛰어 내려앉고, 고치에 실을 감는다(0165).
    Clover,
    /// 새 잡몹(0146): 뒤쥐(두 번 물기)·고슴도치(공으로 구르기)·두꺼비(혀)·박쥐(날며 내리꽂기)·실버섯(포자 구름).
    Shrew,
    Hedgehog,
    Toad,
    Bat,
    SporeCap,
}

impl Kind {
    pub fn species(self) -> &'static Species {
        species::of(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// 돌격형: 플레이어 쪽으로 걷기.
    Approach,
    /// 돌격형: 돌진 예고(멈춰서 떨림).
    Windup { time: f32 },
    /// 돌격형: 돌진. 방향은 시작 시 고정.
    Charge { time: f32, dir: Vec2 },
    /// 돌격형: 돌진 뒤 멈춤(남은 시간).
    Rest { time: f32 },
    /// 원거리형: 거리 유지.
    Keep,
    /// 원거리형: 조준 예고.
    Aim { time: f32 },
    /// 뛰기형: 웅크림(예고). 착지할 자리를 정해 둔다.
    Crouch { time: f32, target: Vec2 },
    /// 뛰기형: 체공. `from`에서 `to`로 포물선.
    Leap { time: f32, from: Vec2, to: Vec2 },
    /// 공통: 경직(남은 시간).
    Stunned { time: f32 },
    /// 대장: 울부짖어 부하를 부른다(예고 뒤 Event::Summon). 도토: 땅을 판다(예고 뒤 Event::Bury).
    Howl { time: f32 },
    /// 모모: 땅속(맞지 않고 닿아도 안 아프다). 흙 둔덕이 플레이어를 따라오다 멈추고 떤다.
    Under { time: f32 },
    /// 모모: 튀어나옴. 첫머리에만 둘레 피해.
    Pop { time: f32 },
    /// 콩이: 웅크려 버티기. 발톱은 등딱지에 튕긴다(`guarding`).
    Guard { time: f32 },
    /// 두꺼비: 혀를 내민 채(시작 틱에 월드가 맞았는지 본다, 0146).
    Lash { time: f32, dir: Vec2 },
    /// 실버섯: 포자를 터뜨림(시작 틱에 월드가 구름을 놓는다, 0146).
    Puff { time: f32 },
    /// 토끼풀(0165): 뒷발 구르기 예고. 끝나면 `Event::Thump`(월드가 충격파와 실 그물을 놓는다).
    Thump { time: f32 },
    /// 토끼풀(0165): 고치로 가 실을 더 감는다(반 아래, 한 번). `CLOVER_THREAD_AT`에 `Event::Thread`(실 한 가닥).
    Gather { time: f32 },
}

/// 플레이어의 타격을 받은 결과(0170 R10). 월드는 이것만 보고 연출·통계·소리를 낸다 — 종 규칙은 전부 `Enemy::receive` 안.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Struck {
    /// 닿지 않았다: 이미 이 스윙에 맞았다, 땅속, 모양 밖, 나는 박쥐에 솟구치기(0146).
    Miss,
    /// 등딱지·가시에 튕겼다(콩이·고슴도치, 0054): 발톱만. 플레이어가 밀려난다.
    Deflected,
    /// 맞았다. `dir`은 민 방향. `broke_guard`면 등딱지를 열었다(솟구치기·드릴·강공격, 0168).
    Hit { dir: Vec2, broke_guard: bool },
    /// 밀치기만 닿았다(0214): 피해·경직 없이 `dir`로 밀렸다.
    Shoved { dir: Vec2 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// 방에 나타나기를 마쳤다(땅에서 솟음 / 천장에서 떨어짐, 0257). 흙·먼지는 월드가.
    Arrived(Entry, Vec2),
    Charge,
    WallHit,
    Shoot,
    /// 뛰기형이 튀어 오름 / 내려앉음.
    Leap,
    Land,
    /// 대장이 부하를 부른다(월드가 생쥐를 둘레에 낳는다).
    Summon,
    /// 모모가 땅속으로 / 밖으로.
    Dig,
    Pop,
    /// 도토가 도토리를 묻었다(월드가 플레이어 발밑·둘레에 묻힌 도토리를 놓는다). 몇 개인지.
    Bury(usize),
    /// 두꺼비가 혀를 내밀었다 / 실버섯이 포자를 터뜨렸다(소리용, 판정은 월드가 상태로 본다).
    Tongue,
    Spore,
    /// 토끼풀이 발을 굴렀다(그 자리) / 실 한 가닥을 뻗었다(자리, 방향). 판정은 월드가(0165).
    Thump(Vec2),
    Thread(Vec2, Vec2),
    /// 보스가 체력 반 아래에서 2단계로 깨어났다(그 자리, 0233). 한 싸움에 한 번.
    Rouse(Vec2),
}

/// 방에 나타나는 방법(0257): 그 자리에서 깨기 / 땅에서 솟기 / 천장에서 떨어지기.
/// 솟기·떨어지기 동안(`wake_delay`)은 맞지도 때리지도 않고 그림만 — 둔덕이 부풀거나 그림자가 커진다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Wake,
    Burrow,
    Drop,
}

pub struct Enemy {
    pub kind: Kind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    /// 걸어온 길이(px, 그림 전용): 걷기 칸을 시계가 아니라 걸음으로 넘긴다(0241).
    pub walked: f32,
    pub vel: Vec2,
    pub hp: i32,
    /// 처음 체력(실뜨기가 바꾼다). 막대·반 아래 판정은 이걸 본다.
    pub max_hp: i32,
    /// 예고 시간 배율(실뜨기 2단계)·걷기 배율(실뜨기 6단계).
    pub windup_mul: f32,
    /// 예고 중 겨눈 쪽을 돌리는 지금 빠르기(rad/s, 0254). 예고가 시작되면 0부터.
    pub turn_speed: f32,
    pub speed_mul: f32,
    /// 실뜨기를 이미 걸었나(월드가 새로 나온 적에 한 번).
    pub knotted: bool,
    /// 보스 2단계(0233): 체력 반 아래에서 한 번 깨어났다 — 예고·쉬기·걸음이 빨라지고 덜 멈칫한다.
    pub roused: bool,
    pub state: State,
    pub facing: Vec2,
    pub flash: f32,
    pub alive: bool,
    /// 마지막으로 맞은 스윙 번호. 같은 스윙에 두 번 맞지 않는다.
    pub last_hit_by: u32,
    /// 벽 쿵(0168): 강공격에 맞아 날아가는 동안 벽에 박히면 입을 피해(0 = 없음). 경직이 끝나면 사라진다.
    pub slam: i32,
    /// 이번 틱에 벽에 박혔다(월드가 소리·파편을 낸다).
    pub slammed: bool,
    /// 가시뿌리 덤불에 다시 찔리기까지(0258): 걷는 적은 덤불 위에 서 있으면 이 간격으로 찔린다.
    pub thorn_cd: f32,
    fire_cd: f32,
    /// 남은 독(초)과 다음 독 피해까지 남은 시간.
    pub poison: f32,
    poison_tick: f32,
    /// 이번 틱에 독으로 쓰러졌다(월드가 처치로 센다).
    pub died_of_poison: bool,
    /// 필드의 적: 플레이어가 가까이 오거나 맞기 전까지 제자리.
    pub asleep: bool,
    /// 던전 방: 이 시간 동안 제자리(들어온 플레이어가 자리 잡게).
    pub wake_delay: f32,
    /// 나타나는 방법과 그 길이(0257, `arrive`). `Wake`면 그냥 깨기를 기다린다.
    pub entry: Entry,
    entry_time: f32,
    /// 대장·모모: 다음 공격 차례, 대장이 부른 횟수, 모모가 남은 이어 뛰기.
    pattern: u32,
    summons: u32,
    chain: u32,
    /// 보스가 몇 번째 만남인가(모모: 1~3, 한 수씩 는다). 다른 적은 1.
    pub level: u8,
    /// 엘리트(실타래 칸, 0147): 체력 셋 배, 예고가 짧고 덜 밀린다.
    pub elite: bool,
    /// 흰 실 그물(0165)에 걸린 남은 시간. 걷는 속도가 `WEB_SLOW`배(나는 것·보스는 안 걸린다).
    pub webbed: f32,
    /// 토끼풀(0165): 지키는 자리의 가운데(아기들의 고치). 늘 여기와 플레이어 사이에 선다.
    pub guard: Option<Vec2>,
}

pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub alive: bool,
    /// 패링으로 되돌린 것: 적을 때리고 플레이어는 맞지 않는다.
    pub friendly: bool,
}

impl Projectile {
    pub fn radius(&self) -> f32 {
        t::PROJECTILE_RADIUS
    }

    pub fn update(&mut self, room: &Room, dt: f32) {
        self.prev_pos = self.pos;
        self.pos += self.vel * dt;
        if room.is_solid(self.pos, self.radius()) {
            self.alive = false;
        }
    }
}

/// 한 틱의 바깥 상황(0169 R5): 상태 함수들이 같은 것을 본다. `events`·`projectiles`는 월드의 것(빌림).
pub(super) struct Ctx<'a> {
    pub player_pos: Vec2,
    /// 플레이어를 노릴 수 있나(살아 있고 땅속이 아니다).
    pub targetable: bool,
    pub room: &'a Room,
    pub dt: f32,
    pub to_player: Vec2,
    pub dist: f32,
    pub dir_to_player: Vec2,
    pub projectiles: &'a mut Vec<Projectile>,
    pub events: &'a mut Vec<Event>,
}

impl Enemy {
    /// 예고 시간(종 수치 × 실뜨기 배율).
    /// 예고 동안 `want` 쪽으로 겨눈다(0254): 한 번에 휙 돌지 않고, 0에서 가속해 최대 빠르기까지만 돈다.
    /// `time`은 그 예고 상태의 지난 시간 — 첫 틱이면 빠르기를 0으로(새 예고).
    pub(super) fn aim_toward(&mut self, want: Vec2, time: f32, dt: f32) {
        if time <= dt * 1.5 {
            self.turn_speed = 0.0;
        }
        let want = want.normalize_or(self.facing);
        let cur = self.facing.normalize_or(want);
        self.turn_speed = (self.turn_speed + t::ENEMY_TURN_ACCEL * dt).min(t::ENEMY_TURN_MAX);
        let diff = cur.perp_dot(want).atan2(cur.dot(want));
        let step = self.turn_speed * dt;
        self.facing = if diff.abs() <= step {
            want
        } else {
            Vec2::from_angle(step * diff.signum()).rotate(cur)
        };
    }

    pub fn windup(&self) -> f32 {
        self.kind.species().windup * self.windup_mul
    }

    /// 실뜨기 매듭을 건다(한 번만): 체력·예고. `tells` = 포포의 응원 "공격 예고 길게"(0256)도 같이.
    pub fn apply_knots(&mut self, k: crate::knots::Knots, tells: f32) {
        if self.knotted {
            return;
        }
        self.knotted = true;
        self.max_hp = (self.max_hp as f32 * k.enemy_hp_mul()).ceil() as i32;
        if self.is_boss() {
            self.max_hp = (self.max_hp as f32 * k.boss_hp_mul()).ceil() as i32;
        }
        self.hp = self.max_hp;
        self.windup_mul *= k.windup_mul() * tells;
        self.speed_mul *= k.enemy_speed_mul();
    }

    /// 걷기(거리 맞추기) 속도(종 수치 × 실뜨기 배율).
    pub fn speed(&self) -> f32 {
        let web = if self.webbed > 0.0 { t::WEB_SLOW } else { 1.0 };
        let p2 = if self.roused { t::BOSS_P2_SPEED } else { 1.0 };
        self.kind.species().speed * self.speed_mul * web * p2
    }

    pub fn new(kind: Kind, pos: Vec2) -> Self {
        let sp = kind.species();
        let hp = sp.hp;
        let state = idle_of(sp.behavior);
        Self {
            kind,
            pos,
            prev_pos: pos,
            walked: 0.0,
            vel: Vec2::ZERO,
            hp,
            max_hp: hp,
            windup_mul: 1.0,
            turn_speed: 0.0,
            speed_mul: 1.0,
            knotted: false,
            roused: false,
            state,
            facing: vec2(-1.0, 0.0),
            flash: 0.0,
            alive: true,
            last_hit_by: 0,
            fire_cd: sp.fire_every * 0.5,
            poison: 0.0,
            poison_tick: t::POISON_TICK,
            died_of_poison: false,
            slam: 0,
            slammed: false,
            thorn_cd: 0.0,
            asleep: false,
            wake_delay: 0.0,
            entry: Entry::Wake,
            entry_time: 0.0,
            pattern: 0,
            summons: 0,
            chain: 0,
            level: 1,
            elite: false,
            webbed: 0.0,
            guard: None,
        }
    }

    /// 엘리트로(0147). 실뜨기보다 먼저 건다(실뜨기는 배율을 곱한다).
    pub fn make_elite(&mut self) {
        self.elite = true;
        self.max_hp = (self.max_hp as f32 * t::ELITE_HP).ceil() as i32;
        self.hp = self.max_hp;
        self.windup_mul *= t::ELITE_WINDUP;
    }

    pub fn sp(&self) -> &'static Species {
        self.kind.species()
    }

    pub fn radius(&self) -> f32 {
        self.sp().radius
    }

    /// 지금 닿으면 플레이어가 입는 피해.
    pub fn contact_damage(&self) -> Option<i32> {
        match self.state {
            State::Charge { .. } => Some(self.sp().damage),
            // 내려앉기 직전 몇 틱.
            State::Leap { time, .. } if time >= self.sp().dash_time - t::LEAP_LAND_WINDOW => {
                Some(self.sp().damage)
            }
            State::Pop { time } if time < t::MOMO_POP_WINDOW => Some(self.sp().damage),
            _ => None,
        }
    }

    /// 공격 예고의 진행(0176 → 0179): 0 = 예고 시작, 1 = 공격이 나가는 순간. 예고 중이 아니면 None.
    /// 길이는 각 상태가 쓰는 식(`windup()`·배율·`thump_windup()`)과 같아 그림과 판정이 어긋나지 않는다.
    /// 그림은 공격 방향 안내(돌진 띠·조준 점선·내려앉을 자리) 자체가 적 쪽부터 차오른다(`render/fx.rs draw_telegraphs`).
    pub fn telegraph(&self) -> Option<f32> {
        let (time, dur) = match self.state {
            State::Windup { time } | State::Aim { time } | State::Crouch { time, .. } => {
                (time, self.windup())
            }
            State::Howl { time } => {
                let m = if self.sp().behavior == Behavior::Doto {
                    t::DOTO_BURY_MUL
                } else {
                    t::CHIEF_HOWL_MUL
                };
                (time, self.windup() * m)
            }
            State::Thump { time } => (time, self.thump_windup()),
            _ => return None,
        };
        Some(tele_progress(time, dur))
    }

    /// 받아칠 수 있는 공격인가(0160): 곧게 오는 돌진(물기·구르기·내리꽂기도 `Charge`)만. 내려앉기·튀어나오기는
    /// 둘레 공격이라 안 된다. 예고도 띠·점선(황금) ↔ 고리(진홍, ✕)로 갈린다.
    pub fn parryable(&self) -> bool {
        match self.state {
            State::Charge { .. } => true,
            // 두꺼비 혀가 뻗는 동안도 곧게 오는 공격(0258) — 받아칠 위협으로 센다(박자 스냅을 건너뛴다).
            State::Lash { time, .. } => time <= t::TOAD_EXTEND_TIME,
            _ => false,
        }
    }

    /// 곧게 오는 공격의 방향(돌진·혀, 0258): 닿기 직전 받아치기(`PARRY_REACH`)가 다가오는 것인지 본다.
    pub fn strike_dir(&self) -> Option<Vec2> {
        match self.state {
            State::Charge { dir, .. } => Some(dir),
            State::Lash { dir, time } if time <= t::TOAD_EXTEND_TIME => Some(dir),
            _ => None,
        }
    }

    /// 피해 판정 반지름(내려앉는·튀어나오는 순간은 둘레까지).
    pub fn contact_radius(&self) -> f32 {
        match self.state {
            State::Leap { .. } | State::Pop { .. } => self.sp().land_radius,
            _ => self.radius(),
        }
    }

    /// 맞을 수 있나(땅속의 모모는 안 맞는다).
    pub fn hittable(&self) -> bool {
        !matches!(self.state, State::Under { .. }) && self.arriving().is_none()
    }

    /// 방에 나타난다(0257): `time`초 동안 솟거나 떨어지고, 끝나면 `Event::Arrived` 뒤 바로 움직인다.
    pub fn arrive(&mut self, entry: Entry, time: f32) {
        self.entry = entry;
        self.entry_time = time.max(0.01);
        self.wake_delay = time;
    }

    /// 나타나는 중이면 (방법, 진행 0..1).
    pub fn arriving(&self) -> Option<(Entry, f32)> {
        (self.entry != Entry::Wake && self.wake_delay > 0.0)
            .then(|| (self.entry, 1.0 - self.wake_delay / self.entry_time))
    }

    /// 날고 있다(박쥐, 0146): 솟구치기·솟구침 밀어내기가 닿지 않는다. 발톱·드릴은 닿는다.
    pub fn flying(&self) -> bool {
        self.sp().behavior == Behavior::Swooper
    }

    /// 체공 높이(그리기용 px). 바닥 = 0. 박쥐는 늘 떠 있고, 내리꽂을 때 낮아진다.
    pub fn height(&self) -> f32 {
        // 천장에서 떨어지는 중(0257): 가만히 놓은 돌처럼 점점 빨리.
        if let Some((Entry::Drop, k)) = self.arriving() {
            return t::DROP_HEIGHT * (1.0 - k * k);
        }
        match self.state {
            State::Leap { time, .. } => {
                let k = (time / self.sp().dash_time).clamp(0.0, 1.0);
                4.0 * t::LEAP_HEIGHT * k * (1.0 - k)
            }
            State::Charge { .. } if self.flying() => t::BAT_HEIGHT * 0.4,
            _ if self.flying() => t::BAT_HEIGHT,
            _ => 0.0,
        }
    }

    /// 돌진이 플레이어에게 명중했을 때.
    pub fn on_hit_player(&mut self) {
        if matches!(self.state, State::Charge { .. }) {
            self.state = State::Rest {
                time: self.sp().rest,
            };
        }
    }

    pub fn update(
        &mut self,
        player_pos: Vec2,
        player_targetable: bool,
        room: &Room,
        dt: f32,
        projectiles: &mut Vec<Projectile>,
        events: &mut Vec<Event>,
    ) {
        self.prev_pos = self.pos;
        self.flash = (self.flash - dt).max(0.0);
        self.thorn_cd = (self.thorn_cd - dt).max(0.0);
        self.tick_poison(dt);
        let sp = self.sp();
        if sp.behavior == Behavior::Dummy {
            // 허수아비: 넉백만 받고 제자리로 돌아오지 않는다. 경직은 즉시 푼다.
            self.pos += self.vel * dt;
            combat::apply_friction(&mut self.vel, dt);
            let r = self.radius();
            room.resolve_circle(&mut self.pos, r);
            return;
        }
        let to_player = player_pos - self.pos;
        let dist = to_player.length();
        let dir_to_player = to_player.normalize_or_zero();
        // 보스는 가까이 와도 깨지 않는다(공터에 들어서면 모험이 깨운다).
        if self.asleep && player_targetable && dist < t::ENEMY_AGGRO_RADIUS && !self.is_boss() {
            self.asleep = false;
        }
        if self.wake_delay > 0.0 {
            self.wake_delay -= dt;
            if self.wake_delay <= 0.0 && self.entry != Entry::Wake {
                events.push(Event::Arrived(self.entry, self.pos));
                self.entry = Entry::Wake;
            }
        }
        if self.asleep || self.wake_delay > 0.0 {
            // 자는 동안은 넉백만.
            self.pos += self.vel * dt;
            combat::apply_friction(&mut self.vel, dt);
            let r = self.radius();
            room.resolve_circle(&mut self.pos, r);
            return;
        }
        if self.is_boss() && !self.roused && self.alive && self.hurt_half() {
            self.rouse(events);
        }

        let mut c = Ctx {
            player_pos,
            targetable: player_targetable,
            room,
            dt,
            to_player,
            dist,
            dir_to_player,
            projectiles,
            events,
        };
        match self.state {
            State::Stunned { time } => self.tick_stunned(&mut c, time),
            State::Approach
                if matches!(
                    sp.behavior,
                    Behavior::Chief
                        | Behavior::Momo
                        | Behavior::Doto
                        | Behavior::Kongi
                        | Behavior::Clover
                ) =>
            {
                self.tick_boss_approach(&mut c)
            }
            State::Thump { time } => self.tick_thump(&mut c, time),
            State::Gather { time } => self.tick_gather(&mut c, time),
            State::Howl { time } => self.tick_howl(&mut c, time),
            State::Approach => self.tick_approach(&mut c),
            State::Windup { time } => self.tick_windup(&mut c, time),
            State::Charge { time, dir } => self.tick_charge(&mut c, time, dir),
            State::Rest { time } => self.tick_rest(&mut c, time),
            State::Keep => self.tick_keep(&mut c),
            State::Crouch { time, target } => self.tick_crouch(&mut c, time, target),
            State::Leap { time, from, to } => self.tick_leap(&mut c, time, from, to),
            State::Under { time } => self.tick_under(&mut c, time),
            State::Guard { time } => self.tick_guard(&mut c, time),
            State::Pop { time } => self.tick_pop(&mut c, time),
            State::Lash { time, dir } => self.tick_lash(&mut c, time, dir),
            State::Puff { time } => self.tick_puff(&mut c, time),
            State::Aim { time } => self.tick_aim(&mut c, time),
        }

        let before = self.vel;
        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        let hit_wall = room.resolve_circle(&mut self.pos, r);
        // 벽 쿵(0168): 강공격 넉백으로 날아가다 벽에 박히면 한 번 더 아프고 튕긴다.
        if self.slam > 0 {
            if hit_wall && before.length() >= t::WALL_SLAM_SPEED {
                let dmg = self.slam;
                self.slam = 0;
                self.slammed = true;
                let back = -before.normalize_or_zero();
                let stun = match self.state {
                    State::Stunned { time } => time.max(0.4),
                    _ => 0.4,
                };
                self.hurt(dmg, back, 70.0, stun);
            } else if !matches!(self.state, State::Stunned { .. }) || before.length() < 20.0 {
                self.slam = 0;
            }
        }
    }

    fn idle_state(&self) -> State {
        idle_of(self.sp().behavior)
    }

    /// 발 구르기 예고(0165): 반 아래면 짧다. 실뜨기 배율은 그대로.
    pub fn thump_windup(&self) -> f32 {
        let base = if self.hurt_half() {
            t::CLOVER_THUMP_WINDUP_HURT
        } else {
            t::CLOVER_THUMP_WINDUP
        };
        base * self.windup_mul
    }

    /// 콩이·고슴도치가 공으로 말려 있다(웅크림·버티기·구르기): 발톱은 등딱지·가시에 튕긴다.
    pub fn guarding(&self) -> bool {
        matches!(self.sp().behavior, Behavior::Kongi | Behavior::Roller)
            && matches!(
                self.state,
                State::Windup { .. } | State::Charge { .. } | State::Guard { .. }
            )
    }

    /// 발톱이 등딱지에 튕겼다(패링 거울): 버티던 중이면 곧바로 굴러 온다(예고는 짧게).
    pub fn shell_hit(&mut self) {
        self.asleep = false;
        self.wake_delay = 0.0;
        self.flash = t::HIT_FLASH * 0.5;
        if let State::Guard { .. } = self.state {
            self.chain = self.kongi_bounces();
            self.state = State::Windup {
                time: self.windup() * t::KONGI_COUNTER_SKIP,
            };
        }
    }

    /// 솟구치기·드릴이 등딱지를 열었다: 오래 어질어질(피해는 따로 `hurt`).
    pub fn break_guard(&mut self) {
        self.chain = 0;
        self.state = State::Stunned {
            time: t::KONGI_BREAK_STUN,
        };
    }

    /// 패링에 받아쳐졌다: 구르던 콩이는 멈춰 어질어질(다른 보스는 `hurt`가 다룬다).
    pub fn on_parried(&mut self) {
        // 토끼풀(0165): 첫 발을 받아치면 둘째 차기가 끊긴다.
        if self.sp().behavior == Behavior::Clover {
            self.chain = 0;
        }
        if matches!(self.sp().behavior, Behavior::Kongi | Behavior::Roller)
            && matches!(self.state, State::Charge { .. })
        {
            self.chain = 0;
            self.state = State::Rest {
                time: self.sp().wall_rest,
            };
        }
    }

    /// 체력이 반 아래(보스가 한 수 세진다).
    fn hurt_half(&self) -> bool {
        self.hp * 2 <= self.max_hp
    }

    /// 보스 2단계로 깨어난다(0233, 한 번): 예고가 짧아지고(`windup_mul`), 걸음·쉬기·멈칫은 `roused`를 보는 곳에서.
    /// 하던 동작은 그대로 잇는다 — 연출은 월드가 `Event::Rouse`로.
    fn rouse(&mut self, events: &mut Vec<Event>) {
        self.roused = true;
        self.windup_mul *= t::BOSS_P2_WINDUP;
        self.flash = t::HIT_FLASH;
        events.push(Event::Rouse(self.pos));
    }

    /// 공격 동작 중(예고 포함): 보스는 맞아도 멈추지 않는다(슈퍼아머). 피하고 빈틈에 때리는 보스.
    fn armored(&self) -> bool {
        self.is_boss()
            && matches!(
                self.state,
                State::Windup { .. }
                    | State::Charge { .. }
                    | State::Crouch { .. }
                    | State::Leap { .. }
                    | State::Howl { .. }
                    | State::Pop { .. }
                    | State::Aim { .. }
                    | State::Guard { .. }
                    | State::Thump { .. }
                    | State::Gather { .. }
            )
    }

    /// 보스(체력 막대를 화면에 따로 그린다).
    pub fn is_boss(&self) -> bool {
        matches!(
            self.sp().behavior,
            Behavior::Chief | Behavior::Momo | Behavior::Doto | Behavior::Kongi | Behavior::Clover
        )
    }

    /// 플레이어의 타격 `s`를 받는다(0170 R10): 닿나, 튕기나, 맞으면 어느 쪽으로 밀리고 멈추나, 독·벽 쿵까지 — 종 규칙은 여기 한 곳.
    /// `origin`·`facing`은 플레이어 자리·방향(밀 방향을 정한다).
    pub fn receive(&mut self, s: &Strike, origin: Vec2, facing: Vec2) -> Struck {
        if self.last_hit_by == s.id
            || !self.hittable()
            || !s.shape.overlaps_circle(self.pos, self.radius())
            // 나는 박쥐에는 솟구치기가 닿지 않는다(0146).
            || (self.flying() && s.kind == StrikeKind::Emerge)
            // 도토의 던지는 손은 적을 때리지 않는다 — 탄이 맞힌다(0195).
            || s.kind == StrikeKind::Toss
            // 나는 박쥐는 바닥 함정(플레이어가 깐 가시 덤불·묻은 도토리)에도 안 닿는다(0258).
            || (self.flying() && matches!(s.kind, StrikeKind::Prick | StrikeKind::Burst))
        {
            return Struck::Miss;
        }
        self.last_hit_by = s.id;
        // 회피의 밀치기(0214): 밀기만 — 피해·번쩍·경직이 없어 하던 동작(예고·돌진)은 그대로 이어진다. 보스는 덜 밀린다.
        if s.kind == StrikeKind::Shove {
            let dir = (self.pos - origin).normalize_or(facing);
            let boss = if self.is_boss() { t::SHOVE_BOSS } else { 1.0 };
            self.vel = dir * s.knockback * self.sp().knock_mul * boss;
            return Struck::Shoved { dir };
        }
        // 콩이의 등딱지(0054): 발톱은 튕긴다(패링 거울). 솟구치기(아래 배)·드릴은 뚫고 연다.
        // 강공격은 튕기지 않고 자세를 깬다(0168: 모으기 = 자세 깨기).
        let guarded = self.guarding();
        if guarded && matches!(s.kind, StrikeKind::Claw(_) | StrikeKind::Prick) && !s.crush {
            self.shell_hit();
            return Struck::Deflected;
        }
        let dir = (self.pos - origin).normalize_or_zero();
        let dir = if dir == Vec2::ZERO {
            facing
        } else if s.kind == StrikeKind::Drill {
            // 드릴은 앞으로 밀어 데려간다(옆으로 흩지 않는다).
            (facing * 1.6 + dir * 0.4).normalize()
        } else {
            dir
        };
        self.hurt(s.damage, dir, s.knockback, s.stun);
        let broke_guard = guarded && self.alive;
        if broke_guard {
            self.break_guard();
        }
        if s.poison > 0.0 && self.alive {
            self.apply_poison(s.poison);
        }
        // 벽 쿵(0168): 강공격 넉백으로 벽에 박히면 한 번 더.
        if s.kind == StrikeKind::Heavy && self.alive {
            self.slam = t::WALL_SLAM_DAMAGE;
        }
        Struck::Hit { dir, broke_guard }
    }

    /// 독을 건다. 이미 걸려 있으면 더 긴 쪽.
    pub fn apply_poison(&mut self, secs: f32) {
        if self.poison <= 0.0 {
            self.poison_tick = t::POISON_TICK;
        }
        self.poison = self.poison.max(secs);
    }

    fn tick_poison(&mut self, dt: f32) {
        if self.poison <= 0.0 || !self.alive {
            return;
        }
        self.poison -= dt;
        self.poison_tick -= dt;
        if self.poison_tick <= 0.0 {
            self.poison_tick += t::POISON_TICK;
            self.hp -= 1;
            self.flash = t::HIT_FLASH * 0.5;
            if self.hp <= 0 {
                self.alive = false;
                self.died_of_poison = true;
            }
        }
        if self.poison < 0.0 {
            self.poison = 0.0;
        }
    }

    /// 피격. 넉백 방향은 단위 벡터. 무거운 종은 덜 밀린다. 체공 중엔 떨어진다(그 자리에서 경직).
    /// 대장: 공격 중엔 멈추지 않고, 아닐 때도 짧게만 멈칫한다. 피해 0(패링·밀어내기)은 그대로 멈춘다 — 패링이 보상.
    pub fn hurt(&mut self, dmg: i32, knock_dir: Vec2, knockback: f32, stun: f32) {
        self.asleep = false;
        self.wake_delay = 0.0;
        self.hp -= dmg;
        self.flash = t::HIT_FLASH;
        let elite = if self.elite { t::ELITE_KNOCK } else { 1.0 };
        self.vel = knock_dir * knockback * self.sp().knock_mul * elite;
        if self.is_boss() && dmg > 0 {
            let stun = stun * if self.roused { t::BOSS_P2_STUN } else { 1.0 };
            if !self.armored() {
                let left = match self.state {
                    // 벽에 박고 어질어질한 동안은 그 시간을 지킨다(빈틈을 맞아서 줄이지 않는다).
                    State::Rest { time } => time,
                    _ => 0.0,
                };
                self.state = if left > stun * t::CHIEF_STUN_MUL {
                    State::Rest { time: left }
                } else {
                    State::Stunned {
                        time: stun * t::CHIEF_STUN_MUL,
                    }
                };
            }
        } else {
            self.state = State::Stunned { time: stun };
        }
        if self.hp <= 0 {
            self.alive = false;
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self.state {
            State::Approach => "approach",
            State::Windup { .. } => "windup",
            State::Charge { .. } => "CHARGE",
            State::Rest { .. } => "rest",
            State::Keep => "keep",
            State::Aim { .. } => "aim",
            State::Crouch { .. } => "crouch",
            State::Leap { .. } => "LEAP",
            State::Stunned { .. } => "stunned",
            State::Howl { .. } => "HOWL",
            State::Under { .. } => "under",
            State::Pop { .. } => "POP",
            State::Guard { .. } => "GUARD",
            State::Lash { .. } => "LASH",
            State::Puff { .. } => "PUFF",
            State::Thump { .. } => "THUMP",
            State::Gather { .. } => "gather",
        }
    }
}

fn idle_of(b: Behavior) -> State {
    match b {
        Behavior::Charger
        | Behavior::Chief
        | Behavior::Momo
        | Behavior::Doto
        | Behavior::Kongi
        | Behavior::Clover
        | Behavior::Darter
        | Behavior::Roller => State::Approach,
        Behavior::Shooter
        | Behavior::Leaper
        | Behavior::Tongue
        | Behavior::Swooper
        | Behavior::Spore => State::Keep,
        Behavior::Dummy => State::Rest { time: f32::MAX },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 회피의 밀치기(0214): 피해·번쩍·경직 없이 밀리기만 하고, 하던 공격 예고는 그대로 이어진다.
    #[test]
    fn shove_pushes_without_hurting_or_interrupting() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(60.0, 50.0));
        e.asleep = false;
        e.state = State::Windup { time: 0.1 };
        let hp = e.hp;
        let s = Strike {
            id: 7,
            kind: StrikeKind::Shove,
            shape: crate::combat::Shape::Circle {
                center: vec2(50.0, 50.0),
                r: t::SHOVE_RADIUS,
            },
            damage: 0,
            knockback: t::SHOVE_KNOCK,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        };
        let got = e.receive(&s, vec2(50.0, 50.0), vec2(1.0, 0.0));
        assert!(matches!(got, Struck::Shoved { dir } if dir.x > 0.99));
        assert_eq!(e.hp, hp);
        assert_eq!(e.flash, 0.0);
        assert_eq!(e.state, State::Windup { time: 0.1 }, "예고를 끊지 않는다");
        assert!(e.vel.x > 0.0, "밀린다");
    }

    /// 공격 예고의 진행(0179): 예고 시작 0, 끝 1, 공격 중·서 있을 때는 없음.
    #[test]
    fn telegraph_progress_follows_the_windup() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(50.0, 50.0));
        assert_eq!(e.telegraph(), None);
        e.state = State::Windup { time: 0.0 };
        assert_eq!(e.telegraph(), Some(0.0));
        e.state = State::Windup { time: e.windup() };
        assert_eq!(e.telegraph(), Some(1.0));
        // 다 찬 그림은 공격 `TELE_FULL_HOLD`초 전부터 보인다(0188): 그 직전까진 고르게 차오르고, 그 뒤로는 1.
        let lead = t::TELE_FULL_HOLD.min(e.windup() * 0.25);
        e.state = State::Crouch {
            time: (e.windup() - lead) * 0.5,
            target: Vec2::ZERO,
        };
        assert!((e.telegraph().unwrap() - 0.5).abs() < 1e-5);
        e.state = State::Windup {
            time: e.windup() - lead * 0.5,
        };
        assert_eq!(e.telegraph(), Some(1.0), "공격 직전엔 다 찬 채로");
        e.state = State::Windup {
            time: e.windup() - lead - 0.02,
        };
        assert!(e.telegraph().unwrap() < 1.0, "그 전엔 아직 덜 찼다");
        e.state = State::Aim { time: 0.0 };
        assert_eq!(e.telegraph(), Some(0.0));
        e.state = State::Charge {
            time: 0.0,
            dir: vec2(1.0, 0.0),
        };
        assert_eq!(e.telegraph(), None, "공격 중엔 예고 없음");
    }

    fn strike(kind: StrikeKind, at: Vec2, damage: i32) -> Strike {
        Strike {
            id: 7,
            kind,
            shape: combat::Shape::Circle {
                center: at,
                r: 12.0,
            },
            damage,
            knockback: 60.0,
            stun: 0.3,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        }
    }

    /// 조약돌 덧신(0246): `crush`가 붙은 발차기는 등딱지에 튕기지 않고 연다. 가시 덤불(`Prick`)은 튕긴다.
    #[test]
    fn crushing_kicks_open_a_guard_and_pricks_bounce() {
        let at = vec2(100.0, 100.0);
        let mut e = Enemy::new(Kind::Kongi, at);
        e.state = State::Guard { time: 1.0 };
        let s = Strike {
            crush: true,
            ..strike(StrikeKind::Claw(1), at, 3)
        };
        let r = e.receive(&s, at - vec2(10.0, 0.0), vec2(1.0, 0.0));
        assert!(
            matches!(
                r,
                Struck::Hit {
                    broke_guard: true,
                    ..
                }
            ),
            "{r:?}"
        );
        let mut e = Enemy::new(Kind::Kongi, at);
        e.state = State::Guard { time: 1.0 };
        let r = e.receive(
            &strike(StrikeKind::Prick, at, 1),
            at - vec2(10.0, 0.0),
            vec2(1.0, 0.0),
        );
        assert_eq!(r, Struck::Deflected);
    }

    /// 등딱지(0054·0168): 버티는 콩이에게 발톱은 튕기고(피해 없음), 강공격은 자세를 깨고 들어간다(피해 + 긴 경직).
    #[test]
    fn guard_deflects_claws_and_breaks_under_a_heavy() {
        let at = vec2(100.0, 100.0);
        let mut e = Enemy::new(Kind::Kongi, at);
        e.state = State::Guard { time: 1.0 };
        let hp = e.hp;
        let r = e.receive(
            &strike(StrikeKind::Claw(0), at, 2),
            at - vec2(10.0, 0.0),
            vec2(1.0, 0.0),
        );
        assert_eq!(r, Struck::Deflected);
        assert_eq!(e.hp, hp, "튕기면 피해 없음");
        let mut e = Enemy::new(Kind::Kongi, at);
        e.state = State::Guard { time: 1.0 };
        let r = e.receive(
            &strike(StrikeKind::Heavy, at, 3),
            at - vec2(10.0, 0.0),
            vec2(1.0, 0.0),
        );
        assert!(
            matches!(
                r,
                Struck::Hit {
                    broke_guard: true,
                    ..
                }
            ),
            "{r:?}"
        );
        assert_eq!(e.hp, hp - 3);
        assert!(
            matches!(e.state, State::Stunned { .. }),
            "등딱지가 열리면 어질어질"
        );
        assert_eq!(e.slam, t::WALL_SLAM_DAMAGE, "강공격은 벽 쿵을 건다");
    }

    /// 같은 스윙은 같은 적을 두 번 때리지 않고, 민 방향은 플레이어에서 적 쪽이다.
    #[test]
    fn one_swing_hits_once_and_pushes_away_from_the_player() {
        let at = vec2(100.0, 100.0);
        let mut e = Enemy::new(Kind::Kongi, at);
        let s = strike(StrikeKind::Claw(1), at, 1);
        match e.receive(&s, at - vec2(10.0, 0.0), vec2(0.0, 1.0)) {
            Struck::Hit { dir, broke_guard } => {
                assert!(!broke_guard);
                assert!(dir.x > 0.99, "{dir}");
            }
            r => panic!("{r:?}"),
        }
        assert_eq!(e.receive(&s, at, vec2(1.0, 0.0)), Struck::Miss);
    }

    /// 나는 박쥐(0146)에는 솟구치기가 닿지 않고 발톱은 닿는다.
    #[test]
    fn bats_dodge_the_emerge_but_not_claws() {
        let at = vec2(100.0, 100.0);
        let mut e = Enemy::new(Kind::Bat, at);
        assert!(e.flying());
        assert_eq!(
            e.receive(&strike(StrikeKind::Emerge, at, 2), at, vec2(1.0, 0.0)),
            Struck::Miss
        );
        assert!(matches!(
            e.receive(&strike(StrikeKind::Claw(0), at, 1), at, vec2(1.0, 0.0)),
            Struck::Hit { .. }
        ));
    }

    /// 벽 쿵(0168): 강공격 넉백(`slam`)으로 빠르게 날아가다 벽에 박히면 피해 +1·튕김·`slammed`. 느리거나 `slam`이 없으면 그냥 멈춘다.
    #[test]
    fn heavy_knockback_slams_into_the_wall_for_extra_damage() {
        let room = Room::new();
        let mut pr = Vec::new();
        let mut ev = Vec::new();
        let left = vec2(-1.0, 0.0);
        let make = |speed: f32, slam: i32| {
            let mut e = Enemy::new(Kind::FieldMouse, vec2(room.inner.x + 12.0, 100.0));
            e.hurt(0, left, speed, 0.5);
            e.slam = slam;
            e
        };
        let mut e = make(400.0, 1);
        let hp = e.hp;
        for _ in 0..6 {
            e.update(
                vec2(300.0, 100.0),
                true,
                &room,
                t::TICK_DT,
                &mut pr,
                &mut ev,
            );
        }
        assert_eq!(e.hp, hp - 1, "벽에 박히면 한 번 더 아프다");
        assert!(
            e.slammed && e.slam == 0,
            "월드가 소리·파편을 낼 표시, 한 번만"
        );
        assert!(e.vel.x > 0.0, "벽에서 튕긴다 {:?}", e.vel);
        let mut e = make(400.0, 0);
        let hp = e.hp;
        for _ in 0..6 {
            e.update(
                vec2(300.0, 100.0),
                true,
                &room,
                t::TICK_DT,
                &mut pr,
                &mut ev,
            );
        }
        assert_eq!(e.hp, hp, "평타 넉백은 벽 쿵이 없다");
        let mut e = make(60.0, 1);
        let hp = e.hp;
        for _ in 0..6 {
            e.update(
                vec2(300.0, 100.0),
                true,
                &room,
                t::TICK_DT,
                &mut pr,
                &mut ev,
            );
        }
        assert_eq!(e.hp, hp, "느리게 닿으면 쿵이 아니다");
    }

    /// 토끼풀(0165): 발 구르기(Thump 이벤트) → 높이 뛰기 → 가까우면 두 번 차기(첫 발을 받아치면 끊김), 반 아래면 한 번 품기
    /// (실 한 가닥 Thread 이벤트). 멀면 고치 앞 자리로 간다.
    #[test]
    fn clover_thumps_leaps_double_kicks_and_gathers_once() {
        let room = Room::new();
        let mut ev = Vec::new();
        let mut pr = Vec::new();
        let cocoon = vec2(160.0, 40.0);
        let mut e = Enemy::new(Kind::Clover, vec2(160.0, 70.0));
        e.guard = Some(cocoon);
        let near = vec2(160.0, 100.0);
        let far = vec2(160.0, 170.0);
        let mut tick = |e: &mut Enemy, p: Vec2, ev: &mut Vec<Event>| {
            e.update(p, true, &room, t::TICK_DT, &mut pr, ev);
        };
        // 1. 발 구르기: 예고 뒤 Thump.
        tick(&mut e, near, &mut ev);
        assert!(matches!(e.state, State::Thump { .. }), "{:?}", e.state);
        for _ in 0..((t::CLOVER_THUMP_WINDUP / t::TICK_DT) as usize + 2) {
            tick(&mut e, near, &mut ev);
        }
        assert!(ev.iter().any(|x| matches!(x, Event::Thump(_))), "쿵");
        while matches!(e.state, State::Rest { .. }) {
            tick(&mut e, near, &mut ev);
        }
        tick(&mut e, near, &mut ev);
        // 2. 높이 뛰기.
        assert!(matches!(e.state, State::Crouch { .. }), "{:?}", e.state);
        let mut n = 0;
        while !matches!(e.state, State::Approach) && n < 400 {
            tick(&mut e, near, &mut ev);
            n += 1;
        }
        // 3. 가까우면 두 번 차기: 첫 차기가 끝나면 둘째 예고로.
        let mut n = 0;
        while !matches!(e.state, State::Windup { .. }) && n < 400 {
            tick(&mut e, near, &mut ev);
            n += 1;
        }
        assert!(
            matches!(e.state, State::Windup { .. }),
            "차기 예고 {:?}",
            e.state
        );
        assert_eq!(e.chain, t::CLOVER_KICKS - 1);
        let mut n = 0;
        while !matches!(e.state, State::Charge { .. }) && n < 100 {
            tick(&mut e, near, &mut ev);
            n += 1;
        }
        assert!(e.parryable(), "차기는 받아칠 수 있다(황금)");
        let mut n = 0;
        while matches!(e.state, State::Charge { .. }) && n < 100 {
            tick(&mut e, near, &mut ev);
            n += 1;
        }
        assert!(
            matches!(e.state, State::Windup { .. }),
            "둘째 차기 {:?}",
            e.state
        );
        assert_eq!(e.chain, 0);
        // 받아치면 남은 차기가 끊긴다.
        e.chain = 1;
        e.on_parried();
        assert_eq!(e.chain, 0);
        // 4. 멀면 고치 앞 자리로(플레이어가 아니라).
        e.state = State::Approach;
        e.pattern = 2;
        let before = e.pos;
        for _ in 0..30 {
            tick(&mut e, far, &mut ev);
        }
        assert!(e.pos != before, "자리로 간다");
        assert!(
            (e.pos - cocoon).length() < (far - cocoon).length(),
            "고치와 플레이어 사이"
        );
        // 5. 반 아래면 한 번 품기: 1초에 실 한 가닥.
        e.hp = e.max_hp / 2;
        e.state = State::Approach;
        ev.clear();
        tick(&mut e, near, &mut ev);
        assert!(matches!(e.state, State::Gather { .. }), "{:?}", e.state);
        for _ in 0..((t::CLOVER_THREAD_AT / t::TICK_DT) as usize + 2) {
            tick(&mut e, near, &mut ev);
        }
        assert!(
            ev.iter().any(|x| matches!(x, Event::Thread(..))),
            "실 한 가닥"
        );
        for _ in 0..((t::CLOVER_GATHER_TIME / t::TICK_DT) as usize + 2) {
            tick(&mut e, near, &mut ev);
        }
        while matches!(e.state, State::Rest { .. }) {
            tick(&mut e, near, &mut ev);
        }
        assert!(!matches!(e.state, State::Gather { .. }), "품기는 한 번");
    }

    /// 받아치기 규칙(0160): 곧게 오는 돌진만, 내려앉기는 안 된다.
    #[test]
    fn only_straight_charges_are_parryable() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(100.0, 100.0));
        e.state = State::Charge {
            time: 0.0,
            dir: vec2(1.0, 0.0),
        };
        assert!(e.parryable());
        let mut l = Enemy::new(Kind::RedSquirrel, vec2(100.0, 100.0));
        l.state = State::Leap {
            time: l.sp().dash_time,
            from: vec2(80.0, 100.0),
            to: vec2(120.0, 100.0),
        };
        assert!(l.contact_damage().is_some(), "내려앉는 순간은 아프다");
        assert!(!l.parryable(), "하지만 받아칠 수는 없다");
    }

    const FIELD_HP: i32 = species::FIELD_MOUSE.hp;

    /// 청설모: 웅크렸다가 뛰고, 내려앉는 순간에만 둘레 피해.
    #[test]
    fn squirrel_crouches_leaps_and_lands() {
        let room = Room::new();
        let mut e = Enemy::new(Kind::RedSquirrel, room.center());
        e.fire_cd = 0.0;
        let player = room.center() + vec2(70.0, 0.0);
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut saw = (false, false, false);
        for _ in 0..240 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            match e.state {
                State::Crouch { .. } => saw.0 = true,
                State::Leap { .. } => {
                    saw.1 = true;
                    if e.contact_damage().is_some() {
                        saw.2 = true;
                        assert!(e.contact_radius() > e.radius());
                    }
                }
                _ => assert!(e.contact_damage().is_none()),
            }
        }
        assert!(saw.0 && saw.1 && saw.2, "{saw:?}");
        assert!(ev.contains(&Event::Land));
    }

    /// 대장: 돌진과 뛰어 찍기를 번갈아 하고, 공격 중엔 맞아도 멈추지 않으며, 체력이 반 아래면 부하를 부른다.
    #[test]
    fn chief_alternates_armors_and_summons() {
        let room = Room::new();
        let mut e = Enemy::new(Kind::Chief, room.center());
        let player = room.center() + vec2(60.0, 0.0);
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut saw = (false, false);
        for _ in 0..600 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            match e.state {
                State::Windup { .. } => {
                    if !saw.0 {
                        e.hurt(1, vec2(1.0, 0.0), 100.0, 0.3);
                        assert!(
                            matches!(e.state, State::Windup { .. }),
                            "예고 중엔 안 끊긴다"
                        );
                    }
                    saw.0 = true;
                }
                State::Crouch { .. } => saw.1 = true,
                _ => {}
            }
        }
        assert!(saw.0 && saw.1, "{saw:?}");
        assert!(!ev.contains(&Event::Summon), "체력이 많으면 부르지 않는다");
        e.hp = e.sp().hp / 2;
        e.alive = true;
        for _ in 0..1800 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
        }
        let n = ev.iter().filter(|x| **x == Event::Summon).count();
        assert!(n >= 1 && n as u32 <= t::CHIEF_SUMMONS, "{n}");
    }

    /// 보스 2단계(0233): 반 아래로 내려간 다음 틱에 한 번 깨어난다 — 예고가 짧아지고, 걸음이 빨라지고, 덜 멈칫한다.
    /// 깨어나는 소식은 한 번만, 잡몹은 깨어나지 않는다.
    #[test]
    fn bosses_rouse_once_below_half() {
        let room = Room::new();
        let player = room.center() + vec2(200.0, 0.0);
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        for kind in [
            Kind::Chief,
            Kind::Momo,
            Kind::Doto,
            Kind::Kongi,
            Kind::Clover,
        ] {
            let mut e = Enemy::new(kind, room.center());
            let (windup, speed) = (e.windup(), e.speed());
            e.hurt(1, vec2(1.0, 0.0), 0.0, 0.3);
            let calm = match e.state {
                State::Stunned { time } => time,
                _ => panic!("{kind:?}: 빈틈에 맞으면 멈칫"),
            };
            ev.clear();
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            assert!(!e.roused && !ev.iter().any(|x| matches!(x, Event::Rouse(_))));
            e.hp = e.max_hp / 2;
            e.state = State::Approach;
            for _ in 0..3 {
                e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            }
            let n = ev.iter().filter(|x| matches!(x, Event::Rouse(_))).count();
            assert_eq!(n, 1, "{kind:?}: 한 번만 깨어난다");
            assert!(
                e.roused && e.windup() < windup && e.speed() > speed,
                "{kind:?}"
            );
            e.state = State::Approach;
            e.hurt(1, vec2(1.0, 0.0), 0.0, 0.3);
            match e.state {
                State::Stunned { time } => assert!(time < calm, "{kind:?}: 덜 멈칫"),
                _ => panic!("{kind:?}"),
            }
        }
        let mut rat = Enemy::new(Kind::FieldMouse, room.center());
        rat.hp = 1;
        rat.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
        assert!(!rat.roused, "잡몹은 깨어나지 않는다");
    }

    /// 모모: 1번째는 뛰기·차기만, 2번째부터 굴로 숨는다(숨은 동안 안 맞는다). 3번째는 뛰기를 이어서 한다.
    #[test]
    fn momo_learns_one_move_per_fight() {
        let room = Room::new();
        let player = room.center() + vec2(50.0, 0.0);
        let run = |level: u8| {
            let mut e = Enemy::new(Kind::Momo, room.center());
            e.level = level;
            let (mut pr, mut ev) = (Vec::new(), Vec::new());
            let (mut dug, mut kicked, mut lands) = (false, false, 0);
            for _ in 0..1200 {
                e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
                match e.state {
                    State::Under { .. } => {
                        dug = true;
                        assert!(!e.hittable());
                        assert!(e.contact_damage().is_none());
                    }
                    State::Charge { .. } => kicked = true,
                    _ => {}
                }
            }
            lands += ev.iter().filter(|x| **x == Event::Land).count();
            (dug, kicked, lands, ev.contains(&Event::Pop))
        };
        let (dug, kicked, _, _) = run(1);
        assert!(!dug && kicked);
        let (dug, _, l2, popped) = run(2);
        assert!(dug && popped);
        let (_, _, l3, _) = run(3);
        assert!(l3 > l2, "세 번째는 더 자주 내려앉는다: {l2} → {l3}");
    }

    /// 도토: 던지기(부채꼴 셋) → 뛰기 → 묻기를 돌고, 체력이 반 아래면 다섯을 던지고 넷을 묻는다.
    #[test]
    fn doto_throws_fans_leaps_and_buries() {
        let room = Room::new();
        let player = room.center() + vec2(70.0, 0.0);
        let run = |hp: i32| {
            let mut e = Enemy::new(Kind::Doto, room.center());
            e.hp = hp;
            let (mut pr, mut ev) = (Vec::new(), Vec::new());
            let mut most = 0;
            for _ in 0..900 {
                let before = pr.len();
                e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
                most = most.max(pr.len() - before);
                pr.clear();
            }
            (most, ev)
        };
        let (fan, ev) = run(species::DOTO.hp);
        assert_eq!(fan, t::DOTO_FAN);
        assert!(ev.contains(&Event::Land));
        assert!(ev.contains(&Event::Bury(t::DOTO_MINES)));
        let (fan, ev) = run(species::DOTO.hp / 2);
        assert_eq!(fan, t::DOTO_FAN_HURT);
        assert!(ev.contains(&Event::Bury(t::DOTO_MINES_HURT)));
    }

    /// 콩이: 구르면 벽에 튕기고(튕긴 수만큼 WallHit), 다 튕기면 어질어질. 버티는 동안은 말려 있고, 치면 바로 굴러 온다.
    #[test]
    fn kongi_rolls_bounces_and_guards() {
        let room = Room::new();
        let player = room.center() + vec2(60.0, 0.0);
        let mut e = Enemy::new(Kind::Kongi, room.center());
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let (mut rolled, mut guarded, mut dizzy) = (false, false, false);
        for _ in 0..900 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            match e.state {
                State::Charge { .. } => {
                    rolled = true;
                    assert!(e.guarding() && e.contact_damage().is_some());
                }
                State::Guard { .. } => {
                    guarded = true;
                    assert!(e.guarding() && e.contact_damage().is_none());
                }
                State::Rest { .. } if rolled => {
                    dizzy = true;
                    assert!(!e.guarding(), "어질어질할 때가 틈");
                }
                _ => {}
            }
        }
        assert!(rolled && guarded && dizzy, "{rolled} {guarded} {dizzy}");
        let walls = ev.iter().filter(|x| **x == Event::WallHit).count();
        assert!(walls > t::KONGI_BOUNCES as usize, "튕김 {walls}");
        // 버티는 중에 치면 바로 굴러 올 채비(짧은 예고).
        let mut g = Enemy::new(Kind::Kongi, room.center());
        g.state = State::Guard { time: 0.2 };
        g.shell_hit();
        assert!(matches!(g.state, State::Windup { time } if time > 0.0));
        g.break_guard();
        assert!(matches!(g.state, State::Stunned { .. }) && !g.guarding());
    }

    /// 엘리트(0147): 체력 셋 배, 예고가 짧고 덜 밀린다. 실뜨기는 그 위에 곱한다.
    #[test]
    fn elite_is_tougher_and_knots_stack() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        let plain = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        e.make_elite();
        assert_eq!(e.max_hp, (plain.max_hp as f32 * t::ELITE_HP).ceil() as i32);
        assert!(e.windup() < plain.windup());
        let before = e.windup();
        e.apply_knots(crate::knots::Knots::new(2), 1.0);
        assert!(
            e.windup() <= before,
            "실뜨기가 엘리트 예고를 덮어쓰지 않는다"
        );
        let mut p = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        let mut q = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        q.make_elite();
        p.hurt(0, vec2(1.0, 0.0), 100.0, 0.1);
        q.hurt(0, vec2(1.0, 0.0), 100.0, 0.1);
        assert!(q.vel.length() < p.vel.length());
    }

    /// 땃쥐(0146): 한 번 떨고 두 번 연달아 돌진한 뒤에야 쉰다.
    #[test]
    fn shrew_bites_twice_then_rests() {
        let room = Room::new();
        let player = room.center() + vec2(40.0, 0.0);
        let mut e = Enemy::new(Kind::Shrew, room.center());
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let (mut dashes, mut was_charging) = (0, false);
        for _ in 0..600 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            let charging = matches!(e.state, State::Charge { .. });
            if charging && !was_charging {
                dashes += 1;
            }
            was_charging = charging;
            if matches!(e.state, State::Rest { .. }) && dashes > 0 {
                break;
            }
        }
        assert_eq!(dashes, t::SHREW_BITES as usize);
    }

    /// 고슴도치(0146): 말려 구르는 동안 발톱이 튕기고, 벽에 박으면 어질어질(틈). 패링해도 멈춘다.
    #[test]
    fn hedgehog_rolls_guarded_and_is_dizzy_after_wall() {
        let room = Room::new();
        let player = room.center() + vec2(80.0, 0.0);
        let mut e = Enemy::new(Kind::Hedgehog, room.center());
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let (mut rolled, mut dizzy) = (false, false);
        for _ in 0..600 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            match e.state {
                State::Charge { .. } => {
                    rolled = true;
                    assert!(e.guarding());
                }
                State::Rest { .. } if rolled => {
                    dizzy = true;
                    assert!(!e.guarding());
                }
                _ => {}
            }
        }
        assert!(rolled && dizzy && ev.contains(&Event::WallHit));
        let mut g = Enemy::new(Kind::Hedgehog, room.center());
        g.state = State::Charge {
            time: 0.1,
            dir: vec2(1.0, 0.0),
        };
        g.on_parried();
        assert!(matches!(g.state, State::Rest { .. }));
    }

    /// 두꺼비(0146): 거리를 두고 겨누다가 플레이어 쪽으로 혀를 내민다. 혀를 내민 동안은 몸으로 아프지 않다.
    #[test]
    fn toad_lashes_toward_player() {
        let room = Room::new();
        let player = room.center() + vec2(50.0, 0.0);
        let mut e = Enemy::new(Kind::Toad, room.center());
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut lashed = None;
        for _ in 0..600 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            if let State::Lash { time: 0.0, dir } = e.state {
                lashed = Some(dir);
                break;
            }
        }
        let dir = lashed.expect("혀를 내민다");
        assert!(dir.dot((player - e.pos).normalize()) > 0.95);
        assert!(ev.contains(&Event::Tongue) && pr.is_empty());
    }

    /// 예고 중 겨누기(0254): 플레이어가 옆으로 확 돌아도 겨눈 쪽은 가속해 따라가고 최대 빠르기를 넘지 않는다.
    #[test]
    fn windup_aim_turns_with_acceleration_and_a_top_speed() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        e.facing = vec2(1.0, 0.0);
        let want = vec2(0.0, 1.0);
        let dt = t::TICK_DT;
        e.aim_toward(want, dt, dt);
        let first = e.facing.y.atan2(e.facing.x);
        assert!(
            first > 0.0 && first < t::ENEMY_TURN_MAX * dt,
            "첫 틱은 아주 조금 {first}"
        );
        let mut time = dt;
        while time < 0.45 {
            time += dt;
            e.aim_toward(want, time, dt);
        }
        let turned = e.facing.y.atan2(e.facing.x);
        let most = t::ENEMY_TURN_MAX * 0.45;
        assert!(
            turned < most && turned > 0.3,
            "0.45초에 {turned} rad(최대 {most})"
        );
        // 충분히 지나면 다다른다.
        for _ in 0..120 {
            time += dt;
            e.aim_toward(want, time, dt);
        }
        assert!(e.facing.dot(want) > 0.999);
    }

    /// 두꺼비 혀(0253): 내민 순간엔 길이 0이라 안 맞고, 혀끝이 자라 닿아야 맞는다. 다 뻗은 뒤 말려 들어갈 때는 안 맞는다.
    #[test]
    fn toad_tongue_hits_only_once_it_has_grown_to_the_player() {
        let room = Room::new();
        let mut e = Enemy::new(Kind::Toad, room.center());
        let dir = vec2(1.0, 0.0);
        let near = e.pos + dir * (e.radius() + 20.0);
        let far = e.pos + dir * (e.radius() + 60.0);
        let at = |e: &mut Enemy, time: f32| e.state = State::Lash { time, dir };
        at(&mut e, 0.0);
        assert!(!e.tongue_reaches(&room, far, 3.0) && !e.tongue_reaches(&room, near, 3.0));
        at(&mut e, t::TOAD_EXTEND_TIME * 0.4);
        assert!(
            e.tongue_reaches(&room, near, 3.0),
            "가까운 데는 먼저 닿는다"
        );
        assert!(!e.tongue_reaches(&room, far, 3.0), "먼 데는 아직");
        at(&mut e, t::TOAD_EXTEND_TIME);
        assert!(e.tongue_reaches(&room, far, 3.0), "다 뻗으면 끝까지");
        at(&mut e, t::TOAD_EXTEND_TIME + t::TOAD_LASH_TIME * 0.3);
        assert!(
            !e.tongue_reaches(&room, near, 3.0),
            "말려 들어갈 때는 안 맞는다"
        );
        assert!(
            tongue_out(t::TOAD_EXTEND_TIME * 0.5) > 0.5 && tongue_out(t::TOAD_EXTEND_TIME) == 1.0
        );
    }

    /// 박쥐(0146): 늘 떠 있고(솟구치기 안 닿음), 내리꽂을 때 낮아진다.
    #[test]
    fn bat_flies_and_dives_lower() {
        let room = Room::new();
        let player = room.center() + vec2(60.0, 0.0);
        let mut e = Enemy::new(Kind::Bat, room.center());
        assert!(e.flying() && e.height() >= t::BAT_HEIGHT);
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut dove = false;
        for _ in 0..600 {
            e.update(player, true, &room, t::TICK_DT, &mut pr, &mut ev);
            if matches!(e.state, State::Charge { .. }) {
                dove = true;
                assert!(e.height() < t::BAT_HEIGHT && e.contact_damage().is_some());
            }
        }
        assert!(dove);
    }

    /// 포자버섯(0146): 멀면 가만히, 가까이 오면 제자리에서 뿜는다. 움직이지 않는다.
    #[test]
    fn spore_cap_puffs_only_when_near() {
        let room = Room::new();
        let at = room.center();
        let (mut pr, mut ev) = (Vec::new(), Vec::new());
        let mut far = Enemy::new(Kind::SporeCap, at);
        for _ in 0..600 {
            far.update(
                at + vec2(120.0, 0.0),
                true,
                &room,
                t::TICK_DT,
                &mut pr,
                &mut ev,
            );
        }
        assert!(!ev.contains(&Event::Spore));
        let mut near = Enemy::new(Kind::SporeCap, at);
        for _ in 0..600 {
            near.update(
                at + vec2(30.0, 0.0),
                true,
                &room,
                t::TICK_DT,
                &mut pr,
                &mut ev,
            );
        }
        assert!(ev.contains(&Event::Spore));
        assert!((near.pos - at).length() < 0.5);
    }

    #[test]
    fn poison_ticks_down_hp_and_kills() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(100.0, 100.0));
        assert_eq!(e.hp, FIELD_HP);
        e.apply_poison(t::VENOM_POISON_FINISHER);
        let dt = t::TICK_DT;
        let mut secs = 0.0;
        while e.alive && secs < 10.0 {
            e.tick_poison(dt);
            secs += dt;
        }
        // 2.5초 독 = 0.8초마다 1 → 3틱, 체력 3인 돌격형은 쓰러진다.
        assert!(!e.alive);
        assert!(e.died_of_poison);
        assert!(secs <= t::VENOM_POISON_FINISHER + dt);
    }

    #[test]
    fn short_poison_deals_one_tick() {
        let mut e = Enemy::new(Kind::FieldMouse, vec2(0.0, 0.0));
        e.apply_poison(t::VENOM_POISON);
        for _ in 0..600 {
            e.tick_poison(t::TICK_DT);
        }
        assert_eq!(e.hp, FIELD_HP - 1);
        assert_eq!(e.poison, 0.0);
        assert!(e.alive);
    }
}

/// 예고 진행(0..1): 공격 `TELE_FULL_HOLD`초 전에 1이 되어 그대로 남는다(0188). 적·떨어지는 돌·할매 당기기가 같이 쓴다.
pub fn tele_progress(time: f32, dur: f32) -> f32 {
    let lead = t::TELE_FULL_HOLD.min(dur * 0.25);
    (time / (dur - lead).max(1e-3)).clamp(0.0, 1.0)
}
