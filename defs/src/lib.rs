
pub mod render;
pub use render::{
    ResourcePreloads,
    DrawingDescription
};

pub mod control;
pub use control::Control;

use cgmath::Matrix4;

pub trait Camera {
    fn update_aspect(&mut self, aspect_ratio: f32);
    fn update(&mut self, time_step_millis: u64, controller: &dyn Control);
    fn get_view_matrix(&self) -> Matrix4<f32>;
    fn get_projection_matrix(&self) -> Matrix4<f32>;
}

pub trait SceneManager {
    fn queue_scene(&self, new_scene: Box<dyn SceneInfo>);
}

pub trait SceneInfo {
    fn make_preloads(&self) -> ResourcePreloads;
    fn make_description(&self) -> DrawingDescription;
    fn update_aspect_ratio(&mut self, aspect_ratio: f32);
    fn update_camera(&mut self, time_step_millis: u64, controller: &dyn Control) -> Option<Box<dyn SceneInfo>>;
    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize, step_index: usize) -> (*const u8, usize);
}
