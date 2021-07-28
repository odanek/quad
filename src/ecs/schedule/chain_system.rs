use crate::ecs::{
    component::ComponentId,
    query::access::{Access, FilteredAccess},
    resource::ResourceId,
    system::function_system::SystemMeta,
    System, World,
};

use super::Schedule;

pub trait NoSystem {}

impl NoSystem for () {}

pub struct ChainSystem<S, T> {
    meta: SystemMeta,
    left: S,
    right: T,
}

impl<S, T> ChainSystem<S, T>
where
    S: System,
    T: System,
{
    pub fn new(left: S, right: T) -> Self {
        let name = format!("Chain: {} -> {}", left.name(), right.name());

        let mut meta = SystemMeta::new(name);
        meta.resource_access.extend(left.resource_access());
        meta.resource_access.extend(right.resource_access());
        meta.component_access.extend(left.component_access());
        meta.component_access.extend(right.component_access());

        Self { meta, left, right }
    }
}

impl<S, T> System for ChainSystem<S, T>
where
    S: System,
    T: System<In = S::Out>,
{
    type In = S::In;

    type Out = T::Out;

    fn name(&self) -> &str {
        &self.meta.name
    }

    fn resource_access(&self) -> &Access<ResourceId> {
        &self.meta.resource_access
    }

    fn component_access(&self) -> &FilteredAccess<ComponentId> {
        &self.meta.component_access
    }

    unsafe fn run(&mut self, input: Self::In, world: &World) -> Self::Out {
        self.right.run(self.left.run(input, world), world)
    }

    fn apply_buffers(&mut self, world: &mut crate::ecs::World) {
        self.left.apply_buffers(world);
        self.right.apply_buffers(world);
    }
}

pub struct EmptyChainBuilder {}

impl Default for EmptyChainBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl EmptyChainBuilder {
    pub fn add<S>(self, system: S) -> ChainBuilder<S>
    where
        S: System,
    {
        ChainBuilder { system }
    }
}

pub struct ChainBuilder<T> {
    system: T,
}

impl<T> ChainBuilder<T>
where
    T: System,
{
    pub fn add<S>(self, system: S) -> ChainBuilder<ChainSystem<T, S>>
    where
        S: System<In = T::Out>,
    {
        ChainBuilder {
            system: ChainSystem::new(self.system, system),
        }
    }

    pub fn system(self) -> T {
        self.system
    }

    pub fn build(self) -> Schedule<T::In, T::Out> {
        Schedule::new(self.system)
    }
}
