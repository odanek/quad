use super::{
    component::{ComponentId, ResourceId, Tick},
    query::access::{Access, FilteredAccess},
    World,
};

pub mod command;
pub mod event;
pub mod function_system;
pub mod local;
pub mod query;
pub mod removed_components;
pub mod resource;
pub mod system_param;

pub trait System: Send + Sync + 'static {
    type In: 'static;
    type Out: 'static;

    fn name(&self) -> &str;
    fn resource_access(&self) -> &Access<ResourceId>;
    fn component_access(&self) -> &FilteredAccess<ComponentId>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn run(&mut self, input: Self::In, world: &World) -> Self::Out;

    fn apply_buffers(&mut self, world: &mut World);
}

#[derive(Copy, Clone, Debug)]
pub struct SystemTicks {
    pub(crate) last_change_tick: Tick,
    pub(crate) change_tick: Tick,
}

impl SystemTicks {
    pub(crate) fn new(last_change_tick: Tick, change_tick: Tick) -> Self {
        Self {
            last_change_tick,
            change_tick,
        }
    }
}

pub trait IntoSystem<In, Out, Params> {
    type System: System<In = In, Out = Out>;

    fn system(self, world: &mut World) -> Self::System;
}

pub struct AlreadyWasSystem;

impl<In, Out, Sys: System<In = In, Out = Out>> IntoSystem<In, Out, AlreadyWasSystem> for Sys {
    type System = Sys;

    fn system(self, _world: &mut World) -> Sys {
        self
    }
}
