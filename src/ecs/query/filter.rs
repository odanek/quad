use std::{cell::UnsafeCell, marker::PhantomData, ptr};

use crate::ecs::{
    component::{Component, ComponentId, ComponentTicks},
    entity::archetype::Archetype,
    storage::Tables,
    system::SystemTicks,
    World,
};

use super::{
    access::FilteredAccess,
    fetch::{Fetch, FetchState, WorldQuery},
};

pub trait FilterFetch: for<'a> Fetch<'a> {
    unsafe fn archetype_filter_fetch(&mut self, archetype_index: usize) -> bool;
}

impl<T> FilterFetch for T
where
    T: for<'a> Fetch<'a, Item = bool>,
{
    #[inline]
    unsafe fn archetype_filter_fetch(&mut self, archetype_index: usize) -> bool {
        self.archetype_fetch(archetype_index)
    }
}

pub struct With<T>(PhantomData<T>);

impl<T: Component> WorldQuery for With<T> {
    type Fetch = WithFetch<T>;
    type State = WithState<T>;
}

pub struct WithFetch<T> {
    marker: PhantomData<T>,
}

pub struct WithState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

unsafe impl<T: Component> FetchState for WithState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        Self {
            component_id,
            marker: PhantomData,
        }
    }

    #[inline]
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        access.add_with(self.component_id);
    }

    #[inline]
    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

impl<'a, T: Component> Fetch<'a> for WithFetch<T> {
    type Item = bool;
    type State = WithState<T>;

    unsafe fn new(_world: &World, _state: &Self::State, _system_ticks: SystemTicks) -> Self {
        Self {
            marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        _state: &Self::State,
        _archetype: &Archetype,
        _tables: &Tables,
    ) {
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, _archetype_index: usize) -> Self::Item {
        true
    }
}

pub struct Without<T>(PhantomData<T>);

impl<T: Component> WorldQuery for Without<T> {
    type Fetch = WithoutFetch<T>;
    type State = WithoutState<T>;
}

pub struct WithoutFetch<T> {
    marker: PhantomData<T>,
}

pub struct WithoutState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

unsafe impl<T: Component> FetchState for WithoutState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        Self {
            component_id,
            marker: PhantomData,
        }
    }

    #[inline]
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        access.add_without(self.component_id);
    }

    #[inline]
    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        !archetype.contains(self.component_id)
    }
}

impl<'a, T: Component> Fetch<'a> for WithoutFetch<T> {
    type Item = bool;
    type State = WithoutState<T>;

    unsafe fn new(_world: &World, _state: &Self::State, _system_ticks: SystemTicks) -> Self {
        Self {
            marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        _state: &Self::State,
        _archetype: &Archetype,
        _tables: &Tables,
    ) {
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, _archetype_index: usize) -> bool {
        true
    }
}

pub struct Or<T>(pub T);

pub struct OrFetch<T: FilterFetch> {
    fetch: T,
    matches: bool,
}

macro_rules! impl_query_filter_tuple {
    ($(($filter: ident, $state: ident)),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'a, $($filter: FilterFetch),*> FilterFetch for ($($filter,)*) {
            #[inline]
            unsafe fn archetype_filter_fetch(&mut self, archetype_index: usize) -> bool {
                let ($($filter,)*) = self;
                true $(&& $filter.archetype_filter_fetch(archetype_index))*
            }
        }

        impl<$($filter: WorldQuery),*> WorldQuery for Or<($($filter,)*)>
            where $($filter::Fetch: FilterFetch),*
        {
            type Fetch = Or<($(OrFetch<$filter::Fetch>,)*)>;
            type State = Or<($($filter::State,)*)>;
        }


        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'a, $($filter: FilterFetch),*> Fetch<'a> for Or<($(OrFetch<$filter>,)*)> {
            type State = Or<($(<$filter as Fetch<'a>>::State,)*)>;
            type Item = bool;

            unsafe fn new(world: &World, state: &Self::State, system_ticks: SystemTicks) -> Self {
                let ($($filter,)*) = &state.0;
                Or(($(OrFetch {
                    fetch: $filter::new(world, $filter, system_ticks),
                    matches: false,
                },)*))
            }

            #[inline]
            unsafe fn set_archetype(&mut self, state: &Self::State, archetype: &Archetype, tables: &Tables) {
                let ($($filter,)*) = &mut self.0;
                let ($($state,)*) = &state.0;
                $(
                    $filter.matches = $state.matches_archetype(archetype);
                    if $filter.matches {
                        $filter.fetch.set_archetype($state, archetype, tables);
                    }
                )*
            }

            #[inline]
            unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> bool {
                let ($($filter,)*) = &mut self.0;
                false $(|| ($filter.matches && $filter.fetch.archetype_filter_fetch(archetype_index)))*
            }
        }

        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        unsafe impl<$($filter: FetchState),*> FetchState for Or<($($filter,)*)> {
            fn new(world: &mut World) -> Self {
                Or(($($filter::new(world),)*))
            }

            fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
                let ($($filter,)*) = &self.0;
                $($filter.update_component_access(access);)*
            }

            fn matches_archetype(&self, archetype: &Archetype) -> bool {
                let ($($filter,)*) = &self.0;
                false $(|| $filter.matches_archetype(archetype))*
            }
        }
    };
}

all_pair_tuples!(impl_query_filter_tuple);

pub struct Added<T>(PhantomData<T>);

pub struct AddedFetch<T> {
    table_ticks: *const UnsafeCell<ComponentTicks>,
    marker: PhantomData<T>,
    system_ticks: SystemTicks,
}

pub struct AddedState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Component> WorldQuery for Added<T> {
    type Fetch = AddedFetch<T>;
    type State = AddedState<T>;
}

unsafe impl<T: Component> FetchState for AddedState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        Self {
            component_id,
            marker: PhantomData,
        }
    }

    #[inline]
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(self.component_id) {
            panic!("$state_name<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>());
        }
        access.add_read(self.component_id);
    }

    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

impl<'w, T: Component> Fetch<'w> for AddedFetch<T> {
    type State = AddedState<T>;
    type Item = bool;

    unsafe fn new(_world: &World, _state: &Self::State, system_ticks: SystemTicks) -> Self {
        Self {
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            marker: PhantomData,
            system_ticks,
        }
    }

    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
        tables: &Tables,
    ) {
        let table = &tables[archetype.table_id()];
        self.table_ticks = table
            .get_column(state.component_id)
            .unwrap()
            .get_ticks_ptr();
    }

    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> bool {
        let ticks = &*(&*self.table_ticks.add(archetype_index)).get();
        ticks.is_added(self.system_ticks.last_change_tick)
    }
}

pub struct Changed<T>(PhantomData<T>);

pub struct ChangedFetch<T> {
    table_ticks: *const UnsafeCell<ComponentTicks>,
    marker: PhantomData<T>,
    system_ticks: SystemTicks,
}

pub struct ChangedState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Component> WorldQuery for Changed<T> {
    type Fetch = ChangedFetch<T>;
    type State = ChangedState<T>;
}

unsafe impl<T: Component> FetchState for ChangedState<T> {
    fn new(world: &mut World) -> Self {
        let component_id = world.register_component::<T>();
        Self {
            component_id,
            marker: PhantomData,
        }
    }

    #[inline]
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(self.component_id) {
            panic!("$state_name<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>());
        }
        access.add_read(self.component_id);
    }

    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

impl<'w, T: Component> Fetch<'w> for ChangedFetch<T> {
    type State = ChangedState<T>;
    type Item = bool;

    unsafe fn new(_world: &World, _state: &Self::State, system_ticks: SystemTicks) -> Self {
        Self {
            table_ticks: ptr::null::<UnsafeCell<ComponentTicks>>(),
            marker: PhantomData,
            system_ticks,
        }
    }

    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
        tables: &Tables,
    ) {
        let table = &tables[archetype.table_id()];
        self.table_ticks = table
            .get_column(state.component_id)
            .unwrap()
            .get_ticks_ptr();
    }

    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> bool {
        let ticks = &*(&*self.table_ticks.add(archetype_index)).get();
        ticks.is_changed(self.system_ticks.last_change_tick)
    }
}
