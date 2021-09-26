use crate::{
    ecs::{Event, World},
    window::WindowBuilder,
};

use super::{event::AppEvents, App};

pub struct AppBuilder {
    main_window: WindowBuilder,
    world: Box<World>,
    events: AppEvents,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            main_window: WindowBuilder::default(),
            world: Box::new(World::default()),
            events: Default::default(),
        }
    }
}

impl AppBuilder {
    pub fn main_window(mut self, window: WindowBuilder) -> AppBuilder {
        self.main_window = window;
        self
    }

    pub fn add_event<T: Event>(mut self) -> AppBuilder {
        self.events.add::<T>(&mut self.world);
        self
    }

    pub fn build(self) -> App {
        App {
            main_window: self.main_window.build(),
            world: self.world,
            events: self.events,
        }
    }
}
