
use crate::GeneratorError;
use crate::deserialiser::types::scene::*;
use heck::CamelCase;

pub fn generate_struct_definition(
    config: &Scene,
    resources_dir_name: &'static str
) -> Result<String, GeneratorError> {
    let struct_name = format!("{}Scene", config.id.to_camel_case());

    let (camera_type, camera_constructor) = match config.camera {
        Camera::null => (
            "NullCamera",
            "NullCamera::default()"
        ),
        Camera::player => (
            "PlayerCamera",
            "PlayerCamera::new(1.0, 10.0, -3.0, -15.0, std::f32::consts::FRAC_PI_6 * 5.0)"
        ),
        Camera::flight_path => unimplemented!()
    };

    let (text_gen_decls, text_gen_constructors) = match config.resources.fonts.len() {
        0 => (
            String::new(),
            String::new()
        ),
        1 => (
            String::from("\n    text_generator: TextGenerator,"),
            format!(
                "\n            text_generator: TextGenerator::from_resource(include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/fonts/{}\"))),",
                resources_dir_name, config.resources.fonts[0].file
            )
        ),
        _ => {
            let mut decls = String::new();
            let mut constructors = String::new();
            for (i, _) in config.resources.fonts.iter().enumerate() {
                decls = format!("{}\n    text_generator_{}: TextGenerator,", decls, i);
                constructors = format!(
                    "{}\n            text_generator_{}: TextGenerator::from_resource(include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/fonts/{}\"))),",
                    constructors,
                    i,
                    resources_dir_name, config.resources.fonts[i].file
                );
            }
            (decls, constructors)
        }
    };

    let (ubo_decls, ubo_constructors) = {
        let mut decls = String::new();
        let mut constructors = String::new();
        for pass in config.passes.iter() {
            for step in pass.steps.iter() {

                let ubo_type = match step.render {
                    RenderFunction::basic_textured => "MvpUbo",
                    RenderFunction::basic_textured_clipped_y => "MvpClippingUbo",
                    RenderFunction::text_paint => "TextPaintUbo",
                    RenderFunction::cube => "MvpUbo",
                    RenderFunction::cube_clipped_y => "MvpClippingUbo",
                    RenderFunction::reflection_pre_render => "MvpClippingUbo"
                };
                decls = format!("{}\n    ubo_{}_{}: {},", decls, pass.name, step.name, ubo_type);

                let ubo_constructor = match step.render {
                    RenderFunction::basic_textured => "MvpUbo { matrix: Matrix4::identity() }",
                    RenderFunction::basic_textured_clipped_y => "MvpClippingUbo {\n                matrix: Matrix4::identity(),\n                y_bias: 0.0,\n                y_plane_normal: -1.0,\n                unused: [0.0, 0.0]\n            }",
                    RenderFunction::text_paint => "TextPaintUbo {\n                camera_matrix: Matrix4::identity(),\n                paint_color: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 1.0 }\n            }",
                    RenderFunction::cube => "MvpUbo { matrix: Matrix4::identity() }",
                    RenderFunction::cube_clipped_y => "MvpClippingUbo {\n                matrix: Matrix4::identity(),\n                y_bias: 0.0,\n                y_plane_normal: -1.0,\n                unused: [0.0, 0.0]\n            }",
                    RenderFunction::reflection_pre_render => "MvpClippingUbo {\n                matrix: Matrix4::identity(),\n                y_bias: 0.0,\n                y_plane_normal: -1.0,\n                unused: [0.0, 0.0]\n            }"
                };
                constructors = format!("{}\n            ubo_{}_{}: {},", constructors, pass.name, step.name, ubo_constructor);
            }
        }
        (decls, constructors)
    };

    let content = format!("
pub struct {} {{
    camera: {},{}{}
}}

impl {} {{
    pub fn new() -> {} {{
        {} {{
            camera: {},{}{}
        }}
    }}
}}
",
                      struct_name,
                      camera_type,
                      text_gen_decls,
                      ubo_decls,
                      struct_name,
                      struct_name,
                      struct_name,
                      camera_constructor,
                      text_gen_constructors,
                      ubo_constructors);
    Ok(content)
}

pub fn generate_get_ubo_fn(config: &Scene) -> Result<String, GeneratorError> {
    let struct_name = format!("{}Scene", config.id.to_camel_case());

    let ubo_ptr_mappings = {
        let mut ptr_mappings = String::new();
        for (pass_index, pass) in config.passes.iter().enumerate() {
            for (step_index, step) in pass.steps.iter().enumerate() {

                let ubo_type = match step.render {
                    RenderFunction::basic_textured => "MvpUbo",
                    RenderFunction::basic_textured_clipped_y => "MvpClippingUbo",
                    RenderFunction::text_paint => "TextPaintUbo",
                    RenderFunction::cube => "MvpUbo",
                    RenderFunction::cube_clipped_y => "MvpClippingUbo",
                    RenderFunction::reflection_pre_render => "MvpClippingUbo"
                };

                ptr_mappings = format!("{}\n            ({}, {}) => (\n                &self.ubo_{}_{} as *const {} as *const u8,\n                std::mem::size_of::<{}>()),", ptr_mappings, pass_index, step_index, pass.name, step.name, ubo_type, ubo_type);
            }
        }
        ptr_mappings
    };

    let content = format!("\
    unsafe fn get_ubo_data_ptr_and_size(
        &self,
        pass_index: usize,
        step_index: usize
    ) -> (*const u8, usize) {{
        match (pass_index, step_index) {{{}
            _ => panic!(\"Cannot get UBO for {}\")
        }}
    }}", ubo_ptr_mappings, struct_name);
    Ok(content)
}
