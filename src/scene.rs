use super::world::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn begin(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World) -> SceneResult;
    fn end(&mut self);
}
