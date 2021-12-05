use crate::{
    asset::{free_unused_assets_system, AssetServer, AssetServerSettings, FileAssetIo},
    ecs::{Event, Events, IntoSystem, Resource, World},
    input::{
        KeyInput, KeyboardInput, MouseButtonInput, MouseInput, MouseMotion, MouseScrollUnit,
        MouseWheel, TouchInput, Touches,
    },
    tasks::{logical_core_count, IoTaskPool, TaskPoolBuilder},
    time::Time,
    ty::{Vec2, Vec2i},
    window::{
        CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, Window,
        WindowBackendScaleFactorChanged, WindowCloseRequested, WindowFocused, WindowId,
        WindowMoved, WindowResized, WindowScaleFactorChanged,
    },
};

use super::{
    scene::SceneContext,
    system::{Stage, Systems},
    Scene, SceneResult,
};

pub struct AppContext {
    world: Box<World>,
    systems: Box<Systems>,
    main_window: Window,
    scene: Box<dyn Scene>,
}

impl AppContext {
    pub fn new(
        world: Box<World>,
        systems: Box<Systems>,
        scene: Box<dyn Scene>,
        main_window: Window,
    ) -> Self {
        let mut ctx = Self {
            main_window,
            world,
            systems,
            scene,
        };

        ctx.add_standard_pools(1, usize::MAX);
        ctx.add_standard_systems();
        ctx.add_standard_resources();
        ctx.add_standard_events();

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

        let position = Vec2i::new(position.x, position.y);
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

    pub fn handle_touch(&mut self, id: WindowId, touch: winit::event::Touch) {
        debug_assert!(id == self.main_window.id());

        let winit_window = self.main_window.winit_window();
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

    fn add_standard_pools(&mut self, min_total_threads: usize, max_total_threads: usize) {
        let total_threads = logical_core_count().clamp(min_total_threads, max_total_threads);
        log::trace!("Assigning {} cores to default task pools", total_threads);

        let remaining_threads = total_threads;

        let io_threads = get_number_of_threads(remaining_threads, total_threads, 0.25, 1, 4);

        log::trace!("IO Threads: {}", io_threads);
        // remaining_threads = remaining_threads.saturating_sub(io_threads);

        self.world.insert_resource(IoTaskPool(
            TaskPoolBuilder::default()
                .num_threads(io_threads)
                .thread_name("IO Task Pool".to_string())
                .build(),
        ));
    }

    fn add_standard_systems(&mut self) {
        self.systems.add(
            Stage::PreUpdate,
            free_unused_assets_system.system(&mut self.world),
        );
    }

    fn add_standard_resources(&mut self) {
        self.add_standard_resource::<KeyboardInput>();
        self.add_standard_resource::<MouseInput>();
        self.add_standard_resource::<Touches>();
        self.add_standard_resource::<Time>();
        self.add_asset_server();
    }

    fn add_asset_server(&mut self) {
        // TODO: Make the task pool and asset server settings configurable
        let task_pool = self.world.resource::<IoTaskPool>().0.clone();
        self.world.insert_resource(AssetServerSettings::default());
        let settings = self.world.resource::<AssetServerSettings>();
        let source = Box::new(FileAssetIo::new(&settings.asset_folder));
        let asset_server = AssetServer::with_boxed_io(source, task_pool);
        self.world.insert_resource(asset_server);
    }

    fn add_standard_events(&mut self) {
        self.add_standard_event::<WindowCloseRequested>();
        self.add_standard_event::<WindowResized>();
        self.add_standard_event::<WindowMoved>();
        self.add_standard_event::<KeyInput>();
        self.add_standard_event::<MouseButtonInput>();
        self.add_standard_event::<MouseWheel>();
        self.add_standard_event::<CursorMoved>();
        self.add_standard_event::<CursorEntered>();
        self.add_standard_event::<CursorLeft>();
        self.add_standard_event::<MouseMotion>();
        self.add_standard_event::<TouchInput>();
        self.add_standard_event::<ReceivedCharacter>();
        self.add_standard_event::<WindowFocused>();
        self.add_standard_event::<WindowBackendScaleFactorChanged>();
        self.add_standard_event::<WindowScaleFactorChanged>();
    }

    fn add_standard_resource<T: Resource + Default>(&mut self) {
        self.world.insert_resource(T::default());
    }

    fn add_standard_event<T: Event>(&mut self) {
        self.systems.add_event::<T>(&mut self.world);
    }

    // fn add_standard_asset<T: Asset>(&mut self) {
    //     self.systems.add_asset::<T>(&mut self.world);
    // }
}

// TODO: Extract (Bevy TaskPoolThreadsAsignmentPolicy)
fn get_number_of_threads(
    remaining_threads: usize,
    total_threads: usize,
    percent: f32,
    min_threads: usize,
    max_threads: usize,
) -> usize {
    let mut desired = (total_threads as f32 * percent).round() as usize;
    desired = desired.min(remaining_threads);
    desired.clamp(min_threads, max_threads)
}
