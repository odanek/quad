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
        unsafe { bundle_info.write_components(table, new_location.row, bundle) };

        self
    }

    pub fn insert<T: Component>(&mut self, value: T) -> &mut Self {
        self.insert_bundle((value,))
    }

    pub fn remove_bundle<T: Bundle>(&mut self) -> Option<T> {
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
                bundle_info
            )?
        };

        if new_archetype_id == old_location.archetype_id {
            return None;
        }

        let old_archetype = &mut archetypes[old_location.archetype_id];
        let mut bundle_components = bundle_info.component_ids.iter().cloned();
        let entity = self.entity;

        // let result = unsafe {
        //     T::from_components(|| {
        //         let component_id = bundle_components.next().unwrap();
        //         // SAFE: entity location is valid and table row is removed below
        //         take_component(
        //             components,
        //             storages,
        //             old_archetype,
        //             removed_components,
        //             component_id,
        //             entity,
        //             old_location,
        //         )
        //     })
        // };

        // let remove_result = old_archetype.swap_remove(old_location.index);
        // if let Some(swapped_entity) = remove_result.swapped_entity {
        //     entities.meta[swapped_entity.id as usize].location = old_location;
        // }
        // let old_table_row = remove_result.table_row;
        // let old_table_id = old_archetype.table_id();
        // let new_archetype = &mut archetypes[new_archetype_id];

        // let new_location = if old_table_id == new_archetype.table_id() {
        //     unsafe { new_archetype.allocate(entity, old_table_row) }
        // } else {
        //     let (old_table, new_table) = storages
        //         .tables
        //         .get_2_mut(old_table_id, new_archetype.table_id());

        //     // SAFE: table_row exists. All "missing" components have been extracted into the bundle
        //     // above and the caller takes ownership
        //     let move_result =
        //         unsafe { old_table.move_to_and_forget_missing_unchecked(old_table_row, new_table) };

        //     // SAFE: new_table_row is a valid position in new_archetype's table
        //     let new_location = unsafe { new_archetype.allocate(entity, move_result.new_row) };

        //     // if an entity was moved into this entity's table spot, update its table row
        //     if let Some(swapped_entity) = move_result.swapped_entity {
        //         let swapped_location = entities.get(swapped_entity).unwrap();
        //         let archetype = &mut archetypes[swapped_location.archetype_id];
        //         archetype.set_entity_table_row(swapped_location.index, old_table_row);
        //     }

        //     new_location
        // };

        // self.location = new_location;
        // entities.meta[self.entity.id as usize].location = new_location;

        // Some(result)
        None
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

unsafe fn remove_bundle_from_archetype(
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    components: &mut Components,
    archetype_id: ArchetypeId,
    bundle_info: &BundleInfo
) -> Option<ArchetypeId> {
    // let remove_bundle_result = {
    //     let current_archetype = &mut archetypes[archetype_id];        
    //     current_archetype.edges().get_remove_bundle(bundle_info.id)
    // };
    // let result = if let Some(result) = remove_bundle_result {
    //     result
    // } else {
    //     let mut next_table_components;
    //     let mut next_sparse_set_components;
    //     let next_table_id;
    //     {
    //         let current_archetype = &mut archetypes[archetype_id];
    //         let mut removed_table_components = Vec::new();
    //         let mut removed_sparse_set_components = Vec::new();
    //         for component_id in bundle_info.component_ids.iter().cloned() {
    //             if current_archetype.contains(component_id) {
    //                 let component_info = components.get_info_unchecked(component_id);
    //                 match component_info.storage_type() {
    //                     StorageType::Table => removed_table_components.push(component_id),
    //                     StorageType::SparseSet => removed_sparse_set_components.push(component_id),
    //                 }
    //             } else {
    //                 // a component in the bundle was not present in the entity's archetype, so this
    //                 // removal is invalid cache the result in the archetype
    //                 // graph
    //                 current_archetype
    //                     .edges_mut()
    //                     .set_remove_bundle(bundle_info.id, None);
    //                 return None;
    //             }
    //         }

    //         // sort removed components so we can do an efficient "sorted remove". archetype
    //         // components are already sorted
    //         removed_table_components.sort();
    //         removed_sparse_set_components.sort();
    //         next_table_components = current_archetype.table_components().to_vec();
    //         next_sparse_set_components = current_archetype.sparse_set_components().to_vec();
    //         sorted_remove(&mut next_table_components, &removed_table_components);
    //         sorted_remove(
    //             &mut next_sparse_set_components,
    //             &removed_sparse_set_components,
    //         );

    //         next_table_id = if removed_table_components.is_empty() {
    //             current_archetype.table_id()
    //         } else {
    //             // SAFE: all components in next_table_components exist
    //             storages
    //                 .tables
    //                 .get_id_or_insert(&next_table_components, components)
    //         };
    //     }

    //     let new_archetype_id = archetypes.get_id_or_insert(
    //         next_table_id,
    //         next_table_components,
    //         next_sparse_set_components,
    //     );
    //     Some(new_archetype_id)
    // };
    // let current_archetype = &mut archetypes[archetype_id];
    // // cache the result in an edge
    // if intersection {
    //     current_archetype
    //         .edges_mut()
    //         .set_remove_bundle_intersection(bundle_info.id, result);
    // } else {
    //     current_archetype
    //         .edges_mut()
    //         .set_remove_bundle(bundle_info.id, result);
    // }
    // result

    None
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