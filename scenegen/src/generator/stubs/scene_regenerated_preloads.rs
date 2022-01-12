
use crate::GeneratorError;
use crate::deserialiser::types::{*, scene::*};

fn get_codec_from_file_name(config: &Scene, texture: &Texture) -> Result<String, GeneratorError> {
    let file_as_lower = match &texture.file {
        Some(f) => f.to_lowercase(),
        None => return Err(GeneratorError::InvalidSpec(
            format!("Scene {} has texture {} with no type or file", config.id, texture.id)))
    };
    let extension_begin = match file_as_lower.find(".") {
        Some(i) => &file_as_lower[i..],
        None => return Err(GeneratorError::InvalidSpec(
            format!("Scene {} has file texture {} with no file extension", config.id, texture.id)))
    };
    let codec = match extension_begin {
        ".jpeg" | ".jpg" => String::from("TextureCodec::Jpeg"),
        ".png" => String::from("TextureCodec::Png"),
        _ => return Err(GeneratorError::InvalidSpec(
            format!("Scene {} has file texture {} with an unknown file extension", config.id, texture.id)))
    };
    Ok(codec)
}

pub fn generate_preloads(config: &Scene) -> Result<String, GeneratorError> {

    let mut model_load_operations = String::new();
    for model in config.resources.models.iter() {
        let load_op = match model.generator {
            None => format!(
                "        let ({}_vertex_data, {}_vertex_count) = engine::util::decode_model({}_MODEL_BYTES);",
                model.id, model.id, model.id.to_uppercase()
            ),
            Some(ModelGenerator::text) => format!(
        "        let {}_vertex_data = self.text_generator.generate_vertex_buffer(
            \"Hello!\",
            (-1.0, -1.0),
            (2.0, 1.0),
            0.125,
            TextAlignment::Start,
            TextAlignment::Start);
        let {}_vertex_count = {}_vertex_data.len();",
                model.id, model.id, model.id
            ),
            Some(ModelGenerator::skybox) => format!(
                "        let ({}_vertex_data, {}_vertex_count) = engine::util::make_skybox_vertices(20.0);",
                model.id, model.id
            )
        };
        let insert_op = format!("
        vbo_loads.insert(VBO_INDEX_{}, VboCreationData {{
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: {}_vertex_data,
            vertex_count: {}_vertex_count,
            draw_indexed: false,
            index_data: None
        }});\n", model.id.to_uppercase(), model.id, model.id);
        model_load_operations = format!("{}{}{}", model_load_operations, load_op, insert_op);
    }

    let mut texture_load_operations = String::new();
    for texture in config.resources.textures.iter() {
        match texture.kind {
            None => {
                let codec = get_codec_from_file_name(config, texture)?;
                let load_op = format!("
        let {}_texture = engine::util::decode_texture(
            {}_TEXTURE_BYTES,
            {},
            ImageUsage::TextureSampleOnly)
            .unwrap();", texture.id, texture.id.to_uppercase(), codec);
                let insert_op = format!(
                    "        texture_loads.insert(TEXTURE_INDEX_{}, {}_texture);",
                    texture.id.to_uppercase(), texture.id
                );
                texture_load_operations = format!("{}{}\n{}", texture_load_operations, load_op, insert_op);
            },
            Some(TextureKind::uninitialised) => {
                // TODO - Enforce or infer specific usage of texture at this point
                let content = format!("
        texture_loads.insert(TEXTURE_INDEX_{}, TextureCreationData {{
            layer_data: None,
            width: OFFSCREEN_RENDER_SIZE,
            height: OFFSCREEN_RENDER_SIZE,
            format: TexturePixelFormat::Rgba,
            usage: ImageUsage::OffscreenRenderSampleColorWriteDepth
        }});", texture.id.to_uppercase());
                texture_load_operations = format!("{}{}", texture_load_operations, content);
            },
            Some(TextureKind::cubemap) => {
                let codec = get_codec_from_file_name(config, texture)?;
                let load_op = format!("
        let {}_texture = engine::util::decode_texture_array(
            vec![
                {}_TEXTURE_LF_BYTES,
                {}_TEXTURE_RT_BYTES,
                {}_TEXTURE_DN_BYTES,
                {}_TEXTURE_UP_BYTES,
                {}_TEXTURE_FT_BYTES,
                {}_TEXTURE_BK_BYTES
            ],
            {},
            ImageUsage::Skybox)
            .unwrap();", texture.id, texture.id.to_uppercase(), texture.id.to_uppercase(), texture.id.to_uppercase(), texture.id.to_uppercase(), texture.id.to_uppercase(), texture.id.to_uppercase(), codec);
                let insert_op = format!(
                    "        texture_loads.insert(TEXTURE_INDEX_{}, {}_texture);",
                    texture.id.to_uppercase(), texture.id
                );
                texture_load_operations = format!("{}{}\n{}", texture_load_operations, load_op, insert_op);
            }
        }
    }

    let content = format!("\
    fn make_preloads(&self) -> ResourcePreloads {{
        let mut vbo_loads = HashMap::<usize, VboCreationData>::new();
        let mut texture_loads = HashMap::<usize, TextureCreationData>::new();

{}{}

        ResourcePreloads {{
            vbo_preloads: vbo_loads,
            texture_preloads: texture_loads
        }}
    }}\
    ", model_load_operations, texture_load_operations);
    Ok(content)
}
