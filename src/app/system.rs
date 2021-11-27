use std::collections::HashMap;

use crate::{
    asset::Asset,
    ecs::{Event, Events, IntoSystem, System, World},
};

type BoxedSystem = Box<dyn System<In = (), Out = ()>>;

#[derive(PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    PostUpdate,
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

    pub fn add_event<T: Event>(&mut self, world: &mut World) {
        world.insert_resource(Events::<T>::default());
        self.add(Stage::PreUpdate, Events::<T>::update_system.system(world));
    }

    pub fn add_asset<T: Asset>(&mut self, world: &mut World) {
        let assets = {
            // let asset_server = self.world.get_resource::<AssetServer>().unwrap();
            // asset_server.register_asset_type::<T>()
        };

        // world.insert_resource(assets);

        // .add_system_to_stage(
        //     AssetStage::AssetEvents,
        //     Assets::<T>::asset_event_system.system(),
        // )
        // .add_system_to_stage(
        //     AssetStage::LoadAssets,
        //     update_asset_storage_system::<T>.system(),
        // )

        // self.add_event::<AssetEvent<T>>(world);
    }
}
