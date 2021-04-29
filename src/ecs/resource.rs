use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};
#[derive(Default)]
pub struct Resources {
    map: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl Resources {
    pub fn add<T: 'static>(&mut self, resource: Box<T>) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, RefCell::new(resource));
        // .expect_none("Resource already exists"); // TODO
    }

    pub fn remove<T: 'static>(&mut self) -> Option<Box<T>> {
        let type_id = TypeId::of::<T>();
        let resource = self.map.remove(&type_id)?;
        Some(resource.into_inner().downcast::<T>().unwrap())
    }

    pub fn get<T: 'static>(&self) -> Option<Ref<T>> {
        let type_id = TypeId::of::<T>();
        let resource = self.map.get(&type_id)?;        
        let borrowed = resource.borrow();
        Some(Ref::map(borrowed, |value| value.downcast_ref::<T>().unwrap()))
    }

    pub fn get_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        let type_id = TypeId::of::<T>();
        let resource = self.map.get(&type_id)?;        
        let borrowed = resource.borrow_mut();
        Some(RefMut::map(borrowed, |value| value.downcast_mut::<T>().unwrap()))
    }
}
