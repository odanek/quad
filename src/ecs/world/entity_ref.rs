use std::any::TypeId;

use crate::ecs::{
    archetype::Archetype, bundle::Bundle, component::Component, entity::EntityLocation, Entity,
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
        // let current_location = self.location;

        // // Use a non-generic function to cut down on monomorphization
        // unsafe fn get_insert_bundle_info<'a>(
        //     entities: &mut Entities,
        //     archetypes: &'a mut Archetypes,
        //     components: &mut Components,
        //     storages: &mut Storages,
        //     bundle_info: &BundleInfo,
        //     current_location: EntityLocation,
        //     entity: Entity,
        // ) -> (&'a Archetype, &'a Vec<ComponentStatus>, EntityLocation) {
        //     // SAFE: component ids in `bundle_info` and self.location are valid
        //     let new_archetype_id = add_bundle_to_archetype(
        //         archetypes,
        //         storages,
        //         components,
        //         current_location.archetype_id,
        //         bundle_info,
        //     );
        //     if new_archetype_id == current_location.archetype_id {
        //         let archetype = &archetypes[current_location.archetype_id];
        //         let edge = archetype.edges().get_add_bundle(bundle_info.id).unwrap();
        //         (archetype, &edge.bundle_status, current_location)
        //     } else {
        //         let (old_table_row, old_table_id) = {
        //             let old_archetype = &mut archetypes[current_location.archetype_id];
        //             let result = old_archetype.swap_remove(current_location.index);
        //             if let Some(swapped_entity) = result.swapped_entity {
        //                 entities.meta[swapped_entity.id as usize].location = current_location;
        //             }
        //             (result.table_row, old_archetype.table_id())
        //         };

        //         let new_table_id = archetypes[new_archetype_id].table_id();

        //         let new_location = if old_table_id == new_table_id {
        //             archetypes[new_archetype_id].allocate(entity, old_table_row)
        //         } else {
        //             let (old_table, new_table) =
        //                 storages.tables.get_2_mut(old_table_id, new_table_id);
        //             // PERF: store "non bundle" components in edge, then just move those to avoid
        //             // redundant copies
        //             let move_result =
        //                 old_table.move_to_superset_unchecked(old_table_row, new_table);

        //             let new_location =
        //                 archetypes[new_archetype_id].allocate(entity, move_result.new_row);
        //             // if an entity was moved into this entity's table spot, update its table row
        //             if let Some(swapped_entity) = move_result.swapped_entity {
        //                 let swapped_location = entities.get(swapped_entity).unwrap();
        //                 archetypes[swapped_location.archetype_id]
        //                     .set_entity_table_row(swapped_location.index, old_table_row);
        //             }
        //             new_location
        //         };

        //         entities.meta[entity.id as usize].location = new_location;
        //         let (old_archetype, new_archetype) =
        //             archetypes.get_2_mut(current_location.archetype_id, new_archetype_id);
        //         let edge = old_archetype
        //             .edges()
        //             .get_add_bundle(bundle_info.id)
        //             .unwrap();
        //         (&*new_archetype, &edge.bundle_status, new_location)

        //         // Sparse set components are intentionally ignored here. They don't need to move
        //     }
        // }

        // let (archetype, bundle_status, new_location) = unsafe {
        //     get_insert_bundle_info(
        //         entities,
        //         archetypes,
        //         components,
        //         storages,
        //         bundle_info,
        //         current_location,
        //         entity,
        //     )
        // };
        // self.location = new_location;

        // let table = &storages.tables[archetype.table_id()];
        // let table_row = archetype.entity_table_row(new_location.index);
        // // SAFE: table row is valid
        // unsafe {
        //     bundle_info.write_components(
        //         &mut storages.sparse_sets,
        //         entity,
        //         table,
        //         table_row,
        //         bundle_status,
        //         bundle,
        //         change_tick,
        //     )
        // };
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
