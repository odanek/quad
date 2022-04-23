use crate::ecs::{
    component::{Resource, ResourceId, Tick},
    system::function_system::SystemMeta,
    Res, ResMut, World,
};
use std::{any::type_name, marker::PhantomData};

use super::{
    system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState},
    SystemTicks,
};

pub struct ResState<T> {
    resource_id: ResourceId,
    marker: PhantomData<T>,
}

unsafe impl<T: Resource> ReadOnlySystemParamFetch for ResState<T> {}

impl<'w, T: Resource> SystemParam for Res<'w, T> {
    type Fetch = ResState<T>;
}

unsafe impl<T: Resource> SystemParamState for ResState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<T>();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "Res<{}> in system {} conflicts with a previous ResMut<{0}> access.",
                type_name::<T>(),
                system_meta.name
            );
        }
        access.add_read(resource_id);

        Self {
            resource_id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResState<T> {
    type Item = Res<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        let (resource, ticks) = world
            .resources()
            .get_unchecked(state.resource_id)
            .unwrap_or_else(|| {
                panic!(
                    "Resource requested by {} does not exist: {}",
                    system_meta.name,
                    type_name::<T>()
                )
            });
        let system_ticks = SystemTicks::new(system_meta.last_change_tick, change_tick);
        Res::new(&*resource, &*ticks, system_ticks)
    }
}

pub struct OptionResState<T>(ResState<T>);

unsafe impl<T: Resource> ReadOnlySystemParamFetch for OptionResState<T> {}

impl<'a, T: Resource> SystemParam for Option<Res<'a, T>> {
    type Fetch = OptionResState<T>;
}

unsafe impl<T: Resource> SystemParamState for OptionResState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        Self(ResState::new(world, system_meta))
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResState<T> {
    type Item = Option<Res<'w, T>>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        let (resource, ticks) = world.resources().get_unchecked(state.0.resource_id)?;
        let system_ticks = SystemTicks::new(system_meta.last_change_tick, change_tick);
        Some(Res::new(&*resource, &*ticks, system_ticks))
    }
}

pub struct ResMutState<T> {
    resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    type Fetch = ResMutState<T>;
}

unsafe impl<T: Resource> SystemParamState for ResMutState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<T>();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "ResMut<{}> in system {} conflicts with a previous ResMut<{0}> access.",
                type_name::<T>(),
                system_meta.name
            );
        } else if access.has_read(resource_id) {
            panic!(
                "ResMut<{}> in system {} conflicts with a previous Res<{0}> access.",
                type_name::<T>(),
                system_meta.name
            );
        }
        access.add_write(resource_id);

        Self {
            resource_id,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutState<T> {
    type Item = ResMut<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        let (resource, ticks) = world
            .resources()
            .get_mut_unchecked(state.resource_id)
            .unwrap_or_else(|| {
                panic!(
                    "Resource requested by {} does not exist: {}",
                    system_meta.name,
                    type_name::<T>()
                )
            });
        let system_ticks = SystemTicks::new(system_meta.last_change_tick, change_tick);
        ResMut::new(&mut *resource, &mut *ticks, system_ticks)
    }
}

pub struct OptionResMutState<T>(ResMutState<T>);

impl<'a, T: Resource> SystemParam for Option<ResMut<'a, T>> {
    type Fetch = OptionResMutState<T>;
}

unsafe impl<T: Resource> SystemParamState for OptionResMutState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        Self(ResMutState::new(world, system_meta))
    }
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResMutState<T> {
    type Item = Option<ResMut<'w, T>>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        let (resource, ticks) = world.resources().get_mut_unchecked(state.0.resource_id)?;
        let system_ticks = SystemTicks::new(system_meta.last_change_tick, change_tick);
        Some(ResMut::new(&mut *resource, &mut *ticks, system_ticks))
    }
}
