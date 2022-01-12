
use crate::GeneratorError;
use crate::deserialiser::types::{*, app::*, scene::*};

fn translate_texture_format(format: &TextureFormat) -> Result<String, GeneratorError> {
    match format {
        TextureFormat::r8 => Err(GeneratorError::InvalidSpec(
            format!("Texture format r8 not implemented"))),
        TextureFormat::rgb8 => Err(GeneratorError::InvalidSpec(
            format!("Texture format rgb8 not implemented"))),
        TextureFormat::rgba8 => Ok("TexturePixelFormat::Rgba".to_string()),
        TextureFormat::d16 => Ok("TexturePixelFormat::Unorm16".to_string())
    }
}

fn translate_shader(shader: &RenderFunction) -> String {
    let function = match shader {
        RenderFunction::basic_textured => "Shader::PlainPnt",
        RenderFunction::basic_textured_clipped_y => "Shader::PlainPntClipped",
        RenderFunction::text_paint => "Shader::Text",
        RenderFunction::cube => "Shader::Cube",
        RenderFunction::cube_clipped_y => "Shader::CubeClipped",
        RenderFunction::reflection_pre_render => "Shader::Water"
    };
    String::from(function)
}

pub fn generate_description(shared_resources: &Resources, config: &Scene) -> Result<String, GeneratorError> {
    let mut passes = String::new();
    for pass in config.passes.iter() {

        let target = match &pass.target_texture_ids {
            None => String::from("target: FramebufferTarget::DefaultFramebuffer"),
            Some(texture_ids) => {

                let (colour_format, colour_texture_index) = {
                    if let Some(resource) = config.resources.textures.iter().find(|texture| &texture.id == &texture_ids.colour_texture_id) {
                        (translate_texture_format(&resource.format)?, format!("TEXTURE_INDEX_{}", texture_ids.colour_texture_id.to_uppercase()))
                    } else if let Some(resource) = shared_resources.textures.iter().find(|texture| &texture.id == &texture_ids.colour_texture_id) {
                        (translate_texture_format(&resource.format)?, format!("shared_indices::TEXTURE_INDEX_{}", texture_ids.colour_texture_id.to_uppercase()))
                    } else {
                        return Err(GeneratorError::InvalidSpec(
                            format!("(Scene {}) Texture doesn't exist: {}", config.id, texture_ids.colour_texture_id)))
                    }
                };

                let (depth_format, depth_texture_index) = match &texture_ids.depth_texture_id {
                    Some(id) => {
                        if let Some(resource) = config.resources.textures.iter().find(|texture| &texture.id == id) {
                            (translate_texture_format(&resource.format)?, format!("TEXTURE_INDEX_{}", id.to_uppercase()))
                        } else if let Some(resource) = shared_resources.textures.iter().find(|texture| &texture.id == id) {
                            (translate_texture_format(&resource.format)?, format!("shared_indices::TEXTURE_INDEX_{}", id.to_uppercase()))
                        } else {
                            return Err(GeneratorError::InvalidSpec(
                                format!("(Scene {}) Texture doesn't exist: {}", config.id, texture_ids.colour_texture_id)))
                        }
                    },
                    None => ("None".to_string(), "TexturePixelFormat::None".to_string())
                };
                format!("target: FramebufferTarget::Texture(FramebufferCreationData {{
                        color_texture_index: {},
                        depth_texture_index: Some({}),
                        width: OFFSCREEN_RENDER_SIZE as usize,
                        height: OFFSCREEN_RENDER_SIZE as usize,
                        color_format: {},
                        depth_format: {}
                    }})", colour_texture_index, depth_texture_index, colour_format, depth_format)
            }
        };

        let mut steps = String::from("");
        for step in pass.steps.iter() {

            let shader = translate_shader(&step.render);

            let mut texture_indices = String::new();
            let texture_count = step.texture_ids.len();
            for (index, texture) in step.texture_ids.iter().enumerate() {
                let texture_index_name = {
                    if let Some(_) = config.resources.textures.iter().find(|t| texture == &t.id) {
                        format!("TEXTURE_INDEX_{}", texture.to_uppercase())
                    } else if let Some(_) = shared_resources.textures.iter().find(|t| texture == &t.id) {
                        format!("shared_indices::TEXTURE_INDEX_{}", texture.to_uppercase())
                    } else {
                        return Err(GeneratorError::InvalidSpec(
                            format!("(Scene {}) Texture doesn't exist: {}", config.id, texture)))
                    }
                };
                if index < texture_count - 1 {
                    texture_indices = format!("{}{}, ", texture_indices, texture_index_name);
                } else {
                    texture_indices = format!("{}{}", texture_indices, texture_index_name);
                }
            }

            let vbo_index_name = {
                if let Some(_) = config.resources.models.iter().find(|m| &step.model_id == &m.id) {
                    format!("VBO_INDEX_{}", step.model_id.to_uppercase())
                } else if let Some(_) = shared_resources.models.iter().find(|m| &step.model_id == &m.id) {
                    format!("shared_indices::VBO_INDEX_{}", step.model_id.to_uppercase())
                } else {
                    return Err(GeneratorError::InvalidSpec(
                        format!("(Scene {}) Model doesn't exist: {}", config.id, step.model_id)))
                }
            };
            steps = format!("{}
                        DrawingStep {{
                            shader: {},
                            vbo_index: {},
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_indices: vec![{}],
                            depth_test: true
                        }},", steps, shader, vbo_index_name, texture_indices);
        }

        passes = format!("{}
                DrawingPass {{
                    {},
                    steps: vec![{}
                    ]
                }},",
            passes,
            target,
            steps
        );
    }
    let content = format!("
    fn make_description(&self) -> DrawingDescription {{
        DrawingDescription {{
            passes: vec![{}
            ]
        }}
    }}
    ", passes);
    Ok(content)
}
