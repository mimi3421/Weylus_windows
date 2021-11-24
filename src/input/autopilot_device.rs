use autopilot::geometry::Size;
use autopilot::mouse;
use autopilot::mouse::ScrollDirection;
use autopilot::screen::size as screen_size;

use winapi::shared::windef::{HWND, POINT};
use winapi::um::winuser::*;

use tracing::warn;

use crate::input::device::{InputDevice, InputDeviceType};
use crate::protocol::{Button, KeyboardEvent, KeyboardEventType, PointerEvent, WheelEvent};

use crate::capturable::Capturable;

pub struct AutoPilotDevice {
    capturable: Box<dyn Capturable>,
    syntheticPointerDeviceHandle: *mut HSYNTHETICPOINTERDEVICE__,
}

impl AutoPilotDevice {
    pub fn new(capturable: Box<dyn Capturable>) -> Self {
        self.capturable = capturable;
        self.syntheticPointerDeviceHandle = unsafe { CreateSyntheticPointerDevice(PT_PEN, 1, 1) }
    }
}

impl InputDevice for AutoPilotDevice {
    fn send_wheel_event(&mut self, event: &WheelEvent) {
        match event.dy {
            1..=i32::MAX => mouse::scroll(ScrollDirection::Up, 1),
            i32::MIN..=-1 => mouse::scroll(ScrollDirection::Down, 1),
            0 => {}
        }
    }

    fn send_pointer_event(&mut self, event: &PointerEvent) {
        if !event.is_primary {
            return;
        }
        if let Err(err) = self.capturable.before_input() {
            warn!("Failed to activate window, sending no input ({})", err);
            return;
        }
        let geometry = self.capturable.geometry_relative();
        if let Err(err) = geometry {
            warn!("Failed to get window geometry, sending no input ({})", err);
            return;
        }
        let (x_rel, y_rel, width_rel, height_rel) = geometry.unwrap();
        #[cfg(not(target_os = "macos"))]
        let Size { width, height } = screen_size();
        #[cfg(target_os = "macos")]
        let (_, _, width, height) = match crate::capturable::core_graphics::screen_coordsys() {
            Ok(bounds) => bounds,
            Err(err) => {
                warn!("Could determine global coordinate system: {}", err);
                return;
            }
        };
        let x = (event.x * width_rel + x_rel) * width;
        let y = (event.y * height_rel + y_rel) * height;
        unsafe {
            let mut pointerTypeInfo = POINTER_TYPE_INFO {
                type_: PT_PEN,
                u: std::mem::zeroed(),
            };
            *pointerTypeInfo.u.pen_info_mut() = POINTER_PEN_INFO {
                pointerInfo: POINTER_INFO {
                    pointerType: PT_PEN,
                    pointerId: event.pointer_id,
                    frameId: 0,
                    pointerFlags: POINTER_FLAG_INRANGE
                        | POINTER_FLAG_INCONTACT
                        //| POINTER_FLAG_FIRSTBUTTON
                        | POINTER_FLAG_PRIMARY,
                    sourceDevice: 0 as *mut winapi::ctypes::c_void,
                    hwndTarget: 0 as HWND,
                    ptPixelLocation: POINT { x: x, y: y },
                    ptHimetricLocation: POINT { x: 0, y: 0 },
                    ptPixelLocationRaw: POINT { x: x, y: y },
                    ptHimetricLocationRaw: POINT { x: 0, y: 0 },
                    dwTime: 0,
                    historyCount: 1,
                    InputData: 0,
                    dwKeyStates: 0,
                    PerformanceCount: 0,
                    ButtonChangeType: POINTER_CHANGE_NONE,
                },
                penFlags: PEN_FLAG_NONE,
                penMask: PEN_MASK_PRESSURE | PEN_MASK_ROTATION | PEN_MASK_TILT_X | PEN_MASK_TILT_Y,
                pressure: event.pressure,
                rotation: 0,
                tiltX: 0,
                tiltY: 0,
            };
        }
        // if let Err(err) = mouse::move_to(autopilot::geometry::Point::new(
        //     (event.x * width_rel + x_rel) * width,
        //     (event.y * height_rel + y_rel) * height,
        // )) {
        //     warn!("Could not move mouse: {}", err);
        // }
        // match event.button {
        //     Button::PRIMARY => {
        //         mouse::toggle(mouse::Button::Left, event.buttons.contains(event.button))
        //     }
        //     Button::AUXILARY => {
        //         mouse::toggle(mouse::Button::Middle, event.buttons.contains(event.button))
        //     }
        //     Button::SECONDARY => {
        //         mouse::toggle(mouse::Button::Right, event.buttons.contains(event.button))
        //     }
        //     _ => (),
        // }
    }

    fn send_keyboard_event(&mut self, event: &KeyboardEvent) {
        use autopilot::key::{Character, Code, KeyCode};

        let state = match event.event_type {
            KeyboardEventType::UP => false,
            KeyboardEventType::DOWN => true,
            // autopilot doesn't handle this, so just do nothing
            KeyboardEventType::REPEAT => return,
        };

        fn map_key(code: &str) -> Option<KeyCode> {
            match code {
                "Escape" => Some(KeyCode::Escape),
                "Enter" => Some(KeyCode::Return),
                "Backspace" => Some(KeyCode::Backspace),
                "Tab" => Some(KeyCode::Tab),
                "Space" => Some(KeyCode::Space),
                "CapsLock" => Some(KeyCode::CapsLock),
                "F1" => Some(KeyCode::F1),
                "F2" => Some(KeyCode::F2),
                "F3" => Some(KeyCode::F3),
                "F4" => Some(KeyCode::F4),
                "F5" => Some(KeyCode::F5),
                "F6" => Some(KeyCode::F6),
                "F7" => Some(KeyCode::F7),
                "F8" => Some(KeyCode::F8),
                "F9" => Some(KeyCode::F9),
                "F10" => Some(KeyCode::F10),
                "F11" => Some(KeyCode::F11),
                "F12" => Some(KeyCode::F12),
                "F13" => Some(KeyCode::F13),
                "F14" => Some(KeyCode::F14),
                "F15" => Some(KeyCode::F15),
                "F16" => Some(KeyCode::F16),
                "F17" => Some(KeyCode::F17),
                "F18" => Some(KeyCode::F18),
                "F19" => Some(KeyCode::F19),
                "F20" => Some(KeyCode::F20),
                "F21" => Some(KeyCode::F21),
                "F22" => Some(KeyCode::F22),
                "F23" => Some(KeyCode::F23),
                "F24" => Some(KeyCode::F24),
                "Home" => Some(KeyCode::Home),
                "ArrowUp" => Some(KeyCode::UpArrow),
                "PageUp" => Some(KeyCode::PageUp),
                "ArrowLeft" => Some(KeyCode::LeftArrow),
                "ArrowRight" => Some(KeyCode::RightArrow),
                "End" => Some(KeyCode::End),
                "ArrowDown" => Some(KeyCode::DownArrow),
                "PageDown" => Some(KeyCode::PageDown),
                "Delete" => Some(KeyCode::Delete),
                "ControlLeft" | "ControlRight" => Some(KeyCode::Control),
                "AltLeft" | "AltRight" => Some(KeyCode::Alt),
                "MetaLeft" | "MetaRight" => Some(KeyCode::Meta),
                "ShiftLeft" | "ShiftRight" => Some(KeyCode::Shift),
                _ => None,
            }
        }
        let key = map_key(&event.code);
        let mut flags = Vec::new();
        if event.ctrl {
            flags.push(autopilot::key::Flag::Control);
        }
        if event.alt {
            flags.push(autopilot::key::Flag::Alt);
        }
        if event.meta {
            flags.push(autopilot::key::Flag::Meta);
        }
        if event.shift {
            flags.push(autopilot::key::Flag::Shift);
        }
        match key {
            Some(key) => autopilot::key::toggle(&Code(key), state, &flags, 0),
            None => {
                for c in event.key.chars() {
                    autopilot::key::toggle(&Character(c), state, &flags, 0);
                }
            }
        }
    }

    fn set_capturable(&mut self, capturable: Box<dyn Capturable>) {
        self.capturable = capturable;
    }

    fn device_type(&self) -> InputDeviceType {
        InputDeviceType::AutoPilotDevice
    }
}
