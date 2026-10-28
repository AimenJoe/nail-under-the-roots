//! 효과음. 시작할 때 합성한 WAV와 파일 소리(`assets/sfx`, docs/decisions/0081)를 메모리에 올린다.
//!
//! - 파일 소리: `assets/sfx/<이름>_<n>.wav`(CC0, `tools/audio/make_audio.py`가 만든다, 출처는 assets/LICENSES.md).
//!   변형이 여럿이면 바로 전과 다른 것을 고른다(같은 타격음이 기계처럼 반복되지 않게). 파일을 못 읽으면 합성한 소리로.
//! - 합성 소리: 드릴·반딧불 음계·패링처럼 이 게임만의 소리와, 파일이 없을 때의 대신.
//! - 메뉴 소리(UiMove·UiConfirm·UiBack)는 앱이 메뉴 장면에서 한꺼번에 낸다(`App::menu_sound`).

use std::cell::Cell;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

use crate::tuning;

const SR: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Id {
    Claw(usize),
    Burrow,
    Emerge,
    /// 힘을 다 모아 솟구쳤다(0158): 보통 솟구침보다 낮고 무겁게.
    EmergeBig,
    /// 땅속에서 힘을 모으는 동안 땅이 운다(0158). 솟구치면 끊는다.
    Rumble,
    EnemyHit,
    EnemyDie,
    Hurt,
    Die,
    Shoot,
    Charge,
    /// 박자에 맞춘 타(0167 → 0174): 값은 연속으로 맞춘 수(1부터) — 도·미·솔·높은 도로 오르고 되풀이.
    Beat(usize),
    /// 박자가 왔다(0180): 나무 두드리는 짧은 '똑'(메트로놈). 종소리보다 낮고 작다.
    BeatTick,
    /// 모으기 시작(0168), 맞힘 '틱'(0168).
    ChargeUp,
    HitTick,
    WallHit,
    /// 반딧불 줍기. 연달아 주우면 음이 조금씩 올라간다.
    Glowbug,
    /// 회복(먹기·되살아남).
    Heal,
    /// 필살기 드릴: 시동부터 돌진 끝까지 하나("부아아앙"). 벽에 부딪히면 끊는다.
    /// 번호 = 드릴 재주 갈래(0 보통, 1 긴 드릴, 2 짧은 드릴, docs/decisions/0047) — 돌진 길이마다 소리를 따로 만든다.
    Drill(usize),
    /// 드릴이 벽에 쿵.
    DrillWall,
    /// 패링(맑은 금속음).
    Parry,
    /// 메뉴 커서가 움직였다.
    UiMove,
    /// 메뉴에서 골랐다.
    UiConfirm,
    /// 메뉴를 닫았다·뒤로.
    UiBack,
    /// 샀다(씨앗을 냈다).
    Buy,
    /// 안 된다(씨앗이 모자라다 등).
    Deny,
    /// 보물 상자가 열린다.
    Chest,
    /// 마무리 타·솟구치기가 맞았다: 낮고 무거운 쿵(0090).
    Impact,
    /// 실이 풀린다(0159): 돌아오는 이웃 — 맑게 올라가는 두 음 + 반짝임.
    Unbind,
    /// 부술 수 있는 것이 깨졌다(0175): 퍽 + 흙 소음.
    Break,
}

/// 파일 소리 표: (이름, 변형 수, 크기). 크기는 MASTER_VOLUME·설정에 곱한다 — 귀로 맞춘다.
/// 봉우리는 만들 때 모두 −1dBFS로 맞췄다. 자주 나는 메뉴 소리는 작게.
const FILES: [(&str, usize, f32); 9] = [
    ("ui_move", 1, 0.35),
    ("ui_confirm", 1, 0.5),
    ("ui_back", 1, 0.5),
    ("buy", 1, 0.7),
    ("deny", 1, 0.55),
    ("enemy_hit", 3, 0.75),
    ("enemy_die", 3, 0.85),
    ("hurt", 2, 0.9),
    ("chest", 1, 0.8),
];

/// 이 소리의 파일 표 자리.
fn file_of(id: Id) -> Option<usize> {
    Some(match id {
        Id::UiMove => 0,
        Id::UiConfirm => 1,
        Id::UiBack => 2,
        Id::Buy => 3,
        Id::Deny => 4,
        Id::EnemyHit => 5,
        Id::EnemyDie => 6,
        Id::Hurt => 7,
        Id::Chest => 8,
        _ => return None,
    })
}

/// 로직이 낼 소리 한 건: 내기 / (길게 우는 소리) 끊기.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    Play(Id),
    Stop(Id),
}

/// 로직이 쌓아 두는 소리 큐(0169 R1). 월드·모험·컷씬은 오디오 장치(`Sfx`)를 모르고 여기에만 쌓는다 — 그래서 테스트에서
/// 월드를 틱 단위로 돌리고 "어떤 소리가 났나"를 물을 수 있다. 앱이 틱·컷씬 뒤에 `Sfx::flush`로 비운다.
#[derive(Debug, Default)]
pub struct Queue(Vec<Cmd>);

impl Queue {
    pub fn play(&mut self, id: Id) {
        self.0.push(Cmd::Play(id));
    }

    pub fn stop(&mut self, id: Id) {
        self.0.push(Cmd::Stop(id));
    }

    /// 이 소리를 내라고 쌓였나(테스트).
    #[cfg(test)]
    pub fn played(&self, id: Id) -> bool {
        self.0.contains(&Cmd::Play(id))
    }

    /// 이 소리를 몇 번 내라고 쌓였나(테스트).
    #[cfg(test)]
    pub fn count(&self, id: Id) -> usize {
        self.0.iter().filter(|c| **c == Cmd::Play(id)).count()
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, Cmd> {
        self.0.drain(..)
    }
}

pub struct Sfx {
    claw: [Sound; 3],
    burrow: Sound,
    emerge: Sound,
    emerge_big: Sound,
    rumble: Sound,
    enemy_hit: Sound,
    enemy_die: Sound,
    hurt: Sound,
    die: Sound,
    shoot: Sound,
    charge: Sound,
    beat: [Sound; 4],
    beat_tick: Sound,
    /// 모으기 시작부터 올라가는 소리(0168).
    charge_up: Sound,
    /// 맞힘 소리 위에 얹는 짧고 높은 '틱'(0168).
    hit_tick: Sound,
    wall_hit: Sound,
    impact: Sound,
    unbind: Sound,
    brk: Sound,
    glowbug: [Sound; 4],
    heal: Sound,
    drill: [Sound; 3],
    drill_wall: Sound,
    parry: Sound,
    /// 파일 소리(`FILES` 차례). 못 읽은 것은 빈 목록.
    files: Vec<Vec<Sound>>,
    /// 파일 소리마다 바로 전에 낸 변형.
    last_variant: std::cell::RefCell<Vec<usize>>,
    /// 변형 고르기(게임의 rand와 따로: 소리가 게임 난수 차례를 바꾸지 않게).
    rng: Cell<u32>,
    /// 지금까지 낸(내려고 한) 소리 수. 앱이 메뉴 소리를 덧붙일지 본다(`App::menu_sound`).
    played: Cell<u32>,
    /// 반딧불 음 높이 차례.
    glow_step: Cell<usize>,
    /// 설정의 소리 크기(0..1). MASTER_VOLUME에 곱한다.
    volume: Cell<f32>,
    /// 소리마다 마지막으로 낸 때(0081): 같은 소리를 아주 짧은 사이에 겹쳐 내지 않는다(한 틱에 여럿 맞을 때).
    last: std::cell::RefCell<Vec<(usize, f64)>>,
}

/// 같은 소리를 다시 내기까지 최소 간격(초).
const SAME_SOUND_GAP: f64 = 0.03;

impl Sfx {
    /// 시작할 때 읽는 소리 파일(assets 기준, `load`와 같은 이름). main이 한꺼번에 받는다(0092).
    pub fn files() -> Vec<String> {
        FILES
            .iter()
            .flat_map(|(name, n, _)| (0..*n).map(move |i| format!("sfx/{name}_{i}.wav")))
            .collect()
    }

    pub async fn load() -> Self {
        let mut files = Vec::new();
        for (name, n, _) in FILES {
            let mut v = Vec::new();
            for i in 0..n {
                let path = format!("sfx/{name}_{i}.wav");
                match crate::loading::bytes(&path).await {
                    Ok(bytes) => match load_sound_from_bytes(&bytes).await {
                        Ok(s) => v.push(s),
                        Err(e) => eprintln!("효과음 {path}: 풀 수 없다({e})"),
                    },
                    Err(e) => eprintln!("효과음 {path}: 읽을 수 없다({e})"),
                }
            }
            files.push(v);
        }
        Self {
            last_variant: std::cell::RefCell::new(vec![usize::MAX; files.len()]),
            files,
            rng: Cell::new(0x2545_F491),
            played: Cell::new(0),
            claw: [
                make(&[
                    Layer::noise(0.06, 0.5, 3.0),
                    Layer::tone(0.07, 420.0, 180.0, Wave::Square, 0.35, 2.0),
                ])
                .await,
                make(&[
                    Layer::noise(0.06, 0.5, 3.0),
                    Layer::tone(0.07, 520.0, 220.0, Wave::Square, 0.35, 2.0),
                ])
                .await,
                make(&[
                    Layer::noise(0.14, 0.7, 2.0),
                    Layer::tone(0.22, 170.0, 55.0, Wave::Square, 0.7, 1.5),
                    Layer::tone(0.22, 85.0, 40.0, Wave::Sine, 0.8, 1.2),
                ])
                .await,
            ],
            burrow: make(&[
                Layer::tone(0.22, 380.0, 110.0, Wave::Triangle, 0.5, 1.0),
                Layer::noise(0.2, 0.25, 1.0),
            ])
            .await,
            emerge: make(&[
                Layer::tone(0.14, 110.0, 520.0, Wave::Triangle, 0.6, 1.0),
                Layer::noise(0.1, 0.6, 2.5),
                Layer::tone(0.18, 70.0, 45.0, Wave::Sine, 0.8, 1.5),
            ])
            .await,
            emerge_big: make(&[
                Layer::tone(0.2, 80.0, 420.0, Wave::Triangle, 0.7, 0.9),
                Layer::noise(0.22, 0.7, 1.6),
                Layer::tone(0.34, 55.0, 32.0, Wave::Sine, 1.0, 1.2),
            ])
            .await,
            rumble: make(&[
                Layer::noise(0.5, 0.28, 0.3),
                Layer::tone(0.5, 42.0, 58.0, Wave::Sine, 0.55, 0.2),
            ])
            .await,
            enemy_hit: make(&[Layer::noise(0.04, 0.6, 4.0)]).await,
            enemy_die: make(&[
                Layer::noise(0.25, 0.6, 1.5),
                Layer::tone(0.25, 300.0, 60.0, Wave::Square, 0.3, 1.0),
            ])
            .await,
            hurt: make(&[
                Layer::tone(0.25, 220.0, 160.0, Wave::Square, 0.5, 0.8),
                Layer::tone(0.25, 227.0, 150.0, Wave::Square, 0.4, 0.8),
            ])
            .await,
            die: make(&[
                Layer::tone(0.7, 300.0, 40.0, Wave::Square, 0.5, 0.6),
                Layer::noise(0.5, 0.3, 1.0),
            ])
            .await,
            shoot: make(&[Layer::tone(0.09, 760.0, 420.0, Wave::Sine, 0.4, 1.0)]).await,
            charge: make(&[Layer::tone(0.12, 90.0, 260.0, Wave::Triangle, 0.45, 0.8)]).await,
            // 박자 콤보(0167 → 0174): 짧은 종소리, 연속으로 맞추면 도·미·솔·높은 도.
            beat_tick: make(&[
                Layer::tone(0.035, 880.0, 660.0, Wave::Triangle, 0.22, 2.5),
                Layer::noise(0.008, 0.08, 3.0),
            ])
            .await,
            beat: [
                make(&beat_layers(1046.5)).await,
                make(&beat_layers(1318.5)).await,
                make(&beat_layers(1568.0)).await,
                make(&beat_layers(2093.0)).await,
            ],
            charge_up: make(&[Layer::tone(0.45, 110.0, 440.0, Wave::Triangle, 0.2, 0.25)]).await,
            hit_tick: make(&[
                Layer::tone(0.03, 2600.0, 1900.0, Wave::Triangle, 0.18, 1.0),
                Layer::noise(0.02, 0.1, 1.5),
            ])
            .await,
            wall_hit: make(&[
                Layer::noise(0.1, 0.6, 2.0),
                Layer::tone(0.12, 90.0, 50.0, Wave::Sine, 0.6, 1.5),
            ])
            .await,
            impact: make(&[
                Layer::tone(0.3, 130.0, 38.0, Wave::Sine, 1.0, 1.1),
                Layer::noise(0.12, 0.7, 2.5),
                Layer::tone(0.16, 260.0, 80.0, Wave::Square, 0.25, 1.6),
            ])
            .await,
            unbind: make(&[
                Layer::tone(0.5, 520.0, 1040.0, Wave::Sine, 0.45, 1.2),
                Layer::tone(0.6, 780.0, 1560.0, Wave::Triangle, 0.25, 1.6),
                Layer::noise(0.3, 0.12, 3.0),
            ])
            .await,
            brk: make(&[
                Layer::noise(0.09, 0.5, 2.2),
                Layer::tone(0.1, 220.0, 70.0, Wave::Square, 0.3, 1.6),
            ])
            .await,
            glowbug: [
                make(&[
                    Layer::tone(0.07, 1320.0, 1500.0, Wave::Sine, 0.35, 1.5),
                    Layer::tone(0.05, 2640.0, 2900.0, Wave::Sine, 0.12, 2.0),
                ])
                .await,
                make(&[
                    Layer::tone(0.07, 1480.0, 1680.0, Wave::Sine, 0.35, 1.5),
                    Layer::tone(0.05, 2960.0, 3200.0, Wave::Sine, 0.12, 2.0),
                ])
                .await,
                make(&[
                    Layer::tone(0.07, 1660.0, 1880.0, Wave::Sine, 0.35, 1.5),
                    Layer::tone(0.05, 3320.0, 3600.0, Wave::Sine, 0.12, 2.0),
                ])
                .await,
                make(&[
                    Layer::tone(0.08, 1760.0, 2000.0, Wave::Sine, 0.35, 1.5),
                    Layer::tone(0.06, 3520.0, 3900.0, Wave::Sine, 0.12, 2.0),
                ])
                .await,
            ],
            heal: make(&[
                Layer::tone(0.12, 523.0, 523.0, Wave::Triangle, 0.4, 0.8),
                Layer::tone(0.24, 659.0, 1046.0, Wave::Triangle, 0.35, 1.0),
                Layer::tone(0.3, 1046.0, 1318.0, Wave::Sine, 0.15, 1.5),
            ])
            .await,
            drill: [
                load_samples(&engine(tuning::DRILL_REV, tuning::DRILL_TIME)).await,
                load_samples(&engine(
                    tuning::DRILL_REV,
                    tuning::DRILL_TIME * tuning::TAL_LONG_DRILL,
                ))
                .await,
                load_samples(&engine(
                    tuning::DRILL_REV,
                    tuning::DRILL_TIME * tuning::TAL_SHORT_DRILL,
                ))
                .await,
            ],
            drill_wall: make(&[
                Layer::noise(0.3, 0.9, 1.5),
                Layer::tone(0.35, 120.0, 35.0, Wave::Sine, 1.0, 1.2),
                Layer::tone(0.2, 240.0, 60.0, Wave::Square, 0.4, 1.5),
            ])
            .await,
            parry: make(&[
                Layer::tone(0.25, 1980.0, 1900.0, Wave::Sine, 0.45, 1.2),
                Layer::tone(0.18, 2970.0, 2900.0, Wave::Sine, 0.2, 1.6),
                Layer::noise(0.04, 0.5, 4.0),
            ])
            .await,
            glow_step: Cell::new(0),
            volume: Cell::new(1.0),
            last: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.set(v.clamp(0.0, 1.0));
    }

    /// 지금까지 낸 소리 수(앱이 한 프레임 사이에 소리가 났는지 본다).
    pub fn played(&self) -> u32 {
        self.played.get()
    }

    pub fn play(&self, id: Id) {
        self.played.set(self.played.get().wrapping_add(1));
        if self.volume.get() <= 0.0 {
            return;
        }
        let Some((s, gain, key)) = self.pick(id) else {
            return;
        };
        let now = macroquad::time::get_time();
        {
            let mut last = self.last.borrow_mut();
            match last.iter_mut().find(|(k, _)| *k == key) {
                Some((_, t)) if now - *t < SAME_SOUND_GAP => return,
                Some((_, t)) => *t = now,
                None => last.push((key, now)),
            }
        }
        play_sound(
            s,
            PlaySoundParams {
                looped: false,
                volume: tuning::MASTER_VOLUME * self.volume.get() * gain,
            },
        );
    }

    /// 로직이 쌓아 둔 소리 큐를 비우며 낸다.
    pub fn flush(&self, q: &mut Queue) {
        for c in q.drain() {
            match c {
                Cmd::Play(id) => self.play(id),
                Cmd::Stop(id) => self.stop(id),
            }
        }
    }

    /// 길게 울리는 소리를 끊는다(드릴이 벽에 부딪혔을 때).
    pub fn stop(&self, id: Id) {
        if let Some(s) = self.synth(id) {
            stop_sound(s);
        }
    }

    /// 낼 소리, 크기, 같은 소리 간격을 셀 열쇠. 파일 소리의 변형들은 열쇠 하나를 같이 쓴다.
    fn pick(&self, id: Id) -> Option<(&Sound, f32, usize)> {
        if let Some(f) = file_of(id) {
            let set = &self.files[f];
            if !set.is_empty() {
                let mut last = self.last_variant.borrow_mut();
                let mut i = 0;
                if set.len() > 1 {
                    let mut r = self.rng.get();
                    r ^= r << 13;
                    r ^= r >> 17;
                    r ^= r << 5;
                    self.rng.set(r);
                    // 바로 전 것을 빼고 고른다.
                    i = r as usize % (set.len() - 1);
                    if i >= last[f] {
                        i += 1;
                    }
                }
                last[f] = i;
                return Some((&set[i], FILES[f].2, f));
            }
        }
        self.synth(id).map(|s| (s, 1.0, s as *const Sound as usize))
    }

    /// 합성한 소리. 메뉴·상자처럼 파일로만 있는 소리는 None(조용히).
    fn synth(&self, id: Id) -> Option<&Sound> {
        Some(match id {
            Id::Claw(i) => &self.claw[i.min(2)],
            Id::Burrow => &self.burrow,
            Id::Emerge => &self.emerge,
            Id::EmergeBig => &self.emerge_big,
            Id::Rumble => &self.rumble,
            Id::EnemyHit => &self.enemy_hit,
            Id::EnemyDie => &self.enemy_die,
            Id::Hurt => &self.hurt,
            Id::Die => &self.die,
            Id::Shoot => &self.shoot,
            Id::Charge => &self.charge,
            Id::Beat(n) => &self.beat[n.saturating_sub(1) % self.beat.len()],
            Id::BeatTick => &self.beat_tick,
            Id::ChargeUp => &self.charge_up,
            Id::HitTick => &self.hit_tick,
            Id::WallHit => &self.wall_hit,
            Id::Impact => &self.impact,
            Id::Unbind => &self.unbind,
            Id::Break => &self.brk,
            Id::Glowbug => {
                let i = self.glow_step.get();
                self.glow_step.set((i + 1) % self.glowbug.len());
                &self.glowbug[i]
            }
            Id::Heal => &self.heal,
            Id::Drill(i) => &self.drill[i.min(2)],
            Id::DrillWall => &self.drill_wall,
            Id::Parry => &self.parry,
            Id::Buy => &self.glowbug[0],
            Id::UiMove | Id::UiConfirm | Id::UiBack | Id::Deny | Id::Chest => return None,
        })
    }
}

// ---- 합성 -----------------------------------------------------------------

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Triangle,
    Noise,
}

/// 한 레이어: 주파수 f0→f1로 미끄러지는 파형 × 감쇠 포락선 (1-p)^decay.
#[derive(Clone, Copy)]
struct Layer {
    dur: f32,
    f0: f32,
    f1: f32,
    wave: Wave,
    gain: f32,
    decay: f32,
}

impl Layer {
    fn tone(dur: f32, f0: f32, f1: f32, wave: Wave, gain: f32, decay: f32) -> Self {
        Self {
            dur,
            f0,
            f1,
            wave,
            gain,
            decay,
        }
    }

    fn noise(dur: f32, gain: f32, decay: f32) -> Self {
        Self {
            dur,
            f0: 0.0,
            f1: 0.0,
            wave: Wave::Noise,
            gain,
            decay,
        }
    }
}

/// 박자 종소리(0174): 기본음 + 한 옥타브 위를 살짝, 아주 짧은 때림 소음.
fn beat_layers(f: f32) -> [Layer; 3] {
    [
        Layer::tone(0.09, f, f * 0.99, Wave::Triangle, 0.3, 0.55),
        Layer::tone(0.05, f * 2.0, f * 2.0, Wave::Sine, 0.08, 1.0),
        Layer::noise(0.012, 0.12, 2.0),
    ]
}

async fn make(layers: &[Layer]) -> Sound {
    load_samples(&render(layers)).await
}

async fn load_samples(samples: &[i16]) -> Sound {
    let bytes = wav_bytes(samples);
    load_sound_from_bytes(&bytes)
        .await
        .expect("합성 WAV 로드 실패")
}

/// 드릴 엔진 "부아아앙": 시동(`rev`초) 동안 음이 낮은 데서 빠르게 올라가고, 돌진(`dash`초) 동안 높은 데서 버틴다.
/// 톱니 + 사각파에 엔진처럼 떨리는 음높이(22Hz)와 털털거리는 크기(31Hz), 낮은 흙 소음.
/// 시동이 끝나는 순간 쿵(낮은 사인 + 소음 터짐). 끝 0.15초는 잦아든다.
fn engine(rev: f32, dash: f32) -> Vec<i16> {
    use std::f32::consts::TAU;
    let total = rev + dash;
    let n = (total * SR as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let (mut ph, mut ph2, mut boom_ph) = (0.0f32, 0.0f32, 0.0f32);
    let mut rng: u32 = 0x1234_5678;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let base = if t < rev {
            let k = t / rev;
            70.0 * (190.0f32 / 70.0).powf(k * k)
        } else {
            190.0 + 50.0 * ((t - rev) / dash)
        };
        let f = base * (1.0 + 0.06 * (TAU * 22.0 * t).sin());
        ph = (ph + f / SR as f32).fract();
        ph2 = (ph2 + f * 2.01 / SR as f32).fract();
        let saw = ph * 2.0 - 1.0;
        let sq = if ph2 < 0.5 { 0.6 } else { -0.6 };
        let am = 0.7 + 0.3 * (TAU * 31.0 * t).sin();
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        let noise = (rng as f32 / u32::MAX as f32) * 2.0 - 1.0;
        lp += (noise - lp) * 0.08;
        let env = (t / 0.05).min(1.0) * ((total - t) / 0.15).clamp(0.0, 1.0);
        let grow = if t < rev { 0.45 + 0.55 * t / rev } else { 1.0 };
        let mut s = (saw * 0.32 + sq * 0.16) * am * grow + lp * 0.5;
        // 튀어 나가는 쿵
        let bt = t - rev;
        if (0.0..0.25).contains(&bt) {
            let k = bt / 0.25;
            boom_ph = (boom_ph + (95.0 - 55.0 * k) / SR as f32).fract();
            s += (boom_ph * TAU).sin() * 0.7 * (1.0 - k).powf(1.5)
                + noise * 0.4 * (1.0 - k).powf(3.0);
        }
        out.push(((s * env).clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    out
}

fn render(layers: &[Layer]) -> Vec<i16> {
    let max_dur = layers.iter().map(|l| l.dur).fold(0.0, f32::max);
    let n = (max_dur * SR as f32) as usize;
    let mut buf = vec![0.0f32; n];
    let mut rng: u32 = 0x9E37_79B9;
    for l in layers {
        let ln = ((l.dur * SR as f32) as usize).min(n);
        let mut phase = 0.0f32;
        for (i, out) in buf.iter_mut().enumerate().take(ln) {
            let p = i as f32 / ln as f32;
            let env = (1.0 - p).powf(l.decay);
            let f = l.f0 + (l.f1 - l.f0) * p;
            phase = (phase + f / SR as f32).fract();
            let s = match l.wave {
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
                Wave::Noise => {
                    // xorshift
                    rng ^= rng << 13;
                    rng ^= rng >> 17;
                    rng ^= rng << 5;
                    (rng as f32 / u32::MAX as f32) * 2.0 - 1.0
                }
            };
            *out += s * env * l.gain;
        }
    }
    // 시작 클릭 방지: 첫 1ms 페이드 인.
    let fade = (SR / 1000) as usize;
    for (i, s) in buf.iter_mut().enumerate().take(fade) {
        *s *= i as f32 / fade as f32;
    }
    buf.iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

/// 16비트 모노 PCM WAV.
fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVE");
    v.extend_from_slice(b"fmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes()); // PCM
    v.extend_from_slice(&1u16.to_le_bytes()); // mono
    v.extend_from_slice(&SR.to_le_bytes());
    v.extend_from_slice(&(SR * 2).to_le_bytes()); // byte rate
    v.extend_from_slice(&2u16.to_le_bytes()); // block align
    v.extend_from_slice(&16u16.to_le_bytes()); // bits
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}
