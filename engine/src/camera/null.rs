
use defs::Camera;
use defs::Control;
use cgmath::{Matrix4, SquareMatrix};

pub struct NullCamera {}

impl NullCamera {
    pub fn new() -> NullCamera {
        NullCamera {}
    }
}

impl Camera for NullCamera {
    fn update_aspect(&mut self, _aspect_ratio: f32) {}

    fn update(&mut self, _time_step_millis: u64, _controller: &dyn Control) {}

    fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn get_projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }
}
