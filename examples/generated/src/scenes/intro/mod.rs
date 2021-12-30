
use defs::{
    Camera,
    SceneUpdates,
    Scene,
    control::Control
};
use engine::camera::player::PlayerCamera;

include!("descriptions.gen.rs");

impl Scene for IntroScene {}

impl SceneUpdates for IntroScene {

    fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.update_aspect(aspect_ratio);
    }

    fn update_camera(
        &mut self,
        time_step_millis: u64,
        controller: &dyn Control
    ) -> Option<Box<dyn Scene>> {
        self.camera.update(time_step_millis, controller);
        None
    }
}
