use crate::ecs::{World, archetype::Archetype};

use super::function_system::SystemMeta;

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a>;
}

pub unsafe trait SystemParamState: Send + Sync + 'static {
    type Config: Send + Sync;

    fn init(world: &mut World, system_meta: &mut SystemMeta, config: Self::Config) -> Self;
    
    #[inline]
    fn new_archetype(&mut self, _archetype: &Archetype, _system_meta: &mut SystemMeta) {}
    
    #[inline]
    fn apply(&mut self, _world: &mut World) {}
    
    fn default_config() -> Self::Config;
}

pub unsafe trait ReadOnlySystemParamFetch {}

pub trait SystemParamFetch<'a>: SystemParamState {
    type Item;

    unsafe fn get_param(
        state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item;
}

macro_rules! impl_system_param_tuple {
    ($($param: ident),*) => {
        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type Fetch = ($($param::Fetch,)*);
        }

        // SAFE: tuple consists only of ReadOnlySystemParamFetches
        unsafe impl<$($param: ReadOnlySystemParamFetch),*> ReadOnlySystemParamFetch for ($($param,)*) {}

        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'a, $($param: SystemParamFetch<'a>),*> SystemParamFetch<'a> for ($($param,)*) {
            type Item = ($($param::Item,)*);

            #[inline]
            unsafe fn get_param(
                state: &'a mut Self,
                system_meta: &SystemMeta,
                world: &'a World
            ) -> Self::Item {

                let ($($param,)*) = state;
                ($($param::get_param($param, system_meta, world),)*)
            }
        }

        /// SAFE: implementors of each SystemParamState in the tuple have validated their impls
        #[allow(non_snake_case)]
        unsafe impl<$($param: SystemParamState),*> SystemParamState for ($($param,)*) {
            type Config = ($(<$param as SystemParamState>::Config,)*);
            #[inline]
            fn init(_world: &mut World, _system_meta: &mut SystemMeta, config: Self::Config) -> Self {
                let ($($param,)*) = config;
                (($($param::init(_world, _system_meta, $param),)*))
            }

            #[inline]
            fn apply(&mut self, _world: &mut World) {
                let ($($param,)*) = self;
                $($param.apply(_world);)*
            }

            fn default_config() -> ($(<$param as SystemParamState>::Config,)*) {
                ($(<$param as SystemParamState>::default_config(),)*)
            }
        }
    };
}

all_tuples!(impl_system_param_tuple);
