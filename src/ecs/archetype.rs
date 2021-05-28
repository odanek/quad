use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use super::{
    bundle::BundleId, component::ComponentId, entity::EntityLocation, storage::TableId, Entity,
};

#[derive(Default)]
pub struct Edges {
    pub add_bundle: HashMap<BundleId, ArchetypeId>,
    pub remove_bundle: HashMap<BundleId, ArchetypeId>,
}

impl Edges {
    #[inline]
    pub fn get_add_bundle(&self, bundle_id: BundleId) -> Option<&ArchetypeId> {
        self.add_bundle.get(&bundle_id)
    }

    #[inline]
    pub fn set_add_bundle(&mut self, bundle_id: BundleId, archetype_id: ArchetypeId) {
        self.add_bundle.insert(bundle_id, archetype_id);
    }

    #[inline]
    pub fn get_remove_bundle(&self, bundle_id: BundleId) -> Option<ArchetypeId> {
        self.remove_bundle.get(&bundle_id).cloned()
    }

    #[inline]
    pub fn set_remove_bundle(&mut self, bundle_id: BundleId, archetype_id: ArchetypeId) {
        self.remove_bundle.insert(bundle_id, archetype_id);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ArchetypeId(usize);

impl ArchetypeId {
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct ArchetypeIdentity {
    components: Vec<ComponentId>,
}

impl ArchetypeIdentity {
    pub fn new(components: &[ComponentId]) -> Self {
        Self {
            components: components.to_vec(),
        }
    }
}

pub struct Archetype {
    id: ArchetypeId,
    table_id: TableId,
    entities: Vec<Entity>,
    edges: Edges,
    components: Vec<ComponentId>,
}

impl Archetype {
    #[inline]
    pub fn new(id: ArchetypeId, components: &[ComponentId], table_id: TableId) -> Self {
        Self {
            id,
            table_id,
            entities: Default::default(),
            edges: Default::default(),
            components: components.to_vec(),
        }
    }

    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    #[inline]
    pub fn table_id(&self) -> TableId {
        self.table_id
    }

    #[inline]
    pub fn edges(&self) -> &Edges {
        &self.edges
    }

    #[inline]
    pub(crate) fn edges_mut(&mut self) -> &mut Edges {
        &mut self.edges
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[inline]
    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.components.contains(&component_id)
    }

    #[inline]
    pub fn components(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.components.iter().cloned()
    }

    pub fn next_location(&self) -> EntityLocation {
        EntityLocation {
            archetype_id: self.id,
            row: self.entities.len(),
        }
    }

    pub unsafe fn allocate(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<Entity> {
        let is_last = index == self.entities.len() - 1;
        self.entities.swap_remove(index);

        if is_last {
            None
        } else {
            Some(self.entities[index])
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArchetypeGeneration(u32);

impl ArchetypeGeneration {
    #[inline]
    pub fn new(generation: u32) -> Self {
        ArchetypeGeneration(generation)
    }

    #[inline]
    pub fn value(self) -> u32 {
        self.0
    }
}

pub struct Archetypes {
    archetypes: Vec<Archetype>,
    archetype_ids: HashMap<ArchetypeIdentity, ArchetypeId>,
}

impl Default for Archetypes {
    fn default() -> Self {
        let mut archetypes = Archetypes {
            archetypes: Vec::new(),
            archetype_ids: Default::default(),
        };

        archetypes.get_id_or_insert(TableId::empty(), &[]);
        archetypes
    }
}

impl Archetypes {
    #[inline]
    pub fn generation(&self) -> ArchetypeGeneration {
        ArchetypeGeneration(self.archetypes.len() as u32)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    #[inline]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.index())
    }

    #[inline]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.index())
    }

    #[inline]
    pub fn empty(&self) -> &Archetype {
        unsafe { self.archetypes.get_unchecked(ArchetypeId::empty().index()) }
    }

    #[inline]
    pub fn empty_mut(&mut self) -> &mut Archetype {
        unsafe {
            self.archetypes
                .get_unchecked_mut(ArchetypeId::empty().index())
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    pub fn get_id_or_insert(
        &mut self,
        table_id: TableId,
        components: &[ComponentId],
    ) -> ArchetypeId {
        let archetypes = &mut self.archetypes;
        let identity = ArchetypeIdentity::new(components);

        *self.archetype_ids.entry(identity).or_insert_with(move || {
            let id = ArchetypeId(archetypes.len());

            archetypes.push(Archetype::new(id, components, table_id));
            id
        })
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
