
use crate::vk_renderer::{
    images::ImageWrapper,
    render_core::RenderCore
};

use defs::{FramebufferCreationData, TexturePixelFormat, ImageUsage};

use ash::vk;
use ash::version::DeviceV1_0;

pub struct FramebufferWrapper {
    pub framebuffer: vk::Framebuffer,
    pub color_attachment: Option<ImageWrapper>,
    pub depth_attachment: Option<ImageWrapper>
}

impl FramebufferWrapper {
    pub unsafe fn new_from_config(render_core: &RenderCore, renderpass: vk::RenderPass, config: &FramebufferCreationData) -> Result<FramebufferWrapper, String> {

        let width = config.width as u32;
        let height = config.height as u32;
        let depth_image: Option<ImageWrapper> = match config.depth_format {
            TexturePixelFormat::Unorm16 => Some(
                ImageWrapper::new(render_core, ImageUsage::DepthBuffer, TexturePixelFormat::Unorm16, width, height, None)?
            ),
            _ => return Err(format!("Invalid depth format: {:?}", config.depth_format))
        };
        let colour_image: Option<ImageWrapper> = match config.color_format {
            TexturePixelFormat::RGBA => Some(
                ImageWrapper::new(render_core, ImageUsage::OffscreenRenderTextureSample, TexturePixelFormat::RGBA, width, height, None)?
            ),
            _ => return Err(format!("Invalid color format: {:?}", config.color_format))
        };

        let mut attachment_image_view = vec![];
        if let Some(wrapper) = colour_image.as_ref() {
            attachment_image_view.push(wrapper.image_view);
        }
        if let Some(wrapper) = depth_image.as_ref() {
            attachment_image_view.push(wrapper.image_view);
        }

        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(attachment_image_view.as_slice())
            .width(width)
            .height(height)
            .layers(1);
        let framebuffer = render_core.device
            .create_framebuffer(&framebuffer_info, None)
            .map_err(|e| format!("{:?}", e))?;

        Ok(FramebufferWrapper {
            framebuffer,
            color_attachment: colour_image,
            depth_attachment: depth_image
        })
    }
}
