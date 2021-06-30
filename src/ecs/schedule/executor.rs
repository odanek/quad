use crate::ecs::{system::SystemId, BoxedSystem, IntoSystem, System, World};

pub struct Executor {
    system_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        Self { system_id: 0 }
    }
}

impl Executor {
    pub fn system<T, In, Out>(
        &mut self,
        world: &mut World,
        system: impl IntoSystem<T>,
    ) -> BoxedSystem<In, Out>
    where
        T: System<In = In, Out = Out>,
    {
        let id = SystemId::new(self.system_id);
        self.system_id += 1;
        let mut result = Box::new(system.system(id));
        result.initialize(world);
        result
    }

    #[inline]
    pub fn run<Out: 'static>(
        &mut self,
        world: &mut World,
        system: &mut BoxedSystem<(), Out>,
    ) -> Out {
        unsafe { system.run((), world) }
    }

    #[inline]
    pub fn run_with<In: 'static, Out: 'static>(
        &mut self,
        world: &mut World,
        system: &mut BoxedSystem<In, Out>,
        param: In,
    ) -> Out {
        unsafe { system.run(param, world) }
    }
}
