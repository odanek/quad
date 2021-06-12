use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

// TODO: ResourceId

#[derive(Default)]
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    #[inline]
    pub fn add<T: Resource>(&mut self, resource: T) {
        self.map.insert(resource.type_id(), Box::new(resource));
        // .expect_none("Resource already exists"); // TODO
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id)?.downcast().ok().map(|v| *v)
    }

    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)?.downcast_ref()
    }

    #[inline]
    pub unsafe fn get_unchecked<T: Resource>(&self) -> Option<*const T> {
        let type_id = TypeId::of::<T>();
        let reference = self.map.get(&type_id)?.downcast_mut()?;
        Some(reference as _)
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.map.get_mut(&type_id)?.downcast_mut()
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<T: Resource>(&self) -> Option<*mut T> {
        let type_id = TypeId::of::<T>();
        let reference = self.map.get_mut(&type_id)?.downcast_mut()?;
        unsafe { Some(reference as _) }
    }
}
