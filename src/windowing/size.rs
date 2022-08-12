#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct LogicalSize {
    pub width: f64,
    pub height: f64,
}

impl LogicalSize {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FullScreen {
    Borderless,
}

#[derive(Copy, Clone, PartialEq)]
pub enum WindowSize {
    Physical(PhysicalSize),
    Logical(LogicalSize),
    FullScreen(FullScreen),
}

impl Default for WindowSize {
    fn default() -> Self {
        Self::FullScreen(FullScreen::Borderless)
    }
}

impl From<PhysicalSize> for WindowSize {
    fn from(size: PhysicalSize) -> Self {
        Self::Physical(size)
    }
}

impl From<LogicalSize> for WindowSize {
    fn from(size: LogicalSize) -> Self {
        Self::Logical(size)
    }
}

impl From<FullScreen> for WindowSize {
    fn from(size: FullScreen) -> Self {
        Self::FullScreen(size)
    }
}
