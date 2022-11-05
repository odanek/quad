use thiserror::Error;

use crate::ecs::{
    component::{ComponentId, Tick},
    entity::{ArchetypeGeneration, ArchetypeId},
    system::SystemTicks,
    Entity, World,
};

use super::{
    access::FilteredAccess,
    fetch::{ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
    iter::QueryIter,
};

pub struct QueryState<Q: WorldQuery, F: ReadOnlyWorldQuery = ()> {
    pub(crate) archetype_generation: ArchetypeGeneration,
    pub(crate) matched_archetype_ids: Vec<ArchetypeId>,
    pub(crate) component_access: FilteredAccess<ComponentId>,
    pub(crate) fetch_state: Q::State,
    pub(crate) filter_state: F::State,
}

impl<Q: WorldQuery, F: ReadOnlyWorldQuery> QueryState<Q, F> {
    pub fn new(world: &mut World) -> Self {
        let fetch_state = Q::new_state(world);
        let filter_state = F::new_state(world);

        let mut component_access = FilteredAccess::default();
        Q::update_component_access(&fetch_state, &mut component_access);

        let mut filter_component_access = FilteredAccess::default();
        F::update_component_access(&filter_state, &mut filter_component_access);

        component_access.extend(&filter_component_access);

        Self {
            archetype_generation: ArchetypeGeneration::initial(),
            fetch_state,
            filter_state,
            component_access,
            matched_archetype_ids: Default::default(),
        }
    }

    // TODO Ugly
    pub fn as_readonly(&self) -> &QueryState<Q::ReadOnly, F::ReadOnly> {
        unsafe {
            &*(self as *const QueryState<Q, F> as *const QueryState<Q::ReadOnly, F::ReadOnly>)
        }
    }

    #[inline]
    pub fn is_empty(&self, world: &World) -> bool {
        let tick = Tick::default();
        // TODO: Bevy uses nop
        unsafe {
            self.iter_unchecked_manual(world, SystemTicks::new(tick, tick))
                .next()
                .is_none()
        }
    }

    pub fn update_archetypes(&mut self, world: &World) {
        let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

        for archetype_index in archetype_index_range {
            let archetype = &archetypes[ArchetypeId::new(archetype_index)];

            if Q::matches_archetype(&self.fetch_state, archetype)
                && F::matches_archetype(&self.filter_state, archetype)
            {
                self.matched_archetype_ids.push(archetype.id());
            }
        }
    }

    #[inline]
    pub fn get<'w>(
        &mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<ROQueryItem<'w, Q>, QueryEntityError> {
        self.update_archetypes(world);
        unsafe {
            self.as_readonly()
                .get_unchecked_manual(world, entity, world.ticks())
        }
    }

    #[inline]
    pub fn get_mut<'w, 's>(
        &'s mut self,
        world: &'w mut World,
        entity: Entity,
    ) -> Result<Q::Item<'w>, QueryEntityError> {
        self.update_archetypes(world);
        unsafe { self.get_unchecked_manual(world, entity, world.ticks()) }
    }

    #[inline]
    pub fn get_manual<'w, 's>(
        &'s self,
        world: &'w World,
        entity: Entity,
    ) -> Result<ROQueryItem<'w, Q>, QueryEntityError> {
        // TODO Validate world
        unsafe {
            self.as_readonly()
                .get_unchecked_manual(world, entity, world.ticks())
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_manual<'w>(
        &self,
        world: &'w World,
        entity: Entity,
        system_ticks: SystemTicks,
    ) -> Result<Q::Item<'w>, QueryEntityError> {
        let location = world
            .entities()
            .get(entity)
            .ok_or(QueryEntityError::NoSuchEntity)?;
        if !self.matched_archetype_ids.contains(&location.archetype_id) {
            return Err(QueryEntityError::QueryDoesNotMatch);
        }
        let archetype = &world.archetype(location.archetype_id);
        let mut fetch = Q::new_fetch(world, &self.fetch_state, system_ticks);
        let mut filter = F::new_fetch(world, &self.filter_state, system_ticks);
        let table = &world.storages().tables[archetype.table_id()];

        Q::set_archetype(&mut fetch, &self.fetch_state, archetype, table);
        F::set_archetype(&mut filter, &self.filter_state, archetype, table);
        if F::filter_fetch(&mut filter, entity, location.index) {
            Ok(Q::fetch(&mut fetch, entity, location.index))
        } else {
            Err(QueryEntityError::QueryDoesNotMatch)
        }
    }

    #[inline]
    pub fn single<'w>(&mut self, world: &'w World) -> ROQueryItem<'w, Q> {
        self.get_single(world).unwrap()
    }

    #[inline]
    pub fn get_single<'w>(
        &mut self,
        world: &'w World,
    ) -> Result<ROQueryItem<'w, Q>, QuerySingleError> {
        self.update_archetypes(world);
        unsafe {
            self.as_readonly()
                .get_single_unchecked_manual(world, world.ticks())
        }
    }

    #[inline]
    pub fn single_mut<'w>(&mut self, world: &'w mut World) -> Q::Item<'w> {
        self.get_single_mut(world).unwrap()
    }

    #[inline]
    pub fn get_single_mut<'w>(
        &mut self,
        world: &'w mut World,
    ) -> Result<Q::Item<'w>, QuerySingleError> {
        self.update_archetypes(world);
        unsafe { self.get_single_unchecked_manual(world, world.ticks()) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_single_unchecked_manual<'w>(
        &self,
        world: &'w World,
        system_ticks: SystemTicks,
    ) -> Result<Q::Item<'w>, QuerySingleError> {
        let mut query = self.iter_unchecked_manual(world, system_ticks);
        let first = query.next();
        let extra = query.next().is_some();

        match (first, extra) {
            (Some(r), false) => Ok(r),
            (None, _) => Err(QuerySingleError::NoEntities(std::any::type_name::<Self>())),
            (Some(_), _) => Err(QuerySingleError::MultipleEntities(std::any::type_name::<
                Self,
            >())),
        }
    }

    #[inline]
    pub fn iter<'w, 's>(
        &'s mut self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q::ReadOnly, F::ReadOnly> {
        self.update_archetypes(world);
        unsafe {
            self.as_readonly()
                .iter_unchecked_manual(world, world.ticks())
        }
    }

    #[inline]
    pub fn iter_mut<'w, 's>(&'s mut self, world: &'w mut World) -> QueryIter<'w, 's, Q, F> {
        self.update_archetypes(world);
        unsafe { self.iter_unchecked_manual(world, world.ticks()) }
    }

    #[inline]
    pub fn iter_manual<'w, 's>(
        &'s self,
        world: &'w World,
    ) -> QueryIter<'w, 's, Q::ReadOnly, F::ReadOnly> {
        // TODO Validate that correct world is used
        unsafe {
            self.as_readonly()
                .iter_unchecked_manual(world, world.ticks())
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn iter_unchecked_manual<'w, 's>(
        &'s self,
        world: &'w World,
        system_ticks: SystemTicks,
    ) -> QueryIter<'w, 's, Q, F> {
        QueryIter::new(world, self, system_ticks)
    }

    #[inline]
    pub fn for_each<'w, 's>(&'s mut self, world: &'w World, func: impl FnMut(ROQueryItem<'w, Q>)) {
        self.update_archetypes(world);
        unsafe {
            self.as_readonly()
                .for_each_unchecked_manual(world, func, world.ticks());
        }
    }

    #[inline]
    pub fn for_each_mut<'w, 's>(&'s mut self, world: &'w mut World, func: impl FnMut(Q::Item<'w>)) {
        self.update_archetypes(world);
        unsafe {
            self.for_each_unchecked_manual(world, func, world.ticks());
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn for_each_unchecked_manual<'w, 's>(
        &'s self,
        world: &'w World,
        mut func: impl FnMut(Q::Item<'w>),
        system_ticks: SystemTicks,
    ) {
        let mut fetch = Q::new_fetch(world, &self.fetch_state, system_ticks);
        let mut filter = F::new_fetch(world, &self.filter_state, system_ticks);
        let tables = &world.storages().tables;
        for archetype_id in self.matched_archetype_ids.iter() {
            let archetype = world.archetype(*archetype_id);
            let table = &tables[archetype.table_id()];

            Q::set_archetype(&mut fetch, &self.fetch_state, archetype, table);
            F::set_archetype(&mut filter, &self.filter_state, archetype, table);

            for archetype_index in 0..archetype.len() {
                let entity = *archetype.entities().get_unchecked(archetype_index);
                if !F::filter_fetch(&mut filter, entity, archetype_index) {
                    continue;
                }
                func(Q::fetch(&mut fetch, entity, archetype_index));
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

#[derive(Debug)]
pub enum QuerySingleError {
    NoEntities(&'static str),
    MultipleEntities(&'static str),
}
