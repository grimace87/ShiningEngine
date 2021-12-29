
use crate::GeneratorError;
use crate::deserialiser::scene::*;

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

pub fn generate_description(config: &Scene) -> Result<String, GeneratorError> {
    let mut passes = String::new();
    for pass in config.passes.iter() {

        let target = match &pass.target_texture_ids {
            None => String::from("target: FramebufferTarget::DefaultFramebuffer"),
            Some(texture_ids) => {

                let colour_texture_resource = config.resources.textures.iter()
                    .find(|texture| &texture.id == &texture_ids.colour_texture_id);
                let colour_texture = match colour_texture_resource {
                    Some(texture) => texture,
                    None => return Err(GeneratorError::InvalidSpec(
                        format!("(Scene {}) Texture doesn't exist: {}", config.id, texture_ids.colour_texture_id)))
                };
                let colour_name = format!("TEXTURE_INDEX_{}", colour_texture.id.to_uppercase());
                let colour_format = translate_texture_format(&colour_texture.format)?;

                let (depth_name, depth_format) = match &texture_ids.depth_texture_id {
                    Some(id) => {
                        let resource = config.resources.textures.iter()
                            .find(|texture| &texture.id == id);
                        let texture = match resource {
                            Some(texture) => texture,
                            None => return Err(GeneratorError::InvalidSpec(
                                format!("(Scene {}) Texture doesn't exist: {}", config.id, texture_ids.colour_texture_id)))
                        };
                        let name = format!("TEXTURE_INDEX_{}", texture.id.to_uppercase());
                        let format = translate_texture_format(&texture.format)?;
                        (name, format)
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
                    }})", colour_name, depth_name, colour_format, depth_format)
            }
        };

        let mut steps = String::from("");
        for step in pass.steps.iter() {

            let shader = translate_shader(&step.render);

            let mut texture_indices = String::new();
            let texture_count = step.texture_ids.len();
            for (index, texture) in step.texture_ids.iter().enumerate() {
                if index < texture_count - 1 {
                    texture_indices = format!("{}TEXTURE_INDEX_{}, ", texture_indices, texture.to_uppercase());
                } else {
                    texture_indices = format!("{}TEXTURE_INDEX_{}", texture_indices, texture.to_uppercase());
                }
            }

            let vbo_index = format!("VBO_INDEX_{}", step.model_id.to_uppercase());
            steps = format!("{}
                        DrawingStep {{
                            shader: {},
                            vbo_index: {},
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_indices: vec![{}],
                            depth_test: true
                        }},", steps, shader, vbo_index, texture_indices);
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
