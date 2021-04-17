use super::world::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn update(&self, world: &mut World) -> SceneResult;
}
