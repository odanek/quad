use crate::ecs::{
    resource::{Resource, ResourceId},
    system::function_system::SystemMeta,
    World,
};
use std::{
    any::type_name,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::system_param::{SystemParam, SystemParamFetch, SystemParamState};

pub struct Res<'w, T: Resource> {
    pub(crate) value: &'w T,
}

impl<'w, T: Resource> Debug for Res<'w, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Res").field(&self.value).finish()
    }
}

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> AsRef<T> for Res<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

pub struct ResState<T> {
    _resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type Fetch = ResState<T>;
}

impl<T: Resource> SystemParamState for ResState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<T>();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "Res<{}> in system {} conflicts with a previous ResMut<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
                type_name::<T>(), system_meta.name);
        }
        access.add_read(resource_id);

        Self {
            _resource_id: resource_id,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Resource> SystemParamFetch<'a> for ResState<T> {
    type Item = Res<'a, T>;

    #[inline]
    unsafe fn get_param(
        _state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        let resource = world.resources().get_unchecked().unwrap_or_else(|| {
            panic!(
                "Resource requested by {} does not exist: {}",
                system_meta.name,
                type_name::<T>()
            )
        });
        Res { value: &*resource }
    }
}

pub struct ResMut<'a, T: Resource> {
    pub(crate) value: &'a mut T,
}

impl<'w, T: Resource> Deref for ResMut<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> DerefMut for ResMut<'w, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'w, T: Resource> AsRef<T> for ResMut<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'w, T: Resource> AsMut<T> for ResMut<'w, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

pub struct ResMutState<T> {
    _resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    type Fetch = ResMutState<T>;
}

impl<T: Resource> SystemParamState for ResMutState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<T>();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "ResMut<{}> in system {} conflicts with a previous ResMut<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
                type_name::<T>(), system_meta.name);
        } else if access.has_read(resource_id) {
            panic!(
                "ResMut<{}> in system {} conflicts with a previous Res<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
                type_name::<T>(), system_meta.name);
        }
        access.add_write(resource_id);

        Self {
            _resource_id: resource_id,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Resource> SystemParamFetch<'a> for ResMutState<T> {
    type Item = ResMut<'a, T>;

    #[inline]
    unsafe fn get_param(
        _state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        let resource = world.resources().get_mut_unchecked().unwrap_or_else(|| {
            panic!(
                "Resource requested by {} does not exist: {}",
                system_meta.name,
                type_name::<T>()
            )
        });
        ResMut {
            value: &mut *resource,
        }
    }
}
