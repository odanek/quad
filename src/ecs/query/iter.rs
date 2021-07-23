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
    archetype_id_iter: Iter<'s, ArchetypeId>,
    fetch: Q::Fetch,
    filter: F::Fetch,
    current_len: usize,
    current_index: usize,
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
            fetch,
            filter,
            archetype_id_iter: query_state.matched_archetypes.iter(),
            current_len: 0,
            current_index: 0,
        }
    }

    pub(crate) fn none_remaining(mut self) -> bool {
        unsafe {
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

impl<'w, 's, Q: WorldQuery, F: WorldQuery> Iterator for QueryIter<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
{
    type Item = <Q::Fetch as Fetch<'w>>::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            loop {
                if self.current_index == self.current_len {
                    let archetype_id = self.archetype_id_iter.next()?;
                    let archetype = &self.archetypes[*archetype_id];
                    self.fetch
                        .set_archetype(&self.query_state.fetch_state, archetype, self.tables);
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
    archetype_id_iter: Iter<'s, ArchetypeId>,
    fetch: Q::Fetch,
    filter: F::Fetch,
    current_len: usize,
    current_index: usize,
}

impl<'s, Q: WorldQuery, F: WorldQuery> Clone for QueryIterationCursor<'s, Q, F>
where
    Q::Fetch: Clone,
    F::Fetch: Clone,
{
    fn clone(&self) -> Self {
        Self {
            archetype_id_iter: self.archetype_id_iter.clone(),
            fetch: self.fetch.clone(),
            filter: self.filter.clone(),
            current_len: self.current_len,
            current_index: self.current_index,
        }
    }
}

impl<'s, Q: WorldQuery, F: WorldQuery> QueryIterationCursor<'s, Q, F>
where
    F::Fetch: FilterFetch,
{
    unsafe fn new_empty(world: &World, query_state: &'s QueryState<Q, F>) -> Self {
        QueryIterationCursor {
            archetype_id_iter: [].iter(),
            ..Self::new(world, query_state)
        }
    }

    unsafe fn new(world: &World, query_state: &'s QueryState<Q, F>) -> Self {
        let fetch = <Q::Fetch as Fetch>::new(world, &query_state.fetch_state);
        let filter = <F::Fetch as Fetch>::new(world, &query_state.filter_state);
        QueryIterationCursor {
            fetch,
            filter,
            archetype_id_iter: query_state.matched_archetypes.iter(),
            current_len: 0,
            current_index: 0,
        }
    }

    #[inline]
    unsafe fn peek_last<'w>(&mut self) -> Option<<Q::Fetch as Fetch<'w>>::Item> {
        if self.current_index > 0 {
            Some(self.fetch.archetype_fetch(self.current_index - 1))
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
