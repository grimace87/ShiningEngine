
use defs::{
    EngineError,
    render::{
        TexturePixelFormat,
        ImageUsage
    }
};
use ash::{
    vk,
    Device,
    version::DeviceV1_0
};

/// ImageCreationParams struct
/// Description for creating an image; should cover all use cases needed by the engine
struct ImageCreationParams {
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    aspect: vk::ImageAspectFlags,
    view_type: vk::ImageViewType,
    layer_count: u32
}

/// ImageWrapper struct
/// Wraps a Vulkan image, image view, the format used by the image, and the memory allocation
/// backing the image
pub struct ImageWrapper {
    allocation: vk_mem::Allocation,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub format: vk::Format
}

impl ImageWrapper {

    /// Create a new instance with nothing useful in it
    pub fn empty() -> ImageWrapper {
        ImageWrapper {
            allocation: vk_mem::Allocation::null(),
            image: vk::Image::null(),
            image_view: vk::ImageView::null(),
            format: vk::Format::UNDEFINED
        }
    }

    /// Create a new instance, fully initialised
    pub unsafe fn new(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        usage: ImageUsage,
        format: TexturePixelFormat,
        width: u32,
        height: u32,
        init_layer_data: Option<&Vec<Vec<u8>>>
    ) -> Result<ImageWrapper, EngineError> {

        let creation_params = match (usage, format) {
            // Typical depth buffer
            (ImageUsage::DepthBuffer, TexturePixelFormat::Unorm16) => {
                if init_layer_data.is_some() {
                    return Err(EngineError::RenderError(
                        String::from("Initialising depth buffer not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::D16_UNORM,
                    usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::DEPTH,
                    view_type: vk::ImageViewType::TYPE_2D,
                    layer_count: 1
                }
            },

            // Typical off-screen-rendered color attachment
            (ImageUsage::OffscreenRenderSampleColorWriteDepth, TexturePixelFormat::RGBA) => {
                if init_layer_data.is_some() {
                    return Err(EngineError::RenderError(
                        String::from("Initialising off-screen render image not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::TYPE_2D,
                    layer_count: 1
                }
            },

            // Typical off-screen-rendered depth attachment
            (ImageUsage::OffscreenRenderSampleColorWriteDepth, TexturePixelFormat::Unorm16) => {
                if init_layer_data.is_some() {
                    return Err(EngineError::RenderError(
                        String::from("Initialising off-screen render image not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::D16_UNORM,
                    usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::DEPTH,
                    view_type: vk::ImageViewType::TYPE_2D,
                    layer_count: 1
                }
            },

            // Typical initialised texture
            (ImageUsage::TextureSampleOnly, TexturePixelFormat::RGBA) => {
                if init_layer_data.is_none() {
                    return Err(EngineError::RenderError(
                        String::from("Not initialising sample-only texture not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::TYPE_2D,
                    layer_count: 1
                }
            },

            // Typical sky box (cube map)
            (ImageUsage::Skybox, TexturePixelFormat::RGBA) => {
                if init_layer_data.is_none() {
                    return Err(EngineError::RenderError(
                        String::from("Not initialising sample-only texture not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::CUBE,
                    layer_count: 6
                }
            },

            // Unhandled cases
            _ => {
                return Err(EngineError::RenderError(
                    String::from("Tried to create an image with an unhandled config")));
            }
        };

        let (allocation, image, image_view) = Self::make_image_and_view(
            render_core,
            width,
            height,
            &creation_params)?;

        if let Some(layer_data) = init_layer_data {
            Self::initialise_read_only_color_texture(
                render_core,
                width,
                height,
                &image,
                layer_data)?;
        }

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: creation_params.format
        })
    }

    /// Create the image and image view
    unsafe fn make_image_and_view(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        width: u32,
        height: u32,
        creation_params: &ImageCreationParams
    ) -> Result<(vk_mem::Allocation, vk::Image, vk::ImageView), EngineError> {
        let queue_families = [
            render_core.physical_device_properties.graphics_queue_family_index
        ];
        let extent3d = vk::Extent3D { width, height, depth: 1 };
        let flags = match creation_params.view_type {
            vk::ImageViewType::CUBE => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            _ => vk::ImageCreateFlags::empty()
        };

        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .flags(flags)
            .format(creation_params.format)
            .extent(extent3d)
            .mip_levels(1)
            .array_layers(creation_params.layer_count)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(creation_params.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_families)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (image, allocation, _) = render_core
            .get_mem_allocator()
            .create_image(&image_info, &allocation_info)
            .map_err(|e| {
                EngineError::RenderError(format! ("Allocation error: {:?}", e))
            })?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(creation_params.aspect)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(creation_params.layer_count);
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(creation_params.view_type)
            .format(creation_params.format)
            .subresource_range(*subresource_range);
        let image_view = render_core.device
            .create_image_view(&image_view_create_info, None)
            .map_err(|e| {
                EngineError::RenderError(format!("{:?}", e))
            })?;

        Ok((allocation, image, image_view))
    }

    /// Destroy all resources held by the instance
    pub unsafe fn destroy(
        &self,
        device: &Device,
        allocator: &vk_mem::Allocator
    ) -> Result<(), EngineError> {
        device.destroy_image_view(self.image_view, None);
        allocator.destroy_image(self.image, &self.allocation)
            .map_err(|e| {
                EngineError::RenderError(format!("Error freeing image: {:?}", e))
            })
    }

    /// Initialise the image's memory with texture data; uses a staging buffer to allocate device-
    /// local memory and transitions the image into the optimal layout for reading in samplers in
    /// shaders
    unsafe fn initialise_read_only_color_texture(
        render_core: &crate::vk_renderer::render_core::RenderCore,
        width: u32,
        height: u32,
        image: &vk::Image,
        layer_data: &Vec<Vec<u8>>) -> Result<(), EngineError> {

        if layer_data.is_empty() {
            panic!("Passed empty layer data as ImageWrapper init data")
        }
        let layer_count = layer_data.len();
        let layer_size_bytes = layer_data[0].len();

        // Staging buffer
        let expected_data_size: usize = layer_count * 4 * width as usize * height as usize;
        if expected_data_size != layer_count * layer_size_bytes {
            panic!("Image data does not match expected size");
        }
        let mut staging_buffer = crate::vk_renderer::buffers::BufferWrapper::new(
            render_core.get_mem_allocator(),
            layer_count * layer_size_bytes,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu)?;
        for (layer_no, data) in layer_data.iter().enumerate() {
            staging_buffer.update::<u8>(
                render_core.get_mem_allocator(),
                (layer_no * layer_size_bytes) as isize,
                data.as_ptr() as *const u8,
                layer_size_bytes)?;
        }

        // Allocate a single-use command buffer and begin recording
        // Using the transfer queue for this - note that it doesn't support all access or pipeline stage flags
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(render_core.transfer_command_buffer_pool)
            .command_buffer_count(1);
        let copy_command_buffer = render_core.device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .map_err(|e| {
                EngineError::RenderError(format!("Error allocating command buffer: {:?}", e))
            })?[0];
        let command_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        render_core.device.begin_command_buffer(copy_command_buffer, &command_begin_info)
            .map_err(|e| {
                EngineError::RenderError(format!("Error starting copy command buffer: {:?}", e))
            })?;

        // Initial memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(*image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: layer_count as u32
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
            layer_count: layer_count as u32
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
            *image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region]
        );

        // Final memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(*image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: layer_count as u32
            })
            .build();
        render_core.device.cmd_pipeline_barrier(
            copy_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Finish recording commands, create a fence, run the command, wait for fence, clean up
        render_core.device.end_command_buffer(copy_command_buffer)
            .map_err(|e| {
                EngineError::RenderError(format!("Error ending command buffer: {:?}", e))
            })?;
        let submit_infos = [
            vk::SubmitInfo::builder()
                .command_buffers(&[copy_command_buffer])
                .build()
        ];
        let fence = render_core.device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .map_err(|e| {
                EngineError::RenderError(format!("Error creating fence: {:?}", e))
            })?;
        render_core.device
            .queue_submit(render_core.transfer_queue, &submit_infos, fence)
            .map_err(|e| {
                EngineError::RenderError(format!("Error submitting to queue: {:?}", e))
            })?;
        render_core.device
            .wait_for_fences(&[fence], true, std::u64::MAX)
            .map_err(|e| {
                EngineError::RenderError(format!("Error waiting for fence: {:?}", e))
            })?;
        render_core.device
            .destroy_fence(fence, None);
        staging_buffer.destroy(render_core.get_mem_allocator())?;
        render_core.device
            .free_command_buffers(
            render_core.transfer_command_buffer_pool,
            &[copy_command_buffer]);

        Ok(())
    }
}
