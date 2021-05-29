
pub mod null;
pub mod player;

use self::player::PlayerCamera;
use crate::control::Control;

use cgmath::Matrix4;

pub trait Camera {
    fn update_aspect(&mut self, aspect_ratio: f32);
    fn advance(&mut self, time_step_millis: u64, controller: &dyn Control);
    fn get_matrix(&self) -> Matrix4<f32>;
}
