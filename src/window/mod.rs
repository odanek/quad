mod builder;
pub mod event;
mod size;

pub use builder::WindowBuilder; // TODO: Shouldn't be exposed from the crate
pub use size::{FullScreen, LogicalSize, PhysicalSize, Size};

// TODO: Window ids, multiple window handling
#[derive(Debug, Copy, Clone)]
pub struct WindowId(u32);

pub struct Window {
    _window: winit::window::Window,
    pub(crate) event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }
}
