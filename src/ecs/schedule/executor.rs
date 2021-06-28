use crate::ecs::{IntoSystem, System, system::SystemId};

pub struct Executor {
    system_id: usize,
}

impl Default for Executor {
    fn default() -> Self {
        Self { system_id: 0 }
    }
}

impl Executor {
    pub fn system<T: System>(&mut self, system: impl IntoSystem<T>) -> T {
        self.system_id += 1;
        system.system(SystemId::new(self.system_id))
    }
}
