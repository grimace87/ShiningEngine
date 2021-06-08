
use defs::{Control, KeyCode, InputState};

pub struct UserControl {
    dx: f32,
    dy: f32
}

impl UserControl {
    pub fn new() -> UserControl {
        UserControl {
            dx: 0.0,
            dy: 0.0
        }
    }
}

impl Control for UserControl {
    fn update(&mut self) {

    }

    fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState) {
        match keycode {
            KeyCode::Left => {
                self.dx = match state {
                    InputState::Pressed => -1.0,
                    InputState::Released => 0.0
                };
            },
            KeyCode::Right => {
                self.dx = match state {
                    InputState::Pressed => 1.0,
                    InputState::Released => 0.0
                };
            },
            KeyCode::Up => {
                self.dy = match state {
                    InputState::Pressed => 1.0,
                    InputState::Released => 0.0
                };
            },
            KeyCode::Down => {
                self.dy = match state {
                    InputState::Pressed => -1.0,
                    InputState::Released => 0.0
                };
            },
            _ => {}
        }
    }

    fn get_dx(&self) -> f32 {
        self.dx
    }

    fn get_dy(&self) -> f32 {
        self.dy
    }
}
