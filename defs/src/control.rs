
/// KeyCode enum
/// Platform-independent abstraction for key codes that the engine recognises.
pub enum KeyCode {
    Unknown,
    Left,
    Up,
    Down,
    Right
}

/// InputState enum
/// Platform-independent abstraction for the state that an input can be in at any given moment.
pub enum InputState {
    Pressed,
    Released
}

/// Control trait
/// Abstraction for an entity that polls and receives input states.
pub trait Control {

    /// Instruct this control to update itself
    fn update(&mut self);

    /// Process a keyboard event
    fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState);

    /// Retrieve the left/right direction currently being input
    fn get_dx(&self) -> f32;

    /// Retrieve the up/down direction currently being input
    fn get_dy(&self) -> f32;
}
