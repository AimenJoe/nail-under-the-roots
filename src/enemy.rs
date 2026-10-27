//! 적: 뿌리에 물든 땅속 이웃. 종류(`Kind`)마다 행동 하나와 수치가 `species.rs` 표에 있다.
//! 행동: 돌격(쥐들), 던지기(다람쥐), 뛰어 덮치기(청설모), 보스 넷(대장·모모·도토·콩이). 플레이어 위치와 `Room`만 본다.

use macroquad::prelude::*;

use crate::combat;
use crate::room::Room;
use crate::species::{self, Behavior, Species};
use crate::tuning as t;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
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
}

pub struct Enemy {
    pub kind: Kind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
    pub hp: i32,
    /// 처음 체력(실뜨기가 바꾼다). 막대·반 아래 판정은 이걸 본다.
    pub max_hp: i32,
    /// 예고 시간 배율(실뜨기 2단계)·걷기 배율(실뜨기 6단계).
    pub windup_mul: f32,
    pub speed_mul: f32,
    /// 실뜨기를 이미 걸었나(월드가 새로 나온 적에 한 번).
    pub knotted: bool,
    pub state: State,
    pub facing: Vec2,
    pub flash: f32,
    pub alive: bool,
    /// 마지막으로 맞은 스윙 번호. 같은 스윙에 두 번 맞지 않는다.
    pub last_hit_by: u32,
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
    /// 대장·모모: 다음 공격 차례, 대장이 부른 횟수, 모모가 남은 이어 뛰기.
    pattern: u32,
    summons: u32,
    chain: u32,
    /// 보스가 몇 번째 만남인가(모모: 1~3, 한 수씩 는다). 다른 적은 1.
    pub level: u8,
    /// 엘리트(실타래 칸, 0147): 체력 셋 배, 예고가 짧고 덜 밀린다.
    pub elite: bool,
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

impl Enemy {
    /// 예고 시간(종 수치 × 실뜨기 배율).
    pub fn windup(&self) -> f32 {
        self.kind.species().windup * self.windup_mul
    }

    /// 실뜨기 매듭을 건다(한 번만): 체력·예고.
    pub fn apply_knots(&mut self, k: crate::knots::Knots) {
        if self.knotted {
            return;
        }
        self.knotted = true;
        self.max_hp = (self.max_hp as f32 * k.enemy_hp_mul()).ceil() as i32;
        if self.is_boss() {
            self.max_hp = (self.max_hp as f32 * k.boss_hp_mul()).ceil() as i32;
        }
        self.hp = self.max_hp;
        self.windup_mul *= k.windup_mul();
        self.speed_mul *= k.enemy_speed_mul();
    }

    /// 걷기(거리 맞추기) 속도(종 수치 × 실뜨기 배율).
    pub fn speed(&self) -> f32 {
        self.kind.species().speed * self.speed_mul
    }

    pub fn new(kind: Kind, pos: Vec2) -> Self {
        let sp = kind.species();
        let hp = sp.hp;
        let state = idle_of(sp.behavior);
        Self {
            kind,
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
            hp,
            max_hp: hp,
            windup_mul: 1.0,
            speed_mul: 1.0,
            knotted: false,
            state,
            facing: vec2(-1.0, 0.0),
            flash: 0.0,
            alive: true,
            last_hit_by: 0,
            fire_cd: sp.fire_every * 0.5,
            poison: 0.0,
            poison_tick: t::POISON_TICK,
            died_of_poison: false,
            asleep: false,
            wake_delay: 0.0,
            pattern: 0,
            summons: 0,
            chain: 0,
            level: 1,
            elite: false,
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

    /// 받아칠 수 있는 공격인가(0160): 곧게 오는 돌진(물기·구르기·내리꽂기도 `Charge`)만. 내려앉기·튀어나오기는
    /// 둘레 공격이라 안 된다. 예고도 띠·점선(황금) ↔ 고리(진홍, ✕)로 갈린다.
    pub fn parryable(&self) -> bool {
        matches!(self.state, State::Charge { .. })
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
        !matches!(self.state, State::Under { .. })
    }

    /// 날고 있다(박쥐, 0146): 솟구치기·솟구침 밀어내기가 닿지 않는다. 발톱·드릴은 닿는다.
    pub fn flying(&self) -> bool {
        self.sp().behavior == Behavior::Swooper
    }

    /// 체공 높이(그리기용 px). 바닥 = 0. 박쥐는 늘 떠 있고, 내리꽂을 때 낮아진다.
    pub fn height(&self) -> f32 {
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
        }
        if self.asleep || self.wake_delay > 0.0 {
            // 자는 동안은 넉백만.
            self.pos += self.vel * dt;
            combat::apply_friction(&mut self.vel, dt);
            let r = self.radius();
            room.resolve_circle(&mut self.pos, r);
            return;
        }

        match self.state {
            State::Stunned { time } => {
                let time = time - dt;
                if time <= 0.0 {
                    self.state = self.idle_state();
                } else {
                    self.state = State::Stunned { time };
                }
            }
            State::Approach
                if matches!(
                    sp.behavior,
                    Behavior::Chief | Behavior::Momo | Behavior::Doto | Behavior::Kongi
                ) =>
            {
                if player_targetable {
                    self.facing = dir_to_player;
                    self.state = match sp.behavior {
                        Behavior::Chief => self.chief_next(dist, player_pos),
                        Behavior::Momo => self.momo_next(dist, player_pos, events),
                        Behavior::Kongi => self.kongi_next(dist),
                        _ => self.doto_next(dist, player_pos),
                    };
                    if matches!(self.state, State::Approach) {
                        // 도토는 너무 가까우면 물러난다(던지는 보스).
                        let away = sp.behavior == Behavior::Doto && dist < sp.keep.0;
                        let dir = if away { -dir_to_player } else { dir_to_player };
                        self.pos += dir * self.speed() * dt;
                    }
                }
            }
            State::Howl { time } => {
                let time = time + dt;
                let doto = sp.behavior == Behavior::Doto;
                let dur = self.windup()
                    * if doto {
                        t::DOTO_BURY_MUL
                    } else {
                        t::CHIEF_HOWL_MUL
                    };
                if time >= dur {
                    if doto {
                        events.push(Event::Bury(if self.hurt_half() {
                            t::DOTO_MINES_HURT
                        } else {
                            t::DOTO_MINES
                        }));
                    } else {
                        self.summons += 1;
                        events.push(Event::Summon);
                    }
                    self.state = State::Rest { time: sp.rest };
                } else {
                    self.state = State::Howl { time };
                }
            }
            State::Approach => {
                if player_targetable {
                    self.facing = dir_to_player;
                    if dist < sp.trigger {
                        if sp.behavior == Behavior::Darter {
                            self.chain = t::SHREW_BITES - 1;
                        }
                        self.state = State::Windup { time: 0.0 };
                    } else {
                        self.pos += dir_to_player * self.speed() * dt;
                    }
                }
            }
            State::Windup { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= self.windup() {
                    self.state = State::Charge {
                        time: 0.0,
                        dir: self.facing,
                    };
                    events.push(Event::Charge);
                } else {
                    self.state = State::Windup { time };
                }
            }
            State::Charge { time, dir } => {
                let time = time + dt;
                self.pos += dir * sp.dash_speed * dt;
                let r = self.radius();
                let dur = if sp.behavior == Behavior::Momo {
                    t::MOMO_KICK_TIME
                } else {
                    sp.dash_time
                };
                let before = self.pos;
                let hit_wall = room.resolve_circle(&mut self.pos, r);
                if hit_wall && sp.behavior == Behavior::Kongi && self.chain > 0 {
                    // 콩이: 벽에 튕긴다(반사 + 플레이어 쪽으로 조금 꺾기). 다 튕기면 아래처럼 어질어질.
                    self.chain -= 1;
                    let n = (self.pos - before).normalize_or(-dir);
                    let bounced = dir - 2.0 * dir.dot(n) * n;
                    let aim = (player_pos - self.pos).normalize_or(bounced);
                    let new_dir = (bounced * (1.0 - t::KONGI_BOUNCE_AIM)
                        + aim * t::KONGI_BOUNCE_AIM)
                        .normalize_or(bounced);
                    self.facing = new_dir;
                    events.push(Event::WallHit);
                    self.state = State::Charge { time, dir: new_dir };
                } else if hit_wall {
                    self.state = State::Rest { time: sp.wall_rest };
                    self.flash = t::HIT_FLASH;
                    events.push(Event::WallHit);
                } else if time >= dur && sp.behavior == Behavior::Darter && self.chain > 0 {
                    // 뒤쥐: 곧바로 다시 겨눠 한 번 더 문다(예고는 짧게).
                    self.chain -= 1;
                    self.state = State::Windup {
                        time: self.windup() * (1.0 - t::SHREW_REAIM),
                    };
                } else if time >= dur {
                    self.chain = 0;
                    self.state = State::Rest { time: sp.rest };
                } else {
                    self.state = State::Charge { time, dir };
                }
            }
            State::Rest { time } => {
                let time = time - dt;
                self.state = if time <= 0.0 {
                    idle_of(sp.behavior)
                } else {
                    State::Rest { time }
                };
            }
            State::Keep => {
                if player_targetable {
                    self.facing = dir_to_player;
                    let (near, far) = sp.keep;
                    if dist < near {
                        self.pos -= dir_to_player * self.speed() * dt;
                    } else if dist > far {
                        self.pos += dir_to_player * self.speed() * dt;
                    } else if matches!(sp.behavior, Behavior::Leaper | Behavior::Swooper) {
                        // 청설모·박쥐: 거리 안에서는 옆으로 돈다(가만히 있지 않는다).
                        let side = vec2(-dir_to_player.y, dir_to_player.x);
                        self.pos += side * self.speed() * 0.6 * dt;
                    }
                    // 실버섯은 가까이 와야 부푼다. 다른 것은 늘 센다.
                    if sp.behavior != Behavior::Spore || dist < sp.trigger {
                        self.fire_cd -= dt;
                    }
                    if self.fire_cd <= 0.0 {
                        self.state = match sp.behavior {
                            Behavior::Leaper => State::Crouch {
                                time: 0.0,
                                target: player_pos,
                            },
                            Behavior::Swooper => State::Windup { time: 0.0 },
                            _ => State::Aim { time: 0.0 },
                        };
                    }
                }
            }
            State::Crouch { time, target } => {
                let time = time + dt;
                self.facing = (target - self.pos).normalize_or(self.facing);
                if time >= self.windup() {
                    // 너무 멀면 갈 수 있는 데까지만.
                    let reach = (target - self.pos).clamp_length_max(sp.trigger * 1.2);
                    self.state = State::Leap {
                        time: 0.0,
                        from: self.pos,
                        to: self.pos + reach,
                    };
                    events.push(Event::Leap);
                } else {
                    self.state = State::Crouch { time, target };
                }
            }
            State::Leap { time, from, to } => {
                let time = time + dt;
                let k = (time / sp.dash_time).min(1.0);
                self.pos = from + (to - from) * k;
                if time >= sp.dash_time {
                    events.push(Event::Land);
                    self.fire_cd = sp.fire_every;
                    self.state = if self.chain > 0 && player_targetable {
                        // 모모 세 번째: 내려앉자마자 다시 웅크린다(예고는 짧게).
                        self.chain -= 1;
                        State::Crouch {
                            time: self.windup() * t::MOMO_CHAIN_SKIP,
                            target: player_pos,
                        }
                    } else {
                        self.chain = 0;
                        State::Rest { time: sp.rest }
                    };
                } else {
                    self.state = State::Leap { time, from, to };
                }
            }
            State::Under { time } => {
                let time = time + dt;
                if time < t::MOMO_DIG_TRACK {
                    // 둔덕이 플레이어를 따라온다(걸음보다 느리게).
                    if player_targetable {
                        let step = t::MOMO_DIG_SPEED * dt;
                        self.pos += to_player.clamp_length_max(step);
                    }
                    self.state = State::Under { time };
                } else if time >= t::MOMO_DIG_TRACK + self.windup() {
                    events.push(Event::Pop);
                    self.state = State::Pop { time: 0.0 };
                } else {
                    self.state = State::Under { time };
                }
            }
            State::Guard { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                self.state = if time >= t::KONGI_GUARD_TIME {
                    State::Rest {
                        time: sp.rest * 0.5,
                    }
                } else {
                    State::Guard { time }
                };
            }
            State::Pop { time } => {
                let time = time + dt;
                self.state = if time >= t::MOMO_POP_TIME {
                    State::Rest { time: sp.rest }
                } else {
                    State::Pop { time }
                };
            }
            State::Lash { time, dir } => {
                let time = time + dt;
                self.state = if time >= t::TOAD_LASH_TIME {
                    State::Rest { time: sp.rest }
                } else {
                    State::Lash { time, dir }
                };
            }
            State::Puff { time } => {
                let time = time + dt;
                self.state = if time >= t::SPORE_PUFF_TIME {
                    State::Rest { time: sp.rest }
                } else {
                    State::Puff { time }
                };
            }
            State::Aim { time } => {
                let time = time + dt;
                if player_targetable {
                    self.facing = dir_to_player;
                }
                if time >= self.windup() && sp.behavior == Behavior::Tongue {
                    // 두꺼비: 겨눈 쪽으로 혀를 내민다(맞았는지는 월드가 이 틱에 본다).
                    events.push(Event::Tongue);
                    self.fire_cd = sp.fire_every;
                    self.state = State::Lash {
                        time: 0.0,
                        dir: self.facing,
                    };
                } else if time >= self.windup() && sp.behavior == Behavior::Spore {
                    // 포자버섯: 제자리에 포자 구름(월드가 놓는다).
                    events.push(Event::Spore);
                    self.fire_cd = sp.fire_every;
                    self.state = State::Puff { time: 0.0 };
                } else if time >= self.windup() {
                    // 도토: 부채꼴로 여럿(가운데가 플레이어 쪽), 던진 뒤 잠깐 쉰다.
                    let n = match sp.behavior {
                        Behavior::Doto if self.hurt_half() => t::DOTO_FAN_HURT,
                        Behavior::Doto => t::DOTO_FAN,
                        _ => 1,
                    };
                    for i in 0..n {
                        let a = (i as f32 - (n - 1) as f32 * 0.5) * t::DOTO_FAN_SPREAD;
                        let dir = Vec2::from_angle(a).rotate(self.facing);
                        let start = self.pos + dir * (self.radius() + t::PROJECTILE_RADIUS + 1.0);
                        projectiles.push(Projectile {
                            pos: start,
                            prev_pos: start,
                            vel: dir * sp.shot_speed,
                            alive: true,
                            friendly: false,
                        });
                    }
                    events.push(Event::Shoot);
                    self.fire_cd = sp.fire_every;
                    self.state = if sp.behavior == Behavior::Doto {
                        State::Rest { time: sp.rest }
                    } else {
                        State::Keep
                    };
                } else {
                    self.state = State::Aim { time };
                }
            }
        }

        self.pos += self.vel * dt;
        combat::apply_friction(&mut self.vel, dt);
        let r = self.radius();
        room.resolve_circle(&mut self.pos, r);
    }

    fn idle_state(&self) -> State {
        idle_of(self.sp().behavior)
    }

    /// 대장의 다음 수: 체력이 반 아래면 세 번에 한 번 부하를 부르고(두 번까지), 아니면 돌진과 뛰어 찍기를 번갈아.
    /// 멀면 걸어서 다가간다(Approach 그대로).
    fn chief_next(&mut self, dist: f32, player_pos: Vec2) -> State {
        let sp = self.sp();
        let hurt = self.hp * 2 <= self.max_hp;
        if hurt && self.summons < t::CHIEF_SUMMONS && self.pattern % 3 == 2 {
            self.pattern += 1;
            return State::Howl { time: 0.0 };
        }
        if self.pattern.is_multiple_of(2) {
            if dist < sp.trigger {
                self.pattern += 1;
                return State::Windup { time: 0.0 };
            }
        } else if dist < sp.trigger * 1.3 {
            self.pattern += 1;
            return State::Crouch {
                time: 0.0,
                target: player_pos,
            };
        }
        State::Approach
    }

    /// 모모의 다음 수(0046). 만날수록 한 수씩 는다:
    /// 1번째 높이 뛰기 ↔ 뒷발 차기, 2번째 + 굴로 숨기, 3번째 + 뛰기를 세 번 이어서.
    /// 차기는 가까워야, 뛰기는 조금 멀어도 된다. 안 닿으면 걸어서 다가간다.
    fn momo_next(&mut self, dist: f32, player_pos: Vec2, events: &mut Vec<Event>) -> State {
        let sp = self.sp();
        let moves = if self.level >= 2 { 3 } else { 2 };
        match self.pattern % moves {
            0 if dist < sp.trigger * 1.5 => {
                self.pattern += 1;
                self.chain = if self.level >= 3 {
                    t::MOMO_HOP_CHAIN - 1
                } else {
                    0
                };
                State::Crouch {
                    time: 0.0,
                    target: player_pos,
                }
            }
            1 if dist < sp.trigger => {
                self.pattern += 1;
                State::Windup { time: 0.0 }
            }
            2 => {
                self.pattern += 1;
                events.push(Event::Dig);
                State::Under { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    /// 도토의 다음 수(0052): 부채꼴 던지기 → 높이 뛰기 → 도토리 묻기를 돈다.
    /// 던지기는 멀어도(유지 거리 끝까지), 뛰기는 조금 멀어도 된다. 묻기는 거리와 상관없이. 안 되면 거리를 맞춘다.
    fn doto_next(&mut self, dist: f32, player_pos: Vec2) -> State {
        let sp = self.sp();
        match self.pattern % 3 {
            0 if dist < sp.keep.1 * 1.3 => {
                self.pattern += 1;
                State::Aim { time: 0.0 }
            }
            1 if dist < sp.trigger * 1.5 => {
                self.pattern += 1;
                State::Crouch {
                    time: 0.0,
                    target: player_pos,
                }
            }
            2 => {
                self.pattern += 1;
                State::Howl { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    /// 콩이의 다음 수(0054): 구르기 ↔ 웅크려 버티기. 구르기는 거리 안에서, 버티기는 가까이 걸어온 뒤. 아니면 걸어온다.
    fn kongi_next(&mut self, dist: f32) -> State {
        let sp = self.sp();
        match self.pattern % 2 {
            0 if dist < sp.trigger => {
                self.pattern += 1;
                self.chain = self.kongi_bounces();
                State::Windup { time: 0.0 }
            }
            1 if dist < t::KONGI_GUARD_NEAR => {
                self.pattern += 1;
                State::Guard { time: 0.0 }
            }
            _ => State::Approach,
        }
    }

    fn kongi_bounces(&self) -> u32 {
        if self.hurt_half() {
            t::KONGI_BOUNCES_HURT
        } else {
            t::KONGI_BOUNCES
        }
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
            )
    }

    /// 보스(체력 막대를 화면에 따로 그린다).
    pub fn is_boss(&self) -> bool {
        matches!(
            self.sp().behavior,
            Behavior::Chief | Behavior::Momo | Behavior::Doto | Behavior::Kongi
        )
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
        e.apply_knots(crate::knots::Knots::new(2));
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
