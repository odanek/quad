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

impl<'w, 's, Q: WorldQuery, F: WorldQuery> Iterator for QueryIter<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
{
    type Item = <Q::Fetch as Fetch<'w>>::Item;

    // NOTE: If you are changing query iteration code, remember to update the following places, where relevant:
    // QueryIter, QueryIterationCursor, QueryState::for_each_unchecked_manual, QueryState::par_for_each_unchecked_manual
    // We can't currently reuse QueryIterationCursor in QueryIter for performance reasons. See #1763 for context.
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.is_dense {
                loop {
                    if self.current_index == self.current_len {
                        let table_id = self.table_id_iter.next()?;
                        let table = &self.tables[*table_id];
                        self.fetch.set_table(&self.query_state.fetch_state, table);
                        self.filter.set_table(&self.query_state.filter_state, table);
                        self.current_len = table.len();
                        self.current_index = 0;
                        continue;
                    }

                    if !self.filter.table_filter_fetch(self.current_index) {
                        self.current_index += 1;
                        continue;
                    }

                    let item = self.fetch.table_fetch(self.current_index);

                    self.current_index += 1;
                    return Some(item);
                }
            } else {
                loop {
                    if self.current_index == self.current_len {
                        let archetype_id = self.archetype_id_iter.next()?;
                        let archetype = &self.archetypes[*archetype_id];
                        self.fetch.set_archetype(
                            &self.query_state.fetch_state,
                            archetype,
                            self.tables,
                        );
                        self.filter.set_archetype(
                            &self.query_state.filter_state,
                            archetype,
                            self.tables,
                        );
                        self.current_len = archetype.len();
                        self.current_index = 0;
                        continue;
                    }

                    if !self.filter.archetype_filter_fetch(self.current_index) {
                        self.current_index += 1;
                        continue;
                    }

                    let item = self.fetch.archetype_fetch(self.current_index);
                    self.current_index += 1;
                    return Some(item);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_size = self
            .query_state
            .matched_archetypes
            .iter()
            .map(|id| self.world.archetype(*id).len())
            .sum();

        (0, Some(max_size))
    }
}

// impl<'w, 's, Q: WorldQuery> ExactSizeIterator for QueryIter<'w, 's, Q, ()> {
//     fn len(&self) -> usize {
//         self.query_state
//             .matched_archetypes
//             .iter()
//             .map(|id| self.world.archetype(*id).len())
//             .sum()
//     }
// }

// TODO: QueryIterationCursor, etc.