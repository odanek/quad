use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::{
    scene::Scene,
    window::{Window, WindowBuilder},
};

pub struct Quad {
    main_window: Window,
}

impl Quad {
    pub fn builder() -> QuadBuilder {
        QuadBuilder::default()
    }

    pub fn run(mut self, scene: Box<dyn Scene>) {
        let event_loop = self.main_window.event_loop;
        let mut exit = false;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    if !exit {
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

pub struct QuadBuilder {
    main_window: WindowBuilder,
}

impl Default for QuadBuilder {
    fn default() -> Self {
        QuadBuilder {
            main_window: WindowBuilder::default(),
        }
    }
}

impl QuadBuilder {
    pub fn main_window(mut self, window: WindowBuilder) -> QuadBuilder {
        self.main_window = window;
        self
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let quad = Quad {
            main_window: self.main_window.build(),
        };
        quad.run(scene);
    }
}
