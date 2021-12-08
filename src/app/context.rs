use crate::{
    ecs::{Events, World},
    input::{
        KeyInput, KeyboardInput, MouseButtonInput, MouseInput, MouseMotion, MouseScrollUnit,
        MouseWheel, TouchInput, Touches,
    },
    timing::Time,
    ty::{Vec2, Vec2i},
    windowing::{
        CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, WindowBackendScaleFactorChanged,
        WindowCloseRequested, WindowFocused, WindowId, WindowMoved, WindowResized,
        WindowScaleFactorChanged, Windows,
    },
};

use super::{
    scene::SceneContext,
    systems::{Stage, Systems},
    Scene, SceneResult,
};

pub struct AppContext {
    world: World,
    systems: Systems,
    scene: Box<dyn Scene>,
}

impl AppContext {
    pub fn new(world: World, systems: Systems, scene: Box<dyn Scene>) -> Self {
        Self {
            world,
            systems,
            scene,
        }
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

    pub fn get_window_id(&self, id: winit::window::WindowId) -> Option<WindowId> {
        self.world.resource::<Windows>().get_id(id)
    }

    pub fn handle_window_close_requested(&mut self, id: WindowId) {
        self.world
            .resource_mut::<Events<WindowCloseRequested>>()
            .send(WindowCloseRequested { id });
    }

    pub fn handle_window_resized(&mut self, id: WindowId, width: u32, height: u32) {
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_physical_size(width, height);

        let logical_width = window.width();
        let logical_height = window.height();

        let mut resize_events = self.world.resource_mut::<Events<WindowResized>>();
        resize_events.send(WindowResized {
            id,
            width: logical_width,
            height: logical_height,
        });
    }

    pub fn handle_window_moved(
        &mut self,
        id: WindowId,
        position: winit::dpi::PhysicalPosition<i32>,
    ) {
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let position = Vec2i::new(position.x, position.y);
        window.update_position(Some(position));

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
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let winit_window = window.winit_window();
        let position = position.to_logical(winit_window.scale_factor());
        let inner_size = winit_window
            .inner_size()
            .to_logical::<f32>(winit_window.scale_factor());

        let y_position = inner_size.height - position.y;
        let position = Vec2::new(position.x, y_position);
        window.update_cursor_position(Some(position));

        self.world
            .resource_mut::<Events<CursorMoved>>()
            .send(CursorMoved { id, position });
    }

    pub fn handle_cursor_entered(&mut self, id: WindowId) {
        self.world
            .resource_mut::<Events<CursorEntered>>()
            .send(CursorEntered { id });
    }

    pub fn handle_cursor_left(&mut self, id: WindowId) {
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_cursor_position(None);

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

    pub fn handle_touch(&mut self, id: WindowId, touch: winit::event::Touch) {
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let winit_window = window.winit_window();
        let location = touch.location.to_logical(winit_window.scale_factor());

        // TODO
        // if cfg!(target_os = "android") || cfg!(target_os = "ios") {
        //     let window_height = windows.get_primary().unwrap().height();
        //     location.y = window_height - location.y;
        // }
        let touch_input = TouchInput::from_winit_event(touch, location);
        self.world
            .resource_mut::<Touches>()
            .process_event(&touch_input);
        self.world
            .resource_mut::<Events<TouchInput>>()
            .send(touch_input);
    }

    pub fn handle_received_character(&mut self, id: WindowId, c: char) {
        self.world
            .resource_mut::<Events<ReceivedCharacter>>()
            .send(ReceivedCharacter { id, char: c });
    }

    pub fn handle_window_focused(&mut self, id: WindowId, focused: bool) {
        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_focused(focused);

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
        self.world
            .resource_mut::<Events<WindowBackendScaleFactorChanged>>()
            .send(WindowBackendScaleFactorChanged { id, scale_factor });

        let mut windows = self.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let old_scale_factor = window.scale_factor();
        let old_physical_width = window.physical_width();
        let old_physical_height = window.physical_height();
        window.update_backend_scale_factor(scale_factor);
        window.update_physical_size(inner_size.width, inner_size.height);
        let logical_width = window.width();
        let logical_height = window.height();

        #[allow(clippy::float_cmp)]
        if old_scale_factor != scale_factor {
            self.world
                .resource_mut::<Events<WindowScaleFactorChanged>>()
                .send(WindowScaleFactorChanged { id, scale_factor });
        }

        if old_physical_width != inner_size.width || old_physical_height != inner_size.height {
            self.world
                .resource_mut::<Events<WindowResized>>()
                .send(WindowResized {
                    id,
                    width: logical_width,
                    height: logical_height,
                });
        }
    }

    fn before_scene_update(&mut self) {
        self.advance_time();
        self.systems.run(Stage::LoadAssets, &mut self.world);
        self.systems.run(Stage::PreUpdate, &mut self.world);
    }

    fn after_scene_update(&mut self) {
        // Physics, animations, ...
        // Draw

        self.systems.run(Stage::PostUpdate, &mut self.world);
        self.systems.run(Stage::AssetEvents, &mut self.world);

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
        self.world.resource_mut::<Touches>().flush();
    }
}
