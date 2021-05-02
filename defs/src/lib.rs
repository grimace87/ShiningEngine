
use raw_window_handle::HasRawWindowHandle;
use cgmath::Matrix4;

pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}

pub trait RendererApi {
    fn new(window_owner: &dyn HasRawWindowHandle) -> Result<Self, String> where Self : Sized;
    fn draw_next_frame(&mut self, camera_matrix: Matrix4<f32>) -> Result<PresentResult, String>;
    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String>;
    fn get_aspect_ratio(&self) -> f32;
}
