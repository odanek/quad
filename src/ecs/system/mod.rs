use std::sync::atomic::{AtomicUsize, Ordering};

use super::World;

mod function_system;
mod system_param;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemId(pub usize);

static LAST_ID: AtomicUsize = AtomicUsize::new(0);

impl SystemId {
    pub fn new() -> Self {                
        SystemId(LAST_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub trait System: Send + Sync + 'static {
    type In;
    type Out;
    
    fn name(&self) -> &'static str;    
    fn id(&self) -> SystemId;

    // fn new_archetype(&mut self, archetype: &Archetype);
    // fn component_access(&self) -> &Access<ComponentId>;
    // fn archetype_component_access(&self) -> &Access<ArchetypeComponentId>;

    unsafe fn run_unsafe(&mut self, input: Self::In, world: &World) -> Self::Out;
    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {        
        unsafe { self.run_unsafe(input, world) }
    }

    fn apply_buffers(&mut self, world: &mut World);
    fn initialize(&mut self, _world: &mut World);
}

/// A convenience type alias for a boxed [`System`] trait object.
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;