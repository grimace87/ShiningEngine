
use defs::{SceneInfo, DrawingDescription, DrawingPass, Shader, VertexFormat, PostStep};
use engine::util::{TextureCodec, decode_texture, decode_model};

use cgmath::{Matrix4, Vector4, SquareMatrix};

const MENU_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\MenuScene.mdl"));
const FACES_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\Grimace.mdl"));
const TERRAIN_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/simple_outdoor_texture.jpg");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");

#[repr(C)]
struct CameraUbo {
    camera_matrix: Matrix4<f32>
}

#[repr(C)]
struct TextPaintUbo {
    camera_matrix: Matrix4<f32>,
    paint_color: Vector4<f32>
}

pub struct MenuScene {
    camera_ubo: CameraUbo,
    text_paint_ubo: TextPaintUbo
}

impl MenuScene {
    pub fn new() -> MenuScene {
        MenuScene {
            camera_ubo: CameraUbo {
                camera_matrix: Matrix4::identity()
            },
            text_paint_ubo: TextPaintUbo {
                camera_matrix: Matrix4::identity(),
                paint_color: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 }
            }
        }
    }
}

impl SceneInfo for MenuScene {

    fn make_description(&self) -> DrawingDescription {

        let (scene_model_data, scene_vertex_count) = decode_model(MENU_MODEL_BYTES);
        let (face_model_data, face_vertex_count) = decode_model(FACES_MODEL_BYTES);

        let scene_texture = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg).unwrap();
        let font_texture = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png).unwrap();

        DrawingDescription {
            passes: vec![
                DrawingPass {
                    shader: Shader::PlainPnt,
                    vertex_format: VertexFormat::PositionNormalTexture,
                    vertex_data: scene_model_data,
                    vertex_count: scene_vertex_count,
                    draw_indexed: false,
                    index_data: None,
                    texture: scene_texture,
                    depth_test: true
                },
                DrawingPass {
                    shader: Shader::PlainPnt,
                    vertex_format: VertexFormat::PositionNormalTexture,
                    vertex_data: face_model_data,
                    vertex_count: face_vertex_count,
                    draw_indexed: false,
                    index_data: None,
                    texture: font_texture,
                    depth_test: true
                }
            ],
            post_step: PostStep::Nothing
        }
    }

    fn on_camera_updated(&mut self, matrix: &Matrix4<f32>) {
        self.camera_ubo.camera_matrix = matrix.clone();
        self.text_paint_ubo.camera_matrix = matrix.clone();
    }

    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize) -> (*const u8, usize) {
        match pass_index {
            0 => (&self.camera_ubo as *const CameraUbo as *const u8, std::mem::size_of::<CameraUbo>()),
            1 => (&self.text_paint_ubo as *const TextPaintUbo as *const u8, std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for MenuScene")
        }
    }
}
