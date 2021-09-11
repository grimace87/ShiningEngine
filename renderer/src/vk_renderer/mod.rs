
pub mod buffers;
pub mod images;
mod render_core;
mod per_image_resources;

use crate::vk_renderer::{
    render_core::RenderCore,
    per_image_resources::PerImageResources
};

use defs::{
    EngineError,
    SceneInfo,
    render::{
        RendererApi,
        PresentResult,
        ResourcePreloads,
        DrawingDescription,
        FeatureDeclaration
    }
};

use ash::Entry;
use raw_window_handle::HasRawWindowHandle;

pub struct VkRenderer {
    function_loader: Entry,
    render_core: RenderCore,
    per_image_resources: Vec<PerImageResources>
}

impl RendererApi for VkRenderer {

    fn new(window_owner: &dyn HasRawWindowHandle, features: &[FeatureDeclaration], resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<Self, EngineError> {

        // Vulkan core - instance, device, swapchain, queues, command pools
        let entry = unsafe {
            Entry::new().map_err(|e| {
                EngineError::RenderError(format!("Entry creation failed: {:?}", e))
            })?
        };
        let render_core = RenderCore::new(&entry, window_owner, features, resource_preloads)?;

        // Per-swapchain-image resources - command buffers, whatever pipelines, buffers etc. are required
        let command_buffers = unsafe {
            render_core.regenerate_command_buffers()?
        };
        let mut per_image_resources = vec![];
        assert_eq!(command_buffers.len(), render_core.image_views.len());
        for (swapchain_image_index, command_buffer) in command_buffers.iter().enumerate() {
            let resources = PerImageResources::new(&render_core, swapchain_image_index, description, *command_buffer)?;
            unsafe {
                resources.record_command_buffer(&render_core, description, *command_buffer)?;
            }
            per_image_resources.push(resources);
        }

        Ok(VkRenderer {
            function_loader: entry,
            render_core,
            per_image_resources
        })
    }

    fn draw_next_frame(&mut self, scene_info: &dyn SceneInfo) -> Result<PresentResult, EngineError> {
        unsafe {
            let (swapchain_image_index, up_to_date) = self.render_core.acquire_next_image()?;
            if !up_to_date {
                return Ok(PresentResult::SwapchainOutOfDate);
            }
            self.per_image_resources[swapchain_image_index].on_pre_render(&mut self.render_core, scene_info);
            let command_buffer = self.per_image_resources[swapchain_image_index].get_command_buffer();
            self.render_core.submit_command_buffer(command_buffer)?;
            self.render_core.present_image()
        }
    }

    fn recreate_surface(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), EngineError> {
        self.render_core.wait_until_idle().unwrap();

        for resources in self.per_image_resources.iter_mut() {
            resources.destroy_resources(&self.render_core);
        }
        self.per_image_resources.clear();

        let command_buffers = unsafe {
            self.render_core.regenerate_command_buffers()?
        };

        unsafe {
            self.render_core.recreate_surface(&self.function_loader, window_owner)?;
            assert_eq!(command_buffers.len(), self.render_core.image_views.len());
            for (swapchain_image_index, command_buffer) in command_buffers.iter().enumerate() {
                let resources = PerImageResources::new(&self.render_core, swapchain_image_index, description, *command_buffer)?;
                resources.record_command_buffer(&self.render_core, description, *command_buffer)?;
                self.per_image_resources.push(resources);
            }
        }
        Ok(())
    }

    fn recreate_scene_resources(&mut self, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<(), EngineError> {
        self.render_core.wait_until_idle().unwrap();
        unsafe {
            self.render_core.load_new_resources(resource_preloads).unwrap();
        }

        for resources in self.per_image_resources.iter_mut() {
            resources.destroy_resources(&self.render_core);
        }
        self.per_image_resources.clear();

        let command_buffers = unsafe {
            self.render_core.regenerate_command_buffers()?
        };

        assert_eq!(command_buffers.len(), self.render_core.image_views.len());
        for (swapchain_image_index, command_buffer) in command_buffers.iter().enumerate() {
            let resources = PerImageResources::new(&self.render_core, swapchain_image_index, description, *command_buffer)?;
            unsafe {
                resources.record_command_buffer(&self.render_core, description, *command_buffer)?;
            }
            self.per_image_resources.push(resources);
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
