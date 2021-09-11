use std::{
    any::{type_name, Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
};

use crate::ecs::{query::access::AccessIndex, system::SystemTicks};

use super::{ComponentTicks, ResMut, Tick};

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

struct ResourceWrapper {
    resource: UnsafeCell<Box<dyn Any>>,
    ticks: UnsafeCell<ComponentTicks>,
}

impl ResourceWrapper {
    fn new<T: Resource>(resource: T, change_tick: Tick) -> Self {
        Self {
            resource: UnsafeCell::new(Box::new(resource)),
            ticks: UnsafeCell::new(ComponentTicks::new(change_tick)),
        }
    }
}

#[derive(Default)]
pub struct Resources {
    resources: Vec<ResourceInfo>,
    id_map: HashMap<TypeId, ResourceId>,
    map: HashMap<ResourceId, ResourceWrapper>,
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
    pub fn add<T: Resource>(&mut self, resource: T, change_tick: Tick) -> Option<T> {
        let id = self.get_or_insert_id::<T>();
        let wrapper = ResourceWrapper::new(resource, change_tick);
        let old_wrapper = self.map.insert(id, wrapper)?;
        old_wrapper
            .resource
            .into_inner()
            .downcast()
            .ok()
            .map(|v| *v)
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.remove(&id)?;
        wrapper.resource.into_inner().downcast().ok().map(|v| *v)
    }

    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.get(&id)?;
        let resource = unsafe { &*wrapper.resource.get() };
        resource.downcast_ref()
    }

    #[inline]
    pub fn get_unchecked<T: Resource>(&self) -> Option<*const T> {
        self.get::<T>().map(|r| r as _)
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self, system_ticks: SystemTicks) -> Option<ResMut<T>> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.get_mut(&id)?;
        let resource = wrapper.resource.get_mut().downcast_mut()?;
        let ticks = wrapper.ticks.get_mut();
        Some(ResMut::new(resource, ticks, system_ticks))
    }

    #[inline]
    pub fn get_mut_unchecked<T: Resource>(&self) -> Option<(*mut T, *mut ComponentTicks)> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.get(&id)?;
        let resource = unsafe { &mut *wrapper.resource.get() }.downcast_mut::<T>()?;
        let ticks = wrapper.ticks.get();
        Some((resource as *mut T, ticks))
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
