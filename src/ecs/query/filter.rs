use std::{cell::UnsafeCell, marker::PhantomData, ptr};

use crate::{
    ecs::{
        component::{Component, ComponentId, ComponentTicks},
        entity::{Archetype, Entity},
        storage::Table,
        system::SystemTicks,
        World,
    },
    macros::{all_pair_tuples, all_tuples},
};

use super::{
    access::FilteredAccess,
    fetch::{ReadOnlyWorldQuery, WorldQuery},
};

pub struct With<T>(PhantomData<T>);

unsafe impl<T: Component> WorldQuery for With<T> {
    type Fetch<'w> = ();
    type Item<'w> = ();
    type ReadOnly = Self;
    type State = ComponentId;

    fn new_state(world: &mut World) -> ComponentId {
        world.register_component::<T>()
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        access.add_with(*state);
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch(_world: &World, _state: &Self::State, _system_ticks: SystemTicks) {}

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
        _fetch: &mut Self::Fetch<'_>,
        _entity: Entity,
        _table_row: usize,
    ) -> Self::Item<'w> {
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for With<T> {}

pub struct Without<T>(PhantomData<T>);

unsafe impl<T: Component> WorldQuery for Without<T> {
    type Fetch<'w> = ();
    type Item<'w> = ();
    type ReadOnly = Self;
    type State = ComponentId;

    fn new_state(world: &mut World) -> ComponentId {
        world.register_component::<T>()
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        access.add_without(*state);
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        !archetype.contains(*state)
    }

    unsafe fn new_fetch(_world: &World, _state: &Self::State, _system_ticks: SystemTicks) {}

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
        _entity: Entity,
        _table_row: usize,
    ) -> Self::Item<'w> {
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for Without<T> {}

pub struct Or<T>(pub T);

pub struct OrFetch<'w, T: WorldQuery> {
    fetch: T::Fetch<'w>,
    matches: bool,
}

macro_rules! impl_query_filter_tuple {
    ($(($filter: ident, $state: ident)),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        #[allow(clippy::unused_unit)]
        unsafe impl<$($filter: WorldQuery),*> WorldQuery for Or<($($filter,)*)> {
            type Fetch<'w> = ($(OrFetch<'w, $filter>,)*);
            type Item<'w> = bool;
            type ReadOnly = Or<($($filter::ReadOnly,)*)>;
            type State = ($($filter::State,)*);

            unsafe fn new_fetch<'w>(world: &'w World, state: &Self::State, system_ticks: SystemTicks) -> Self::Fetch<'w> {
                let ($($filter,)*) = state;
                ($(OrFetch {
                    fetch: $filter::new_fetch(world, $filter, system_ticks),
                    matches: false,
                },)*)
            }

            #[inline]
            unsafe fn set_archetype<'w>(
                fetch: &mut Self::Fetch<'w>,
                state: & Self::State,
                archetype: &'w Archetype,
                table: &'w Table
            ) {
                let ($($filter,)*) = fetch;
                let ($($state,)*) = &state;
                $(
                    $filter.matches = $filter::matches_archetype($state, archetype);
                    if $filter.matches {
                        $filter::set_archetype(&mut $filter.fetch, $state, archetype, table);
                    }
                )*
            }

            #[inline(always)]
            unsafe fn fetch<'w>(
                fetch: &mut Self::Fetch<'w>,
                _entity: Entity,
                _table_row: usize
            ) -> Self::Item<'w> {
                let ($($filter,)*) = fetch;
                false $(|| ($filter.matches && $filter::filter_fetch(&mut $filter.fetch, _entity, _table_row)))*
            }

            #[inline(always)]
            unsafe fn filter_fetch<'w>(
                fetch: &mut Self::Fetch<'w>,
                entity: Entity,
                table_row: usize
            ) -> bool {
                Self::fetch(fetch, entity, table_row)
            }

            fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
                let ($($filter,)*) = state;

                // We do not unconditionally add `$filter`'s `with`/`without` accesses to `access`
                // as this would be unsound. For example the following two queries should conflict:
                // - Query<&mut B, Or<(With<A>, ())>>
                // - Query<&mut B, Without<A>>
                //
                // If we were to unconditionally add `$name`'s `with`/`without` accesses then `Or<(With<A>, ())>`
                // would have a `With<A>` access which is incorrect as this `WorldQuery` will match entities that
                // do not have the `A` component. This is the same logic as the `AnyOf<...>: WorldQuery` impl.
                //
                // The correct thing to do here is to only add a `with`/`without` access to `_access` if all
                // `$filter` params have that `with`/`without` access. More jargony put- we add the intersection
                // of all `with`/`without` accesses of the `$filter` params to `access`.
                let mut _intersected_access = access.clone();
                let mut _not_first = false;
                $(
                    if _not_first {
                        let mut intermediate = access.clone();
                        $filter::update_component_access($filter, &mut intermediate);
                        _intersected_access.extend_intersect_filter(&intermediate);
                        _intersected_access.extend_access(&intermediate);
                    } else {
                        $filter::update_component_access($filter, &mut _intersected_access);
                        _not_first = true;
                    }
                )*

                *access = _intersected_access;
            }

            fn new_state(world: &mut World) -> Self::State {
                ($($filter::new_state(world),)*)
            }

            fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
                let ($($filter,)*) = _state;
                false $(|| $filter::matches_archetype($filter, _archetype))*
            }
        }

        // SAFETY: filters are read only
        unsafe impl<$($filter: ReadOnlyWorldQuery),*> ReadOnlyWorldQuery for Or<($($filter,)*)> {}
    };
}

all_pair_tuples!(impl_query_filter_tuple);

pub struct Added<T>(PhantomData<T>);

pub struct AddedFetch<'w, T> {
    table_ticks: *const UnsafeCell<ComponentTicks>,
    marker: PhantomData<&'w [T]>,
    system_ticks: SystemTicks,
}

unsafe impl<T: Component> WorldQuery for Added<T> {
    type Fetch<'w> = AddedFetch<'w, T>;
    type Item<'w> = bool;
    type ReadOnly = Self;
    type State = ComponentId;

    fn new_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(*state) {
            panic!("AddedState<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>());
        }
        access.add_read(*state);
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        system_ticks: SystemTicks,
    ) -> Self::Fetch<'w> {
        AddedFetch {
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            marker: PhantomData,
            system_ticks,
        }
    }

    unsafe fn set_archetype(
        fetch: &mut Self::Fetch<'_>,
        state: &Self::State,
        _archetype: &Archetype,
        table: &Table,
    ) {
        fetch.table_ticks = table.get_column(*state).unwrap().get_ticks_ptr();
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        let ticks = &*(*fetch.table_ticks.add(table_row)).get();
        ticks.is_added(fetch.system_ticks.last_change_tick)
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for Added<T> {}

pub struct Changed<T>(PhantomData<T>);

pub struct ChangedFetch<'w, T> {
    table_ticks: *const UnsafeCell<ComponentTicks>,
    marker: PhantomData<&'w [T]>,
    system_ticks: SystemTicks,
}

unsafe impl<T: Component> WorldQuery for Changed<T> {
    type Fetch<'w> = ChangedFetch<'w, T>;
    type Item<'w> = bool;
    type ReadOnly = Self;
    type State = ComponentId;

    fn new_state(world: &mut World) -> Self::State {
        world.register_component::<T>()
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(*state) {
            panic!("AddedState<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>());
        }
        access.add_read(*state);
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.contains(*state)
    }

    unsafe fn new_fetch<'w>(
        _world: &'w World,
        _state: &Self::State,
        system_ticks: SystemTicks,
    ) -> Self::Fetch<'w> {
        ChangedFetch {
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            marker: PhantomData,
            system_ticks,
        }
    }

    unsafe fn set_archetype(
        fetch: &mut Self::Fetch<'_>,
        state: &Self::State,
        _archetype: &Archetype,
        table: &Table,
    ) {
        fetch.table_ticks = table.get_column(*state).unwrap().get_ticks_ptr();
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: usize,
    ) -> Self::Item<'w> {
        let ticks = &*(*fetch.table_ticks.add(table_row)).get();
        ticks.is_changed(fetch.system_ticks.last_change_tick)
    }
}

unsafe impl<T: Component> ReadOnlyWorldQuery for Changed<T> {}

pub trait ArchetypeFilter {}

impl<T> ArchetypeFilter for With<T> {}
impl<T> ArchetypeFilter for Without<T> {}

macro_rules! impl_archetype_filter_tuple {
    ($($filter: ident),*) => {
        impl<$($filter: ArchetypeFilter),*> ArchetypeFilter for ($($filter,)*) {}

        impl<$($filter: ArchetypeFilter),*> ArchetypeFilter for Or<($($filter,)*)> {}
    };
}

all_tuples!(impl_archetype_filter_tuple);
