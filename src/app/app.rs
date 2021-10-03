use crate::{
    ecs::{Event, World},
    window::{WindowBuilder, WindowId},
};

use super::{context::AppContext, event::AppEvents, runner::winit_runner, Scene};

pub type AppEventLoop = winit::event_loop::EventLoop<()>;

pub struct App {
    main_window: WindowBuilder,
    world: Box<World>,
    events: Box<AppEvents>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            main_window: WindowBuilder::default(),
            world: Box::new(World::default()),
            events: Default::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn main_window(mut self, window: WindowBuilder) -> Self {
        self.main_window = window;
        self
    }

    pub fn add_event<T: Event>(mut self) -> Self {
        self.events.add::<T>(&mut self.world);
        self
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window = self.main_window.build(WindowId::new(0), &event_loop);
        let context = AppContext::new(self.world, self.events, scene, main_window);
        winit_runner(context, event_loop);
    }
}
