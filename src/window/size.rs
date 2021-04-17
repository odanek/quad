#[derive(Copy, Clone, PartialEq)]
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
pub enum Size {
    Physical(PhysicalSize),
    Logical(LogicalSize),
    FullScreen(FullScreen),
}

impl From<PhysicalSize> for Size {
    fn from(size: PhysicalSize) -> Self {
        Self::Physical(size)
    }
}

impl From<LogicalSize> for Size {
    fn from(size: LogicalSize) -> Self {
        Self::Logical(size)
    }
}

impl From<FullScreen> for Size {
    fn from(size: FullScreen) -> Self {
        Self::FullScreen(size)
    }
}
