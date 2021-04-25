use std::collections::HashSet;

use super::KeyCode;

pub struct KeyboardInput {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl Default for KeyboardInput {
    fn default() -> Self {
        KeyboardInput {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl KeyboardInput {
    pub(crate) fn press(&mut self, key: KeyCode) {
        self.pressed.insert(key);
        self.just_pressed.insert(key);
    }

    pub(crate) fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }
}
