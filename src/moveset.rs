//! 무브셋: 무기·캐릭터마다 다른 콤보를 데이터로. 플레이어 코드는 이 표만 읽는다.
//!
//! 규칙(docs/decisions/0009): 마지막 타가 가장 강하고 느리다. 굴파기(회피)는 각 타의 `dodge_cancel_after` 뒤에
//! 스윙을 끊을 수 있어 후딜레이를 취소하고 다음 콤보로 섞는다. 공격 입력은 활성 구간이 끝난 뒤 다음 타로 이어진다.
//! 수치의 출처는 tuning.rs(발톱). 새 무기는 여기에 `Moveset` 상수 하나.

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
}

#[derive(Debug, Clone, Copy)]
pub struct Moveset {
    pub name: &'static str,
    pub hits: &'static [Hit],
    /// 스윙이 끝난 뒤 다음 타 입력이 유효한 시간.
    pub combo_window: f32,
    /// 공격 중 이동 속도 배율.
    pub move_factor: f32,
}

impl Moveset {
    pub fn last(&self) -> usize {
        self.hits.len() - 1
    }
}

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
        range: t::CLAW_RANGE,
        lunge: t::CLAW_LUNGE,
        dodge_cancel_after: 0.0,
    }
}

/// Naïl의 발톱 3연타. 1·2타 빠름, 3타 느리고 묵직. 언제든 굴파기로 끊는다.
pub static CLAWS: Moveset = Moveset {
    name: "발톱",
    hits: &[claw(0), claw(1), claw(2)],
    combo_window: t::CLAW_COMBO_WINDOW,
    move_factor: t::CLAW_MOVE_FACTOR,
};

/// 실험용 무거운 무기(2타). 느리고 넓고, 활성 구간이 끝나야 굴파기로 끊을 수 있다.
/// 무기가 바뀌면 손맛이 얼마나 달라지는지 보는 용도.
pub static HEAVY: Moveset = Moveset {
    name: "무거운 발톱(실험)",
    hits: &[
        Hit {
            swing: 0.24,
            active: (0.10, 0.18),
            recovery: 0.0,
            damage: 2,
            knockback: 170.0,
            stun: 0.3,
            hitstop: 0.08,
            shake: 0.3,
            range: 18.0,
            lunge: 55.0,
            dodge_cancel_after: 0.18,
        },
        Hit {
            swing: 0.36,
            active: (0.16, 0.26),
            recovery: 0.26,
            damage: 3,
            knockback: 300.0,
            stun: 0.55,
            hitstop: 0.14,
            shake: 0.6,
            range: 20.0,
            lunge: 75.0,
            dodge_cancel_after: 0.26,
        },
    ],
    combo_window: 0.35,
    move_factor: 0.25,
};
