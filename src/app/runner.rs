use winit::event::DeviceEvent;

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
                    WindowEvent::MouseWheel { delta, .. } => {
                        context.handle_mouse_wheel(delta);
                    }
                    WindowEvent::CloseRequested => {
                        context.handle_window_close_requested(window_id);
                        exit = true;
                    }
                    WindowEvent::Resized(size) => {
                        context.handle_window_resized(window_id, size.width, size.height);
                    }
                    WindowEvent::Moved(position) => {
                        context.handle_window_moved(window_id, position);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        context.handle_cursor_moved(window_id, position);
                    }
                    WindowEvent::CursorEntered { .. } => {
                        context.handle_cursor_entered(window_id);
                    }
                    WindowEvent::CursorLeft { .. } => {
                        context.handle_cursor_left(window_id);
                    }
                    WindowEvent::Touch(touch) => {
                        // TODO
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        context.handle_window_character(window_id, c);
                    }
                    WindowEvent::Focused(focused) => {
                        context.handle_window_focused(window_id, focused);
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                        ..
                    } => {
                        // TODO
                    }
                    _ => (),
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // TODO
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
