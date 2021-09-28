use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl ButtonState {
    pub fn is_pressed(&self) -> bool {
        matches!(self, ButtonState::Pressed)
    }
}

pub struct Buttons<T> {
    pressed: HashSet<T>,
    just_pressed: HashSet<T>,
    just_released: HashSet<T>,
}

impl<T> Default for Buttons<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T> Buttons<T>
where
    T: Copy + Eq + Hash,
{
    pub(crate) fn press(&mut self, button: T) {
        self.pressed.insert(button);
        self.just_pressed.insert(button);
    }

    pub(crate) fn release(&mut self, button: T) {
        self.pressed.remove(&button);
        self.just_released.insert(button);
    }

    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&self, button: T) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: T) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn just_released(&self, button: T) -> bool {
        self.just_released.contains(&button)
    }

    pub fn button_state(&self, button: T) -> ButtonState {
        if self.pressed(button) {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        }
    }
}
