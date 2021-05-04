use std::collections::HashMap;

use crate::ecs::{archetype::ArchetypeId, component::ComponentId, Entity};

use super::BlobVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(u32);

impl TableId {
    #[inline]
    pub fn new(index: u32) -> Self {
        TableId(index)
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn empty() -> TableId {
        TableId(0)
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

pub struct Tables {
    tables: Vec<Table>,
    table_ids: HashMap<u32, TableId>,
}
