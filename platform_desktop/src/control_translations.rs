
use defs::control::{KeyCode, InputState};

use winit::event::{VirtualKeyCode, ElementState};

pub fn translate_code(winit_code: VirtualKeyCode) -> KeyCode {
    match winit_code {
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Down => KeyCode::Down,
        _ => KeyCode::Unknown
    }
}

pub fn translate_state(winit_state: ElementState) -> InputState {
    match winit_state {
        ElementState::Pressed => InputState::Pressed,
        ElementState::Released => InputState::Released
    }
}
