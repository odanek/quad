use std::any::TypeId;

use crate::ecs::{
    component::{CmptMut, Component, Tick},
    query::{
        fetch::{Fetch, ReadOnlyFetch, WorldQuery},
        filter::FilterFetch,
        iter::QueryIter,
        state::{QueryEntityError, QueryState},
    },
    system::function_system::SystemMeta,
    Entity, World,
};

use super::{
    system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState},
    SystemTicks,
};

#[derive(Debug)]
pub enum QueryComponentError {
    MissingReadAccess,
    MissingWriteAccess,
    MissingComponent,
    NoSuchEntity,
}

#[derive(Debug)]
pub enum QuerySingleError {
    NoEntities,
    MultipleEntities,
}
// TODO: This definition allows to have With, Without, Or in Q, while it should be possible only in F
pub struct Query<'w, 's, Q: WorldQuery, F: WorldQuery = ()>
where
    F::Fetch: FilterFetch,
{
    world: &'w World,
    state: &'s QueryState<Q, F>,
    system_ticks: SystemTicks,
}

impl<'w, 's, Q: WorldQuery, F: WorldQuery> Query<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
{
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
    pub fn iter(&self) -> QueryIter<'w, 's, Q, Q::ReadOnlyFetch, F>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe {
            self.state
                .iter_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> QueryIter<'w, 's, Q, Q::Fetch, F> {
        unsafe {
            self.state
                .iter_unchecked_manual(self.world, self.system_ticks)
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn iter_unsafe(&self) -> QueryIter<'_, '_, Q, Q::Fetch, F> {
        self.state
            .iter_unchecked_manual(self.world, self.system_ticks)
    }

    #[inline]
    pub fn for_each(&self, f: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item))
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe {
            self.state
                .for_each_unchecked_manual(self.world, f, self.system_ticks)
        };
    }

    #[inline]
    pub fn for_each_mut(&mut self, f: impl FnMut(<Q::Fetch as Fetch<'w, 's>>::Item)) {
        unsafe {
            self.state
                .for_each_unchecked_manual(self.world, f, self.system_ticks)
        };
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Result<<Q::Fetch as Fetch>::Item, QueryEntityError>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        unsafe {
            self.state
                .get_unchecked_manual(self.world, entity, self.system_ticks)
        }
    }

    #[inline]
    pub fn get_mut(
        &mut self,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch>::Item, QueryEntityError> {
        unsafe {
            self.state
                .get_unchecked_manual(self.world, entity, self.system_ticks)
        }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked(
        &self,
        entity: Entity,
    ) -> Result<<Q::Fetch as Fetch>::Item, QueryEntityError> {
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

    pub fn single(&self) -> Result<<Q::ReadOnlyFetch as Fetch<'w, 's>>::Item, QuerySingleError>
    where
        Q::Fetch: ReadOnlyFetch,
    {
        let mut query = self.iter();
        let first = query.next();
        let extra = query.next().is_some();

        match (first, extra) {
            (Some(r), false) => Ok(r),
            (None, _) => Err(QuerySingleError::NoEntities),
            (Some(_), _) => Err(QuerySingleError::MultipleEntities),
        }
    }

    pub fn single_mut(&mut self) -> Result<<Q::Fetch as Fetch<'w, 's>>::Item, QuerySingleError> {
        let mut query = self.iter_mut();
        let first = query.next();
        let extra = query.next().is_some();

        match (first, extra) {
            (Some(r), false) => Ok(r),
            (None, _) => Err(QuerySingleError::NoEntities),
            (Some(_), _) => Err(QuerySingleError::MultipleEntities),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state.is_empty(self.world)
    }
}

impl<'w, 's, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParam for Query<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
{
    type Fetch = QueryState<Q, F>;
}

unsafe impl<Q: WorldQuery, F: WorldQuery> ReadOnlySystemParamFetch for QueryState<Q, F>
where
    Q::Fetch: ReadOnlyFetch,
    F::Fetch: FilterFetch,
{
}

impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamState for QueryState<Q, F>
where
    F::Fetch: FilterFetch,
{
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let state = QueryState::new(world);
        if !system_meta
            .component_access
            .is_compatible(&state.component_access)
        {
            panic!("Query parameters in system {} access components in a way that conflicts with Rust mutability rules.", system_meta.name);
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

impl<'w, 's, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamFetch<'w, 's>
    for QueryState<Q, F>
where
    F::Fetch: FilterFetch,
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

pub struct QuerySet<'w, 's, T> {
    query_states: &'s T,
    world: &'w World,
    system_ticks: SystemTicks,
}
pub struct QuerySetState<T>(T);

impl<
        'w,
        's,
        Q0: WorldQuery + 'static,
        Q1: WorldQuery + 'static,
        F0: WorldQuery + 'static,
        F1: WorldQuery + 'static,
    > SystemParam for QuerySet<'w, 's, (QueryState<Q0, F0>, QueryState<Q1, F1>)>
where
    F0::Fetch: FilterFetch,
    F1::Fetch: FilterFetch,
{
    type Fetch = QuerySetState<(QueryState<Q0, F0>, QueryState<Q1, F1>)>;
}

unsafe impl<
        Q0: WorldQuery + 'static,
        Q1: WorldQuery + 'static,
        F0: WorldQuery + 'static,
        F1: WorldQuery + 'static,
    > ReadOnlySystemParamFetch for QuerySetState<(QueryState<Q0, F0>, QueryState<Q1, F1>)>
where
    Q0::Fetch: ReadOnlyFetch,
    Q1::Fetch: ReadOnlyFetch,
    F0::Fetch: FilterFetch,
    F1::Fetch: FilterFetch,
{
}

impl<
        Q0: WorldQuery + 'static,
        Q1: WorldQuery + 'static,
        F0: WorldQuery + 'static,
        F1: WorldQuery + 'static,
    > SystemParamState for QuerySetState<(QueryState<Q0, F0>, QueryState<Q1, F1>)>
where
    F0::Fetch: FilterFetch,
    F1::Fetch: FilterFetch,
{
    fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
        let q0 = QueryState::<Q0, F0>::new(world);
        if !system_meta
            .component_access
            .is_compatible(&q0.component_access)
        {
            panic!("Query parameters in system {} access components in a way that conflicts with Rust mutability rules.", system_meta.name);
        }

        let q1 = QueryState::<Q1, F1>::new(world);
        if !system_meta
            .component_access
            .is_compatible(&q1.component_access)
        {
            panic!("Query parameters in system {} access components in a way that conflicts with Rust mutability rules.", system_meta.name);
        }

        system_meta
            .component_access
            .add(q0.component_access.clone());
        system_meta
            .component_access
            .add(q1.component_access.clone());
        QuerySetState((q0, q1))
    }

    #[inline]
    fn update(&mut self, world: &World, _system_meta: &mut SystemMeta) {
        self.0 .0.update_archetypes(world);
        self.0 .1.update_archetypes(world);
    }
}

impl<
        'w,
        's,
        Q0: WorldQuery + 'static,
        Q1: WorldQuery + 'static,
        F0: WorldQuery + 'static,
        F1: WorldQuery + 'static,
    > SystemParamFetch<'w, 's> for QuerySetState<(QueryState<Q0, F0>, QueryState<Q1, F1>)>
where
    F0::Fetch: FilterFetch,
    F1::Fetch: FilterFetch,
{
    type Item = QuerySet<'w, 's, (QueryState<Q0, F0>, QueryState<Q1, F1>)>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        system_meta: &SystemMeta,
        world: &'w World,
        change_tick: Tick,
    ) -> Self::Item {
        QuerySet {
            query_states: &state.0,
            world,
            system_ticks: SystemTicks::new(system_meta.last_change_tick, change_tick),
        }
    }
}
impl<'w, 's, Q0: WorldQuery, Q1: WorldQuery, F0: WorldQuery, F1: WorldQuery>
    QuerySet<'w, 's, (QueryState<Q0, F0>, QueryState<Q1, F1>)>
where
    F0::Fetch: FilterFetch,
    F1::Fetch: FilterFetch,
{
    pub fn q0(&mut self) -> Query<'_, '_, Q0, F0> {
        unsafe { Query::new(self.world, &self.query_states.0, self.system_ticks) }
    }
    pub fn q1(&mut self) -> Query<'_, '_, Q1, F1> {
        unsafe { Query::new(self.world, &self.query_states.1, self.system_ticks) }
    }
}
