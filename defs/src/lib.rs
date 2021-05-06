
use model::factory::StaticVertex;

use raw_window_handle::HasRawWindowHandle;
use cgmath::Matrix4;

pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}

pub trait RendererApi {
    fn new(window_owner: &dyn HasRawWindowHandle, descriptions: &Vec<SceneDescription>) -> Result<Self, String> where Self : Sized;
    fn draw_next_frame(&mut self, camera_matrix: Matrix4<f32>) -> Result<PresentResult, String>;
    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle, descriptions: &Vec<SceneDescription>) -> Result<(), String>;
    fn get_aspect_ratio(&self) -> f32;
}

pub enum VertexFormat {
    PositionNormalTexture
}

pub enum TexturePixelFormat {
    RGBA
}

pub enum PostStep {
    Nothing
}

pub struct SceneDescription {
    pub vertex_format: VertexFormat,
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>,
    pub texture_format: TexturePixelFormat,
    pub texture_data: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub depth_test: bool,
    pub post_step: PostStep
}
