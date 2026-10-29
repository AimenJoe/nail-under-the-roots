//! 입력 레이어. 키보드·게임패드·터치·마우스를 `Actions`로 번역한다.
//!
//! 이 파일 밖에서는 `KeyCode`도 `gamepads::Button`도 보지 않는다.
//! (KB: reference/macroquad/gamepad-and-android-input.md)
//!
//! 게임 동작은 바꿀 수 있다(`Bindings`, 설정 → 조작, docs/decisions/0022). 기본: 오른쪽 네 버튼 = 자주 쓰는 넷
//! (X 공격 · A 굴파기 · B 필살기 · Y 아이템), 덜 쓰는 것은 어깨·가운데(LB/RB 아이템 넘기기, Select 지도, Start 메뉴).
//! 키보드는 같은 모양의 다이아몬드 J(공격)·K(굴파기)·L(필살기)·I(아이템). 메뉴 확인·뒤로·이동은 고정.
//! 화면의 버튼 안내는 `prompt()`로 지금 배치를 읽는다(패드가 있으면 패드 이름, 없으면 키).
//!
//! 고정 타임스텝과의 관계: 렌더 프레임마다 `poll()`로 상태를 읽고, "눌림(edge)"은 래치에 모아 둔다.
//! 로직 틱마다 `take()`가 래치를 소비한다. 프레임에 틱이 두 번이면 첫 틱만 눌림을 받고,
//! 틱이 없으면(고주사율) 다음 틱까지 눌림이 보존된다. 메뉴 장면은 프레임마다 한 번 `take()`한다.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

#[cfg(not(target_os = "android"))]
use gamepads::{Button, Gamepads};
use macroquad::prelude::*;

use crate::tuning;

/// 바꿀 수 있는 게임 동작.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Attack,
    Dodge,
    Special,
    UseItem,
    ItemPrev,
    ItemNext,
    Map,
}

pub const ACTS: [Act; 7] = [
    Act::Attack,
    Act::Dodge,
    Act::Special,
    Act::UseItem,
    Act::ItemPrev,
    Act::ItemNext,
    Act::Map,
];

/// 패드 버튼(W3C 표준 배치 위치). gamepads 크레이트가 없는 Android에서도 컴파일되게 따로 둔다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pad {
    South,
    East,
    West,
    North,
    L1,
    R1,
    L2,
    R2,
    Select,
    Start,
    L3,
    R3,
}

/// 저장·캡처 순서.
pub const PADS: [Pad; 12] = [
    Pad::South,
    Pad::East,
    Pad::West,
    Pad::North,
    Pad::L1,
    Pad::R1,
    Pad::L2,
    Pad::R2,
    Pad::Select,
    Pad::Start,
    Pad::L3,
    Pad::R3,
];

impl Pad {
    /// 화면 이름(Xbox 배치 글자. 소니·닌텐도도 위치는 같다).
    pub fn label(self) -> &'static str {
        match self {
            Pad::South => "A",
            Pad::East => "B",
            Pad::West => "X",
            Pad::North => "Y",
            Pad::L1 => "LB",
            Pad::R1 => "RB",
            Pad::L2 => "LT",
            Pad::R2 => "RT",
            Pad::Select => "Select",
            Pad::Start => "Start",
            Pad::L3 => "L3",
            Pad::R3 => "R3",
        }
    }

    #[cfg(not(target_os = "android"))]
    fn button(self) -> Button {
        match self {
            Pad::South => Button::ActionDown,
            Pad::East => Button::ActionRight,
            Pad::West => Button::ActionLeft,
            Pad::North => Button::ActionUp,
            Pad::L1 => Button::FrontLeftUpper,
            Pad::R1 => Button::FrontRightUpper,
            Pad::L2 => Button::FrontLeftLower,
            Pad::R2 => Button::FrontRightLower,
            Pad::Select => Button::LeftCenterCluster,
            Pad::Start => Button::RightCenterCluster,
            Pad::L3 => Button::LeftStick,
            Pad::R3 => Button::RightStick,
        }
    }
}

/// Android: 패치한 miniquad가 패드 버튼 키코드를 쓰지 않는 F13~F25로 넘긴다(third_party/miniquad/PATCHED.md, 0075).
/// 테스트가 패치 표와 맞는지 보려고 테스트 빌드에서도 컴파일한다.
#[cfg(any(target_os = "android", test))]
impl Pad {
    fn key(self) -> KeyCode {
        match self {
            Pad::South => KeyCode::F13,
            Pad::East => KeyCode::F14,
            Pad::West => KeyCode::F15,
            Pad::North => KeyCode::F16,
            Pad::L1 => KeyCode::F17,
            Pad::R1 => KeyCode::F18,
            Pad::L2 => KeyCode::F19,
            Pad::R2 => KeyCode::F20,
            Pad::L3 => KeyCode::F21,
            Pad::R3 => KeyCode::F22,
            Pad::Start => KeyCode::F23,
            Pad::Select => KeyCode::F24,
        }
    }
}

/// 바꿀 수 있는 키(저장 번호 = 이 표의 자리). 이동(WASD·화살표)·Enter·Esc는 고정이라 없다.
const KEYS: [(KeyCode, &str); 44] = [
    (KeyCode::J, "J"),
    (KeyCode::K, "K"),
    (KeyCode::L, "L"),
    (KeyCode::I, "I"),
    (KeyCode::Q, "Q"),
    (KeyCode::E, "E"),
    (KeyCode::M, "M"),
    (KeyCode::Space, "Space"),
    (KeyCode::B, "B"),
    (KeyCode::C, "C"),
    (KeyCode::F, "F"),
    (KeyCode::G, "G"),
    (KeyCode::H, "H"),
    (KeyCode::N, "N"),
    (KeyCode::O, "O"),
    (KeyCode::P, "P"),
    (KeyCode::R, "R"),
    (KeyCode::T, "T"),
    (KeyCode::U, "U"),
    (KeyCode::V, "V"),
    (KeyCode::X, "X"),
    (KeyCode::Y, "Y"),
    (KeyCode::Z, "Z"),
    (KeyCode::Key1, "1"),
    (KeyCode::Key2, "2"),
    (KeyCode::Key3, "3"),
    (KeyCode::Key4, "4"),
    (KeyCode::Key5, "5"),
    (KeyCode::Key6, "6"),
    (KeyCode::Key7, "7"),
    (KeyCode::Key8, "8"),
    (KeyCode::Key9, "9"),
    (KeyCode::Key0, "0"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Slash, "/"),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Apostrophe, "'"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::LeftShift, "Shift"),
    (KeyCode::RightShift, "RShift"),
    (KeyCode::LeftControl, "Ctrl"),
    (KeyCode::LeftAlt, "Alt"),
    (KeyCode::Backspace, "Bksp"),
];

/// 동작마다 키 하나, 패드 버튼 하나(KEYS·PADS의 번호).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bindings {
    pub keys: [u8; 7],
    pub pads: [u8; 7],
}

impl Bindings {
    /// J K L I / Q E / M,  X A B Y / LB RB / Select.
    pub const DEFAULT: Bindings = Bindings {
        keys: [0, 1, 2, 3, 4, 5, 6],
        pads: [2, 0, 1, 3, 4, 5, 8],
    };

    fn idx(a: Act) -> usize {
        ACTS.iter().position(|x| *x == a).unwrap_or(0)
    }

    pub fn key(&self, a: Act) -> KeyCode {
        KEYS[self.keys[Self::idx(a)] as usize % KEYS.len()].0
    }

    pub fn pad(&self, a: Act) -> Pad {
        PADS[self.pads[Self::idx(a)] as usize % PADS.len()]
    }

    pub fn key_label(&self, a: Act) -> &'static str {
        KEYS[self.keys[Self::idx(a)] as usize % KEYS.len()].1
    }

    /// 키를 동작에 붙인다. 다른 동작이 쓰던 키면 서로 바꾼다. 표에 없는 키(이동·Esc 등)면 false.
    pub fn bind_key(&mut self, a: Act, k: KeyCode) -> bool {
        let Some(n) = KEYS.iter().position(|(kk, _)| *kk == k) else {
            return false;
        };
        let i = Self::idx(a);
        if let Some(j) = self.keys.iter().position(|x| *x as usize == n) {
            self.keys[j] = self.keys[i];
        }
        self.keys[i] = n as u8;
        true
    }

    /// 패드 버튼을 붙인다. Start는 메뉴 고정이라 안 된다. 겹치면 서로 바꾼다.
    pub fn bind_pad(&mut self, a: Act, p: Pad) -> bool {
        if p == Pad::Start {
            return false;
        }
        let n = PADS.iter().position(|x| *x == p).unwrap_or(0);
        let i = Self::idx(a);
        if let Some(j) = self.pads.iter().position(|x| *x as usize == n) {
            self.pads[j] = self.pads[i];
        }
        self.pads[i] = n as u8;
        true
    }

    /// 저장된 번호가 표 밖이거나 겹치면 기본값.
    pub fn sanitized(self) -> Bindings {
        let ok_k = self.keys.iter().all(|k| (*k as usize) < KEYS.len());
        let ok_p = self
            .pads
            .iter()
            .all(|p| (*p as usize) < PADS.len() && PADS[*p as usize] != Pad::Start);
        let uniq = |v: &[u8; 7]| (0..7).all(|i| (i + 1..7).all(|j| v[i] != v[j]));
        if ok_k && ok_p && uniq(&self.keys) && uniq(&self.pads) {
            self
        } else {
            Bindings::DEFAULT
        }
    }
}

static BINDINGS: RwLock<Bindings> = RwLock::new(Bindings::DEFAULT);
/// 마지막으로 패드를 썼나(안내를 패드 이름으로).
/// 지금 쓰는 조작 장치. 안내 글(버튼 이름·"확인" 같은 동사)이 이걸 따른다(사람 피드백 2026-10-25).
/// 마지막으로 실제로 누른 장치: 키를 누르면 키보드, 패드를 만지면 패드, 화면을 만지면 터치.
/// 마우스 클릭은 바꾸지 않는다 — 모바일 브라우저는 터치마다 가짜 마우스 클릭도 만든다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keys,
    Pad,
    Touch,
}

static DEVICE: AtomicU8 = AtomicU8::new(0);

pub fn device() -> Device {
    match DEVICE.load(Ordering::Relaxed) {
        1 => Device::Pad,
        2 => Device::Touch,
        _ => Device::Keys,
    }
}

fn set_device(d: Device) {
    DEVICE.store(d as u8, Ordering::Relaxed);
}

/// 처음 장치: 손가락이 주 포인터인 기기(폰·태블릿 브라우저, Android)면 터치. 아무것도 누르기 전의 첫 화면(인트로)용.
fn initial_device() -> Device {
    #[cfg(target_arch = "wasm32")]
    {
        extern "C" {
            fn nail_coarse_pointer() -> u32;
        }
        // SAFETY: 인자 없이 정수를 돌려주는 JS 함수(web/index.html의 nail_device 플러그인).
        if unsafe { nail_coarse_pointer() } != 0 {
            return Device::Touch;
        }
        Device::Keys
    }
    #[cfg(target_os = "android")]
    {
        Device::Touch
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        Device::Keys
    }
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_device_crate_version() -> u32 {
    1
}

/// 메뉴·안내 글의 동사 자리. 장치마다 이름이 다르다(키 이름, 패드 버튼, 터치는 낱말).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    /// 메뉴에서 고르기(확인).
    Confirm,
    /// 메뉴에서 뒤로·닫기.
    Back,
    /// 게임 중 일시정지 메뉴.
    Menu,
    /// 게임 중 걷기.
    Walk,
    /// 메뉴에서 위아래로 고르기.
    Pick,
    /// 설정 값 좌우로 바꾸기.
    Change,
    /// 일시정지 탭 넘기기.
    Tabs,
    /// 컷씬 건너뛰기.
    Skip,
    /// 쓰러진 뒤 다시 시작(게임 중 확인 = 공격 버튼).
    Restart,
}

/// 동사 자리의 지금 장치 이름.
pub fn verb(v: Verb) -> &'static str {
    let s = crate::i18n::s();
    match device() {
        Device::Keys => match v {
            Verb::Confirm | Verb::Restart => "Enter",
            Verb::Back | Verb::Menu | Verb::Skip => "Esc",
            Verb::Walk => "WASD",
            Verb::Pick => "↑↓",
            Verb::Change => "←→",
            Verb::Tabs => "Q/E",
        },
        Device::Pad => match v {
            Verb::Confirm | Verb::Restart => Pad::South.label(),
            Verb::Back | Verb::Skip => Pad::East.label(),
            Verb::Menu => Pad::Start.label(),
            Verb::Walk => s.v_stick,
            Verb::Pick => "↑↓",
            Verb::Change => "←→",
            Verb::Tabs => "LB/RB",
        },
        Device::Touch => match v {
            Verb::Confirm | Verb::Pick | Verb::Tabs => s.v_touch,
            Verb::Back => s.v_touch_out,
            Verb::Menu => s.tab_menu,
            Verb::Walk => s.v_drag,
            Verb::Change => "◀▶",
            Verb::Skip => s.v_touch_here,
            Verb::Restart => s.act_attack,
        },
    }
}

/// 동사 이름이 누르는 것(키·패드 버튼·터치 화면 버튼)인가 — 그렇다면 안내 글에서 키캡으로 그린다(0153).
/// 터치의 "탭"·"바깥 탭"·"끌기"는 손짓이라 글 그대로, 화면 버튼(메뉴·공격)은 키캡.
pub fn verb_is_button(v: Verb) -> bool {
    device() != Device::Touch || matches!(v, Verb::Menu | Verb::Restart)
}

pub fn bindings() -> Bindings {
    BINDINGS.read().map(|b| *b).unwrap_or(Bindings::DEFAULT)
}

pub fn set_bindings(b: Bindings) {
    if let Ok(mut g) = BINDINGS.write() {
        *g = b.sanitized();
    }
}

/// 화면 안내용 버튼 이름: 지금 쓰는 장치(키·패드·터치 버튼 글자)에 맞춰.
pub fn prompt(a: Act) -> &'static str {
    let b = bindings();
    match device() {
        Device::Pad => b.pad(a).label(),
        Device::Keys => b.key_label(a),
        // 터치 버튼은 그림(0172)이라 글에는 동작 이름을 쓴다(설정 화면의 이름과 같다).
        Device::Touch => {
            let s = crate::i18n::s();
            match a {
                Act::Attack => s.act_attack,
                Act::Dodge => s.act_dodge,
                Act::Special => s.act_special,
                Act::UseItem => s.act_use_item,
                Act::ItemPrev | Act::ItemNext => "◀▶",
                // 터치에는 지도 버튼이 없다: 메뉴 → 지도 탭.
                Act::Map => s.tab_menu,
            }
        }
    }
}

/// 설정에서 다시 붙이기: 이번 프레임에 새로 눌린 키(표에 있는 것)나 패드 버튼.
#[derive(Clone, Copy, Debug)]
pub enum Captured {
    Key(KeyCode),
    Pad(Pad),
}

/// 게임 로직과 메뉴가 보는 유일한 입력.
#[derive(Default, Debug, Clone, Copy)]
pub struct Actions {
    /// 이동 방향. 길이 0..=1 (스틱 기울기 반영). 입력 없으면 0.
    pub move_dir: Vec2,
    /// 조준 방향(오른쪽 스틱). 없으면 0. 공격 방향은 이게 있으면 이걸, 없으면 마지막 이동 방향.
    pub aim: Vec2,
    pub attack: bool,
    /// 공격 버튼을 누르고 있다(지금 상태). 평타 뒤에도 누르고 있으면 강공격을 모은다(0161).
    pub attack_held: bool,
    pub dodge: bool,
    /// 굴파기 버튼을 누르고 있다(눌림 에지가 아니라 지금 상태). 누르고 있는 동안 땅속에 더 머문다(0101).
    pub dodge_held: bool,
    /// 필살기(SP가 차면). L / B(○)·L2 / 터치 SP.
    pub special: bool,
    /// 퀵 아이템 먹기. F / Y(△) / 터치 ITEM.
    pub use_item: bool,
    /// 퀵 슬롯 넘기기(앞·뒤). Q·E / LB·RB / 터치 <>(뒤로).
    pub item_prev: bool,
    pub item_next: bool,
    /// 지도 바로 열기. M / Select.
    pub map: bool,
    /// 메뉴 확인. Enter / A(×) / 터치 ATK.
    pub confirm: bool,
    /// 메뉴 뒤로. Esc / B(○).
    pub cancel: bool,
    /// 일시정지 메뉴 열기. Esc / Start / Select / 터치 MENU.
    pub menu: bool,
    pub restart: bool,
    /// 탭 이동. Q·E / L1·R1.
    pub tab_prev: bool,
    pub tab_next: bool,
    /// 메뉴 탐색 에지. 누르고 있으면 일정 간격으로 반복된다.
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub toggle_debug: bool,
    pub toggle_hitboxes: bool,
    /// 이번 프레임의 탭/클릭 위치(화면 논리 px). 메뉴 모드의 터치(손가락을 움직이지 않고 뗐을 때)와 마우스 왼쪽 클릭.
    pub tap: Option<Vec2>,
    /// 메뉴에서 손가락으로 끌어 올린 거리(화면 논리 px, 위로 끌면 +: 아래 줄이 드러난다, 0199).
    pub drag: f32,
    /// 마우스 휠 칸(아래로 굴리면 +).
    pub wheel: i32,
}

impl Actions {
    fn merge_edges(&mut self, o: &Actions) {
        self.attack |= o.attack;
        self.dodge |= o.dodge;
        self.special |= o.special;
        self.use_item |= o.use_item;
        self.item_prev |= o.item_prev;
        self.item_next |= o.item_next;
        self.map |= o.map;
        self.confirm |= o.confirm;
        self.cancel |= o.cancel;
        self.menu |= o.menu;
        self.restart |= o.restart;
        self.tab_prev |= o.tab_prev;
        self.tab_next |= o.tab_next;
        self.up |= o.up;
        self.down |= o.down;
        self.left |= o.left;
        self.right |= o.right;
        self.toggle_debug |= o.toggle_debug;
        self.toggle_hitboxes |= o.toggle_hitboxes;
        if self.tap.is_none() {
            self.tap = o.tap;
        }
        self.drag += o.drag;
        self.wheel += o.wheel;
    }
}

pub struct Input {
    /// Android에서는 gamepads 크레이트가 빠진다. 패드 버튼은 패치한 miniquad가 F13~F25 키로(0075),
    /// 스틱·HAT D패드는 축으로(joystick_axes, 0143), 키코드로 오는 D패드는 화살표 키로 넘긴다.
    #[cfg(not(target_os = "android"))]
    pads: Gamepads,
    /// Android: 패드 버튼을 한 번이라도 받았나(연결된 패드 수 대신).
    #[cfg(target_os = "android")]
    pad_seen: bool,
    /// 아직 틱이 소비하지 않은 눌림.
    pending: Actions,
    /// 가장 최근 프레임의 축 상태.
    held: Actions,
    pub gamepad_count: usize,
    touch: TouchState,
    nav: NavRepeat,
    /// 메뉴 모드: 터치는 가상 스틱·버튼 대신 탭이 된다.
    menu_mode: bool,
}

/// 메뉴 탐색: 이동 방향(키·D패드·스틱)을 한 칸 눌림으로 바꾼다. 방향이 잡히는 순간 한 칸,
/// 누르고 있으면 NAV_REPEAT_DELAY 뒤부터 NAV_REPEAT_EVERY마다. 스틱은 큰 축 하나만 보고,
/// NAV_PRESS를 넘어야 잡히고 NAV_RELEASE 밑으로 내려와야 풀린다(히스테리시스: 경계에서 떨려도 한 칸만).
#[derive(Default)]
struct NavRepeat {
    dir: (i8, i8),
    timer: f32,
}

/// 터치 조작(모바일 브라우저 보조). 왼쪽 반은 떠다니는 가상 스틱, 오른쪽 아래는 버튼.
/// 화면 좌표(논리 px) 기준. 렌더가 `overlay()`로 읽어 그린다.
#[derive(Default)]
struct TouchState {
    stick: Option<TouchStick>,
    /// 지난 프레임까지 본 터치 id. 처음 보는 id가 "눌림"이다.
    /// (`TouchPhase::Started`에 기대면 안 된다: 실제 손가락은 touchstart 직후 touchmove가 같은 프레임에 들어와
    /// macroquad의 터치 맵에서 Started가 Moved로 덮인다.)
    known: Vec<u64>,
    /// 굴파기 버튼 위에서 시작해 아직 떼지 않은 손가락(누르고 있기, 0101).
    dig: Option<u64>,
    /// 공격 버튼 위에서 시작해 아직 떼지 않은 손가락(강공격 모으기, 0161).
    atk: Option<u64>,
    /// 이번 프레임에 화면에 손가락이 있었다(버튼·스틱이 아니어도). 장치 = 터치로 본다(0036·0172).
    active: bool,
    /// 메뉴 모드에서 짚은 손가락(0199): 움직이지 않고 떼면 탭, 움직이면 끌어 굴리기.
    menu: Option<MenuFinger>,
}

#[derive(Clone, Copy)]
struct MenuFinger {
    id: u64,
    start: Vec2,
    last: Vec2,
    dragging: bool,
}

#[derive(Clone, Copy)]
struct TouchStick {
    id: u64,
    origin: Vec2,
    pos: Vec2,
}

/// 렌더용 터치 오버레이 정보.
#[derive(Clone, Copy)]
pub struct TouchOverlay {
    pub stick: Option<(Vec2, Vec2)>,
    pub stick_radius: f32,
    pub buttons: [TouchButton; 6],
}

/// 터치 버튼이 하는 일. 그림(픽토그램)은 렌더가 캐릭터 표(`characters::Icon`)에서 고른다(0172).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TouchKind {
    Attack,
    Dodge,
    Menu,
    Special,
    Item,
    ItemSwitch,
}

#[derive(Clone, Copy)]
pub struct TouchButton {
    pub center: Vec2,
    pub radius: f32,
    pub kind: TouchKind,
    /// 다시 쓸 수 있기까지의 진행(1 = 지금 쓸 수 있음). 앱이 월드의 대기를 넣는다(docs/decisions/0038).
    pub ready: f32,
    /// 대기와 별개로 지금 쓸 수 없다(고른 아이템이 없다). 그림을 옅게.
    pub dim: bool,
}

impl TouchOverlay {
    /// 그 일을 하는 버튼. 칸 번호로 고르면 배치 순서를 바꿀 때 어긋난다(아이템 대기가 넘기기 버튼에, 필살기 게이지가
    /// 아이템 버튼에 그려졌다 — 사장 2026-10-29).
    pub fn button_mut(&mut self, kind: TouchKind) -> &mut TouchButton {
        self.buttons
            .iter_mut()
            .find(|b| b.kind == kind)
            .expect("터치 버튼은 일마다 하나")
    }
}

/// 버튼 배치(논리 px). 화면 짧은 변 기준으로 손가락 크기에 맞춘다.
fn touch_layout() -> TouchOverlay {
    layout_for(screen_width(), screen_height())
}

/// 오른손 엄지가 쉬는 오른쪽 아래에 공격(가장 크게), 그 둘레 호를 따라 굴파기·필살기·아이템을 둔다.
/// 공격은 가장 자주 누르는 버튼이라 엄지 크기(지름 ≈ 짧은 변의 27%, 가로 폰에서 약 100pt)로 —
/// 예전엔 굴파기와 같은 크기(약 70pt)라 작다는 피드백(2026-10-25). 플랫폼 최소 터치 크기(44pt·48dp)는 바닥일 뿐이다.
fn layout_for(w: f32, h: f32) -> TouchOverlay {
    let u = w.min(h);
    let r = u * 0.09;
    let atk_r = r * 1.5;
    let dig_r = r * 1.1;
    let sp_r = r * 0.85;
    let atk = vec2(w - atk_r * 1.3, h - atk_r * 1.3);
    // 굴파기: 공격 왼쪽, 조금 아래(엄지가 왼쪽으로 쓸 때 닿는 자리).
    let dig = vec2(atk.x - (atk_r + dig_r + r * 0.5), h - dig_r * 1.2);
    // 필살기: 공격 바로 위.
    let sp = vec2(atk.x, atk.y - (atk_r + sp_r + r * 0.45));
    TouchOverlay {
        stick: None,
        stick_radius: u * 0.14,
        buttons: [
            TouchButton {
                center: atk,
                radius: atk_r,
                kind: TouchKind::Attack,
                ready: 1.0,
                dim: false,
            },
            TouchButton {
                center: dig,
                radius: dig_r,
                kind: TouchKind::Dodge,
                ready: 1.0,
                dim: false,
            },
            TouchButton {
                center: vec2(r * 1.2, r * 1.2),
                radius: r * 0.75,
                kind: TouchKind::Menu,
                ready: 1.0,
                dim: false,
            },
            TouchButton {
                center: sp,
                radius: sp_r,
                kind: TouchKind::Special,
                ready: 1.0,
                dim: false,
            },
            TouchButton {
                // 아이템: 굴파기와 필살기 사이, 왼쪽 위 대각선.
                center: vec2(dig.x + r * 0.2, sp.y + r * 0.4),
                radius: r * 0.8,
                kind: TouchKind::Item,
                ready: 1.0,
                dim: false,
            },
            TouchButton {
                center: vec2(dig.x - r * 2.0, sp.y - r * 0.2),
                radius: r * 0.55,
                kind: TouchKind::ItemSwitch,
                ready: 1.0,
                dim: false,
            },
        ],
    }
}

impl Input {
    /// 이번 프레임에 전체 화면 키(F11, 데스크톱, 0240)를 눌렀나. 창 일이라 `Actions`(게임 동작)에 넣지 않는다.
    pub fn fullscreen_pressed(&self) -> bool {
        crate::DESKTOP && is_key_pressed(KeyCode::F11)
    }

    pub fn new() -> Self {
        // 터치를 마우스로 흉내 내지 않는다: 메뉴의 탭·끌기는 read_touch가 맡는다(0199).
        simulate_mouse_with_touch(false);
        set_device(initial_device());
        Self {
            #[cfg(not(target_os = "android"))]
            pads: Gamepads::new(),
            #[cfg(target_os = "android")]
            pad_seen: false,
            pending: Actions::default(),
            held: Actions::default(),
            gamepad_count: 0,
            touch: TouchState::default(),
            nav: NavRepeat::default(),
            menu_mode: false,
        }
    }

    pub fn set_menu_mode(&mut self, on: bool) {
        if self.menu_mode != on {
            self.menu_mode = on;
            self.touch.stick = None;
            self.touch.menu = None;
        }
    }

    /// 지금 쓰는 장치가 터치이고(마지막으로 만진 것이 화면, 0036) 게임 모드면 오버레이 정보를 돌려준다.
    /// 패드·키보드를 만지면 바로 사라진다(0172).
    pub fn overlay(&self) -> Option<TouchOverlay> {
        if device() != Device::Touch || self.menu_mode {
            return None;
        }
        let mut o = touch_layout();
        o.stick = self.touch.stick.map(|s| (s.origin, s.pos));
        Some(o)
    }

    /// 렌더 프레임마다 한 번.
    pub fn poll(&mut self) {
        let kb = read_keyboard();
        let (pad, count) = self.read_gamepads();
        self.gamepad_count = count;
        // 안내 글: 패드를 만지면 패드 이름, 키를 누르면 키 이름, 화면을 만지면 터치 글.
        if pad.move_dir != Vec2::ZERO || any_edge(&pad) {
            set_device(Device::Pad);
        } else if kb.move_dir != Vec2::ZERO || any_edge(&kb) {
            set_device(Device::Keys);
        }
        let touch = self.read_touch();
        if self.touch.active
            || touch.tap.is_some()
            || touch.move_dir != Vec2::ZERO
            || any_edge(&touch)
        {
            set_device(Device::Touch);
        }
        let mouse = read_mouse();

        let mut now = kb;
        now.merge_edges(&pad);
        now.merge_edges(&touch);
        now.merge_edges(&mouse);
        // 축: 키보드 > 게임패드 > 터치 스틱.
        if now.move_dir == Vec2::ZERO {
            now.move_dir = pad.move_dir;
        }
        if now.move_dir == Vec2::ZERO {
            now.move_dir = touch.move_dir;
        }
        now.aim = pad.aim;
        now.dodge_held = kb.dodge_held || pad.dodge_held || touch.dodge_held;
        now.attack_held = kb.attack_held || pad.attack_held || touch.attack_held;

        // 메뉴 탐색 에지(반복 포함)는 이동 방향에서 만든다. 키·D패드의 눌림 에지(아래 read_*)와
        // 같은 프레임에 겹치면 |=라 한 칸이다. 스틱에는 눌림 에지가 없으므로 첫 칸도 여기서 낸다.
        let nav = self.nav_edges(now.move_dir, get_frame_time());
        now.up |= nav.1 < 0;
        now.down |= nav.1 > 0;
        now.left |= nav.0 < 0;
        now.right |= nav.0 > 0;

        self.pending.merge_edges(&now);
        self.held = now;
    }

    /// 로직 틱마다(또는 메뉴에서는 프레임마다) 한 번. 눌림을 소비한다.
    pub fn take(&mut self) -> Actions {
        let mut a = self.pending;
        a.move_dir = self.held.move_dir;
        a.aim = self.held.aim;
        a.dodge_held = self.held.dodge_held;
        a.attack_held = self.held.attack_held;
        self.pending = Actions::default();
        a
    }

    fn nav_edges(&mut self, mv: Vec2, dt: f32) -> (i8, i8) {
        let d = nav_dir(self.nav.dir, mv);
        if d == (0, 0) {
            self.nav = NavRepeat::default();
            return (0, 0);
        }
        if d != self.nav.dir {
            self.nav.dir = d;
            self.nav.timer = 0.0;
            return d;
        }
        self.nav.timer += dt;
        if self.nav.timer > tuning::NAV_REPEAT_DELAY {
            self.nav.timer -= tuning::NAV_REPEAT_EVERY;
            return d;
        }
        (0, 0)
    }

    /// 터치. 게임 모드: 왼쪽 반에서 시작한 손가락은 스틱, 버튼 위에서 시작한 손가락은 눌림.
    /// 메뉴 모드: 새 손가락은 탭.
    fn read_touch(&mut self) -> Actions {
        let mut a = Actions::default();
        let touches = touches();
        self.touch.active = !touches.is_empty();
        if touches.is_empty()
            && self.touch.stick.is_none()
            && self.touch.known.is_empty()
            && self.touch.menu.is_none()
        {
            return a;
        }
        let layout = touch_layout();
        let half = screen_width() * 0.5;
        // macroquad 0.4.16: screen_width()·mouse_position()은 DPI로 나눈 논리 픽셀인데 touches()만 물리 픽셀이다.
        // 고해상도 폰(DPR 2~3)에서 버튼 판정이 빗나가는 원인. 같은 단위로 맞춘다.
        let dpi = screen_dpi_scale().max(0.01);
        let touches: Vec<Touch> = touches
            .into_iter()
            .map(|mut t| {
                t.position /= dpi;
                t
            })
            .collect();
        for t in &touches {
            let ended = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
            let is_new = !self.touch.known.contains(&t.id);
            // 한 프레임 안에 start와 end가 다 들어온 짧은 탭도 눌림이다(프레임이 느린 폰에서 흔함).
            if is_new {
                if !ended {
                    self.touch.known.push(t.id);
                }
                if self.menu_mode {
                    // 한 프레임 안에 짚고 뗀 짧은 탭은 바로, 아니면 뗄 때 정한다(끌었으면 탭이 아니다, 0199).
                    if ended {
                        a.tap = Some(t.position);
                    } else if self.touch.menu.is_none() {
                        self.touch.menu = Some(MenuFinger {
                            id: t.id,
                            start: t.position,
                            last: t.position,
                            dragging: false,
                        });
                    }
                } else if let Some(b) = layout
                    .buttons
                    .iter()
                    .find(|b| (t.position - b.center).length() <= b.radius * 1.3)
                {
                    match b.kind {
                        TouchKind::Attack => {
                            a.attack = true;
                            a.confirm = true;
                            if !ended {
                                self.touch.atk = Some(t.id);
                            }
                        }
                        TouchKind::Dodge => {
                            a.dodge = true;
                            if !ended {
                                self.touch.dig = Some(t.id);
                            }
                        }
                        TouchKind::Special => a.special = true,
                        TouchKind::Item => a.use_item = true,
                        TouchKind::ItemSwitch => a.item_next = true,
                        TouchKind::Menu => a.menu = true,
                    }
                } else if t.position.x < half && self.touch.stick.is_none() && !ended {
                    self.touch.stick = Some(TouchStick {
                        id: t.id,
                        origin: t.position,
                        pos: t.position,
                    });
                }
            }
            if let Some(s) = self.touch.stick.as_mut() {
                if s.id == t.id {
                    s.pos = t.position;
                }
            }
            if let Some(m) = self.touch.menu.as_mut() {
                if m.id == t.id {
                    // 화면 높이의 3%를 넘게 움직이면 끌기로 본다(손가락 떨림은 탭).
                    if !m.dragging && (t.position - m.start).length() > screen_height() * 0.03 {
                        m.dragging = true;
                        a.drag += m.start.y - t.position.y;
                        m.last = t.position;
                    } else if m.dragging {
                        a.drag += m.last.y - t.position.y;
                        m.last = t.position;
                    }
                    if ended {
                        if !m.dragging {
                            a.tap = Some(m.start);
                        }
                        self.touch.menu = None;
                    }
                }
            }
            if ended {
                self.touch.known.retain(|id| *id != t.id);
                if self.touch.dig == Some(t.id) {
                    self.touch.dig = None;
                }
                if self.touch.atk == Some(t.id) {
                    self.touch.atk = None;
                }
                if self.touch.stick.map(|s| s.id) == Some(t.id) {
                    self.touch.stick = None;
                }
            }
        }
        // 손가락이 사라졌는데 Ended를 못 받은 경우(브라우저 탭 전환 등) 정리.
        self.touch
            .known
            .retain(|id| touches.iter().any(|t| t.id == *id));
        if let Some(s) = self.touch.stick {
            if !touches.iter().any(|t| t.id == s.id) {
                self.touch.stick = None;
            }
        }
        if let Some(id) = self.touch.dig {
            if !touches.iter().any(|t| t.id == id) {
                self.touch.dig = None;
            }
        }
        if let Some(id) = self.touch.atk {
            if !touches.iter().any(|t| t.id == id) {
                self.touch.atk = None;
            }
        }
        if let Some(m) = self.touch.menu {
            if !touches.iter().any(|t| t.id == m.id) {
                self.touch.menu = None;
            }
        }
        a.dodge_held = self.touch.dig.is_some();
        a.attack_held = self.touch.atk.is_some();
        if let Some(s) = self.touch.stick {
            let v = (s.pos - s.origin) / layout.stick_radius;
            let len = v.length();
            if len > 0.15 {
                a.move_dir = v / len * ((len - 0.15) / 0.85).min(1.0);
            }
        }
        a
    }

    #[cfg(not(target_os = "android"))]
    fn read_gamepads(&mut self) -> (Actions, usize) {
        self.pads.poll();
        read_gamepads(&self.pads)
    }

    /// 다시 붙이기용: 이번 프레임에 눌린 키(표 안) 또는 패드 버튼. 이동·Enter·Esc는 돌려주지 않는다.
    pub fn capture(&self) -> Option<Captured> {
        if let Some(k) = get_last_key_pressed() {
            if is_key_pressed(k) && KEYS.iter().any(|(kk, _)| *kk == k) {
                return Some(Captured::Key(k));
            }
        }
        #[cfg(not(target_os = "android"))]
        for pad in self.pads.all() {
            for p in PADS {
                if pad.is_just_pressed(p.button()) {
                    return Some(Captured::Pad(p));
                }
            }
        }
        #[cfg(target_os = "android")]
        for p in PADS {
            if is_key_pressed(p.key()) {
                return Some(Captured::Pad(p));
            }
        }
        None
    }

    /// Android: 패드 버튼은 F13~F25 키(Pad::key), 스틱·HAT은 축(android_axes), 키코드 D패드는 화살표 키로 들어온다. 화살표도 패드로 센다
    /// (Android에 키보드를 붙이는 일은 드물고, 키보드로 세면 D패드를 누를 때마다 안내 글이 키보드 글자로 바뀐다).
    #[cfg(target_os = "android")]
    fn read_gamepads(&mut self) -> (Actions, usize) {
        let mut a = Actions::default();
        let mut m = Vec2::ZERO;
        if is_key_down(KeyCode::Left) {
            m.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            m.x += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            m.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            m.y += 1.0;
        }
        let (mv, aim) = android_axes(macroquad::miniquad::native::joystick_axes(), m);
        a.move_dir = mv;
        a.aim = aim;
        let b = bindings();
        let j = |p: Pad| is_key_pressed(p.key());
        a.attack = j(b.pad(Act::Attack));
        a.dodge = j(b.pad(Act::Dodge));
        a.dodge_held = is_key_down(b.pad(Act::Dodge).key());
        a.attack_held = is_key_down(b.pad(Act::Attack).key());
        a.special = j(b.pad(Act::Special));
        a.use_item = j(b.pad(Act::UseItem));
        a.item_prev = j(b.pad(Act::ItemPrev));
        a.item_next = j(b.pad(Act::ItemNext));
        a.map = j(b.pad(Act::Map));
        // 메뉴: A 확인, B(또는 기기 뒤로) 뒤로, Start 메뉴, LB·RB 탭(고정) — 데스크톱·웹 패드와 같다.
        a.confirm = j(Pad::South);
        a.cancel = j(Pad::East) || is_key_pressed(KeyCode::Back);
        a.menu = j(Pad::Start);
        a.tab_prev = j(Pad::L1);
        a.tab_next = j(Pad::R1);
        a.up = is_key_pressed(KeyCode::Up);
        a.down = is_key_pressed(KeyCode::Down);
        a.left = is_key_pressed(KeyCode::Left);
        a.right = is_key_pressed(KeyCode::Right);
        if mv != Vec2::ZERO || aim != Vec2::ZERO || PADS.iter().any(|p| is_key_down(p.key())) {
            self.pad_seen = true;
        }
        (a, usize::from(self.pad_seen))
    }
}

/// Android 패드 축(패치한 miniquad의 joystick_axes: 왼쪽 X·Y, 오른쪽 Z·RZ, HAT X·Y, 아래가 +y) + 화살표 키(키코드로 오는 D패드)
/// → (이동, 조준). 왼쪽 스틱이 데드존을 넘으면 그 세기 그대로, 아니면 D패드(HAT·화살표)를 8방향으로.
/// 조준은 오른쪽 스틱 방향(길이 1). 데스크톱·웹 패드(read_gamepads)와 같은 데드존을 쓴다(0143).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn android_axes(axes: [f32; 6], keys: Vec2) -> (Vec2, Vec2) {
    let stick = apply_deadzone(vec2(axes[0], axes[1]));
    let mv = if stick != Vec2::ZERO {
        stick
    } else {
        let hat = vec2(axes[4].round(), axes[5].round());
        (keys + hat)
            .clamp(Vec2::splat(-1.0), Vec2::ONE)
            .normalize_or_zero()
    };
    (
        mv,
        apply_deadzone(vec2(axes[2], axes[3])).normalize_or_zero(),
    )
}

/// 이동 벡터 → 메뉴 방향 한 축. 지금 잡힌 방향은 NAV_RELEASE까지 유지하고,
/// 새 방향은 큰 축이 NAV_PRESS를 넘을 때만 잡는다(대각선은 큰 쪽으로).
fn nav_dir(held: (i8, i8), mv: Vec2) -> (i8, i8) {
    let along = held.0 as f32 * mv.x + held.1 as f32 * mv.y;
    if held != (0, 0) && along > tuning::NAV_RELEASE {
        return held;
    }
    if mv.x.abs() >= mv.y.abs() {
        if mv.x.abs() > tuning::NAV_PRESS {
            return (mv.x.signum() as i8, 0);
        }
    } else if mv.y.abs() > tuning::NAV_PRESS {
        return (0, mv.y.signum() as i8);
    }
    (0, 0)
}

fn read_keyboard() -> Actions {
    let b = bindings();
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
    let k = |a: Act| is_key_pressed(b.key(a));
    Actions {
        move_dir: m.normalize_or_zero(),
        aim: Vec2::ZERO,
        attack: k(Act::Attack),
        dodge: k(Act::Dodge),
        dodge_held: is_key_down(b.key(Act::Dodge)),
        attack_held: is_key_down(b.key(Act::Attack)),
        special: k(Act::Special),
        use_item: k(Act::UseItem),
        item_prev: k(Act::ItemPrev),
        item_next: k(Act::ItemNext),
        map: k(Act::Map),
        // 메뉴: Enter·Space 또는 공격 키로 확인, Esc 또는 필살기 키로 뒤로. 탭은 Q·E(아이템 넘기기 키) 고정.
        confirm: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || k(Act::Attack),
        cancel: is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace),
        menu: is_key_pressed(KeyCode::Escape),
        restart: false,
        tab_prev: is_key_pressed(KeyCode::Q),
        tab_next: is_key_pressed(KeyCode::E),
        toggle_debug: is_key_pressed(KeyCode::F1),
        toggle_hitboxes: is_key_pressed(KeyCode::F2),
        // 눌렀다 뗀 것이 한 프레임 안에 끝나도 메뉴 탐색은 한 칸 움직여야 한다(is_key_down만 보면 놓친다).
        up: is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
        down: is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
        left: is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
        right: is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
        ..Default::default()
    }
}

/// 이동을 뺀 눌림이 하나라도 있나(안내 글 장치 판정).
/// 장치를 바꾸는 누름: 게임 동작·메뉴 동사·메뉴 탐색 한 칸(D패드·화살표 톡). 스틱·이동 축은 따로 본다.
fn any_edge(a: &Actions) -> bool {
    a.attack
        || a.dodge
        || a.special
        || a.use_item
        || a.item_prev
        || a.item_next
        || a.map
        || a.confirm
        || a.cancel
        || a.menu
        || a.up
        || a.down
        || a.left
        || a.right
        || a.tab_prev
        || a.tab_next
}

/// 마우스는 보조: 왼쪽 클릭을 탭으로, 휠을 칸으로. 터치가 흉내 내는 가짜 마우스는 앱이 꺼 둔다
/// (`simulate_mouse_with_touch(false)`, 0199) — 켜 두면 손가락을 짚는 순간 클릭이 되어 끌기가 탭이 된다.
fn read_mouse() -> Actions {
    let mut a = Actions::default();
    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        a.tap = Some(vec2(x, y));
    }
    let (_, wy) = mouse_wheel();
    if wy != 0.0 {
        a.wheel = if wy < 0.0 { 1 } else { -1 };
    }
    a
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
        // 게임 동작은 배치표대로. 메뉴: A 확인, B 뒤로, Start 메뉴, LB·RB 탭(고정).
        let b = bindings();
        let j = |x: Act| pad.is_just_pressed(b.pad(x).button());
        a.attack |= j(Act::Attack);
        a.dodge |= j(Act::Dodge);
        a.dodge_held |= pad.is_currently_pressed(b.pad(Act::Dodge).button());
        a.attack_held |= pad.is_currently_pressed(b.pad(Act::Attack).button());
        a.special |= j(Act::Special);
        a.use_item |= j(Act::UseItem);
        a.item_prev |= j(Act::ItemPrev);
        a.item_next |= j(Act::ItemNext);
        a.map |= j(Act::Map);
        a.confirm |= pad.is_just_pressed(Button::ActionDown);
        a.cancel |= pad.is_just_pressed(Button::ActionRight);
        a.menu |= pad.is_just_pressed(Button::RightCenterCluster);
        a.tab_prev |= pad.is_just_pressed(Button::FrontLeftUpper);
        a.tab_next |= pad.is_just_pressed(Button::FrontRightUpper);
        a.up |= pad.is_just_pressed(Button::DPadUp);
        a.down |= pad.is_just_pressed(Button::DPadDown);
        a.left |= pad.is_just_pressed(Button::DPadLeft);
        a.right |= pad.is_just_pressed(Button::DPadRight);
    }
    (a, count)
}

/// 반지름 데드존. 데드존 바깥을 0..1로 다시 펴서 작은 기울기도 쓸 수 있게 한다.
fn apply_deadzone(v: Vec2) -> Vec2 {
    let len = v.length();
    if len < tuning::STICK_DEADZONE {
        return Vec2::ZERO;
    }
    let scaled = ((len - tuning::STICK_DEADZONE) / (1.0 - tuning::STICK_DEADZONE)).min(1.0);
    v / len * scaled
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Android 패드 축(0143): 왼쪽 스틱은 세기 그대로, 데드존 안이면 D패드(HAT·화살표), 오른쪽 스틱은 조준 방향.
    #[test]
    fn android_axes_map_sticks_and_hat() {
        let none = [0.0; 6];
        assert_eq!(android_axes(none, Vec2::ZERO), (Vec2::ZERO, Vec2::ZERO));
        // 데드존 안 떨림은 움직이지 않는다.
        assert_eq!(
            android_axes([0.1, -0.1, 0.15, 0.0, 0.0, 0.0], Vec2::ZERO),
            (Vec2::ZERO, Vec2::ZERO)
        );
        // 반쯤 기울인 왼쪽 스틱 = 반보다 작은 세기(데드존을 빼고 편 값), 방향 그대로(아래가 +y).
        let (mv, _) = android_axes([0.0, 0.6, 0.0, 0.0, 0.0, 0.0], Vec2::ZERO);
        assert!((mv - vec2(0.0, 0.5)).length() < 1e-5, "{mv:?}");
        // 스틱이 서 있으면 HAT, HAT도 없으면 화살표 키. 대각선은 길이 1.
        let (mv, _) = android_axes([0.0, 0.0, 0.0, 0.0, -1.0, 1.0], Vec2::ZERO);
        assert!((mv - vec2(-1.0, 1.0).normalize()).length() < 1e-5);
        let (mv, _) = android_axes(none, vec2(1.0, 0.0));
        assert_eq!(mv, vec2(1.0, 0.0));
        // HAT와 같은 쪽 화살표가 함께 와도 한 칸 세기.
        let (mv, _) = android_axes([0.0, 0.0, 0.0, 0.0, 1.0, 0.0], vec2(1.0, 0.0));
        assert_eq!(mv, vec2(1.0, 0.0));
        // 스틱이 기울면 D패드보다 스틱.
        let (mv, _) = android_axes([0.0, -1.0, 0.0, 0.0, 1.0, 0.0], Vec2::ZERO);
        assert_eq!(mv, vec2(0.0, -1.0));
        // 오른쪽 스틱(Z·RZ)은 조준 방향만(살짝 기울여도 길이 1).
        let (_, aim) = android_axes([0.0, 0.0, 0.4, -0.4, 0.0, 0.0], Vec2::ZERO);
        assert!((aim - vec2(1.0, -1.0).normalize()).length() < 1e-5);
    }

    /// Android 키 반복(0171): 패치한 miniquad가 반복 이벤트의 repeat 플래그를 Java → JNI → key_down_event까지 넘기는지.
    /// 빠지면 패드 버튼을 누르고 있는 동안 is_key_pressed가 계속 참이라 땅파기·강타를 모을 수 없다.
    #[test]
    fn android_key_repeat_reaches_macroquad() {
        let java = include_str!("../third_party/miniquad/java/MainActivity.java");
        assert!(java.contains("QuadNative.surfaceOnKeyDown(keyCode, event.getRepeatCount() > 0)"));
        let native = include_str!("../third_party/miniquad/java/QuadNative.java");
        assert!(native.contains("surfaceOnKeyDown(int keycode, boolean repeat)"));
        let rs = include_str!("../third_party/miniquad/src/native/android.rs");
        assert!(rs.contains("repeat: ndk_sys::jboolean"));
        assert!(rs.contains(".key_down_event(keycode, self.keymods, repeat)"));
    }

    /// Android 패드 버튼: 패치한 miniquad 표(KEYCODE_BUTTON_* → F13~F25)와 Pad::key가 같은 짝이고, 키가 겹치지 않는다.
    /// 한쪽만 고치면 Android에서 버튼이 엉뚱하게 눌린다(0075).
    #[test]
    fn android_pad_keys_match_patched_miniquad() {
        let table = include_str!("../third_party/miniquad/src/native/android/keycodes.rs");
        let name = |p: Pad| match p {
            Pad::South => "BUTTON_A",
            Pad::East => "BUTTON_B",
            Pad::West => "BUTTON_X",
            Pad::North => "BUTTON_Y",
            Pad::L1 => "BUTTON_L1",
            Pad::R1 => "BUTTON_R1",
            Pad::L2 => "BUTTON_L2",
            Pad::R2 => "BUTTON_R2",
            Pad::L3 => "BUTTON_THUMBL",
            Pad::R3 => "BUTTON_THUMBR",
            Pad::Start => "BUTTON_START",
            Pad::Select => "BUTTON_SELECT",
        };
        let mut seen = Vec::new();
        for p in PADS {
            let k = p.key();
            assert!(!seen.contains(&k), "{p:?} 키 겹침");
            seen.push(k);
            let line = table
                .lines()
                .find(|l| l.trim_end().ends_with(&format!("// KEYCODE_{}", name(p))))
                .unwrap_or_else(|| panic!("패치 표에 KEYCODE_{} 없음", name(p)));
            assert!(
                line.contains(&format!("KeyCode::{k:?},")),
                "{p:?}: 표는 `{}`, 게임은 {k:?}",
                line.trim()
            );
        }
    }

    /// 겹치는 키를 붙이면 서로 바뀐다. Start·이동 키는 붙지 않는다.
    #[test]
    fn rebinding_swaps_and_rejects_reserved() {
        let mut b = Bindings::DEFAULT;
        assert_eq!(b.key(Act::Attack), KeyCode::J);
        assert!(b.bind_key(Act::Attack, KeyCode::K));
        assert_eq!(b.key(Act::Attack), KeyCode::K);
        assert_eq!(b.key(Act::Dodge), KeyCode::J, "서로 바뀜");
        assert!(!b.bind_key(Act::Attack, KeyCode::W), "이동 키는 안 된다");
        assert!(!b.bind_pad(Act::Attack, Pad::Start));
        assert!(b.bind_pad(Act::UseItem, Pad::West));
        assert_eq!(b.pad(Act::UseItem), Pad::West);
        assert_eq!(b.pad(Act::Attack), Pad::North);
        assert_eq!(b.sanitized(), b);
        let broken = Bindings {
            keys: [0, 0, 1, 2, 3, 4, 5],
            ..b
        };
        assert_eq!(broken.sanitized(), Bindings::DEFAULT);
    }

    #[test]
    fn stick_nav_needs_press_and_holds_until_release() {
        assert_eq!(nav_dir((0, 0), vec2(0.0, 0.4)), (0, 0));
        assert_eq!(nav_dir((0, 0), vec2(0.0, 0.7)), (0, 1));
        // 경계에서 떨려도 유지(히스테리시스).
        assert_eq!(nav_dir((0, 1), vec2(0.1, 0.4)), (0, 1));
        assert_eq!(nav_dir((0, 1), vec2(0.0, 0.2)), (0, 0));
        // 대각선은 큰 축. 잡힌 축이 약해지면 다른 축으로.
        assert_eq!(nav_dir((0, 0), vec2(0.8, -0.6)), (1, 0));
        assert_eq!(nav_dir((1, 0), vec2(0.2, 0.9)), (0, 1));
    }

    /// 터치 버튼: 공격이 가장 크고, 버튼끼리(그린 동그라미) 겹치지 않고 화면 안에 있다. 가로·세로 폰, 태블릿.
    /// 일마다 버튼은 하나(앱이 `button_mut(kind)`로 대기를 넣는다 — 칸 번호로 넣다 아이템·필살기가 어긋났다, 2026-10-29).
    #[test]
    fn every_touch_kind_has_exactly_one_button() {
        let mut l = layout_for(844.0, 390.0);
        for k in [
            TouchKind::Attack,
            TouchKind::Dodge,
            TouchKind::Menu,
            TouchKind::Special,
            TouchKind::Item,
            TouchKind::ItemSwitch,
        ] {
            assert_eq!(l.buttons.iter().filter(|b| b.kind == k).count(), 1, "{k:?}");
            l.button_mut(k).ready = 0.5;
        }
        assert!(l.buttons.iter().all(|b| b.ready == 0.5));
    }

    #[test]
    fn touch_buttons_attack_biggest_no_overlap() {
        for (w, h) in [
            (844.0, 390.0),
            (390.0, 844.0),
            (1180.0, 820.0),
            (667.0, 375.0),
        ] {
            let l = layout_for(w, h);
            let atk = l
                .buttons
                .iter()
                .find(|b| b.kind == TouchKind::Attack)
                .unwrap();
            assert!(l.buttons.iter().all(|b| b.radius <= atk.radius), "{w}x{h}");
            assert!(
                atk.radius * 2.0 >= 0.25 * w.min(h),
                "공격 지름은 짧은 변의 25% 이상"
            );
            for (i, a) in l.buttons.iter().enumerate() {
                assert!(
                    a.center.x - a.radius >= 0.0 && a.center.x + a.radius <= w,
                    "{w}x{h} {:?}",
                    a.kind
                );
                assert!(
                    a.center.y - a.radius >= 0.0 && a.center.y + a.radius <= h,
                    "{w}x{h} {:?}",
                    a.kind
                );
                for b in &l.buttons[i + 1..] {
                    let d = (a.center - b.center).length();
                    assert!(
                        d >= a.radius + b.radius,
                        "{w}x{h}: {:?} {:?} 겹침",
                        a.kind,
                        b.kind
                    );
                }
            }
        }
    }

    /// 안내 글: 장치마다 동사·버튼 이름이 바뀐다. 터치는 터치 버튼 글자·낱말, 키보드는 키, 패드는 패드 버튼.
    #[test]
    fn hints_follow_the_device() {
        let all = [
            Verb::Confirm,
            Verb::Back,
            Verb::Menu,
            Verb::Walk,
            Verb::Pick,
            Verb::Change,
            Verb::Tabs,
            Verb::Skip,
            Verb::Restart,
        ];
        set_device(Device::Keys);
        assert_eq!(verb(Verb::Confirm), "Enter");
        assert_eq!(verb(Verb::Walk), "WASD");
        set_device(Device::Pad);
        assert_eq!(verb(Verb::Confirm), Pad::South.label());
        assert_eq!(verb(Verb::Tabs), "LB/RB");
        set_device(Device::Touch);
        assert_eq!(prompt(Act::Attack), crate::i18n::s().act_attack);
        assert_eq!(prompt(Act::Dodge), crate::i18n::s().act_dodge);
        assert_eq!(verb(Verb::Confirm), crate::i18n::s().v_touch);
        for d in [Device::Keys, Device::Pad, Device::Touch] {
            set_device(d);
            assert!(all.iter().all(|v| !verb(*v).is_empty()), "{d:?}");
            // 키·패드는 다 누르는 것, 터치는 화면 버튼(메뉴·공격)만(손짓 "탭"은 글 그대로).
            let caps: Vec<_> = all.iter().filter(|v| verb_is_button(**v)).collect();
            match d {
                Device::Touch => assert_eq!(caps, [&Verb::Menu, &Verb::Restart]),
                _ => assert_eq!(caps.len(), all.len()),
            }
        }
        set_device(Device::Keys);
    }
}
