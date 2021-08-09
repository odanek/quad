use std::marker::PhantomData;

use crate::ecs::{component::ComponentId, Component, Entity, World};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
};

pub struct RemovedComponents<'a, T> {
    world: &'a World,
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<'a, T> RemovedComponents<'a, T> {
    pub fn iter(&self) -> std::iter::Cloned<std::slice::Iter<'_, Entity>> {
        self.world.removed_with_id(self.component_id)
    }
}

pub struct RemovedComponentsState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<'a, T: Component> SystemParam for RemovedComponents<'a, T> {
    type Fetch = RemovedComponentsState<T>;
}

impl<T: Component> SystemParamState for RemovedComponentsState<T> {
    fn new(world: &mut World, _system_meta: &mut SystemMeta) -> Self {
        Self {
            component_id: world.register_component::<T>(),
            marker: PhantomData,
        }
    }
}

impl<'a, T: Component> SystemParamFetch<'a> for RemovedComponentsState<T> {
    type Item = RemovedComponents<'a, T>;

    #[inline]
    unsafe fn get_param(
        state: &'a mut Self,
        _system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        RemovedComponents {
            world,
            component_id: state.component_id,
            marker: PhantomData,
        }
    }
}
