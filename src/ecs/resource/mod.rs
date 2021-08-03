use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use super::query::access::AccessIndex;

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ResourceId(usize);

impl AccessIndex for ResourceId {
    fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct ResourceInfo {
    id: ResourceId,
    type_id: TypeId,
    name: &'static str,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
impl Resources {
    #[inline]
    pub fn get_id<T: Resource>(&self) -> Option<ResourceId> {
        let type_id = TypeId::of::<T>();
        self.id_map.get(&type_id).copied()
    }

    #[inline]
    pub fn get_info(&self, id: ResourceId) -> Option<&ResourceInfo> {
        self.resources.get(id.index())
    }

    #[inline]
    pub fn add<T: Resource>(&mut self, resource: T) -> Option<T> {
        let id = self.get_or_insert_id::<T>();
        self.map
            .insert(id, Box::new(resource))?
            .downcast()
            .ok()
            .map(|v| *v)
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
    pub fn get_unchecked<T: Resource>(&self) -> Option<*const T> {
        self.get::<T>().map(|r| r as _)
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let id = self.get_id::<T>()?;
        self.map.get_mut(&id)?.downcast_mut()
    }

    #[inline]
    pub fn get_mut_unchecked<T: Resource>(&self) -> Option<*mut T> {
        self.get::<T>().map(|r| r as *const T as _)
    }

    pub fn get_or_insert_id<T: Resource>(&mut self) -> ResourceId {
        let type_id = TypeId::of::<T>();
        let resources = &mut self.resources;
        *self.id_map.entry(type_id).or_insert_with(|| {
            let id = ResourceId(resources.len());
            let info = ResourceInfo {
                id,
                type_id,
                name: type_name::<T>(),
            };
            resources.push(info);
            id
        })
    }
}
