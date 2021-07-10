use super::{fetch::WorldQuery, filter::FilterFetch};

pub struct QueryState<Q: WorldQuery, F: WorldQuery = ()>
where
    F::Fetch: FilterFetch,
{
    // world_id: WorldId,
    // pub(crate) archetype_generation: ArchetypeGeneration,
    // pub(crate) matched_tables: FixedBitSet,
    // pub(crate) matched_archetypes: FixedBitSet,
    // pub(crate) archetype_component_access: Access<ArchetypeComponentId>,
    // pub(crate) component_access: FilteredAccess<ComponentId>,
    // pub(crate) matched_table_ids: Vec<TableId>,
    // pub(crate) matched_archetype_ids: Vec<ArchetypeId>,
    pub(crate) fetch_state: Q::State,
    pub(crate) filter_state: F::State,
}
