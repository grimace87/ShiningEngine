
pub mod player;

use self::player::PlayerCamera;
use crate::control::Control;

use cgmath::Matrix4;

pub trait Camera<T> where T : Control {
    fn update_aspect(&mut self, aspect_ratio: f32);
    fn advance(&mut self, time_step_millis: u64, controller: &T);
    fn get_matrix(&self) -> Matrix4<f32>;
}

pub fn new_camera(aspect_ratio: f32) -> PlayerCamera {
    PlayerCamera::new(aspect_ratio)
}
