use super::{
    button::{ButtonState, Buttons},
    KeyCode,
};

pub struct KeyboardInput {
    buttons: Buttons<KeyCode>,
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self {
            buttons: Default::default(),
        }
    }
}

impl KeyboardInput {
    pub(crate) fn press(&mut self, key: KeyCode) {
        self.buttons.press(key);
    }

    pub(crate) fn release(&mut self, key: KeyCode) {
        self.buttons.release(key);
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
