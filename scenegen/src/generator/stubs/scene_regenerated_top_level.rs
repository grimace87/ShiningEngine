
use crate::GeneratorError;
use crate::deserialiser::scene::*;

pub fn generate_top_level(
    config: &Scene,
    resources_dir_name: &'static str
) -> Result<String, GeneratorError> {

    let mut additional_util_imports = String::new();
    if config.resources.fonts.len() > 0 {
        additional_util_imports = format!("{}
        textbuffer::{{
            TextGenerator,
            TextAlignment
        }},", additional_util_imports)
    }

    let mut additional_cgmath_imports = String::new();
    let mut will_paint_text = false;
    for pass in config.passes.iter() {
        for step in pass.steps.iter() {
            if step.render == RenderFunction::text_paint {
                will_paint_text = true;
            }
        }
    }
    if will_paint_text {
        additional_cgmath_imports = format!("{}, Vector4", additional_cgmath_imports);
    }

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
                    let decl = format!("const {}_TEXTURE_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}\"));", texture.id.to_uppercase(), resources_dir_name, src_file);
                    byte_decls = format!("{}\n{}", byte_decls, decl);
                },
                Some(TextureKind::cubemap) => {
                    let (name_part, extension) = match src_file.rfind(".") {
                        Some(index) => (&src_file[0..index], &src_file[index..]),
                        None => panic!("Could not find an extension in a cubemap texture file name")
                    };
                    let decls = vec![
                        format!("const {}_TEXTURE_LF_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_lf{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension),
                        format!("const {}_TEXTURE_RT_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_rt{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension),
                        format!("const {}_TEXTURE_UP_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_up{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension),
                        format!("const {}_TEXTURE_DN_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_dn{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension),
                        format!("const {}_TEXTURE_FT_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_ft{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension),
                        format!("const {}_TEXTURE_BK_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/textures/{}_bk{}\"));", texture.id.to_uppercase(), resources_dir_name, name_part, extension)
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

    let gen_content = format!("\
use defs::{{
    SceneInfo,
    render::{{
        Shader,
        VertexFormat,
        FramebufferTarget,
        ResourcePreloads,
        VboCreationData,
        TextureCreationData,
        ImageUsage,
        DrawingDescription,
        DrawingPass,
        DrawingStep
    }},
    ubo::*
}};
use engine::{{
    util::{{{}
        TextureCodec
    }}
}};

use cgmath::{{Matrix4, SquareMatrix{}}};
use std::collections::HashMap;
{}
{}
{}

const OFFSCREEN_RENDER_SIZE: u32 = 1024;", additional_util_imports, additional_cgmath_imports, byte_decls, vbo_index_decls, texture_index_decls);
    Ok(gen_content)
}
