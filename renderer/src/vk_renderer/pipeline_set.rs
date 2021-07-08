
use crate::vk_renderer::{
    RenderCore,
    renderpass::RenderpassWrapper,
    pipeline::PipelineWrapper
};

use defs::{DrawingPass, SceneInfo};

use ash::{
    vk,
    version::DeviceV1_0
};

pub struct PipelineSet {
    pipelines: Vec<PipelineWrapper>
}

impl PipelineSet {

    pub fn new(render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper, description: &DrawingPass, command_buffer: &vk::CommandBuffer) -> Result<PipelineSet, String> {

        let pipelines = description.steps
            .iter()
            .map(|_description| PipelineWrapper::new().unwrap())
            .collect();

        let mut pipeline_set = PipelineSet { pipelines };
        unsafe {
            pipeline_set.create_resources(render_core, renderpass_wrapper, description)?;
            pipeline_set.record_command_buffer(command_buffer, render_core, renderpass_wrapper)?;
        }

        Ok(pipeline_set)
    }

    pub unsafe fn create_resources(&mut self, render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper, description: &DrawingPass) -> Result<(), String> {
        for (i, pipeline) in self.pipelines.iter_mut().enumerate() {
            pipeline.create_resources(render_core, renderpass_wrapper, &description.steps[i])?;
        }
        Ok(())
    }

    pub fn destroy_resources(&mut self, render_core: &RenderCore) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(render_core);
        }
    }

    unsafe fn record_command_buffer(&self, command_buffer: &vk::CommandBuffer, render_core: &RenderCore, renderpass_wrapper: &RenderpassWrapper) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo::builder();
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
            .render_pass(renderpass_wrapper.renderpass)
            .framebuffer(renderpass_wrapper.swapchain_framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_core.get_extent()?
            })
            .clear_values(&clear_values);

        render_core.device.begin_command_buffer(*command_buffer, &begin_info)
            .map_err(|e| format!("{:?}", e))?;
        render_core.device.cmd_begin_render_pass(*command_buffer, &renderpass_begin_info, vk::SubpassContents::INLINE);

        // Draw calls for each pipeline (one pipeline per drawing step)
        for pipeline in self.pipelines.iter() {
            pipeline
                .record_commands(*command_buffer, render_core)
                .unwrap();
        }

        render_core.device.cmd_end_render_pass(*command_buffer);
        render_core.device.end_command_buffer(*command_buffer)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub unsafe fn update_uniform_buffer(&mut self, render_core: &mut RenderCore, scene_info: &dyn SceneInfo) -> Result<(), String> {
        for (i, pipeline) in self.pipelines.iter_mut().enumerate() {
            let (data_ptr, size_bytes) = scene_info.get_ubo_data_ptr_and_size(i);
            pipeline.update_uniform_buffer(render_core, data_ptr, size_bytes)?;
        }
        Ok(())
    }
}
