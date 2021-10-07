use std::ops::Deref;

use crate::ecs::{Component, Entity};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Parent(pub Entity);
impl Component for Parent {}

impl Deref for Parent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
