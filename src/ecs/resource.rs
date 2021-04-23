use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct ResourceContainer {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for ResourceContainer {
    fn default() -> Self {
        ResourceContainer {
            map: Default::default(),
        }
    }
}

impl ResourceContainer {
    pub fn add<T: 'static>(&mut self, resource: Box<T>) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, resource);
        // .expect_none("Resource already exists"); // TODO
    }

    pub fn remove<T: 'static>(&mut self) -> Box<T> {
        let type_id = TypeId::of::<T>();
        self.map
            .remove(&type_id)
            .expect("Resource not found")
            .downcast::<T>()
            .expect("Invalid resource type")
    }

    pub fn get<'a, T: 'static>(&'a self) -> &'a T {
        let type_id = TypeId::of::<T>();
        self.map
            .get(&type_id)
            .expect("Resource not found")
            .downcast_ref::<T>()
            .expect("Invalid resource type")
    }

    pub fn get_mut<'a, T: 'static>(&'a mut self) -> &'a mut T {
        let type_id = TypeId::of::<T>();
        self.map
            .get_mut(&type_id)
            .expect("Resource not found")
            .downcast_mut::<T>()
            .expect("Invalid resource type")
    }
}
