
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

pub enum KeyCode {
    Unknown,
    Left,
    Up,
    Down,
    Right
}

pub enum InputState {
    Pressed,
    Released
}

pub trait Control {
    fn update(&mut self);
    fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState);
    fn get_dx(&self) -> f32;
    fn get_dy(&self) -> f32;
}

pub trait Camera {
    fn update_aspect(&mut self, aspect_ratio: f32);
    fn update(&mut self, time_step_millis: u64, controller: &dyn Control);
    fn get_matrix(&self) -> Matrix4<f32>;
}

pub trait RendererApi {
    fn new(window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<Self, String> where Self : Sized;
    fn draw_next_frame(&mut self, scene_info: &dyn SceneInfo) -> Result<PresentResult, String>;
    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), String>;
    fn recreate_scene_resources(&mut self, description: &DrawingDescription) -> Result<(), String>;
    fn get_aspect_ratio(&self) -> f32;
}

pub enum FramebufferTarget {
    DefaultFramebuffer
}

pub struct DecodedTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TexturePixelFormat
}

pub struct DrawingStep {
    pub shader: Shader,
    pub vertex_format: VertexFormat,
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>,
    pub texture: DecodedTexture,
    pub depth_test: bool
}

pub struct DrawingPass {
    pub target: FramebufferTarget,
    pub steps: Vec<DrawingStep>
}

#[derive(Default)]
pub struct DrawingDescription {
    pub passes: Vec<DrawingPass>
}

pub trait SceneManager {
    fn queue_scene(&self, new_scene: Box<dyn SceneInfo>);
}

pub trait SceneInfo {
    fn make_description(&self) -> DrawingDescription;
    fn update_aspect_ratio(&mut self, aspect_ratio: f32);
    fn update_camera(&mut self, time_step_millis: u64, controller: &dyn Control) -> Option<Box<dyn SceneInfo>>;
    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize) -> (*const u8, usize);
}
