use std::{ops::Deref, slice};

use crate::ecs::{Component, Entity};

#[derive(Eq, PartialEq, Clone, Debug, Default, Component)]
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

impl<'a> IntoIterator for &'a Children {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
