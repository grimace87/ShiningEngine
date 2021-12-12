
mod app;
mod scene_regenerated_top_level;
mod scene_regenerated_struct;
mod scene_regenerated_preloads;
mod scene_regenerated_description;
mod scene_regenerated;
mod scene_starter;

use app::generate_app_root_content;
use scene_regenerated::generate_regenerated_scene_contents;
use scene_starter::generate_starter_scene_contents;
use crate::deserialiser::app::App;
use crate::deserialiser::scene::Scene;
use crate::GeneratorError;

/// Generate stubs for an app spec as a String of content. Should be saved to src/app.rs.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_app_stubs(config: &App) -> Result<String, GeneratorError> {
    let app_content = generate_app_root_content(config)?;
    Ok(app_content)
}

/// Generate stubs for a scene two Strings of content - one which should be saved to
/// src/scenes/<scene_name>/mod.rs, and another that should be saved to
/// src/scenes/<scene_name>/details.rs but only if that file does not yet exist.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_scene_stubs(config: &Scene) -> Result<(String, String), GeneratorError> {
    let regenerated_scene_contents = generate_regenerated_scene_contents(config)?;
    let starter_scene_contents = generate_starter_scene_contents(config)?;
    Ok((regenerated_scene_contents, starter_scene_contents))
}
