use std::ops::Deref;

use crate::ecs::{Component, Entity};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct Parent(pub Entity);

impl Deref for Parent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
