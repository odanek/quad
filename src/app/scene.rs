use crate::ecs::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn start(&mut self, _world: &mut World) {}
    fn stop(&mut self, _world: &mut World) {}
    fn pause(&mut self, _world: &mut World) {}
    fn resume(&mut self, _world: &mut World) {}
    fn update(&mut self, _world: &mut World) -> SceneResult {
        SceneResult::Ok
    }
}
