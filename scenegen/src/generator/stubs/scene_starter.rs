
use crate::GeneratorError;
use crate::deserialiser::types::scene::*;
use heck::CamelCase;

pub fn generate_starter_scene_contents(config: &Scene) -> Result<String, GeneratorError> {
    let struct_name = format!("{}Scene", config.id.to_camel_case());
    let content = format!("
use defs::{{
    Camera,
    SceneUpdates,
    Scene,
    control::Control
}};
use engine::camera::player::PlayerCamera;

include!(\"descriptions.gen.rs\");

impl Scene for {} {{}}

impl SceneUpdates for {} {{

    fn update_aspect_ratio(&mut self, aspect_ratio: f32) {{
        // self.camera.update_aspect(aspect_ratio);
    }}

    fn on_time_elapsed(
        &mut self,
        time_step_millis: u64,
        controller: &dyn Control
    ) -> Option<Box<dyn Scene>> {{
        None
    }}

    fn on_pre_render(&mut self) {{
        // self.some_ubo.matrix = self.camera.get_projection_matrix();
    }}
}}
", struct_name, struct_name);
    Ok(content)
}
