use crate::ty::Vec2;

pub struct MouseInput {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MouseButtonState {
    Pressed,
    Released,
}

impl MouseButtonState {
    pub fn is_pressed(&self) -> bool {
        matches!(self, MouseButtonState::Pressed)
    }
}

#[derive(Debug, Clone)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: MouseButtonState,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
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