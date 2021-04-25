use crate::{
    ecs::World,
    input::KeyboardInput,
    scene::{Scene, SceneResult},
};

pub struct Context {
    pub world: Box<World>,
    pub scene: Box<dyn Scene>,
}

impl Context {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        Context {
            world: Box::new(Default::default()),
            scene,
        }
    }

    pub fn register_resources(&mut self) {
        self.world.add_resource(Box::new(KeyboardInput::default()))
    }

    pub fn start_scene(&mut self) {
        self.scene.begin(&mut self.world);
    }

    pub fn update_scene(&mut self) -> SceneResult {
        self.scene.update(&mut self.world)
    }

    pub fn end_scene(&mut self) {
        self.scene.end();
    }
}
