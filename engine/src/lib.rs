
pub mod camera;
pub mod control;
pub mod timer;
pub mod util;
pub mod scene;

use self::{
    camera::{
        Camera,
        null::NullCamera,
        player::PlayerCamera
    },
    control::{
        Control,
        KeyCode,
        InputState,
        null::NullControl,
        user::UserControl
    },
    timer::{
        Timer,
        null::NullTimer,
        global::GlobalTimer
    }
};
use defs::{RendererApi, PresentResult, DrawingDescription, SceneInfo, SceneManager};
use renderer::null::NullRenderer;

use cgmath::Matrix4;
use raw_window_handle::HasRawWindowHandle;
use std::marker::PhantomData;
use crate::scene::SceneHost;

pub struct Engine<R> where R : RendererApi {
    scene_host: SceneHost,
    phantom_renderer: PhantomData<R>,
    renderer: Box<dyn RendererApi>,
    camera: Box<dyn Camera>,
    controller: Box<dyn Control>,
    timer: Box<dyn Timer>,
    drawing_description: DrawingDescription,
}

impl<R: 'static> Engine<R> where R : RendererApi {

    pub fn new_uninitialised(scene_info: Box<dyn SceneInfo>) -> Engine<R> {
        Engine {
            scene_host: SceneHost::new(scene_info),
            phantom_renderer: PhantomData::default(),
            renderer: Box::new(NullRenderer::new()),
            camera: Box::new(NullCamera::new()),
            controller: Box::new(NullControl::new()),
            timer: Box::new(NullTimer::new()),
            drawing_description: DrawingDescription::default(),
        }
    }

    pub fn initialise(&mut self, window_owner: &dyn HasRawWindowHandle) {

        let description = self.scene_host.get_current().make_description();
        let renderer = R::new(window_owner, &description).unwrap();
        let aspect_ratio = renderer.get_aspect_ratio();

        self.renderer = Box::new(renderer);
        self.camera = Box::new(PlayerCamera::new(aspect_ratio));
        self.controller = Box::new(UserControl::new());
        self.timer = Box::new(GlobalTimer::new());
        self.drawing_description = description;
    }

    pub fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState) {
        self.controller.process_keyboard_event(keycode, state);
    }

    pub fn update_aspect(&mut self, aspect_ratio: f32) {
        self.camera.update_aspect(aspect_ratio);
    }

    pub fn pull_time_step_millis(&mut self) -> u64 {
        self.timer.pull_time_step_millis()
    }

    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        self.camera.get_matrix()
    }

    pub fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let aspect_ratio: f32;

        self.renderer.recreate_swapchain(window_owner, &self.drawing_description)?;
        aspect_ratio = self.renderer.get_aspect_ratio();

        self.update_aspect(aspect_ratio);
        Ok(())
    }

    pub fn update(&mut self, time_step_millis: u64) {

        self.controller.update();
        self.camera.update(time_step_millis, self.controller.as_ref());

        if let Some(new_scene) = self.scene_host.update_current(&self.get_camera_matrix()) {
            self.scene_host.queue_scene(new_scene);
        }

        if self.scene_host.drain_queue() {
            let description = self.scene_host.get_current().make_description();
            self.renderer.recreate_scene_resources(&description).unwrap();
        }
    }

    pub fn render(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let updated_aspect_ratio: f32;
        match self.renderer.draw_next_frame(self.scene_host.get_current()) {
            Ok(PresentResult::Ok) => return Ok(()),
            Ok(PresentResult::SwapchainOutOfDate) => {
                self.renderer.recreate_swapchain(window_owner, &self.drawing_description).unwrap();
                updated_aspect_ratio = self.renderer.get_aspect_ratio();
            },
            Err(e) => return Err(format!("{}", e))
        };

        self.update_aspect(updated_aspect_ratio);
        Ok(())
    }
}
