use crate::app::SceneResult;

use super::{app::AppEventLoop, context::AppContext};

pub fn winit_runner(mut context: AppContext, event_loop: AppEventLoop) {
    let mut active = true;
    let mut exit = false;

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
                let window_id = if let Some(window) = context.get_window(winit_window_id) {
                    window.id()
                } else {
                    log::debug!(
                        "Skipping event for unknown winit window id {:?}",
                        winit_window_id
                    );
                    return;
                };

                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        context.handle_keyboard_event(input)
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        context.handle_mouse_button(button, state)
                    }
                    WindowEvent::CloseRequested => exit = true,
                    WindowEvent::Resized(size) => {
                        context.handle_window_resize(window_id, size.width, size.height)
                    }
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
