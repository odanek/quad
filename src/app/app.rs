use crate::{
    app::SceneResult,
    ecs::World,
    window::{Window, WindowBuilder, WindowId},
};

use super::{builder::AppBuilder, context::AppContext, event::AppEvents, Scene};

pub struct App {
    event_loop: winit::event_loop::EventLoop<()>,
    main_window: Window,
    world: Box<World>,
    events: Box<AppEvents>,
}

impl App {
    pub(crate) fn new(
        window_spec: WindowBuilder,
        world: Box<World>,
        events: Box<AppEvents>,
    ) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window_id = WindowId::new(0);
        let main_window = window_spec.build(main_window_id, &event_loop);

        Self {
            event_loop,
            main_window,
            world,
            events,
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let main_window = self.main_window;
        let event_loop = self.event_loop;

        let mut active = true;
        let mut exit = false;

        let mut context = AppContext::new(self.world, self.events, scene);
        context.insert_resources();
        context.start_scene();

        event_loop.run(move |event, _, control_flow| {
            use winit::{
                event::{Event, WindowEvent},
                event_loop::ControlFlow,
            };

            if exit {
                *control_flow = ControlFlow::Exit;
                return;
            } else {
                *control_flow = ControlFlow::Poll;
            }

            match event {
                Event::MainEventsCleared => {
                    if active && !exit {
                        exit = matches!(context.update_scene(), SceneResult::Quit);
                    }
                }
                Event::WindowEvent {
                    event,
                    window_id: winit_window_id,
                    ..
                } => {
                    if winit_window_id != main_window.winit_id() {
                        log::debug!(
                            "Skipping event for unknown winit window id {:?}",
                            winit_window_id
                        );
                        return;
                    }

                    match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            context.handle_keyboard_event(input);
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            context.handle_mouse_button(button, state);
                        }
                        WindowEvent::CloseRequested => exit = true,
                        WindowEvent::Resized(size) => context.handle_window_resize(size),
                        _ => (),
                    }
                }
                Event::Suspended => {
                    active = false;
                }
                Event::Resumed => {
                    active = true;
                }
                _ => (),
            }
        });
    }
}
