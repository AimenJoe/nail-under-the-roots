//! 손맛 실험실 Lab(0269 R33, 요구 2.8A): 몇몇 손맛 수치를 게임을 돌리면서 바꿔 본다.
//!
//! - 값의 원본은 `tuning.rs` 상수 그대로다. 여기 표 `KNOBS`는 "Lab에 내놓는 상수"의 목록과 조절 범위만 갖는다.
//! - 게임 코드는 그 상수를 `tun(K::이름)`으로 읽는다. **운영판(`--no-default-features`)에는 덮어쓰기 코드가 없어** 늘 상수 값이다.
//! - 개발판은 덮어쓰기 표(`OVR`)를 먼저 본다. 값은 일시정지의 "연구실"(한국어가 아니면 "Lab", `app/lab.rs`)에서 바꾸고, 프리셋 칸 셋은 진행 기록·설정과
//!   따로 저장한다(`profile::lab_read`·`lab_write`).
//! - 새 손잡이 = `knobs!` 한 줄 + 그 상수를 읽는 자리를 `tun(K::…)`로. 표에 있는데 아무도 `tun`으로 안 읽으면 시험이 잡는다.

use crate::tuning as t;

/// 손잡이 묶음(화면에서 머리줄로 가른다).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Shake,
    Hurt,
    Attack,
    Move,
    Timing,
}

#[cfg_attr(not(feature = "dev"), allow(dead_code))]
impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Shake => "흔들림",
            Group::Hurt => "피격 등급",
            Group::Attack => "공격 손맛",
            Group::Move => "이동·회피",
            Group::Timing => "박자·입력",
        }
    }
}

/// 손잡이 하나: 상수 이름(= 프리셋 줄의 이름), 묶음, 기본값(상수), 범위, 한 칸.
#[cfg_attr(not(feature = "dev"), allow(dead_code))]
pub struct Knob {
    pub name: &'static str,
    pub group: Group,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

macro_rules! knobs {
    ($( $name:ident = $default:expr, $group:ident, $min:expr, $max:expr, $step:expr; )*) => {
        /// 손잡이 이름. 상수 이름과 같다.
        #[allow(non_camel_case_types, clippy::upper_case_acronyms, clippy::enum_variant_names)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum K { $($name),* }
        pub const KNOBS: &[Knob] = &[
            $( Knob { name: stringify!($name), group: Group::$group, default: $default, min: $min, max: $max, step: $step } ),*
        ];
    };
}

knobs! {
    SHAKE_MAX_OFFSET = t::SHAKE_MAX_OFFSET, Shake, 0.0, 10.0, 0.5;
    SHAKE_DECAY = t::SHAKE_DECAY, Shake, 0.5, 6.0, 0.1;
    KICK_DECAY = t::KICK_DECAY, Shake, 5.0, 60.0, 1.0;
    HURT_LIGHT_HITSTOP = t::HURT_LIGHT.hitstop, Hurt, 0.0, 0.3, 0.01;
    HURT_LIGHT_SHAKE = t::HURT_LIGHT.shake, Hurt, 0.0, 1.0, 0.05;
    HURT_LIGHT_KICK = t::HURT_LIGHT.kick, Hurt, 0.0, 10.0, 0.5;
    HURT_LIGHT_EDGE = t::HURT_LIGHT.edge, Hurt, 0.0, 1.0, 0.05;
    HURT_HEAVY_HITSTOP = t::HURT_HEAVY.hitstop, Hurt, 0.0, 0.3, 0.01;
    HURT_HEAVY_SHAKE = t::HURT_HEAVY.shake, Hurt, 0.0, 1.0, 0.05;
    HURT_HEAVY_KICK = t::HURT_HEAVY.kick, Hurt, 0.0, 10.0, 0.5;
    HURT_HEAVY_EDGE = t::HURT_HEAVY.edge, Hurt, 0.0, 1.0, 0.05;
    HURT_BOSS_HITSTOP = t::HURT_BOSS.hitstop, Hurt, 0.0, 0.3, 0.01;
    HURT_BOSS_SHAKE = t::HURT_BOSS.shake, Hurt, 0.0, 1.0, 0.05;
    HURT_BOSS_KICK = t::HURT_BOSS.kick, Hurt, 0.0, 10.0, 0.5;
    HURT_BOSS_EDGE = t::HURT_BOSS.edge, Hurt, 0.0, 1.0, 0.05;
    HURT_HAZARD_HITSTOP = t::HURT_HAZARD.hitstop, Hurt, 0.0, 0.3, 0.01;
    HURT_HAZARD_SHAKE = t::HURT_HAZARD.shake, Hurt, 0.0, 1.0, 0.05;
    HURT_HAZARD_KICK = t::HURT_HAZARD.kick, Hurt, 0.0, 10.0, 0.5;
    HURT_HAZARD_EDGE = t::HURT_HAZARD.edge, Hurt, 0.0, 1.0, 0.05;
    HURT_SEEP_SHAKE = t::HURT_SEEP.shake, Hurt, 0.0, 1.0, 0.05;
    HURT_SEEP_EDGE = t::HURT_SEEP.edge, Hurt, 0.0, 1.0, 0.05;
    KICK_LIGHT = t::KICK_LIGHT, Attack, 0.0, 10.0, 0.5;
    KICK_FINISHER = t::KICK_FINISHER, Attack, 0.0, 10.0, 0.5;
    KICK_HEAVY = t::KICK_HEAVY, Attack, 0.0, 10.0, 0.5;
    PARRY_HITSTOP = t::PARRY_HITSTOP, Attack, 0.0, 0.4, 0.01;
    PARRY_SHAKE = t::PARRY_SHAKE, Attack, 0.0, 1.0, 0.05;
    ENEMY_DIE_SHAKE = t::ENEMY_DIE_SHAKE, Attack, 0.0, 1.0, 0.05;
    PLAYER_SPEED = t::PLAYER_SPEED, Move, 40.0, 160.0, 5.0;
    PLAYER_HURT_INVULN = t::PLAYER_HURT_INVULN, Move, 0.2, 2.0, 0.05;
    BURROW_DASH_FACTOR = t::BURROW_DASH_FACTOR, Move, 1.0, 4.0, 0.1;
    BURROW_COOLDOWN = t::BURROW_COOLDOWN, Move, 0.1, 2.0, 0.05;
    RUN_SPEED = t::RUN_SPEED, Move, 1.0, 2.5, 0.05;
    BEAT_SCALE = 1.0, Timing, 0.5, 2.0, 0.05;
    BEAT_WINDOW = t::BEAT_WINDOW, Timing, 0.02, 0.3, 0.01;
    HEAVY_CHARGE = t::HEAVY_CHARGE, Timing, 0.1, 1.5, 0.05;
    PARRY_WINDOW = t::PARRY_WINDOW, Timing, 0.02, 0.4, 0.01;
    INPUT_BUFFER = t::INPUT_BUFFER, Timing, 0.0, 0.3, 0.01;
}

/// 손잡이 수.
pub const N: usize = KNOBS.len();

/// 손잡이 `k`의 지금 값. 운영판은 늘 상수(기본값).
#[inline]
pub fn tun(k: K) -> f32 {
    #[cfg(feature = "dev")]
    if let Some(v) = dev::get(k as usize) {
        return v;
    }
    KNOBS[k as usize].default
}

/// 개발판의 덮어쓰기(운영판에는 이 모듈이 없다).
#[cfg(feature = "dev")]
pub mod dev {
    use super::{KNOBS, N};
    use std::sync::RwLock;

    /// 덮어쓴 값(NaN = 덮어쓰지 않음 → 상수).
    static OVR: RwLock<[f32; N]> = RwLock::new([f32::NAN; N]);

    pub fn get(i: usize) -> Option<f32> {
        let v = OVR.read().ok()?[i];
        (!v.is_nan()).then_some(v)
    }

    /// 지금 값(덮어쓴 값 아니면 기본값).
    pub fn value(i: usize) -> f32 {
        get(i).unwrap_or(KNOBS[i].default)
    }

    /// 값을 정한다. 기본값과 같으면 덮어쓰기를 지운다. 범위 밖은 범위 끝으로.
    pub fn set(i: usize, v: f32) {
        let k = &KNOBS[i];
        let v = v.clamp(k.min, k.max);
        if let Ok(mut o) = OVR.write() {
            o[i] = if (v - k.default).abs() < 1e-6 {
                f32::NAN
            } else {
                v
            };
        }
    }

    /// 한 칸 올리거나 내린다(`dir` ±1).
    pub fn nudge(i: usize, dir: f32) {
        let k = &KNOBS[i];
        // 한 칸 단위로 맞춘다(떠다니는 소수 오차가 쌓이지 않게).
        let steps = ((value(i) - k.min) / k.step).round() + dir;
        set(i, k.min + steps * k.step);
    }

    pub fn reset_all() {
        if let Ok(mut o) = OVR.write() {
            *o = [f32::NAN; N];
        }
    }

    pub fn changed(i: usize) -> bool {
        get(i).is_some()
    }

    /// 지금 덮어쓰기 표(NaN = 상수).
    pub fn snapshot() -> [f32; N] {
        OVR.read().map(|o| *o).unwrap_or([f32::NAN; N])
    }

    /// 프리셋 글로 덮어쓰기를 바꾼다. 읽은 줄 수.
    pub fn load(text: &str) -> usize {
        let (vals, n) = super::decode(text);
        if let Ok(mut o) = OVR.write() {
            *o = vals;
        }
        n
    }
}

/// 프리셋 글: 덮어쓴 손잡이만 `이름=값` 한 줄씩.
#[cfg_attr(not(feature = "dev"), allow(dead_code))]
pub fn encode(vals: &[f32; N]) -> String {
    let mut s = String::new();
    for (k, v) in KNOBS.iter().zip(vals) {
        if !v.is_nan() {
            s.push_str(&format!("{}={}\n", k.name, v));
        }
    }
    s
}

/// 프리셋 글 → 덮어쓰기 표. 모르는 이름(상수가 바뀜)·틀린 줄은 버리고, 범위 밖은 범위 끝으로, 기본값과 같으면 덮어쓰지 않음.
/// (표, 읽은 줄 수).
#[cfg_attr(not(feature = "dev"), allow(dead_code))]
pub fn decode(text: &str) -> ([f32; N], usize) {
    let mut vals = [f32::NAN; N];
    let mut n = 0;
    for line in text.lines() {
        let Some((name, v)) = line.split_once('=') else {
            continue;
        };
        let (Some(i), Ok(v)) = (
            KNOBS.iter().position(|k| k.name == name.trim()),
            v.trim().parse::<f32>(),
        ) else {
            continue;
        };
        let k = &KNOBS[i];
        let v = v.clamp(k.min, k.max);
        vals[i] = if (v - k.default).abs() < 1e-6 {
            f32::NAN
        } else {
            v
        };
        n += 1;
    }
    (vals, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knobs_start_at_their_constants_and_are_read_somewhere() {
        // 기본값이 범위 안이고 한 칸이 양수.
        for k in KNOBS {
            assert!(k.min <= k.default && k.default <= k.max, "{} 범위", k.name);
            assert!(k.step > 0.0, "{} 한 칸", k.name);
        }
        // 표에 올린 손잡이는 게임 코드 어딘가가 `K::이름`으로 읽는다(안 읽으면 Lab에서 바꿔도 아무 일도 없다).
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut all = String::new();
        fn walk(d: &std::path::Path, out: &mut String) {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("lab.rs") {
                    out.push_str(&std::fs::read_to_string(&p).unwrap());
                }
            }
        }
        walk(&src, &mut all);
        // 손맛 숫자는 이름으로(0269 R32): 흔들림·멈춤을 부르는 자리에 숫자를 직접 쓰지 않는다.
        for pat in [
            "add_trauma(0",
            "add_trauma(1",
            "add_hitstop(0",
            "add_hitstop(1",
            "Step::Shake(0",
        ] {
            assert!(!all.contains(pat), "{pat}…: 숫자 대신 tuning.rs의 이름을");
        }
        for k in KNOBS {
            assert!(
                all.contains(&format!("K::{}", k.name)),
                "{}: 아무도 tun(K::{})로 읽지 않는다",
                k.name,
                k.name
            );
        }
    }

    #[test]
    fn presets_round_trip_and_ignore_unknown_lines() {
        // 전역 덮어쓰기는 건드리지 않는다(다른 시험이 같은 때 손잡이를 읽는다) — 순수한 글 ↔ 표만.
        let mut vals = [f32::NAN; N];
        vals[K::PLAYER_SPEED as usize] = t::PLAYER_SPEED + 5.0;
        let text = encode(&vals);
        assert_eq!(text, format!("PLAYER_SPEED={}\n", t::PLAYER_SPEED + 5.0));
        let (back, n) = decode(&format!(
            "{text}OLD_NAME=3\nbad line\nINPUT_BUFFER=99\nKICK_LIGHT={}\n",
            t::KICK_LIGHT
        ));
        assert_eq!(n, 3);
        assert_eq!(back[K::PLAYER_SPEED as usize], t::PLAYER_SPEED + 5.0);
        // 범위 밖은 끝으로, 기본값과 같으면 덮어쓰지 않음.
        assert_eq!(
            back[K::INPUT_BUFFER as usize],
            KNOBS[K::INPUT_BUFFER as usize].max
        );
        assert!(back[K::KICK_LIGHT as usize].is_nan());
    }
}
