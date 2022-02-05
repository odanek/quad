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
pub struct Systems {
    systems: HashMap<Stage, Vec<BoxedSystem>>,
}

impl Systems {
    pub fn add<S>(&mut self, stage: Stage, system: S)
    where
        S: System<In = (), Out = ()>,
    {
        let vec = self.systems.entry(stage).or_insert_with(Vec::new);
        vec.push(Box::new(system));
    }

    pub fn run(&mut self, stage: Stage, world: &mut World) {
        if let Some(systems) = self.systems.get_mut(&stage) {
            for system in systems {
                unsafe {
                    system.run((), world);
                }
            }
        }
    }
}
