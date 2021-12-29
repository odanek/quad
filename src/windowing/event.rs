use crate::{
    ecs::Event,
    ty::{Vec2, Vec2i},
};

use super::WindowId;

#[derive(Debug, Clone, Event)]
pub struct WindowCloseRequested {
    pub id: WindowId,
}

#[derive(Debug, Clone, Event)]
pub struct WindowResized {
    pub id: WindowId,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Event)]
pub struct CloseWindow {
    pub id: WindowId,
}

#[derive(Debug, Clone, Event)]
pub struct WindowCreated {
    pub id: WindowId,
}

#[derive(Debug, Clone, Event)]
pub struct WindowFocused {
    pub id: WindowId,
    pub focused: bool,
}

#[derive(Debug, Clone, Event)]
pub struct WindowScaleFactorChanged {
    pub id: WindowId,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Event)]
pub struct WindowBackendScaleFactorChanged {
    pub id: WindowId,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Event)]
pub struct WindowMoved {
    pub id: WindowId,
    pub position: Vec2i,
}

#[derive(Debug, Clone, Event)]
pub struct ReceivedCharacter {
    pub id: WindowId,
    pub char: char,
}

#[derive(Debug, Clone, Event)]
pub struct CursorMoved {
    pub id: WindowId,
    pub position: Vec2,
}

#[derive(Debug, Clone, Event)]
pub struct CursorEntered {
    pub id: WindowId,
}

#[derive(Debug, Clone, Event)]
pub struct CursorLeft {
    pub id: WindowId,
}
