use crate::ecs::{Schedule, World};

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScenePhase {
    Start,
    // Stop,
    // Pause,
    // Resume,
    Update,
}

#[derive(Default)]
pub struct SceneSchedule {
    pub start: Option<Schedule>,
    pub stop: Option<Schedule>,
    pub pause: Option<Schedule>,
    pub resume: Option<Schedule>,
    pub update: Option<Schedule<(), SceneResult>>,
}

pub trait Scene {
    fn run(&mut self, _world: &mut World) -> SceneSchedule;
}
