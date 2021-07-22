
use crate::vk_renderer::{
    render_core::RenderCore,
    renderpass::RenderpassWrapper,
    pipeline_set::PipelineSet
};
use defs::{
    SceneInfo,
    DrawingDescription
};

use ash::vk;

pub struct PerImageResources {
    pub renderpasses: Vec<RenderpassWrapper>,
    pub renderpass_pipeline_sets: Vec<PipelineSet>,
    command_buffer: vk::CommandBuffer
}

impl PerImageResources {

    pub fn new(render_core: &RenderCore, image_index: usize, description: &DrawingDescription, command_buffer: vk::CommandBuffer) -> PerImageResources {

        let renderpasses: Vec<RenderpassWrapper> = description.passes.iter()
            .map(|pass| RenderpassWrapper::new(&render_core, image_index, &pass.target).unwrap())
            .collect();
        let renderpass_pipeline_sets = description.passes.iter()
            .enumerate()
            .map(|(i, pass)| PipelineSet::new(&render_core, &renderpasses[i], pass, &command_buffer).unwrap())
            .collect();

        PerImageResources {
            renderpasses,
            renderpass_pipeline_sets,
            command_buffer
        }
    }

    pub fn destroy_resources(&mut self, render_core: &RenderCore) {
        for pipeline in self.renderpass_pipeline_sets.iter_mut() {
            pipeline.destroy_resources(render_core);
        }
        for renderpass in self.renderpasses.iter_mut() {
            renderpass.destroy_resources(render_core);
        }
    }

    pub unsafe fn on_pre_render(&mut self, render_core: &mut RenderCore, scene_info: &dyn SceneInfo) {
        for pipeline_set in self.renderpass_pipeline_sets.iter_mut() {
            pipeline_set.update_uniform_buffer(render_core, scene_info).unwrap();
        }
    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}
