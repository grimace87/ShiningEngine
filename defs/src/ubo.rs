
use cgmath::{Matrix4, Vector4};

#[repr(C)]
pub struct MvpUbo {
    pub matrix: Matrix4<f32>
}

#[repr(C)]
pub struct MvpClippingUbo {
    pub matrix: Matrix4<f32>,
    pub y_bias: f32,
    pub y_plane_normal: f32,
    pub unused: [f32; 2]
}

#[repr(C)]
pub struct TextPaintUbo {
    pub camera_matrix: Matrix4<f32>,
    pub paint_color: Vector4<f32>
}

#[repr(C)]
pub struct CameraUbo {
    pub camera_matrix: Matrix4<f32>
}
