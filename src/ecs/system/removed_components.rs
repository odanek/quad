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

#[cfg(test)]
mod test {
    use crate::ecs::{Entity, IntoSystem, RemovedComponents, Res, ResMut, Scheduler, World};

    #[test]
    fn remove_tracking() {
        let mut world = World::new();
        struct Despawned(Entity);
        let a = world.spawn().insert_bundle(("abc", 123)).id();
        world.spawn().insert_bundle(("abc", 123));
        world.insert_resource(false);
        world.insert_resource(Despawned(a));

        world.entity_mut(a).despawn();

        fn validate_removed(
            removed_i32: RemovedComponents<i32>,
            despawned: Res<Despawned>,
            mut ran: ResMut<bool>,
        ) {
            assert_eq!(
                removed_i32.iter().collect::<Vec<_>>(),
                &[despawned.0],
                "despawning results in 'removed component' state"
            );

            *ran = true;
        }

        Scheduler::single(validate_removed.system(&mut world)).run(&mut world);
        assert!(*world.get_resource::<bool>().unwrap(), "system ran");
    }
}
