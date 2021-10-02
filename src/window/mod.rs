mod builder;
pub mod event;
mod size;

pub(crate) use builder::WindowBuilder;
pub use size::{FullScreen, LogicalSize, PhysicalSize, Size};

// TODO: Window ids, multiple window handling
#[derive(Debug, Copy, Clone)]
pub struct WindowId(u32);

impl WindowId {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

pub struct Window {
    id: WindowId,
    winit_window: winit::window::Window,
}

impl Window {
    pub fn id(&self) -> WindowId {
        self.id
    }

    pub(crate) fn winit_id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }
}
