
use defs::{
    EngineError,
    render::{
        FramebufferTarget,
        FramebufferCreationData,
        TexturePixelFormat
    }
};
use ash::{
    vk,
    version::DeviceV1_0
};

/// RenderpassWrapper struct
/// Wraps resources related to renderpasses, including framebuffers. Resources need to be recreated
/// if the swapchain is recreated.
pub struct RenderpassWrapper {
    pub renderpass: vk::RenderPass,
    pub swapchain_framebuffer: vk::Framebuffer,
    pub custom_framebuffer: Option<vk::Framebuffer>
}

impl RenderpassWrapper {

    /// Create a new instance, with all resources initialised
    pub fn new(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        image_index: usize,
        framebuffer_target: &FramebufferTarget
    ) -> Result<RenderpassWrapper, EngineError> {
        let mut wrapper = RenderpassWrapper {
            renderpass: vk::RenderPass::null(),
            swapchain_framebuffer: vk::Framebuffer::null(),
            custom_framebuffer: None
        };
        unsafe {
            wrapper.create_resources(render_core, image_index, framebuffer_target)?;
        }
        Ok(wrapper)
    }

    /// Destroy the resources held
    pub fn destroy_resources(&self, render_core: &crate::vk_renderer::render_core::RenderCore) {
        unsafe {
            render_core.device.destroy_framebuffer(self.swapchain_framebuffer, None);
            if let Some(framebuffer) = self.custom_framebuffer.as_ref() {
                render_core.device.destroy_framebuffer(*framebuffer, None);
            }
            render_core.device.destroy_render_pass(self.renderpass, None);
        }
    }

    /// Call one of the other create-resources functions, depending on whether to render into a
    /// swapchain image or an offscreen target
    unsafe fn create_resources(
        &mut self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        image_index: usize,
        framebuffer_target: &FramebufferTarget
    ) -> Result<(), EngineError> {
        match framebuffer_target {
            FramebufferTarget::Texture(creation_data) => {
                self.create_offscreen_renderpass_resources(
                    render_core,
                    creation_data,
                    true)
            },
            FramebufferTarget::DefaultFramebuffer => {
                self.create_swapchain_renderpass_resources(
                    render_core,
                    image_index)
            }
        }
    }

    /// Create all resources for rendering into a swapchain image
    unsafe fn create_swapchain_renderpass_resources(
        &mut self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        image_index: usize
    ) -> Result<(), EngineError> {

        let depth_image = match render_core.get_depth_image() {
            Some(image) => image,
            _ => return Err(EngineError::RenderError(
                String::from("Creating new renderpass wrapper with no depth image available")
            ))
        };

        // Define subpass with single colour attachment
        let surface_format = render_core.get_surface_formats()?
            .first()
            .unwrap()
            .format;
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(surface_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .samples(vk::SampleCountFlags::TYPE_1)
                .build(),
            vk::AttachmentDescription::builder()
                .format(depth_image.format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .build()
        ];
        let color_attachment_refs = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };
        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];
        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                )
                .build()
        ];

        // Create the renderpass with this one subpass
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        let renderpass = render_core.device
            .create_render_pass(&renderpass_info, None)
            .map_err(|e| {
                EngineError::RenderError(format!("{:?}", e))
            })?;

        // Create framebuffers for the swapchain image views for use in this renderpass
        let framebuffer = self.create_swapchain_framebuffer(
            render_core,
            image_index,
            renderpass,
            depth_image)?;

        self.renderpass = renderpass;
        self.swapchain_framebuffer = framebuffer;
        self.custom_framebuffer = None;

        Ok(())
    }

    /// Create all resources for rendering into an offscreen framebuffer
    unsafe fn create_offscreen_renderpass_resources(
        &mut self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        config: &FramebufferCreationData,
        discard_existing_image_content: bool
    ) -> Result<(), EngineError> {

        // TODO - Something useful with this flag
        if !discard_existing_image_content {
            panic!(
                "Unhandled case RenderpassWrapper::create_offscreen_renderpass_resources with \
                discard_existing_image_content set to false"
            );
        }

        // Get the texture to use for color attachment
        let color_texture_image_view = render_core
            .query_texture(config.color_texture_index)?
            .image_view;
        let color_format = match config.color_format {
            TexturePixelFormat::Rgba => vk::Format::R8G8B8A8_UNORM,
            _ => return Err(EngineError::RenderError(
                format!("Cannot set color attachment tp {:?}", config.color_format)))
        };

        // Define subpass with single colour attachment and optionally depth attachment
        let initial_layout = vk::ImageLayout::UNDEFINED;
        let mut attachments = vec![vk::AttachmentDescription::builder()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(initial_layout)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build()];
        let depth_texture_image_view = match config.depth_texture_index {
            Some(depth_texture_index) => {
                // Get the texture to use for color attachment
                let depth_texture_image_view = render_core
                    .query_texture(depth_texture_index)?
                    .image_view;
                match config.depth_format {
                    TexturePixelFormat::Unorm16 => {
                        attachments.push(vk::AttachmentDescription::builder()
                            .format(vk::Format::D16_UNORM)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::DONT_CARE)
                            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                            .initial_layout(initial_layout)
                            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                            .samples(vk::SampleCountFlags::TYPE_1)
                            .build());
                    },
                    _ => return Err(EngineError::RenderError(
                        format!("Cannot set color attachment tp {:?}", config.color_format))
                    )
                };
                Some(depth_texture_image_view)
            },
            _ => None
        };

        let color_attachment_refs = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];

        // TODO - Depth attachment is optional
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];
        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build()
        ];

        // Create the renderpass with this one subpass
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments.as_slice())
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        let renderpass = render_core.device
            .create_render_pass(&renderpass_info, None)
            .map_err(|e| {
                EngineError::RenderError(format!("{:?}", e))
            })?;

        // Create framebuffers for swapchain image views, or new framebuffers from scratch, for use in this renderpass
        self.renderpass = renderpass;
        self.swapchain_framebuffer = vk::Framebuffer::null();
        self.custom_framebuffer = Some(Self::create_offscreen_framebuffer(
            render_core,
            renderpass,
            config,
            color_texture_image_view,
            depth_texture_image_view)?);

        Ok(())
    }

    /// Create a framebuffer for rendering into a swapchain image
    unsafe fn create_swapchain_framebuffer(
        &self,
        render_core: &crate::vk_renderer::render_core::RenderCore,
        image_index: usize,
        renderpass: vk::RenderPass,
        depth_image: &crate::vk_renderer::images::ImageWrapper
    ) -> Result<vk::Framebuffer, EngineError> {
        let extent = render_core.get_extent()?;
        let attachments_array = [
            render_core.image_views[image_index],
            depth_image.image_view
        ];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(&attachments_array)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        let framebuffer = render_core.device
            .create_framebuffer(&framebuffer_info, None)
            .map_err(|e| {
                EngineError::RenderError(format!("{:?}", e))
            })?;
        Ok(framebuffer)
    }

    /// Create a framebuffer for rendering into an offscreen image
    unsafe fn create_offscreen_framebuffer(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        renderpass: vk::RenderPass,
        config: &FramebufferCreationData,
        color_image: vk::ImageView,
        depth_image: Option<vk::ImageView>
    ) -> Result<vk::Framebuffer, EngineError> {

        let width = config.width as u32;
        let height = config.height as u32;

        let mut attachment_image_view = vec![color_image];
        if let Some(image_view) = depth_image.as_ref() {
            attachment_image_view.push(*image_view);
        }

        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(attachment_image_view.as_slice())
            .width(width)
            .height(height)
            .layers(1);
        render_core.device
            .create_framebuffer(&framebuffer_info, None)
            .map_err(|e| {
                EngineError::RenderError(format!("{:?}", e))
            })
    }
}
