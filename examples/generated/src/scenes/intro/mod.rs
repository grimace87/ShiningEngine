
use defs::{
    Camera,
    SceneUpdates,
    Scene,
    control::Control
};
use engine::camera::player::PlayerCamera;
use crate::scenes::forest::ForestScene;

include!("descriptions.gen.rs");

impl Scene for IntroScene {}

impl SceneUpdates for IntroScene {

    fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.update_aspect(aspect_ratio);
    }

    fn on_time_elapsed(
        &mut self,
        time_step_millis: u64,
        controller: &dyn Control
    ) -> Option<Box<dyn Scene>> {
        self.camera.update(time_step_millis, controller);
        let out_of_bounds = f32::abs(self.camera.get_x() - 10.0) > 3.0 || f32::abs(self.camera.get_y() + 3.0) > 3.0 || f32::abs(self.camera.get_z() + 15.0) > 3.0;
        match out_of_bounds {
            true => Some(Box::new(ForestScene::new())),
            false => None
        }
    }

    fn on_pre_render(&mut self) {
        let p_matrix = self.camera.get_projection_matrix();
        let v_matrix = self.camera.get_view_matrix();
        let pv_matrix = p_matrix * v_matrix;

        self.ubo_compose_skybox.matrix = pv_matrix;
        self.ubo_compose_text_overlay.camera_matrix = Matrix4::identity();
        self.ubo_compose_text_overlay.paint_color = Vector4 { x: 1.0, y: rand::random(), z: 0.0, w: 1.0 };
    }
}
