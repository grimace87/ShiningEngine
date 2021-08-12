
use crate::vk_renderer::{
    render_core::RenderCore,
    per_image_resources::renderpass::RenderpassWrapper,
    per_image_resources::PipelineSet
};
use defs::DrawingPass;

pub struct PerPassResources {
    pub renderpass: RenderpassWrapper,
    pub renderpass_pipeline_set: PipelineSet
}

impl PerPassResources {
    pub fn new(render_core: &RenderCore, swapchain_image_index: usize, pass: &DrawingPass) -> Result<PerPassResources, String> {
        let renderpass = RenderpassWrapper::new(&render_core, swapchain_image_index, &pass.target)?;
        let renderpass_pipeline_set = PipelineSet::new(&render_core, &renderpass, pass)?;
        Ok(PerPassResources {
            renderpass,
            renderpass_pipeline_set
        })
    }
}
