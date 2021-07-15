use crate::ecs::{
    query::{fetch::WorldQuery, filter::FilterFetch, state::QueryState},
    World,
};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
};

pub struct Query<'w, Q: WorldQuery, F: WorldQuery /* = ()*/>
where
    F::Fetch: FilterFetch,
{
    pub(crate) world: &'w World,
    pub(crate) state: &'w QueryState<Q, F>,
}

impl<'w, Q: WorldQuery, F: WorldQuery> Query<'w, Q, F>
where
    F::Fetch: FilterFetch,
{
    #[inline]
    pub(crate) unsafe fn new(world: &'w World, state: &'w QueryState<Q, F>) -> Self {
        Self { world, state }
    }
}

impl<'a, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParam for Query<'a, Q, F>
where
    F::Fetch: FilterFetch,
{
    type Fetch = QueryState<Q, F>;
}

// SAFE: Relevant query ComponentId and ArchetypeComponentId access is applied to SystemMeta. If
// this QueryState conflicts with any prior access, a panic will occur.
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
        system_meta.component_access.extend(&state.component_access);
        // system_meta
        //     .archetype_component_access
        //     .extend(&state.archetype_component_access);
        state
    }

    // fn new_archetype(&mut self, archetype: &Archetype, system_meta: &mut SystemMeta) {
    //     self.new_archetype(archetype);
    //     system_meta
    //         .archetype_component_access
    //         .extend(&self.archetype_component_access);
    // }
}

impl<'a, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamFetch<'a> for QueryState<Q, F>
where
    F::Fetch: FilterFetch,
{
    type Item = Query<'a, Q, F>;

    #[inline]
    unsafe fn get_param(
        state: &'a mut Self,
        _system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        Query::new(world, state)
    }
}
