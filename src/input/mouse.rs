use crate::ty::Vec2;

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

pub struct MouseInput {
    buttons: Buttons<MouseButton>,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            buttons: Default::default(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct MouseMotion {
    pub delta: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseScrollUnit {
    Line,
    Pixel,
}

#[derive(Debug, Clone)]
pub struct MouseWheel {
    pub unit: MouseScrollUnit,
    pub x: f32,
    pub y: f32,
}

// TODO: Handle in AppContext
// pub fn mouse_button_input_system(
//     mut mouse_button_input: ResMut<Input<MouseButton>>,
//     mut mouse_button_input_events: EventReader<MouseButtonInput>,
// ) {
//     mouse_button_input.clear();
//     for event in mouse_button_input_events.iter() {
//         match event.state {
//             MouseButtonState::Pressed => mouse_button_input.press(event.button),
//             MouseButtonState::Released => mouse_button_input.release(event.button),
//         }
//     }
// }
