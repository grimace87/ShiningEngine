
use defs::{RendererApi, PresentResult, SceneInfo, DrawingDescription};
use raw_window_handle::HasRawWindowHandle;

pub struct NullRenderer;

impl NullRenderer {
    pub fn new() -> NullRenderer {
        NullRenderer
    }
}

impl RendererApi for NullRenderer {
    fn new(_window_owner: &dyn HasRawWindowHandle, _description: &DrawingDescription) -> Result<Self, String> where Self: Sized {
        Ok(NullRenderer)
    }

    fn draw_next_frame(&mut self, _scene_info: &dyn SceneInfo) -> Result<PresentResult, String> {
        Ok(PresentResult::Ok)
    }

    fn recreate_swapchain(&mut self, _window_owner: &dyn HasRawWindowHandle, _description: &DrawingDescription) -> Result<(), String> {
        Ok(())
    }

    fn recreate_scene_resources(&mut self, _description: &DrawingDescription) -> Result<(), String> {
        Ok(())
    }

    fn get_aspect_ratio(&self) -> f32 {
        1.0
    }
}
