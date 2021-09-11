
use defs::control::{
    Control,
    InputState,
    KeyCode
};

/// NullControl struct
/// Empty control implementation; does nothing
pub struct NullControl {}

impl Default for NullControl {

    /// Construct a new instance
    fn default() -> NullControl {
        NullControl {}
    }
}

impl Control for NullControl {

    /// No-op
    fn update(&mut self) {}

    /// No-op
    fn process_keyboard_event(&mut self, _keycode: KeyCode, _state: InputState) {}

    /// Signal no user input for left/right
    fn get_dx(&self) -> f32 {
        0.0
    }

    /// Signal no user input for up/down
    fn get_dy(&self) -> f32 {
        0.0
    }
}
