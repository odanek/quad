use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use quad_macros::impl_param_set;

use crate::{
    ecs::{component::Tick, World},
    macros::all_tuples,
};

use super::function_system::SystemMeta;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SystemParamState: Send + Sync + 'static {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self;

    #[inline]
    fn update(&mut self, _world: &World, _system_meta: &mut SystemMeta) {}

    #[inline]
    fn apply(&mut self, _world: &mut World) {}
}

pub trait SystemParamFetch<'w, 's>: SystemParamState {
    type Item: SystemParam<Fetch = Self>;

    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item;
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait ReadOnlySystemParamFetch {}

pub trait SystemParam: Sized {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub type SystemParamItem<'w, 's, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'w, 's>>::Item;

pub struct SystemState<Param: SystemParam> {
    meta: SystemMeta,
    param_state: <Param as SystemParam>::Fetch,
}

impl<Param: SystemParam> SystemState<Param> {
    pub fn new(world: &mut World) -> Self {
        let mut meta = SystemMeta::new(std::any::type_name::<Param>().to_owned());
        let param_state = <Param::Fetch as SystemParamState>::new(world, &mut meta);
        Self { meta, param_state }
    }

    #[inline]
    pub fn meta(&self) -> &SystemMeta {
        &self.meta
    }

    pub fn apply(&mut self, world: &mut World) {
        self.param_state.apply(world);
    }

    fn update(&mut self, world: &World) {
        self.param_state.update(world, &mut self.meta);
    }

    /// Retrieve the [`SystemParam`] values. This can only be called when all parameters are read-only.
    #[inline]
    pub fn get<'w, 's>(
        &'s mut self,
        world: &'w World,
    ) -> <Param::Fetch as SystemParamFetch<'w, 's>>::Item
    where
        Param::Fetch: ReadOnlySystemParamFetch,
    {
        self.update(world); // TODO Is this necessary?
        unsafe { self.get_unchecked_manual(world, world.change_tick()) } // TODO Should the change tick be incremented here?
    }

    unsafe fn get_unchecked_manual<'w, 's>(
        &'s mut self,
        world: &'w World,
        change_tick: Tick,
    ) -> <Param::Fetch as SystemParamFetch<'w, 's>>::Item {
        <Param::Fetch as SystemParamFetch>::get_param(
            &mut self.param_state,
            &self.meta,
            world,
            change_tick,
        )
    }
}

pub struct StaticSystemParam<'w, 's, P: SystemParam>(SystemParamItem<'w, 's, P>);

impl<'w, 's, P: SystemParam> Deref for StaticSystemParam<'w, 's, P> {
    type Target = SystemParamItem<'w, 's, P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'w, 's, P: SystemParam> DerefMut for StaticSystemParam<'w, 's, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'w, 's, P: SystemParam> StaticSystemParam<'w, 's, P> {
    pub fn into_inner(self) -> SystemParamItem<'w, 's, P> {
        self.0
    }
}

pub struct StaticSystemParamState<S, P>(S, PhantomData<fn() -> P>);

unsafe impl<S: ReadOnlySystemParamFetch, P> ReadOnlySystemParamFetch
    for StaticSystemParamState<S, P>
{
}

impl<'world, 'state, P: SystemParam + 'static> SystemParam
    for StaticSystemParam<'world, 'state, P>
{
    type Fetch = StaticSystemParamState<P::Fetch, P>;
}

impl<'world, 'state, S: SystemParamFetch<'world, 'state>, P: SystemParam + 'static>
    SystemParamFetch<'world, 'state> for StaticSystemParamState<S, P>
where
    P: SystemParam<Fetch = S>,
{
    type Item = StaticSystemParam<'world, 'state, P>;

    unsafe fn get_param(
        state: &'state mut Self,
        system_meta: &SystemMeta,
        world: &'world World,
        change_tick: Tick,
    ) -> Self::Item {
        // Safe: We properly delegate SystemParamState
        StaticSystemParam(S::get_param(&mut state.0, system_meta, world, change_tick))
    }
}

unsafe impl<S: SystemParamState, P: SystemParam + 'static> SystemParamState
    for StaticSystemParamState<S, P>
{
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        Self(S::new(world, system_meta), PhantomData)
    }

    fn update(&mut self, world: &World, system_meta: &mut SystemMeta) {
        self.0.update(world, system_meta)
    }

    fn apply(&mut self, world: &mut World) {
        self.0.apply(world)
    }
}

pub struct ParamSet<'w, 's, T: SystemParam> {
    param_states: &'s mut T::Fetch,
    world: &'w World,
    system_meta: SystemMeta,
    change_tick: Tick,
}
pub struct ParamSetState<T: for<'w, 's> SystemParamFetch<'w, 's>>(T);

impl_param_set!();

macro_rules! impl_system_param_tuple {
    ($($param: ident),*) => {
        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type Fetch = ($($param::Fetch,)*);
        }

        unsafe impl<$($param: ReadOnlySystemParamFetch),*> ReadOnlySystemParamFetch for ($($param,)*) {}

        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'w, 's, $($param: SystemParamFetch<'w, 's>),*> SystemParamFetch<'w, 's> for ($($param,)*) {
            type Item = ($($param::Item,)*);

            #[inline]
            #[allow(clippy::unused_unit)]
            unsafe fn get_param(
                state: &'s mut Self,
                system_meta: &SystemMeta,
                world: &'w World,
                change_tick: Tick,
            ) -> Self::Item {
                let ($($param,)*) = state;
                ($($param::get_param($param, system_meta, world, change_tick),)*)
            }
        }

        #[allow(non_snake_case)]
        unsafe impl<$($param: SystemParamState),*> SystemParamState for ($($param,)*) {
            #[inline]
            fn new(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
                (($($param::new(_world, _system_meta),)*))
            }

            #[inline]
            fn update(&mut self, _world: &World, _system_meta: &mut SystemMeta) {
                let ($($param,)*) = self;
                $($param.update(_world, _system_meta);)*
            }

            #[inline]
            fn apply(&mut self, _world: &mut World) {
                let ($($param,)*) = self;
                $($param.apply(_world);)*
            }
        }
    };
}

all_tuples!(impl_system_param_tuple);
