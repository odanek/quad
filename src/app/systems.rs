use std::collections::HashMap;

use crate::ecs::{System, World};

use super::stage::{StageId, StageLabel};

struct SystemEntry {
    pub initialized: bool,
    pub system: Box<dyn System<In = (), Out = ()>>,
}

#[derive(Default)]
pub struct StageSystems(Vec<SystemEntry>);

impl StageSystems {
    pub fn add<S>(&mut self, system: S)
    where
        S: System<In = (), Out = ()>,
    {
        self.0.push(SystemEntry {
            initialized: false,
            system: Box::new(system),
        });
    }

    pub fn run(&mut self, world: &mut World) {
        for entry in &mut self.0 {
            if !entry.initialized {
                entry.initialized = true;
                entry.system.initialize(world);
            }            
            unsafe {                
                entry.system.run((), world);
            }
        }
    }

    pub fn apply_buffers(&mut self, world: &mut World) {
        for entry in &mut self.0 {
            entry.system.apply_buffers(world);
        }
    }
}

#[derive(Default)]
pub struct Systems {
    systems: HashMap<StageId, StageSystems>,
}

impl Systems {
    pub fn add<L, S>(&mut self, stage: L, system: S)
    where
        L: StageLabel,
        S: System<In = (), Out = ()>,
    {
        self.systems.entry(stage.id()).or_default().add(system);
    }

    pub fn get<L>(&mut self, stage: L) -> Option<&mut StageSystems>
    where
        L: StageLabel,
    {
        self.systems.get_mut(&stage.id())
    }

    pub fn run<L>(&mut self, stage: L, world: &mut World)
    where
        L: StageLabel,
    {
        if let Some(systems) = self.systems.get_mut(&stage.id()) {
            systems.run(world);
            systems.apply_buffers(world);
        }
    }
}
