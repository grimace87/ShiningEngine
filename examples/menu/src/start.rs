
use defs::{
    SceneInfo,
    SceneUpdates,
    Scene,
    control::Control,
    render::{
        DrawingDescription,
        DrawingPass,
        DrawingStep,
        FramebufferTarget,
        Shader,
        VertexFormat,
        VboCreationData,
        TextureCreationData,
        ResourcePreloads,
        ImageUsage
    },
    ubo::*
};
use engine::util::{
    TextureCodec,
    decode_texture,
    map_ui_rects,
    textbuffer::{
        TextGenerator,
        TextAlignment
    }
};
use cgmath::{
    Matrix4,
    Vector4,
    SquareMatrix
};
use std::collections::HashMap;

const MENU_TEXTURE_BYTES: &[u8] = include_bytes!("../../resources/textures/menu_texture.png");
const MUSICA_FONT_BYTES: &[u8] = include_bytes!("../../resources/textures/Musica.png");

const VBO_INDEX_BG: usize = 0;
const VBO_INDEX_HUD: usize = 1;

const TEXTURE_INDEX_BG: usize = 0;
const TEXTURE_INDEX_FONT: usize = 1;

pub struct StartMenuScene {
    text_generator: TextGenerator,
    camera_ubo: CameraUbo,
    text_paint_ubo: TextPaintUbo,
    frame_counter: usize
}

impl Default for StartMenuScene {
    fn default() -> StartMenuScene {
        StartMenuScene {
            text_generator: TextGenerator::from_resource(
                include_str!("../../resources/font/Musica.fnt")
            ),
            camera_ubo: CameraUbo {
                camera_matrix: Matrix4::identity()
            },
            text_paint_ubo: TextPaintUbo {
                camera_matrix: Matrix4::identity(),
                paint_color: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 }
            },
            frame_counter: 0
        }
    }
}

impl Scene for StartMenuScene {}

impl SceneInfo for StartMenuScene {

    fn make_preloads(&self) -> ResourcePreloads {

        let (menu_model_data, menu_vertex_count) = {
            let float_data = map_ui_rects(vec![
                [-1.0, -1.0, 1.0, -0.5, 0.0, 0.0, 1.0, 0.25],
                [-1.0, 0.5, 1.0, 1.0, 0.0, 0.25, 1.0, 0.0]
            ]);
            let vertex_count = float_data.len();
            (float_data, vertex_count)
        };

        let hud_data = self.text_generator.generate_vertex_buffer(
            "Ey, mate",
            (-1.0, -1.0),
            (2.0, 1.0),
            0.125,
            TextAlignment::Start,
            TextAlignment::Start);
        let hud_data_vertex_count = hud_data.len();

        let mut vbo_loads = HashMap::<usize, VboCreationData>::new();
        vbo_loads.insert(VBO_INDEX_BG, VboCreationData {
            vertex_format: VertexFormat::PositionNormalTexture,
            vertex_data: menu_model_data,
            vertex_count: menu_vertex_count,
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

        let menu_texture = decode_texture(
            MENU_TEXTURE_BYTES,
            TextureCodec::Png,
            ImageUsage::TextureSampleOnly)
            .unwrap();
        let font_texture = decode_texture(
            MUSICA_FONT_BYTES,
            TextureCodec::Png,
            ImageUsage::TextureSampleOnly)
            .unwrap();
        let mut texture_loads = HashMap::<usize, TextureCreationData>::new();
        texture_loads.insert(TEXTURE_INDEX_BG, menu_texture);
        texture_loads.insert(TEXTURE_INDEX_FONT, font_texture);

        ResourcePreloads {
            vbo_preloads: vbo_loads,
            texture_preloads: texture_loads
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
                            vbo_index: VBO_INDEX_HUD,
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

    unsafe fn get_ubo_data_ptr_and_size(
        &self,
        pass_index: usize,
        step_index: usize
    ) -> (*const u8, usize) {
        match (pass_index, step_index) {
            (0, 0) => (
                &self.camera_ubo as *const CameraUbo as *const u8,
                std::mem::size_of::<CameraUbo>()),
            (0, 1) => (
                &self.text_paint_ubo as *const TextPaintUbo as *const u8,
                std::mem::size_of::<TextPaintUbo>()),
            _ => panic!("Cannot get UBO for StartMenuScene")
        }
    }
}

impl SceneUpdates for StartMenuScene {

    fn update_aspect_ratio(&mut self, _aspect_ratio: f32) {}

    fn on_time_elapsed(
        &mut self,
        _time_step_millis: u64,
        _controller: &dyn Control
    ) -> Option<Box<dyn Scene>> {
        self.frame_counter += 1;
        if self.frame_counter == 60 {
            Some(Box::new(crate::submenu::SubMenuScene::new()))
        } else {
            None
        }
    }

    fn on_pre_render(&mut self) {
        let red: f32 = 1.0;
        self.text_paint_ubo.paint_color.x = red;
        self.text_paint_ubo.paint_color.z = 1.0 - red;
    }
}
