
use defs::{Camera, SceneInfo, DrawingDescription, DrawingPass, Shader, VertexFormat, PostStep, Control};
use engine::{
    camera::null::NullCamera,
    util::{
        TextureCodec,
        decode_texture,
        map_ui_rects,
        textbuffer::{TextGenerator, TextAlignment}
    }
};

use cgmath::{Matrix4, Vector4, SquareMatrix};

const MENU_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/menu_texture.png");
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

pub struct SubMenuScene {
    camera: NullCamera,
    text_generator: TextGenerator,
    camera_ubo: CameraUbo,
    text_paint_ubo: TextPaintUbo
}

impl SubMenuScene {
    pub fn new() -> SubMenuScene {
        SubMenuScene {
            camera: NullCamera::new(),
            text_generator: TextGenerator::from_resource(
                include_str!("../../resources/font/Musica.fnt")
            ),
            camera_ubo: CameraUbo {
                camera_matrix: Matrix4::identity()
            },
            text_paint_ubo: TextPaintUbo {
                camera_matrix: Matrix4::identity(),
                paint_color: Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 1.0 }
            }
        }
    }
}

impl SceneInfo for SubMenuScene {

    fn make_description(&self) -> DrawingDescription {

        let (menu_model_data, menu_vertex_count) = {
            let float_data = map_ui_rects(vec![
                [-1.0, -1.0, 1.0, -0.5, 0.0, 0.0, 1.0, 0.25],
                [-1.0, 0.5, 1.0, 1.0, 0.0, 0.25, 1.0, 0.0]
            ]);
            let vertex_count = float_data.len();
            (float_data, vertex_count)
        };

        let menu_texture = decode_texture(MENU_TEXTURE_BYTES, TextureCodec::Png).unwrap();
        let font_texture = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png).unwrap();

        let hud_data = self.text_generator.generate_vertex_buffer(
            "Howzat mate!",
            -1.0,
            0.0,
            2.0,
            1.0,
            0.125,
            TextAlignment::Centre,
            TextAlignment::Centre);
        let hud_data_size = hud_data.len();

        DrawingDescription {
            passes: vec![
                DrawingPass {
                    shader: Shader::PlainPnt,
                    vertex_format: VertexFormat::PositionNormalTexture,
                    vertex_data: menu_model_data,
                    vertex_count: menu_vertex_count,
                    draw_indexed: false,
                    index_data: None,
                    texture: menu_texture,
                    depth_test: true
                },
                DrawingPass {
                    shader: Shader::Text,
                    vertex_format: VertexFormat::PositionNormalTexture,
                    vertex_data: hud_data,
                    vertex_count: hud_data_size,
                    draw_indexed: false,
                    index_data: None,
                    texture: font_texture,
                    depth_test: true
                }
            ],
            post_step: PostStep::Nothing
        }
    }

    fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.update_aspect(aspect_ratio);
    }

    fn update_camera(&mut self, _time_step_millis: u64, _controller: &dyn Control) -> Option<Box<dyn SceneInfo>> {
        None
    }

    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize) -> (*const u8, usize) {
        match pass_index {
            0 => (&self.camera_ubo as *const CameraUbo as *const u8, std::mem::size_of::<CameraUbo>()),
            1 => (&self.text_paint_ubo as *const TextPaintUbo as *const u8, std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for SubMenuScene")
        }
    }
}
