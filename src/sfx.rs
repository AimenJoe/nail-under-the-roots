//! 임시 사운드. 에셋 없이 시작 시 합성한 WAV를 메모리에서 로드한다.
//! 진짜 사운드가 들어오면 이 파일은 `load_sound(path)`로 바뀐다.

use std::cell::Cell;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

use crate::tuning;

const SR: u32 = 44_100;

#[derive(Debug, Clone, Copy)]
pub enum Id {
    Claw(usize),
    Burrow,
    Emerge,
    EnemyHit,
    EnemyDie,
    Hurt,
    Die,
    Shoot,
    Charge,
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
}

pub struct Sfx {
    claw: [Sound; 3],
    burrow: Sound,
    emerge: Sound,
    enemy_hit: Sound,
    enemy_die: Sound,
    hurt: Sound,
    die: Sound,
    shoot: Sound,
    charge: Sound,
    wall_hit: Sound,
    glowbug: [Sound; 4],
    heal: Sound,
    drill: [Sound; 3],
    drill_wall: Sound,
    parry: Sound,
    /// 반딧불 음 높이 차례.
    glow_step: Cell<usize>,
    /// 설정의 소리 크기(0..1). MASTER_VOLUME에 곱한다.
    volume: Cell<f32>,
}

impl Sfx {
    pub async fn load() -> Self {
        Self {
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
            wall_hit: make(&[
                Layer::noise(0.1, 0.6, 2.0),
                Layer::tone(0.12, 90.0, 50.0, Wave::Sine, 0.6, 1.5),
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
        }
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.set(v.clamp(0.0, 1.0));
    }

    pub fn play(&self, id: Id) {
        if self.volume.get() <= 0.0 {
            return;
        }
        let s = self.sound(id);
        play_sound(
            s,
            PlaySoundParams {
                looped: false,
                volume: tuning::MASTER_VOLUME * self.volume.get(),
            },
        );
    }

    /// 길게 울리는 소리를 끊는다(드릴이 벽에 부딪혔을 때).
    pub fn stop(&self, id: Id) {
        stop_sound(self.sound(id));
    }

    fn sound(&self, id: Id) -> &Sound {
        match id {
            Id::Claw(i) => &self.claw[i.min(2)],
            Id::Burrow => &self.burrow,
            Id::Emerge => &self.emerge,
            Id::EnemyHit => &self.enemy_hit,
            Id::EnemyDie => &self.enemy_die,
            Id::Hurt => &self.hurt,
            Id::Die => &self.die,
            Id::Shoot => &self.shoot,
            Id::Charge => &self.charge,
            Id::WallHit => &self.wall_hit,
            Id::Glowbug => {
                let i = self.glow_step.get();
                self.glow_step.set((i + 1) % self.glowbug.len());
                &self.glowbug[i]
            }
            Id::Heal => &self.heal,
            Id::Drill(i) => &self.drill[i.min(2)],
            Id::DrillWall => &self.drill_wall,
            Id::Parry => &self.parry,
        }
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
