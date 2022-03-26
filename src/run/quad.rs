use winit::event_loop::EventLoop;

use crate::{
    app::{App, Stage, TaskPoolOptions},
    asset::{Asset, AssetServerSettings},
    ecs::{Event, FromWorld, IntoSystem, Resource},
    windowing::{WindowBuilder, WindowId},
};

use super::{context::RunContext, runner::winit_runner, Scene};

#[derive(Default)]
pub struct QuadConfig {
    pub task_pool_options: TaskPoolOptions,
    pub asset_server_settings: AssetServerSettings,
    pub main_window: WindowBuilder,
}

pub struct Quad {
    app: App,
    render_app: App,
    event_loop: Option<EventLoop<()>>,
}

impl Quad {
    pub fn new(config: &QuadConfig) -> Self {
        let mut quad = Self {
            app: App::default(),
            render_app: App::default(),
            event_loop: Some(EventLoop::new()),
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

    pub fn run(&mut self, scene: Box<dyn Scene>) {
        let app = std::mem::take(&mut self.app);
        let render_app = std::mem::take(&mut self.render_app);
        let event_loop = self.event_loop.take().unwrap();
        let context = RunContext::new(app, render_app, scene);
        winit_runner(context, event_loop);
    }

    fn add_pools(&mut self, config: &QuadConfig) {
        self.app.create_pools(&config.task_pool_options);
    }

    // TODO Wrap all of this in the MainApp and RenderApp traits
    fn add_plugins(&mut self, config: &QuadConfig) {
        let app = &mut self.app;
        app.add_windowing_plugin();
        let main_window = config
            .main_window
            .build(WindowId::primary(), self.event_loop.as_ref().unwrap());
        app.add_window(main_window);

        app.add_timing_plugin();
        app.add_input_plugin();
        app.add_asset_plugin(&config.asset_server_settings);
        app.add_transform_plugin();
        app.add_render_plugin(&mut self.render_app);
    }
}
