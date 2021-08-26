mod entity_ref;

use std::{any::TypeId, collections::HashMap};

use self::entity_ref::{EntityMut, EntityRef};

use super::{
    component::{bundle::Bundles, Component, ComponentId, Components},
    entity::{
        archetype::{Archetype, ArchetypeId, Archetypes},
        Entities, EntityLocation,
    },
    query::{fetch::WorldQuery, state::QueryState},
    resource::{Resource, ResourceId, Resources},
    storage::Storages,
    Entity,
};

#[derive(Default)]
pub struct World {
    resources: Resources,
    archetypes: Archetypes,
    entities: Entities,
    components: Components,
    storages: Storages,
    bundles: Bundles,
    removed_components: HashMap<ComponentId, Vec<Entity>>,
}

#[allow(dead_code)]
impl World {
    #[inline]
    pub fn new() -> World {
        World::default()
    }

    #[inline]
    pub(crate) fn resources(&self) -> &Resources {
        &self.resources
    }

    #[inline]
    pub(crate) fn entities(&self) -> &Entities {
        &self.entities
    }

    #[inline]
    pub(crate) fn archetypes(&self) -> &Archetypes {
        &self.archetypes
    }

    #[inline]
    pub(crate) fn archetype(&self, id: ArchetypeId) -> &Archetype {
        &self.archetypes[id]
    }

    #[inline]
    pub(crate) fn components(&self) -> &Components {
        &self.components
    }

    #[inline]
    pub(crate) fn storages(&self) -> &Storages {
        &self.storages
    }

    #[inline]
    pub(crate) fn bundles(&self) -> &Bundles {
        &self.bundles
    }

    #[inline]
    pub(crate) fn resource_id<T: Resource>(&self) -> Option<ResourceId> {
        self.resources.get_id::<T>()
    }

    #[inline]
    pub(crate) fn register_resource<T: Resource>(&mut self) -> ResourceId {
        self.resources.get_or_insert_id::<T>()
    }

    #[inline]
    pub(crate) fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.components.get_id(TypeId::of::<T>())
    }

    #[inline]
    pub(crate) fn register_component<T: Component>(&mut self) -> ComponentId {
        self.components.get_or_insert::<T>()
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources.add(resource)
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get()
    }

    #[inline]
    pub fn resource<T: Resource>(&self) -> &T {
        self.get_resource().expect("Resource not found")
    }

    #[inline]
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }

    #[inline]
    pub fn resource_mut<T: Resource>(&mut self) -> &mut T {
        self.get_resource_mut().expect("Resource not found")
    }

    #[inline]
    pub(crate) fn get_component<T: Component>(&self, location: EntityLocation) -> Option<&T> {
        unsafe { get_component(self, TypeId::of::<T>(), location).map(|value| &*value.cast::<T>()) }
    }

    #[inline]
    pub(crate) fn get_component_mut<T: Component>(
        &self,
        location: EntityLocation,
    ) -> Option<&mut T> {
        unsafe {
            get_component(self, TypeId::of::<T>(), location).map(|value| &mut *value.cast::<T>())
        }
    }

    #[inline]
    pub(crate) fn component_unchecked_mut<T: Component>(
        &self,
        location: EntityLocation,
    ) -> Option<&mut T> {
        unsafe {
            get_component(self, TypeId::of::<T>(), location).map(|value| &mut *value.cast::<T>())
        }
    }

    pub fn spawn(&mut self) -> EntityMut {
        let archetype = self.archetypes.empty_mut();
        let location = archetype.next_location();
        let entity = self.entities.alloc(location);
        let table = &mut self.storages.tables[archetype.table_id()];
        archetype.allocate(entity);
        unsafe {
            table.allocate(entity);
            EntityMut::new(self, entity, location)
        }
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.get_entity_mut(entity)
            .map(|e| {
                e.despawn();
                true
            })
            .unwrap_or(false)
    }

    pub fn despawn_all(&mut self) {
        for archetype in self.archetypes.iter_mut() {
            for entity in archetype.entities() {
                self.entities.free(*entity);
            }
            archetype.clear();
            unsafe {
                self.storages.tables[archetype.table_id()].clear();
            }
        }
    }

    #[inline]
    pub fn entity(&self, entity: Entity) -> EntityRef {
        self.get_entity(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
        let location = self.entities.get(entity)?;
        Some(EntityRef::new(self, entity, location))
    }

    #[inline]
    pub fn get_entity_mut(&mut self, entity: Entity) -> Option<EntityMut> {
        let location = self.entities.get(entity)?;
        Some(EntityMut::new(self, entity, location))
    }

    #[inline]
    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut {
        self.get_entity_mut(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn query<Q: WorldQuery>(&mut self) -> QueryState<Q, ()> {
        QueryState::new(self)
    }

    pub fn removed<T: Component>(&self) -> std::iter::Cloned<std::slice::Iter<'_, Entity>> {
        self.component_id::<T>()
            .map_or_else(|| [].iter().cloned(), |id| self.removed_with_id(id))
    }

    pub(crate) fn removed_with_id(
        &self,
        component_id: ComponentId,
    ) -> std::iter::Cloned<std::slice::Iter<'_, Entity>> {
        self.removed_components
            .get(&component_id)
            .map_or_else(|| [].iter().cloned(), |list| list.iter().cloned())
    }

    pub(crate) fn clear_trackers(&mut self) {
        for entities in self.removed_components.values_mut() {
            entities.clear();
        }
    }

    pub(crate) fn flush(&mut self) {
        let archetype = self.archetypes.empty_mut();
        let table = &mut self.storages.tables[archetype.table_id()];
        unsafe {
            self.entities.flush(|entity| {
                let location = archetype.next_location();
                archetype.allocate(entity);
                table.allocate(entity);
                location
            });
        }
    }
}

pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_world: &mut World) -> Self {
        T::default()
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
