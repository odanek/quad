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
    app.init_resource::<Windows>()
        .add_event::<WindowCreated>()
        .add_event::<ReceivedCharacter>()
        .add_event::<WindowCloseRequested>()
        .add_event::<WindowResized>()
        .add_event::<WindowMoved>()
        .add_event::<CursorMoved>()
        .add_event::<CursorEntered>()
        .add_event::<CursorLeft>()
        .add_event::<WindowFocused>()
        .add_event::<WindowBackendScaleFactorChanged>()
        .add_event::<WindowScaleFactorChanged>();
}
