use std::{
    any::Any,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::ecs::{system::SystemId, IntoSystem, System, World};

pub struct SystemKey<In, Out> {
    id: SystemId,
    marker: PhantomData<dyn System<In = In, Out = Out>>,
}

impl<In, Out> Clone for SystemKey<In, Out> {
    fn clone(&self) -> Self {
        SystemKey {
            id: self.id,
            marker: PhantomData,
        }
    }
}

impl<In, Out> Copy for SystemKey<In, Out> {}

impl<In, Out> SystemKey<In, Out> {
    pub fn new(id: SystemId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }
}

pub struct SystemValue<In, Out>(Box<dyn System<In = In, Out = Out>>);

impl<In, Out> Deref for SystemValue<In, Out> {
    type Target = Box<dyn System<In = In, Out = Out>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<In, Out> DerefMut for SystemValue<In, Out> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Executor {
    system_id: usize,
    systems: HashMap<SystemId, Box<dyn Any>>,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            system_id: 0,
            systems: Default::default(),
        }
    }
}

impl Executor {
    pub fn add<T, In, Out>(
        &mut self,
        world: &mut World,
        system: impl IntoSystem<T>,
    ) -> SystemKey<In, Out>
    where
        T: System<In = In, Out = Out>,
        In: 'static,
        Out: 'static,
    {
        let id = SystemId::new(self.system_id);
        self.system_id += 1;

        let key = SystemKey::new(id);
        let value = SystemValue(Box::new(system.system(id, world)));
        self.systems.insert(id, Box::new(value));
        key
    }

    #[inline]
    pub fn remove<In, Out>(&mut self, key: SystemKey<In, Out>) {
        self.systems.remove(&key.id);
    }

    #[inline]
    pub fn run<Out>(&mut self, world: &mut World, key: SystemKey<(), Out>) -> Out
    where
        Out: 'static,
    {
        self.run_with(world, key, ())
    }

    #[inline]
    pub fn run_with<In, Out>(
        &mut self,
        world: &mut World,
        key: SystemKey<In, Out>,
        param: In,
    ) -> Out
    where
        In: 'static,
        Out: 'static,
    {
        let boxed = self
            .systems
            .get_mut(&key.id)
            .unwrap()
            .downcast_mut::<SystemValue<In, Out>>()
            .unwrap();

        unsafe { boxed.run(param, world) }
    }
}
