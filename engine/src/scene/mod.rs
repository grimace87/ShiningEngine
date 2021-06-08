
use defs::{SceneInfo, SceneManager, Control};
use lockfree::queue::Queue;
use std::mem::MaybeUninit;

pub struct SceneHost {
    scene_info: Box<dyn SceneInfo>,
    scene_queue: Queue<MaybeUninit<Box<dyn SceneInfo>>>
}

impl SceneHost {
    pub fn new(scene_info: Box<dyn SceneInfo>) -> SceneHost {
        SceneHost {
            scene_info,
            scene_queue: Queue::new()
        }
    }

    pub fn get_current(&self) -> &dyn SceneInfo {
        self.scene_info.as_ref()
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.scene_info.update_aspect_ratio(aspect_ratio);
    }

    pub fn update_current(&mut self, time_step_millis: u64, controller: &dyn Control) -> Option<Box<dyn SceneInfo>> {
        self.scene_info.update_camera(time_step_millis, controller)
    }

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
    fn queue_scene(&self, new_scene: Box<dyn SceneInfo>) {
        self.scene_queue.push(MaybeUninit::new(new_scene));
    }
}
