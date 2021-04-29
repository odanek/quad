use std::cell::{Ref, RefMut};

use super::{Entities, Entity, EntityMut, EntityRef, Resources};

// Struct of arrays
#[derive(Default)]
pub struct World {
    resources: Resources, 
    entities: Entities,
}

impl World {
    #[inline]
    pub fn add_resource<T: 'static>(&mut self, resource: Box<T>) {
        self.resources.add(resource);
    }

    #[inline]
    pub fn remove_resource<T: 'static>(&mut self) -> Option<Box<T>> {
        self.resources.remove()
    }

    #[inline]
    pub fn get_resource<T: 'static>(&self) -> Option<Ref<T>> {
        self.resources.get()
    }

    #[inline]
    pub fn resource<T: 'static>(&self) -> Ref<T> {
        self.get_resource().expect("Resource not found")
    }

    #[inline]
    pub fn get_resource_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        self.resources.get_mut()
    }

    #[inline]
    pub fn resource_mut<T: 'static>(&self) -> RefMut<T> {
        self.get_resource_mut().expect("Resource not found")
    }

    #[inline]
    pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
        self.entities.get(entity)
    }

    #[inline]
    pub fn entity(&self, entity: Entity) -> EntityRef {
        self.get_entity(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn get_entity_mut(&mut self, entity: Entity) -> Option<EntityMut> {
        self.entities.get_mut(entity)
    }

    #[inline]
    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut {
        self.get_entity_mut(entity).expect("Entity does not exist")
    }
}
