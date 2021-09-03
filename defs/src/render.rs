
pub use crate::SceneInfo;

use model::factory::StaticVertex;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FeatureDeclaration {
    ClipPlanes // Vulkan - see VkPhysicalDeviceFeatures.shaderClipDistance
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Shader {
    PlainPnt,        // Position-Normal-Texture, R8G8B8A8 texture, no lighting
    PlainPntClipped, // Position-Normal-Texture, R8G8B8A8 texture, no lighting, clip Y
    Text,            // Position-Normal-Texture, R8 texture, no lighting
    Cube,            // Position, cube texture, no lighting
    CubeClipped,     // Position, cube texture, no lighting, clip Y
    Water,           // Position-Normal-Texture, R8G8B8A8 texture, no lighting, projective texture coords
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageUsage {
    TextureSampleOnly,
    DepthBuffer,
    OffscreenRenderSampleColorWriteDepth,
    Skybox
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TexturePixelFormat {
    None,
    RGBA,
    Unorm16
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VertexFormat {
    PositionNormalTexture
}

pub enum FramebufferTarget {
    DefaultFramebuffer,
    Texture(FramebufferCreationData)
}

pub struct VboCreationData {
    pub vertex_format: VertexFormat,
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>
}

pub struct TextureCreationData {
    pub layer_data: Option<Vec<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
    pub format: TexturePixelFormat,
    pub usage: ImageUsage
}

pub struct FramebufferCreationData {
    pub color_texture_index: usize,
    pub depth_texture_index: Option<usize>,
    pub width: usize,
    pub height: usize,
    pub color_format: TexturePixelFormat,
    pub depth_format: TexturePixelFormat
}

pub struct DrawingStep {
    pub shader: Shader,
    pub vbo_index: usize,
    pub vbo_format: VertexFormat,
    pub draw_indexed: bool,
    pub texture_indices: Vec<usize>,
    pub depth_test: bool
}

pub struct DrawingPass {
    pub target: FramebufferTarget,
    pub steps: Vec<DrawingStep>
}

pub struct DrawingDescription {
    pub passes: Vec<DrawingPass>
}

pub struct ResourcePreloads {
    pub vbo_preloads: HashMap<usize, VboCreationData>,
    pub texture_preloads: HashMap<usize, TextureCreationData>
}

pub trait RendererApi {
    fn new(window_owner: &dyn HasRawWindowHandle, features: &Vec<FeatureDeclaration>, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<Self, String> where Self : Sized;
    fn draw_next_frame(&mut self, scene_info: &dyn SceneInfo) -> Result<PresentResult, String>;
    fn recreate_surface(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), String>;
    fn recreate_scene_resources(&mut self, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<(), String>;
    fn get_aspect_ratio(&self) -> f32;
}
