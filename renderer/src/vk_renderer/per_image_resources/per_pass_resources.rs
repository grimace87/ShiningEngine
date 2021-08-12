
use crate::vk_renderer::{
    render_core::RenderCore,
    per_image_resources::renderpass::RenderpassWrapper,
    per_image_resources::PipelineSet
};
use defs::DrawingPass;
use ash::{
    vk,
    version::DeviceV1_0
};

pub struct PerPassResources {
    pub renderpass: RenderpassWrapper,
    pub renderpass_pipeline_set: PipelineSet
}

impl PerPassResources {
    pub fn new(render_core: &RenderCore, swapchain_image_index: usize, pass: &DrawingPass) -> Result<PerPassResources, String> {
        let renderpass = RenderpassWrapper::new(&render_core, swapchain_image_index, &pass.target)?;
        let renderpass_pipeline_set = PipelineSet::new(&render_core, &renderpass, pass)?;
        Ok(PerPassResources {
            renderpass,
            renderpass_pipeline_set
        })
    }

    pub unsafe fn record_command_buffer(&self, render_core: &RenderCore, command_buffer: vk::CommandBuffer) -> Result<(), String> {
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
        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass.renderpass)
            .framebuffer(self.renderpass.swapchain_framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_core.get_extent()?
            })
            .clear_values(&clear_values);
        render_core.device.cmd_begin_render_pass(command_buffer, &renderpass_begin_info, vk::SubpassContents::INLINE);
        self.renderpass_pipeline_set.record_command_buffer(render_core, command_buffer)?;
        render_core.device.cmd_end_render_pass(command_buffer);

        Ok(())
    }
}
