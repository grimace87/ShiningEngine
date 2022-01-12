use serde::Deserialize;

pub mod app;
pub mod scene;

/// Resources struct
/// Defines all the resources that will be used by the scene and should therefore be preloaded
/// upfront at the same time
#[derive(Debug, Deserialize, Default)]
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
    pub generator: Option<ModelGenerator>
}

#[derive(Debug, Deserialize)]
pub enum ModelGenerator {
    skybox,
    text
}

/// Texture struct
/// Defines a texture, optionally including some kind of source of its pixel data
#[derive(Debug, Deserialize)]
pub struct Texture {
    pub id: String,
    pub format: TextureFormat,
    pub file: Option<String>,
    pub kind: Option<TextureKind>
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum TextureFormat {
    r8,
    rgb8,
    rgba8,
    d16
}

#[derive(Debug, Deserialize)]
pub enum TextureKind {
    cubemap,
    uninitialised
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
