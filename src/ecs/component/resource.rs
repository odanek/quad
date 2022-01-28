use std::{
    any::{type_name, Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
};

use crate::ecs::{query::access::AccessIndex, system::SystemTicks};

use super::{ComponentTicks, Res, ResMut};

pub trait Resource: Send + Sync + 'static {}

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
    fn new<T: Resource>(resource: T, ticks: ComponentTicks) -> Self {
        Self {
            resource: UnsafeCell::new(Box::new(resource)),
            ticks: UnsafeCell::new(ticks),
        }
    }
}

#[derive(Default)]
pub struct Resources {
    resources: Vec<ResourceInfo>,
    id_map: HashMap<TypeId, ResourceId>,
    map: HashMap<ResourceId, ResourceWrapper>,
}

// TODO: Is this safe?
unsafe impl Sync for Resources {}
unsafe impl Send for Resources {}

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
    pub fn add<T: Resource>(&mut self, resource: T, ticks: ComponentTicks) -> Option<T> {
        let id = self.get_or_insert_id::<T>();
        let wrapper = ResourceWrapper::new(resource, ticks);
        let old_wrapper = self.map.insert(id, wrapper)?;
        old_wrapper
            .resource
            .into_inner()
            .downcast()
            .ok()
            .map(|v| *v)
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<(T, ComponentTicks)> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.remove(&id)?;
        let resource = wrapper.resource.into_inner().downcast().ok()?;
        let ticks = wrapper.ticks.into_inner();
        Some((*resource, ticks))
    }

    #[inline]
    pub fn get<T: Resource>(&self, system_ticks: SystemTicks) -> Option<Res<T>> {
        let id = self.get_id::<T>()?;
        let wrapper = self.map.get(&id)?;
        let resource = unsafe { &*wrapper.resource.get() }.downcast_ref::<T>()?;
        let ticks = unsafe { &*wrapper.ticks.get() };
        Some(Res::new(resource, ticks, system_ticks))
    }

    #[inline]
    pub unsafe fn get_unchecked<T: Resource>(
        &self,
        id: ResourceId,
    ) -> Option<(*const T, *const ComponentTicks)> {
        let wrapper = self.map.get(&id)?;
        let resource = (&*wrapper.resource.get()).downcast_ref::<T>()?;
        let ticks = wrapper.ticks.get();
        Some((resource as *const T, ticks))
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
    pub unsafe fn get_mut_unchecked<T: Resource>(
        &self,
        id: ResourceId,
    ) -> Option<(*mut T, *mut ComponentTicks)> {
        let wrapper = self.map.get(&id)?;
        let resource = (&mut *wrapper.resource.get()).downcast_mut::<T>()?;
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
