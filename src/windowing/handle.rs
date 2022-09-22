use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

#[derive(Debug, Clone)]
pub struct RawWindowHandleWrapper(RawWindowHandle, RawDisplayHandle);

impl RawWindowHandleWrapper {
    pub(crate) fn new(window: RawWindowHandle, display: RawDisplayHandle) -> Self {
        Self(window, display)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_handle(&self) -> HasRawWindowHandleWrapper {
        HasRawWindowHandleWrapper(self.0, self.1)
    }
}

unsafe impl Send for RawWindowHandleWrapper {}
unsafe impl Sync for RawWindowHandleWrapper {}

pub struct HasRawWindowHandleWrapper(RawWindowHandle, RawDisplayHandle);

unsafe impl HasRawWindowHandle for HasRawWindowHandleWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

unsafe impl HasRawDisplayHandle for HasRawWindowHandleWrapper {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.1
    }
}
