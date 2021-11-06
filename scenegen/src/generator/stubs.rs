use std::path::PathBuf;
use crate::deserialiser::structures::{App, Scene};
use crate::generator::writer::WritableFile;
use crate::GeneratorError;

/// Generate stubs for an app spec as a String of content with a relative file path to save it to.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_app_stubs(src_file: &PathBuf, config: &App) -> Result<WritableFile, GeneratorError> {
    let root_content = generate_app_root_content(src_file, config)?;
    Ok(root_content)
}

/// Generate stubs for a scene as a String of content with a relative file path to save it to.
/// To start this process, call crate::generator::writer::process_spec_path from a build script.
pub fn generate_scene_stubs(src_file: &PathBuf, config: &Scene) -> Result<WritableFile, GeneratorError> {
    // TODO - Scene, not app
    let root_content = generate_scene_content(src_file, config)?;
    Ok(root_content)
}

fn generate_app_root_content(src_file: &PathBuf, config: &App) -> Result<WritableFile, GeneratorError> {

    let start_scene = config.start_scene.as_str();

    let use_platform: &str = match config.platform.as_str() {
        "windows" => "use platform_windows::PlatformWindows;",
        _ => return Err(GeneratorError::InvalidSchema(src_file.clone(), "Bad platform".to_string()))
    };
    let use_graphics: &str = match config.graphics.as_str() {
        "vulkan" => "use renderer::vk_renderer::VkRenderer;",
        _ => return Err(GeneratorError::InvalidSchema(src_file.clone(), "Bad graphics".to_string()))
    };

    let title_def: String = format!("const APP_TITLE: &str = \"{}\";", config.name);

    let platform_construct: &str = match config.platform.as_str() {
        "windows" => "PlatformWindows::new_window(APP_TITLE)",
        _ => return Err(GeneratorError::InvalidSchema(src_file.clone(), "Bad platform".to_string()))
    };

    let engine_decl: &str = match config.graphics.as_str() {
        "vulkan" => "let engine: Engine<VkRenderer>",
        _ => return Err(GeneratorError::InvalidSchema(src_file.clone(), "Bad graphics".to_string()))
    };

    let content = format!("
mod scenes;

use scene::{};

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
    let relative_path = {
        let mut path = PathBuf::new();
        path.push("src");
        path.push("app.rs");
        path
    };
    Ok(WritableFile { relative_path, content })
}

fn generate_scene_content(src_file: &PathBuf, _config: &Scene) -> Result<WritableFile, GeneratorError> {
    let relative_path = {
        let mut path = PathBuf::new();
        path.push("src");
        path.push("scenes");
        path.push(src_file.file_stem().unwrap().to_str().unwrap());
        path.push("mod.rs");
        path
    };
    let content = "Hello!".to_string();
    Ok(WritableFile { relative_path, content })
}
