use crate::{
    ecs::{Event, Resource},
    windowing::WindowId,
};

use super::{
    KeyCode,
    button::{ButtonState, Buttons},
    keycode::Key,
};

#[derive(Debug, Clone, Event)]
pub struct KeyInput {
    pub window_id: WindowId,
    pub key_code: KeyCode,
    pub logical_key: Key,
    pub state: ButtonState,
}

#[derive(Default, Resource)]
pub struct KeyboardInput {
    buttons: Buttons<KeyCode>,
}

impl KeyboardInput {
    pub(crate) fn toggle(&mut self, key: KeyCode, state: ButtonState) {
        self.buttons.toggle(key, state);
    }

    pub(crate) fn flush(&mut self) {
        self.buttons.flush();
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.buttons.pressed(key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.buttons.just_pressed(key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.buttons.just_released(key)
    }

    pub fn key_state(&self, key: KeyCode) -> ButtonState {
        self.buttons.button_state(key)
    }
}
