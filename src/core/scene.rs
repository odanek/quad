use crate::ecs::World;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}

// TODO: Remove and use world directly?
pub struct SceneContext<'a> {
    pub world: &'a mut World,
}

impl<'a, 'b> SceneContext<'a> {
    pub fn new(world: &'a mut World) -> SceneContext<'a> {
        SceneContext { world }
    }
}

pub trait Scene {
    fn start(&mut self, _context: SceneContext) {}
    fn stop(&mut self, _context: SceneContext) {}
    fn pause(&mut self, _context: SceneContext) {}
    fn resume(&mut self, _context: SceneContext) {}
    fn update(&mut self, _context: SceneContext) -> SceneResult {
        SceneResult::Ok
    }
}
