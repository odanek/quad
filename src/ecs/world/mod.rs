mod entity_ref;

use self::entity_ref::{EntityMut, EntityRef};

use super::{Entities, Entity, Resources, component::Components, resource::Resource, storage::Storages};

// Struct of arrays
#[derive(Default)]
pub struct World {
    resources: Resources,
    entities: Entities,
    components: Components,
    storages: Storages,
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
    pub fn add_resource<T: Resource>(&mut self, resource: Box<T>) {
        self.resources.add(resource);
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<Box<T>> {
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
    pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
        None
        // self.entities.get(entity)
    }

    #[inline]
    pub fn entity(&self, entity: Entity) -> EntityRef {
        self.get_entity(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn get_entity_mut(&mut self, entity: Entity) -> Option<EntityMut> {
        None
        // self.entities.get_mut(entity)
    }

    #[inline]
    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut {
        self.get_entity_mut(entity).expect("Entity does not exist")
    }
}
