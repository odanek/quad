use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

#[derive(Default)]
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn add<T: Resource>(&mut self, resource: Box<T>) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, resource);
        // .expect_none("Resource already exists"); // TODO
    }

    pub fn remove<T: Resource>(&mut self) -> Option<Box<T>> {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id)?.downcast().ok()
    }

    pub fn get<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)?.downcast_ref()
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.map.get_mut(&type_id)?.downcast_mut()
    }
}
