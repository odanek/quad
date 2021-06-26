use super::World;

mod function_system;
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

    fn initialize(&mut self, _world: &mut World);

    unsafe fn run_unsafe(&mut self, input: Self::In, world: &World) -> Self::Out;
    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {
        unsafe { self.run_unsafe(input, world) }
    }

    fn apply_buffers(&mut self, world: &mut World);

    // fn new_archetype(&mut self, archetype: &Archetype);
    // fn component_access(&self) -> &Access<ComponentId>;
    // fn archetype_component_access(&self) -> &Access<ArchetypeComponentId>;
}

pub trait IntoSystem<SystemType: System> {
    fn system(self, id: SystemId) -> SystemType;
}

impl<Sys: System> IntoSystem<Sys> for Sys {
    fn system(self, id: SystemId) -> Sys {
        self
    }
}
