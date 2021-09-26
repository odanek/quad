use std::collections::HashMap;

use crate::ecs::{Event, Events, IntoSystem, ResourceId, System, World};

#[derive(Default)]
pub struct AppEvents {
    map: HashMap<ResourceId, Box<dyn System<In = (), Out = ()>>>,
}

impl AppEvents {
    pub fn add<T: Event>(&mut self, world: &mut World) {
        let id = world.register_resource::<Events<T>>();

        self.map.entry(id).or_insert_with(|| {
            world.insert_resource(Events::<T>::default());
            Box::new(Events::<T>::update_system.system(world))
        });
    }

    pub fn update(&mut self, world: &mut World) {
        for system in self.map.values_mut() {
            unsafe {
                system.run((), world);
            }
        }
    }
}
