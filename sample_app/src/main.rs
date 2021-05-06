
use defs::{SceneDescription, VertexFormat, PostStep};
use platform_windows::PlatformWindows;
use renderer::vk_renderer::VkRenderer;
use engine::{
    Engine,
    util::{TextureCodec, decode_texture}
};
use model::factory::Model;

const APP_TITLE: &str = "Strength Beyond Fear";

const MENU_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\MenuScene.mdl"));
const FACES_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\Grimace.mdl"));
const TERRAIN_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/simple_outdoor_texture.jpg");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");

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
    let scene_texture = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg).unwrap();
    let font_texture = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png).unwrap();
    let descriptions = vec![
        SceneDescription {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: scene_model_data,
            vertex_count: scene_vertex_count,
            draw_indexed: false,
            index_data: None,
            texture: scene_texture,
            depth_test: true,
            post_step: PostStep::Nothing
        },
        SceneDescription {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: face_model_data,
            vertex_count: face_vertex_count,
            draw_indexed: false,
            index_data: None,
            texture: font_texture,
            depth_test: true,
            post_step: PostStep::Nothing
        }
    ];
    if let Err(e) = platform.run(engine, descriptions) {
        println!("{}", e);
    }
}
