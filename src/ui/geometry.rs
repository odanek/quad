#[derive(Copy, Clone, PartialEq, Debug)]
pub struct UiRect<T: PartialEq> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T: PartialEq + Copy> UiRect<T> {
    pub fn all(value: T) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

impl<T: Default + PartialEq> Default for UiRect<T> {
    fn default() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            top: Default::default(),
            bottom: Default::default(),
        }
    }
}
