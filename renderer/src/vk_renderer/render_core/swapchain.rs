
use ash::{
    vk,
    Device,
    extensions::khr::{
        Surface,
        Swapchain
    },
    version::DeviceV1_0
};
use crate::vk_renderer::render_core::device::PhysicalDeviceProperties;

pub const MIN_SWAPCHAIN_SIZE: u32 = 2;
pub const MAX_SWAPCHAIN_SIZE: u32 = 3;

pub unsafe fn create_swapchain(physical_device_struct: &PhysicalDeviceProperties, surface_fn: &Surface, surface: vk::SurfaceKHR, swapchain_fn: &Swapchain, previous_swapchain: vk::SwapchainKHR) -> Result<vk::SwapchainKHR, String> {

    let physical_device = physical_device_struct.physical_device;
    let graphics_queue_family_index = physical_device_struct.graphics_queue_family_index;

    // Make the swapchain; automatically creates the images (but not image views)
    let surface_capabilities = surface_fn
        .get_physical_device_surface_capabilities(physical_device, surface)
        .map_err(|e| format!("{:?}", e))?;
    let swapchain = {
        let surface_present_modes = surface_fn
            .get_physical_device_surface_present_modes(physical_device, surface)
            .map_err(|e| format!("{:?}", e))?;
        let surface_formats = surface_fn
            .get_physical_device_surface_formats(physical_device, surface)
            .map_err(|e| format!("{:?}", e))?;
        let present_supported = surface_fn
            .get_physical_device_surface_support(physical_device, graphics_queue_family_index, surface)
            .map_err(|e| format!("{:?}", e))?;
        if !present_supported {
            return Err(String::from("Presentation not supported by selected graphics queue family"));
        }
        if !surface_present_modes.contains(&vk::PresentModeKHR::FIFO) {
            return Err(String::from("FIFO presentation mode not supported by selected graphics queue family"));
        }

        let max_too_small = surface_capabilities.max_image_count != 0 && surface_capabilities.max_image_count < MIN_SWAPCHAIN_SIZE;
        let min_too_large = surface_capabilities.min_image_count > MAX_SWAPCHAIN_SIZE;
        if max_too_small || min_too_large {
            return Err(String::from("Requested swapchain size is not supported"));
        }

        let surface_format = surface_formats.first().unwrap();
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(MAX_SWAPCHAIN_SIZE)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_capabilities.current_extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .image_array_layers(1)
            .old_swapchain(previous_swapchain);

        swapchain_fn.create_swapchain(&swapchain_create_info, None)
            .map_err(|e| format!("{:?}", e))?
    };
    Ok(swapchain)
}

pub unsafe fn create_swapchain_image_views(device: &Device, swapchain_fn: &Swapchain, swapchain: vk::SwapchainKHR) -> Result<Vec<vk::ImageView>, String> {
    // Make the image views over the images
    let swapchain_images = swapchain_fn.get_swapchain_images(swapchain)
        .map_err(|e| format!("{:?}", e))?;
    let mut image_views = Vec::with_capacity(swapchain_images.len());
    for image in swapchain_images.iter() {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_UNORM)
            .subresource_range(*subresource_range);
        let image_view = device.create_image_view(&image_view_create_info, None)
            .map_err(|e| format!("{:?}", e))?;
        image_views.push(image_view);
    }

    let image_views: Vec<_> = swapchain_images.iter()
        .map(|image| {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            device.create_image_view(&image_view_create_info, None)
                .map_err(|e| format!("Error creating image views for swapchain: {:?}", e))
                .unwrap()
        })
        .collect();
    Ok(image_views)
}
