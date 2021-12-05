use std::ops::Deref;

use crate::ecs::Resource;

use super::TaskPool;

#[derive(Clone, Debug)]
pub struct IoTaskPool(pub TaskPool);

impl Resource for IoTaskPool {}

impl Deref for IoTaskPool {
    type Target = TaskPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
