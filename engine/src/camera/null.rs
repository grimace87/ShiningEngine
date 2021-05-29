
use crate::camera::Camera;
use crate::Control;
use cgmath::{Matrix4, SquareMatrix};

pub struct NullCamera {}

impl NullCamera {
    pub fn new() -> NullCamera {
        NullCamera {}
    }
}

impl Camera for NullCamera {
    fn update_aspect(&mut self, _aspect_ratio: f32) {}

    fn advance(&mut self, _time_step_millis: u64, _controller: &dyn Control) {}

    fn get_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }
}
