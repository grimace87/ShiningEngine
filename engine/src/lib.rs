
pub mod camera;
pub mod control;
pub mod timer;
pub mod util;

use self::{
    camera::{
        Camera,
        player::PlayerCamera
    },
    control::{
        Control,
        KeyCode,
        InputState,
        user::UserControl
    },
    timer::global::GlobalTimer
};
use defs::{RendererApi, PresentResult, DrawingDescription, SceneInfo, SceneManager};

use cgmath::{SquareMatrix, Matrix4};
use raw_window_handle::HasRawWindowHandle;
use lockfree::queue::Queue;
use std::mem::MaybeUninit;

pub struct Engine<R> where R : RendererApi {
    scene_info: Box<dyn SceneInfo>,
    renderer: Option<R>,
    camera: Option<PlayerCamera>,
    controller: Option<UserControl>,
    timer: Option<GlobalTimer>,
    drawing_description: DrawingDescription,
    scene_queue: Queue<MaybeUninit<Box<dyn SceneInfo>>>
}

impl<R> Engine<R> where R : RendererApi {

    pub fn new_uninitialised(scene_info: Box<dyn SceneInfo>) -> Engine<R> {
        Engine {
            scene_info: scene_info,
            renderer: None,
            camera: None,
            controller: None,
            timer: None,
            drawing_description: DrawingDescription::default(),
            scene_queue: Queue::new()
        }
    }

    pub fn initialise(&mut self, window_owner: &dyn HasRawWindowHandle) {

        let description = (*self.scene_info).make_description();
        let renderer = R::new(window_owner, &description).unwrap();
        let aspect_ratio = renderer.get_aspect_ratio();

        self.renderer = Some(renderer);
        self.camera = Some(camera::new_camera(aspect_ratio));
        self.controller = Some(control::new_control());
        self.timer = Some(GlobalTimer::new());
        self.drawing_description = description;
    }

    pub fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState) {
        match &mut self.controller {
            Some(c) => c.process_keyboard_event(keycode, state),
            _ => {}
        };
    }

    pub fn update_aspect(&mut self, aspect_ratio: f32) {
        match &mut self.camera {
            Some(c) => c.update_aspect(aspect_ratio),
            _ => {}
        };
    }

    pub fn pull_time_step_millis(&mut self) -> u64 {
        match &mut self.timer {
            Some(t) => t.pull_time_step_millis(),
            _ => 0
        }
    }

    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        match &self.camera {
            Some(c) => c.get_matrix(),
            _ => Matrix4::identity()
        }
    }

    pub fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let aspect_ratio: f32;

        if let Some(renderer) = &mut self.renderer {
            renderer.recreate_swapchain(window_owner, &self.drawing_description)?;
            aspect_ratio = renderer.get_aspect_ratio();
        } else {
            return Err(String::from("Recreating swapchain without a renderer set"));
        }

        self.update_aspect(aspect_ratio);
        Ok(())
    }

    pub fn update_before_render(&mut self, time_step_millis: u64) {
        if let Some(controller) = &mut self.controller {
            controller.update();
            match &mut self.camera {
                Some(camera) => camera.advance(time_step_millis, controller),
                _ => {}
            };
        }

        if let Some(new_scene) = (*self.scene_info).on_camera_updated(&self.get_camera_matrix()) {
            self.scene_queue.push(MaybeUninit::new(new_scene));
        }

        while let Some(new_scene) = self.scene_queue.next() {
            if let Some(renderer) = &mut self.renderer {
                let description = unsafe { new_scene.assume_init().as_ref().make_description() };
                renderer.recreate_scene_resources(&description).unwrap();
            }
        }
    }

    pub fn render_frame(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let updated_aspect_ratio: f32;
        if let Some(renderer) = &mut self.renderer {
            match renderer.draw_next_frame(self.scene_info.as_ref()) {
                Ok(PresentResult::Ok) => return Ok(()),
                Ok(PresentResult::SwapchainOutOfDate) => {
                    renderer.recreate_swapchain(window_owner, &self.drawing_description).unwrap();
                    updated_aspect_ratio = renderer.get_aspect_ratio();
                },
                Err(e) => return Err(format!("{}", e))
            };
        } else {
            return Err(String::from("Drawing frame without a renderer set"))
        }

        self.update_aspect(updated_aspect_ratio);
        Ok(())
    }
}

impl<R> SceneManager for Engine<R> where R: RendererApi {
    fn queue_scene(&self, new_scene: Box<dyn SceneInfo>) {
        self.scene_queue.push(MaybeUninit::new(new_scene));
    }
}
