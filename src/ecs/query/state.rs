use crate::ecs::{
    component::ComponentId,
    entity::archetype::{Archetype, ArchetypeGeneration, ArchetypeId},
    Entity, World,
};

use super::{
    access::FilteredAccess,
    fetch::{Fetch, FetchState, ReadOnlyFetch, WorldQuery},
    filter::FilterFetch,
    iter::QueryIter,
};

pub struct QueryState<Q: WorldQuery, F: WorldQuery /* = ()*/>
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

        let mut state = Self {
            archetype_generation: ArchetypeGeneration::initial(),
            fetch_state,
            filter_state,
            component_access,
            matched_archetypes: Default::default(),
        };
        state.update_archetypes(world);
        state
    }

    #[inline]
    pub fn is_empty(&self, world: &World) -> bool {
        unsafe { self.iter_unchecked_manual(world).none_remaining() }
    }

    pub fn update_archetypes(&mut self, world: &World) {
        let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

        for archetype_index in archetype_index_range {
            self.new_archetype(&archetypes[ArchetypeId::new(archetype_index)]);
        }
    }

    pub fn new_archetype(&mut self, archetype: &Archetype) {
        if self.fetch_state.matches_archetype(archetype)
            && self.filter_state.matches_archetype(archetype)
        {
            self.matched_archetypes.push(archetype.id());
        }
    }

    #[inline]
    pub fn get<'w>(
        &mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w>>::Item, QueryEntityError>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe { self.get_unchecked(world, entity) }
    }

    #[inline]
    pub fn get_mut<'w>(
        &mut self,
        world: &'w mut World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w>>::Item, QueryEntityError> {
        unsafe { self.get_unchecked(world, entity) }
    }

    #[inline]
    pub unsafe fn get_unchecked<'w>(
        &mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w>>::Item, QueryEntityError> {
        self.update_archetypes(world); // TODO: Are these calls necessary?
        self.get_unchecked_manual(world, entity)
    }

    pub unsafe fn get_unchecked_manual<'w>(
        &self,
        world: &'w World,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch<'w>>::Item, QueryEntityError> {
        let location = world
            .entities()
            .get(entity)
            .ok_or(QueryEntityError::NoSuchEntity)?;
        if !self.matched_archetypes.contains(&location.archetype_id) {
            return Err(QueryEntityError::QueryDoesNotMatch);
        }
        let archetype = &world.archetype(location.archetype_id);
        let mut fetch = <Q::Fetch as Fetch>::new(world, &self.fetch_state);
        let mut filter = <F::Fetch as Fetch>::new(world, &self.filter_state);

        fetch.set_archetype(&self.fetch_state, archetype, &world.storages().tables);
        filter.set_archetype(&self.filter_state, archetype, &world.storages().tables);
        if filter.archetype_filter_fetch(location.index) {
            Ok(fetch.archetype_fetch(location.index))
        } else {
            Err(QueryEntityError::QueryDoesNotMatch)
        }
    }

    #[inline]
    pub fn iter<'w, 's>(&'s mut self, world: &'w World) -> QueryIter<'w, 's, Q, F>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe { self.iter_unchecked(world) }
    }

    #[inline]
    pub fn iter_mut<'w, 's>(&'s mut self, world: &'w mut World) -> QueryIter<'w, 's, Q, F> {
        unsafe { self.iter_unchecked(world) }
    }

    #[inline]
    pub unsafe fn iter_unchecked<'w, 's>(
        &'s mut self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q, F> {
        self.update_archetypes(world);
        self.iter_unchecked_manual(world)
    }

    #[inline]
    pub(crate) unsafe fn iter_unchecked_manual<'w, 's>(
        &'s self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q, F> {
        QueryIter::new(world, self)
    }

    #[inline]
    pub fn for_each<'w>(
        &mut self,
        world: &'w World,
        func: impl FnMut(<Q::Fetch as Fetch<'w>>::Item),
    ) where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe {
            self.for_each_unchecked(world, func);
        }
    }

    #[inline]
    pub fn for_each_mut<'w>(
        &mut self,
        world: &'w mut World,
        func: impl FnMut(<Q::Fetch as Fetch<'w>>::Item),
    ) {
        unsafe {
            self.for_each_unchecked(world, func);
        }
    }

    #[inline]
    pub unsafe fn for_each_unchecked<'w>(
        &mut self,
        world: &'w World,
        func: impl FnMut(<Q::Fetch as Fetch<'w>>::Item),
    ) {
        self.update_archetypes(world);
        self.for_each_unchecked_manual(world, func);
    }

    pub(crate) unsafe fn for_each_unchecked_manual<'w, 's>(
        &'s self,
        world: &'w World,
        mut func: impl FnMut(<Q::Fetch as Fetch<'w>>::Item),
    ) {
        let mut fetch = <Q::Fetch as Fetch>::new(world, &self.fetch_state);
        let mut filter = <F::Fetch as Fetch>::new(world, &self.filter_state);
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

pub enum QueryEntityError {
    QueryDoesNotMatch,
    NoSuchEntity,
}
