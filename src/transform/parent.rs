use crate::ecs::{Component, Entity};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct Parent(pub(crate) Entity);

impl Parent {
    /// Gets the [`Entity`] ID of the parent.
    pub fn get(&self) -> Entity {
        self.0
    }
}
