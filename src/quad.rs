use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::{builder::QuadBuilder, context::Context};

use super::{scene::Scene, window::Window};

pub struct Quad {
    pub main_window: Window,
}

impl Quad {
    pub fn builder() -> QuadBuilder {
        QuadBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let event_loop = self.main_window.event_loop;
        let mut exit = false;

        let mut context = Context::new(scene);
        context.register_resources();
        context.start_scene();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    if !exit {
                        let _result = context.update_scene();
                        // Update
                        // Draw
                    }
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(virtual_code),
                                    state: ElementState::Released,
                                    ..
                                },
                            ..
                        } => match virtual_code {
                            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            _ => (),
                        },
                        WindowEvent::CloseRequested => {
                            exit = true;
                            *control_flow = ControlFlow::Exit
                        }
                        WindowEvent::Resized(size) => {
                            if size.width != 0 || size.height != 0 {
                                // Resized
                            } else {
                                // Minimized
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        });
    }
}
