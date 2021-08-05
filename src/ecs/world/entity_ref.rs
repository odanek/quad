use std::any::TypeId;

use crate::ecs::{
    component::{
        bundle::{Bundle, BundleInfo},
        Component, ComponentStatus, Components,
    },
    entity::{
        archetype::{Archetype, ArchetypeId, Archetypes},
        Entities, EntityLocation,
    },
    storage::Storages,
    Entity,
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

    #[inline]
    pub fn get<T: Component>(&self) -> Option<&'w T> {
        unsafe {
            get_component(self.world, TypeId::of::<T>(), self.location)
                .map(|value| &*value.cast::<T>())
        }
    }

    #[inline]
    pub(crate) fn get_unchecked_mut<T: Component>(&self) -> Option<&'w mut T> {
        unsafe {
            get_component(self.world, TypeId::of::<T>(), self.location)
                .map(|value| &mut *value.cast::<T>())
        }
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
        unsafe {
            get_component(self.world, TypeId::of::<T>(), self.location)
                .map(|value| &*value.cast::<T>())
        }
    }

    #[inline]
    pub fn get_mut<T: Component>(&mut self) -> Option<&'w mut T> {
        unsafe {
            get_component(self.world, TypeId::of::<T>(), self.location)
                .map(|value| &mut *value.cast::<T>())
        }
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

        let edge = archetypes[current_location.archetype_id]
            .edges()
            .get_add_bundle(bundle_info.id)
            .unwrap();
        let archetype = &archetypes[new_location.archetype_id];
        let table = &mut storages.tables[archetype.table_id()];

        // TODO: If we overwrite, where are old components dropped?
        unsafe {
            bundle_info.write_components(table, new_location.index, bundle, &edge.bundle_status)
        };

        self
    }

    pub fn insert(&mut self, value: impl Component) -> &mut Self {
        self.insert_bundle((value,))
    }

    pub fn remove_bundle<T: Bundle>(&mut self) -> T {
        let archetypes = &mut self.world.archetypes;
        let storages = &mut self.world.storages;
        let components = &mut self.world.components;
        let entities = &mut self.world.entities;

        let bundle_info = self.world.bundles.init_info::<T>(components);
        let old_location = self.location;
        let new_archetype_id = unsafe {
            remove_bundle_from_archetype(
                archetypes,
                storages,
                components,
                old_location.archetype_id,
                bundle_info,
            )
        };

        let old_archetype = &mut archetypes[old_location.archetype_id];
        let mut bundle_components = bundle_info.component_ids.iter().cloned();
        let entity = self.entity;

        let result = unsafe {
            T::from_components(|| {
                let component_id = bundle_components.next().unwrap();
                let table = &storages.tables[old_archetype.table_id()];
                let column = table
                    .get_column(component_id)
                    .expect("The entity does not contain given component");
                column.get_unchecked(old_location.index)
            })
        };

        if new_archetype_id == old_location.archetype_id {
            return result;
        }

        let remove_result = old_archetype.swap_remove(old_location.index);
        if let Some(swapped_entity) = remove_result {
            entities.update_location(swapped_entity, old_location);
        }
        let old_table_row = old_location.index;
        let old_table_id = old_archetype.table_id();
        let new_archetype = &mut archetypes[new_archetype_id];

        let (old_table, new_table) = storages
            .tables
            .get_2_mut(old_table_id, new_archetype.table_id());

        unsafe { old_table.move_to_and_forget_missing_unchecked(old_table_row, new_table) };

        let new_location = new_archetype.next_location();
        new_archetype.allocate(entity);

        self.location = new_location;
        entities.update_location(entity, new_location);

        result
    }

    pub fn remove_bundle_intersection<T: Bundle>(&mut self) {
        // TODO: Implement
    }

    pub fn remove<T: Component>(&mut self) -> T {
        self.remove_bundle::<(T,)>().0
    }

    pub fn despawn(self) {
        let world = self.world;
        let location = world
            .entities
            .free(self.entity)
            .expect("Despawned entity does not exist");

        let archetype = &mut world.archetypes[location.archetype_id];
        let remove_result = archetype.swap_remove(location.index);
        if let Some(swapped_entity) = remove_result {
            world.entities.update_location(swapped_entity, location);
        }

        let table_row = location.index;
        unsafe { world.storages.tables[archetype.table_id()].swap_remove_unchecked(table_row) };
    }
}

unsafe fn get_component(
    world: &World,
    type_id: TypeId,
    location: EntityLocation,
) -> Option<*mut u8> {
    let component_id = world.components.get_id(type_id)?;
    let archetype = &world.archetypes[location.archetype_id];
    let table = &world.storages.tables[archetype.table_id()];
    let column = table.get_column(component_id)?;
    Some(column.get_unchecked(location.index))
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
        let result = old_archetype.swap_remove(current_location.index);
        if let Some(swapped_entity) = result {
            entities.update_location(swapped_entity, current_location)
        }

        let old_table_id = old_archetype.table_id();
        let new_table_id = archetypes[new_archetype_id].table_id();
        let (old_table, new_table) = storages.tables.get_2_mut(old_table_id, new_table_id);
        old_table.move_to_superset_unchecked(current_location.index, new_table);

        let new_archetype = &mut archetypes[new_archetype_id];
        let new_location = new_archetype.next_location();
        new_archetype.allocate(entity);

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
        return add_bundle.archetype_id;
    }
    let mut new_components = Vec::new();
    let mut bundle_status = Vec::with_capacity(bundle_info.component_ids.len());

    let current_archetype = &mut archetypes[archetype_id];
    for component_id in bundle_info.component_ids.iter().cloned() {
        if current_archetype.contains(component_id) {
            bundle_status.push(ComponentStatus::Mutated);
        } else {
            bundle_status.push(ComponentStatus::Added);
            new_components.push(component_id);
        }
    }

    if new_components.is_empty() {
        let edges = current_archetype.edges_mut();
        edges.set_add_bundle(bundle_info.id, archetype_id, bundle_status);
        archetype_id
    } else {
        let current_archetype = &archetypes[archetype_id];
        new_components.extend(current_archetype.components());
        new_components.sort();

        let table_id = storages
            .tables
            .get_id_or_insert(&new_components, components);

        let new_archetype_id = archetypes.get_id_or_insert(table_id, &new_components);

        archetypes[archetype_id].edges_mut().set_add_bundle(
            bundle_info.id,
            new_archetype_id,
            bundle_status,
        );
        new_archetype_id
    }
}

unsafe fn remove_bundle_from_archetype(
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    components: &mut Components,
    archetype_id: ArchetypeId,
    bundle_info: &BundleInfo,
) -> ArchetypeId {
    let current_archetype = &mut archetypes[archetype_id];

    let remove_bundle_result = current_archetype.edges().get_remove_bundle(bundle_info.id);
    if let Some(result) = remove_bundle_result {
        return result;
    }

    let mut removed_components = bundle_info.component_ids.clone();
    removed_components.sort();
    let mut next_components = current_archetype.components().collect();
    sorted_remove(&mut next_components, &removed_components);

    let next_table_id = if removed_components.is_empty() {
        current_archetype.table_id()
    } else {
        storages
            .tables
            .get_id_or_insert(&next_components, components)
    };

    let new_archetype_id = archetypes.get_id_or_insert(next_table_id, &next_components);

    archetypes[archetype_id]
        .edges_mut()
        .set_remove_bundle(bundle_info.id, new_archetype_id);

    new_archetype_id
}

fn sorted_remove<T: Eq + Ord + Copy>(source: &mut Vec<T>, remove: &[T]) {
    let mut remove_index = 0;
    source.retain(|value| {
        while remove_index < remove.len() && *value > remove[remove_index] {
            remove_index += 1;
        }

        if remove_index < remove.len() {
            *value != remove[remove_index]
        } else {
            true
        }
    })
}
