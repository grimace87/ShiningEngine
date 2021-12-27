
pub mod camera;
pub mod control;
pub mod timer;
pub mod util;
pub mod scene;

use defs::{
    EngineError,
    Scene,
    SceneManager,
    control::{
        Control,
        KeyCode,
        InputState
    },
    render::{
        RendererApi,
        PresentResult,
        FeatureDeclaration,
        DrawingDescription
    }
};
use renderer::null::NullRenderer;

use raw_window_handle::HasRawWindowHandle;
use std::marker::PhantomData;

/// Engine struct
/// The encapsulation of everything needed to run the application, with the exception of OS-
/// specific components.
pub struct Engine<R> where R : RendererApi {
    declared_features: Vec<FeatureDeclaration>,
    scene_host: crate::scene::SceneHost,
    phantom_renderer: PhantomData<R>,
    renderer: Box<dyn RendererApi>,
    controller: Box<dyn Control>,
    timer: Box<dyn crate::timer::Timer>,
    drawing_description: DrawingDescription,
}

impl<R: 'static> Engine<R> where R : RendererApi {

    /// Create a new instance, uninitialised except for having captured, but not yet used, the
    /// scene description
    pub fn new_uninitialised(
        scene_info: Box<dyn Scene>,
        features: Vec<FeatureDeclaration>
    ) -> Engine<R> {
        Engine {
            declared_features: features,
            scene_host: crate::scene::SceneHost::new(scene_info),
            phantom_renderer: PhantomData::default(),
            renderer: Box::new(NullRenderer::default()),
            controller: Box::new(crate::control::null::NullControl::default()),
            timer: Box::new(crate::timer::null::NullTimer::default()),
            drawing_description: DrawingDescription { passes: Vec::new() },
        }
    }

    /// Initialise the engine, given an available window to use. Creates the renderer and does
    /// the initialisation required by the current scene.
    pub fn initialise(&mut self, window_owner: &dyn HasRawWindowHandle) {

        let resource_preloads = self.scene_host.get_current().make_preloads();
        let description = self.scene_host.get_current().make_description();
        let renderer = R::new(
            window_owner, &self.declared_features, &resource_preloads, &description).unwrap();
        let aspect_ratio = renderer.get_aspect_ratio();
        self.scene_host.update_aspect_ratio(aspect_ratio);

        self.renderer = Box::new(renderer);
        self.controller = Box::new(crate::control::user::UserControl::default());
        self.timer = Box::new(crate::timer::global::GlobalTimer::default());
        self.drawing_description = description;
    }

    /// Pass keyboard events to the controller
    pub fn process_keyboard_event(&mut self, keycode: KeyCode, state: InputState) {
        self.controller.process_keyboard_event(keycode, state);
    }

    /// Retrieve a time step from the engine's timer
    pub fn pull_time_step_millis(&mut self) -> u64 {
        self.timer.pull_time_step_millis()
    }

    /// Instruct the renderer to recreate a suitable surface for this window, and notify
    /// components as needed of the updated aspect ratio.
    pub fn recreate_surface(
        &mut self,
        window_owner: &dyn HasRawWindowHandle
    ) -> Result<(), EngineError> {
        let aspect_ratio: f32;
        self.renderer.recreate_surface(window_owner, &self.drawing_description)?;
        aspect_ratio = self.renderer.get_aspect_ratio();
        self.scene_host.update_aspect_ratio(aspect_ratio);
        Ok(())
    }

    /// Perform a pre-render update event. This instructs the controller to do any internal
    /// updates that it can, instructs the current scene to do an update, and transition to a new
    /// scene if one was requested.
    pub fn update(&mut self, time_step_millis: u64) {
        self.controller.update();
        let next_scene =
            self.scene_host.update_current(time_step_millis, self.controller.as_ref());
        if let Some(new_scene) = next_scene {
            self.scene_host.queue_scene(new_scene);
        }
        if self.scene_host.drain_queue() {
            let resource_preloads = self.scene_host.get_current().make_preloads();
            let description = self.scene_host.get_current().make_description();
            self.renderer.recreate_scene_resources(&resource_preloads, &description).unwrap();
        }
    }

    /// Perform the render event. Instructs the renderer to draw the frame and then does some
    /// post-render handling, such as responding to the renderer reporting an out-of-date
    /// swapchain.
    pub fn render(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), EngineError> {

        let updated_aspect_ratio: f32;
        match self.renderer.draw_next_frame(self.scene_host.get_current()) {
            Ok(PresentResult::Ok) => return Ok(()),
            Ok(PresentResult::SwapchainOutOfDate) => {
                self.renderer.recreate_surface(window_owner, &self.drawing_description)?;
                updated_aspect_ratio = self.renderer.get_aspect_ratio();
            },
            Err(e) => return Err(e)
        };

        self.scene_host.update_aspect_ratio(updated_aspect_ratio);
        Ok(())
    }
}
