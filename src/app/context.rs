use crate::{
    ecs::{Event, Events, Resource, World},
    input::{CursorEntered, CursorLeft, KeyInput, KeyboardInput, MouseButtonInput, MouseInput},
    time::Time,
    window::{event::WindowResized, Window, WindowId},
};

use super::{event::AppEvents, scene::SceneContext, Scene, SceneResult};

pub struct AppContext {
    world: Box<World>,
    events: Box<AppEvents>,
    main_window: Window,
    scene: Box<dyn Scene>,
}

impl AppContext {
    pub fn new(
        world: Box<World>,
        events: Box<AppEvents>,
        scene: Box<dyn Scene>,
        main_window: Window,
    ) -> Self {
        let mut ctx = Self {
            main_window,
            world,
            events,
            scene,
        };

        ctx.add_default_resources();
        ctx.add_default_events();

        ctx
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

    pub fn get_window(&self, id: winit::window::WindowId) -> Option<&Window> {
        if self.main_window.winit_id() == id {
            Some(&self.main_window)
        } else {
            None
        }
    }

    pub fn handle_window_resize(&mut self, id: WindowId, width: u32, height: u32) {
        debug_assert!(id == self.main_window.id());

        let main_window = &mut self.main_window;
        main_window.update_physical_size(width, height);

        let mut resize_events = self.world.resource_mut::<Events<WindowResized>>();
        resize_events.send(WindowResized {
            id,
            width: main_window.width(),
            height: main_window.height(),
        });

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
        self.world
            .resource_mut::<Events<KeyInput>>()
            .send(KeyInput {
                scan_code: input.scancode,
                state: input.state.into(),
                key_code: input.virtual_keycode.map(Into::into),
            });
    }

    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.world
            .resource_mut::<MouseInput>()
            .toggle(button.into(), state.into());
        self.world
            .resource_mut::<Events<MouseButtonInput>>()
            .send(MouseButtonInput {
                button: button.into(),
                state: state.into(),
            });
    }

    pub fn handle_cursor_enter(&mut self, id: WindowId) {
        debug_assert!(id == self.main_window.id());
        self.world
            .resource_mut::<Events<CursorEntered>>()
            .send(CursorEntered { id });
    }

    pub fn handle_cursor_leave(&mut self, id: WindowId) {
        debug_assert!(id == self.main_window.id());

        self.main_window.update_cursor_position(None);

        self.world
            .resource_mut::<Events<CursorLeft>>()
            .send(CursorLeft { id });
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

    fn add_default_resources(&mut self) {
        self.add_default_resource::<KeyboardInput>();
        self.add_default_resource::<MouseInput>();
        self.add_default_resource::<Time>();
    }

    fn add_default_events(&mut self) {
        self.add_default_event::<WindowResized>();
        self.add_default_event::<KeyInput>();
        self.add_default_event::<MouseButtonInput>();
        self.add_default_event::<CursorEntered>();
        self.add_default_event::<CursorLeft>();
    }

    fn add_default_resource<T: Resource + Default>(&mut self) {
        self.world.insert_resource(T::default());
    }

    fn add_default_event<T: Event>(&mut self) {
        self.events.add::<T>(&mut self.world);
    }
}
