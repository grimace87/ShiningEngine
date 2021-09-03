
use crate::vk_renderer::{
    RenderCore,
    per_image_resources::{
        renderpass::RenderpassWrapper,
        pipeline::PipelineWrapper
    }
};

use defs::render::{DrawingPass, SceneInfo, FramebufferTarget};

use ash::vk;

pub struct PipelineSet {
    pipelines: Vec<PipelineWrapper>
}

impl PipelineSet {

    pub fn new(render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper, description: &DrawingPass) -> Result<PipelineSet, String> {

        let pipelines = description.steps
            .iter()
            .map(|_description| PipelineWrapper::new().unwrap())
            .collect();

        let mut pipeline_set = PipelineSet { pipelines };
        unsafe {
            pipeline_set.create_resources(render_core, renderpass_wrapper, description)?;
        }

        Ok(pipeline_set)
    }

    unsafe fn create_resources(&mut self, render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper, description: &DrawingPass) -> Result<(), String> {
        let render_extent = match &description.target {
            FramebufferTarget::Texture(framebuffer_config) => vk::Extent2D { width: framebuffer_config.width as u32, height: framebuffer_config.height as u32 },
            _ => render_core.get_extent()?
        };
        for (i, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.create_resources(render_core, renderpass_wrapper, &description.steps[i], render_extent)?;
        }
        Ok(())
    }

    pub fn destroy_resources(&mut self, render_core: &RenderCore) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(render_core);
        }
    }

    pub unsafe fn record_command_buffer(&self, render_core: &RenderCore, command_buffer: vk::CommandBuffer) -> Result<(), String> {
        for pipeline in self.pipelines.iter() {
            pipeline
                .record_commands(command_buffer, render_core)
                .unwrap();
        }
        Ok(())
    }

    pub unsafe fn update_uniform_buffer(&mut self, render_core: &mut RenderCore, scene_info: &dyn SceneInfo, pass_index: usize) -> Result<(), String> {
        for (step_index, pipeline) in self.pipelines.iter_mut().enumerate() {
            let (data_ptr, size_bytes) = scene_info.get_ubo_data_ptr_and_size(pass_index, step_index);
            pipeline.update_uniform_buffer(render_core, data_ptr, size_bytes)?;
        }
        Ok(())
    }
}
