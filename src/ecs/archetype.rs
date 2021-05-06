use super::storage::TableId;

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
}

impl Archetype {
    #[inline]
    pub fn new(id: ArchetypeId, table_id: TableId) -> Self {
        Self {
            id,
            table_id
        }
    }

    #[inline]
    pub fn table_id(&self) -> TableId {
        self.table_id
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

        archetypes.archetypes.push(Archetype::new(ArchetypeId::empty(), TableId::empty()));
        archetypes
    }
}

impl Archetypes {
    #[inline]
    pub fn empty_mut(&mut self) -> &mut Archetype {
        unsafe {
            self.archetypes.get_unchecked_mut(ArchetypeId::empty().index())
        }
    }
}