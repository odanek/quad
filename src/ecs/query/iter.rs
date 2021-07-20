use std::slice::Iter;

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
    table_id_iter: Iter<'s, TableId>,
    archetype_id_iter: Iter<'s, ArchetypeId>,
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

    pub(crate) fn none_remaining(mut self) -> bool {
        unsafe {
            if self.is_dense {
                loop {
                    if self.current_index == self.current_len {
                        let table_id = match self.table_id_iter.next() {
                            Some(table_id) => table_id,
                            None => return true,
                        };
                        let table = &self.tables[*table_id];
                        self.filter.set_table(&self.query_state.filter_state, table);
                        self.current_len = table.len();
                        self.current_index = 0;
                        continue;
                    }

                    if !self.filter.table_filter_fetch(self.current_index) {
                        self.current_index += 1;
                        continue;
                    }

                    return false;
                }
            } else {
                loop {
                    if self.current_index == self.current_len {
                        let archetype_id = match self.archetype_id_iter.next() {
                            Some(archetype_id) => archetype_id,
                            None => return true,
                        };
                        let archetype = &self.archetypes[*archetype_id];
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

                    return false;
                }
            }
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

struct QueryIterationCursor<'s, Q: WorldQuery, F: WorldQuery> {
    table_id_iter: Iter<'s, TableId>,
    archetype_id_iter: Iter<'s, ArchetypeId>,
    fetch: Q::Fetch,
    filter: F::Fetch,
    current_len: usize,
    current_index: usize,
    is_dense: bool,
}

impl<'s, Q: WorldQuery, F: WorldQuery> Clone for QueryIterationCursor<'s, Q, F>
where
    Q::Fetch: Clone,
    F::Fetch: Clone,
{
    fn clone(&self) -> Self {
        Self {
            table_id_iter: self.table_id_iter.clone(),
            archetype_id_iter: self.archetype_id_iter.clone(),
            fetch: self.fetch.clone(),
            filter: self.filter.clone(),
            current_len: self.current_len,
            current_index: self.current_index,
            is_dense: self.is_dense,
        }
    }
}

impl<'s, Q: WorldQuery, F: WorldQuery> QueryIterationCursor<'s, Q, F>
where
    F::Fetch: FilterFetch,
{
    unsafe fn new_empty(world: &World, query_state: &'s QueryState<Q, F>) -> Self {
        QueryIterationCursor {
            table_id_iter: [].iter(),
            archetype_id_iter: [].iter(),
            ..Self::new(world, query_state)
        }
    }

    unsafe fn new(world: &World, query_state: &'s QueryState<Q, F>) -> Self {
        let fetch = <Q::Fetch as Fetch>::new(world, &query_state.fetch_state);
        let filter = <F::Fetch as Fetch>::new(world, &query_state.filter_state);
        QueryIterationCursor {
            is_dense: fetch.is_dense() && filter.is_dense(),
            fetch,
            filter,
            table_id_iter: query_state.matched_tables.iter(),
            archetype_id_iter: query_state.matched_archetypes.iter(),
            current_len: 0,
            current_index: 0,
        }
    }

    #[inline]
    unsafe fn peek_last<'w>(&mut self) -> Option<<Q::Fetch as Fetch<'w>>::Item> {
        if self.current_index > 0 {
            if self.is_dense {
                Some(self.fetch.table_fetch(self.current_index - 1))
            } else {
                Some(self.fetch.archetype_fetch(self.current_index - 1))
            }
        } else {
            None
        }
    }

    #[inline(always)]
    unsafe fn next<'w>(
        &mut self,
        tables: &'w Tables,
        archetypes: &'w Archetypes,
        query_state: &'s QueryState<Q, F>,
    ) -> Option<<Q::Fetch as Fetch<'w>>::Item> {
        if self.is_dense {
            loop {
                if self.current_index == self.current_len {
                    let table_id = self.table_id_iter.next()?;
                    let table = &tables[*table_id];
                    self.fetch.set_table(&query_state.fetch_state, table);
                    self.filter.set_table(&query_state.filter_state, table);
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
                    let archetype = &archetypes[*archetype_id];
                    self.fetch
                        .set_archetype(&query_state.fetch_state, archetype, tables);
                    self.filter
                        .set_archetype(&query_state.filter_state, archetype, tables);
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
