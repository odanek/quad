use std::{
    any::type_name,
    cell::UnsafeCell,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::{
    ecs::{
        World,
        component::{CmptMut, Component, ComponentId, ComponentTicks},
        entity::{Archetype, Entity},
        storage::Table,
        system::SystemTicks,
    },
    macros::all_pair_tuples,
};

use super::access::FilteredAccess;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait WorldQuery {
    type State: Send + Sync + Sized;
    type Fetch<'a>;
    type Item<'a>;
    type ReadOnly: ReadOnlyWorldQuery<State = Self::State>;

    fn new_state(world: &mut World) -> Self::State;

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>);

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn new_fetch<'w>(
        world: &'w World,
        state: &Self::State,
        system_ticks: SystemTicks,
    ) -> Self::Fetch<'w>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    );

    #[allow(clippy::missing_safety_doc)]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn filter_fetch(
        _fetch: &mut Self::Fetch<'_>,
        _entity: Entity,
        _table_row: usize,
    ) -> bool {
        true
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait ReadOnlyWorldQuery: WorldQuery<ReadOnly = Self> {}
pub type QueryItem<'w, Q> = <Q as WorldQuery>::Item<'w>;
pub type ROQueryItem<'w, Q> = QueryItem<'w, <Q as WorldQuery>::ReadOnly>;

unsafe impl WorldQuery for Entity {
    type State = ();
    type Fetch<'w> = ();
    type Item<'w> = Entity;
    type ReadOnly = Self;

    #[inline]
    fn new_state(_world: &mut World) -> Self::State {}

    #[inline]
    fn update_component_access(_state: &Self::State, _access: &mut FilteredAccess<ComponentId>) {}

    #[inline]
    fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
        true
    }

    #[inline]
    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        _system_ticks: SystemTicks,
    ) -> Self::Fetch<'w> {
    }

    #[inline]
    unsafe fn set_archetype(
        _fetch: &mut Self::Fetch<'_>,
        _state: &Self::State,
        _archetype: &Archetype,
        _table: &Table,
    ) {
    }

    #[inline]
    unsafe fn fetch<'w>(
        _fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        _table_row: usize,
    ) -> Self::Item<'w> {
        entity
    }
}

unsafe impl ReadOnlyWorldQuery for Entity {}

pub struct ReadFetch<'w, T> {
    table_components: NonNull<T>, // TODO
    _marker: PhantomData<&'w [T]>,
}

unsafe impl<T: Component> WorldQuery for &T {
    type State = ComponentId;
    type Fetch<'w> = ReadFetch<'w, T>;
    type Item<'w> = &'w T;
    type ReadOnly = Self;

    fn new_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(*state) {
            panic!(
                "&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                type_name::<T>()
            );
        }
        access.add_read(*state)
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        _system_ticks: SystemTicks,
    ) -> Self::Fetch<'w> {
        ReadFetch {
            table_components: NonNull::dangling(),
            _marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        fetch: &mut Self::Fetch<'_>,
        state: &Self::State,
        _archetype: &Archetype,
        table: &Table,
    ) {
        fetch.table_components = table.get_column(*state).unwrap().get_data_ptr().cast::<T>();
    }

    #[inline]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        &*fetch.table_components.as_ptr().add(table_row)
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for &T {}

pub struct WriteFetch<'w, T> {
    table_components: NonNull<T>,                   // TODO
    table_ticks: *const UnsafeCell<ComponentTicks>, // TODO
    system_ticks: SystemTicks,
    _marker: PhantomData<&'w [T]>,
}

unsafe impl<'__w, T: Component> WorldQuery for &'__w mut T {
    type Fetch<'w> = WriteFetch<'w, T>;
    type Item<'w> = CmptMut<'w, T>;
    type ReadOnly = &'__w T;
    type State = ComponentId;

    fn new_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_read(*state) {
            panic!(
                "&mut {} conflicts with a previous access in this query. Mutable component access must be unique.",
                type_name::<T>()
            );
        }
        access.add_write(*state);
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        system_ticks: SystemTicks,
    ) -> WriteFetch<'w, T> {
        WriteFetch {
            table_components: NonNull::dangling(),
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            system_ticks,
            _marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        fetch: &mut Self::Fetch<'_>,
        state: &Self::State,
        _archetype: &Archetype,
        table: &Table,
    ) {
        let column = table.get_column(*state).unwrap();
        fetch.table_components = column.get_data_ptr().cast::<T>();
        fetch.table_ticks = column.get_ticks_ptr();
    }

    #[inline]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        let value = &mut *fetch.table_components.as_ptr().add(table_row);
        let component_ticks = &mut *(*fetch.table_ticks.add(table_row)).get();
        CmptMut::new(value, component_ticks, fetch.system_ticks)
    }
}

pub struct OptionFetch<'w, T: WorldQuery> {
    fetch: T::Fetch<'w>,
    matches: bool,
}

unsafe impl<T: WorldQuery> WorldQuery for Option<T> {
    type Fetch<'w> = OptionFetch<'w, T>;
    type Item<'w> = Option<T::Item<'w>>;
    type ReadOnly = Option<T::ReadOnly>;
    type State = T::State;

    fn new_state(world: &mut World) -> T::State {
        T::new_state(world)
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        let mut intermediate = access.clone();
        T::update_component_access(state, &mut intermediate);
        access.extend_access(&intermediate);
    }

    #[inline]
    fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
        true
    }

    unsafe fn new_fetch<'w>(
        world: &'w World,
        state: &Self::State,
        system_ticks: SystemTicks,
    ) -> OptionFetch<'w, T> {
        OptionFetch {
            fetch: T::new_fetch(world, state, system_ticks),
            matches: false,
        }
    }

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut OptionFetch<'w, T>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        fetch.matches = T::matches_archetype(state, archetype);
        if fetch.matches {
            T::set_archetype(&mut fetch.fetch, state, archetype, table);
        }
    }

    #[inline]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        if fetch.matches {
            Some(T::fetch(&mut fetch.fetch, entity, table_row))
        } else {
            None
        }
    }
}

unsafe impl<T: ReadOnlyWorldQuery> ReadOnlyWorldQuery for Option<T> {}

pub struct ChangeTrackers<T: Component> {
    component_ticks: ComponentTicks,
    system_ticks: SystemTicks,
    marker: PhantomData<T>,
}

impl<T: Component> ChangeTrackers<T> {
    pub fn is_added(&self) -> bool {
        self.component_ticks
            .is_added(self.system_ticks.last_change_tick)
    }

    pub fn is_changed(&self) -> bool {
        self.component_ticks
            .is_changed(self.system_ticks.last_change_tick)
    }
}

pub struct ChangeTrackersFetch<'w, T> {
    table_ticks: *const ComponentTicks,
    system_ticks: SystemTicks,
    marker: PhantomData<&'w [T]>,
}

unsafe impl<T: Component> WorldQuery for ChangeTrackers<T> {
    type Fetch<'w> = ChangeTrackersFetch<'w, T>;
    type Item<'w> = ChangeTrackers<T>;
    type ReadOnly = Self;
    type State = ComponentId;

    fn new_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(*state) {
            panic!(
                "ChangeTrackers<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>()
            );
        }
        access.add_read(*state)
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        system_ticks: SystemTicks,
    ) -> ChangeTrackersFetch<'w, T> {
        ChangeTrackersFetch {
            table_ticks: ptr::null::<ComponentTicks>(),
            system_ticks,
            marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        fetch: &mut Self::Fetch<'_>,
        state: &Self::State,
        _archetype: &Archetype,
        table: &Table,
    ) {
        fetch.table_ticks = table.get_column(*state).unwrap().get_ticks_const_ptr();
    }

    #[inline]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        ChangeTrackers {
            component_ticks: (*fetch.table_ticks.add(table_row)).clone(),
            system_ticks: fetch.system_ticks,
            marker: PhantomData,
        }
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for ChangeTrackers<T> {}

macro_rules! impl_tuple_fetch {
    ($(($name: ident, $state: ident)),*) => {
        #[allow(non_snake_case)]
        #[allow(clippy::unused_unit)]
        unsafe impl<$($name: WorldQuery),*> WorldQuery for ($($name,)*) {
            type Fetch<'w> = ($($name::Fetch<'w>,)*);
            type Item<'w> = ($($name::Item<'w>,)*);
            type ReadOnly = ($($name::ReadOnly,)*);
            type State = ($($name::State,)*);

            fn new_state(_world: &mut World) -> Self::State {
                ($($name::new_state(_world),)*)
            }

            fn update_component_access(state: &Self::State, _access: &mut FilteredAccess<ComponentId>) {
                let ($($name,)*) = state;
                $($name::update_component_access($name, _access);)*
            }

            fn matches_archetype(state: &Self::State, _archetype: &Archetype) -> bool {
                let ($($name,)*) = state;
                true $(&& $name::matches_archetype($name, _archetype))*
            }

            #[allow(clippy::unused_unit)]
            unsafe fn new_fetch<'w>(_world: &'w World, state: &Self::State, _system_ticks: SystemTicks) -> Self::Fetch<'w> {
                let ($($name,)*) = state;
                ($($name::new_fetch(_world, $name, _system_ticks),)*)
            }

            #[inline]
            unsafe fn set_archetype<'w>(
                _fetch: &mut Self::Fetch<'w>,
                _state: &Self::State,
                _archetype: &'w Archetype,
                _table: &'w Table
            ) {
                let ($($name,)*) = _fetch;
                let ($($state,)*) = _state;
                $($name::set_archetype($name, $state, _archetype, _table);)*
            }

            #[inline(always)]
            #[allow(clippy::unused_unit)]
            unsafe fn fetch<'w>(
                _fetch: &mut Self::Fetch<'w>,
                _entity: Entity,
                _table_row: usize
            ) -> Self::Item<'w> {
                let ($($name,)*) = _fetch;
                ($($name::fetch($name, _entity, _table_row),)*)
            }

            #[inline(always)]
            unsafe fn filter_fetch(
                _fetch: &mut Self::Fetch<'_>,
                _entity: Entity,
                _table_row: usize
            ) -> bool {
                let ($($name,)*) = _fetch;
                true $(&& $name::filter_fetch($name, _entity, _table_row))*
            }
        }

        unsafe impl<$($name: ReadOnlyWorldQuery),*> ReadOnlyWorldQuery for ($($name,)*) {}
    };
}

all_pair_tuples!(impl_tuple_fetch);
