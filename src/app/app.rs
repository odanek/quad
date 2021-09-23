use crate::{app::SceneResult, window::Window};

use super::{Scene, builder::AppBuilder, context::Context};

pub struct App {
    pub(crate) main_window: Window,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let window = self.main_window;
        let event_loop = window.event_loop;
        let mut exit = false;

        let mut context = Context::new(scene);
        context.insert_resources();
        context.start_scene();

        event_loop.run(move |event, _, control_flow| {
            use winit::{
                event::{Event, WindowEvent},
                event_loop::ControlFlow,
            };

            match event {
                Event::MainEventsCleared => {
                    if !exit {
                        exit = matches!(context.update_scene(), SceneResult::Quit);
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        context.handle_keyboard_event(input)
                    }
                    WindowEvent::CloseRequested => exit = true,
                    WindowEvent::Resized(size) => context.handle_window_resize(size),
                    _ => (),
                },
                _ => (),
            }

            *control_flow = if exit {
                ControlFlow::Exit
            } else {
                ControlFlow::Poll
            };
        });
    }
}
