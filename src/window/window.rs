use super::builder::WindowBuilder;

pub struct Window {
    pub(crate) window: winit::window::Window,
    pub(crate) event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }
}
