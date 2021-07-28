use std::marker::PhantomData;

use self::chain_system::EmptyChainBuilder;

use super::{System, World};

mod chain_system;
mod parallel_system;

pub struct Schedule<In, Out> {
    system: Box<dyn System<In = In, Out = Out>>,
    marker: PhantomData<(In, Out)>,
}

impl<In, Out> Schedule<In, Out>
where
    In: 'static,
    Out: 'static,
{
    pub fn new<S>(system: S) -> Self
    where
        S: System<In = In, Out = Out>,
    {
        Self {
            system: Box::new(system),
            marker: PhantomData,
        }
    }

    pub fn run_with(&mut self, input: In, world: &mut World) -> Out {
        let result = unsafe { self.system.run(input, world) };
        self.system.apply_buffers(world);
        result
    }
}

impl<Out> Schedule<(), Out>
where
    Out: 'static,
{
    pub fn run(&mut self, world: &mut World) -> Out {
        let result = unsafe { self.system.run((), world) };
        self.system.apply_buffers(world);
        result
    }
}

pub struct Scheduler {}

impl Scheduler {
    // pub fn parallel() -> ParallelBuilder {
    //     ParallelBuilder::default()
    // }

    pub fn chain() -> EmptyChainBuilder {
        Default::default()
    }
}
