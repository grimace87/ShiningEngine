
mod scene;

use scene::SceneryScene;

use platform_windows::PlatformWindows;
use renderer::vk_renderer::VkRenderer;
use engine::Engine;
use defs::FeatureDeclaration;

const APP_TITLE: &str = "Scenery Example";

fn main() {

    let mut platform = PlatformWindows::new_window(APP_TITLE)
        .unwrap_or_else(|e| {
            println!("Error creating window: {}", e);
            std::process::exit(1);
        });

    let engine: Engine<VkRenderer> = Engine::new_uninitialised(
        Box::from(SceneryScene::new()),
        vec![FeatureDeclaration::ClipPlanes]);

    platform.run(engine)
        .unwrap_or_else(|e| {
            println!("Error while running: {}", e);
            std::process::exit(1);
        });
}
