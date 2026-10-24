//! 입력 레이어. 키보드·게임패드를 `Actions`로 번역한다.
//!
//! 이 파일 밖에서는 `KeyCode`도 `gamepads::Button`도 보지 않는다.
//! (KB: reference/macroquad/gamepad-and-android-input.md)
//!
//! 고정 타임스텝과의 관계: 렌더 프레임마다 `poll()`로 상태를 읽고, "눌림(edge)"은 래치에 모아 둔다.
//! 로직 틱마다 `take()`가 래치를 소비한다. 프레임에 틱이 두 번이면 첫 틱만 눌림을 받고,
//! 틱이 없으면(고주사율) 다음 틱까지 눌림이 보존된다.

#[cfg(not(target_os = "android"))]
use gamepads::{Button, Gamepads};
use macroquad::prelude::*;

#[cfg(not(target_os = "android"))]
use crate::tuning;

/// 게임 로직이 보는 유일한 입력.
#[derive(Default, Debug, Clone, Copy)]
pub struct Actions {
    /// 이동 방향. 길이 0..=1 (스틱 기울기 반영). 입력 없으면 0.
    pub move_dir: Vec2,
    /// 조준 방향(오른쪽 스틱). 없으면 0. 공격 방향은 이게 있으면 이걸, 없으면 마지막 이동 방향.
    pub aim: Vec2,
    pub attack: bool,
    pub dodge: bool,
    pub confirm: bool,
    pub cancel: bool,
    pub restart: bool,
    pub toggle_debug: bool,
    pub toggle_hitboxes: bool,
}

impl Actions {
    fn merge_edges(&mut self, o: &Actions) {
        self.attack |= o.attack;
        self.dodge |= o.dodge;
        self.confirm |= o.confirm;
        self.cancel |= o.cancel;
        self.restart |= o.restart;
        self.toggle_debug |= o.toggle_debug;
        self.toggle_hitboxes |= o.toggle_hitboxes;
    }
}

pub struct Input {
    /// Android에서는 gamepads 크레이트가 빠진다. D-pad는 miniquad가 화살표 키로 넘겨 주므로 키보드 경로로 들어온다.
    #[cfg(not(target_os = "android"))]
    pads: Gamepads,
    /// 아직 틱이 소비하지 않은 눌림.
    pending: Actions,
    /// 가장 최근 프레임의 축 상태.
    held: Actions,
    pub gamepad_count: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "android"))]
            pads: Gamepads::new(),
            pending: Actions::default(),
            held: Actions::default(),
            gamepad_count: 0,
        }
    }

    /// 렌더 프레임마다 한 번.
    pub fn poll(&mut self) {
        let kb = read_keyboard();
        let (pad, count) = self.read_gamepads();
        self.gamepad_count = count;

        let mut now = kb;
        now.merge_edges(&pad);
        // 축: 키보드가 눌려 있으면 키보드, 아니면 스틱.
        if now.move_dir == Vec2::ZERO {
            now.move_dir = pad.move_dir;
        }
        now.aim = pad.aim;

        self.pending.merge_edges(&now);
        self.held = now;
    }

    #[cfg(not(target_os = "android"))]
    fn read_gamepads(&mut self) -> (Actions, usize) {
        self.pads.poll();
        read_gamepads(&self.pads)
    }

    #[cfg(target_os = "android")]
    fn read_gamepads(&mut self) -> (Actions, usize) {
        (Actions::default(), 0)
    }

    /// 로직 틱마다 한 번. 눌림을 소비한다.
    pub fn take(&mut self) -> Actions {
        let mut a = self.pending;
        a.move_dir = self.held.move_dir;
        a.aim = self.held.aim;
        self.pending = Actions::default();
        a
    }
}

fn read_keyboard() -> Actions {
    let mut m = Vec2::ZERO;
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        m.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        m.x += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        m.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        m.y += 1.0;
    }
    Actions {
        move_dir: m.normalize_or_zero(),
        aim: Vec2::ZERO,
        attack: is_key_pressed(KeyCode::J) || is_key_pressed(KeyCode::Z),
        dodge: is_key_pressed(KeyCode::K)
            || is_key_pressed(KeyCode::X)
            || is_key_pressed(KeyCode::Space),
        confirm: is_key_pressed(KeyCode::Enter),
        cancel: is_key_pressed(KeyCode::Escape),
        restart: is_key_pressed(KeyCode::R),
        toggle_debug: is_key_pressed(KeyCode::F1),
        toggle_hitboxes: is_key_pressed(KeyCode::F2),
    }
}

/// 연결된 모든 패드를 합친다(1인 게임이라 어느 패드든 같은 플레이어).
/// 스틱 y는 두 백엔드(gilrs, 브라우저 플러그인) 모두 위가 +이므로 화면 좌표(아래가 +)로 뒤집는다.
#[cfg(not(target_os = "android"))]
fn read_gamepads(pads: &Gamepads) -> (Actions, usize) {
    let mut a = Actions::default();
    let mut count = 0;
    for pad in pads.all() {
        count += 1;
        let (lx, ly) = pad.left_stick();
        let mut mv = vec2(lx, -ly);
        if pad.is_currently_pressed(Button::DPadLeft) {
            mv.x -= 1.0;
        }
        if pad.is_currently_pressed(Button::DPadRight) {
            mv.x += 1.0;
        }
        if pad.is_currently_pressed(Button::DPadUp) {
            mv.y -= 1.0;
        }
        if pad.is_currently_pressed(Button::DPadDown) {
            mv.y += 1.0;
        }
        let mv = apply_deadzone(mv);
        if mv != Vec2::ZERO {
            a.move_dir = mv;
        }
        let (rx, ry) = pad.right_stick();
        let aim = apply_deadzone(vec2(rx, -ry));
        if aim != Vec2::ZERO {
            a.aim = aim.normalize();
        }
        // 공격 X/□ 또는 R1, 굴파기 A/× 또는 R2.
        a.attack |=
            pad.is_just_pressed(Button::ActionLeft) || pad.is_just_pressed(Button::FrontRightUpper);
        a.dodge |=
            pad.is_just_pressed(Button::ActionDown) || pad.is_just_pressed(Button::FrontRightLower);
        a.confirm |= pad.is_just_pressed(Button::ActionDown);
        a.cancel |= pad.is_just_pressed(Button::ActionRight)
            || pad.is_just_pressed(Button::LeftCenterCluster);
        a.restart |= pad.is_just_pressed(Button::RightCenterCluster);
    }
    (a, count)
}

/// 반지름 데드존. 데드존 바깥을 0..1로 다시 펴서 작은 기울기도 쓸 수 있게 한다.
#[cfg(not(target_os = "android"))]
fn apply_deadzone(v: Vec2) -> Vec2 {
    let len = v.length();
    if len < tuning::STICK_DEADZONE {
        return Vec2::ZERO;
    }
    let scaled = ((len - tuning::STICK_DEADZONE) / (1.0 - tuning::STICK_DEADZONE)).min(1.0);
    v / len * scaled
}
