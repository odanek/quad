use crate::{
    ecs::{Event, World},
    input::MouseButtonInput,
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
        let app = Self {
            main_window: WindowBuilder::default(),
            world: Box::new(World::default()),
            events: Default::default(),
        };
        app.add_default_events()
    }
}

impl AppBuilder {
    pub(crate) fn add_default_events(self) -> Self {
        self.add_event::<MouseButtonInput>()
    }

    pub fn main_window(mut self, window: WindowBuilder) -> Self {
        self.main_window = window;
        self
    }

    pub fn add_event<T: Event>(mut self) -> Self {
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
