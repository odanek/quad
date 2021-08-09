use crate::{ecs::World, input::KeyboardInput};

use super::{scene::SceneContext, Scene, SceneResult};

// TODO: World should not be owned by Context. Remove Context entirely?
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

    pub fn insert_resources(&mut self) {
        self.world.insert_resource(KeyboardInput::default());
    }

    pub fn start_scene(&mut self) {
        self.scene.start(SceneContext::new(&mut self.world));
    }

    pub fn _stop_scene(&mut self) {
        self.scene.stop(SceneContext::new(&mut self.world));
    }

    pub fn _pause_scene(&mut self) {
        self.scene.pause(SceneContext::new(&mut self.world));
    }

    pub fn _resume_scene(&mut self) {
        self.scene.resume(SceneContext::new(&mut self.world));
    }

    pub fn update_scene(&mut self) -> SceneResult {        
        let result = self.scene.update(SceneContext::new(&mut self.world));
        result
    }

    pub fn handle_scene_update(&mut self) -> bool {
        let result = self.update_scene();
        if matches!(result, SceneResult::Quit) {
            return true
        }

        // Physics, animations, ...
        // Draw

        self.world.clear_trackers();
        false
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
