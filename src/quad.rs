use crate::{
    app::{winit_runner, App, AppContext, Scene, TaskPoolOptions, Stage},
    asset::{Asset, AssetServerSettings},
    ecs::{Event, FromWorld, IntoSystem, Resource},
    windowing::{WindowBuilder, WindowId},
};

#[derive(Default)]
pub struct QuadConfig {
    pub task_pool_options: TaskPoolOptions,
    pub asset_server_settings: AssetServerSettings,
}

pub struct Quad {
    app: App,
    main_window: WindowBuilder,
}

impl Quad {
    pub fn new(config: &QuadConfig) -> Self {
        let mut quad = Self {
            app: App::default(),
            main_window: WindowBuilder::default(),
        };
        quad.add_pools(config);
        quad.add_plugins(config);
        quad
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) -> &mut Self {
        self.app.init_resource::<T>();
        self
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.app.insert_resource(resource);
        self
    }

    pub fn add_system_to_stage<S, Params>(&mut self, stage: Stage, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Params>,
    {
        self.app.add_system_to_stage(stage, system);
        self
    }

    pub fn add_event<T: Event>(&mut self) -> &mut Self {
        self.app.add_event::<T>();
        self
    }

    pub fn add_asset<T: Asset>(&mut self) -> &mut Self {
        self.app.add_asset::<T>();
        self
    }

    pub fn main_window(&mut self, window: WindowBuilder) -> &mut Self {
        self.main_window = window;
        self
    }

    pub fn run(&mut self, scene: Box<dyn Scene>) {
        let mut app = std::mem::take(&mut self.app);
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window = self.main_window.build(WindowId::new(0), &event_loop);
        app.add_window(main_window);
        let context = AppContext::new(app, scene);
        winit_runner(context, event_loop);
    }

    fn add_pools(&mut self, config: &QuadConfig) {
        self.app.create_pools(&config.task_pool_options);
    }

    fn add_plugins(&mut self, config: &QuadConfig) {
        self.app.add_timing_plugin();
        self.app.add_windowing_plugin();
        self.app.add_input_plugin();
        self.app.add_asset_plugin(&config.asset_server_settings);
    }
}
