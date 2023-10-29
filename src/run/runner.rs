use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::ControlFlow, error::EventLoopError,
};

use super::context::RunContext;

pub type AppEventLoop = winit::event_loop::EventLoop<()>;

pub fn winit_runner(mut context: RunContext, event_loop: AppEventLoop) -> Result<(), EventLoopError> {
    let mut active = true;
    let mut exit = false;

    event_loop.run(move |event, event_loop| {
        if exit {
            event_loop.exit();
            return;
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        match event {
            Event::AboutToWait => {
                if active && !exit {
                    exit = context.update();
                }
            }
            Event::WindowEvent {
                event,
                window_id: winit_window_id,
                ..
            } => {
                let window_id = if let Some(id) = context.get_window_id(winit_window_id) {
                    id
                } else {
                    log::debug!(
                        "Skipping event for unknown winit window id {:?}",
                        winit_window_id
                    );
                    return;
                };

                match event {
                    WindowEvent::KeyboardInput { ref event, .. } => {
                        context.handle_keyboard_event(window_id, event)
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
                        context.handle_touch(window_id, touch);
                    }
                    WindowEvent::Focused(focused) => {
                        context.handle_window_focused(window_id, focused);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        context.handle_scale_factor_changed(window_id, scale_factor);
                    }
                    _ => (),
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                context.handle_mouse_motion(delta);
            }
            Event::Suspended => {
                active = false;
            }
            Event::Resumed => {
                active = true;
            }
            _ => (),
        }
    })
}
