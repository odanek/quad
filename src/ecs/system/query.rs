use std::any::TypeId;

use crate::ecs::{
    Entity, World,
    component::{CmptMut, Component, Tick},
    query::{
        fetch::{ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
        iter::QueryIter,
        state::{QueryEntityError, QuerySingleError, QueryState},
    },
    system::function_system::SystemMeta,
};

use super::{
    SystemTicks,
    system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState},
};

#[derive(Debug)]
pub enum QueryComponentError {
    MissingReadAccess,
    MissingWriteAccess,
    MissingComponent,
    NoSuchEntity,
}

// TODO: This definition allows to have With, Without, Or in Q, while it should be possible only in F
pub struct Query<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery = ()> {
    world: &'w World,
    state: &'s QueryState<Q, F>,
    system_ticks: SystemTicks,
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> Query<'w, 's, Q, F> {
    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(
        world: &'w World,
        state: &'s QueryState<Q, F>,
        system_ticks: SystemTicks,
    ) -> Self {
        Self {
            world,
            state,
            system_ticks,
        }
    }

    #[inline]
    pub fn iter(&self) -> QueryIter<'_, 's, Q::ReadOnly, F::ReadOnly> {
        unsafe {
            self.state
                .as_readonly()
                .iter_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> QueryIter<'_, 's, Q, F> {
        unsafe {
            self.state
                .iter_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn iter_unsafe(&self) -> QueryIter<'_, 's, Q, F> {
        self.state
            .iter_unchecked_manual(self.world, self.system_ticks)
    }

    #[inline]
    pub fn for_each<'a>(&'a self, f: impl FnMut(ROQueryItem<'a, Q>)) {
        unsafe {
            self.state
                .as_readonly()
                .for_each_unchecked_manual(self.world, f, self.system_ticks)
        };
    }

    #[inline]
    pub fn for_each_mut<'a>(&'a mut self, f: impl FnMut(Q::Item<'a>)) {
        unsafe {
            self.state
                .for_each_unchecked_manual(self.world, f, self.system_ticks)
        };
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Result<ROQueryItem<'_, Q>, QueryEntityError> {
        unsafe {
            self.state
                .as_readonly()
                .get_unchecked_manual(self.world, entity, self.system_ticks)
        }
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Result<Q::Item<'_>, QueryEntityError> {
        unsafe {
            self.state
                .get_unchecked_manual(self.world, entity, self.system_ticks)
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked(&self, entity: Entity) -> Result<Q::Item<'_>, QueryEntityError> {
        self.state
            .get_unchecked_manual(self.world, entity, self.system_ticks)
    }

    #[inline]
    pub fn get_component<T: Component>(&self, entity: Entity) -> Result<&T, QueryComponentError> {
        let world = self.world;
        let location = world
            .entities()
            .get(entity)
            .ok_or(QueryComponentError::NoSuchEntity)?;
        let component_id = world
            .components()
            .get_id(TypeId::of::<T>())
            .ok_or(QueryComponentError::MissingComponent)?;
        if self.state.component_access.has_read(component_id) {
            world
                .get_component::<T>(location)
                .ok_or(QueryComponentError::MissingComponent)
        } else {
            Err(QueryComponentError::MissingReadAccess)
        }
    }

    #[inline]
    pub fn get_component_mut<T: Component>(
        &mut self,
        entity: Entity,
    ) -> Result<CmptMut<T>, QueryComponentError> {
        let world = self.world;
        let location = world
            .entities()
            .get(entity)
            .ok_or(QueryComponentError::NoSuchEntity)?;
        let component_id = world
            .components()
            .get_id(TypeId::of::<T>())
            .ok_or(QueryComponentError::MissingComponent)?;
        if self.state.component_access.has_write(component_id) {
            unsafe {
                world
                    .get_component_unchecked_mut::<T>(location)
                    .map(|(data, ticks)| {
                        CmptMut::new(&mut *data.cast::<T>(), &mut *ticks, self.system_ticks)
                    })
                    .ok_or(QueryComponentError::MissingComponent)
            }
        } else {
            Err(QueryComponentError::MissingWriteAccess)
        }
    }

    #[inline]
    pub fn single(&self) -> ROQueryItem<'_, Q> {
        self.get_single().unwrap()
    }

    #[inline]
    pub fn get_single(&self) -> Result<ROQueryItem<'_, Q>, QuerySingleError> {
        unsafe {
            self.state
                .as_readonly()
                .get_single_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    pub fn single_mut(&mut self) -> Q::Item<'_> {
        self.get_single_mut().unwrap()
    }

    #[inline]
    pub fn get_single_mut(&mut self) -> Result<Q::Item<'_>, QuerySingleError> {
        unsafe {
            self.state
                .get_single_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state.is_empty(self.world)
    }
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> IntoIterator for &'w Query<'_, 's, Q, F> {
    type Item = ROQueryItem<'w, Q>;
    type IntoIter = QueryIter<'w, 's, Q::ReadOnly, F::ReadOnly>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> IntoIterator for &'w mut Query<'_, 's, Q, F> {
    type Item = Q::Item<'w>;
    type IntoIter = QueryIter<'w, 's, Q, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'w, 's, Q: WorldQuery + 'static, F: ReadOnlyWorldQuery + 'static> SystemParam
    for Query<'w, 's, Q, F>
{
    type Fetch = QueryState<Q, F>;
}

// SAFETY: QueryState is constrained to read-only fetches, so it only reads World.
unsafe impl<Q: ReadOnlyWorldQuery, F: ReadOnlyWorldQuery> ReadOnlySystemParamFetch
    for QueryState<Q, F>
{
}

unsafe impl<Q: WorldQuery + 'static, F: ReadOnlyWorldQuery + 'static> SystemParamState
    for QueryState<Q, F>
{
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let state = QueryState::new(world);
        if !system_meta
            .component_access
            .is_compatible(&state.component_access)
        {
            panic!(
                "Query parameters in system {} access components in a way that conflicts with Rust mutability rules.",
                system_meta.name
            );
        }
        system_meta
            .component_access
            .add(state.component_access.clone());
        state
    }

    #[inline]
    fn update(&mut self, world: &World, _system_meta: &mut SystemMeta) {
        self.update_archetypes(world);
    }
}

impl<'w, 's, Q: WorldQuery + 'static, F: ReadOnlyWorldQuery + 'static> SystemParamFetch<'w, 's>
    for QueryState<Q, F>
{
    type Item = Query<'w, 's, Q, F>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        Query::new(
            world,
            state,
            SystemTicks::new(system_meta.last_change_tick, change_tick),
        )
    }
}
