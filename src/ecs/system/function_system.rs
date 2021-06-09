use crate::ecs::{World, archetype::{ArchetypeGeneration, ArchetypeId}, world::WorldId};

use super::{SystemId, system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState}};

pub struct SystemMeta {
    pub(crate) id: SystemId,
    pub(crate) name: &'static str,
    // pub(crate) component_access_set: FilteredAccessSet<ComponentId>,
    // pub(crate) archetype_component_access: Access<ArchetypeComponentId>,
}

impl SystemMeta {
    fn new<T>() -> Self {
        Self {
            id: SystemId::new(),
            name: std::any::type_name::<T>(),
            // archetype_component_access: Access::default(),
            // component_access_set: FilteredAccessSet::default(),
        }
    }
}

pub struct SystemState<Param: SystemParam> {
    meta: SystemMeta,
    param_state: <Param as SystemParam>::Fetch,
    world_id: WorldId,
    archetype_generation: ArchetypeGeneration,
}

impl<Param: SystemParam> SystemState<Param> {
    pub fn new(world: &mut World) -> Self {
        let config = <Param::Fetch as SystemParamState>::default_config();
        Self::with_config(world, config)
    }

    pub fn with_config(
        world: &mut World,
        config: <Param::Fetch as SystemParamState>::Config,
    ) -> Self {
        let mut meta = SystemMeta::new::<Param>();
        let param_state = <Param::Fetch as SystemParamState>::init(world, &mut meta, config);
        Self {
            meta,
            param_state,
            world_id: world.id(),
            archetype_generation: ArchetypeGeneration::initial(),
        }
    }

    #[inline]
    pub fn meta(&self) -> &SystemMeta {
        &self.meta
    }

    /// Retrieve the [`SystemParam`] values. This can only be called when all parameters are read-only.
    #[inline]
    pub fn get<'a>(&'a mut self, world: &'a World) -> <Param::Fetch as SystemParamFetch<'a>>::Item
    where
        Param::Fetch: ReadOnlySystemParamFetch,
    {
        self.validate_world_and_update_archetypes(world);
        unsafe { self.get_unchecked_manual(world) }
    }

    #[inline]
    pub fn get_mut<'a>(
        &'a mut self,
        world: &'a mut World,
    ) -> <Param::Fetch as SystemParamFetch<'a>>::Item {
        self.validate_world_and_update_archetypes(world);
        unsafe { self.get_unchecked_manual(world) }
    }

    pub fn apply(&mut self, world: &mut World) {
        self.param_state.apply(world);
    }

    #[inline]
    pub fn matches_world(&self, world: &World) -> bool {
        self.world_id == world.id()
    }

    fn validate_world_and_update_archetypes(&mut self, world: &World) {
        assert!(self.matches_world(world), "Encountered a mismatched World. A SystemState cannot be used with Worlds other than the one it was created with.");
        let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

        for archetype_index in archetype_index_range {
            self.param_state.new_archetype(
                &archetypes[ArchetypeId::new(archetype_index)],
                &mut self.meta,
            );
        }
    }

    #[inline]
    pub unsafe fn get_unchecked_manual<'a>(
        &'a mut self,
        world: &'a World,
    ) -> <Param::Fetch as SystemParamFetch<'a>>::Item {
        let param = <Param::Fetch as SystemParamFetch>::get_param(
            &mut self.param_state,
            &self.meta,
            world,
        );
        param
    }
}

pub trait IntoSystem<Params, SystemType: System> {
    fn system(self) -> SystemType;
}

impl<Sys: System> IntoSystem<(), Sys> for Sys {
    fn system(self) -> Sys {
        self
    }
}
