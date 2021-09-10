use crate::ecs::{
    component::{Resource, ResourceId, Tick},
    system::function_system::SystemMeta,
    Res, ResMut, World,
};
use std::{any::type_name, marker::PhantomData};

use super::system_param::{SystemParam, SystemParamFetch, SystemParamState};

pub struct ResState<T> {
    _resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'w, T: Resource> SystemParam for Res<'w, T> {
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

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResState<T> {
    type Item = Res<'w, T>;

    #[inline]
    unsafe fn get_param(
        _state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
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

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutState<T> {
    type Item = ResMut<'w, T>;

    #[inline]
    unsafe fn get_param(
        _state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
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
