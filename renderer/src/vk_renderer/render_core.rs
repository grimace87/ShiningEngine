
use crate::vk_renderer::images::ImageWrapper;

use defs::{PresentResult, ResourcePreloads, VertexFormat};

use std::{
    ffi::CString,
    os::raw::c_char
};
use ash::{
    vk,
    Entry,
    Instance,
    Device,
    version::{EntryV1_0, InstanceV1_0, DeviceV1_0},
    extensions::{
        ext::DebugUtils,
        khr::{Swapchain, Surface}
    }
};
use vk_mem::AllocatorCreateFlags;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
use crate::vk_renderer::buffers::BufferWrapper;

pub const PROJ_VK_SWAPCHAIN_SIZE: usize = 2;

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity);
    let ty = format!("{:?}", message_type);
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

#[derive(Copy, Clone)]
pub struct PhysicalDeviceProperties {
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
}

/// Wraps resources that either never change while the app is running, or rarely change (perhaps
/// when a window is resized or the surface becomes invalidated for some other reason).
///
/// ### Main components
/// - Vulkan instance and debug utilities
/// - Vulkan physical device, logical device and queue family indices
/// - Vulkan swapchain with image views for its images
pub struct RenderCore {
    pub instance: Instance,
    utils_messenger: vk::DebugUtilsMessengerEXT,
    debug_utils: DebugUtils,
    pub device: Device,
    pub physical_device_properties: PhysicalDeviceProperties,
    mem_allocator: vk_mem::Allocator,
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub graphics_command_buffer_pool: vk::CommandPool,
    pub transfer_command_buffer_pool: vk::CommandPool,
    sync_image_available: Vec<vk::Semaphore>,
    sync_may_begin_rendering: Vec<vk::Fence>,
    sync_rendering_finished: Vec<vk::Semaphore>,
    current_image_acquired: usize,
    surface_fn: Surface,
    surface: vk::SurfaceKHR,
    swapchain_fn: Swapchain,
    swapchain: vk::SwapchainKHR,
    pub image_views: Vec<vk::ImageView>,
    depth_image: Option<ImageWrapper>,
    vbo_objects: HashMap<usize, (usize, BufferWrapper)>,
    texture_objects: HashMap<usize, ImageWrapper>
}

impl Drop for RenderCore {
    fn drop(&mut self) {
        unsafe {
            self.destroy_swapchain();
            self.destroy_surface();
            for semaphore in self.sync_rendering_finished.iter_mut() {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for fence in self.sync_may_begin_rendering.iter_mut() {
                self.device.destroy_fence(*fence, None);
            }
            for semaphore in self.sync_image_available.iter_mut() {
                self.device.destroy_semaphore(*semaphore, None);
            }
            self.device.destroy_command_pool(self.transfer_command_buffer_pool, None);
            self.device.destroy_command_pool(self.graphics_command_buffer_pool, None);
            self.destroy_all_resources();
            self.mem_allocator.destroy();
            self.device.destroy_device(None);
            self.debug_utils.destroy_debug_utils_messenger(self.utils_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

impl RenderCore {

    pub fn new(entry: &Entry, window_owner: &dyn HasRawWindowHandle, resource_preloads: &ResourcePreloads) -> Result<RenderCore, String> {
        Ok(unsafe {
            let mut core = Self::new_with_surface_without_swapchain(entry, window_owner)?;
            core.create_swapchain()?;
            core.load_new_resources(resource_preloads).unwrap();
            core
        })
    }

    unsafe fn new_with_surface_without_swapchain(entry: &Entry, window_owner: &dyn HasRawWindowHandle) -> Result<RenderCore, String> {

        // TODO - MATCH ALL THIS FUNCTIONS WITH EXAMPLE AROUND LINE 230 AND SO ON FROM:
        // https://github.com/MaikKlein/ash/blob/master/examples/src/lib.rs

        let instance = {

            let required_platform_extensions = Self::get_window_extensions(window_owner)?;

            // App info
            let engine_name = CString::new("ThisIsAnEngine").unwrap();
            let app_name = CString::new("Strength Beyond Fear").unwrap();
            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(vk::make_version(0, 1, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_version(0, 0, 1))
                .api_version(vk::make_version(1, 0, 0));

            // Instance extensions
            let mut extension_name_pointers: Vec<*const c_char> = vec![
                ash::extensions::ext::DebugUtils::name().as_ptr()
            ];
            extension_name_pointers.extend(&required_platform_extensions);

            // Validation layers
            // TODO - Check for layer support. This one seemingly not supported by Intel HD 530.
            let layer_names: Vec<CString> = vec![
                CString::new("VK_LAYER_KHRONOS_validation").unwrap()
            ];
            let layer_name_pointers: Vec<*const c_char> = layer_names
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect();

            // Create the instance
            let instance_create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&extension_name_pointers)
                .enabled_layer_names(&layer_name_pointers);
            entry
                .create_instance(&instance_create_info, None)
                .map_err(|e| format!("Instance creation failed: {:?}", e))?
        };

        // Create debug messenger
        let debug_utils = ash::extensions::ext::DebugUtils::new(entry, &instance);
        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        };
        let utils_messenger = debug_utils
            .create_debug_utils_messenger(&debug_create_info, None)
            .map_err(|e| format!("Debug messenger creation failed: {:?}", e))?;

        // Create surface and surface loader, and chosoe a compatible physical device
        let surface_fn = Surface::new(entry, &instance);
        let surface = RenderCore::make_new_surface(entry, &instance, window_owner);
        let physical_device_properties = RenderCore::select_physical_device(&instance, &surface_fn, &surface)?;
        let physical_device = physical_device_properties.physical_device;

        // Find queue indices for graphics and transfer (ideally different but could be the same)
        let queue_family_properties = instance
            .get_physical_device_queue_family_properties(physical_device);
        let (graphics_queue_family_index, transfer_queue_family_index) = {
            let mut found_graphics_queue_index = None;
            let mut found_transfer_queue_index = None;
            for (index, queue_family) in queue_family_properties.iter().enumerate() {
                if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    found_graphics_queue_index = Some(index as u32);
                }
                if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                    if found_transfer_queue_index.is_none() || !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        found_transfer_queue_index = Some(index as u32);
                    }
                }
            }
            (
                found_graphics_queue_index.unwrap(),
                found_transfer_queue_index.unwrap()
            )
        };

        // Device extensions required
        let device_extensions: Vec<*const c_char> = vec![ Swapchain::name().as_ptr() ];

        // NOTE: Device validation layers are deprecated and usually the layers enabled on the
        // instance will cover all API calls
        let layer_names: Vec<CString> = vec![
            CString::new("VK_LAYER_KHRONOS_validation").unwrap()
        ];
        let layer_name_pointers: Vec<*const c_char> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        // Make the logical device
        let priorities = [1.0f32];
        let queue_infos = [
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_queue_family_index)
                .queue_priorities(&priorities)
                .build(),
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(transfer_queue_family_index)
                .queue_priorities(&priorities)
                .build()
        ];
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layer_name_pointers)
            .enabled_extension_names(&device_extensions);
        let device = instance
            .create_device(physical_device, &device_create_info, None)
            .map_err(|e| format!("{:?}", e))?;

        // Get queues
        let graphics_queue = device.get_device_queue(graphics_queue_family_index, 0);
        let transfer_queue = device.get_device_queue(transfer_queue_family_index, 0);

        // Create a memory allocator
        let allocator_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: device.clone(),
            instance: instance.clone(),
            flags: AllocatorCreateFlags::NONE,
            preferred_large_heap_block_size: 0,
            frame_in_use_count: 0,
            heap_size_limits: None
        };
        let mem_allocator = vk_mem::Allocator::new(&allocator_info)
            .map_err(|e| format!("{:?}", e))?;

        // One command buffer pool per queue family
        let graphics_pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let graphics_command_buffer_pool = device
            .create_command_pool(&graphics_pool_info, None)
            .map_err(|e| format!("{:?}", e))?;
        let transfer_pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(transfer_queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let transfer_command_buffer_pool = device
            .create_command_pool(&transfer_pool_info, None)
            .map_err(|e| format!("{:?}", e))?;

        // Synchronisation objects
        let mut sync_image_available = vec![];
        let mut sync_may_begin_rendering = vec![];
        let mut sync_rendering_finished = vec![];
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..PROJ_VK_SWAPCHAIN_SIZE {
            let semaphore_available = device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| format!("{:?}", e))?;
            let fence_begin_rendering = device
                .create_fence(&fence_create_info, None)
                .map_err(|e| format!("{:?}", e))?;
            let semaphore_finished = device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| format!("{:?}", e))?;
            sync_image_available.push(semaphore_available);
            sync_may_begin_rendering.push(fence_begin_rendering);
            sync_rendering_finished.push(semaphore_finished);
        }

        let swapchain_fn = Swapchain::new(&instance, &device);

        Ok(
            RenderCore {
                instance,
                utils_messenger,
                debug_utils,
                device,
                physical_device_properties,
                mem_allocator,
                graphics_queue,
                transfer_queue,
                graphics_command_buffer_pool,
                transfer_command_buffer_pool,
                sync_image_available,
                sync_may_begin_rendering,
                sync_rendering_finished,
                current_image_acquired: PROJ_VK_SWAPCHAIN_SIZE - 1,
                surface_fn,
                surface,
                swapchain_fn,
                swapchain: vk::SwapchainKHR::null(),
                image_views: vec![],
                depth_image: None,
                vbo_objects: HashMap::new(),
                texture_objects: HashMap::new()
            }
        )
    }

    pub unsafe fn load_new_resources(&mut self, resource_preloads: &ResourcePreloads) -> Result<(), String> {

        // VBOs
        for (vbo_index, creation_data) in resource_preloads.vbo_preloads.iter() {
            let vertex_size_bytes: usize = match creation_data.vertex_format {
                VertexFormat::PositionNormalTexture => 32
            };
            let buffer = {
                let mut buffer = BufferWrapper::new_vertex_buffer(
                    &self.mem_allocator,
                    creation_data.vertex_count * vertex_size_bytes)?;
                buffer.update_from_vec(&self.mem_allocator, &creation_data.vertex_data)?;
                buffer
            };
            self.vbo_objects.insert(*vbo_index, (creation_data.vertex_count, buffer));
        }

        // Textures
        for (texture_index, creation_data) in resource_preloads.texture_preloads.iter() {
            let texture = ImageWrapper::new_initialised_texture_image_rgba(
                self,
                creation_data.width,
                creation_data.height,
                &creation_data.data)?;
            self.texture_objects.insert(*texture_index, texture);
        }

        Ok(())
    }

    unsafe fn select_physical_device(
            instance: &ash::Instance,
            surface_loader: &ash::extensions:: khr::Surface,
            surface: &vk::SurfaceKHR
            ) -> Result<PhysicalDeviceProperties, String> {

        let physical_devices = instance
            .enumerate_physical_devices()
            .map_err(|e| format!("{:?}", e))?;
        if physical_devices.is_empty() {
            return Err(String::from("No physical devices found"));
        }

        let unset_value: u32 = u32::MAX;
        for physical_device in physical_devices.iter() {
            let queue_family_properties = instance.get_physical_device_queue_family_properties(*physical_device);
            let mut graphics_index: u32 = unset_value;
            let mut transfer_index: u32 = unset_value;
            for (index, properties) in queue_family_properties.iter().enumerate() {
                let supports_graphics = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let supports_surface = surface_loader
                    .get_physical_device_surface_support(*physical_device, index as u32, *surface)
                    .unwrap();
                let supports_transfer = properties.queue_flags.contains(vk::QueueFlags::TRANSFER);

                let graphics_and_surface = supports_graphics && supports_surface;
                if graphics_and_surface {
                    graphics_index = index as u32;
                }
                if supports_transfer && (transfer_index == unset_value || !graphics_and_surface) {
                    transfer_index = index as u32;
                }
            }
            if graphics_index != unset_value && transfer_index != unset_value {
                return Ok(PhysicalDeviceProperties {
                    physical_device: *physical_device,
                    graphics_queue_family_index: graphics_index,
                    transfer_queue_family_index: transfer_index
                });
            }
        }

        Err(String::from("Could not find a suitable physical device"))
    }

    unsafe fn destroy_all_resources(&mut self) {
        for (_key, (_, buffer)) in self.vbo_objects.iter() {
            buffer.destroy(&self.mem_allocator).unwrap();
        }
        for (_key, image) in self.texture_objects.iter() {
            image.destroy(&self.device, &self.mem_allocator).unwrap();
        }
    }

    pub unsafe fn query_vbo(&self, index: usize) -> Result<(usize, vk::Buffer), String> {
        match self.vbo_objects.get(&index) {
            Some((vertex_count, buffer)) => Ok((*vertex_count, buffer.buffer)),
            None => Err(String::from("Queried VBO that is not loaded"))
        }
    }

    pub unsafe fn query_texture(&self, index: usize) -> Result<vk::ImageView, String> {
        match self.texture_objects.get(&index) {
            Some(texture) => Ok(texture.image_view),
            None => Err(String::from("Queried texture that is not loaded"))
        }
    }

    unsafe fn make_new_surface(entry: &Entry, instance: &Instance, window_owner: &dyn HasRawWindowHandle) -> vk::SurfaceKHR {
        ash_window::create_surface(entry, instance, window_owner, None).unwrap()
    }

    pub unsafe fn destroy_swapchain(&mut self) {
        if let Some(image) = &self.depth_image {
            image.destroy(&self.device, &self.mem_allocator).unwrap();
        }
        for image_view in self.image_views.iter_mut() {
            self.device.destroy_image_view(*image_view, None);
        }
        self.swapchain_fn.destroy_swapchain(self.swapchain, None);
    }

    pub unsafe fn destroy_surface(&mut self) {
        self.surface_fn.destroy_surface(self.surface, None);
    }

    pub unsafe fn create_surface(&mut self, entry: &Entry, window_owner: &dyn HasRawWindowHandle) {
        self.surface = RenderCore::make_new_surface(entry, &self.instance, window_owner);
    }

    pub unsafe fn create_swapchain(&mut self) -> Result<(), String> {

        // Make the swapchain; automatically creates the images (but not image views)
        let surface_capabilities = self.surface_fn
            .get_physical_device_surface_capabilities(self.physical_device_properties.physical_device, self.surface)
            .map_err(|e| format!("{:?}", e))?;
        let swapchain = {
            let surface_present_modes = self.surface_fn
                .get_physical_device_surface_present_modes(self.physical_device_properties.physical_device, self.surface)
                .map_err(|e| format!("{:?}", e))?;
            let surface_formats = self.surface_fn
                .get_physical_device_surface_formats(self.physical_device_properties.physical_device, self.surface)
                .map_err(|e| format!("{:?}", e))?;
            let present_supported = self.surface_fn
                .get_physical_device_surface_support(self.physical_device_properties.physical_device, self.physical_device_properties.graphics_queue_family_index, self.surface)
                .map_err(|e| format!("{:?}", e))?;
            if !present_supported {
                return Err(String::from("Presentation not supported by selected graphics queue family"));
            }
            if !surface_present_modes.contains(&vk::PresentModeKHR::FIFO) {
                return Err(String::from("FIFO presentation mode not supported by selected graphics queue family"));
            }

            if surface_capabilities.min_image_count > PROJ_VK_SWAPCHAIN_SIZE as u32 || surface_capabilities.max_image_count < PROJ_VK_SWAPCHAIN_SIZE as u32 {
                return Err(String::from("Requested swapchain size is not supported"));
            }

            let surface_format = surface_formats.first().unwrap();
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(self.surface)
                .min_image_count(PROJ_VK_SWAPCHAIN_SIZE as u32)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_capabilities.current_extent)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(surface_capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true)
                .image_array_layers(1);

            self.swapchain_fn.create_swapchain(&swapchain_create_info, None)
                .map_err(|e| format!("{:?}", e))?
        };

        // Make the image views over the images
        let swapchain_images = self.swapchain_fn.get_swapchain_images(swapchain)
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
            let image_view = self.device.create_image_view(&image_view_create_info, None)
                .map_err(|e| format!("{:?}", e))?;
            image_views.push(image_view);
        }

        if image_views.len() != self.sync_image_available.len() {
            return Err(format!("Recreated swapchain has {} images; had {} before", image_views.len(), self.sync_image_available.len()));
        }

        self.swapchain = swapchain;
        self.image_views.clear();
        for image in image_views.iter() {
            self.image_views.push(*image);
        }

        let extent = self.get_extent()?;
        let depth_image = ImageWrapper::new_depth_image(
            &self,
            extent.width as u32,
            extent.height as u32)?;

        self.depth_image = Some(depth_image);

        Ok(())
    }

    fn get_window_extensions(window_owner: &dyn HasRawWindowHandle) -> Result<Vec<*const c_char>, String> {
        let extensions_as_c_str = ash_window::enumerate_required_extensions(window_owner)
            .map_err(|e| format!("{:?}", e))?
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<*const c_char>>();
        Ok(extensions_as_c_str)
    }

    pub unsafe fn get_surface_formats(&self) -> Result<Vec<vk::SurfaceFormatKHR>, String> {
        self.surface_fn.get_physical_device_surface_formats(self.physical_device_properties.physical_device, self.surface)
            .map_err(|e| format!("{:?}", e))
    }

    pub fn get_extent(&self) -> Result<vk::Extent2D, String> {
        let surface_capabilities = unsafe {
            self.surface_fn.get_physical_device_surface_capabilities(self.physical_device_properties.physical_device, self.surface)
                .map_err(|e| format!("{:?}", e))?
        };
        Ok(surface_capabilities.current_extent)
    }

    pub fn get_depth_image(&self) -> Option<&ImageWrapper> {
        match &self.depth_image {
            Some(image) => Some(image),
            _ => None
        }
    }

    pub fn wait_until_idle(&self) -> Result<(), String> {
        unsafe {
            self.device.device_wait_idle()
                .map_err(|e| format!("Waiting device idle error: {:?}", e))?;
        }
        Ok(())
    }

    pub fn get_mem_allocator(&self) -> &vk_mem::Allocator {
        &self.mem_allocator
    }

    // Increment current image number to focus on the next image in the chain, to wait for its
    // synchronisation objects and so on.
    //
    // Acquires an image while signalling a semaphore, then waits on a fence to know that the
    // image is available to draw on.
    pub unsafe fn acquire_next_image(&mut self) -> Result<usize, String> {
        let sync_objects_index = (self.current_image_acquired + 1) % (self.image_views.len());
        let (image_index, _) = self.swapchain_fn.acquire_next_image(
            self.swapchain,
            std::u64::MAX,
            self.sync_image_available[sync_objects_index],
            vk::Fence::null()
            ).map_err(|e| format!("Image acquire failure: {:?}", e))?;
        self.current_image_acquired = image_index as usize;
        assert_eq!(sync_objects_index, image_index as usize);

        self.device.wait_for_fences(
            &[self.sync_may_begin_rendering[self.current_image_acquired]],
            true,
            std::u64::MAX
            ).map_err(|e| format!("Waiting on fence error: {:?}", e))?;
        self.device.reset_fences(&[self.sync_may_begin_rendering[self.current_image_acquired]])
            .map_err(|e| format!("Resetting fence error: {:?}", e))?;

        Ok(self.current_image_acquired)
    }

    pub unsafe fn submit_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), String> {
        let semaphores_available = [self.sync_image_available[self.current_image_acquired]];
        let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let semaphores_finished = [self.sync_rendering_finished[self.current_image_acquired]];
        let command_buffers = [command_buffer];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&semaphores_finished)
            .build()];
        self.device.queue_submit(self.graphics_queue, &submit_info, self.sync_may_begin_rendering[self.current_image_acquired])
            .map_err(|e| format!("Queue submit error: {:?}", e))?;
        Ok(())
    }

    pub unsafe fn present_image(&self) -> Result<PresentResult, String> {
        let semaphores_finished = [self.sync_rendering_finished[self.current_image_acquired]];
        let swapchains = [self.swapchain];
        let indices = [self.current_image_acquired as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores_finished)
            .swapchains(&swapchains)
            .image_indices(&indices);
        return match self.swapchain_fn.queue_present(self.graphics_queue, &present_info) {
            Ok(_) => Ok(PresentResult::Ok),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                println!("YEEEEE!");
                Ok(PresentResult::SwapchainOutOfDate)
            },
            Err(e) => Err(format!("Present error: {:?}", e))
        };
    }
}
