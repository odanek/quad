use std::{any::{Any, TypeId, type_name}, collections::HashMap};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ResourceId(usize);

#[derive(Debug)]
pub struct ResourceInfo {
    id: ResourceId,
    type_id: TypeId,
    name: &'static str,
}

impl ResourceInfo {
    #[inline]
    pub fn id(&self) -> ResourceId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

#[derive(Default)]
pub struct Resources {
    resources: Vec<ResourceInfo>,
    id_map: HashMap<TypeId, ResourceId>,
    map: HashMap<ResourceId, Box<dyn Any>>,
}

impl Resources {
    #[inline]
    pub fn get_id<T: Resource>(&self) -> Option<ResourceId> {
        self.id_map.get(&TypeId::of()).copied()
    }

    fn insert_id<T: Resource>(&mut self) -> ResourceId {
        let type_id = TypeId::of::<T>();
        let id = ResourceId(self.resources.len());
        let info = ResourceInfo {
            id,
            type_id,
            name: type_name::<T>()
        };
        self.resources.push(info);
        self.id_map.insert(type_id, id); // .expect_none("Resource already exists"); // TODO
        id
    }

    #[inline]
    pub fn add<T: Resource>(&mut self, resource: T) -> ResourceId {
        let id = self.insert_id::<T>();
        self.map.insert(id, Box::new(resource));
        id        
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let id = self.get_id::<T>()?;
        self.map.remove(&id)?.downcast().ok().map(|v| *v)
    }

    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let id = self.get_id::<T>()?;
        self.map.get(&id)?.downcast_ref()
    }

    #[inline]
    pub unsafe fn get_unchecked<T: Resource>(&self) -> Option<*const T> {
        self.get::<T>().map(|r| r as _)
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let id = self.get_id::<T>()?;
        self.map.get_mut(&id)?.downcast_mut()
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<T: Resource>(&self) -> Option<*mut T> {
        self.get_mut::<T>().map(|r| r as _)
    }
}
