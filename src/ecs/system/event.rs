use std::{any::type_name, marker::PhantomData};

use crate::ecs::{
    component::{ResourceId, Tick},
    Event, EventReader, EventWriter, Events, ReadOnlySystemParamFetch, World,
};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
};

pub struct EventReaderState<T: Event> {
    resource_id: ResourceId,
    last_event_count: usize,
    marker: PhantomData<T>,
}

impl<'w, 's, T: Event> SystemParam for EventReader<'w, 's, T> {
    type Fetch = EventReaderState<T>;
}

unsafe impl<T: Event> SystemParamState for EventReaderState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<Events<T>>();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "EventReader<{}> in system {} conflicts with a previous ResMut<{0}> access.",
                type_name::<T>(),
                system_meta.name
            );
        }
        access.add_read(resource_id);

        Self {
            resource_id,
            last_event_count: 0,
            marker: PhantomData,
        }
    }
}

impl<'w, 's, T: Event> SystemParamFetch<'w, 's> for EventReaderState<T> {
    type Item = EventReader<'w, 's, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
    ) -> Self::Item {
        let (resource, _ticks) = world
            .resources()
            .get_unchecked(state.resource_id)
            .unwrap_or_else(|| {
                panic!(
                    "Resource requested by {} does not exist: {}",
                    system_meta.name,
                    type_name::<Events<T>>()
                )
            });

        EventReader::new(&*resource, &mut state.last_event_count)
    }
}

unsafe impl<T: Event> ReadOnlySystemParamFetch for EventReaderState<T> {}

pub struct EventWriterState<T: Event> {
    resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'w, T: Event> SystemParam for EventWriter<'w, T> {
    type Fetch = EventWriterState<T>;
}

unsafe impl<T: Event> SystemParamState for EventWriterState<T> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let resource_id = world.register_resource::<Events<T>>();
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

impl<'w, 's, T: Event> SystemParamFetch<'w, 's> for EventWriterState<T> {
    type Item = EventWriter<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
    ) -> Self::Item {
        let (resource, _ticks) = world
            .resources()
            .get_mut_unchecked(state.resource_id)
            .unwrap_or_else(|| {
                panic!(
                    "Resource requested by {} does not exist: {}",
                    system_meta.name,
                    type_name::<Events<T>>()
                )
            });

        EventWriter::new(&mut *resource)
    }
}
