use super::{
    component::ComponentId,
    query::access::{Access, FilteredAccess},
    resource::ResourceId,
    World,
};

pub mod command;
pub mod function_system;
pub mod param;

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
