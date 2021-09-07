
use defs::{
    SceneInfo,
    EngineError,
    render::{
        RendererApi,
        PresentResult,
        DrawingDescription,
        ResourcePreloads,
        FeatureDeclaration
    }
};
use raw_window_handle::HasRawWindowHandle;

/// NullRenderer struct
/// Implementation of the RendererApi trait that does nothing
pub struct NullRenderer;

impl NullRenderer {

    /// Create a new instance; has no fields
    pub fn new() -> NullRenderer {
        NullRenderer
    }
}

impl RendererApi for NullRenderer {

    /// Create a new instance, given the window and app requirements; still does nothing
    fn new(
        _window_owner: &dyn HasRawWindowHandle,
        _features: &Vec<FeatureDeclaration>,
        _resource_preloads: &ResourcePreloads,
        _description: &DrawingDescription
    ) -> Result<Self, EngineError> where Self: Sized {
        Ok(NullRenderer)
    }

    /// No-op; acts as if it succeeded
    fn draw_next_frame(&mut self, _scene_info: &dyn SceneInfo) -> Result<PresentResult, EngineError> {
        Ok(PresentResult::Ok)
    }

    /// No-op
    fn recreate_surface(&mut self, _window_owner: &dyn HasRawWindowHandle, _description: &DrawingDescription) -> Result<(), EngineError> {
        Ok(())
    }

    /// No-op
    fn recreate_scene_resources(&mut self, _resource_preloads: &ResourcePreloads, _description: &DrawingDescription) -> Result<(), EngineError> {
        Ok(())
    }

    /// Return sensible aspect ratio number; static value
    fn get_aspect_ratio(&self) -> f32 {
        1.0
    }
}
