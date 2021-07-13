use std::slice::Iter;

use crate::ecs::{World, entity::archetype::{ArchetypeId, Archetypes}, storage::{TableId, Tables}};

use super::{fetch::WorldQuery, filter::FilterFetch, state::QueryState};

pub struct QueryIter<'w, 's, Q: WorldQuery, F: WorldQuery>
where
    F::Fetch: FilterFetch,
{
    tables: &'w Tables,
    archetypes: &'w Archetypes,
    query_state: &'s QueryState<Q, F>,
    world: &'w World,
    table_id_iter: Iter<'s, TableId>,
    archetype_id_iter: Iter<'s, ArchetypeId>,
    fetch: Q::Fetch,
    filter: F::Fetch,
    current_len: usize,
    current_index: usize,
    is_dense: bool,
}
