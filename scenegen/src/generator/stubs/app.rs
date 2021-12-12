
use crate::deserialiser::app::*;
use crate::GeneratorError;

pub fn generate_app_root_content(config: &App) -> Result<String, GeneratorError> {

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
