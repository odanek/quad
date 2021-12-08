mod builder;
mod event;
mod size;
mod window;
mod windows;

pub(crate) use builder::WindowBuilder;
pub use event::*;
pub use size::{FullScreen, LogicalSize, PhysicalSize, Size};
pub use window::{Window, WindowId};
pub use windows::Windows;

use crate::app::App;

pub fn windowing_plugin(app: &mut App) {
    // TODO: Add Windows resource

    app.add_event::<ReceivedCharacter>();
    app.add_event::<WindowCloseRequested>();
    app.add_event::<WindowResized>();
    app.add_event::<WindowMoved>();
    app.add_event::<CursorMoved>();
    app.add_event::<CursorEntered>();
    app.add_event::<CursorLeft>();
    app.add_event::<WindowFocused>();
    app.add_event::<WindowBackendScaleFactorChanged>();
    app.add_event::<WindowScaleFactorChanged>();
}
