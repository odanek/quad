use crate::{
    app::{App, MainApp},
    audio::AudioDevice,
    ecs::Events,
    input::{
        KeyInput, KeyboardInput, MouseButtonInput, MouseInput, MouseMotion, MouseScrollUnit,
        MouseWheel, TouchInput, Touches,
    },
    ty::{Vec2, Vec2i},
    windowing::{
        CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, WindowBackendScaleFactorChanged,
        WindowCloseRequested, WindowFocused, WindowId, WindowMoved, WindowResized,
        WindowScaleFactorChanged, Windows,
    },
};

use super::{Scene, SceneStage, SceneResult};

pub struct RunContext {
    app: App,
    render_app: App,
    _audio_device: AudioDevice,
    stage: SceneStage,
    scene: Vec<Box<dyn Scene>>,
}

impl RunContext {
    pub fn new(
        app: App,
        render_app: App,
        audio_device: AudioDevice,
        scene: Box<dyn Scene>,
    ) -> Self {
        Self {
            app,
            render_app,
            _audio_device: audio_device,
            stage: SceneStage::Start,
            scene: vec![scene],
        }
    }

    pub fn update(&mut self) -> bool {
        if let Some(scene) = self.scene.last_mut() {
            let result = self
                .app
                .update_main_app(&mut self.render_app, scene.as_mut(), self.stage);

            match result {
                SceneResult::Ok(stage) => {
                    self.stage = stage;
                    false
                }
                SceneResult::Push(new_scene, stage) => {
                    self.scene.push(new_scene);
                    self.stage = stage;
                    false
                }
                SceneResult::Pop(stage) => {
                    self.scene.pop();
                    self.stage = stage;
                    false
                }
                SceneResult::Replace(new_scene, stage) => {
                    self.scene.pop();
                    self.scene.push(new_scene);
                    self.stage = stage;
                    false
                }
                SceneResult::Quit => true,
            }
        } else {
            true
        }
    }

    pub fn get_window_id(&self, id: winit::window::WindowId) -> Option<WindowId> {
        self.app.world.resource::<Windows>().get_id(id)
    }

    pub fn handle_window_close_requested(&mut self, id: WindowId) {
        self.app
            .world
            .resource_mut::<Events<WindowCloseRequested>>()
            .send(WindowCloseRequested { id });
    }

    pub fn handle_window_resized(&mut self, id: WindowId, width: u32, height: u32) {
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_physical_size(width, height);

        let logical_width = window.width();
        let logical_height = window.height();

        let mut resize_events = self.app.world.resource_mut::<Events<WindowResized>>();
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
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let position = Vec2i::new(position.x, position.y);
        window.update_position(Some(position));

        self.app
            .world
            .resource_mut::<Events<WindowMoved>>()
            .send(WindowMoved { id, position });
    }

    pub fn handle_keyboard_event(&mut self, input: winit::event::KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            self.app
                .world
                .resource_mut::<KeyboardInput>()
                .toggle(keycode.into(), input.state.into());
        }
        self.app
            .world
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
        self.app
            .world
            .resource_mut::<MouseInput>()
            .toggle(button.into(), state.into());
        self.app
            .world
            .resource_mut::<Events<MouseButtonInput>>()
            .send(MouseButtonInput {
                button: button.into(),
                state: state.into(),
            });
    }

    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                self.app
                    .world
                    .resource_mut::<Events<MouseWheel>>()
                    .send(MouseWheel {
                        unit: MouseScrollUnit::Line,
                        x,
                        y,
                    });
            }
            winit::event::MouseScrollDelta::PixelDelta(p) => {
                self.app
                    .world
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
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let winit_window = window.winit_window();
        let position = position.to_logical(winit_window.scale_factor());
        let inner_size = winit_window
            .inner_size()
            .to_logical::<f32>(winit_window.scale_factor());

        let y_position = inner_size.height - position.y;
        let position = Vec2::new(position.x, y_position);
        window.update_cursor_position(Some(position));

        self.app
            .world
            .resource_mut::<Events<CursorMoved>>()
            .send(CursorMoved { id, position });
    }

    pub fn handle_cursor_entered(&mut self, id: WindowId) {
        self.app
            .world
            .resource_mut::<Events<CursorEntered>>()
            .send(CursorEntered { id });
    }

    pub fn handle_cursor_left(&mut self, id: WindowId) {
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_cursor_position(None);

        self.app
            .world
            .resource_mut::<Events<CursorLeft>>()
            .send(CursorLeft { id });
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.app
            .world
            .resource_mut::<Events<MouseMotion>>()
            .send(MouseMotion {
                delta: Vec2::new(delta.0 as f32, delta.1 as f32),
            });
    }

    pub fn handle_touch(&mut self, id: WindowId, touch: winit::event::Touch) {
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();

        let winit_window = window.winit_window();
        let location = touch.location.to_logical(winit_window.scale_factor());

        // TODO
        // if cfg!(target_os = "android") || cfg!(target_os = "ios") {
        //     let window_height = windows.get_primary().unwrap().height();
        //     location.y = window_height - location.y;
        // }
        let touch_input = TouchInput::from_winit_event(touch, location);
        self.app
            .world
            .resource_mut::<Touches>()
            .process_event(&touch_input);
        self.app
            .world
            .resource_mut::<Events<TouchInput>>()
            .send(touch_input);
    }

    pub fn handle_received_character(&mut self, id: WindowId, c: char) {
        self.app
            .world
            .resource_mut::<Events<ReceivedCharacter>>()
            .send(ReceivedCharacter { id, char: c });
    }

    pub fn handle_window_focused(&mut self, id: WindowId, focused: bool) {
        let mut windows = self.app.world.resource_mut::<Windows>();
        let window = windows.get_mut(id).unwrap();
        window.update_focused(focused);

        self.app
            .world
            .resource_mut::<Events<WindowFocused>>()
            .send(WindowFocused { id, focused });
    }

    pub fn handle_scale_factor_changed(
        &mut self,
        id: WindowId,
        scale_factor: f64,
        inner_size: winit::dpi::PhysicalSize<u32>,
    ) {
        self.app
            .world
            .resource_mut::<Events<WindowBackendScaleFactorChanged>>()
            .send(WindowBackendScaleFactorChanged { id, scale_factor });

        let mut windows = self.app.world.resource_mut::<Windows>();
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
            self.app
                .world
                .resource_mut::<Events<WindowScaleFactorChanged>>()
                .send(WindowScaleFactorChanged { id, scale_factor });
        }

        if old_physical_width != inner_size.width || old_physical_height != inner_size.height {
            self.app
                .world
                .resource_mut::<Events<WindowResized>>()
                .send(WindowResized {
                    id,
                    width: logical_width,
                    height: logical_height,
                });
        }
    }
}
