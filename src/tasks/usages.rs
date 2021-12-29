use std::ops::Deref;

use crate::ecs::Resource;

use super::TaskPool;

#[derive(Clone, Debug, Resource)]
pub struct IoTaskPool(pub TaskPool);

impl Deref for IoTaskPool {
    type Target = TaskPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
