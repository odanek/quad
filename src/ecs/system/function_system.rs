use super::SystemId;

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