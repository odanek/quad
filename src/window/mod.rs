mod builder;
mod event;
mod size;

pub(crate) use builder::WindowBuilder;
pub use event::*;
pub use size::{FullScreen, LogicalSize, PhysicalSize, Size};

use crate::ty::{IVec2, Vec2};

// TODO: Window ids, multiple window handling
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WindowId(u32);

impl WindowId {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

pub struct Window {
    id: WindowId,
    physical_width: u32,
    physical_height: u32,
    backend_scale_factor: f64,
    position: Option<IVec2>,
    cursor_position: Option<Vec2>,
    focused: bool,
    winit_window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(id: WindowId, winit_window: winit::window::Window) -> Self {
        let position = winit_window
            .outer_position()
            .ok()
            .map(|position| IVec2::new(position.x, position.y));
        Window {
            id,
            physical_width: winit_window.inner_size().width,
            physical_height: winit_window.inner_size().height,
            backend_scale_factor: winit_window.scale_factor(),
            position,
            cursor_position: None,
            focused: true,
            winit_window,
        }
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        self.id
    }

    #[inline]
    pub fn width(&self) -> f32 {
        (self.physical_width as f64 / self.scale_factor()) as f32
    }

    #[inline]
    pub fn height(&self) -> f32 {
        (self.physical_height as f64 / self.scale_factor()) as f32
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.backend_scale_factor
    }

    pub(crate) fn winit_id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    pub(crate) fn winit_window(&self) -> &winit::window::Window {
        &self.winit_window
    }

    pub(crate) fn update_physical_size(&mut self, width: u32, height: u32) {
        self.physical_width = width;
        self.physical_height = height;
    }

    pub(crate) fn update_position(&mut self, position: Option<IVec2>) {
        self.position = position;
    }

    pub(crate) fn update_cursor_position(&mut self, position: Option<Vec2>) {
        self.cursor_position = position;
    }

    pub(crate) fn update_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    #[inline]
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }
}
