mod builder;
mod size;

pub use builder::WindowBuilder;
pub use size::{FullScreen, LogicalSize, PhysicalSize, Size};
pub struct Window {
    pub window: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }
}
