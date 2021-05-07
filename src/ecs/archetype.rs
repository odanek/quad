use std::ops::{Index, IndexMut};

use super::{Entity, entity::EntityLocation, storage::TableId};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

// pub struct Archetype {
//     id: ArchetypeId,
//     entities: Vec<Entity>,
//     edges: Edges,
//     table_info: TableInfo,
//     table_components: Cow<'static, [ComponentId]>,
//     pub(crate) components: SparseSet<ComponentId, ArchetypeComponentInfo>,
// }

pub struct Archetype {
    id: ArchetypeId,
    table_id: TableId,
    entities: Vec<Entity>,
}

impl Archetype {
    #[inline]
    pub fn new(id: ArchetypeId, table_id: TableId) -> Self {
        Self { id, table_id, entities: Vec::new() }
    }

    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    #[inline]
    pub fn table_id(&self) -> TableId {
        self.table_id
    }

    pub fn next_location(&self) -> EntityLocation {
        EntityLocation {
            archetype: self.id,
            row: self.entities.len() as u32,
        }
    }

    pub unsafe fn allocate(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

pub struct Archetypes {
    pub(crate) archetypes: Vec<Archetype>,
}

impl Default for Archetypes {
    fn default() -> Self {
        let mut archetypes = Archetypes {
            archetypes: Vec::new(),
        };

        archetypes
            .archetypes
            .push(Archetype::new(ArchetypeId::empty(), TableId::empty()));
        archetypes
    }
}

impl Archetypes {
    #[inline]
    pub fn empty_mut(&mut self) -> &mut Archetype {
        unsafe {
            self.archetypes
                .get_unchecked_mut(ArchetypeId::empty().index())
        }
    }
}

impl Index<ArchetypeId> for Archetypes {
    type Output = Archetype;

    #[inline]
    fn index(&self, index: ArchetypeId) -> &Self::Output {
        &self.archetypes[index.index()]
    }
}

impl IndexMut<ArchetypeId> for Archetypes {
    #[inline]
    fn index_mut(&mut self, index: ArchetypeId) -> &mut Self::Output {
        &mut self.archetypes[index.index()]
    }
}
