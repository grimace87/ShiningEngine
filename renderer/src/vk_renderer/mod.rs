
pub mod buffers;
pub mod images;
mod render_core;
mod renderpass;
mod pipeline_config;
mod pipeline;
mod pipeline_set;

use self::{
    render_core::RenderCore,
    renderpass::RenderpassWrapper,
    pipeline_set::PipelineSet
};

use defs::{RendererApi, PresentResult};

use ash::Entry;
use raw_window_handle::HasRawWindowHandle;
use cgmath::Matrix4;

pub struct VkRenderer {
    function_loader: Entry,
    render_core: RenderCore,
    renderpass: RenderpassWrapper,
    pipelines: PipelineSet
}

impl RendererApi for VkRenderer {
    fn new(window_owner: &dyn HasRawWindowHandle) -> Result<Self, String> {
        let entry = unsafe {
            Entry::new().map_err(|e| format!("Entry creation failed: {:?}", e))?
        };
        let render_core = RenderCore::new(&entry, window_owner)?;
        let renderpass = RenderpassWrapper::new(&render_core)?;
        let pipelines = PipelineSet::new(&render_core, &renderpass)?;

        Ok(VkRenderer {
            function_loader: entry,
            render_core,
            renderpass,
            pipelines
        })
    }

    fn draw_next_frame(&mut self, camera_matrix: Matrix4<f32>) -> Result<PresentResult, String> {
        unsafe {
            let image_index = self.render_core.acquire_next_image()?;
            self.pipelines.update_camera_matrix(&mut self.render_core, camera_matrix).unwrap();
            let command_buffer = self.pipelines.get_command_buffer(image_index);
            self.render_core.submit_command_buffer(command_buffer)?;
            return self.render_core.present_image();
        }
    }

    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle) -> Result<(), String> {
        self.render_core.wait_until_idle().unwrap();
        self.pipelines.destroy_resources(&self.render_core);
        self.renderpass.destroy_resources(&self.render_core);
        unsafe {
            self.render_core.destroy_swapchain();
            self.render_core.create_swapchain(&self.function_loader, window_owner)?;
            self.renderpass.create_resources(&self.render_core)?;
            self.pipelines.create_resources(&self.render_core, &self.renderpass)?;
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
        self.pipelines.destroy_resources(&self.render_core);
        self.renderpass.destroy_resources(&self.render_core);
    }
}
