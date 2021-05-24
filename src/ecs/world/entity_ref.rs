use std::any::TypeId;

use crate::ecs::{
    archetype::{Archetype, ArchetypeId, Archetypes},
    bundle::{Bundle, BundleInfo},
    component::{Component, Components},
    entity::EntityLocation,
    storage::Storages,
    Entities, Entity,
};

use super::World;

pub struct EntityRef<'w> {
    world: &'w World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityRef<'w> {
    #[inline]
    pub(crate) fn new(world: &'w World, entity: Entity, location: EntityLocation) -> Self {
        EntityRef {
            world,
            entity,
            location,
        }
    }

    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn location(&self) -> EntityLocation {
        self.location
    }

    #[inline]
    pub fn archetype(&self) -> &Archetype {
        &self.world.archetypes[self.location.archetype]
    }

    #[inline]
    pub fn contains<T: Component>(&self) -> bool {
        self.world
            .components()
            .get_id(TypeId::of::<T>())
            .map_or(false, |id| self.archetype().contains(id))
    }
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityMut<'w> {
    #[inline]
    pub(crate) fn new(world: &'w mut World, entity: Entity, location: EntityLocation) -> Self {
        EntityMut {
            world,
            entity,
            location,
        }
    }

    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn location(&self) -> EntityLocation {
        self.location
    }

    #[inline]
    pub fn archetype(&self) -> &Archetype {
        &self.world.archetypes[self.location.archetype]
    }

    #[inline]
    pub fn contains<T: Component>(&self) -> bool {
        self.world
            .components()
            .get_id(TypeId::of::<T>())
            .map_or(false, |id| self.archetype().contains(id))
    }

    #[inline]
    pub fn get<T: Component>(&self) -> Option<&'w T> {
        todo!()
    }

    #[inline]
    pub fn get_mut<T: Component>(&mut self) -> Option<&'w mut T> {
        todo!()
    }

    // TODO: move relevant methods to World (add/remove bundle)
    pub fn insert_bundle<T: Bundle>(&mut self, bundle: T) -> &mut Self {
        let entity = self.entity;
        let entities = &mut self.world.entities;
        let archetypes = &mut self.world.archetypes;
        let components = &mut self.world.components;
        let storages = &mut self.world.storages;

        let bundle_info = self.world.bundles.init_info::<T>(components);
        let current_location = self.location;

        let (archetype, new_location) = unsafe {
            get_insert_bundle_info(
                entities,
                archetypes,
                components,
                storages,
                bundle_info,
                current_location,
                entity,
            )
        };
        self.location = new_location;

        let table = &storages.tables[archetype.table_id()];
        unsafe { bundle_info.write_components(entity, table, new_location.row, bundle) };

        self
    }

    pub fn remove_bundle<T: Bundle>(&mut self) -> Option<T> {
        todo!()
    }

    pub fn insert<T: Component>(&mut self, value: T) -> &mut Self {
        self.insert_bundle((value,))
    }

    pub fn remove<T: Component>(&mut self) -> Option<T> {
        self.remove_bundle::<(T,)>().map(|v| v.0)
    }

    pub fn despawn(self) {
        todo!()
    }
}

// Use a non-generic function to cut down on monomorphization
unsafe fn get_insert_bundle_info<'a>(
    entities: &mut Entities,
    archetypes: &'a mut Archetypes,
    components: &mut Components,
    storages: &mut Storages,
    bundle_info: &BundleInfo,
    current_location: EntityLocation,
    entity: Entity,
) -> (&'a Archetype, EntityLocation) {
    let new_archetype_id = add_bundle_to_archetype(
        archetypes,
        storages,
        components,
        current_location.archetype,
        bundle_info,
    );
    if new_archetype_id == current_location.archetype {
        let archetype = &archetypes[current_location.archetype];
        let edge = archetype.edges().get_add_bundle(bundle_info.id).unwrap();
        (archetype, &edge.bundle_status, current_location)
    } else {
        let (old_table_row, old_table_id) = {
            let old_archetype = &mut archetypes[current_location.archetype];
            let result = old_archetype.swap_remove(current_location.index);
            if let Some(swapped_entity) = result.swapped_entity {
                entities.meta[swapped_entity.id as usize].location = current_location;
            }
            (result.table_row, old_archetype.table_id())
        };

        let new_table_id = archetypes[new_archetype_id].table_id();

        let new_location = if old_table_id == new_table_id {
            archetypes[new_archetype_id].allocate(entity, old_table_row)
        } else {
            let (old_table, new_table) = storages.tables.get_2_mut(old_table_id, new_table_id);
            // PERF: store "non bundle" components in edge, then just move those to avoid
            // redundant copies
            let move_result = old_table.move_to_superset_unchecked(old_table_row, new_table);

            let new_location = archetypes[new_archetype_id].allocate(entity, move_result.new_row);
            // if an entity was moved into this entity's table spot, update its table row
            if let Some(swapped_entity) = move_result.swapped_entity {
                let swapped_location = entities.get(swapped_entity).unwrap();
                archetypes[swapped_location.archetype_id]
                    .set_entity_table_row(swapped_location.index, old_table_row);
            }
            new_location
        };

        entities.meta[entity.id as usize].location = new_location;
        let (old_archetype, new_archetype) =
            archetypes.get_2_mut(current_location.archetype, new_archetype_id);
        let edge = old_archetype
            .edges()
            .get_add_bundle(bundle_info.id)
            .unwrap();
        (&*new_archetype, &edge.bundle_status, new_location)

        // Sparse set components are intentionally ignored here. They don't need to move
    }
}

unsafe fn add_bundle_to_archetype(
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    components: &mut Components,
    archetype_id: ArchetypeId,
    bundle_info: &BundleInfo,
) -> ArchetypeId {
    if let Some(add_bundle) = archetypes[archetype_id]
        .edges()
        .get_add_bundle(bundle_info.id)
    {
        return *add_bundle;
    }
    let mut new_table_components = Vec::new();

    let current_archetype = &mut archetypes[archetype_id];
    for component_id in bundle_info.component_ids.iter().cloned() {
        if !current_archetype.contains(component_id) {
            let component_info = components.get_info(component_id).unwrap();
            new_table_components.push(component_id)
        }
    }

    if new_table_components.is_empty() {
        let edges = current_archetype.edges_mut();
        edges.set_add_bundle(bundle_info.id, archetype_id);
        archetype_id
    } else {
        let current_archetype = &archetypes[archetype_id];
        new_table_components.extend(current_archetype.components());
        // sort to ignore order while hashing
        new_table_components.sort();

        // SAFE: all component ids in `new_table_components` exist
        let table_id = storages
            .tables
            .get_id_or_insert(&new_table_components, components);

        let table_components = new_table_components;

        let new_archetype_id =
            archetypes.get_id_or_insert(table_id, table_components, sparse_set_components);
        // add an edge from the old archetype to the new archetype
        archetypes[archetype_id].edges_mut().set_add_bundle(
            bundle_info.id,
            new_archetype_id,
            bundle_status,
        );
        new_archetype_id
    }
}
