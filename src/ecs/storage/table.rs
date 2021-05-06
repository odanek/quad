use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::ecs::{archetype::ArchetypeId, component::ComponentId, Entity};

use super::BlobVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(u32);

impl TableId {
    #[inline]
    pub fn new(id: u32) -> Self {
        TableId(id)
    }

    #[inline]
    pub const fn empty() -> TableId {
        TableId(0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

pub struct Column {
    pub(crate) component_id: ComponentId,
    pub(crate) data: BlobVec,
}

pub struct Table {
    columns: HashMap<ComponentId, Column>,
    entities: Vec<Entity>,
    archetypes: Vec<ArchetypeId>,
    grow_amount: usize,
    capacity: usize,
}

#[derive(Default)]
pub struct Tables {
    tables: Vec<Table>,
}

impl Index<TableId> for Tables {
    type Output = Table;

    #[inline]
    fn index(&self, index: TableId) -> &Self::Output {
        &self.tables[index.index()]
    }
}

impl IndexMut<TableId> for Tables {
    #[inline]
    fn index_mut(&mut self, index: TableId) -> &mut Self::Output {
        &mut self.tables[index.index()]
    }
}
