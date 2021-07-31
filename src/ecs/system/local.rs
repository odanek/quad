use crate::ecs::{component::Component, system::function_system::SystemMeta, FromWorld, World};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::system_param::{SystemParam, SystemParamFetch, SystemParamState};

pub struct Local<'a, T: Component>(&'a mut T);

impl<'a, T: Component> Debug for Local<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Local").field(&self.0).finish()
    }
}

impl<'a, T: Component> Deref for Local<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: Component> DerefMut for Local<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct LocalState<T: Component>(T);

impl<'a, T: Component + FromWorld> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

impl<T: Component + FromWorld> SystemParamState for LocalState<T> {
    fn new(world: &mut World, _system_meta: &mut SystemMeta) -> Self {
        Self(T::from_world(world))
    }
}

impl<'a, T: Component + FromWorld> SystemParamFetch<'a> for LocalState<T> {
    type Item = Local<'a, T>;

    #[inline]
    unsafe fn get_param(
        state: &'a mut Self,
        _system_meta: &SystemMeta,
        _world: &'a World,
    ) -> Self::Item {
        Local(&mut state.0)
    }
}
