use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::ecs::system::SystemTicks;

use super::{ComponentTicks, Resource};

pub trait DetectChanges {
    fn is_added(&self) -> bool;
    fn is_changed(&self) -> bool;
    fn set_changed(&mut self);
}

pub struct Res<'w, T: Resource> {
    pub(crate) value: &'w T,
}

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

pub struct ResMut<'w, T: Resource> {
    pub(crate) value: &'w mut T,
}

impl<'w, T: Resource> Deref for ResMut<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> DerefMut for ResMut<'w, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'w, T: Resource> AsRef<T> for ResMut<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'w, T: Resource> AsMut<T> for ResMut<'w, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

pub struct CmptMut<'a, T> {
    pub(crate) value: &'a mut T,
    pub(crate) component_ticks: &'a mut ComponentTicks,
    pub(crate) system_ticks: SystemTicks,
}

impl<'a, T> CmptMut<'a, T> {
    #[inline]
    pub(crate) fn new(
        value: &'a mut T,
        component_ticks: &'a mut ComponentTicks,
        system_ticks: SystemTicks,
    ) -> Self {
        Self {
            value,
            component_ticks,
            system_ticks,
        }
    }
}

impl<'a, T> DetectChanges for CmptMut<'a, T> {
    #[inline]
    fn is_added(&self) -> bool {
        self.component_ticks
            .is_added(self.system_ticks.last_change_tick)
    }

    #[inline]
    fn is_changed(&self) -> bool {
        self.component_ticks
            .is_changed(self.system_ticks.last_change_tick)
    }

    #[inline]
    fn set_changed(&mut self) {
        self.component_ticks
            .set_changed(self.system_ticks.change_tick);
    }
}

impl<'a, T> Deref for CmptMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for CmptMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set_changed();
        self.value
    }
}

impl<'a, T> AsRef<T> for CmptMut<'a, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> AsMut<T> for CmptMut<'a, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}
