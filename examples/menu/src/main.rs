
mod start;

use start::StartMenuScene;

use platform_windows::PlatformWindows;
use renderer::vk_renderer::VkRenderer;
use engine::Engine;

const APP_TITLE: &str = "Menu Example";

fn main() {

    let mut platform = PlatformWindows::new_window(APP_TITLE)
        .unwrap_or_else(|e| {
            println!("Error creating window: {}", e);
            std::process::exit(1);
        });

    let engine: Engine<VkRenderer> = Engine::new_uninitialised(Box::from(StartMenuScene::new()));

    platform.run(engine)
        .unwrap_or_else(|e| {
            println!("Error while running: {}", e);
            std::process::exit(1);
        });
}
