use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

pub struct ResourceContainer {
    map: HashMap<TypeId, RefCell<Box<dyn Any>>>,
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
        self.map.insert(type_id, RefCell::new(resource));
        // .expect_none("Resource already exists"); // TODO
    }

    pub fn remove<T: 'static>(&mut self) -> Box<T> {
        let type_id = TypeId::of::<T>();
        self.map
            .remove(&type_id)
            .expect("Resource not found")
            .into_inner()
            .downcast::<T>()
            .expect("Invalid resource type")
    }

    pub fn get<T: 'static>(&self) -> Ref<T> {
        let type_id = TypeId::of::<T>();
        let borrowed = self.map.get(&type_id).expect("Resource not found").borrow();
        Ref::map(borrowed, |value| {
            value.downcast_ref::<T>().expect("Invalid resource type")
        })
    }

    pub fn get_mut<T: 'static>(&self) -> RefMut<T> {
        let type_id = TypeId::of::<T>();
        let borrowed = self
            .map
            .get(&type_id)
            .expect("Resource not found")
            .borrow_mut();
        RefMut::map(borrowed, |value| {
            value.downcast_mut::<T>().expect("Invalid resource type")
        })
    }
}
