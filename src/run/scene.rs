use crate::ecs::World;

// TODO What about Pause and Stop?
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneStage {
    Start,
    Stop,
    Pause,
    Resume,
    Update,
}

pub enum SceneResult {
    Ok(SceneStage),
    Quit,
    Pop(SceneStage),
    Push(Box<dyn Scene>, SceneStage),
    Replace(Box<dyn Scene>, SceneStage),
}

pub trait Scene {
    fn update(&mut self, stage: SceneStage, world: &mut World) -> SceneResult;
}
