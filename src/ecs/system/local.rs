use crate::ecs::{component::Tick, system::function_system::SystemMeta, FromWorld, World};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::system_param::{
    ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState,
};

pub trait LocalResource: Send + Sync + 'static {}
impl<T> LocalResource for T where T: Send + Sync + 'static {}

pub struct Local<'a, T: LocalResource>(&'a mut T);

impl<'a, T: LocalResource> Debug for Local<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Local").field(&self.0).finish()
    }
}

impl<'a, T: LocalResource> Deref for Local<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: LocalResource> DerefMut for Local<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct LocalState<T: LocalResource>(T);

unsafe impl<T: LocalResource> ReadOnlySystemParamFetch for LocalState<T> {}

impl<'a, T: LocalResource + FromWorld> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

unsafe impl<T: LocalResource + FromWorld> SystemParamState for LocalState<T> {
    fn new(world: &mut World, _system_meta: &mut SystemMeta) -> Self {
        Self(T::from_world(world))
    }
}

impl<'w, 's, T: LocalResource + FromWorld> SystemParamFetch<'w, 's> for LocalState<T> {
    type Item = Local<'s, T>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        _system_meta: &SystemMeta,
        _world: &'w World,
        _change_tick: Tick,
    ) -> Self::Item {
        Local(&mut state.0)
    }
}
