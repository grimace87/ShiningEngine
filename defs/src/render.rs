
use model::types::StaticVertex;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;

/// FeatureDeclaration enum
/// Platform feature requirements that may be declared by an application or component thereof in
/// advance, in case it's needed during initialisation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FeatureDeclaration {
    ClipPlanes // Vulkan - see VkPhysicalDeviceFeatures.shaderClipDistance
}

/// PresentResult struct
/// Possible outcomes of a presentation action.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}

/// Shader enum
/// An enumeration of the available shaders in the engine.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Shader {
    PlainPnt,        // Position-Normal-Texture, R8G8B8A8 texture, no lighting
    PlainPntClipped, // Position-Normal-Texture, R8G8B8A8 texture, no lighting, clip Y
    Text,            // Position-Normal-Texture, R8 texture, no lighting
    Cube,            // Position, cube texture, no lighting
    CubeClipped,     // Position, cube texture, no lighting, clip Y
    Water,           // Position-Normal-Texture, R8G8B8A8 texture, no lighting, projective texture
                     // coords
}

/// ImageUsage enum
/// An enumeration of what purpose image resources can be used for
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageUsage {
    TextureSampleOnly,
    DepthBuffer,
    OffscreenRenderSampleColorWriteDepth,
    Skybox
}

/// TexturePixelFormat enum
/// Abstraction of the set of pixel formats known by the engine
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TexturePixelFormat {
    None,
    Rgba,
    Unorm16
}

/// VertexFormat enum
/// Abstraction of the set of vertex formats known by the engine
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VertexFormat {
    PositionNormalTexture
}

/// FramebufferTarget enum
/// Specification of the target for rendering - either directly to a swapchain image or to an
/// off-screen target specified by a description structure
pub enum FramebufferTarget {
    DefaultFramebuffer,
    Texture(FramebufferCreationData)
}

/// VboCreationData struct
/// Specification for how a vertex buffer is to be created
pub struct VboCreationData {
    pub vertex_format: VertexFormat,
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>
}

/// TextureCreationData struct
/// Specification for how a texture resource is to be created
pub struct TextureCreationData {
    pub layer_data: Option<Vec<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
    pub format: TexturePixelFormat,
    pub usage: ImageUsage
}

/// FramebufferCreationData struct
/// Specification for how a framebuffer (render target) resource is to be created
pub struct FramebufferCreationData {
    pub color_texture_index: usize,
    pub depth_texture_index: Option<usize>,
    pub width: usize,
    pub height: usize,
    pub color_format: TexturePixelFormat,
    pub depth_format: TexturePixelFormat
}

/// DrawingStep struct
/// Description of a rendering step, including the shader and the resources needed by the shader.
pub struct DrawingStep {
    pub shader: Shader,
    pub vbo_index: usize,
    pub vbo_format: VertexFormat,
    pub draw_indexed: bool,
    pub texture_indices: Vec<usize>,
    pub depth_test: bool
}

/// DrawingPass struct
/// Description of a set of drawing commands, run in sequence, targeting a single framebuffer
/// (render target).
pub struct DrawingPass {
    pub target: FramebufferTarget,
    pub steps: Vec<DrawingStep>
}

/// DrawingDescription
/// Complete description of how to render a frame, encompassing all steps necessary.
pub struct DrawingDescription {
    pub passes: Vec<DrawingPass>
}

/// ResourcePreloads struct
/// Encapsulates everything needed to initialise all of the resources that need to be preloaded in
/// order to render a scene.
pub struct ResourcePreloads {
    pub vbo_preloads: HashMap<usize, VboCreationData>,
    pub texture_preloads: HashMap<usize, TextureCreationData>
}

/// RendererApi trait
/// Interface between the abstract scene descriptions and the graphics API that renders everything.
pub trait RendererApi {

    /// Construct a new instance of this implementation
    fn new(
        window_owner: &dyn HasRawWindowHandle,
        features: &[FeatureDeclaration],
        resource_preloads: &ResourcePreloads,
        description: &DrawingDescription
    ) -> Result<Self, crate::EngineError> where Self : Sized;

    /// Instruct this implementation to render the next frame, assuming necessary pre-rendering
    /// operations have been performed already
    fn draw_next_frame(
        &mut self,
        scene_info: &dyn crate::Scene
    ) -> Result<PresentResult, crate::EngineError>;

    /// Instruct this implementation to recreate the drawing surface
    fn recreate_surface(
        &mut self,
        window_owner: &dyn HasRawWindowHandle,
        description: &DrawingDescription
    ) -> Result<(), crate::EngineError>;

    /// Instruct this implementation to create the resources needed for the given scene
    fn recreate_scene_resources(
        &mut self,
        resource_preloads: &ResourcePreloads,
        description: &DrawingDescription
    ) -> Result<(), crate::EngineError>;

    /// Retrueve the current aspect ratio of the client area, as known to this implementation
    fn get_aspect_ratio(&self) -> f32;
}
