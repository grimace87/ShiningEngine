
use defs::control::{
    KeyCode,
    InputState
};
use winit::event::{
    VirtualKeyCode,
    ElementState
};

/// Translate Winit key codes into the abstract codes from the defs crate
pub fn translate_code(winit_code: VirtualKeyCode) -> KeyCode {
    match winit_code {
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Down => KeyCode::Down,
        _ => KeyCode::Unknown
    }
}

/// Translate Winit element states into the abstract states from the defs crate
pub fn translate_state(winit_state: ElementState) -> InputState {
    match winit_state {
        ElementState::Pressed => InputState::Pressed,
        ElementState::Released => InputState::Released
    }
}
