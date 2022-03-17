use std::collections::HashMap;

use crate::ecs::{System, World};

type BoxedSystem = Box<dyn System<In = (), Out = ()>>;

// TODO: Fix mixing of Main app and Render app
#[derive(PartialEq, Eq, Hash)]
pub enum Stage {
    // Main App
    LoadAssets,
    PreUpdate,
    PostUpdate,
    AssetEvents,
    Flush,

    // Render App
    RenderExtract,
    RenderPrepare,
    RenderQueue,
    RenderPhaseSort,
    RenderRender,
    RenderCleanup,
}

#[derive(Default)]
pub struct SequentialSystems(Vec<BoxedSystem>);

impl SequentialSystems {
    pub fn add<S>(&mut self, system: S)
    where
        S: System<In = (), Out = ()>,
    {
        self.0.push(Box::new(system));
    }

    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.0 {
            unsafe {
                system.run((), world);
            }
        }
    }

    pub fn apply_buffers(&mut self, world: &mut World) {
        for system in &mut self.0 {
            system.apply_buffers(world);
        }
    }
}

#[derive(Default)]
pub struct Systems {
    systems: HashMap<Stage, SequentialSystems>,
}

impl Systems {
    pub fn add<S>(&mut self, stage: Stage, system: S)
    where
        S: System<In = (), Out = ()>,
    {
        self.systems.entry(stage).or_default().add(system);
    }

    pub fn get(&mut self, stage: Stage) -> Option<&mut SequentialSystems> {
        self.systems.get_mut(&stage)
    }

    pub fn run(&mut self, stage: Stage, world: &mut World) {
        if let Some(systems) = self.systems.get_mut(&stage) {
            systems.run(world);
            systems.apply_buffers(world);
        }
    }
}
