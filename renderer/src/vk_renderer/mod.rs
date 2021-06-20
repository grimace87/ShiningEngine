
pub mod buffers;
pub mod images;
pub mod framebuffers;
mod render_core;
mod renderpass;
mod pipeline;
mod pipeline_set;

use self::{
    render_core::RenderCore,
    renderpass::RenderpassWrapper,
    pipeline_set::PipelineSet
};

use defs::{RendererApi, PresentResult, DrawingDescription, SceneInfo, ResourcePreloads};

use ash::Entry;
use raw_window_handle::HasRawWindowHandle;

pub struct VkRenderer {
    function_loader: Entry,
    render_core: RenderCore,
    renderpasses: Vec<RenderpassWrapper>,
    pipelines: Vec<PipelineSet>
}

impl RendererApi for VkRenderer {

    fn new(window_owner: &dyn HasRawWindowHandle, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<Self, String> {
        let entry = unsafe {
            Entry::new().map_err(|e| format!("Entry creation failed: {:?}", e))?
        };
        let render_core = RenderCore::new(&entry, window_owner, resource_preloads)?;
        let renderpasses: Vec<RenderpassWrapper> = description.passes.iter()
            .map(|pass| RenderpassWrapper::new(&render_core, &pass.target).unwrap())
            .collect();
        let pipelines = description.passes.iter()
            .enumerate()
            .map(|(i, pass)| PipelineSet::new(&render_core, &renderpasses[i], pass).unwrap())
            .collect();

        Ok(VkRenderer {
            function_loader: entry,
            render_core,
            renderpasses,
            pipelines
        })
    }

    // TODO - Build command buffers such that all renderpasses are used, not just that represented by pipelines[0]
    fn draw_next_frame(&mut self, scene_info: &dyn SceneInfo) -> Result<PresentResult, String> {
        unsafe {
            let image_index = self.render_core.acquire_next_image()?;
            self.pipelines[0].update_uniform_buffer(&mut self.render_core, scene_info).unwrap();
            let command_buffer = self.pipelines[0].get_command_buffer(image_index);
            self.render_core.submit_command_buffer(command_buffer)?;
            return self.render_core.present_image();
        }
    }

    fn recreate_swapchain(&mut self, window_owner: &dyn HasRawWindowHandle, description: &DrawingDescription) -> Result<(), String> {
        self.render_core.wait_until_idle().unwrap();

        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(&self.render_core);
        }
        for renderpass in self.renderpasses.iter_mut() {
            renderpass.destroy_resources(&self.render_core);
        }
        unsafe {
            self.render_core.destroy_swapchain();
            self.render_core.create_swapchain(&self.function_loader, window_owner)?;
            for (i, renderpass) in self.renderpasses.iter_mut().enumerate() {
                renderpass.create_resources(&self.render_core, &description.passes[i].target)?;
                self.pipelines[i].create_resources(&self.render_core, renderpass, &description.passes[i])?;
            }
        }
        Ok(())
    }

    fn recreate_scene_resources(&mut self, resource_preloads: &ResourcePreloads, description: &DrawingDescription) -> Result<(), String> {
        self.render_core.wait_until_idle().unwrap();
        unsafe {
            self.render_core.load_new_resources(resource_preloads).unwrap();
        }

        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(&self.render_core);
        }
        for renderpass in self.renderpasses.iter_mut() {
            renderpass.destroy_resources(&self.render_core);
        }
        unsafe {
            for (i, renderpass) in self.renderpasses.iter_mut().enumerate() {
                renderpass.create_resources(&self.render_core, &description.passes[i].target)?;
                self.pipelines[i].create_resources(&self.render_core, renderpass, &description.passes[i])?;
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
        for pipeline in self.pipelines.iter_mut() {
            pipeline.destroy_resources(&self.render_core);
        }
        for renderpass in self.renderpasses.iter_mut() {
            renderpass.destroy_resources(&self.render_core);
        }
    }
}
