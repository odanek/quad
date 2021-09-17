use crate::ecs::{
    component::{ComponentId, ResourceId},
    query::access::{Access, FilteredAccess},
    system::function_system::SystemMeta,
    IntoSystem, System, World,
};

use super::Schedule;

pub struct ChainSystem<S, T> {
    meta: SystemMeta,
    left: S,
    right: T,
}

impl<S, T> ChainSystem<S, T>
where
    S: System,
    T: System<In = S::Out>,
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

pub struct EmptyChainBuilder<'w> {
    world: &'w mut World,
}

impl<'w> EmptyChainBuilder<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self { world }
    }
}

impl<'w> EmptyChainBuilder<'w> {
    pub fn add<In, Out, Param, S>(self, input: S) -> ChainBuilder<'w, S::System>
    where
        S: IntoSystem<In, Out, Param>,
    {
        let system = input.system(self.world);
        ChainBuilder {
            world: self.world,
            system,
        }
    }
}

pub struct ChainBuilder<'w, T> {
    world: &'w mut World,
    system: T,
}

impl<'w, T> ChainBuilder<'w, T>
where
    T: System,
{
    pub fn add<Out, Param, S>(self, input: S) -> ChainBuilder<'w, ChainSystem<T, S::System>>
    where
        S: IntoSystem<T::Out, Out, Param>,
    {
        let system = input.system(self.world);
        ChainBuilder {
            world: self.world,
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
