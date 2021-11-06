use serde::Deserialize;

/// App struct
/// Defines some top-level properties of the application
#[derive(Debug, Deserialize)]
pub struct App {
    pub name: String,
    pub features: Vec<String>,
    pub platform: String,
    pub graphics: String
}

/// Scene struct
/// Defines a scene of the application - generally this will be a continuous piece of gameplay
/// without interruptions, such as a player flying across a planet - and typically separated from
/// other scenes by a screen transition of some sort in which there's briefly no player interaction
#[derive(Debug, Deserialize)]
pub struct Scene {
    pub id: String,
    pub camera: String,
    pub resources: Resources,
    pub passes: Vec<Pass>
}

/// Resources struct
/// Defines all the resources that will be used by the scene and should therefore be preloaded
/// upfront at the same time
#[derive(Debug, Deserialize)]
pub struct Resources {
    pub models: Vec<Model>,
    pub textures: Vec<Texture>,
    pub fonts: Vec<Font>
}

/// Model struct
/// Defines a model, including some kind of source of its vertex data
#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub file: Option<String>,
    pub generator: Option<String>
}

/// Texture struct
/// Defines a texture, optionally including some kind of source of its pixel data
#[derive(Debug, Deserialize)]
pub struct Texture {
    pub id: String,
    pub file: Option<String>,
    pub kind: Option<String>
}

/// Font struct
/// Defines a font, including its glyph definition file and a reference to the texture which holds
/// its pixel data
#[derive(Debug, Deserialize)]
pub struct Font {
    pub id: String,
    pub file: String,
    pub texture_id: String
}

/// Pass struct
/// Defines one of the one-or-more rendering passes required to draw this scene, including a shader
/// and render target (offscreen buffer or the default framebuffer), plus the various steps to draw
/// using that configuration
#[derive(Debug, Deserialize)]
pub struct Pass {
    pub kind: String,
    pub target_texture_id: Option<String>,
    pub render: String,
    pub steps: Vec<Step>
}

/// Step struct
/// Defines a step within a pass - this basically comprises drawing a model with whatever number of
/// textures are required by the shader for the parent pass
#[derive(Debug, Deserialize)]
pub struct Step {
    pub model_id: String,
    pub texture_ids: Vec<String>
}
