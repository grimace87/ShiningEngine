
use crate::vk_renderer::{
    render_core::RenderCore,
    buffers::BufferWrapper
};

use ash::{
    vk,
    Device,
    version::DeviceV1_0
};

pub const PROJ_VK_DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;
pub const PROJ_VK_TEXTURE_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;

pub struct ImageWrapper {
    allocation: vk_mem::Allocation,
    image: vk::Image,
    pub image_view: vk::ImageView,
    pub format: vk::Format
}

impl ImageWrapper {

    unsafe fn make_image_and_view(
        render_core: &RenderCore,
        width: u32,
        height: u32,
        format: vk::Format,
        image_usage_flags: vk::ImageUsageFlags,
        sharing_mode: vk::SharingMode,
        aspect_flags: vk::ImageAspectFlags
    ) -> Result<(vk_mem::Allocation, vk::Image, vk::ImageView), String> {
        let queue_families = [render_core.physical_device_properties.graphics_queue_family_index];
        let extent3d = vk::Extent3D { width, height, depth: 1 };
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(image_usage_flags)
            .sharing_mode(sharing_mode)
            .queue_family_indices(&queue_families);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (image, allocation, _) = render_core.get_mem_allocator().create_image(&image_info, &allocation_info)
            .map_err(|e| format! ("Allocation error: {:?}", e))?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_flags)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(*subresource_range);
        let image_view = render_core.device
            .create_image_view(&image_view_create_info, None)
            .map_err(|e| format!("{:?}", e))?;

        Ok((allocation, image, image_view))
    }

    pub fn empty() -> ImageWrapper {
        ImageWrapper {
            allocation: vk_mem::Allocation::null(),
            image: vk::Image::null(),
            image_view: vk::ImageView::null(),
            format: vk::Format::UNDEFINED
        }
    }

    pub unsafe fn destroy(&self, device: &Device, allocator: &vk_mem::Allocator) -> Result<(), String> {
        device.destroy_image_view(self.image_view, None);
        allocator.destroy_image(self.image, &self.allocation)
            .map_err(|e| format!("Error freeing image: {:?}", e))
    }

    pub unsafe fn new_depth_image(
        render_core: &RenderCore,
        width: u32,
        height: u32) -> Result<ImageWrapper, String> {

        let (allocation, image, image_view) =
            Self::make_image_and_view(
                render_core,
                width,
                height,
                self::PROJ_VK_DEPTH_FORMAT,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::SharingMode::EXCLUSIVE,
                vk::ImageAspectFlags::DEPTH)?;

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: self::PROJ_VK_DEPTH_FORMAT
        })
    }

    pub unsafe fn new_texture_image_uninitialised(
        render_core: &RenderCore,
        width: u32,
        height: u32) -> Result<ImageWrapper, String> {

        let (allocation, image, image_view) =
            Self::make_image_and_view(
                render_core,
                width,
                height,
                self::PROJ_VK_TEXTURE_FORMAT,
                vk::ImageUsageFlags::SAMPLED,
                vk::SharingMode::EXCLUSIVE,
                vk::ImageAspectFlags::DEPTH)?;

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: self::PROJ_VK_TEXTURE_FORMAT
        })
    }

    pub unsafe fn new_initialised_texture_image_rgba(
        render_core: &RenderCore,
        width: u32,
        height: u32,
        init_data: &Vec<u8>) -> Result<ImageWrapper, String> {

        let pixel_size_bytes: usize = 4;

        let (allocation, image, image_view) =
            Self::make_image_and_view(
                render_core,
                width,
                height,
                self::PROJ_VK_TEXTURE_FORMAT,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                vk::SharingMode::EXCLUSIVE,
                vk::ImageAspectFlags::COLOR)?;

        // Staging buffer
        let mut staging_buffer = BufferWrapper::new_staging_buffer(
            render_core.get_mem_allocator(),
            pixel_size_bytes * width as usize * height as usize
        )?;
        staging_buffer.update_from_vec(render_core.get_mem_allocator(), init_data)?;

        // Allocate a single-use command buffer and begin recording
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(render_core.graphics_command_buffer_pool)
            .command_buffer_count(1);
        let copy_command_buffer = render_core.device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .map_err(|e| format!("Error allocating command buffer: {:?}", e))?[0];
        let command_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        render_core.device.begin_command_buffer(copy_command_buffer, &command_begin_info)
            .map_err(|e| format!("Error starting copy command buffer: {:?}", e))?;

        // Initial memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            })
            .build();
        render_core.device.cmd_pipeline_barrier(
            copy_command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Copy command
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1
        };
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D { width, height, depth: 1 },
            image_subresource,
            ..Default::default()
        };
        render_core.device.cmd_copy_buffer_to_image(
            copy_command_buffer,
            staging_buffer.buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region]
        );

        // Final memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            })
            .build();
        render_core.device.cmd_pipeline_barrier(
            copy_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Finish recording commands, create a fence, run the command, wait for fence, clean up
        render_core.device.end_command_buffer(copy_command_buffer)
            .map_err(|e| format!("Error ending command buffer: {:?}", e))?;
        let submit_infos = [
            vk::SubmitInfo::builder()
                .command_buffers(&[copy_command_buffer])
                .build()
        ];
        let fence = render_core.device.create_fence(&vk::FenceCreateInfo::default(), None)
            .map_err(|e| format!("Error creating fence: {:?}", e))?;
        render_core.device.queue_submit(render_core.graphics_queue, &submit_infos, fence)
            .map_err(|e| format!("Error submitting to queue: {:?}", e))?;
        render_core.device.wait_for_fences(&[fence], true, std::u64::MAX)
            .map_err(|e| format!("Error waiting for fence: {:?}", e))?;
        render_core.device.destroy_fence(fence, None);
        staging_buffer.destroy(render_core.get_mem_allocator())?;
        render_core.device.free_command_buffers(render_core.graphics_command_buffer_pool, &[copy_command_buffer]);

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: self::PROJ_VK_TEXTURE_FORMAT
        })
    }
}
