use crate::{ecs::World, input::KeyboardInput};

use super::{Scene, SceneResult, scene::SceneTransition};

pub struct Context {
    pub world: Box<World>,
    pub scene: Box<dyn Scene>,
}

impl Context {
    pub fn new(provider: SceneTransition) -> Self {
        let mut world = Box::new(Default::default()); 
        let scene = provider(&mut world);
        Context {
            world,
            scene,
        }
    }

    pub fn register_resources(&mut self) {
        self.world.add_resource(KeyboardInput::default());
    }

    pub fn update_scene(&mut self) -> SceneResult {
        self.scene.update(&mut self.world)
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
