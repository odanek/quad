use crate::ty::{Vec2, Vec2i};

use super::WindowBuilder;

// TODO: Window ids, multiple window handling
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(u32);

impl WindowId {
    pub(crate) fn primary() -> Self {
        Self(0)
    }

    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

pub struct Window {
    id: WindowId,
    physical_width: u32,
    physical_height: u32,
    backend_scale_factor: f64,
    scale_factor_override: Option<f64>,
    position: Option<Vec2i>,
    cursor_position: Option<Vec2>,
    focused: bool,
    winit_window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(id: WindowId, winit_window: winit::window::Window) -> Self {
        let position = winit_window
            .outer_position()
            .ok()
            .map(|position| Vec2i::new(position.x, position.y));

        Window {
            id,
            physical_width: winit_window.inner_size().width,
            physical_height: winit_window.inner_size().height,
            backend_scale_factor: winit_window.scale_factor(),
            scale_factor_override: None,
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
    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    #[inline]
    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    #[inline]
    pub fn backend_scale_factor(&self) -> f64 {
        self.backend_scale_factor
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor_override
            .unwrap_or(self.backend_scale_factor)
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

    pub(crate) fn update_backend_scale_factor(&mut self, scale_factor: f64) {
        self.backend_scale_factor = scale_factor;
    }

    pub(crate) fn update_position(&mut self, position: Option<Vec2i>) {
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
