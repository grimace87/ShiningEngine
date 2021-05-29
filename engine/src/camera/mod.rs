
pub mod null;
pub mod player;

use crate::control::Control;

use cgmath::Matrix4;

pub trait Camera {
    fn update_aspect(&mut self, aspect_ratio: f32);
    fn update(&mut self, time_step_millis: u64, controller: &dyn Control);
    fn get_matrix(&self) -> Matrix4<f32>;
}
