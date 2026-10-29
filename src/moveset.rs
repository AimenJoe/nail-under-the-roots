//! 무브셋: 무기마다 다른 콤보를 데이터로. 플레이어 코드는 이 표만 읽는다.
//!
//! 규칙(docs/decisions/0009): 마지막 타가 가장 강하고 느리다. 굴파기(회피)는 각 타의 `dodge_cancel_after` 뒤에
//! 스윙을 끊을 수 있어 후딜레이를 취소하고 다음 콤보로 섞는다. 공격 입력은 활성 구간이 끝난 뒤 다음 타로 이어진다.
//! 어떤 캐릭터가 어떤 무기를 쓰는지는 equipment.rs(docs/decisions/0011). 새 무기는 여기에 `Moveset` 하나 + 장비 표 한 줄.

use macroquad::color::Color;

use crate::tuning as t;

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// 스윙 길이.
    pub swing: f32,
    /// 히트박스 활성 구간 (시작, 끝).
    pub active: (f32, f32),
    /// 스윙 뒤 추가 경직.
    pub recovery: f32,
    pub damage: i32,
    pub knockback: f32,
    pub stun: f32,
    pub hitstop: f32,
    pub shake: f32,
    /// 전방 반원 반지름.
    pub range: f32,
    /// 스윙 시작 시 전진.
    pub lunge: f32,
    /// 이 시간 뒤부터 굴파기로 끊을 수 있다. 0 = 언제나.
    pub dodge_cancel_after: f32,
    /// 맞은 적에게 거는 독(초). 0 = 없음. 독은 POISON_TICK마다 1씩 깎는다.
    pub poison: f32,
    /// 던지기(도토, 0195): 활성 구간 첫 틱에 탄을 낸다. `range`는 그때 손앞의 짧은 판정(받아치기·탄 베기)만 —
    /// 적은 손으로 때리지 않고 탄이 맞힌다. `Throw::None`이면 보통 근접 타.
    pub throw: Throw,
    /// 활성 구간 첫 틱에 발 앞에 가시 덤불을 남긴다(가시덩굴 각반, 0246). 밟은 적이 찔린다.
    pub thorns: bool,
    /// 등딱지에 튕기지 않고 연다(조약돌 덧신의 마무리 타, 0246).
    pub crush: bool,
}

/// 던지는 탄(0195). 한 번에 `count`개를 `spread`(라디안) 간격의 부채꼴로. 탄은 `reach`(px)를 날면 떨어진다.
/// `splash` > 0이면 맞거나 떨어진 자리에서 그 반지름만큼 둘레를 친다(큰 도토리).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Throw {
    None,
    Shot {
        count: u8,
        spread: f32,
        speed: f32,
        reach: f32,
        radius: f32,
        splash: f32,
        /// 벽에 튕기는 횟수(솔방울, 0246).
        bounce: u8,
        /// 멈추는 자리(적·벽)에서 작은 조각으로 갈라지는 수(호두, 0246).
        split: u8,
        /// 멈추는 자리에 가시를 남긴다(밤송이, 0246).
        thorns: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Moveset {
    pub hits: &'static [Hit],
    /// 스윙이 끝난 뒤 다음 타 입력이 유효한 시간.
    pub combo_window: f32,
    /// 공격 중 이동 속도 배율.
    pub move_factor: f32,
    /// 발톱 자국 색.
    pub mark: Color,
    /// 박자 창 배율(꽃잎 발목끈은 넓다, 0246).
    pub beat_window: f32,
    /// 박자에 맞춘 타가 게이지를 채우는 배율(0246).
    pub beat_sp: f32,
}

impl Moveset {
    pub fn last(&self) -> usize {
        self.hits.len() - 1
    }

    /// 강공격(0161): 누르고 있다가 떼면. 마지막 타를 바탕으로 더 세고 넓고 조금 느리다.
    pub fn heavy(&self) -> Hit {
        heavy_of(self.hits[self.last()])
    }
}

/// 강공격 수치(0161 → 0168): 마지막 타에서 — 피해는 마지막 타 + 1(3 이상), 반지름 1.3배, 넉백 1.7배(벽 쿵이 노림수가 된다),
/// 경직 +0.5, 멈춤·흔들림 한 단 위, 스윙·후딜레이 조금 더 길고, 활성 구간이 끝나야 굴파기로 끊는다. 무기마다 따로 적지 않는다.
/// 같은 시간의 콤보(피해 4)보다 약하면 아무도 안 쓴다(0166).
/// 강타 활성 시작(초): 0 — 뗀 다음 틱에 판정(0189).
const HEAVY_ACTIVE_START: f32 = 0.0;

pub fn heavy_of(last: Hit) -> Hit {
    Hit {
        swing: last.swing + 0.06,
        // 떼는 순간 나간다(0189): 모으기가 이미 준비 동작이라, 마무리 타의 0.05초 준비를 또 기다리면 늦게 느껴졌다.
        active: (
            HEAVY_ACTIVE_START,
            HEAVY_ACTIVE_START + (last.active.1 - last.active.0) + 0.03,
        ),
        recovery: last.recovery + 0.08,
        damage: (last.damage + 1).max(3),
        knockback: last.knockback * 1.7,
        stun: last.stun + 0.5,
        hitstop: last.hitstop + 0.07,
        shake: last.shake + 0.2,
        range: last.range * 1.3,
        lunge: last.lunge + 10.0,
        dodge_cancel_after: HEAVY_ACTIVE_START + (last.active.1 - last.active.0) + 0.03,
        poison: last.poison,
        thorns: last.thorns,
        crush: last.crush,
        // 던지는 무기의 강타는 큰 도토리 하나(0195): 느리고 크게 날아가 맞은 자리에서 튄다.
        throw: match last.throw {
            Throw::None => Throw::None,
            Throw::Shot {
                speed,
                reach,
                radius,
                bounce,
                split,
                thorns,
                ..
            } => Throw::Shot {
                count: 1,
                spread: 0.0,
                speed: speed * t::BIG_ACORN_SPEED,
                reach: reach * t::BIG_ACORN_REACH,
                radius: radius * t::BIG_ACORN_SIZE,
                splash: t::BIG_ACORN_SPLASH,
                bounce,
                split,
                thorns,
            },
        },
    }
}

const BASE: Hit = Hit {
    swing: 0.12,
    active: (0.02, 0.08),
    recovery: 0.0,
    damage: 1,
    knockback: 90.0,
    stun: 0.15,
    hitstop: 0.08,
    shake: 0.15,
    range: t::CLAW_RANGE,
    lunge: t::CLAW_LUNGE,
    dodge_cancel_after: 0.0,
    poison: 0.0,
    throw: Throw::None,
    thorns: false,
    crush: false,
};

const fn claw(i: usize) -> Hit {
    Hit {
        swing: t::CLAW_SWING_TIME[i],
        active: t::CLAW_ACTIVE[i],
        recovery: t::CLAW_RECOVERY[i],
        damage: t::CLAW_DAMAGE[i],
        knockback: t::CLAW_KNOCKBACK[i],
        stun: t::CLAW_STUN[i],
        hitstop: t::CLAW_HITSTOP[i],
        shake: t::CLAW_SHAKE[i],
        ..BASE
    }
}

/// 새싹 발톱(가벼움): 4연타, 짧고 빠르다. 한 방은 약하지만 끊김 없이 이어진다.
pub static SPROUT: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.09,
            active: (0.015, 0.06),
            knockback: 55.0,
            stun: 0.10,
            hitstop: 0.05,
            shake: 0.08,
            range: 12.0,
            lunge: 32.0,
            ..BASE
        },
        Hit {
            swing: 0.09,
            active: (0.015, 0.06),
            knockback: 55.0,
            stun: 0.10,
            hitstop: 0.05,
            shake: 0.08,
            range: 12.0,
            lunge: 32.0,
            ..BASE
        },
        Hit {
            swing: 0.09,
            active: (0.015, 0.06),
            knockback: 55.0,
            stun: 0.10,
            hitstop: 0.05,
            shake: 0.08,
            range: 12.0,
            lunge: 32.0,
            ..BASE
        },
        Hit {
            swing: 0.16,
            active: (0.04, 0.11),
            recovery: 0.08,
            // 마무리 타 넉백을 두더지 발톱과 같은 비율로 올림(0090, 예전 150).
            knockback: 205.0,
            stun: 0.35,
            hitstop: 0.11,
            shake: 0.36,
            range: 13.0,
            lunge: 50.0,
            ..BASE
        },
    ],
    combo_window: 0.28,
    move_factor: 0.6,
    mark: Color::new(0.80, 0.98, 0.70, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 두더지 발톱(기본): 1·2타 빠름, 3타 느리고 묵직. 언제든 굴파기로 끊는다.
pub static CLAWS: Moveset = Moveset {
    hits: &[claw(0), claw(1), claw(2)],
    combo_window: t::CLAW_COMBO_WINDOW,
    move_factor: t::CLAW_MOVE_FACTOR,
    mark: Color::new(0.97, 0.93, 0.85, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 모모의 뒷발(0056): 앞발 톡톡 둘 + 몸을 돌려 뒷발 차기(보스 모모의 차기). 마지막 타가 멀리 밀친다.
pub static KICKS: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.11,
            active: (0.02, 0.07),
            knockback: 80.0,
            stun: 0.14,
            range: 13.0,
            lunge: 30.0,
            ..BASE
        },
        Hit {
            swing: 0.12,
            active: (0.03, 0.08),
            knockback: 95.0,
            stun: 0.16,
            range: 14.0,
            lunge: 36.0,
            ..BASE
        },
        Hit {
            swing: 0.26,
            active: (0.10, 0.17),
            recovery: 0.12,
            damage: 2,
            // 0090: 예전 260.
            knockback: 340.0,
            stun: 0.48,
            hitstop: 0.14,
            shake: 0.55,
            range: 17.0,
            lunge: 22.0,
            dodge_cancel_after: 0.17,
            ..BASE
        },
    ],
    combo_window: 0.30,
    move_factor: 0.5,
    mark: Color::new(1.0, 0.82, 0.88, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 도토의 던지기(0195): 1·2타는 곧게 하나, 3타는 부채꼴 셋. 손앞의 짧은 판정(`range`)은 받아치기용.
/// 던지는 동안 잘 걷는다(멀리서 싸우는 몸짓). 1·2타는 가볍고(피해 1), 가까운 적은 뒤로 뛰기로 떼어 낸다.
const fn acorn(count: u8, spread: f32) -> Throw {
    Throw::Shot {
        count,
        spread,
        speed: t::ACORN_SPEED,
        reach: t::ACORN_REACH,
        radius: t::ACORN_RADIUS,
        splash: 0.0,
        bounce: 0,
        split: 0,
        thorns: false,
    }
}

pub static ACORNS: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.15,
            active: (0.04, 0.10),
            knockback: 50.0,
            stun: 0.12,
            hitstop: 0.04,
            shake: 0.06,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: acorn(1, 0.0),
            ..BASE
        },
        Hit {
            swing: 0.15,
            active: (0.04, 0.10),
            knockback: 55.0,
            stun: 0.12,
            hitstop: 0.04,
            shake: 0.06,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: acorn(1, 0.0),
            ..BASE
        },
        Hit {
            swing: 0.24,
            active: (0.07, 0.14),
            recovery: 0.10,
            knockback: 120.0,
            stun: 0.25,
            hitstop: 0.07,
            shake: 0.22,
            range: t::ACORN_HAND,
            lunge: 0.0,
            dodge_cancel_after: 0.14,
            throw: acorn(3, t::ACORN_FAN_SPREAD),
            ..BASE
        },
    ],
    combo_window: 0.32,
    move_factor: 0.65,
    mark: Color::new(0.86, 0.62, 0.32, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 바위 발톱(무거움): 2타. 느리고 넓고, 활성 구간이 끝나야 굴파기로 끊는다.
pub static STONE: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.24,
            active: (0.10, 0.18),
            damage: 2,
            knockback: 170.0,
            stun: 0.3,
            hitstop: 0.10,
            shake: 0.3,
            range: 18.0,
            lunge: 55.0,
            dodge_cancel_after: 0.18,
            ..BASE
        },
        Hit {
            swing: 0.36,
            active: (0.16, 0.26),
            recovery: 0.26,
            damage: 3,
            // 0090: 예전 300.
            knockback: 390.0,
            stun: 0.6,
            hitstop: 0.18,
            shake: 0.7,
            range: 20.0,
            lunge: 75.0,
            dodge_cancel_after: 0.26,
            ..BASE
        },
    ],
    combo_window: 0.35,
    move_factor: 0.25,
    mark: Color::new(0.78, 0.76, 0.74, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 독 발톱: 3연타, 한 방은 가볍지만 맞은 적에게 독. 마지막 타는 독이 길다.
pub static VENOM: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.13,
            knockback: 70.0,
            stun: 0.12,
            hitstop: 0.06,
            shake: 0.12,
            lunge: 40.0,
            poison: t::VENOM_POISON,
            ..BASE
        },
        Hit {
            swing: 0.13,
            knockback: 70.0,
            stun: 0.12,
            hitstop: 0.06,
            shake: 0.12,
            lunge: 40.0,
            poison: t::VENOM_POISON,
            ..BASE
        },
        Hit {
            swing: 0.22,
            active: (0.05, 0.14),
            recovery: 0.12,
            knockback: 160.0,
            stun: 0.30,
            hitstop: 0.08,
            shake: 0.30,
            lunge: 45.0,
            poison: t::VENOM_POISON_FINISHER,
            ..BASE
        },
    ],
    combo_window: 0.30,
    move_factor: 0.45,
    mark: Color::new(0.62, 0.92, 0.40, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

// ---- 모모·도토 무기 셋씩(0246) -------------------------------------------------

/// 모모 가시덩굴 각반: 가볍고 빠른 4연타. 마무리 차기가 발 앞에 가시 덤불을 남긴다(밟은 적이 찔린다).
pub static BRIAR: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.09,
            active: (0.015, 0.06),
            knockback: 50.0,
            stun: 0.10,
            hitstop: 0.05,
            shake: 0.08,
            range: 13.0,
            lunge: 30.0,
            ..BASE
        },
        Hit {
            swing: 0.09,
            active: (0.015, 0.06),
            knockback: 50.0,
            stun: 0.10,
            hitstop: 0.05,
            shake: 0.08,
            range: 13.0,
            lunge: 30.0,
            ..BASE
        },
        Hit {
            swing: 0.10,
            active: (0.02, 0.07),
            knockback: 60.0,
            stun: 0.12,
            hitstop: 0.05,
            shake: 0.1,
            range: 14.0,
            lunge: 32.0,
            ..BASE
        },
        Hit {
            swing: 0.22,
            active: (0.08, 0.14),
            recovery: 0.10,
            knockback: 230.0,
            stun: 0.35,
            hitstop: 0.11,
            shake: 0.4,
            range: 16.0,
            lunge: 24.0,
            dodge_cancel_after: 0.14,
            thorns: true,
            ..BASE
        },
    ],
    combo_window: 0.28,
    move_factor: 0.55,
    mark: Color::new(0.62, 0.84, 0.46, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 모모 조약돌 덧신: 느리고 무거운 2연타. 마무리 차기는 등딱지에 튕기지 않고 연다.
pub static PEBBLE: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.24,
            active: (0.09, 0.16),
            damage: 2,
            knockback: 180.0,
            stun: 0.32,
            hitstop: 0.10,
            shake: 0.32,
            range: 17.0,
            lunge: 40.0,
            dodge_cancel_after: 0.16,
            ..BASE
        },
        Hit {
            swing: 0.36,
            active: (0.15, 0.24),
            recovery: 0.24,
            damage: 3,
            knockback: 400.0,
            stun: 0.6,
            hitstop: 0.18,
            shake: 0.75,
            range: 19.0,
            lunge: 30.0,
            dodge_cancel_after: 0.24,
            crush: true,
            ..BASE
        },
    ],
    combo_window: 0.36,
    move_factor: 0.25,
    mark: Color::new(0.74, 0.72, 0.70, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

/// 모모 꽃잎 발목끈: 가벼운 3연타. 박자 창이 넓고, 박자에 맞춘 차기가 기를 두 배로 채운다.
pub static PETAL: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.11,
            active: (0.02, 0.07),
            knockback: 70.0,
            stun: 0.12,
            range: 13.0,
            lunge: 28.0,
            ..BASE
        },
        Hit {
            swing: 0.12,
            active: (0.03, 0.08),
            knockback: 85.0,
            stun: 0.14,
            range: 14.0,
            lunge: 32.0,
            ..BASE
        },
        Hit {
            swing: 0.24,
            active: (0.09, 0.16),
            recovery: 0.10,
            damage: 2,
            knockback: 280.0,
            stun: 0.40,
            hitstop: 0.12,
            shake: 0.45,
            range: 16.0,
            lunge: 22.0,
            dodge_cancel_after: 0.16,
            ..BASE
        },
    ],
    combo_window: 0.32,
    move_factor: 0.5,
    mark: Color::new(1.0, 0.72, 0.84, 1.0),
    beat_window: t::PETAL_BEAT_WINDOW,
    beat_sp: t::PETAL_BEAT_SP,
};

/// 탄 하나 모양(도토 무기 공용): `acorn`을 바탕으로 속도·사거리·크기·튕김·갈라짐·가시만 바꾼다.
const fn shot(count: u8, spread: f32, speed: f32, reach: f32, radius: f32) -> Throw {
    Throw::Shot {
        count,
        spread,
        speed,
        reach,
        radius,
        splash: 0.0,
        bounce: 0,
        split: 0,
        thorns: false,
    }
}

const fn bouncing(t: Throw) -> Throw {
    match t {
        Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            split,
            thorns,
            ..
        } => Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            bounce: 1,
            split,
            thorns,
        },
        Throw::None => Throw::None,
    }
}

/// 도토 솔방울: 가볍고 멀리 난다. 벽에 한 번 튕긴다. 3타는 둘.
pub static PINECONE: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.12,
            active: (0.03, 0.08),
            knockback: 45.0,
            stun: 0.10,
            hitstop: 0.04,
            shake: 0.05,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: bouncing(shot(1, 0.0, t::PINE_SPEED, t::PINE_REACH, t::ACORN_RADIUS)),
            ..BASE
        },
        Hit {
            swing: 0.12,
            active: (0.03, 0.08),
            knockback: 50.0,
            stun: 0.10,
            hitstop: 0.04,
            shake: 0.05,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: bouncing(shot(1, 0.0, t::PINE_SPEED, t::PINE_REACH, t::ACORN_RADIUS)),
            ..BASE
        },
        Hit {
            swing: 0.20,
            active: (0.06, 0.12),
            recovery: 0.08,
            knockback: 110.0,
            stun: 0.22,
            hitstop: 0.06,
            shake: 0.18,
            range: t::ACORN_HAND,
            lunge: 0.0,
            dodge_cancel_after: 0.12,
            throw: bouncing(shot(2, 0.16, t::PINE_SPEED, t::PINE_REACH, t::ACORN_RADIUS)),
            ..BASE
        },
    ],
    combo_window: 0.30,
    move_factor: 0.7,
    mark: Color::new(0.62, 0.44, 0.26, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

const fn splitting(t: Throw, n: u8) -> Throw {
    match t {
        Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            bounce,
            thorns,
            ..
        } => Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            bounce,
            split: n,
            thorns,
        },
        Throw::None => Throw::None,
    }
}

/// 도토 호두: 느리고 무겁다(2타). 멈추는 자리(적·벽)에서 작은 조각 셋으로 갈라진다.
pub static WALNUT: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.24,
            active: (0.08, 0.14),
            damage: 2,
            knockback: 120.0,
            stun: 0.25,
            hitstop: 0.08,
            shake: 0.2,
            range: t::ACORN_HAND,
            lunge: 0.0,
            dodge_cancel_after: 0.14,
            throw: splitting(
                shot(1, 0.0, t::WALNUT_SPEED, t::WALNUT_REACH, t::WALNUT_RADIUS),
                3,
            ),
            ..BASE
        },
        Hit {
            swing: 0.32,
            active: (0.12, 0.18),
            recovery: 0.16,
            damage: 3,
            knockback: 220.0,
            stun: 0.45,
            hitstop: 0.12,
            shake: 0.4,
            range: t::ACORN_HAND,
            lunge: 0.0,
            dodge_cancel_after: 0.18,
            throw: splitting(
                shot(1, 0.0, t::WALNUT_SPEED, t::WALNUT_REACH, t::WALNUT_RADIUS),
                3,
            ),
            ..BASE
        },
    ],
    combo_window: 0.36,
    move_factor: 0.5,
    mark: Color::new(0.72, 0.58, 0.40, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};

const fn spiky(t: Throw) -> Throw {
    match t {
        Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            bounce,
            split,
            ..
        } => Throw::Shot {
            count,
            spread,
            speed,
            reach,
            radius,
            splash,
            bounce,
            split,
            thorns: true,
        },
        Throw::None => Throw::None,
    }
}

/// 도토 밤송이: 3타. 마무리 둘이 떨어진 자리에 가시를 남긴다(밟은 적이 찔린다).
pub static CHESTNUT: Moveset = Moveset {
    hits: &[
        Hit {
            swing: 0.16,
            active: (0.04, 0.10),
            knockback: 55.0,
            stun: 0.12,
            hitstop: 0.04,
            shake: 0.06,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: shot(1, 0.0, t::ACORN_SPEED, t::ACORN_REACH, t::ACORN_RADIUS),
            ..BASE
        },
        Hit {
            swing: 0.16,
            active: (0.04, 0.10),
            knockback: 60.0,
            stun: 0.12,
            hitstop: 0.04,
            shake: 0.06,
            range: t::ACORN_HAND,
            lunge: 0.0,
            throw: shot(1, 0.0, t::ACORN_SPEED, t::ACORN_REACH, t::ACORN_RADIUS),
            ..BASE
        },
        Hit {
            swing: 0.26,
            active: (0.08, 0.15),
            recovery: 0.10,
            knockback: 125.0,
            stun: 0.25,
            hitstop: 0.07,
            shake: 0.22,
            range: t::ACORN_HAND,
            lunge: 0.0,
            dodge_cancel_after: 0.15,
            throw: spiky(shot(
                2,
                0.22,
                t::ACORN_SPEED,
                t::CHESTNUT_REACH,
                t::ACORN_RADIUS,
            )),
            ..BASE
        },
    ],
    combo_window: 0.32,
    move_factor: 0.6,
    mark: Color::new(0.56, 0.36, 0.22, 1.0),
    beat_window: 1.0,
    beat_sp: 1.0,
};
