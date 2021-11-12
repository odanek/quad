use std::slice::Iter;

use crate::ecs::{
    entity::{ArchetypeId, Archetypes},
    storage::Tables,
    system::SystemTicks,
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
    pub unsafe fn new(
        world: &'w World,
        query_state: &'s QueryState<Q, F>,
        system_ticks: SystemTicks,
    ) -> Self {
        let fetch = <Q::Fetch as Fetch>::new(world, &query_state.fetch_state, system_ticks);
        let filter = <F::Fetch as Fetch>::new(world, &query_state.filter_state, system_ticks);

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

    pub fn none_remaining(mut self) -> bool {
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
    type Item = <Q::Fetch as Fetch<'w, 's>>::Item;

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

impl<'w, 's, Q: WorldQuery> ExactSizeIterator for QueryIter<'w, 's, Q, ()> {
    fn len(&self) -> usize {
        self.query_state
            .matched_archetypes
            .iter()
            .map(|id| self.world.archetype(*id).len())
            .sum()
    }
}
