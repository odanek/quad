use crate::ecs::{component::ComponentId, entity::archetype::ArchetypeId, storage::TableId, World};

use super::{
    access::FilteredAccess,
    fetch::{FetchState, WorldQuery},
    filter::FilterFetch,
};

pub struct QueryState<Q: WorldQuery, F: WorldQuery /* = ()*/>
where
    F::Fetch: FilterFetch,
{
    // pub(crate) archetype_generation: ArchetypeGeneration,
    pub(crate) matched_tables: Vec<TableId>,
    pub(crate) matched_archetypes: Vec<ArchetypeId>,
    // pub(crate) archetype_component_access: Access<ArchetypeComponentId>,
    pub(crate) component_access: FilteredAccess<ComponentId>,
    // pub(crate) matched_table_ids: Vec<TableId>,
    // pub(crate) matched_archetype_ids: Vec<ArchetypeId>,
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

        // Use a temporary empty FilteredAccess for filters. This prevents them from conflicting with the
        // main Query's `fetch_state` access. Filters are allowed to conflict with the main query fetch
        // because they are evaluated *before* a specific reference is constructed.
        let mut filter_component_access = FilteredAccess::default();
        filter_state.update_component_access(&mut filter_component_access);

        // Merge the temporary filter access with the main access. This ensures that filter access is
        // properly considered in a global "cross-query" context (both within systems and across systems).
        component_access.extend(&filter_component_access);

        Self {
            // world_id: world.id(),
            // archetype_generation: ArchetypeGeneration::initial(),
            // matched_table_ids: Vec::new(),
            // matched_archetype_ids: Vec::new(),
            fetch_state,
            filter_state,
            component_access,
            matched_tables: Default::default(),
            matched_archetypes: Default::default(),
            // archetype_component_access: Default::default(),
        }
        // state.validate_world_and_update_archetypes(world);
    }
}
