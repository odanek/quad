use crate::{
    ecs::{Event, Events, Resource, World},
    input::{
        KeyInput, KeyboardInput, MouseButtonInput, MouseInput, MouseMotion, MouseScrollUnit,
        MouseWheel,
    },
    time::Time,
    ty::{IVec2, Vec2},
    window::{
        CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, Window,
        WindowBackendScaleFactorChanged, WindowCloseRequested, WindowFocused, WindowId,
        WindowMoved, WindowResized, WindowScaleFactorChanged,
    },
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

    pub fn handle_window_close_requested(&mut self, id: WindowId) {
        self.world
            .resource_mut::<Events<WindowCloseRequested>>()
            .send(WindowCloseRequested { id });
    }

    pub fn handle_window_resized(&mut self, id: WindowId, width: u32, height: u32) {
        debug_assert!(id == self.main_window.id());

        let main_window = &mut self.main_window;
        main_window.update_physical_size(width, height);

        let mut resize_events = self.world.resource_mut::<Events<WindowResized>>();
        resize_events.send(WindowResized {
            id,
            width: main_window.width(),
            height: main_window.height(),
        });
    }

    pub fn handle_window_moved(
        &mut self,
        id: WindowId,
        position: winit::dpi::PhysicalPosition<i32>,
    ) {
        debug_assert!(id == self.main_window.id());

        let position = IVec2::new(position.x, position.y);
        self.main_window.update_position(Some(position));
        self.world
            .resource_mut::<Events<WindowMoved>>()
            .send(WindowMoved { id, position });
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

    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                self.world
                    .resource_mut::<Events<MouseWheel>>()
                    .send(MouseWheel {
                        unit: MouseScrollUnit::Line,
                        x,
                        y,
                    });
            }
            winit::event::MouseScrollDelta::PixelDelta(p) => {
                self.world
                    .resource_mut::<Events<MouseWheel>>()
                    .send(MouseWheel {
                        unit: MouseScrollUnit::Pixel,
                        x: p.x as f32,
                        y: p.y as f32,
                    });
            }
        }
    }

    pub fn handle_cursor_moved(
        &mut self,
        id: WindowId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) {
        debug_assert!(id == self.main_window.id());
        let winit_window = self.main_window.winit_window();
        let position = position.to_logical(winit_window.scale_factor());
        let inner_size = winit_window
            .inner_size()
            .to_logical::<f32>(winit_window.scale_factor());

        let y_position = inner_size.height - position.y;
        let position = Vec2::new(position.x, y_position);
        self.main_window.update_cursor_position(Some(position));

        self.world
            .resource_mut::<Events<CursorMoved>>()
            .send(CursorMoved { id, position });
    }

    pub fn handle_cursor_entered(&mut self, id: WindowId) {
        debug_assert!(id == self.main_window.id());
        self.world
            .resource_mut::<Events<CursorEntered>>()
            .send(CursorEntered { id });
    }

    pub fn handle_cursor_left(&mut self, id: WindowId) {
        debug_assert!(id == self.main_window.id());

        self.main_window.update_cursor_position(None);

        self.world
            .resource_mut::<Events<CursorLeft>>()
            .send(CursorLeft { id });
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.world
            .resource_mut::<Events<MouseMotion>>()
            .send(MouseMotion {
                delta: Vec2::new(delta.0 as f32, delta.1 as f32),
            });
    }

    pub fn handle_window_character(&mut self, id: WindowId, c: char) {
        self.world
            .resource_mut::<Events<ReceivedCharacter>>()
            .send(ReceivedCharacter { id, char: c });
    }

    pub fn handle_window_focused(&mut self, id: WindowId, focused: bool) {
        debug_assert!(id == self.main_window.id());
        self.main_window.update_focused(focused);

        self.world
            .resource_mut::<Events<WindowFocused>>()
            .send(WindowFocused { id, focused });
    }

    pub fn handle_scale_factor_changed(
        &mut self,
        id: WindowId,
        scale_factor: f64,
        inner_size: winit::dpi::PhysicalSize<u32>,
    ) {
        debug_assert!(id == self.main_window.id());

        let window = &mut self.main_window;
        self.world
            .resource_mut::<Events<WindowBackendScaleFactorChanged>>()
            .send(WindowBackendScaleFactorChanged { id, scale_factor });

        #[allow(clippy::float_cmp)]
        if window.scale_factor() != scale_factor {
            self.world
                .resource_mut::<Events<WindowScaleFactorChanged>>()
                .send(WindowScaleFactorChanged { id, scale_factor });
        }

        window.update_backend_scale_factor(scale_factor);

        if window.physical_width() != inner_size.width
            || window.physical_height() != inner_size.height
        {
            self.world
                .resource_mut::<Events<WindowResized>>()
                .send(WindowResized {
                    id,
                    width: window.width(),
                    height: window.height(),
                });
        }
        window.update_physical_size(inner_size.width, inner_size.height);
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
        self.add_default_event::<WindowCloseRequested>();
        self.add_default_event::<WindowResized>();
        self.add_default_event::<WindowMoved>();
        self.add_default_event::<KeyInput>();
        self.add_default_event::<MouseButtonInput>();
        self.add_default_event::<MouseWheel>();
        self.add_default_event::<CursorMoved>();
        self.add_default_event::<CursorEntered>();
        self.add_default_event::<CursorLeft>();
        self.add_default_event::<MouseMotion>();
        self.add_default_event::<ReceivedCharacter>();
        self.add_default_event::<WindowFocused>();
    }

    fn add_default_resource<T: Resource + Default>(&mut self) {
        self.world.insert_resource(T::default());
    }

    fn add_default_event<T: Event>(&mut self) {
        self.events.add::<T>(&mut self.world);
    }
}
