use crate::ecs::{
    archetype::Archetype,
    resource::Resource,
    World,
};
use std::{any::type_name, fmt::Debug, marker::PhantomData, ops::Deref};

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

pub struct Res<'w, T: Resource> {
    value: &'w T,
}

unsafe impl<T: Resource> ReadOnlySystemParamFetch for ResState<T> {}

impl<'w, T: Resource> Debug for Res<'w, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Res").field(&self.value).finish()
    }
}

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> AsRef<T> for Res<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

pub struct ResState<T> {
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
