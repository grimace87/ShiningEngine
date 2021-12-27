
use defs::{
    Scene,
    SceneManager,
    control::Control
};
use lockfree::queue::Queue;
use std::mem::MaybeUninit;

/// SceneHost struct
/// Manages scene objects, including holding on to the currently-active scene, and allowing new
/// scenes to be queued up to be transitioned to as soon as possible.
pub struct SceneHost {
    scene_info: Box<dyn Scene>,
    scene_queue: Queue<MaybeUninit<Box<dyn Scene>>>
}

impl SceneHost {

    /// Create a new instance, with an initial scene object and an empty queue of new scenes
    pub fn new(scene_info: Box<dyn Scene>) -> SceneHost {
        SceneHost {
            scene_info,
            scene_queue: Queue::new()
        }
    }

    /// Get an immutable reference to the current scene
    pub fn get_current(&self) -> &dyn Scene {
        self.scene_info.as_ref()
    }

    /// Pass on notification of changed aspect ratio to the current scene object
    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.scene_info.update_aspect_ratio(aspect_ratio);
    }

    /// Perform per-frame updates based on current camera position and the time step since the last
    /// invocation. If a new scene scene should be queued - given the update that occurred - it
    //  will be returned.
    pub fn update_current(
        &mut self,
        time_step_millis: u64,
        controller: &dyn Control
    ) -> Option<Box<dyn Scene>> {
        self.scene_info.update_camera(time_step_millis, controller)
    }

    /// Flush the scene queue, activating any new scenes found as they replace the current scene
    pub fn drain_queue(&mut self) -> bool {
        let mut changed = false;
        while let Some(new_scene) = self.scene_queue.next() {
            changed = true;
            self.scene_info = unsafe { new_scene.assume_init() };
        }
        changed
    }
}

impl SceneManager for SceneHost {

    /// Enqueue a new scene, to be activated on the next invocation of drain_queue
    fn queue_scene(&self, new_scene: Box<dyn Scene>) {
        self.scene_queue.push(MaybeUninit::new(new_scene));
    }
}
