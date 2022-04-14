use crate::ecs::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn update(&mut self, _world: &mut World) -> SceneResult;
}
