
pub mod user;

use self::user::UserControl;

pub enum KeyCode {
    Unknown,
    Left,
    Up,
    Down,
    Right
}

pub enum InputState {
    Pressed,
    Released
}

pub trait Control {
    fn update(&mut self);
    fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState);
    fn get_dx(&self) -> f32;
    fn get_dy(&self) -> f32;
}

pub fn new_control() -> UserControl {
    UserControl::new()
}
