
mod app_module;
mod app_regenerated;
mod scenes_list_module;
mod scenes_list_regenerated;
mod scene_regenerated_top_level;
mod scene_regenerated_struct;
mod scene_regenerated_preloads;
mod scene_regenerated_description;
mod scene_regenerated;
mod scene_starter;

use app_module::generate_app_root_content;
use app_regenerated::generate_app_regenerated_content;
use scenes_list_module::generate_scenes_list_root_content;
use scenes_list_regenerated::generate_scenes_list_regenerated_content;
use scene_regenerated::generate_regenerated_scene_contents;
use scene_starter::generate_starter_scene_contents;
use crate::deserialiser::types::Resources;
use crate::deserialiser::types::app::App;
use crate::deserialiser::types::scene::Scene;
use crate::GeneratorError;

/// Generate stubs for an app spec as a String of content. Should be saved to src/app.rs.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_app_stubs(config: &App) -> Result<(String, String), GeneratorError> {
    let app_module_content = generate_app_root_content();
    let app_generated_content = generate_app_regenerated_content(config)?;
    Ok((app_module_content, app_generated_content))
}

/// Generate stubs for an scenes module as a String of content. Should be saved to src/scenes/mod.rs.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_scene_listing_stubs(configs: &Vec<Scene>) -> Result<(String, String), GeneratorError> {
    let scenes_list_module_content = generate_scenes_list_root_content();
    let scenes_list_generated_content = generate_scenes_list_regenerated_content(configs)?;
    Ok((scenes_list_module_content, scenes_list_generated_content))
}

/// Generate stubs for a scene two Strings of content - one which should be saved to
/// src/scenes/<scene_name>/mod.rs, and another that should be saved to
/// src/scenes/<scene_name>/details.rs but only if that file does not yet exist.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_scene_stubs(
    scene_number_one_based: usize,
    shared_resources: &Resources,
    config: &Scene,
    resources_dir_name: &'static str
) -> Result<(String, String), GeneratorError> {
    let scene_module_contents = generate_starter_scene_contents(config)?;
    let scene_generated_contents = generate_regenerated_scene_contents(scene_number_one_based, shared_resources, config, resources_dir_name)?;
    Ok((scene_module_contents, scene_generated_contents))
}
