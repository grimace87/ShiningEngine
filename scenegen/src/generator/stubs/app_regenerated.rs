
use crate::deserialiser::types::app::*;
use crate::GeneratorError;
use heck::CamelCase;

pub fn generate_app_regenerated_content(config: &App) -> Result<String, GeneratorError> {

    let start_scene = config.start_scene_id.as_str();
    let struct_name = format!("{}Scene", config.start_scene_id.to_camel_case());

    let use_platform: &str = match config.platform {
        AppPlatform::windows => "use platform_windows::PlatformWindows;"
    };
    let use_graphics: &str = match config.graphics {
        AppGraphicsApi::vulkan => "use renderer::vk_renderer::VkRenderer;"
    };

    let title_def: String = format!("const APP_TITLE: &str = \"{}\";", config.name);

    let mut vbo_index_decls = String::new();
    for (i, model) in config.shared_resources.models.iter().enumerate() {
        vbo_index_decls = format!("{}\n    pub const VBO_INDEX_{}: usize = 0x00000000{:08};", vbo_index_decls, model.id.to_uppercase(), i);
    }

    let mut texture_index_decls = String::new();
    for (i, texture) in config.shared_resources.textures.iter().enumerate() {
        texture_index_decls = format!("{}\n    pub const TEXTURE_INDEX_{}: usize = 0x00000000{:08};", texture_index_decls, texture.id.to_uppercase(), i);
    }

    let platform_construct: &str = match config.platform {
        AppPlatform::windows => "PlatformWindows::new_window(APP_TITLE)"
    };

    let engine_decl: &str = match config.graphics {
        AppGraphicsApi::vulkan => "let engine: Engine<VkRenderer>"
    };

    let content = format!("
use crate::scenes::{}::{};

{}
{}
use engine::Engine;
use defs::render::FeatureDeclaration;

{}
pub mod shared_indices {{{}{}
}}

pub struct App {{}}

impl App {{

    pub fn new() -> App {{
        App {{}}
    }}

    pub fn run(&self) {{

        let mut platform = {}
            .unwrap_or_else(|e| {{
                println!(\"Error creating window: {{:?}}\", e);
                std::process::exit(1);
            }});

        {} = Engine::new_uninitialised(
            Box::from({}::new()),
            vec![FeatureDeclaration::ClipPlanes]);

        platform.run(engine)
            .unwrap_or_else(|e| {{
                println!(\"Error while running: {{:?}}\", e);
                std::process::exit(1);
            }});
    }}
}}
", start_scene, struct_name, use_platform, use_graphics, title_def, vbo_index_decls, texture_index_decls, platform_construct, engine_decl, struct_name);
    Ok(content)
}
