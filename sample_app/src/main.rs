
use defs::RendererApi;
use platform_windows::PlatformWindows;
use renderer::vk_renderer::VkRenderer;
use engine::Engine;

use raw_window_handle::HasRawWindowHandle;

const APP_TITLE: &str = "Strength Beyond Fear";

fn main() {
    let mut platform = PlatformWindows::new_window(APP_TITLE).unwrap();
    let engine: Engine<VkRenderer> = Engine::new_uninitialised();
    if let Err(e) = platform.run(engine) {
        println!("{}", e);
    }
}
