use serde::Deserialize;
use crate::deserialiser::types::Resources;

/// App struct
/// Defines some top-level properties of the application
#[derive(Debug, Deserialize)]
pub struct App {
    pub name: String,
    pub features: Vec<AppFeature>,
    pub platform: AppPlatform,
    pub graphics: AppGraphicsApi,
    pub start_scene_id: String,
    pub shared_resources: Resources
}

#[derive(Debug, Deserialize)]
pub enum AppFeature {
    clip_planes
}

#[derive(Debug, Deserialize)]
pub enum AppPlatform {
    windows
}

#[derive(Debug, Deserialize)]
pub enum AppGraphicsApi {
    vulkan
}
