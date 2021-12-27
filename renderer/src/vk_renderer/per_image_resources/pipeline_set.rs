
use defs::{
    SceneInfo,
    Scene,
    EngineError,
    render::{
        DrawingPass,
        FramebufferTarget
    }
};
use ash::vk;

/// PipelineSet struct
/// A set of pipelines to be rendered in a single drawing pass - one per step
pub struct PipelineSet {
    pipelines: Vec<crate::vk_renderer::per_image_resources::pipeline::PipelineWrapper>
}

impl PipelineSet {

    /// Create a new instance, with all necessary resources created
    pub fn new(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        renderpass_wrapper: &crate::vk_renderer::per_image_resources::renderpass::RenderpassWrapper,
        description: &DrawingPass
    ) -> Result<PipelineSet, EngineError> {

        let pipelines = description.steps
            .iter()
            .map(|_description|
                crate::vk_renderer::per_image_resources::pipeline::PipelineWrapper::new()
            )
            .collect();

        let mut pipeline_set = PipelineSet { pipelines };
        unsafe {
            pipeline_set.create_resources(render_core, renderpass_wrapper, description)?;
        }

        Ok(pipeline_set)
    }

    /// Create the needed resources; used internally
    unsafe fn create_resources(
        &mut self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        renderpass_wrapper: &crate::vk_renderer::per_image_resources::renderpass::RenderpassWrapper,
        description: &DrawingPass
    ) -> Result<(), EngineError> {
        let render_extent = match &description.target {
            FramebufferTarget::Texture(framebuffer_config) => vk::Extent2D {
                width: framebuffer_config.width as u32,
                height: framebuffer_config.height as u32
            },
            _ => render_core.get_extent()?
        };
        for (i, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.create_resources(
                render_core,
                renderpass_wrapper,
                &description.steps[i],
                render_extent)?;
        }
        Ok(())
    }

    /// Destroy the resources created earlier and held by this instance
    pub fn destroy_resources(&mut self, render_core: &crate::vk_renderer::render_core::RenderCore) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(render_core);
        }
    }

    /// Record commands needed to render the rendering pass represented by this pipeline set; it is
    /// assumed that begin/end renderpass commands are performed separately to this
    pub unsafe fn record_command_buffer(
        &self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        command_buffer: vk::CommandBuffer
    ) {
        for pipeline in self.pipelines.iter() {
            pipeline.record_commands(command_buffer, render_core);
        }
    }

    /// Update the unform buffer for each pipeline within this set
    pub unsafe fn update_uniform_buffer(
        &mut self,
        render_core: &mut crate::vk_renderer::render_core::RenderCore,
        scene_info: &dyn Scene,
        pass_index: usize
    ) -> Result<(), EngineError> {
        for (step_index, pipeline) in self.pipelines.iter_mut().enumerate() {
            let (data_ptr, size_bytes) =
                scene_info.get_ubo_data_ptr_and_size(pass_index, step_index);
            pipeline.update_uniform_buffer(render_core, data_ptr, size_bytes)?;
        }
        Ok(())
    }
}
