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
        &self.world.archetypes[self.location.archetype_id]
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
        &self.world.archetypes[self.location.archetype_id]
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

        let new_location = unsafe {
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

        let archetype = &archetypes[new_location.archetype_id];
        let table = &storages.tables[archetype.table_id()];

        // TODO: If we overwrite, where are old components dropped?
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

unsafe fn get_insert_bundle_info(
    entities: &mut Entities,
    archetypes: &mut Archetypes,
    components: &mut Components,
    storages: &mut Storages,
    bundle_info: &BundleInfo,
    current_location: EntityLocation,
    entity: Entity,
) -> EntityLocation {
    let new_archetype_id = add_bundle_to_archetype(
        archetypes,
        storages,
        components,
        current_location.archetype_id,
        bundle_info,
    );

    if new_archetype_id == current_location.archetype_id {
        current_location
    } else {
        let old_archetype = &mut archetypes[current_location.archetype_id];        
        let result = old_archetype.swap_remove(current_location.row);
        if let Some(swapped_entity) = result {
            entities.update_location(swapped_entity, current_location)
        }        

        let old_table_id = old_archetype.table_id();
        let new_table_id = archetypes[new_archetype_id].table_id();
        let (old_table, new_table) = storages.tables.get_2_mut(old_table_id, new_table_id);
        old_table.move_to_superset_unchecked(current_location.row, new_table);

        let new_location = archetypes[new_archetype_id].next_location();            
        archetypes[new_archetype_id].allocate(entity);        

        entities.update_location(entity, new_location);

        new_location
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
    let mut new_components = Vec::new();

    let current_archetype = &mut archetypes[archetype_id];
    for component_id in bundle_info.component_ids.iter().cloned() {
        if !current_archetype.contains(component_id) {
            new_components.push(component_id)
        }
    }

    if new_components.is_empty() {
        let edges = current_archetype.edges_mut();
        edges.set_add_bundle(bundle_info.id, archetype_id);
        archetype_id
    } else {
        let current_archetype = &archetypes[archetype_id];
        new_components.extend(current_archetype.components());
        new_components.sort();

        let table_id = storages
            .tables
            .get_id_or_insert(&new_components, components);

        let new_archetype_id = archetypes.get_id_or_insert(table_id, &new_components);

        archetypes[archetype_id]
            .edges_mut()
            .set_add_bundle(bundle_info.id, new_archetype_id);
        new_archetype_id
    }
}
