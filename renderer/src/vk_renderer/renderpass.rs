
use crate::vk_renderer::RenderCore;

use ash::{
    vk,
    version::DeviceV1_0
};

pub struct RenderpassWrapper {
    pub renderpass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>
}

/// Wraps resources related to renderpasses, including framebuffers. Resources need to be recreated
/// if the swapchain is recreated.
impl RenderpassWrapper {

    pub fn new(render_core: &RenderCore) -> Result<RenderpassWrapper, String> {
        let mut wrapper = RenderpassWrapper {
            renderpass: vk::RenderPass::null(),
            framebuffers: vec![]
        };
        unsafe {
            wrapper.create_resources(render_core)?;
        }
        Ok(wrapper)
    }

    pub fn destroy_resources(&self, render_core: &RenderCore) {
        unsafe {
            for framebuffer in self.framebuffers.iter() {
                render_core.device.destroy_framebuffer(*framebuffer, None);
            }
            render_core.device.destroy_render_pass(self.renderpass, None);
        }
    }

    pub unsafe fn create_resources(&mut self, render_core: &RenderCore) -> Result<(), String> {

        let depth_image = match render_core.get_depth_image() {
            Some(image) => image,
            _ => return Err(String::from("Creating new renderpass wrapper with no depth image available"))
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
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build()
        ];

        // Create the renderpass with this one subpass
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        let renderpass = render_core.device
            .create_render_pass(&renderpass_info, None)
            .map_err(|e| format!("{:?}", e))?;

        // Create framebuffers for the swapchain image views for use in this renderpass
        let extent = render_core.get_extent()?;
        let mut framebuffers = vec![];
        for image_view in render_core.image_views.iter() {
            let attachments_array = [*image_view, depth_image.image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&attachments_array)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            let framebuffer = render_core.device
                .create_framebuffer(&framebuffer_info, None)
                .map_err(|e| format!("{:?}", e))?;
            framebuffers.push(framebuffer);
        }

        self.renderpass = renderpass;
        self.framebuffers.clear();
        for framebuffer in framebuffers.iter() {
            self.framebuffers.push(*framebuffer);
        }

        Ok(())
    }
}
