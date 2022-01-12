use serde::Deserialize;
use crate::deserialiser::types::Resources;

/// Scene struct
/// Defines a scene of the application - generally this will be a continuous piece of gameplay
/// without interruptions, such as a player flying across a planet - and typically separated from
/// other scenes by a screen transition of some sort in which there's briefly no player interaction
#[derive(Debug, Deserialize)]
pub struct Scene {
    pub id: String,
    pub camera: Camera,
    pub resources: Resources,
    pub passes: Vec<Pass>
}

#[derive(Debug, Deserialize)]
pub enum Camera {
    player,
    flight_path,
    null
}

/// Pass struct
/// Defines one of the one-or-more rendering passes required to draw this scene, including a shader
/// and render target (offscreen buffer or the default framebuffer), plus the various steps to draw
/// using that configuration
#[derive(Debug, Deserialize)]
pub struct Pass {
    pub name: String,
    pub kind: PassKind,
    pub target_texture_ids: Option<TextureTarget>,
    pub steps: Vec<Step>
}

#[derive(Debug, Deserialize)]
pub enum PassKind {
    default,
    offscreen
}

#[derive(Debug, Deserialize)]
pub struct TextureTarget {
    pub colour_texture_id: String,
    pub depth_texture_id: Option<String>
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum RenderFunction {
    basic_textured,
    basic_textured_clipped_y,
    text_paint,
    cube,
    cube_clipped_y,
    reflection_pre_render
}

/// Step struct
/// Defines a step within a pass - this basically comprises drawing a model with whatever number of
/// textures are required by the shader for the parent pass
#[derive(Debug, Deserialize)]
pub struct Step {
    pub name: String,
    pub render: RenderFunction,
    pub model_id: String,
    pub texture_ids: Vec<String>
}
