use crate::{
    ecs::{Event, Resource},
    ty::Vec2,
};

use super::button::{ButtonState, Buttons};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(val) => MouseButton::Other(val),
        }
    }
}

#[derive(Default, Resource)]
pub struct MouseInput {
    buttons: Buttons<MouseButton>,
}

impl MouseInput {
    pub(crate) fn toggle(&mut self, button: MouseButton, state: ButtonState) {
        self.buttons.toggle(button, state);
    }

    pub(crate) fn flush(&mut self) {
        self.buttons.flush();
    }

    pub fn pressed(&self, button: MouseButton) -> bool {
        self.buttons.pressed(button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.buttons.just_pressed(button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.buttons.just_released(button)
    }

    pub fn button_state(&self, button: MouseButton) -> ButtonState {
        self.buttons.button_state(button)
    }
}

#[derive(Debug, Clone, Event)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ButtonState,
}

#[derive(Debug, Clone, Event)]
pub struct MouseMotion {
    pub delta: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseScrollUnit {
    Line,
    Pixel,
}

#[derive(Debug, Clone, Event)]
pub struct MouseWheel {
    pub unit: MouseScrollUnit,
    pub x: f32,
    pub y: f32,
}
