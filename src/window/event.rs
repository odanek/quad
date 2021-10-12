use crate::ecs::Event;

use super::WindowId;

#[derive(Debug, Clone)]
pub struct WindowResized {
    pub id: WindowId,
    pub width: f32,
    pub height: f32,
}

impl Event for WindowResized {}

#[derive(Debug, Clone)]
pub struct CloseWindow {
    pub id: WindowId,
}

impl Event for CloseWindow {}

#[derive(Debug, Clone)]
pub struct WindowCreated {
    pub id: WindowId,
}

impl Event for WindowCreated {}

#[derive(Debug, Clone)]
pub struct WindowFocused {
    pub id: WindowId,
    pub focused: bool,
}

impl Event for WindowFocused {}

#[derive(Debug, Clone)]
pub struct WindowScaleFactorChanged {
    pub id: WindowId,
    pub scale_factor: f64,
}

impl Event for WindowScaleFactorChanged {}

#[derive(Debug, Clone)]
pub struct WindowBackendScaleFactorChanged {
    pub id: WindowId,
    pub scale_factor: f64,
}

impl Event for WindowBackendScaleFactorChanged {}

#[derive(Debug, Clone)]
pub struct WindowMoved {
    pub id: WindowId,
    pub position: cgm::Vec2<i32>,
}

impl Event for WindowMoved {}

#[derive(Debug, Clone)]
pub struct ReceivedCharacter {
    pub id: WindowId,
    pub char: char,
}

impl Event for ReceivedCharacter {}
