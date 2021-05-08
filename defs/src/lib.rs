
use model::factory::StaticVertex;

use raw_window_handle::HasRawWindowHandle;
use cgmath::Matrix4;

pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}

pub enum Shader {
    PlainPnt, // Position-Normal-Texture, R8G8B8A8 texture, no lighting
    Text,     // Position-Normal-Texture, R8 texture, no lighting
}

pub enum TexturePixelFormat {
    RGBA
}

pub enum VertexFormat {
    PositionNormalTexture
}

pub enum PostStep {
    Nothing
}

impl Default for PostStep {
    fn default() -> Self {
        PostStep::Nothing
    }
}

pub trait RendererApi {
    fn new(window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<Self, String> where Self : Sized;
    fn draw_next_frame(&mut self, camera_matrix: Matrix4<f32>) -> Result<PresentResult, String>;
    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), String>;
    fn get_aspect_ratio(&self) -> f32;
}

pub struct DecodedTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TexturePixelFormat
}

pub struct DrawingPass {
    pub shader: Shader,
    pub vertex_format: VertexFormat,
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>,
    pub texture: DecodedTexture,
    pub depth_test: bool
}

#[derive(Default)]
pub struct DrawingDescription {
    pub passes: Vec<DrawingPass>,
    pub post_step: PostStep
}
