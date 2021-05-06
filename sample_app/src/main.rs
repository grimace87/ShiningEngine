
use defs::{SceneDescription, VertexFormat, TexturePixelFormat, PostStep};
use platform_windows::PlatformWindows;
use renderer::vk_renderer::VkRenderer;
use engine::Engine;
use model::factory::Model;

use image::{
    DynamicImage,
    codecs::jpeg::JpegDecoder,
    codecs::png::PngDecoder
};

use std::io::Cursor;

const APP_TITLE: &str = "Strength Beyond Fear";

const MENU_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\MenuScene.mdl"));
const FACES_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\Grimace.mdl"));
const TERRAIN_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/simple_outdoor_texture.jpg");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");

enum TextureCodec {
    Jpeg,
    Png
}

fn main() {

    let mut platform = PlatformWindows::new_window(APP_TITLE).unwrap();
    let engine: Engine<VkRenderer> = Engine::new_uninitialised();

    let (scene_model_data, scene_vertex_count) = {
        let scene_model = unsafe {
            Model::new_from_bytes(MENU_MODEL_BYTES).unwrap()
        };
        let vertex_count: usize = scene_model.vertices.len();
        (scene_model.vertices, vertex_count)
    };
    let (face_model_data, face_vertex_count) = {
        let faces_model = unsafe {
            Model::new_from_bytes(FACES_MODEL_BYTES).unwrap()
        };
        let vertex_count: usize = faces_model.vertices.len();
        (faces_model.vertices, vertex_count)
    };
    let (scene_texture_data, scene_texture_width, scene_texture_height) = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg).unwrap();
    let (font_texture_data, font_texture_width, font_texture_height) = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png).unwrap();
    let descriptions = vec![
        SceneDescription {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: scene_model_data,
            vertex_count: scene_vertex_count,
            draw_indexed: false,
            index_data: None,
            texture_format: TexturePixelFormat::RGBA,
            texture_data: scene_texture_data,
            texture_width: scene_texture_width,
            texture_height: scene_texture_height,
            depth_test: true,
            post_step: PostStep::Nothing
        },
        SceneDescription {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: face_model_data,
            vertex_count: face_vertex_count,
            draw_indexed: false,
            index_data: None,
            texture_format: TexturePixelFormat::RGBA,
            texture_data: font_texture_data,
            texture_width: font_texture_width,
            texture_height: font_texture_height,
            depth_test: true,
            post_step: PostStep::Nothing
        }
    ];
    if let Err(e) = platform.run(engine, descriptions) {
        println!("{}", e);
    }
}

fn decode_texture(image_file_bytes: &[u8], codec: TextureCodec) -> Result<(Vec<u8>, u32, u32), String> {
    let data_with_dimensions = match codec {
        TextureCodec::Jpeg => {
            let src_cursor = Cursor::new(image_file_bytes.to_vec());
            let decoder = JpegDecoder::new(src_cursor).unwrap();
            let terrain_image_pixel_data = DynamicImage::from_decoder(decoder)
                .map_err(|e| format!("Error opening decoding an image: {:?}", e))?;
            let image_data_rgba = terrain_image_pixel_data.to_rgba8();
            (image_data_rgba.to_vec(), image_data_rgba.width(), image_data_rgba.height())
        },
        TextureCodec::Png => {
            let src_cursor = Cursor::new(image_file_bytes.to_vec());
            let decoder = PngDecoder::new(src_cursor).unwrap();
            let terrain_image_pixel_data = DynamicImage::from_decoder(decoder)
                .map_err(|e| format!("Error opening decoding an image: {:?}", e))?;
            let image_data_rgba = terrain_image_pixel_data.to_rgba8();
            (image_data_rgba.to_vec(), image_data_rgba.width(), image_data_rgba.height())
        }
    };
    Ok(data_with_dimensions)
}
