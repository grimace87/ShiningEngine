
use crate::deserialiser::scene::*;
use crate::GeneratorError;
use heck::CamelCase;

pub fn generate_regenerated_scene_contents(config: &Scene) -> Result<String, GeneratorError> {

    let mut byte_decls = String::new();
    for model in config.resources.models.iter() {
        if let Some(src_file) = &model.file {
            let decl = format!("const {}_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/models/{}\"));", model.id.to_uppercase(), src_file);
            byte_decls = format!("{}\n{}", byte_decls, decl);
        }
    }
    for texture in config.resources.textures.iter() {
        if let Some(src_file) = &texture.file {
            match texture.kind {
                None => {
                    let decl = format!("const {}_TEXTURE_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}\");", texture.id.to_uppercase(), src_file);
                    byte_decls = format!("{}\n{}", byte_decls, decl);
                },
                Some(TextureKind::cubemap) => {
                    let (name_part, extension) = match src_file.rfind(".") {
                        Some(index) => (&src_file[0..index], &src_file[index..]),
                        None => panic!("Could not find an extension in a cubemap texture file name")
                    };
                    let decls = vec![
                        format!("const {}_TEXTURE_LF_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_lf{}\");", name_part.to_uppercase(), name_part, extension),
                        format!("const {}_TEXTURE_RT_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_rt{}\");", name_part.to_uppercase(), name_part, extension),
                        format!("const {}_TEXTURE_UP_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_up{}\");", name_part.to_uppercase(), name_part, extension),
                        format!("const {}_TEXTURE_DN_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_dn{}\");", name_part.to_uppercase(), name_part, extension),
                        format!("const {}_TEXTURE_FT_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_ft{}\");", name_part.to_uppercase(), name_part, extension),
                        format!("const {}_TEXTURE_BK_BYTES: &[u8] = include_bytes!(\"../../resources/textures/{}_bk{}\");", name_part.to_uppercase(), name_part, extension)
                    ];
                    for decl in decls.iter() {
                        byte_decls = format!("{}\n{}", byte_decls, decl);
                    }
                },
                _ => panic!("Unexpected error - texture file specified, but kind is not compatible")
            }
        }
    }

    let mut vbo_index_decls = String::new();
    for (i, model) in config.resources.models.iter().enumerate() {
        vbo_index_decls = format!("{}\nconst VBO_INDEX_{}: usize = {};", vbo_index_decls, model.id.to_uppercase(), i);
    }

    let mut texture_index_decls = String::new();
    for (i, texture) in config.resources.textures.iter().enumerate() {
        texture_index_decls = format!("{}\nconst TEXTURE_INDEX_{}: usize = {};", texture_index_decls, texture.id.to_uppercase(), i);
    }

    let struct_name = format!("{}Scene", config.id.to_camel_case());

    let (camera_type, camera_constructor) = match config.camera {
        Camera::null => (
            "NullCamera",
            "NullCamera::default()"
        ),
        Camera::player => (
            "PlayerCamera",
            "PlayerCamera::new(1.0, 10.0, -3.0, -15.0, std::f32::consts::FRAC_PI_6 * 5.0)"
        ),
        Camera::flight_path => unimplemented!()
    };

    let (text_gen_decls, text_gen_constructors) = match config.resources.fonts.len() {
        0 => (
            String::new(),
            String::new()
        ),
        1 => (
            String::from("\n    text_generator: TextGenerator,"),
            format!(
                "\n            text_generator: TextGenerator::from_resource(include_str!(\"../../resources/font/{}\")),",
                config.resources.fonts[0].file
            )
        ),
        _ => {
            let mut decls = String::new();
            let mut constructors = String::new();
            for (i, _) in config.resources.fonts.iter().enumerate() {
                decls = format!("{}\n    text_generator_{}: TextGenerator,", decls, i);
                constructors = format!(
                    "{}\n            text_generator_{}: TextGenerator::from_resource(include_str!(\"../../resources/font/{}\")),",
                    constructors,
                    i,
                    config.resources.fonts[i].file
                );
            }
            (decls, constructors)
        }
    };

    let (ubo_decls, ubo_constructors, ubo_ptr_mappings) = {
        let mut decls = String::new();
        let mut constructors = String::new();
        let mut ptr_mappings = String::new();
        for (pass_index, pass) in config.passes.iter().enumerate() {
            for (step_index, step) in pass.steps.iter().enumerate() {

                let ubo_type = match pass.render {
                    RenderFunction::basic_textured => "MvpUbo",
                    RenderFunction::text_paint => "TextPaintUbo",
                    RenderFunction::reflection_pre_render => "MvpClippingUbo"
                };
                decls = format!("{}\n    ubo_{}_{}: {},", decls, pass.name, step.name, ubo_type);

                let ubo_constructor = match pass.render {
                    RenderFunction::basic_textured => "MvpUbo { matrix: Matrix4::identity() }",
                    RenderFunction::text_paint => "TextPaintUbo {\n                camera_matrix: Matrix4::identity(),\n                paint_color: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 }\n            }",
                    RenderFunction::reflection_pre_render => "MvpClippingUbo {\n                matrix: Matrix4::identity(),\n                y_bias: 0.0,\n                y_plane_normal: -1.0,\n                unused: [0.0, 0.0]\n            }"
                };
                constructors = format!("{}\n            ubo_{}_{}: {},", constructors, pass.name, step.name, ubo_constructor);

                ptr_mappings = format!("{}\n            ({}, {}) => (\n                &self.ubo_{}_{} as *const {} as *const u8,\n                std::mem::size_of::<{}>()),", ptr_mappings, pass_index, step_index, pass.name, step.name, ubo_type, ubo_type);
            }
        }
        (decls, constructors, ptr_mappings)
    };

    let gen_content = format!("
use defs::{{
    Camera,
    SceneInfo,
    control::Control,
    render::{{
        Shader,
        VertexFormat,
        FramebufferTarget,
        ResourcePreloads,
        VboCreationData,
        TextureCreationData,
        FramebufferCreationData,
        TexturePixelFormat,
        ImageUsage,
        DrawingDescription,
        DrawingPass,
        DrawingStep
    }},
    ubo::*
}};
use engine::{{
    camera::player::PlayerCamera,
    util::{{
        TextureCodec,
        decode_texture,
        decode_texture_array,
        make_skybox_vertices,
        decode_model,
        textbuffer::{{
            TextGenerator,
            TextAlignment
        }}
    }}
}};

use cgmath::{{Matrix4, Vector4, SquareMatrix}};
use std::collections::HashMap;
{}
{}
{}

const OFFSCREEN_RENDER_SIZE: u32 = 1024;

pub struct {} {{
    camera: {},{}{}
}}

impl {} {{
    pub fn new() -> {} {{
        {} {{
            camera: {},{}{}
        }}
    }}
}}

impl SceneInfo for {} {{

    fn make_preloads(&self) -> ResourcePreloads {{

    }}

    fn make_description(&self) -> DrawingDescription {{

    }}

    unsafe fn get_ubo_data_ptr_and_size(
        &self,
        pass_index: usize,
        step_index: usize
    ) -> (*const u8, usize) {{
        match (pass_index, step_index) {{{}
            _ => panic!(\"Cannot get UBO for {}\")
        }}
    }}
}}
",
                                     byte_decls,
                                     vbo_index_decls,
                                     texture_index_decls,
                                     struct_name,
                                     camera_type,
                                     text_gen_decls,
                                     ubo_decls,
                                     struct_name,
                                     struct_name,
                                     struct_name,
                                     camera_constructor,
                                     text_gen_constructors,
                                     ubo_constructors,
                                     struct_name,
                                     ubo_ptr_mappings,
                                     struct_name
    );

    Ok(gen_content)
}
