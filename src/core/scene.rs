use crate::ecs::World;

// TODO: Use commands instead of mutable world?
pub type SceneTransition = Box<dyn FnOnce(&mut World) -> Box<dyn Scene>>;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(SceneTransition),
    Replace(SceneTransition),
}

pub trait Scene {
    fn update(&mut self, world: &mut World) -> SceneResult;
}
