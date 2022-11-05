use std::{iter::FusedIterator, slice::Iter};

use crate::ecs::{
    entity::{ArchetypeId, Archetypes, Entity},
    storage::Tables,
    system::SystemTicks,
    World,
};

use super::{
    fetch::{ReadOnlyWorldQuery, WorldQuery},
    filter::ArchetypeFilter,
    state::QueryState,
};

pub struct QueryIter<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> {
    tables: &'w Tables,
    archetypes: &'w Archetypes,
    query_state: &'s QueryState<Q, F>,
    cursor: QueryIterationCursor<'w, 's, Q, F>,
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> QueryIter<'w, 's, Q, F> {
    pub unsafe fn new(
        world: &'w World,
        query_state: &'s QueryState<Q, F>,
        system_ticks: SystemTicks,
    ) -> Self {
        QueryIter {
            query_state,
            tables: &world.storages().tables,
            archetypes: world.archetypes(),
            cursor: QueryIterationCursor::new(world, query_state, system_ticks),
        }
    }
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> Iterator for QueryIter<'w, 's, Q, F> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.cursor
                .next(self.tables, self.archetypes, self.query_state)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_size = self
            .query_state
            .matched_archetype_ids
            .iter()
            .map(|id| self.archetypes[*id].len())
            .sum();

        (0, Some(max_size))
    }
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> FusedIterator for QueryIter<'w, 's, Q, F> {}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> ExactSizeIterator for QueryIter<'w, 's, Q, F>
where
    F: ArchetypeFilter,
{
    fn len(&self) -> usize {
        self.query_state
            .matched_archetype_ids
            .iter()
            .map(|id| self.archetypes[*id].len())
            .sum()
    }
}

struct QueryIterationCursor<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> {
    archetype_id_iter: Iter<'s, ArchetypeId>,
    archetype_entities: &'w [Entity],
    fetch: Q::Fetch<'w>,
    filter: F::Fetch<'w>,
    current_len: usize,
    current_index: usize,
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> QueryIterationCursor<'w, 's, Q, F> {
    unsafe fn new(
        world: &'w World,
        query_state: &'s QueryState<Q, F>,
        system_ticks: SystemTicks,
    ) -> Self {
        let fetch = Q::new_fetch(world, &query_state.fetch_state, system_ticks);
        let filter = F::new_fetch(world, &query_state.filter_state, system_ticks);
        QueryIterationCursor {
            fetch,
            filter,
            archetype_id_iter: query_state.matched_archetype_ids.iter(),
            archetype_entities: &[],
            current_len: 0,
            current_index: 0,
        }
    }

    #[inline(always)]
    unsafe fn next(
        &mut self,
        tables: &'w Tables,
        archetypes: &'w Archetypes,
        query_state: &'s QueryState<Q, F>,
    ) -> Option<Q::Item<'w>> {
        loop {
            if self.current_index == self.current_len {
                let archetype_id = self.archetype_id_iter.next()?;
                let archetype = &archetypes[*archetype_id];
                let table = &tables[archetype.table_id()];
                Q::set_archetype(&mut self.fetch, &query_state.fetch_state, archetype, table);
                F::set_archetype(
                    &mut self.filter,
                    &query_state.filter_state,
                    archetype,
                    table,
                );
                self.archetype_entities = archetype.entities();
                self.current_len = archetype.len();
                self.current_index = 0;
                continue;
            }

            let entity = *self.archetype_entities.get_unchecked(self.current_index);
            if !F::filter_fetch(&mut self.filter, entity, self.current_index) {
                self.current_index += 1;
                continue;
            }

            let item = Q::fetch(&mut self.fetch, entity, self.current_index);
            self.current_index += 1;
            return Some(item);
        }
    }
}
