use winit::{dpi::PhysicalSize, event::{ElementState, Event, WindowEvent}, event_loop::ControlFlow};

use crate::{
    builder::QuadBuilder,
    context::Context,
    input::{KeyCode, KeyboardInput},
    SceneResult,
};

use super::{scene::Scene, window::Window};

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
        context.start_scene();

        event_loop.run(move |event, _, control_flow| {            
            match event {
                Event::MainEventsCleared => {
                    if !exit {
                        exit = handle_scene_update(&mut context);
                        flush_keyboard_events(&mut context);
                    }
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::KeyboardInput { input, .. } => handle_keyboard_event(&mut context, input),
                        WindowEvent::CloseRequested => exit = true,
                        WindowEvent::Resized(size) => handle_window_resize(&mut context, size),
                        _ => (),
                    }
                }
                _ => (),
            }

            *control_flow = if exit { ControlFlow::Exit } else { ControlFlow::Poll};
        });
    }
}

// TODO Methods on Context
fn handle_scene_update(context: &mut Context) -> bool {
    // Update
    // Draw
    let result = context.update_scene();
    match result {
        SceneResult::Quit => true,
        _ => false,
    }
}

fn handle_window_resize(context: &mut Context, size: PhysicalSize<u32>) {
    if size.width != 0 || size.height != 0 {
        // Resized
    } else {
        // Minimized
    }
}

// TODO KeyCode mapping
fn handle_keyboard_event(context: &mut Context, input: winit::event::KeyboardInput) {
    if let Some(keycode) = input.virtual_keycode {
        let keyboard_input = context.world.get_resource_mut::<KeyboardInput>();
        match input.state {
            ElementState::Pressed => keyboard_input.press(KeyCode::Escape),
            ElementState::Released => keyboard_input.release(KeyCode::Escape),
        }
    }
}

fn flush_keyboard_events(context: &mut Context) {
    let keyboard_input = context.world.get_resource_mut::<KeyboardInput>();
    keyboard_input.flush();
}

// match virtual_code {
//     VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
//     _ => (),
// }
