use raw_window_handle::HasRawWindowHandle;

use crate::ty::{Size, Vec2, Vec2i};

use super::{handle::RawWindowHandleWrapper, WindowSize};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "vsync")]
pub enum PresentMode {
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
    /// selected platform and backend. Not optimal for mobile.
    Immediate = 0,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
    /// if unavailable on the selected platform and backend. Not optimal for mobile.
    Mailbox = 1,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    Fifo = 2, // NOTE: The explicit ordinal values mirror wgpu and the vulkan spec.
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(u32);

impl Default for WindowId {
    fn default() -> Self {
        Self::primary()
    }
}

impl WindowId {
    pub(crate) fn primary() -> Self {
        Self(0)
    }
}

#[derive(Default)]
pub struct WindowDescriptor {
    pub title: String,
    pub size: WindowSize,
}

pub struct Window {
    id: WindowId,
    present_mode: PresentMode,
    physical_width: u32,
    physical_height: u32,
    backend_scale_factor: f64,
    scale_factor_override: Option<f64>,
    position: Option<Vec2i>,
    cursor_position: Option<Vec2>,
    focused: bool,
    raw_window_handle: RawWindowHandleWrapper,
    winit_window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(
        id: WindowId,
        descriptor: &WindowDescriptor,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Self {
        let mut builder = winit::window::WindowBuilder::new().with_title(descriptor.title.clone());

        match descriptor.size {
            WindowSize::Physical(size) => {
                builder =
                    builder.with_inner_size(winit::dpi::PhysicalSize::new(size.width, size.height));
            }
            WindowSize::Logical(size) => {
                builder =
                    builder.with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
            }
            WindowSize::FullScreen(_) => {
                builder =
                    builder.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
        }

        let winit_window = builder.build(event_loop).expect("Unable to create window");
        let position = winit_window
            .outer_position()
            .ok()
            .map(|position| Vec2i::new(position.x, position.y));

        Window {
            id,
            present_mode: PresentMode::Fifo,
            physical_width: winit_window.inner_size().width,
            physical_height: winit_window.inner_size().height,
            backend_scale_factor: winit_window.scale_factor(),
            scale_factor_override: None,
            position,
            cursor_position: None,
            focused: true,
            raw_window_handle: RawWindowHandleWrapper::new(winit_window.raw_window_handle()),
            winit_window,
        }
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        self.id
    }

    #[inline]
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
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
    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
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
    pub fn physical_size(&self) -> Size<u32> {
        Size {
            width: self.physical_width,
            height: self.physical_height,
        }
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

    pub(crate) fn raw_window_handle(&self) -> RawWindowHandleWrapper {
        self.raw_window_handle.clone()
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
}
