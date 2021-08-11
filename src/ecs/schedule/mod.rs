use std::marker::PhantomData;

use self::chain_system::EmptyChainBuilder;

use super::{System, World};

mod chain_system;
mod parallel_system;

pub struct Schedule<In = (), Out = ()> {
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

pub trait OptionalSchedule {
    type Out: 'static;

    fn run(&mut self, world: &mut World) -> Self::Out;
    fn run_or(&mut self, default: Self::Out, world: &mut World) -> Self::Out;
}

impl<Out> OptionalSchedule for Option<Schedule<(), Out>>
where
    Out: 'static,
{
    type Out = Out;

    fn run(&mut self, world: &mut World) -> Self::Out {
        self.as_mut().unwrap().run(world)
    }

    fn run_or(&mut self, default: Self::Out, world: &mut World) -> Self::Out {
        self.as_mut()
            .map_or(default, |schedule| schedule.run(world))
    }
}

pub trait OptionalScheduleWithInput {
    type In: 'static;
    type Out: 'static;

    fn run_with(&mut self, input: Self::In, world: &mut World) -> Self::Out;
    fn run_with_or(&mut self, default: Self::Out, input: Self::In, world: &mut World) -> Self::Out;
}

impl<In, Out> OptionalScheduleWithInput for Option<Schedule<In, Out>>
where
    In: 'static,
    Out: 'static,
{
    type In = In;
    type Out = Out;

    fn run_with(&mut self, input: Self::In, world: &mut World) -> Self::Out {
        self.as_mut().unwrap().run_with(input, world)
    }

    fn run_with_or(&mut self, default: Self::Out, input: Self::In, world: &mut World) -> Self::Out {
        self.as_mut()
            .map_or(default, |schedule| schedule.run_with(input, world))
    }
}

pub struct Scheduler {}

impl Scheduler {
    pub fn single<S, In, Out>(system: S) -> Schedule<In, Out>
    where
        S: System<In = In, Out = Out>,
        In: 'static,
        Out: 'static,
    {
        Schedule::new(system)
    }

    pub fn chain() -> EmptyChainBuilder {
        Default::default()
    }

    // pub fn parallel() -> ParallelBuilder {
    //     ParallelBuilder::default()
    // }
}
