use thiserror::Error;

use crate::ecs::{
    component::{ComponentId, Tick},
    entity::{ArchetypeGeneration, ArchetypeId},
    system::SystemTicks,
    Entity, World,
};

use super::{
    access::FilteredAccess,
    fetch::{Fetch, FetchState, NopFetch, ReadOnlyFetch, WorldQuery},
    filter::FilterFetch,
    iter::QueryIter,
};

pub struct QueryState<Q: WorldQuery, F: WorldQuery = ()>
where
    F::Fetch: FilterFetch,
{
    pub(crate) archetype_generation: ArchetypeGeneration,
    pub(crate) matched_archetypes: Vec<ArchetypeId>,
    pub(crate) component_access: FilteredAccess<ComponentId>,
    pub(crate) fetch_state: Q::State,
    pub(crate) filter_state: F::State,
}

impl<Q: WorldQuery, F: WorldQuery> QueryState<Q, F>
where
    F::Fetch: FilterFetch,
{
    pub fn new(world: &mut World) -> Self {
        let fetch_state = <Q::State as FetchState>::new(world);
        let filter_state = <F::State as FetchState>::new(world);

        let mut component_access = FilteredAccess::default();
        fetch_state.update_component_access(&mut component_access);

        let mut filter_component_access = FilteredAccess::default();
        filter_state.update_component_access(&mut filter_component_access);

        component_access.extend(&filter_component_access);

        Self {
            archetype_generation: ArchetypeGeneration::initial(),
            fetch_state,
            filter_state,
            component_access,
            matched_archetypes: Default::default(),
        }
    }

    #[inline]
    pub fn is_empty(&self, world: &World) -> bool {
        let tick = Tick::default();
        unsafe {
            self.iter_unchecked_manual::<NopFetch<Q::State>>(world, SystemTicks::new(tick, tick))
                .none_remaining()
        }
    }

    pub fn update_archetypes(&mut self, world: &World) {
        let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

        for archetype_index in archetype_index_range {
            let archetype = &archetypes[ArchetypeId::new(archetype_index)];

            if self.fetch_state.matches_archetype(archetype)
                && self.filter_state.matches_archetype(archetype)
            {
                self.matched_archetypes.push(archetype.id());
            }
        }
    }

    #[inline]
    pub fn get<'w, 's>(
        &'s mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w, 's>>::Item, QueryEntityError>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe { self.get_unchecked(world, entity) }
    }

    #[inline]
    pub fn get_mut<'w, 's>(
        &'s mut self,
        world: &'w mut World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w, 's>>::Item, QueryEntityError> {
        unsafe { self.get_unchecked(world, entity) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<'w, 's>(
        &'s mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w, 's>>::Item, QueryEntityError> {
        self.update_archetypes(world);
        self.get_unchecked_manual::<Q::Fetch>(
            world,
            entity,
            SystemTicks::new(world.last_change_tick(), world.change_tick()),
        )
    }

    #[inline]
    pub fn get_manual<'w, 's>(
        &'s self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::ReadOnlyFetch as Fetch<'w, 's>>::Item, QueryEntityError> {
        // TODO Validate world
        unsafe {
            self.get_unchecked_manual::<Q::ReadOnlyFetch>(
                world,
                entity,
                SystemTicks::new(world.last_change_tick(), world.change_tick()),
            )
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_manual<'w, 's, QF: Fetch<'w, 's, State = Q::State>>(
        &'s self,
        world: &'w World,
        entity: Entity,
        system_ticks: SystemTicks,
    ) -> Result<QF::Item, QueryEntityError> {
        let location = world
            .entities()
            .get(entity)
            .ok_or(QueryEntityError::NoSuchEntity)?;
        if !self.matched_archetypes.contains(&location.archetype_id) {
            return Err(QueryEntityError::QueryDoesNotMatch);
        }
        let archetype = &world.archetype(location.archetype_id);
        let mut fetch = QF::new(world, &self.fetch_state, system_ticks);
        let mut filter = <F::Fetch as Fetch>::new(world, &self.filter_state, system_ticks);

        fetch.set_archetype(&self.fetch_state, archetype, &world.storages().tables);
        filter.set_archetype(&self.filter_state, archetype, &world.storages().tables);
        if filter.archetype_filter_fetch(location.index) {
            Ok(fetch.archetype_fetch(location.index))
        } else {
            Err(QueryEntityError::QueryDoesNotMatch)
        }
    }

    #[inline]
    pub fn iter<'w, 's>(
        &'s mut self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q, Q::ReadOnlyFetch, F> {
        unsafe { self.iter_unchecked(world) }
    }

    #[inline]
    pub fn iter_mut<'w, 's>(
        &'s mut self,
        world: &'w mut World,
    ) -> QueryIter<'w, 's, Q, Q::Fetch, F> {
        unsafe { self.iter_unchecked(world) }
    }

    #[inline]
    pub fn iter_manual<'w, 's>(
        &'s self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q, Q::ReadOnlyFetch, F> {
        // TODO Validate that correct world is used
        unsafe {
            self.iter_unchecked_manual(
                world,
                SystemTicks::new(world.last_change_tick(), world.change_tick()),
            )
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn iter_unchecked<'w, 's, QF: Fetch<'w, 's, State = Q::State>>(
        &'s mut self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q, QF, F> {
        self.update_archetypes(world);
        self.iter_unchecked_manual(
            world,
            SystemTicks::new(world.last_change_tick(), world.change_tick()),
        )
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn iter_unchecked_manual<'w, 's, QF: Fetch<'w, 's, State = Q::State>>(
        &'s self,
        world: &'w World,
        system_ticks: SystemTicks,
    ) -> QueryIter<'w, 's, Q, QF, F> {
        QueryIter::new(world, self, system_ticks)
    }

    #[inline]
    pub fn for_each<'w, 's>(
        &'s mut self,
        world: &'w World,
        func: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item),
    ) where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe {
            self.for_each_unchecked(world, func);
        }
    }

    #[inline]
    pub fn for_each_mut<'w, 's>(
        &'s mut self,
        world: &'w mut World,
        func: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item),
    ) {
        unsafe {
            self.for_each_unchecked(world, func);
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn for_each_unchecked<'w, 's>(
        &'s mut self,
        world: &'w World,
        func: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item),
    ) {
        self.update_archetypes(world);
        self.for_each_unchecked_manual(
            world,
            func,
            SystemTicks::new(world.last_change_tick(), world.change_tick()),
        );
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn for_each_unchecked_manual<'w, 's>(
        &'s self,
        world: &'w World,
        mut func: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item),
        system_ticks: SystemTicks,
    ) {
        let mut fetch = <Q::Fetch as Fetch>::new(world, &self.fetch_state, system_ticks);
        let mut filter = <F::Fetch as Fetch>::new(world, &self.filter_state, system_ticks);
        let tables = &world.storages().tables;
        for archetype_id in self.matched_archetypes.iter() {
            let archetype = world.archetype(*archetype_id);
            fetch.set_archetype(&self.fetch_state, archetype, tables);
            filter.set_archetype(&self.filter_state, archetype, tables);

            for archetype_index in 0..archetype.len() {
                if !filter.archetype_filter_fetch(archetype_index) {
                    continue;
                }
                func(fetch.archetype_fetch(archetype_index));
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum QueryEntityError {
    #[error("The given entity does not have the requested component.")]
    QueryDoesNotMatch,
    #[error("The requested entity does not exist.")]
    NoSuchEntity,
}
