
pub mod textbuffer;

use defs::{TexturePixelFormat, TextureCreationData};
use model::factory::{Model, StaticVertex};

use image::{
    DynamicImage,
    codecs::jpeg::JpegDecoder,
    codecs::png::PngDecoder
};

use std::io::Cursor;

pub enum TextureCodec {
    Jpeg,
    Png
}

pub fn decode_texture(image_file_bytes: &[u8], codec: TextureCodec) -> Result<TextureCreationData, String> {
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
        data: Some(data),
        width,
        height,
        format: TexturePixelFormat::RGBA
    })
}

pub fn decode_model(model_file_bytes: &[u8]) -> (Vec<StaticVertex>, usize) {
    let model = unsafe {
        Model::new_from_bytes(model_file_bytes).unwrap()
    };
    let vertex_count: usize = model.vertices.len();
    (model.vertices, vertex_count)
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
