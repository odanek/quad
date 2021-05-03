#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.0
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
