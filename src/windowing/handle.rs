use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

#[derive(Debug, Clone)]
pub struct WindowHandleWrapper(RawWindowHandle, RawDisplayHandle);

impl WindowHandleWrapper {
    pub(crate) fn new(window: RawWindowHandle, display: RawDisplayHandle) -> Self {
        Self(window, display)
    }
}

unsafe impl Send for WindowHandleWrapper {}
unsafe impl Sync for WindowHandleWrapper {}

impl HasWindowHandle for WindowHandleWrapper {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { Ok(WindowHandle::borrow_raw(self.0)) }
    }
}

impl HasDisplayHandle for WindowHandleWrapper {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { Ok(DisplayHandle::borrow_raw(self.1)) }
    }
}
