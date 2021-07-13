use crate::ecs::{
    entity::archetype::{ArchetypeId, Archetypes},
    storage::{TableId, Tables},
    World,
};

use super::{
    fetch::{Fetch, WorldQuery},
    filter::FilterFetch,
    state::QueryState,
};

pub struct QueryIter<'w, 's, Q: WorldQuery, F: WorldQuery>
where
    F::Fetch: FilterFetch,
{
    tables: &'w Tables,
    archetypes: &'w Archetypes,
    query_state: &'s QueryState<Q, F>,
    world: &'w World,
    table_id_iter: std::collections::hash_set::Iter<'s, TableId>,
    archetype_id_iter: std::collections::hash_set::Iter<'s, ArchetypeId>,
    fetch: Q::Fetch,
    filter: F::Fetch,
    current_len: usize,
    current_index: usize,
    is_dense: bool,
}

impl<'w, 's, Q: WorldQuery, F: WorldQuery> QueryIter<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
{
    pub(crate) unsafe fn new(world: &'w World, query_state: &'s QueryState<Q, F>) -> Self {
        let fetch = <Q::Fetch as Fetch>::new(world, &query_state.fetch_state);
        let filter = <F::Fetch as Fetch>::new(world, &query_state.filter_state);

        QueryIter {
            world,
            query_state,
            tables: &world.storages().tables,
            archetypes: world.archetypes(),
            is_dense: fetch.is_dense() && filter.is_dense(),
            fetch,
            filter,
            table_id_iter: query_state.matched_tables.iter(),
            archetype_id_iter: query_state.matched_archetypes.iter(),
            current_len: 0,
            current_index: 0,
        }
    }
}
