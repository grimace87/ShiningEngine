
pub mod buffers;
pub mod images;
pub mod framebuffers;
mod render_core;
mod renderpass;
mod pipeline;
mod pipeline_set;
mod per_image_resources;

use crate::vk_renderer::{
    render_core::RenderCore,
    per_image_resources::PerImageResources
};

use defs::{RendererApi, PresentResult, DrawingDescription, SceneInfo, ResourcePreloads};

use ash::vk;
use ash::Entry;
use raw_window_handle::HasRawWindowHandle;
use ash::version::DeviceV1_0;

pub struct VkRenderer {
    function_loader: Entry,
    render_core: RenderCore,
    per_image_resources: Vec<PerImageResources>
}

impl RendererApi for VkRenderer {

    fn new(window_owner: &dyn HasRawWindowHandle, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<Self, String> {

        // Vulkan core - instance, device, swapchain, queues, command pools
        let entry = unsafe {
            Entry::new().map_err(|e| format!("Entry creation failed: {:?}", e))?
        };
        let render_core = RenderCore::new(&entry, window_owner, resource_preloads)?;

        // Per-swapchain-image resources - command buffers, whatever pipelines, buffers etc. are required
        let command_buffer_count = render_core.image_views.len() as u32;
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(render_core.graphics_command_buffer_pool)
            .command_buffer_count(command_buffer_count);
        let command_buffers = unsafe {
            render_core.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .map_err(|e| format!("{:?}", e))?
        };
        let mut per_image_resources = vec![];
        let swapchain_image_count = render_core.image_views.len();
        for image_index in 0..swapchain_image_count {
            let resources = PerImageResources::new(&render_core, image_index, description, command_buffers[image_index]);
            per_image_resources.push(resources);
        }

        Ok(VkRenderer {
            function_loader: entry,
            render_core,
            per_image_resources
        })
    }

    // TODO - Build command buffers such that all renderpasses are used, not just that represented by pipelines[0]
    fn draw_next_frame(&mut self, scene_info: &dyn SceneInfo) -> Result<PresentResult, String> {
        unsafe {
            let image_index = self.render_core.acquire_next_image()?;
            self.per_image_resources[image_index].on_pre_render(&mut self.render_core, scene_info);
            let command_buffer = self.per_image_resources[image_index].get_command_buffer();
            self.render_core.submit_command_buffer(command_buffer)?;
            return self.render_core.present_image();
        }
    }

    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), String> {
        self.render_core.wait_until_idle().unwrap();

        for resources in self.per_image_resources.iter_mut() {
            resources.destroy_resources(&self.render_core);
        }
        unsafe {
            self.render_core.destroy_swapchain();
            self.render_core.destroy_surface();
            self.render_core.create_surface(&self.function_loader, window_owner);
            self.render_core.create_swapchain()?;
            for (image_index, resources) in self.per_image_resources.iter_mut().enumerate() {
                resources.create_resources(&self.render_core, image_index, description)?;
            }
        }
        Ok(())
    }

    fn recreate_scene_resources(&mut self, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<(), String> {
        self.render_core.wait_until_idle().unwrap();
        unsafe {
            self.render_core.load_new_resources(resource_preloads).unwrap();
        }

        for resources in self.per_image_resources.iter_mut() {
            resources.destroy_resources(&self.render_core);
        }
        unsafe {
            for (image_index, resources) in self.per_image_resources.iter_mut().enumerate() {
                resources.create_resources(&self.render_core, image_index, description)?;
            }
        }
        Ok(())
    }

    fn get_aspect_ratio(&self) -> f32 {
        if let Ok(extent) = self.render_core.get_extent() {
            extent.width as f32 / extent.height as f32
        } else {
            1.0
        }
    }
}

impl Drop for VkRenderer {
    fn drop(&mut self) {
        self.render_core.wait_until_idle().unwrap();
        for resources in self.per_image_resources.iter_mut() {
            resources.destroy_resources(&self.render_core);
        }
    }
}
