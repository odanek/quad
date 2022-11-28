use super::MainWorld;
use crate::ecs::{
    ReadOnlySystemParamFetch, ResState, SystemMeta, SystemParam, SystemParamFetch, SystemParamItem,
    SystemParamState, SystemState, Tick, World,
};
use std::ops::{Deref, DerefMut};

pub struct Extract<'w, 's, P: SystemParam + 'static>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    item: <P::Fetch as SystemParamFetch<'w, 's>>::Item,
}

impl<'w, 's, P: SystemParam> SystemParam for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Fetch = ExtractState<P>;
}

#[doc(hidden)]
pub struct ExtractState<P: SystemParam + 'static> {
    state: SystemState<P>,
    main_world_state: ResState<MainWorld>,
}

// SAFETY: only accesses MainWorld resource with read only system params using ResState,
// which is initialized in init()
unsafe impl<P: SystemParam + 'static> SystemParamState for ExtractState<P> {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let mut main_world = world.resource_mut::<MainWorld>();
        Self {
            state: SystemState::new(&mut main_world),
            main_world_state: ResState::new(world, system_meta),
        }
    }
}

impl<'w, 's, P: SystemParam + 'static> SystemParamFetch<'w, 's> for ExtractState<P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Item = Extract<'w, 's, P>;

    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        let main_world = ResState::<MainWorld>::get_param(
            &mut state.main_world_state,
            system_meta,
            world,
            change_tick,
        );
        let item = state.state.get(main_world.into_inner());
        Extract { item }
    }
}

impl<'w, 's, P: SystemParam> Deref for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Target = <P::Fetch as SystemParamFetch<'w, 's>>::Item;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'w, 's, P: SystemParam> DerefMut for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<'a, 'w, 's, P: SystemParam> IntoIterator for &'a Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
    &'a SystemParamItem<'w, 's, P>: IntoIterator,
{
    type Item = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::Item;
    type IntoIter = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.item).into_iter()
    }
}
