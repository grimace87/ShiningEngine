
use defs::{
    EngineError,
    render::DrawingPass
};
use ash::{
    vk,
    version::DeviceV1_0
};

/// PerPassResources struct
pub struct PerPassResources {
    pub renderpass: crate::vk_renderer::per_image_resources::renderpass::RenderpassWrapper,
    pub renderpass_pipeline_set: crate::vk_renderer::per_image_resources::pipeline_set::PipelineSet
}

impl PerPassResources {

    /// Create a new instance, with resources created as needed to render the supplied pass
    pub fn new(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        swapchain_image_index: usize,
        pass: &DrawingPass
    ) -> Result<PerPassResources, EngineError> {
        let renderpass =
            crate::vk_renderer::per_image_resources::renderpass::RenderpassWrapper::new(
                &render_core,
                swapchain_image_index,
                &pass.target)?;
        let renderpass_pipeline_set =
            crate::vk_renderer::per_image_resources::pipeline_set::PipelineSet::new(
                &render_core,
                &renderpass,
                pass)?;
        Ok(PerPassResources {
            renderpass,
            renderpass_pipeline_set
        })
    }

    /// Record commands needed to render this pass, including the begin/end renderpass commands
    pub unsafe fn record_command_buffer(
        &self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D
    ) -> Result<(), EngineError> {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0]
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            }
        ];
        let framebuffer = match self.renderpass.custom_framebuffer {
            Some(f) => f,
            _ => self.renderpass.swapchain_framebuffer
        };
        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass.renderpass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_extent
            })
            .clear_values(&clear_values);
        render_core.device.cmd_begin_render_pass(
            command_buffer, &renderpass_begin_info, vk::SubpassContents::INLINE);
        self.renderpass_pipeline_set.record_command_buffer(render_core, command_buffer);
        render_core.device.cmd_end_render_pass(command_buffer);

        Ok(())
    }
}
