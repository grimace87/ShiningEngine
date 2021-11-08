use crate::deserialiser::app::*;
use crate::deserialiser::scene::*;
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
    let scene_contents = generate_scene_contents(config)?;
    Ok(scene_contents)
}

fn generate_app_root_content(config: &App) -> Result<String, GeneratorError> {

    let start_scene = config.start_scene_id.as_str();

    let use_platform: &str = match config.platform {
        AppPlatform::windows => "use platform_windows::PlatformWindows;"
    };
    let use_graphics: &str = match config.graphics {
        AppGraphicsApi::vulkan => "use renderer::vk_renderer::VkRenderer;"
    };

    let title_def: String = format!("const APP_TITLE: &str = \"{}\";", config.name);

    let platform_construct: &str = match config.platform {
        AppPlatform::windows => "PlatformWindows::new_window(APP_TITLE)"
    };

    let engine_decl: &str = match config.graphics {
        AppGraphicsApi::vulkan => "let engine: Engine<VkRenderer>"
    };

    let content = format!("
mod scenes;

use scenes::{};

{}
{}
use engine::Engine;
use defs::render::FeatureDeclaration;

{}

fn main() {{

    let mut platform = {}
        .unwrap_or_else(|e| {{
            println!(\"Error creating window: {{:?}}\", e);
            std::process::exit(1);
        }});

    {} = Engine::new_uninitialised(
        Box::from(SceneryScene::new()),
        vec![FeatureDeclaration::ClipPlanes]);

    platform.run(engine)
        .unwrap_or_else(|e| {{
            println!(\"Error while running: {{:?}}\", e);
            std::process::exit(1);
        }});
}}", start_scene, use_platform, use_graphics, title_def, platform_construct, engine_decl);
    Ok(content)
}

fn generate_scene_contents(_config: &Scene) -> Result<(String, String), GeneratorError> {
    let forced_gen_content = "Hello!".to_string();
    let only_when_missing_gen_content = "Hello!".to_string();
    Ok((forced_gen_content, only_when_missing_gen_content))
}
