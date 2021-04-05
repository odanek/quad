use crate::ecs::{schedule::Schedule, world::World};

use super::builder::AppBuilder;

pub struct App {
    pub world: World,
    pub runner: Box<dyn Fn(App)>,
    pub schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        Self {
            world: Default::default(),
            schedule: Default::default(),
            runner: Box::new(run_once),
        }
    }
}

fn run_once(mut app: App) {
    app.update();
}

impl App {
    pub fn build() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }

    pub fn run(mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(run_once));
        (runner)(self);
    }
}