use super::{component::ComponentId, query::access::Access, resource::ResourceId, World};

mod function_system;
pub mod local_param;
pub mod resource_param;
mod system_param;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemId(pub usize);

impl SystemId {
    pub fn new(id: usize) -> Self {
        SystemId(id)
    }
}

pub trait System: Send + Sync + 'static {
    type In;
    type Out;

    fn name(&self) -> &'static str;
    fn id(&self) -> SystemId;
    fn resource_access(&self) -> &Access<ResourceId>;
    fn component_access(&self) -> &Access<ComponentId>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn run(&mut self, input: Self::In, world: &World) -> Self::Out;

    fn apply_buffers(&mut self, world: &mut World);

    // fn new_archetype(&mut self, archetype: &Archetype);

    // fn archetype_component_access(&self) -> &Access<ArchetypeComponentId>;
}

pub trait IntoSystem<SystemType: System> {
    fn system(self, id: SystemId, world: &mut World) -> SystemType;
}
