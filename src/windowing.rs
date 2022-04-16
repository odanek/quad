mod event;
mod handle;
mod size;
mod window;
mod windows;

pub use event::*;
pub use handle::RawWindowHandleWrapper;
pub use size::{FullScreen, LogicalSize, PhysicalSize, WindowSize};
pub use window::{PresentMode, Window, WindowDescriptor, WindowId};
pub use windows::Windows;

use crate::app::App;

pub fn windowing_plugin(app: &mut App) {
    app.init_resource::<Windows>();

    app.add_event::<WindowCreated>();
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
