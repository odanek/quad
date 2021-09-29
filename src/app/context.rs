use crate::{
    ecs::World,
    input::{KeyboardInput, MouseInput},
    time::Time,
};

use super::{event::AppEvents, scene::SceneContext, Scene, SceneResult};

pub struct AppContext {
    world: Box<World>,
    scene: Box<dyn Scene>,
    events: AppEvents,
}

impl AppContext {
    pub fn new(world: Box<World>, events: AppEvents, scene: Box<dyn Scene>) -> Self {
        Self {
            world,
            events,
            scene,
        }
    }

    pub fn insert_resources(&mut self) {
        self.world.insert_resource(KeyboardInput::default());
        self.world.insert_resource(MouseInput::default());
        self.world.insert_resource(Time::default());
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
        self.before_scene_update();

        let context = SceneContext::new(&mut self.world);
        let result = self.scene.update(context);
        if !matches!(result, SceneResult::Quit) {
            self.after_scene_update();
        }

        result
    }

    pub fn handle_window_resize(&mut self, _size: winit::dpi::PhysicalSize<u32>) {
        // if size.width != 0 || size.height != 0 {
        //     // Resized
        // } else {
        //     // Minimized
        // }
    }

    pub fn handle_keyboard_event(&mut self, input: winit::event::KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            self.world
                .resource_mut::<KeyboardInput>()
                .toggle(keycode.into(), input.state.into());
        }
    }

    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.world
            .resource_mut::<MouseInput>()
            .toggle(button.into(), state.into());
    }

    fn before_scene_update(&mut self) {
        self.advance_time();
        self.events.update(&mut self.world);
    }

    fn after_scene_update(&mut self) {
        // Physics, animations, ...
        // Draw

        self.flush_input_events();
        self.world.clear_trackers();
    }

    fn advance_time(&mut self) {
        let mut time = self.world.resource_mut::<Time>();
        time.update();
    }

    fn flush_input_events(&mut self) {
        self.world.resource_mut::<KeyboardInput>().flush();
        self.world.resource_mut::<MouseInput>().flush();
    }
}
