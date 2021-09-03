
use defs::{
    Camera,
    control::Control,
    render::{SceneInfo, DrawingDescription, DrawingPass, DrawingStep, FramebufferTarget, Shader, VertexFormat, VboCreationData, ResourcePreloads}
};
use engine::{
    camera::null::NullCamera,
    util::textbuffer::{TextGenerator, TextAlignment}
};

use cgmath::{Matrix4, Vector4, SquareMatrix};
use std::collections::HashMap;

const VBO_INDEX_BG: usize = 0; // Re-used
const VBO_INDEX_HUD_SUB: usize = 2;

const TEXTURE_INDEX_BG: usize = 0; // Re-used
const TEXTURE_INDEX_FONT: usize = 1; // Re-used

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

    fn make_preloads(&self) -> ResourcePreloads {
        let hud_data = self.text_generator.generate_vertex_buffer(
            "Howzat mate!",
            -1.0,
            0.0,
            2.0,
            1.0,
            0.125,
            TextAlignment::Centre,
            TextAlignment::Centre);
        let hud_data_vertex_count = hud_data.len();

        let mut vbo_loads = HashMap::<usize, VboCreationData>::new();
        vbo_loads.insert(VBO_INDEX_HUD_SUB, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: hud_data,
            vertex_count: hud_data_vertex_count,
            draw_indexed: false,
            index_data: None
        });

        ResourcePreloads {
            vbo_preloads: vbo_loads,
            texture_preloads: HashMap::new()
        }
    }

    fn make_description(&self) -> DrawingDescription {

        DrawingDescription {
            passes: vec![
                DrawingPass {
                    target: FramebufferTarget::DefaultFramebuffer,
                    steps: vec![
                        DrawingStep {
                            shader: Shader::PlainPnt,
                            vbo_index: VBO_INDEX_BG,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_indices: vec![TEXTURE_INDEX_BG],
                            depth_test: true
                        },
                        DrawingStep {
                            shader: Shader::Text,
                            vbo_index: VBO_INDEX_HUD_SUB,
                            vbo_format: VertexFormat::PositionNormalTexture,
                            draw_indexed: false,
                            texture_indices: vec![TEXTURE_INDEX_FONT],
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

    fn update_camera(&mut self, _time_step_millis: u64, _controller: &dyn Control) -> Option<Box<dyn SceneInfo>> {
        None
    }

    unsafe fn get_ubo_data_ptr_and_size(&self, pass_index: usize, step_index: usize) -> (*const u8, usize) {
        match (pass_index, step_index) {
            (0, 0) => (&self.camera_ubo as *const CameraUbo as *const u8, std::mem::size_of::<CameraUbo>()),
            (0, 1) => (&self.text_paint_ubo as *const TextPaintUbo as *const u8, std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for SubMenuScene")
        }
    }
}
