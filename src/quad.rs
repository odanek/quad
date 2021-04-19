use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::{SceneResult, World};

use super::{
    scene::Scene,
    window::{Window, WindowBuilder},
};

pub struct Quad {
    main_window: Window,
}

pub struct Context {
    world: Box<World>,
    scene: Box<dyn Scene>
}

impl Context {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        Context {
            world: Box::new(World::new()),
            scene
        }
    }

    pub fn start_scene(&mut self) {
        self.scene.start(&mut self.world);
    }

    pub fn update_scene(&mut self) -> SceneResult {
        self.scene.update(&mut self.world)
    }
}

impl Quad {
    pub fn builder() -> QuadBuilder {
        QuadBuilder::default()
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let event_loop = self.main_window.event_loop;
        let mut exit = false;

        let mut context = Context::new(scene);
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
