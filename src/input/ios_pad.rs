//! iOS 게임패드(0288): Apple GameController 프레임워크의 `GCController`를 렌더 프레임마다 읽어 `PadFrame`으로 넘긴다.
//!
//! Android(0075·0143)는 OS가 이벤트를 밀어 주므로 패치한 miniquad가 키·축으로 넘긴다. iOS는 프레임워크가 지금 값을
//! 들고 있어(폴링) miniquad를 고치지 않고 여기서 직접 묻는다. `Input::poll`(miniquad의 그리기 콜백 = 메인 스레드)에서만 부른다.
//! (KB: reference/macroquad/gamepad-and-android-input.md)
//!
//! - 연결: 처음 부를 때 연결·끊김 알림을 구독하고(Apple 권장 — iOS 15부터 프레임워크는 `controllers`를 처음 읽거나
//!   알림을 구독할 때 패드를 찾기 시작한다), 프레임마다 `+[GCController controllers]`를 다시 읽는다.
//!   알림 처리 메서드는 비어 있다 — 붙고 떨어지는 것은 폴링이 그대로 본다.
//! - 버튼은 자리로(`extendedGamepad`: buttonA = 아래 … — 소니·닌텐도 패드도 자리가 같다), Start = buttonMenu,
//!   Select = buttonOptions, 스틱 누르기 = thumbstick 버튼(앞의 둘은 iOS 13+, 스틱 누르기는 12.1+, 우리 최소판은 14).
//!   홈 버튼은 시스템 몫(게임 센터·스크린숏)이라 읽지 않고 시스템 제스처도 끄지 않는다(Apple: "드문 경우에만").
//! - 옛 패드에 없는 단추는 nil이다 — 메시지를 보내지 않고 안 눌린 것으로 센다.
//! - GameController 클래스는 `Class::get`으로 찾는다(`class!`는 없으면 panic) — 없으면 패드 없음.

use macroquad::prelude::{vec2, Vec2};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, NO};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

use super::{Pad, PadFrame};

type Id = *mut Object;

#[link(name = "GameController", kind = "framework")]
extern "C" {
    static GCControllerDidConnectNotification: Id;
    static GCControllerDidDisconnectNotification: Id;
}

static OBSERVE: Once = Once::new();

/// 연결·끊김 알림을 한 번 구독한다(빈 처리기). 관찰자는 앱이 끝날 때까지 둔다.
fn observe() {
    OBSERVE.call_once(|| {
        extern "C" fn changed(_: &Object, _: Sel, _: Id) {}
        let Some(mut decl) = ClassDecl::new("NailPadObserver", class!(NSObject)) else {
            return;
        };
        // SAFETY: NSObject 하위 클래스에 `padChanged:`(인자 하나, 반환 없음)를 붙여 등록하고, 알림 센터에 그 셀렉터로 건다.
        // 알림 이름은 GameController 프레임워크가 내보내는 NSString 상수다. 메인 스레드에서 한 번만.
        unsafe {
            decl.add_method(
                sel!(padChanged:),
                changed as extern "C" fn(&Object, Sel, Id),
            );
            let cls = decl.register();
            let observer: Id = msg_send![cls, new];
            let center: Id = msg_send![class!(NSNotificationCenter), defaultCenter];
            let nil: Id = std::ptr::null_mut();
            for name in [
                GCControllerDidConnectNotification,
                GCControllerDidDisconnectNotification,
            ] {
                let _: () = msg_send![center, addObserver: observer
                                              selector: sel!(padChanged:)
                                              name: name
                                              object: nil];
            }
        }
    });
}

/// `GCControllerButtonInput`이 눌렸나. nil이면 안 눌림.
///
/// SAFETY: `button`은 nil이거나 GCControllerButtonInput.
unsafe fn pressed(button: Id) -> bool {
    if button.is_null() {
        return false;
    }
    let on: BOOL = msg_send![button, isPressed];
    on != NO
}

/// `GCControllerDirectionPad`의 (x, y), 위가 +y. nil이면 0.
///
/// SAFETY: `pad`는 nil이거나 GCControllerDirectionPad.
unsafe fn stick(pad: Id) -> Vec2 {
    if pad.is_null() {
        return Vec2::ZERO;
    }
    let x: Id = msg_send![pad, xAxis];
    let y: Id = msg_send![pad, yAxis];
    if x.is_null() || y.is_null() {
        return Vec2::ZERO;
    }
    let x: f32 = msg_send![x, value];
    let y: f32 = msg_send![y, value];
    vec2(x, y)
}

/// 패드 버튼 → `GCExtendedGamepad`의 단추(nil일 수 있다).
///
/// SAFETY: `g`는 nil이 아닌 GCExtendedGamepad(`read`가 거른다). 이 속성들은 iOS 14에 다 있다(최소판 14).
unsafe fn button(g: Id, p: Pad) -> Id {
    unsafe {
        match p {
            Pad::South => msg_send![g, buttonA],
            Pad::East => msg_send![g, buttonB],
            Pad::West => msg_send![g, buttonX],
            Pad::North => msg_send![g, buttonY],
            Pad::L1 => msg_send![g, leftShoulder],
            Pad::R1 => msg_send![g, rightShoulder],
            Pad::L2 => msg_send![g, leftTrigger],
            Pad::R2 => msg_send![g, rightTrigger],
            Pad::Select => msg_send![g, buttonOptions],
            Pad::Start => msg_send![g, buttonMenu],
            Pad::L3 => msg_send![g, leftThumbstickButton],
            Pad::R3 => msg_send![g, rightThumbstickButton],
        }
    }
}

/// 연결된 패드(확장 프로필이 있는 것)를 모두 합친 지금 상태와 그 수.
pub(super) fn read() -> (PadFrame, usize) {
    observe();
    let mut all = PadFrame::default();
    let mut count = 0;
    objc::rc::autoreleasepool(|| {
        // SAFETY: 클래스 메서드 `controllers`는 NSArray<GCController*>를 돌려준다. 원소의 `extendedGamepad`는 nil일 수 있다
        // (확장 프로필이 없는 패드·리모컨) — 거른다. 메인 스레드에서 부른다.
        unsafe {
            // `class!`는 클래스를 못 찾으면 panic한다 — 찾지 못하면 패드 없음으로(KB ios-build "두 길 모두 꼭").
            let Some(gc) = Class::get("GCController") else {
                return;
            };
            let list: Id = msg_send![gc, controllers];
            if list.is_null() {
                return;
            }
            let n: usize = msg_send![list, count];
            for i in 0..n {
                let c: Id = msg_send![list, objectAtIndex: i];
                if c.is_null() {
                    continue;
                }
                let g: Id = msg_send![c, extendedGamepad];
                if g.is_null() {
                    continue;
                }
                count += 1;
                let mut f = PadFrame::default();
                for p in super::PADS {
                    f.buttons[p as usize] = pressed(button(g, p));
                }
                let dpad: Id = msg_send![g, dpad];
                if !dpad.is_null() {
                    let up: Id = msg_send![dpad, up];
                    let down: Id = msg_send![dpad, down];
                    let left: Id = msg_send![dpad, left];
                    let right: Id = msg_send![dpad, right];
                    f.dpad = [pressed(up), pressed(down), pressed(left), pressed(right)];
                }
                let ls: Id = msg_send![g, leftThumbstick];
                let rs: Id = msg_send![g, rightThumbstick];
                f.left = stick(ls);
                f.right = stick(rs);
                all.merge(&f);
            }
        }
    });
    (all, count)
}
