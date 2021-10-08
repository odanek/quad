use std::marker::PhantomData;

use crate::ecs::{
    component::{ComponentId, Tick},
    Component, Entity, World,
};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
};

pub struct RemovedComponents<'w, T> {
    world: &'w World,
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<'w, T> RemovedComponents<'w, T> {
    pub fn iter(&self) -> std::iter::Cloned<std::slice::Iter<'_, Entity>> {
        self.world.removed_with_id(self.component_id)
    }
}

pub struct RemovedComponentsState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<'w, T: Component> SystemParam for RemovedComponents<'w, T> {
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

impl<'w, 's, T: Component> SystemParamFetch<'w, 's> for RemovedComponentsState<T> {
    type Item = RemovedComponents<'w, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        _system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
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
    use crate::ecs::{
        Component, Entity, IntoSystem, RemovedComponents, Res, ResMut, Resource, Scheduler, World,
    };

    struct Ran(pub bool);
    impl Resource for Ran {}

    struct Despawned(Entity);
    impl Resource for Despawned {}

    struct Text(String);
    impl Component for Text {}
    struct Number(i32);
    impl Component for Number {}

    #[test]
    fn remove_tracking() {
        let mut world = World::new();
        let a = world
            .spawn()
            .insert_bundle((Text("abc".to_owned()), Number(123)))
            .id();
        world.insert_resource(Ran(false));
        world.insert_resource(Despawned(a));

        world.entity_mut(a).despawn();

        fn validate_removed(
            removed_i32: RemovedComponents<Number>,
            despawned: Res<Despawned>,
            mut ran: ResMut<Ran>,
        ) {
            assert_eq!(
                removed_i32.iter().collect::<Vec<_>>(),
                &[despawned.0],
                "despawning results in 'removed component' state"
            );

            ran.0 = true;
        }

        Scheduler::single(validate_removed.system(&mut world)).run(&mut world);
        assert!(world.get_resource::<Ran>().unwrap().0, "system ran");
    }
}
