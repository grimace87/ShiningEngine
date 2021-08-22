
pub mod textbuffer;

use defs::{TexturePixelFormat, TextureCreationData, ImageUsage};
use model::factory::{Model, StaticVertex};

use image::{
    DynamicImage,
    codecs::jpeg::JpegDecoder,
    codecs::png::PngDecoder
};

use std::io::Cursor;

#[derive(Copy, Clone)]
pub enum TextureCodec {
    Jpeg,
    Png
}

pub fn decode_texture(image_file_bytes: &[u8], codec: TextureCodec, usage: ImageUsage) -> Result<TextureCreationData, String> {
    let (data, width, height) = match codec {
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
    Ok(TextureCreationData {
        layer_data: Some(vec![data]),
        width,
        height,
        format: TexturePixelFormat::RGBA,
        usage
    })
}

pub fn decode_texture_array(image_file_bytes: Vec<&[u8]>, codec: TextureCodec, usage: ImageUsage) -> Result<TextureCreationData, String> {
    let decoded_textures: Vec<_> = image_file_bytes.iter()
        .map(|bytes| decode_texture(bytes, codec, usage).unwrap())
        .collect();
    let width = decoded_textures[0].width;
    let height = decoded_textures[0].width;
    if decoded_textures.iter().any(|t| t.width != width || t.height != height) {
        return Err(String::from("Not all textures same size in multi-layer sources"));
    }
    let layer_data: Vec<_> = decoded_textures.into_iter()
        .map(|d| d.layer_data.unwrap().first().unwrap().to_owned())
        .collect();
    Ok(TextureCreationData {
        layer_data: Some(layer_data),
        width,
        height,
        format: TexturePixelFormat::RGBA,
        usage
    })
}

pub fn decode_model(model_file_bytes: &[u8]) -> (Vec<StaticVertex>, usize) {
    let model = unsafe {
        Model::new_from_bytes(model_file_bytes).unwrap()
    };
    let vertex_count: usize = model.vertices.len();
    (model.vertices, vertex_count)
}

/// Make position-normal-texcoords for cube faces
pub fn make_skybox_vertices(size: f32) -> (Vec<StaticVertex>, usize) {
    let neg: f32 = -size;
    let pos: f32 = size;
    let vertices = vec![
        // Left (negative X)
        StaticVertex::from_components(neg, neg, neg, pos, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, neg, pos, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, pos, pos, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, pos, pos, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, pos, pos, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, neg, pos, 0.0, 0.0, 0.0, 0.0),
        // Right (positive X)
        StaticVertex::from_components(pos, neg, neg, neg, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, pos, neg, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, pos, neg, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, pos, neg, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, neg, neg, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, neg, neg, 0.0, 0.0, 0.0, 0.0),
        // Up (negative Y)
        StaticVertex::from_components(pos, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        // Down (positive Y)
        StaticVertex::from_components(pos, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        // Forward (negative Z)
        StaticVertex::from_components(pos, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, neg, 0.0, 0.0, 0.0, 0.0, 0.0),
        // Behind (positive Z)
        StaticVertex::from_components(pos, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, neg, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(neg, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0),
        StaticVertex::from_components(pos, pos, pos, 0.0, 0.0, 0.0, 0.0, 0.0)
    ];
    let vertex_count = vertices.len();
    (vertices, vertex_count)
}

/// Maps sets of floats into a vector of StaticVertex structs.
/// z value will always be zero, and normal pointing along -z.
/// Source floats are:
///   Position [left, top, right, bottom] then texture [left, top, right, bottom]
pub fn map_ui_rects(source: Vec<[f32; 8]>) -> Vec<StaticVertex> {
    let mut all_rects: Vec<StaticVertex> = vec![];
    for s in source {
        let set = [
            StaticVertex::from_components(s[0], s[1], 0.0, 0.0, -1.0, 0.0, s[4], s[5]),
            StaticVertex::from_components(s[0], s[3], 0.0, 0.0, -1.0, 0.0, s[4], s[7]),
            StaticVertex::from_components(s[2], s[3], 0.0, 0.0, -1.0, 0.0, s[6], s[7]),
            StaticVertex::from_components(s[2], s[3], 0.0, 0.0, -1.0, 0.0, s[6], s[7]),
            StaticVertex::from_components(s[2], s[1], 0.0, 0.0, -1.0, 0.0, s[6], s[5]),
            StaticVertex::from_components(s[0], s[1], 0.0, 0.0, -1.0, 0.0, s[4], s[5])
        ];
        all_rects.extend_from_slice(&set);
    }
    all_rects
}
