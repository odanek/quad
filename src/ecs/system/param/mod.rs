use crate::ecs::World;

use super::function_system::SystemMeta;

mod local;
mod query;
mod resource;

pub use local::Local;
pub use query::Query;
pub use resource::{Res, ResMut};

pub trait SystemParamState: Send + Sync + 'static {
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self;

    #[inline]
    fn apply(&mut self, _world: &mut World) {}
}

pub trait SystemParamFetch<'a>: SystemParamState {
    type Item;

    unsafe fn get_param(
        state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item;
}

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a>;
}

macro_rules! impl_system_param_tuple {
    ($($param: ident),*) => {
        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type Fetch = ($($param::Fetch,)*);
        }

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
        impl<$($param: SystemParamState),*> SystemParamState for ($($param,)*) {
            #[inline]
            fn new(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
                (($($param::new(_world, _system_meta),)*))
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
