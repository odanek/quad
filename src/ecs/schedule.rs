use std::marker::PhantomData;

use self::chain_system::EmptyChainBuilder;

use super::{IntoSystem, System, World};

mod chain_system;
mod parallel_system;

pub struct Schedule<In = (), Out = ()> {
    system: Box<dyn System<In = In, Out = Out>>,
    initialized: bool,
    marker: PhantomData<fn(In) -> Out>,
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
            initialized: false,
            marker: PhantomData,
        }
    }

    // TODO Remove
    pub fn run_with(&mut self, input: In, world: &mut World) -> Out {
        if !self.initialized {
            self.initialized = true;
            self.system.initialize(world);
        }
        let result = unsafe { self.system.run(input, world) };
        self.system.apply_buffers(world);
        result
    }
}

impl<Out> Schedule<(), Out>
where
    Out: 'static,
{
    // TODO Remove
    pub fn run(&mut self, world: &mut World) -> Out {
        if !self.initialized {
            self.initialized = true;
            self.system.initialize(world);
        }
        let result = unsafe { self.system.run((), world) };
        self.system.apply_buffers(world);
        result
    }
}

pub struct Scheduler {}

impl Scheduler {
    pub fn single<S, In, Out, Param>(system: S) -> Schedule<In, Out>
    where
        S: IntoSystem<In, Out, Param>,
        In: 'static,
        Out: 'static,
    {
        Schedule::new(system.system())
    }

    pub fn chain(world: &mut World) -> EmptyChainBuilder {
        EmptyChainBuilder::new(world)
    }

    // pub fn parallel() -> ParallelBuilder {
    //     ParallelBuilder::default()
    // }
}
