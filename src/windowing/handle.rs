use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

#[derive(Debug, Clone)]
pub struct RawWindowHandleWrapper(RawWindowHandle);

impl RawWindowHandleWrapper {
    pub(crate) fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_handle(&self) -> HasRawWindowHandleWrapper {
        HasRawWindowHandleWrapper(self.0)
    }
}

unsafe impl Send for RawWindowHandleWrapper {}
unsafe impl Sync for RawWindowHandleWrapper {}

pub struct HasRawWindowHandleWrapper(RawWindowHandle);

unsafe impl HasRawWindowHandle for HasRawWindowHandleWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}
