use crate::{ecs::World, input::KeyboardInput};

use super::{
    scene::{BoxedScene, SceneContext},
    SceneResult,
};

pub struct Context {
    pub world: Box<World>,
    pub scene: BoxedScene,
}

impl Context {
    pub fn new(scene: BoxedScene) -> Self {
        Context {
            world: Box::new(Default::default()),
            scene,
        }
    }

    pub fn register_resources(&mut self) {
        self.world.add_resource(KeyboardInput::default());
    }

    pub fn start_scene(&mut self) {
        self.scene.on_start(SceneContext::new(&mut self.world));
    }

    pub fn _stop_scene(&mut self) {
        self.scene.on_stop(SceneContext::new(&mut self.world));
    }

    pub fn _pause_scene(&mut self) {
        self.scene.on_pause(SceneContext::new(&mut self.world));
    }

    pub fn _resume_scene(&mut self) {
        self.scene.on_resume(SceneContext::new(&mut self.world));
    }

    pub fn update_scene(&mut self) -> SceneResult {
        let result = self.scene.update(SceneContext::new(&mut self.world));
        result
    }

    pub fn handle_scene_update(&mut self) -> bool {
        // Update
        // Draw
        let result = self.update_scene();
        matches!(result, SceneResult::Quit)
    }

    pub fn handle_window_resize(&mut self, _size: winit::dpi::PhysicalSize<u32>) {
        // if size.width != 0 || size.height != 0 {
        //     // Resized
        // } else {
        //     // Minimized
        // }
    }

    pub fn handle_keyboard_event(&mut self, input: winit::event::KeyboardInput) {
        use winit::event::ElementState;

        if let Some(keycode) = input.virtual_keycode {
            let keyboard_input = self.world.resource_mut::<KeyboardInput>();
            match input.state {
                ElementState::Pressed => keyboard_input.press(keycode.into()),
                ElementState::Released => keyboard_input.release(keycode.into()),
            }
        }
    }

    pub fn flush_keyboard_events(&mut self) {
        let keyboard_input = self.world.resource_mut::<KeyboardInput>();
        keyboard_input.flush();
    }
}
