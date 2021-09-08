use std::{
    any::type_name,
    cell::UnsafeCell,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::ecs::{
    component::{CmptMut, Component, ComponentId, ComponentTicks, Tick, Ticks},
    entity::archetype::Archetype,
    storage::Tables,
    Entity, World,
};

use super::access::FilteredAccess;

pub trait WorldQuery {
    type Fetch: for<'a> Fetch<'a, State = Self::State>;
    type State: FetchState;
}

pub trait Fetch<'w>: Sized {
    type Item;
    type State: FetchState;

    unsafe fn new(
        world: &World,
        state: &Self::State,
        last_change_tick: Tick,
        change_tick: Tick,
    ) -> Self;

    unsafe fn set_archetype(&mut self, state: &Self::State, archetype: &Archetype, tables: &Tables);

    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item;
}

pub unsafe trait FetchState: Send + Sync + Sized {
    fn new(world: &mut World) -> Self;
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>);
    fn matches_archetype(&self, archetype: &Archetype) -> bool;
}

pub unsafe trait ReadOnlyFetch {}

impl WorldQuery for Entity {
    type Fetch = EntityFetch;
    type State = EntityState;
}

pub struct EntityFetch {
    entities: *const Entity,
}

unsafe impl ReadOnlyFetch for EntityFetch {}

pub struct EntityState;

unsafe impl FetchState for EntityState {
    fn new(_world: &mut World) -> Self {
        Self
    }

    fn update_component_access(&self, _access: &mut FilteredAccess<ComponentId>) {}

    #[inline]
    fn matches_archetype(&self, _archetype: &Archetype) -> bool {
        true
    }
}

impl<'w> Fetch<'w> for EntityFetch {
    type Item = Entity;
    type State = EntityState;

    unsafe fn new(
        _world: &World,
        _state: &Self::State,
        _last_change_tick: Tick,
        _change_tick: Tick,
    ) -> Self {
        Self {
            entities: std::ptr::null::<Entity>(),
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        _state: &Self::State,
        archetype: &Archetype,
        _tables: &Tables,
    ) {
        self.entities = archetype.entities().as_ptr();
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        *self.entities.add(archetype_index)
    }
}

impl<T: Component> WorldQuery for &T {
    type Fetch = ReadFetch<T>;
    type State = ReadState<T>;
}

pub struct ReadState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

unsafe impl<T: Component> FetchState for ReadState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        ReadState {
            component_id,
            marker: PhantomData,
        }
    }

    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(self.component_id) {
            panic!("&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                type_name::<T>());
        }
        access.add_read(self.component_id)
    }

    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

pub struct ReadFetch<T> {
    table_components: NonNull<T>,
}

impl<T> Clone for ReadFetch<T> {
    fn clone(&self) -> Self {
        Self {
            table_components: self.table_components,
        }
    }
}

unsafe impl<T> ReadOnlyFetch for ReadFetch<T> {}

impl<'w, T: Component> Fetch<'w> for ReadFetch<T> {
    type Item = &'w T;
    type State = ReadState<T>;

    unsafe fn new(
        _world: &World,
        _state: &Self::State,
        _last_change_tick: Tick,
        _change_tick: Tick,
    ) -> Self {
        Self {
            table_components: NonNull::dangling(),
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
        tables: &Tables,
    ) {
        self.table_components = tables[archetype.table_id()]
            .get_column(state.component_id)
            .unwrap()
            .get_data_ptr()
            .cast::<T>();
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        &*self.table_components.as_ptr().add(archetype_index)
    }
}

impl<T: Component> WorldQuery for &mut T {
    type Fetch = WriteFetch<T>;
    type State = WriteState<T>;
}

pub struct WriteFetch<T> {
    table_components: NonNull<T>,
    table_ticks: *const UnsafeCell<ComponentTicks>,
    last_change_tick: Tick,
    change_tick: Tick,
}

impl<T> Clone for WriteFetch<T> {
    fn clone(&self) -> Self {
        Self {
            table_components: self.table_components,
            table_ticks: self.table_ticks,
            last_change_tick: self.last_change_tick,
            change_tick: self.change_tick,
        }
    }
}

pub struct WriteState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

unsafe impl<T: Component> FetchState for WriteState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        WriteState {
            component_id,
            marker: PhantomData,
        }
    }

    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_read(self.component_id) {
            panic!("&mut {} conflicts with a previous access in this query. Mutable component access must be unique.",
                type_name::<T>());
        }
        access.add_write(self.component_id);
    }

    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

impl<'w, T: Component> Fetch<'w> for WriteFetch<T> {
    type Item = CmptMut<'w, T>;
    type State = WriteState<T>;

    unsafe fn new(
        _world: &World,
        _state: &Self::State,
        last_change_tick: Tick,
        change_tick: Tick,
    ) -> Self {
        Self {
            table_components: NonNull::dangling(),
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            last_change_tick,
            change_tick,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
        tables: &Tables,
    ) {
        let column = tables[archetype.table_id()]
            .get_column(state.component_id)
            .unwrap();
        self.table_components = column.get_data_ptr().cast::<T>();
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        CmptMut {
            value: &mut *self.table_components.as_ptr().add(archetype_index),
            ticks: Ticks {
                component_ticks: &mut *(&*self.table_ticks.add(archetype_index)).get(),
                change_tick: self.change_tick,
                last_change_tick: self.last_change_tick,
            },
        }
    }
}

impl<T: WorldQuery> WorldQuery for Option<T> {
    type Fetch = OptionFetch<T::Fetch>;
    type State = OptionState<T::State>;
}

pub struct OptionFetch<T> {
    fetch: T,
    matches: bool,
}

unsafe impl<T: ReadOnlyFetch> ReadOnlyFetch for OptionFetch<T> {}

pub struct OptionState<T: FetchState> {
    state: T,
}

unsafe impl<T: FetchState> FetchState for OptionState<T> {
    fn new(world: &mut World) -> Self {
        Self {
            state: T::new(world),
        }
    }

    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        self.state.update_component_access(access);
    }

    #[inline]
    fn matches_archetype(&self, _archetype: &Archetype) -> bool {
        true
    }
}

impl<'w, T: Fetch<'w>> Fetch<'w> for OptionFetch<T> {
    type Item = Option<T::Item>;
    type State = OptionState<T::State>;

    unsafe fn new(
        world: &World,
        state: &Self::State,
        last_change_tick: Tick,
        change_tick: Tick,
    ) -> Self {
        Self {
            fetch: T::new(world, &state.state, last_change_tick, change_tick),
            matches: false,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
        tables: &Tables,
    ) {
        self.matches = state.state.matches_archetype(archetype);
        if self.matches {
            self.fetch.set_archetype(&state.state, archetype, tables);
        }
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        if self.matches {
            Some(self.fetch.archetype_fetch(archetype_index))
        } else {
            None
        }
    }
}

macro_rules! impl_tuple_fetch {
    ($(($name: ident, $state: ident)),*) => {
        #[allow(non_snake_case)]
        impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);
            type State = ($($name::State,)*);

            #[allow(clippy::unused_unit)]
            unsafe fn new(_world: &World, state: &Self::State, _last_change_tick: Tick, _change_tick: Tick) -> Self {
                let ($($name,)*) = state;
                ($($name::new(_world, $name, _last_change_tick, _change_tick),)*)
            }

            #[inline]
            unsafe fn set_archetype(&mut self, _state: &Self::State, _archetype: &Archetype, _tables: &Tables) {
                let ($($name,)*) = self;
                let ($($state,)*) = _state;
                $($name.set_archetype($state, _archetype, _tables);)*
            }

            #[inline]
            #[allow(clippy::unused_unit)]
            unsafe fn archetype_fetch(&mut self, _archetype_index: usize) -> Self::Item {
                let ($($name,)*) = self;
                ($($name.archetype_fetch(_archetype_index),)*)
            }
        }

        #[allow(non_snake_case)]
        unsafe impl<$($name: FetchState),*> FetchState for ($($name,)*) {
            #[allow(clippy::unused_unit)]
            fn new(_world: &mut World) -> Self {
                ($($name::new(_world),)*)
            }

            fn update_component_access(&self, _access: &mut FilteredAccess<ComponentId>) {
                let ($($name,)*) = self;
                $($name.update_component_access(_access);)*
            }

            fn matches_archetype(&self, _archetype: &Archetype) -> bool {
                let ($($name,)*) = self;
                true $(&& $name.matches_archetype(_archetype))*
            }
        }

        impl<$($name: WorldQuery),*> WorldQuery for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
            type State = ($($name::State,)*);
        }

        unsafe impl<$($name: ReadOnlyFetch),*> ReadOnlyFetch for ($($name,)*) {}
    };
}

all_pair_tuples!(impl_tuple_fetch);
