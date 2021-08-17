
use defs::{Camera, SceneInfo, DrawingDescription, DrawingPass, DrawingStep, Shader, VertexFormat, Control, FramebufferTarget, ResourcePreloads, VboCreationData, TextureCreationData, FramebufferCreationData, TexturePixelFormat, ImageUsage};
use engine::{
    camera::player::PlayerCamera,
    util::{
        TextureCodec,
        decode_texture,
        decode_texture_array,
        make_skybox_vertices,
        decode_model,
        textbuffer::{TextGenerator, TextAlignment}
    }
};

use cgmath::{Matrix4, Vector4, SquareMatrix, Vector3};
use std::collections::HashMap;

const MENU_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/models/SceneTerrain.mdl"));
const RIVER_MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/models/River.mdl"));
const TERRAIN_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/simple_outdoor_texture.jpg");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");
const SKYBOX_LEFT_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_lf.jpg");
const SKYBOX_RIGHT_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_rt.jpg");
const SKYBOX_UP_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_up.jpg");
const SKYBOX_DOWN_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_dn.jpg");
const SKYBOX_FRONT_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_ft.jpg");
const SKYBOX_BACK_BYTES: &[u8] = include_bytes!("../../resources/textures/bluecloud_bk.jpg");

const VBO_INDEX_SCENE: usize = 0;
const VBO_INDEX_RIVER: usize = 1;
const VBO_INDEX_HUD: usize = 2;
const VBO_INDEX_SKYBOX: usize = 3;

const TEXTURE_INDEX_TERRAIN: usize = 0;
const TEXTURE_INDEX_FONT: usize = 1;
const TEXTURE_INDEX_PRE_RENDER_COLOR: usize = 2;
const TEXTURE_INDEX_PRE_RENDER_DEPTH: usize = 3;
const TEXTURE_INDEX_SKYBOX: usize = 4;

const OFFSCREEN_RENDER_SIZE: u32 = 1024;

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
    skybox_pass_ubo: MvpUbo,
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
            skybox_pass_ubo: MvpUbo {
                matrix: Matrix4::identity()
            },
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

    fn make_preloads(&self) -> ResourcePreloads {

        let (scene_model_data, scene_vertex_count) = decode_model(MENU_MODEL_BYTES);
        let (river_model_data, river_vertex_count) = decode_model(RIVER_MODEL_BYTES);
        let (cube_vertex_data, cube_vertex_count) = make_skybox_vertices(20.0);

        let hud_data = self.text_generator.generate_vertex_buffer(
            "Ey, mate",
            -1.0,
            -1.0,
            2.0,
            1.0,
            0.125,
            TextAlignment::Start,
            TextAlignment::Start);
        let hud_data_vertex_count = hud_data.len();

        let mut vbo_loads = HashMap::<usize, VboCreationData>::new();
        vbo_loads.insert(VBO_INDEX_SCENE, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: scene_model_data,
            vertex_count: scene_vertex_count,
            draw_indexed: false,
            index_data: None
        });
        vbo_loads.insert(VBO_INDEX_RIVER, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: river_model_data,
            vertex_count: river_vertex_count,
            draw_indexed: false,
            index_data: None
        });
        vbo_loads.insert(VBO_INDEX_SKYBOX, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: cube_vertex_data,
            vertex_count: cube_vertex_count,
            draw_indexed: false,
            index_data: None
        });
        vbo_loads.insert(VBO_INDEX_HUD, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: hud_data,
            vertex_count: hud_data_vertex_count,
            draw_indexed: false,
            index_data: None
        });

        let scene_texture = decode_texture(TERRAIN_TEXTURE_BYTES, TextureCodec::Jpeg, ImageUsage::TextureSampleOnly).unwrap();
        let font_texture = decode_texture(MUSICA_FONT_BYTES, TextureCodec::Png, ImageUsage::TextureSampleOnly).unwrap();
        let skybox_texture = decode_texture_array(vec![
            SKYBOX_LEFT_BYTES, SKYBOX_RIGHT_BYTES, SKYBOX_DOWN_BYTES, SKYBOX_UP_BYTES, SKYBOX_FRONT_BYTES, SKYBOX_BACK_BYTES
        ], TextureCodec::Jpeg, ImageUsage::Skybox).unwrap();
        let mut texture_loads = HashMap::<usize, TextureCreationData>::new();
        texture_loads.insert(TEXTURE_INDEX_TERRAIN, scene_texture);
        texture_loads.insert(TEXTURE_INDEX_FONT, font_texture);
        texture_loads.insert(TEXTURE_INDEX_SKYBOX, skybox_texture);
        texture_loads.insert(TEXTURE_INDEX_PRE_RENDER_COLOR, TextureCreationData {
            layer_data: None,
            width: OFFSCREEN_RENDER_SIZE,
            height: OFFSCREEN_RENDER_SIZE,
            format: TexturePixelFormat::RGBA,
            usage: ImageUsage::OffscreenRenderSampleColorWriteDepth
        });
        texture_loads.insert(TEXTURE_INDEX_PRE_RENDER_DEPTH, TextureCreationData {
            layer_data: None,
            width: OFFSCREEN_RENDER_SIZE,
            height: OFFSCREEN_RENDER_SIZE,
            format: TexturePixelFormat::Unorm16,
            usage: ImageUsage::OffscreenRenderSampleColorWriteDepth
        });

        ResourcePreloads {
            vbo_preloads: vbo_loads,
            texture_preloads: texture_loads
        }
    }

    fn make_description(&self) -> DrawingDescription {
        DrawingDescription {
            passes: vec![
                DrawingPass {
                    target: FramebufferTarget::Texture(FramebufferCreationData {
                        color_texture_index: TEXTURE_INDEX_PRE_RENDER_COLOR,
                        depth_texture_index: Some(TEXTURE_INDEX_PRE_RENDER_DEPTH),
                        width: OFFSCREEN_RENDER_SIZE as usize,
                        height: OFFSCREEN_RENDER_SIZE as usize,
                        color_format: TexturePixelFormat::RGBA,
                        depth_format: TexturePixelFormat::Unorm16
                    }),
                    steps: vec![
                        DrawingStep {
                            shader: Shader::Cube,
                            vbo_index: VBO_INDEX_SKYBOX,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_index: TEXTURE_INDEX_SKYBOX,
                            depth_test: false
                        },
                        DrawingStep {
                            shader: Shader::PlainPnt,
                            vbo_index: VBO_INDEX_SCENE,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_index: TEXTURE_INDEX_TERRAIN,
                            depth_test: true
                        }
                    ]
                },
                DrawingPass {
                    target: FramebufferTarget::DefaultFramebuffer,
                    steps: vec![
                        DrawingStep {
                            shader: Shader::Cube,
                            vbo_index: VBO_INDEX_SKYBOX,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_index: TEXTURE_INDEX_SKYBOX,
                            depth_test: false
                        },
                        DrawingStep {
                            shader: Shader::PlainPnt,
                            vbo_index: VBO_INDEX_SCENE,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_index: TEXTURE_INDEX_TERRAIN,
                            depth_test: true
                        },
                        DrawingStep {
                            shader: Shader::PlainPnt,
                            vbo_index: VBO_INDEX_RIVER,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,

                            // TODO - One of these per swapchain image
                            texture_index: TEXTURE_INDEX_PRE_RENDER_COLOR,

                            depth_test: true
                        },
                        DrawingStep {
                            shader: Shader::Text,
                            vbo_index: VBO_INDEX_HUD,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_index: TEXTURE_INDEX_FONT,
                            depth_test: true
                        }
                    ]
                }
            ]
        }
    }

    fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.update_aspect(aspect_ratio);
    }

    fn update_camera(&mut self, time_step_millis: u64, controller: &dyn Control) -> Option<Box<dyn SceneInfo>> {
        self.camera.update(time_step_millis, controller);
        let p_matrix = self.camera.get_projection_matrix();
        let mut v_matrix = self.camera.get_view_matrix();
        let pv_matrix = p_matrix * v_matrix;

        self.river_phase += (time_step_millis as f64) * 0.001 * std::f64::consts::PI;
        if self.river_phase > std::f64::consts::TAU {
            self.river_phase -= std::f64::consts::TAU;
        }
        let deviation = self.river_phase.sin() as f32 * 0.01;
        let river_translation = Matrix4::<f32>::from_translation(Vector3 { x: 0.0, y: deviation, z: 0.0 });
        self.river_pass_ubo.matrix = river_translation * pv_matrix;

        let red = 0.5 + 0.5 * pv_matrix.x.x;
        self.terrain_pass_ubo.matrix = pv_matrix.clone();
        self.text_paint_ubo.paint_color.x = red;
        self.text_paint_ubo.paint_color.z = 1.0 - red;

        v_matrix.w.x = 0.0;
        v_matrix.w.y = 0.0;
        v_matrix.w.z = 0.0;
        self.skybox_pass_ubo.matrix = p_matrix * v_matrix;

        None
    }

    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize, step_index: usize) -> (*const u8, usize) {
        match (pass_index, step_index) {
            (0, 0) => (&self.skybox_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            (0, 1) => (&self.terrain_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            (1, 0) => (&self.skybox_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            (1, 1) => (&self.terrain_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            (1, 2) => (&self.river_pass_ubo as *const MvpUbo as *const u8, std::mem::size_of::<MvpUbo>()),
            (1, 3) => (&self.text_paint_ubo as *const TextPaintUbo as *const u8, std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for SceneryScene")
        }
    }
}
