mod entity_ref;

use self::entity_ref::{EntityMut, EntityRef};

use super::{
    component::{bundle::Bundles, Components},
    entity::archetype::Archetypes,
    resource::{Resource, ResourceId},
    storage::Storages,
    Entities, Entity, Resources,
};

#[derive(Default)]
pub struct World {
    resources: Resources,
    archetypes: Archetypes,
    entities: Entities,
    components: Components,
    storages: Storages,
    bundles: Bundles,
}

impl World {
    #[inline]
    pub fn new() -> World {
        World::default()
    }

    #[inline]
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    #[inline]
    pub fn archetypes(&self) -> &Archetypes {
        &self.archetypes
    }

    #[inline]
    pub fn components(&self) -> &Components {
        &self.components
    }

    #[inline]
    pub fn storages(&self) -> &Storages {
        &self.storages
    }

    #[inline]
    pub fn bundles(&self) -> &Bundles {
        &self.bundles
    }

    #[inline]
    pub fn resource_id<T: Resource>(&self) -> Option<ResourceId> {
        self.resources.get_id::<T>()
    }

    #[inline]
    pub fn add_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
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

    pub fn spawn(&mut self) -> EntityMut {
        let archetype = self.archetypes.empty_mut();
        let location = archetype.next_location();
        let entity = self.entities.alloc(location);
        let table = &mut self.storages.tables[archetype.table_id()];
        archetype.allocate(entity);
        unsafe {
            table.allocate();
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
                self.entities.free(entity);
            }
            archetype.clear();
            unsafe {
                self.storages.tables[archetype.table_id()].clear();
            }
        }
    }

    #[inline]
    pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
        let location = self.entities.get(entity)?;
        Some(EntityRef::new(self, entity, location))
    }

    #[inline]
    pub fn entity(&self, entity: Entity) -> EntityRef {
        self.get_entity(entity).expect("Entity does not exist")
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
}

pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_world: &mut World) -> Self {
        T::default()
    }
}
