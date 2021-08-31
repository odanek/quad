use std::ops::Deref;

use crate::ecs::Entity;

#[derive(Eq, PartialEq, Clone, Debug, Default)]
pub struct Children(pub(crate) Vec<Entity>);

impl Children {
    pub fn with(entity: &[Entity]) -> Self {
        let mut data = Vec::with_capacity(8);
        data.extend_from_slice(entity);
        Self(data)
    }
}

impl Deref for Children {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
