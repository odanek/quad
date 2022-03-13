use crate::{ecs::{component::Tick, World}, macros::all_tuples};

use super::function_system::SystemMeta;

pub trait SystemParamState: Send + Sync + 'static {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self;

    #[inline]
    fn update(&mut self, _world: &World, _system_meta: &mut SystemMeta) {}

    #[inline]
    fn apply(&mut self, _world: &mut World) {}
}

pub trait SystemParamFetch<'w, 's>: SystemParamState {
    type Item;

    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item;
}

pub unsafe trait ReadOnlySystemParamFetch {}

pub trait SystemParam: Sized {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub type SystemParamItem<'w, 's, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'w, 's>>::Item;

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
        impl<$($param: SystemParamState),*> SystemParamState for ($($param,)*) {
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
