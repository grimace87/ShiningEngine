
mod pipeline;
mod renderpass;
mod pipeline_set;
mod per_pass_resources;

use defs::{
    SceneInfo,
    EngineError,
    render::{
        DrawingDescription,
        FramebufferTarget,
        FramebufferCreationData
    }
};
use ash::{
    vk,
    version::DeviceV1_0
};

/// PerImageResources struct
/// Encapsulates the resources needed for rendering everything into a particular swapchain image.
/// This will be instantiated once per swapchain image.
pub struct PerImageResources {
    resources: Vec<crate::vk_renderer::per_image_resources::per_pass_resources::PerPassResources>,
    command_buffer: vk::CommandBuffer
}

impl PerImageResources {

    /// Create a new instance, creating the resources needed to render the scene described by the
    /// supplied image; this should be used for rendering into one and only one swapchain image.
    pub fn new(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        swapchain_image_index: usize,
        description: &DrawingDescription,
        command_buffer: vk::CommandBuffer
    ) -> Result<PerImageResources, EngineError> {
        let resources = description.passes.iter()
            .map(|pass| {
                crate::vk_renderer::per_image_resources::per_pass_resources::PerPassResources::new(
                    render_core, swapchain_image_index, pass
                ).unwrap()
            })
            .collect();
        Ok(PerImageResources {
            resources,
            command_buffer
        })
    }

    /// Records the command buffer to perform all drawing for the scene into the intended swapchain
    /// image.
    pub unsafe fn record_command_buffer(
        &self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        description: &DrawingDescription,
        command_buffer: vk::CommandBuffer
    ) -> Result<(), EngineError> {
        let begin_info = vk::CommandBufferBeginInfo::builder();

        // TODO - More sophisticated logic to clear all framebuffers in use once only
        let mut _needing_clear_image = true;
        render_core.device.begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| EngineError::RenderError(format!("{:?}", e)))?;
        for (pass_index, resources) in self.resources.iter().enumerate() {
            let pass = &description.passes[pass_index];
            let render_extent = match &pass.target {
                FramebufferTarget::Texture(framebuffer_config) =>
                    vk::Extent2D {
                        width: framebuffer_config.width as u32,
                        height: framebuffer_config.height as u32
                    },
                _ => render_core.get_extent()?
            };
            resources.record_command_buffer(render_core, command_buffer, render_extent)?;
            if let FramebufferTarget::Texture(framebuffer_spec) = &pass.target {
                self.insert_pipeline_barrier(render_core, &framebuffer_spec, command_buffer)?;
            }
        }
        render_core.device.end_command_buffer(command_buffer)
            .map_err(|e| EngineError::RenderError(format!("{:?}", e)))?;
        Ok(())
    }

    /// Destroy the graphics resources that were previously created by this instance
    pub fn destroy_resources(&mut self, render_core: &crate::vk_renderer::render_core::RenderCore) {
        for resources in self.resources.iter_mut() {
            resources.renderpass_pipeline_set.destroy_resources(render_core);
            resources.renderpass.destroy_resources(render_core);
        }
    }

    /// Perform pre-rendering operations, namely updating the uniform buffer for each pass within
    /// this full render description
    pub unsafe fn on_pre_render(
        &mut self,
        render_core: &mut crate::vk_renderer::render_core::RenderCore,
        scene_info: &dyn SceneInfo
    ) {
        for (pass_index, resources) in self.resources.iter_mut().enumerate() {
            resources.renderpass_pipeline_set
                .update_uniform_buffer(render_core, scene_info, pass_index).unwrap();
        }
    }

    /// Retrieve the command buffer held by this instance
    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    /// Record a pipeline barrier command into the command buffer, containing an image barrier that
    /// translates an image layout as needed
    unsafe fn insert_pipeline_barrier(
        &self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        framebuffer_spec: &FramebufferCreationData,
        command_buffer: vk::CommandBuffer
    ) -> Result<(), EngineError> {
        let mut barriers = vec![];

        let image = render_core.query_texture(framebuffer_spec.color_texture_index)?;
        barriers.push(vk::ImageMemoryBarrier::builder()
            .image(image.image)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            })
            .build());
        render_core.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            barriers.as_slice()
        );
        Ok(())
    }
}
