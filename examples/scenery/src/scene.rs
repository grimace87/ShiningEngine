
use defs::{Camera, SceneInfo, DrawingDescription, DrawingPass, Shader, VertexFormat, PostStep, Control};
use engine::{
    camera::player::PlayerCamera,
    util::{
        TextureCodec,
        decode_texture,
        decode_model,
        textbuffer::{TextGenerator, TextAlignment}
    }
};

use cgmath::{Matrix4, Vector4, SquareMatrix, Vector3};

const MENU_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\SceneTerrain.mdl"));
const RIVER_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\River.mdl"));
const FACES_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\models\\Grimace.mdl"));
const TERRAIN_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/simple_outdoor_texture.jpg");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");

#[repr(C)]
struct MvpUbo {
    matrix: Matrix4<f32>
}

#[repr(C)]
struct TextPaintUbo {
    camera_matrix: Matrix4<f32>,
    paint_color: Vector4<f32>
}

pub struct SceneryScene {
    camera: PlayerCamera,
    text_generator: TextGenerator,
    terrain_pass_ubo: MvpUbo,
    river_pass_ubo: MvpUbo,
    text_paint_ubo: TextPaintUbo,
    river_phase: f64
}

impl SceneryScene {
    pub fn new() -> SceneryScene {
        SceneryScene {
            camera: PlayerCamera::new(1.0),
            text_generator: TextGenerator::from_resource(
                include_str!("../../resources/font/Musica.fnt")
            ),
            terrain_pass_ubo: MvpUbo {
                matrix: Matrix4::identity()
            },
            river_pass_ubo: MvpUbo {
                matrix: Matrix4::identity()
            },
            text_paint_ubo: TextPaintUbo {
                camera_matrix: Matrix4::identity(),
                paint_color: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 }
            },
            river_phase: 0.0
        }
    }
}

impl SceneInfo for SceneryScene {

    fn make_description(&self) -> DrawingDescription {

        let (scene_model_data, scene_vertex_count) = decode_model(MENU_MODEL_BYTES);
        let (river_model_data, river_vertex_count) = decode_model(RIVER_MODEL_BYTES);

        // TODO - Something?
        let (_face_model_data, _face_vertex_count) = decode_model(FACES_MODEL_BYTES);

        // TODO - Share textures between passes
        let scene_texture = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg).unwrap();
        let scene_texture_again = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg).unwrap();
        let font_texture = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png).unwrap();

        let hud_data = self.text_generator.generate_vertex_buffer(
            "Ey, mate",
            -1.0,
            -1.0,
            2.0,
            1.0,
            0.125,
            TextAlignment::Start,
            TextAlignment::Start);
        let hud_data_size = hud_data.len();

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
                    vertex_data: river_model_data,
                    vertex_count: river_vertex_count,
                    draw_indexed: false,
                    index_data: None,
                    texture: scene_texture_again,
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

    fn update_camera(&mut self, time_step_millis: u64, controller: &dyn Control) -> Option<Box<dyn SceneInfo>> {
        self.camera.update(time_step_millis, controller);
        let matrix = self.camera.get_matrix();

        self.river_phase += (time_step_millis as f64) * 0.001 * std::f64::consts::PI;
        if self.river_phase > std::f64::consts::TAU {
            self.river_phase -= std::f64::consts::TAU;
        }
        let deviation = self.river_phase.sin() as f32 * 0.01;
        let river_translation = Matrix4::<f32>::from_translation(Vector3 { x: 0.0, y: deviation, z: 0.0 });
        self.river_pass_ubo.matrix = river_translation * matrix;

        let red = 0.5 + 0.5 * matrix.x.x;
        self.terrain_pass_ubo.matrix = matrix.clone();
        self.text_paint_ubo.paint_color.x = red;
        self.text_paint_ubo.paint_color.z = 1.0 - red;
        None
    }

    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize) -> (*const u8, usize) {
        match pass_index {
            0 => (&self.terrain_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            1 => (&self.river_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            2 => (&self.text_paint_ubo as *const TextPaintUbo as *const u8, std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for SceneryScene")
        }
    }
}
