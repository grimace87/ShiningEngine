
pub enum Shader {
    TextureFlat
}

pub enum Model {
    MenuScene,
    Grimace
}

pub enum TextureSource {
    Terrain,
    MusicaFont
}

pub enum Texture {
    Jpeg(TextureSource),
    Png(TextureSource)
}

pub struct PipelineConfig {
    pub shader: Shader,
    pub model: Model,
    pub texture: Texture
}
