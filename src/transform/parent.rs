use std::ops::{Deref, DerefMut};

use crate::ecs::Entity;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Parent(pub Entity);

impl Deref for Parent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Parent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
