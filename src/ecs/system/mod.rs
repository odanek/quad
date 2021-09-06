use super::{
    component::{ComponentId, ResourceId},
    query::access::{Access, FilteredAccess},
    World,
};

pub mod command;
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

pub trait IntoSystem<In, Out, Params> {
    type System: System<In = In, Out = Out>;

    fn system(self, world: &mut World) -> Self::System;
}
