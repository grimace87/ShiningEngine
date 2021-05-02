
pub mod camera;
pub mod control;
pub mod timer;

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
use defs::{RendererApi, PresentResult};

use cgmath::{SquareMatrix, Matrix4};
use raw_window_handle::HasRawWindowHandle;

pub struct Engine<R> where R : RendererApi {
    renderer: Option<R>,
    camera: Option<PlayerCamera>,
    controller: Option<UserControl>,
    timer: Option<GlobalTimer>
}

impl<R> Engine<R> where R : RendererApi {

    pub fn new_uninitialised() -> Engine<R> {
        Engine {
            renderer: None,
            camera: None,
            controller: None,
            timer: None
        }
    }

    pub fn initialise(&mut self, window_owner: &dyn HasRawWindowHandle) {

        let renderer = R::new(window_owner).unwrap();
        let aspect_ratio = renderer.get_aspect_ratio();

        self.renderer = Some(renderer);
        self.camera = Some(camera::new_camera(aspect_ratio));
        self.controller = Some(control::new_control());
        self.timer = Some(GlobalTimer::new());
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

    pub fn update(&mut self, time_step_millis: u64) {
        if let Some(controller) = &mut self.controller {
            controller.update();
            match &mut self.camera {
                Some(camera) => camera.advance(time_step_millis, controller),
                _ => {}
            };
        }
    }

    pub fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let aspect_ratio: f32;

        if let Some(renderer) = &mut self.renderer {
            renderer.recreate_swapchain(window_owner)?;
            aspect_ratio = renderer.get_aspect_ratio();
        } else {
            return Err(String::from("Recreating swapchain without a renderer set"));
        }

        self.update_aspect(aspect_ratio);
        Ok(())
    }

    pub fn draw_next_frame(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {

        let camera_matrix = self.get_camera_matrix();
        let aspect_ratio: f32;

        if let Some(renderer) = &mut self.renderer {
            match renderer.draw_next_frame(camera_matrix) {
                Ok(PresentResult::Ok) => return Ok(()),
                Ok(PresentResult::SwapchainOutOfDate) => {
                    renderer.recreate_swapchain(window_owner).unwrap();
                    aspect_ratio = renderer.get_aspect_ratio();
                },
                Err(e) => return Err(format!("{}", e))
            };
        } else {
            return Err(String::from("Drawing frame witthout a renderer set"))
        }

        self.update_aspect(aspect_ratio);
        Ok(())
    }
}
