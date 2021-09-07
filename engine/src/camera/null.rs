
use defs::{
    Camera,
    control::Control
};
use cgmath::{
    Matrix4,
    SquareMatrix
};

/// NullCamera struct
/// Camera implementation that does nothing.
pub struct NullCamera {}

impl NullCamera {

    /// Object with no fields
    pub fn new() -> NullCamera {
        NullCamera {}
    }
}

impl Camera for NullCamera {

    /// No-op
    fn update_aspect(&mut self, _aspect_ratio: f32) {}

    /// No-op
    fn update(&mut self, _time_step_millis: u64, _controller: &dyn Control) {}

    /// Returns an identity matrix
    fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    /// Returns an identity matrix
    fn get_projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }
}
