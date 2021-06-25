use crate::window::Window;

use super::{Scene, builder::QuadBuilder, context::Context};

pub struct Quad {
    pub(crate) main_window: Window,
}

impl Quad {
    pub fn builder() -> QuadBuilder {
        QuadBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let window = self.main_window;
        let event_loop = window.event_loop;
        let mut exit = false;

        let mut context = Context::new(scene);
        context.register_resources();

        event_loop.run(move |event, _, control_flow| {
            use winit::{
                event::{Event, WindowEvent},
                event_loop::ControlFlow,
            };

            match event {
                Event::MainEventsCleared => {
                    if !exit {
                        exit = context.handle_scene_update();
                        context.flush_keyboard_events();
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
