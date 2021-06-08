
use defs::{Control, InputState, KeyCode};

pub struct NullControl {}

impl NullControl {
    pub fn new() -> NullControl {
        NullControl {}
    }
}

impl Control for NullControl {
    fn update(&mut self) {}

    fn process_keyboard_event(&mut self, _keycode: KeyCode, _state: InputState) {}

    fn get_dx(&self) -> f32 {
        0.0
    }

    fn get_dy(&self) -> f32 {
        0.0
    }
}
