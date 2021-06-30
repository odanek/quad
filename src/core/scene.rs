use crate::ecs::{Executor, World};

pub type BoxedScene = Box<dyn Scene>;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(BoxedScene),
    Replace(BoxedScene),
}

pub trait Scene {
    fn on_start(&mut self, _world: &mut World, _executor: &mut Executor) {}
    fn on_stop(&mut self, _world: &mut World, _executor: &mut Executor) {}
    fn on_pause(&mut self, _world: &mut World, _executor: &mut Executor) {}
    fn on_resume(&mut self, _world: &mut World, _executor: &mut Executor) {}
    fn update(&mut self, _world: &mut World, _executor: &mut Executor) -> SceneResult {
        SceneResult::Ok
    }
}
