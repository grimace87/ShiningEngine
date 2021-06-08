
use defs::{SceneInfo, SceneManager};
use lockfree::queue::Queue;
use cgmath::Matrix4;
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

    pub fn update_current(&mut self, camera_matrix: &Matrix4<f32>) -> Option<Box<dyn SceneInfo>> {
        self.scene_info.on_camera_updated(camera_matrix)
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
