use crate::ecs::{
    component::ComponentId,
    entity::archetype::Archetype,
    storage::{Table, Tables},
    World,
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
    // fn update_archetype_component_access(
    //     &self,
    //     archetype: &Archetype,
    //     access: &mut Access<ArchetypeComponentId>,
    // );
    fn matches_archetype(&self, archetype: &Archetype) -> bool;
    fn matches_table(&self, table: &Table) -> bool;
}
