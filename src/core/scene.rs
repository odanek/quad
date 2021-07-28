use crate::ecs::World;

pub type BoxedScene = Box<dyn Scene>;

pub enum SceneResult {
    Ok,
    Quit,
    Pop,
    Push(BoxedScene),
    Replace(BoxedScene),
}

pub struct SceneContext<'a> {
    pub world: &'a mut World,
}

impl<'a, 'b> SceneContext<'a> {
    pub fn new(world: &'a mut World) -> SceneContext<'a> {
        SceneContext { world }
    }
}

pub trait Scene {
    fn on_start(&mut self, _context: SceneContext) {}
    fn on_stop(&mut self, _context: SceneContext) {}
    fn on_pause(&mut self, _context: SceneContext) {}
    fn on_resume(&mut self, _context: SceneContext) {}
    fn update(&mut self, _context: SceneContext) -> SceneResult {
        SceneResult::Ok
    }
}
