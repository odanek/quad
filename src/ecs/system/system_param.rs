use crate::ecs::{World, archetype::Archetype, resource::{Resource, ResourceId}};
use std::{any::type_name, marker::PhantomData};

use super::{function_system::SystemMeta, resource_param::{Res, ResMut}};

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


unsafe impl<T: Resource> ReadOnlySystemParamFetch for ResState<T> {}

pub struct ResState<T> {
    resource_id: ResourceId,
    marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type Fetch = ResState<T>;
}

unsafe impl<T: Resource> SystemParamState for ResState<T> {
    type Config = ();

    fn init(world: &mut World, system_meta: &mut SystemMeta, _config: Self::Config) -> Self {
        let resource_id = world.resource_id::<T>().unwrap();
        let access = &mut system_meta.resource_access;
        if access.has_write(resource_id) {
            panic!(
                "Res<{}> in system {} conflicts with a previous ResMut<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
                type_name::<T>(), system_meta.name);
        }
        access.add_read(resource_id);

        Self {
            resource_id,
            marker: PhantomData,
        }
    }

    fn default_config() {}
}

impl<'a, T: Resource> SystemParamFetch<'a> for ResState<T> {
    type Item = Res<'a, T>;

    #[inline]
    unsafe fn get_param(
        _state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        let resource = world.resources().get_unchecked().unwrap_or_else(|| {
            panic!(
                "Resource requested by {} does not exist: {}",
                system_meta.name,
                std::any::type_name::<T>()
            )
        });
        Res { value: &*resource }
    }
}

pub struct ResMutState<T> {
    component_id: ResourceId,
    marker: PhantomData<T>,
}

// impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
//     type Fetch = ResMutState<T>;
// }

// unsafe impl<T: ResourceId> SystemParamState for ResMutState<T> {
//     type Config = ();

//     fn init(world: &mut World, system_meta: &mut SystemMeta, _config: Self::Config) -> Self {
//         let component_id = world.initialize_resource::<T>();
//         let combined_access = system_meta.component_access_set.combined_access_mut();
//         if combined_access.has_write(component_id) {
//             panic!(
//                 "ResMut<{}> in system {} conflicts with a previous ResMut<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
//                 std::any::type_name::<T>(), system_meta.name);
//         } else if combined_access.has_read(component_id) {
//             panic!(
//                 "ResMut<{}> in system {} conflicts with a previous Res<{0}> access. Allowing this would break Rust's mutability rules. Consider removing the duplicate access.",
//                 std::any::type_name::<T>(), system_meta.name);
//         }
//         combined_access.add_write(component_id);

//         let resource_archetype = world.archetypes.resource();
//         let archetype_component_id = resource_archetype
//             .get_archetype_component_id(component_id)
//             .unwrap();
//         system_meta
//             .archetype_component_access
//             .add_write(archetype_component_id);
//         Self {
//             component_id,
//             marker: PhantomData,
//         }
//     }

//     fn default_config() {}
// }

// impl<'a, T: Component> SystemParamFetch<'a> for ResMutState<T> {
//     type Item = ResMut<'a, T>;

//     #[inline]
//     unsafe fn get_param(
//         state: &'a mut Self,
//         system_meta: &SystemMeta,
//         world: &'a World,
//         change_tick: u32,
//     ) -> Self::Item {
//         let value = world
//             .get_resource_unchecked_mut_with_id(state.component_id)
//             .unwrap_or_else(|| {
//                 panic!(
//                     "Resource requested by {} does not exist: {}",
//                     system_meta.name,
//                     std::any::type_name::<T>()
//                 )
//             });
//         ResMut {
//             value: value.value,
//             ticks: Ticks {
//                 component_ticks: value.ticks.component_ticks,
//                 last_change_tick: system_meta.last_change_tick,
//                 change_tick,
//             },
//         }
//     }
// }

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
