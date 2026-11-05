//! nail-under-the-roots 패치(0282): 글자·숫자·기호 키를 **물리 자리**로.
//! PC 스캔코드(set 1)와 Linux evdev 키 코드는 이 블록에서 같은 번호다(KEY_Q 16 … KEY_SLASH 53, KEY_SPACE 57).
//! X11(`xkey.keycode - 8`)·Wayland(`key`)가 원본은 배열을 거친 keysym으로 KeyCode를 정해, AZERTY면 W 자리가 Z로 들어왔다.
//! 웹(`e.code`)·Windows(스캔코드)·macOS(가상 키)는 원래 자리라 이것으로 같아진다. 배열과 상관없는 키(화살표·F키·Enter …)는
//! None — 원래 keysym 번역 그대로.
use crate::event::KeyCode;

pub fn scancode_position(sc: u32) -> Option<KeyCode> {
    use KeyCode::*;
    Some(match sc {
        2 => Key1,
        3 => Key2,
        4 => Key3,
        5 => Key4,
        6 => Key5,
        7 => Key6,
        8 => Key7,
        9 => Key8,
        10 => Key9,
        11 => Key0,
        12 => Minus,
        13 => Equal,
        16 => Q,
        17 => W,
        18 => E,
        19 => R,
        20 => T,
        21 => Y,
        22 => U,
        23 => I,
        24 => O,
        25 => P,
        26 => LeftBracket,
        27 => RightBracket,
        30 => A,
        31 => S,
        32 => D,
        33 => F,
        34 => G,
        35 => H,
        36 => J,
        37 => K,
        38 => L,
        39 => Semicolon,
        40 => Apostrophe,
        41 => GraveAccent,
        43 => Backslash,
        44 => Z,
        45 => X,
        46 => C,
        47 => V,
        48 => B,
        49 => N,
        50 => M,
        51 => Comma,
        52 => Period,
        53 => Slash,
        57 => Space,
        _ => return None,
    })
}
