use crate::ecs::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn on_start(&mut self, _world: &mut World) {}
    fn on_stop(&mut self, _world: &mut World) {}
    fn on_pause(&mut self, _world: &mut World) {}
    fn on_resume(&mut self, _world: &mut World) {}
    fn update(&mut self, _world: &mut World) -> SceneResult {
        SceneResult::Ok
    }
}
