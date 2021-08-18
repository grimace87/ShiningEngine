
mod control_translations;

use self::{
    control_translations::{translate_code, translate_state}
};

use defs::RendererApi;
use engine::Engine;

use winit::{
    event_loop::{EventLoop, ControlFlow},
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    window::{Window, WindowBuilder},
    dpi::LogicalSize,
    platform::run_return::EventLoopExtRunReturn
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PlatformWindows {
    window: Window,
    event_loop: Option<EventLoop<()>>
}

unsafe impl HasRawWindowHandle for PlatformWindows {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}

impl PlatformWindows {

    pub fn new_window(app_title: &str) -> Result<PlatformWindows, String> {

        // Ready the Winit window and event loop
        let event_loop = EventLoop::new();
        let window: Window = WindowBuilder::new()
            .with_title(app_title)
            .with_inner_size(LogicalSize::new(800, 600))
            .build(&event_loop)
            .map_err(|os_err| format!("{:?}", os_err))?;

        Ok(PlatformWindows {
            window,
            event_loop: Some(event_loop)
        })
    }

    pub fn run<R: 'static>(&mut self, mut engine: Engine<R>) -> Result<(), String> where R : RendererApi {

        engine.initialise(self);

        // Loop
        let mut event_loop = self.event_loop.take().unwrap();
        event_loop.run_return(|event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        (Some(keycode), state) => {
                                            engine.process_keyboard_event(translate_code(keycode), translate_state(state))
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        WindowEvent::Resized(_) => {
                            engine.recreate_swapchain(&self.window).unwrap();
                        }
                        _ => {}
                    }
                },
                Event::MainEventsCleared => {
                    let time_passed_millis = engine.pull_time_step_millis();

                    // Update controls and camera
                    engine.update(time_passed_millis);

                    self.window.request_redraw();
                },
                Event::RedrawRequested(_) => {
                    engine.render(self).unwrap();
                },
                _ => ()
            }
        });

        Ok(())
    }
}
