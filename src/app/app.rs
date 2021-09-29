use crate::{app::SceneResult, ecs::World, window::Window};

use super::{builder::AppBuilder, context::AppContext, event::AppEvents, Scene};

pub struct App {
    pub(crate) main_window: Window,
    pub(crate) world: Box<World>,
    pub(crate) events: AppEvents,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let window = self.main_window;
        let event_loop = window.event_loop;
        let mut exit = false;

        let mut context = AppContext::new(self.world, self.events, scene);
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
                        context.handle_keyboard_event(input);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        context.handle_mouse_button(button, state);
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
