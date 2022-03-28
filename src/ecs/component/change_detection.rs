use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::ecs::system::SystemTicks;

use super::{Component, ComponentTicks, Resource};

pub trait DetectChanges {
    fn is_added(&self) -> bool;
    fn is_changed(&self) -> bool;
    fn set_changed(&mut self);
}

pub struct Res<'w, T: Resource> {
    pub(crate) value: &'w T,
    pub(crate) component_ticks: &'w ComponentTicks,
    pub(crate) system_ticks: SystemTicks,
}

impl<'w, T: Resource> Res<'w, T> {
    #[inline]
    pub(crate) fn new(
        value: &'w T,
        component_ticks: &'w ComponentTicks,
        system_ticks: SystemTicks,
    ) -> Self {
        Self {
            value,
            component_ticks,
            system_ticks,
        }
    }

    #[inline]
    pub fn is_added(&self) -> bool {
        self.component_ticks
            .is_added(self.system_ticks.last_change_tick)
    }

    #[inline]
    pub fn is_changed(&self) -> bool {
        self.component_ticks
            .is_changed(self.system_ticks.last_change_tick)
    }

    #[inline]
    pub fn into_inner(self) -> &'w T {
        self.value
    }
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
    pub(crate) component_ticks: &'w mut ComponentTicks,
    pub(crate) system_ticks: SystemTicks,
}

impl<'w, T: Resource> ResMut<'w, T> {
    #[inline]
    pub(crate) fn new(
        value: &'w mut T,
        component_ticks: &'w mut ComponentTicks,
        system_ticks: SystemTicks,
    ) -> Self {
        Self {
            value,
            component_ticks,
            system_ticks,
        }
    }

    #[inline]
    pub fn into_inner(self) -> &'w mut T {
        self.value
    }
}

impl<'w, T: Resource> DetectChanges for ResMut<'w, T> {
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
        self.set_changed();
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

pub struct CmptMut<'w, T: Component> {
    pub(crate) value: &'w mut T,
    pub(crate) component_ticks: &'w mut ComponentTicks,
    pub(crate) system_ticks: SystemTicks,
}

impl<'w, T: Component> CmptMut<'w, T> {
    #[inline]
    pub(crate) fn new(
        value: &'w mut T,
        component_ticks: &'w mut ComponentTicks,
        system_ticks: SystemTicks,
    ) -> Self {
        Self {
            value,
            component_ticks,
            system_ticks,
        }
    }
}

impl<'w, T: Component> DetectChanges for CmptMut<'w, T> {
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

impl<'w, T: Component> Deref for CmptMut<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Component> DerefMut for CmptMut<'w, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set_changed();
        self.value
    }
}

impl<'w, T: Component> AsRef<T> for CmptMut<'w, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'w, T: Component> AsMut<T> for CmptMut<'w, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}
