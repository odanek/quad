use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::ecs::{
    component::{Component, ComponentId},
    entity::archetype::Archetype,
    storage::{Table, Tables},
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

    unsafe fn new(world: &World, state: &Self::State) -> Self;

    fn is_dense(&self) -> bool;

    unsafe fn set_archetype(&mut self, state: &Self::State, archetype: &Archetype, tables: &Tables);

    unsafe fn set_table(&mut self, state: &Self::State, table: &Table);

    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item;

    unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item;
}

pub unsafe trait FetchState: Send + Sync + Sized {
    fn new(world: &mut World) -> Self;
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>);
    fn matches_archetype(&self, archetype: &Archetype) -> bool;
    fn matches_table(&self, table: &Table) -> bool;
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

    #[inline]
    fn matches_table(&self, _table: &Table) -> bool {
        true
    }
}

impl<'w> Fetch<'w> for EntityFetch {
    type Item = Entity;
    type State = EntityState;

    #[inline]
    fn is_dense(&self) -> bool {
        true
    }

    unsafe fn new(_world: &World, _state: &Self::State) -> Self {
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
    unsafe fn set_table(&mut self, _state: &Self::State, table: &Table) {
        self.entities = table.entities().as_ptr();
    }

    #[inline]
    unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
        *self.entities.add(table_row)
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
                std::any::type_name::<T>());
        }
        access.add_read(self.component_id)
    }

    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }

    fn matches_table(&self, table: &Table) -> bool {
        table.has_column(self.component_id)
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

    #[inline]
    fn is_dense(&self) -> bool {
        true
    }

    unsafe fn new(_world: &World, _state: &Self::State) -> Self {
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
        let column = tables[archetype.table_id()]
            .get_column(state.component_id)
            .unwrap();
        self.table_components = column.get_data_ptr().cast::<T>();
    }

    #[inline]
    unsafe fn set_table(&mut self, state: &Self::State, table: &Table) {
        self.table_components = table
            .get_column(state.component_id)
            .unwrap()
            .get_data_ptr()
            .cast::<T>();
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        &*self.table_components.as_ptr().add(archetype_index)
    }

    #[inline]
    unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
        &*self.table_components.as_ptr().add(table_row)
    }
}
